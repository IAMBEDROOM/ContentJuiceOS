use rusqlite::Connection;

use super::error::AssetError;
use super::types::{Asset, AssetType};

/// Inserts a new asset record into the `assets` table.
pub fn insert_asset(conn: &Connection, asset: &Asset) -> Result<(), AssetError> {
    let tags_json =
        serde_json::to_string(&asset.tags).map_err(|e| AssetError::Database(e.to_string()))?;

    conn.execute(
        "INSERT INTO assets (id, original_filename, type, format, file_size, width, height, duration, tags, file_path, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        rusqlite::params![
            asset.id,
            asset.original_filename,
            asset.asset_type.as_db_str(),
            asset.format,
            asset.file_size,
            asset.width,
            asset.height,
            asset.duration,
            tags_json,
            asset.file_path,
            asset.created_at,
        ],
    )
    .map_err(|e| AssetError::Database(e.to_string()))?;

    Ok(())
}

/// Retrieves an asset by its ID. Returns `None` if not found.
#[allow(dead_code)]
pub fn get_asset_by_id(conn: &Connection, id: &str) -> Result<Option<Asset>, AssetError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, original_filename, type, format, file_size, width, height, duration, tags, file_path, created_at
             FROM assets WHERE id = ?1",
        )
        .map_err(|e| AssetError::Database(e.to_string()))?;

    let result = stmt
        .query_row(rusqlite::params![id], |row| {
            let type_str: String = row.get(2)?;
            let tags_str: String = row.get(8)?;

            Ok(Asset {
                id: row.get(0)?,
                original_filename: row.get(1)?,
                asset_type: parse_asset_type(&type_str),
                format: row.get(3)?,
                file_size: row.get(4)?,
                width: row.get(5)?,
                height: row.get(6)?,
                duration: row.get(7)?,
                tags: serde_json::from_str(&tags_str).unwrap_or_default(),
                file_path: row.get(9)?,
                created_at: row.get(10)?,
            })
        })
        .optional()
        .map_err(|e| AssetError::Database(e.to_string()))?;

    Ok(result)
}

/// Parses a DB type string back into an `AssetType` enum.
#[allow(dead_code)]
fn parse_asset_type(s: &str) -> AssetType {
    match s {
        "image" => AssetType::Image,
        "audio" => AssetType::Audio,
        "video" => AssetType::Video,
        "font" => AssetType::Font,
        "animation" => AssetType::Animation,
        "caption" => AssetType::Caption,
        // DB CHECK constraint guarantees valid values; default to Image as fallback
        _ => AssetType::Image,
    }
}

/// Re-export for rusqlite optional query support.
use rusqlite::OptionalExtension;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migration::run_migrations;
    use crate::settings::defaults::seed_defaults_if_empty;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        seed_defaults_if_empty(&conn).unwrap();
        conn
    }

    fn sample_asset() -> Asset {
        Asset {
            id: "test-uuid-1234".to_string(),
            original_filename: "photo.png".to_string(),
            asset_type: AssetType::Image,
            format: "png".to_string(),
            file_size: 1024,
            width: Some(1920),
            height: Some(1080),
            duration: None,
            tags: vec!["test".to_string(), "photo".to_string()],
            file_path: "images/test-uuid-1234_photo.png".to_string(),
            created_at: "2026-03-10 00:00:00".to_string(),
        }
    }

    #[test]
    fn insert_and_get_round_trip() {
        let conn = setup_test_db();
        let asset = sample_asset();

        insert_asset(&conn, &asset).unwrap();

        let fetched = get_asset_by_id(&conn, &asset.id).unwrap().unwrap();
        assert_eq!(fetched.id, asset.id);
        assert_eq!(fetched.original_filename, asset.original_filename);
        assert_eq!(fetched.asset_type, AssetType::Image);
        assert_eq!(fetched.format, "png");
        assert_eq!(fetched.file_size, 1024);
        assert_eq!(fetched.width, Some(1920));
        assert_eq!(fetched.height, Some(1080));
        assert_eq!(fetched.duration, None);
        assert_eq!(fetched.tags, vec!["test", "photo"]);
        assert_eq!(fetched.file_path, asset.file_path);
    }

    #[test]
    fn get_nonexistent_returns_none() {
        let conn = setup_test_db();
        let result = get_asset_by_id(&conn, "nonexistent-id").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn insert_duplicate_id_fails() {
        let conn = setup_test_db();
        let asset = sample_asset();

        insert_asset(&conn, &asset).unwrap();
        let result = insert_asset(&conn, &asset);
        assert!(matches!(result, Err(AssetError::Database(_))));
    }

    #[test]
    fn round_trip_with_video_type() {
        let conn = setup_test_db();
        let asset = Asset {
            id: "video-uuid-5678".to_string(),
            original_filename: "clip.mp4".to_string(),
            asset_type: AssetType::Video,
            format: "mp4".to_string(),
            file_size: 50_000_000,
            width: Some(3840),
            height: Some(2160),
            duration: Some(120.5),
            tags: vec![],
            file_path: "video/video-uuid-5678_clip.mp4".to_string(),
            created_at: "2026-03-10 12:00:00".to_string(),
        };

        insert_asset(&conn, &asset).unwrap();
        let fetched = get_asset_by_id(&conn, &asset.id).unwrap().unwrap();
        assert_eq!(fetched.asset_type, AssetType::Video);
        assert_eq!(fetched.duration, Some(120.5));
        assert_eq!(fetched.width, Some(3840));
    }
}
