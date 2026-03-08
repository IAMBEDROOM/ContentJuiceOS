mod backup;
mod credentials;
mod db;
mod platform;
mod server;
mod settings;

use std::sync::Arc;

use db::Database;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to resolve app data directory");

            let database = Arc::new(
                Database::initialize(&app_data_dir)
                    .expect("Failed to initialize database — cannot start without a working database"),
            );

            let cred_manager =
                credentials::CredentialManager::initialize(Arc::clone(&database));
            app.manage(cred_manager);

            app.manage(database);

            let scheduler =
                backup::BackupScheduler::start(app.handle().clone(), app_data_dir.clone());
            app.manage(scheduler);

            // Create TwitchAuthState before HttpServer (server needs it for callback route)
            let twitch_auth_state =
                Arc::new(platform::twitch::oauth::TwitchAuthState::new());
            app.manage(twitch_auth_state.clone());

            let http_server =
                server::HttpServer::start(app.handle().clone(), twitch_auth_state)
                    .expect("Failed to start embedded HTTP server");
            app.manage(http_server);

            let socket_io_server = server::SocketIoServer::start(app.handle())
                .expect("Failed to start Socket.IO sidecar");
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
