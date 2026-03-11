mod assets;
mod audit;
mod backup;
mod cache;
mod credentials;
mod db;
mod designs;
mod ffmpeg;
mod platform;
mod rate_limiter;
mod retry;
mod server;
mod settings;
mod types;
mod user_error;

use std::sync::Arc;

use db::Database;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to resolve app data directory");

            let database =
                Arc::new(Database::initialize(&app_data_dir).expect(
                    "Failed to initialize database — cannot start without a working database",
                ));

            audit::prune_old_entries(&database);

            let asset_root = assets::resolve_asset_root(&database, &app_data_dir)
                .expect("Failed to resolve asset directory");
            assets::ensure_directories(&asset_root).expect("Failed to create asset directories");

            let cred_manager = credentials::CredentialManager::initialize(Arc::clone(&database));
            app.manage(cred_manager);

            let cache_service = Arc::new(cache::CacheService::new(Arc::clone(&database)));
            cache_service.start_cleanup_task();
            app.manage(cache_service);

            app.manage(database);

            let scheduler =
                backup::BackupScheduler::start(app.handle().clone(), app_data_dir.clone());
            app.manage(scheduler);

            // Create auth states before HttpServer (server needs them for callback routes)
            let twitch_auth_state = Arc::new(platform::twitch::oauth::TwitchAuthState::new());
            app.manage(twitch_auth_state.clone());

            let youtube_auth_state = Arc::new(platform::youtube::oauth::YouTubeAuthState::new());
            app.manage(youtube_auth_state.clone());

            let kick_auth_state = Arc::new(platform::kick::oauth::KickAuthState::new());
            app.manage(kick_auth_state.clone());

            let rate_limiter = Arc::new(rate_limiter::RateLimiterService::new());
            rate_limiter.start_refill_task();
            app.manage(rate_limiter);

            let retry_service = Arc::new(retry::RetryService::new());
            app.manage(retry_service);

            let http_server = server::HttpServer::start(
                app.handle().clone(),
                twitch_auth_state,
                youtube_auth_state,
                kick_auth_state,
            )
            .expect("Failed to start embedded HTTP server");
            app.manage(http_server);

            let socket_io_server = server::SocketIoServer::start(app.handle())
                .expect("Failed to start Socket.IO sidecar");

            let ffmpeg_queue = Arc::new(ffmpeg::FfmpegQueue::new(1, socket_io_server.port()));
            app.manage(ffmpeg_queue);

            app.manage(socket_io_server);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            settings::commands::get_config_section,
            settings::commands::set_config_section,
            settings::commands::get_full_config,
            backup::commands::create_backup,
            backup::commands::list_backups,
            backup::commands::restore_backup,
            backup::commands::delete_backup,
            server::commands::get_server_info,
            server::commands::get_socket_io_info,
            credentials::commands::store_credential,
            credentials::commands::get_credential,
            credentials::commands::delete_credential,
            credentials::commands::has_credential,
            credentials::commands::get_credential_backend,
            credentials::commands::store_platform_tokens,
            credentials::commands::get_platform_tokens,
            platform::commands::get_platform_connections,
            platform::commands::get_platform_connection,
            platform::commands::disconnect_platform,
            platform::twitch::commands::start_twitch_auth,
            platform::twitch::commands::refresh_twitch_tokens,
            platform::twitch::commands::revoke_twitch_auth,
            platform::youtube::commands::start_youtube_auth,
            platform::youtube::commands::refresh_youtube_tokens,
            platform::youtube::commands::revoke_youtube_auth,
            platform::kick::commands::start_kick_auth,
            platform::kick::commands::refresh_kick_tokens,
            platform::kick::commands::revoke_kick_auth,
            rate_limiter::commands::get_rate_limit_status,
            retry::commands::get_platform_health,
            retry::commands::get_all_platform_health,
            retry::commands::get_action_queue_stats,
            retry::commands::drain_action_queue,
            cache::commands::cache_get,
            cache::commands::cache_invalidate,
            cache::commands::cache_stats,
            ffmpeg::commands::ffmpeg_submit_job,
            ffmpeg::commands::ffmpeg_get_job,
            ffmpeg::commands::ffmpeg_list_jobs,
            ffmpeg::commands::ffmpeg_cancel_job,
            ffmpeg::commands::ffprobe_media_info,
            assets::commands::get_asset_root,
            assets::commands::ensure_asset_directories,
            assets::commands::import_asset,
            assets::commands::list_assets,
            assets::commands::get_asset_file_path,
            assets::commands::check_asset_references,
            assets::commands::delete_asset,
            assets::commands::delete_assets_batch,
            designs::commands::create_design,
            designs::commands::get_design,
            designs::commands::list_designs,
            designs::commands::update_design,
            designs::commands::delete_design,
            designs::commands::duplicate_design,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
