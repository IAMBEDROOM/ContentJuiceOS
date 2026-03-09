use serde_json::Value;
use tauri::State;

use std::sync::Arc;

use crate::db::Database;
use crate::user_error::UserFacingError;

use super::repository;

#[tauri::command]
pub fn get_config_section(
    section: String,
    database: State<'_, Arc<Database>>,
) -> Result<Value, String> {
    let conn = database.conn.lock().map_user_err()?;
    repository::get_section(&conn, &section).map_user_err()
}

#[tauri::command]
pub fn set_config_section(
    section: String,
    data: Value,
    database: State<'_, Arc<Database>>,
) -> Result<(), String> {
    let conn = database.conn.lock().map_user_err()?;
    repository::set_section(&conn, &section, &data).map_user_err()
}

#[tauri::command]
pub fn get_full_config(database: State<'_, Arc<Database>>) -> Result<Value, String> {
    let conn = database.conn.lock().map_user_err()?;
    repository::get_full_config(&conn).map_user_err()
}
