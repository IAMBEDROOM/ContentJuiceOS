use std::path::Path;

use chrono::Utc;
use tauri::AppHandle;

use crate::db::Database;
use crate::ffmpeg::probe::probe_media;

use super::error::AssetError;
use super::repository;
use super::storage;
use super::types::{Asset, AssetReference, AssetType, DeleteAssetsResponse, DeleteFailure};
use super::validation;

/// Imports a file into the asset library.
///
/// Validates format and size, copies to the managed directory, extracts metadata
/// via ffprobe, inserts a database record, and returns the completed `Asset`.
pub async fn import_asset_from_path(
    database: &Database,
    app_handle: &AppHandle,
    source_path: &Path,
) -> Result<Asset, AssetError> {
    // 1. Validate source file exists
    if !source_path.exists() || !source_path.is_file() {
        return Err(AssetError::SourceNotFound(
            source_path.display().to_string(),
        ));
    }

    // 2. Extract extension and auto-detect asset type
    let ext = source_path
        .extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| AssetError::InvalidFilename("File has no extension".to_string()))?
        .to_lowercase();

    let asset_type = validation::detect_asset_type(&ext).ok_or_else(|| {
        AssetError::FormatNotSupported(format!(".{ext} is not a recognized format"))
    })?;

    // 3. Validate format against the allowlist for this type
    validation::validate_format(asset_type, &ext)?;

    // 4. Validate file size (fail-fast before copy)
    let source_size = std::fs::metadata(source_path)?.len();
    validation::validate_size(asset_type, source_size)?;

    // 5. Resolve asset root directory
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AssetError::SettingsError(format!("Failed to resolve app data dir: {e}")))?;
    let asset_root = storage::resolve_asset_root(database, &app_data_dir)?;

    // Ensure subdirectories exist
    storage::ensure_directories(&asset_root)?;

    // 6. Copy file to managed directory
    let imported = storage::import_file(&asset_root, source_path, asset_type)?;

    // 7. Extract metadata via ffprobe for applicable types
    let (width, height, duration) =
        extract_metadata(app_handle, asset_type, &imported.absolute_path).await;

    // 8. Build Asset struct
    let asset = Asset {
        id: imported.id,
        original_filename: imported.original_filename,
        asset_type,
        format: imported.format,
        file_size: imported.file_size as i64,
        width,
        height,
        duration,
        tags: vec![],
        file_path: imported.relative_path,
        created_at: Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };

    // 9. Insert into database
    {
        let conn = database
            .conn
            .lock()
            .map_err(|e| AssetError::Database(format!("Failed to lock database: {e}")))?;
        repository::insert_asset(&conn, &asset)?;
    }

    // 10. Return completed asset
    Ok(asset)
}

/// Checks what other entities reference a given asset.
pub fn check_asset_references(
    database: &Database,
    asset_id: &str,
) -> Result<Vec<AssetReference>, AssetError> {
    let conn = database
        .conn
        .lock()
        .map_err(|e| AssetError::Database(format!("Failed to lock database: {e}")))?;
    repository::find_asset_references(&conn, asset_id)
}

/// Deletes a single asset (DB record + file on disk).
///
/// Reference checking logic:
/// - If the asset is a project's source video (hard FK) → always blocked (`DeleteBlocked`)
/// - If other references exist and `force == false` → returns `AssetInUse`
/// - If other references exist and `force == true` → deletes anyway (broken refs accepted)
pub fn delete_asset(
    database: &Database,
    app_handle: &AppHandle,
    asset_id: &str,
    force: bool,
) -> Result<(), AssetError> {
    // Resolve asset root first (this locks DB internally, then releases)
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AssetError::SettingsError(format!("Failed to resolve app data dir: {e}")))?;
    let asset_root = storage::resolve_asset_root(database, &app_data_dir)?;

    let conn = database
        .conn
        .lock()
        .map_err(|e| AssetError::Database(format!("Failed to lock database: {e}")))?;

    // 1. Verify asset exists and get its file_path
    let asset = repository::get_asset_by_id(&conn, asset_id)?
        .ok_or_else(|| AssetError::NotFound(asset_id.to_string()))?;

    // 2. Check references
    let refs = repository::find_asset_references(&conn, asset_id)?;

    if !refs.is_empty() {
        // Check if it's a project source video (hard FK)
        let is_source_video: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM projects WHERE source_video_asset_id = ?1",
                rusqlite::params![asset_id],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|e| AssetError::Database(e.to_string()))?
            > 0;

        if is_source_video {
            return Err(AssetError::DeleteBlocked(format!(
                "Asset {asset_id} is used as a project source video. Delete the project first."
            )));
        }

        if !force {
            return Err(AssetError::AssetInUse {
                asset_id: asset_id.to_string(),
                references: refs,
            });
        }
    }

    // 3. Delete DB record
    repository::delete_asset(&conn, asset_id)?;

    // 4. Delete file from disk (drop conn first since delete_file doesn't need it)
    drop(conn);
    storage::delete_file(&asset_root, &asset.file_path)?;

    Ok(())
}

