mod db;
mod settings;

use db::Database;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to resolve app data directory");

            let database = Database::initialize(&app_data_dir)
                .expect("Failed to initialize database — cannot start without a working database");

            app.manage(database);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            settings::commands::get_config_section,
            settings::commands::set_config_section,
            settings::commands::get_full_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
