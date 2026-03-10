use std::path::Path;
use std::sync::Arc;

use tauri::{AppHandle, Manager, State};

use crate::db::Database;
use crate::user_error::UserFacingError;

use super::repository;
use super::service;
use super::storage;
use super::types::{Asset, AssetListResponse, AssetReference, AssetType, DeleteAssetsResponse};

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

#[tauri::command]
pub async fn import_asset(
    source_path: String,
    app_handle: AppHandle,
    database: State<'_, Arc<Database>>,
) -> Result<Asset, String> {
    service::import_asset_from_path(&database, &app_handle, Path::new(&source_path))
        .await
        .map_user_err()
}

#[tauri::command]
pub fn list_assets(
    type_filter: Option<String>,
    search: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
    database: State<'_, Arc<Database>>,
) -> Result<AssetListResponse, String> {
    let asset_type = type_filter.as_deref().map(|s| match s {
        "image" => AssetType::Image,
        "audio" => AssetType::Audio,
        "video" => AssetType::Video,
        "font" => AssetType::Font,
        "animation" => AssetType::Animation,
        "caption" => AssetType::Caption,
        _ => AssetType::Image,
    });

    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);

    let conn = database
        .conn
        .lock()
        .map_err(|e| format!("Failed to lock database: {e}"))?;

    let assets =
        repository::list_assets(&conn, asset_type.as_ref(), search.as_deref(), limit, offset)
            .map_user_err()?;

    let total =
        repository::count_assets(&conn, asset_type.as_ref(), search.as_deref()).map_user_err()?;

    Ok(AssetListResponse { assets, total })
}

#[tauri::command]
pub fn get_asset_file_path(
    id: String,
    app_handle: AppHandle,
    database: State<'_, Arc<Database>>,
) -> Result<String, String> {
    let conn = database
        .conn
        .lock()
        .map_err(|e| format!("Failed to lock database: {e}"))?;

    let asset = repository::get_asset_by_id(&conn, &id)
        .map_user_err()?
        .ok_or_else(|| format!("Asset not found: {id}"))?;

    let app_data_dir = app_handle.path().app_data_dir().map_user_err()?;
    let root = storage::resolve_asset_root(&database, &app_data_dir).map_user_err()?;

    let absolute = root.join(&asset.file_path);
    Ok(absolute.display().to_string())
}

#[tauri::command]
pub fn check_asset_references(
    id: String,
    database: State<'_, Arc<Database>>,
) -> Result<Vec<AssetReference>, String> {
    service::check_asset_references(&database, &id).map_user_err()
}

#[tauri::command]
pub fn delete_asset(
    id: String,
    force: bool,
    app_handle: AppHandle,
    database: State<'_, Arc<Database>>,
) -> Result<(), String> {
    service::delete_asset(&database, &app_handle, &id, force).map_user_err()
}

#[tauri::command]
pub fn delete_assets_batch(
    ids: Vec<String>,
    force: bool,
    app_handle: AppHandle,
    database: State<'_, Arc<Database>>,
) -> Result<DeleteAssetsResponse, String> {
    service::delete_assets_batch(&database, &app_handle, &ids, force).map_user_err()
}
