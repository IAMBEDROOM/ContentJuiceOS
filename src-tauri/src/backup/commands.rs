use tauri::{AppHandle, Manager, State};

use super::engine::{self, BackupInfo};
use crate::db::Database;

fn backup_dir(app_handle: &AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e: tauri::Error| e.to_string())?;
    Ok(dir.join("backups"))
}

fn max_backups_setting(conn: &rusqlite::Connection) -> u32 {
    conn.query_row(
        "SELECT value FROM settings WHERE key = 'general.maxBackups'",
        [],
        |row| row.get::<_, String>(0),
    )
    .ok()
    .and_then(|v| v.parse().ok())
    .unwrap_or(7)
}

#[tauri::command]
pub fn create_backup(
    database: State<'_, Database>,
    app_handle: AppHandle,
) -> Result<BackupInfo, String> {
    let backup_dir = backup_dir(&app_handle)?;
    let conn = database.conn.lock().map_err(|e| e.to_string())?;
    let info = engine::create_backup(&conn, &backup_dir).map_err(|e| e.to_string())?;
    let max = max_backups_setting(&conn);
    let _ = engine::cleanup_old_backups(&backup_dir, max);
    Ok(info)
}

#[tauri::command]
pub fn list_backups(app_handle: AppHandle) -> Result<Vec<BackupInfo>, String> {
    let backup_dir = backup_dir(&app_handle)?;
    engine::list_backups(&backup_dir).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn restore_backup(
    filename: String,
    database: State<'_, Database>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let backup_dir = backup_dir(&app_handle)?;
    let mut conn = database.conn.lock().map_err(|e| e.to_string())?;

    // Create safety backup before restoring
    engine::create_prerestore_backup(&conn, &backup_dir).map_err(|e| e.to_string())?;

    engine::restore_backup(&mut conn, &backup_dir, &filename).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_backup(filename: String, app_handle: AppHandle) -> Result<(), String> {
    let backup_dir = backup_dir(&app_handle)?;
    engine::delete_backup(&backup_dir, &filename).map_err(|e| e.to_string())
}
