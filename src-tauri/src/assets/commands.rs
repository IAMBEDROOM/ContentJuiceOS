use std::sync::Arc;

use tauri::{AppHandle, Manager, State};

use crate::db::Database;
use crate::user_error::UserFacingError;

use super::storage;

#[tauri::command]
pub fn get_asset_root(
    app_handle: AppHandle,
    database: State<'_, Arc<Database>>,
) -> Result<String, String> {
    let app_data_dir = app_handle.path().app_data_dir().map_user_err()?;
    let root = storage::resolve_asset_root(&database, &app_data_dir).map_user_err()?;
    Ok(root.display().to_string())
}

#[tauri::command]
pub fn ensure_asset_directories(
    app_handle: AppHandle,
    database: State<'_, Arc<Database>>,
) -> Result<String, String> {
    let app_data_dir = app_handle.path().app_data_dir().map_user_err()?;
    let root = storage::resolve_asset_root(&database, &app_data_dir).map_user_err()?;
    storage::ensure_directories(&root).map_user_err()?;
    Ok(root.display().to_string())
}
