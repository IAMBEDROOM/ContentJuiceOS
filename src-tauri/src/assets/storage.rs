use std::fs;
use std::path::{Path, PathBuf};

use uuid::Uuid;

use crate::db::Database;

use super::error::AssetError;
use super::types::{AssetType, ImportedFile, ALL_SUBDIRECTORIES};

/// Resolves the asset root directory from settings or falls back to `{app_data_dir}/assets/`.
///
/// Reads the `general.mediaDirectory` setting from the database. If empty or unset,
/// defaults to the `assets` subdirectory of the app data dir. Custom paths must be absolute.
pub fn resolve_asset_root(database: &Database, app_data_dir: &Path) -> Result<PathBuf, AssetError> {
    let conn = database
        .conn
        .lock()
        .map_err(|e| AssetError::SettingsError(format!("Failed to lock database: {e}")))?;

    let media_dir: String = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'general.mediaDirectory'",
            [],
            |row| row.get(0),
        )
        .unwrap_or_default();

    if media_dir.is_empty() {
        return Ok(app_data_dir.join("assets"));
    }

    let path = PathBuf::from(&media_dir);
    if !path.is_absolute() {
        return Err(AssetError::InvalidRoot(format!(
            "Media directory must be an absolute path, got: {media_dir}"
        )));
    }

    Ok(path)
}

/// Creates all asset subdirectories under the given root. Idempotent.
pub fn ensure_directories(asset_root: &Path) -> Result<(), AssetError> {
    for subdir in ALL_SUBDIRECTORIES {
        fs::create_dir_all(asset_root.join(subdir))?;
    }
    Ok(())
}

#[allow(dead_code)]
/// Imports a file by copying it into the appropriate asset subdirectory.
///
/// The destination filename is `{uuid}_{sanitized_stem}.{ext}`, ensuring
/// uniqueness while preserving a human-readable name on disk.
pub fn import_file(
    asset_root: &Path,
    source_path: &Path,
    asset_type: AssetType,
) -> Result<ImportedFile, AssetError> {
    // Validate source exists and is a file
    if !source_path.exists() || !source_path.is_file() {
        return Err(AssetError::SourceNotFound(
            source_path.display().to_string(),
        ));
    }

    // Extract original filename components
    let original_filename = source_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| AssetError::InvalidFilename("Could not read filename".to_string()))?
        .to_string();

    let ext = source_path
        .extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| AssetError::InvalidFilename("File has no extension".to_string()))?
        .to_lowercase();

    let stem = source_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file");

    // Build destination path
    let id = Uuid::new_v4().to_string();
    let sanitized = sanitize_filename(stem);
    let dest_filename = format!("{id}_{sanitized}.{ext}");
    let subdir = asset_type.subdirectory();
    let dest_path = asset_root.join(subdir).join(&dest_filename);

    // Copy file
    fs::copy(source_path, &dest_path)?;

    let file_size = fs::metadata(&dest_path)?.len();

    // Use forward slashes for the relative path (cross-platform consistency)
    let relative_path = format!("{subdir}/{dest_filename}");

    Ok(ImportedFile {
        id,
        relative_path,
        absolute_path: dest_path.display().to_string(),
        file_size,
        original_filename,
        format: ext,
    })
}

