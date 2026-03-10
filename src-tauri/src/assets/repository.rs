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

/// Lists assets with optional type filter and search, ordered by creation date (newest first).
pub fn list_assets(
    conn: &Connection,
    type_filter: Option<&AssetType>,
    search: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<Asset>, AssetError> {
    let mut sql = String::from(
        "SELECT id, original_filename, type, format, file_size, width, height, duration, tags, file_path, created_at FROM assets",
    );
    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(at) = type_filter {
        conditions.push(format!("type = ?{}", params.len() + 1));
        params.push(Box::new(at.as_db_str().to_string()));
    }

    if let Some(q) = search {
        let like = format!("%{q}%");
        conditions.push(format!(
            "(original_filename LIKE ?{} OR tags LIKE ?{})",
            params.len() + 1,
            params.len() + 2,
        ));
        params.push(Box::new(like.clone()));
        params.push(Box::new(like));
    }

    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    sql.push_str(&format!(
        " ORDER BY created_at DESC LIMIT ?{} OFFSET ?{}",
        params.len() + 1,
        params.len() + 2,
    ));
    params.push(Box::new(limit));
    params.push(Box::new(offset));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| AssetError::Database(e.to_string()))?;

    let rows = stmt
        .query_map(param_refs.as_slice(), |row| {
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
        .map_err(|e| AssetError::Database(e.to_string()))?;

    let mut assets = Vec::new();
    for row in rows {
        assets.push(row.map_err(|e| AssetError::Database(e.to_string()))?);
    }
    Ok(assets)
}

/// Counts assets matching the given filters.
pub fn count_assets(
    conn: &Connection,
    type_filter: Option<&AssetType>,
    search: Option<&str>,
) -> Result<i64, AssetError> {
    let mut sql = String::from("SELECT COUNT(*) FROM assets");
    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(at) = type_filter {
        conditions.push(format!("type = ?{}", params.len() + 1));
        params.push(Box::new(at.as_db_str().to_string()));
    }

    if let Some(q) = search {
        let like = format!("%{q}%");
        conditions.push(format!(
            "(original_filename LIKE ?{} OR tags LIKE ?{})",
            params.len() + 1,
            params.len() + 2,
        ));
        params.push(Box::new(like.clone()));
        params.push(Box::new(like));
    }

    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    conn.query_row(&sql, param_refs.as_slice(), |row| row.get(0))
        .map_err(|e| AssetError::Database(e.to_string()))
}

/// Parses a DB type string back into an `AssetType` enum.
pub fn parse_asset_type(s: &str) -> AssetType {
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

    fn sample_audio_asset() -> Asset {
        Asset {
            id: "audio-uuid-5678".to_string(),
            original_filename: "song.mp3".to_string(),
            asset_type: AssetType::Audio,
            format: "mp3".to_string(),
            file_size: 5_000_000,
            width: None,
            height: None,
            duration: Some(180.0),
            tags: vec!["music".to_string()],
            file_path: "audio/audio-uuid-5678_song.mp3".to_string(),
            created_at: "2026-03-09 00:00:00".to_string(),
        }
    }

    #[test]
    fn list_assets_returns_all() {
        let conn = setup_test_db();
        insert_asset(&conn, &sample_asset()).unwrap();
        insert_asset(&conn, &sample_audio_asset()).unwrap();

        let results = list_assets(&conn, None, None, 50, 0).unwrap();
        assert_eq!(results.len(), 2);
        // Newest first
        assert_eq!(results[0].id, "test-uuid-1234");
    }

    #[test]
    fn list_assets_filters_by_type() {
        let conn = setup_test_db();
        insert_asset(&conn, &sample_asset()).unwrap();
        insert_asset(&conn, &sample_audio_asset()).unwrap();

        let results = list_assets(&conn, Some(&AssetType::Audio), None, 50, 0).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].asset_type, AssetType::Audio);
    }

    #[test]
    fn list_assets_search_by_filename() {
        let conn = setup_test_db();
        insert_asset(&conn, &sample_asset()).unwrap();
        insert_asset(&conn, &sample_audio_asset()).unwrap();

        let results = list_assets(&conn, None, Some("photo"), 50, 0).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].original_filename, "photo.png");
    }

    #[test]
    fn list_assets_search_by_tag() {
        let conn = setup_test_db();
        insert_asset(&conn, &sample_asset()).unwrap();
        insert_asset(&conn, &sample_audio_asset()).unwrap();

        let results = list_assets(&conn, None, Some("music"), 50, 0).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "audio-uuid-5678");
    }

    #[test]
    fn list_assets_pagination() {
        let conn = setup_test_db();
        insert_asset(&conn, &sample_asset()).unwrap();
        insert_asset(&conn, &sample_audio_asset()).unwrap();

        let page1 = list_assets(&conn, None, None, 1, 0).unwrap();
        assert_eq!(page1.len(), 1);

        let page2 = list_assets(&conn, None, None, 1, 1).unwrap();
        assert_eq!(page2.len(), 1);
        assert_ne!(page1[0].id, page2[0].id);
    }

    #[test]
    fn count_assets_matches_list() {
        let conn = setup_test_db();
        insert_asset(&conn, &sample_asset()).unwrap();
        insert_asset(&conn, &sample_audio_asset()).unwrap();

        assert_eq!(count_assets(&conn, None, None).unwrap(), 2);
        assert_eq!(count_assets(&conn, Some(&AssetType::Image), None).unwrap(), 1);
        assert_eq!(count_assets(&conn, None, Some("song")).unwrap(), 1);
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
