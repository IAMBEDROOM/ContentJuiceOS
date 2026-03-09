use serde_json::Value;
use tauri::State;

use std::sync::Arc;

use crate::db::Database;

use super::repository;

#[tauri::command]
pub fn get_config_section(
    section: String,
    database: State<'_, Arc<Database>>,
) -> Result<Value, String> {
    let conn = database.conn.lock().map_err(|e| e.to_string())?;
    repository::get_section(&conn, &section).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_config_section(
    section: String,
    data: Value,
    database: State<'_, Arc<Database>>,
) -> Result<(), String> {
    let conn = database.conn.lock().map_err(|e| e.to_string())?;
    repository::set_section(&conn, &section, &data).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_full_config(database: State<'_, Arc<Database>>) -> Result<Value, String> {
    let conn = database.conn.lock().map_err(|e| e.to_string())?;
    repository::get_full_config(&conn).map_err(|e| e.to_string())
}