/// Sanitizes a filename stem by replacing dangerous characters with underscores.
fn sanitize_filename(stem: &str) -> String {
    let result: String = stem
        .chars()
        .map(|c| match c {
            '/' | '\\' | '<' | '>' | ':' | '"' | '|' | '?' | '*' | '\0' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect();

    let trimmed = result.trim().trim_matches('.').to_string();

    if trimmed.is_empty() {
        "file".to_string()
    } else {
        trimmed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migration::run_migrations;
    use crate::settings::defaults::seed_defaults_if_empty;
    use rusqlite::Connection;
    use std::sync::Mutex;

    /// Helper to create an in-memory Database with migrations and default settings.
    fn setup_test_db() -> Database {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        seed_defaults_if_empty(&conn).unwrap();
        Database {
            conn: Mutex::new(conn),
        }
    }

    /// Helper to create a temp directory with a unique name for test isolation.
    fn temp_test_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("contentjuiceos_test_{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn resolve_asset_root_default() {
        let db = setup_test_db();
        let app_data = PathBuf::from("/fake/app_data");
        let root = resolve_asset_root(&db, &app_data).unwrap();
        assert_eq!(root, app_data.join("assets"));
    }

    #[test]
    fn resolve_asset_root_custom() {
        let db = setup_test_db();
        let custom_path = if cfg!(windows) {
            "C:\\Users\\test\\media"
        } else {
            "/custom/media"
        };

        // Set a custom media directory
        {
            let conn = db.conn.lock().unwrap();
            conn.execute(
                "UPDATE settings SET value = ?1 WHERE key = 'general.mediaDirectory'",
                [custom_path],
            )
            .unwrap();
        }

        let root = resolve_asset_root(&db, Path::new("/fake")).unwrap();
        assert_eq!(root, PathBuf::from(custom_path));
    }

    #[test]
    fn resolve_asset_root_rejects_relative() {
        let db = setup_test_db();

        {
            let conn = db.conn.lock().unwrap();
            conn.execute(
                "UPDATE settings SET value = ?1 WHERE key = 'general.mediaDirectory'",
                ["relative/path"],
            )
            .unwrap();
        }

        let result = resolve_asset_root(&db, Path::new("/fake"));
        assert!(matches!(result, Err(AssetError::InvalidRoot(_))));
    }

    #[test]
    fn ensure_directories_creates_all() {
        let dir = temp_test_dir("ensure_dirs");
        ensure_directories(&dir).unwrap();

        for subdir in ALL_SUBDIRECTORIES {
            assert!(dir.join(subdir).is_dir(), "Missing subdir: {subdir}");
        }

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn ensure_directories_idempotent() {
        let dir = temp_test_dir("ensure_idem");
        ensure_directories(&dir).unwrap();
        ensure_directories(&dir).unwrap(); // second call should not error

        for subdir in ALL_SUBDIRECTORIES {
            assert!(dir.join(subdir).is_dir());
        }

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn import_file_copies_correctly() {
        let dir = temp_test_dir("import_copy");
        ensure_directories(&dir).unwrap();

        // Create a source file
        let source = dir.join("test_photo.png");
        fs::write(&source, b"fake image data").unwrap();

        let result = import_file(&dir, &source, AssetType::Image).unwrap();

        assert!(!result.id.is_empty());
        assert!(result.relative_path.starts_with("images/"));
        assert!(result.relative_path.ends_with(".png"));
        assert_eq!(result.original_filename, "test_photo.png");
        assert_eq!(result.format, "png");
        assert_eq!(result.file_size, 15); // b"fake image data".len()

        // Verify the file was actually copied
        let dest = dir.join(
            &result
                .relative_path
                .replace('/', std::path::MAIN_SEPARATOR_STR),
        );
        assert!(dest.exists());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn import_file_source_not_found() {
        let dir = temp_test_dir("import_notfound");
        ensure_directories(&dir).unwrap();

        let result = import_file(&dir, Path::new("/nonexistent/file.png"), AssetType::Image);
        assert!(matches!(result, Err(AssetError::SourceNotFound(_))));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn import_file_sanitizes_special_chars() {
        let dir = temp_test_dir("import_sanitize");
        ensure_directories(&dir).unwrap();

        // Create a source file with special chars in name
        let source = dir.join("my_file.txt");
        fs::write(&source, b"data").unwrap();

        // Test the sanitizer directly
        let sanitized = sanitize_filename("bad<>:\"/\\|?*name");
        assert!(!sanitized.contains('<'));
        assert!(!sanitized.contains('>'));
        assert!(!sanitized.contains(':'));
        assert!(!sanitized.contains('"'));
        assert!(!sanitized.contains('\\'));
        assert!(!sanitized.contains('/'));
        assert!(!sanitized.contains('|'));
        assert!(!sanitized.contains('?'));
        assert!(!sanitized.contains('*'));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn import_file_filename_pattern() {
        let dir = temp_test_dir("import_pattern");
        ensure_directories(&dir).unwrap();

        let source = dir.join("photo.png");
        fs::write(&source, b"data").unwrap();

        let result = import_file(&dir, &source, AssetType::Image).unwrap();

        // Should match {uuid}_{stem}.{ext} pattern
        let filename = result.relative_path.strip_prefix("images/").unwrap();
        let parts: Vec<&str> = filename.splitn(2, '_').collect();
        assert_eq!(parts.len(), 2);

        // First part should be a valid UUID
        assert!(Uuid::parse_str(parts[0]).is_ok());
        // Second part should be "photo.png"
        assert_eq!(parts[1], "photo.png");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn import_file_preserves_extension() {
        let dir = temp_test_dir("import_ext");
        ensure_directories(&dir).unwrap();

        let source = dir.join("track.MP3");
        fs::write(&source, b"audio data").unwrap();

        let result = import_file(&dir, &source, AssetType::Audio).unwrap();

        // Extension should be lowercased in the format field
        assert_eq!(result.format, "mp3");

        let _ = fs::remove_dir_all(&dir);
    }
}
