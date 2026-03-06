mod backup;
mod db;
mod server;
mod settings;

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

            let database = Database::initialize(&app_data_dir)
                .expect("Failed to initialize database — cannot start without a working database");

            app.manage(database);

            let scheduler =
                backup::BackupScheduler::start(app.handle().clone(), app_data_dir.clone());
            app.manage(scheduler);

            let http_server = server::HttpServer::start(app.handle().clone())
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