/// Deletes multiple assets with reference checking and file cleanup.
///
/// For each asset:
/// - Project source video references → added to `failed` list (always blocked)
/// - Other references + `force == false` → returns `AssetInUse` for the whole batch
/// - Otherwise → deleted from DB and disk
pub fn delete_assets_batch(
    database: &Database,
    app_handle: &AppHandle,
    asset_ids: &[String],
    force: bool,
) -> Result<DeleteAssetsResponse, AssetError> {
    // Pre-compute asset root
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AssetError::SettingsError(format!("Failed to resolve app data dir: {e}")))?;
    let asset_root = storage::resolve_asset_root(database, &app_data_dir)?;

    let conn = database
        .conn
        .lock()
        .map_err(|e| AssetError::Database(format!("Failed to lock database: {e}")))?;

    let mut deleted_count: u64 = 0;
    let mut failed: Vec<DeleteFailure> = Vec::new();
    let mut all_soft_refs: Vec<AssetReference> = Vec::new();

    // First pass: categorize each asset
    let mut deletable: Vec<(String, String)> = Vec::new(); // (id, file_path)

    for id in asset_ids {
        // Check if asset exists
        let asset = match repository::get_asset_by_id(&conn, id)? {
            Some(a) => a,
            None => {
                failed.push(DeleteFailure {
                    asset_id: id.clone(),
                    reason: "Asset not found".to_string(),
                });
                continue;
            }
        };

        let refs = repository::find_asset_references(&conn, id)?;

        // Check for hard FK (project source video)
        let is_source_video: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM projects WHERE source_video_asset_id = ?1",
                rusqlite::params![id],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|e| AssetError::Database(e.to_string()))?
            > 0;

        if is_source_video {
            failed.push(DeleteFailure {
                asset_id: id.clone(),
                reason: "Asset is a project source video. Delete the project first.".to_string(),
            });
            continue;
        }

        if !refs.is_empty() && !force {
            all_soft_refs.extend(refs);
            // Don't fail individual assets — we'll return AssetInUse for the whole batch
            continue;
        }

        deletable.push((id.clone(), asset.file_path.clone()));
    }

    // If we have soft refs and force is false, return AssetInUse
    if !all_soft_refs.is_empty() && !force {
        return Err(AssetError::AssetInUse {
            asset_id: "batch".to_string(),
            references: all_soft_refs,
        });
    }

    // Second pass: delete all deletable assets
    for (id, file_path) in &deletable {
        match repository::delete_asset(&conn, id) {
            Ok(()) => {
                deleted_count += 1;
                // Best-effort file deletion (don't fail the batch if file is missing)
                if let Err(e) = storage::delete_file(&asset_root, file_path) {
                    log::warn!("Failed to delete file for asset {id}: {e}");
                }
            }
            Err(AssetError::DeleteBlocked(msg)) => {
                failed.push(DeleteFailure {
                    asset_id: id.clone(),
                    reason: msg,
                });
            }
            Err(e) => return Err(e),
        }
    }

    Ok(DeleteAssetsResponse {
        deleted_count,
        failed,
    })
}

/// Extracts width, height, and duration from a media file using ffprobe.
/// Returns (None, None, None) for types where metadata extraction doesn't apply (Font, Caption).
/// Logs warnings on failure rather than failing the entire import.
async fn extract_metadata(
    app_handle: &AppHandle,
    asset_type: AssetType,
    file_path: &str,
) -> (Option<i32>, Option<i32>, Option<f64>) {
    match asset_type {
        AssetType::Font | AssetType::Caption => (None, None, None),
        _ => {
            match probe_media(app_handle, file_path).await {
                Ok(info) => {
                    let mut width = None;
                    let mut height = None;

                    // Find the first video stream for dimensions
                    for stream in &info.streams {
                        if stream.codec_type == "video" {
                            width = stream.width.map(|v| v as i32);
                            height = stream.height.map(|v| v as i32);
                            break;
                        }
                    }

                    let duration = info.duration_ms.map(|ms| ms as f64 / 1000.0);

                    (width, height, duration)
                }
                Err(e) => {
                    log::warn!("Metadata extraction failed for {file_path}: {e}");
                    (None, None, None)
                }
            }
        }
    }
}

use tauri::Manager;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_and_validate_round_trip() {
        // Verify the validation module works end-to-end for common types
        let cases = vec![
            ("png", AssetType::Image),
            ("mp3", AssetType::Audio),
            ("mp4", AssetType::Video),
            ("ttf", AssetType::Font),
            ("json", AssetType::Animation),
            ("srt", AssetType::Caption),
        ];

        for (ext, expected_type) in cases {
            let detected = validation::detect_asset_type(ext).unwrap();
            assert_eq!(detected, expected_type, "ext: {ext}");
            assert!(
                validation::validate_format(detected, ext).is_ok(),
                "format validation failed for {ext}"
            );
        }
    }

    #[test]
    fn unsupported_format_rejected() {
        assert!(validation::detect_asset_type("exe").is_none());
        assert!(validation::detect_asset_type("dll").is_none());
    }

    #[test]
    fn oversized_file_rejected() {
        let result = validation::validate_size(AssetType::Image, 100 * 1024 * 1024);
        assert!(matches!(result, Err(AssetError::FileTooLarge { .. })));
    }
}
