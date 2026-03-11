use rusqlite::{Connection, OptionalExtension};

use super::error::DesignError;
use super::types::{Design, DesignTree, DesignType};

/// Parses a DB type string back into a `DesignType` enum.
pub fn parse_design_type(s: &str) -> DesignType {
    match s {
        "alert" => DesignType::Alert,
        "overlay" => DesignType::Overlay,
        "scene" => DesignType::Scene,
        "stinger" => DesignType::Stinger,
        "panel" => DesignType::Panel,
        // DB CHECK constraint guarantees valid values; default to Alert as fallback
        _ => DesignType::Alert,
    }
}

/// Maps a `rusqlite::Row` into a `Design` struct.
fn row_to_design(row: &rusqlite::Row) -> rusqlite::Result<Result<Design, DesignError>> {
    let type_str: String = row.get(2)?;
    let config_str: String = row.get(3)?;
    let thumbnail: Option<String> = row.get(4)?;
    let tags_str: String = row.get(5)?;
    let description: String = row.get(6)?;

    let config: DesignTree = match serde_json::from_str(&config_str) {
        Ok(c) => c,
        Err(e) => return Ok(Err(DesignError::Serialization(e.to_string()))),
    };

    let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();

    Ok(Ok(Design {
        id: row.get(0)?,
        name: row.get(1)?,
        design_type: parse_design_type(&type_str),
        config,
        thumbnail,
        tags,
        description,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    }))
}

const SELECT_COLUMNS: &str =
    "id, name, type, config, thumbnail, tags, description, created_at, updated_at";

/// Inserts a new design record into the `designs` table.
pub fn insert_design(conn: &Connection, design: &Design) -> Result<(), DesignError> {
    let config_json = serde_json::to_string(&design.config)
        .map_err(|e| DesignError::Serialization(e.to_string()))?;
    let tags_json = serde_json::to_string(&design.tags)
        .map_err(|e| DesignError::Serialization(e.to_string()))?;

    conn.execute(
        "INSERT INTO designs (id, name, type, config, thumbnail, tags, description, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            design.id,
            design.name,
            design.design_type.as_db_str(),
            config_json,
            design.thumbnail,
            tags_json,
            design.description,
            design.created_at,
            design.updated_at,
        ],
    )
    .map_err(|e| DesignError::Database(e.to_string()))?;

    Ok(())
}

/// Retrieves a design by its ID. Returns `None` if not found.
pub fn get_design_by_id(conn: &Connection, id: &str) -> Result<Option<Design>, DesignError> {
    let mut stmt = conn
        .prepare(&format!(
            "SELECT {SELECT_COLUMNS} FROM designs WHERE id = ?1"
        ))
        .map_err(|e| DesignError::Database(e.to_string()))?;

    let result = stmt
        .query_row(rusqlite::params![id], row_to_design)
        .optional()
        .map_err(|e| DesignError::Database(e.to_string()))?;

    match result {
        Some(inner) => inner.map(Some),
        None => Ok(None),
    }
}

/// Lists designs with optional type filter and search, ordered by `updated_at DESC`.
pub fn list_designs(
    conn: &Connection,
    type_filter: Option<&DesignType>,
    search: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<Design>, DesignError> {
    let mut sql = format!("SELECT {SELECT_COLUMNS} FROM designs");
    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(dt) = type_filter {
        conditions.push(format!("type = ?{}", params.len() + 1));
        params.push(Box::new(dt.as_db_str().to_string()));
    }

    if let Some(q) = search {
        let like = format!("%{q}%");
        conditions.push(format!(
            "(name LIKE ?{} OR tags LIKE ?{} OR description LIKE ?{})",
            params.len() + 1,
            params.len() + 2,
            params.len() + 3,
        ));
        params.push(Box::new(like.clone()));
        params.push(Box::new(like.clone()));
        params.push(Box::new(like));
    }

    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    sql.push_str(&format!(
        " ORDER BY updated_at DESC LIMIT ?{} OFFSET ?{}",
        params.len() + 1,
        params.len() + 2,
    ));
    params.push(Box::new(limit));
    params.push(Box::new(offset));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| DesignError::Database(e.to_string()))?;

    let rows = stmt
        .query_map(param_refs.as_slice(), row_to_design)
        .map_err(|e| DesignError::Database(e.to_string()))?;

    let mut designs = Vec::new();
    for row in rows {
        let inner = row.map_err(|e| DesignError::Database(e.to_string()))?;
        designs.push(inner?);
    }
    Ok(designs)
}

/// Counts designs matching the given filters.
pub fn count_designs(
    conn: &Connection,
    type_filter: Option<&DesignType>,
    search: Option<&str>,
) -> Result<i64, DesignError> {
    let mut sql = String::from("SELECT COUNT(*) FROM designs");
    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(dt) = type_filter {
        conditions.push(format!("type = ?{}", params.len() + 1));
        params.push(Box::new(dt.as_db_str().to_string()));
    }

    if let Some(q) = search {
        let like = format!("%{q}%");
        conditions.push(format!(
            "(name LIKE ?{} OR tags LIKE ?{} OR description LIKE ?{})",
            params.len() + 1,
            params.len() + 2,
            params.len() + 3,
        ));
        params.push(Box::new(like.clone()));
        params.push(Box::new(like.clone()));
        params.push(Box::new(like));
    }

    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    conn.query_row(&sql, param_refs.as_slice(), |row| row.get(0))
        .map_err(|e| DesignError::Database(e.to_string()))
}

/// Updates specific fields of a design. Only `Some` fields are modified.
///
/// `thumbnail` uses `Option<Option<&str>>`:
/// - `None` → skip (don't change)
/// - `Some(None)` → set to NULL
/// - `Some(Some(val))` → set to value
pub fn update_design(
    conn: &Connection,
    id: &str,
    name: Option<&str>,
    config: Option<&DesignTree>,
    thumbnail: Option<Option<&str>>,
    tags: Option<&Vec<String>>,
    description: Option<&str>,
) -> Result<(), DesignError> {
    let mut set_clauses: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(n) = name {
        params.push(Box::new(n.to_string()));
        set_clauses.push(format!("name = ?{}", params.len()));
    }

    if let Some(c) = config {
        let json =
            serde_json::to_string(c).map_err(|e| DesignError::Serialization(e.to_string()))?;
        params.push(Box::new(json));
        set_clauses.push(format!("config = ?{}", params.len()));
    }

    if let Some(thumb) = thumbnail {
        match thumb {
            Some(val) => {
                params.push(Box::new(val.to_string()));
                set_clauses.push(format!("thumbnail = ?{}", params.len()));
            }
            None => {
                set_clauses.push("thumbnail = NULL".to_string());
            }
        }
    }

    if let Some(t) = tags {
        let json =
            serde_json::to_string(t).map_err(|e| DesignError::Serialization(e.to_string()))?;
        params.push(Box::new(json));
        set_clauses.push(format!("tags = ?{}", params.len()));
    }

    if let Some(d) = description {
        params.push(Box::new(d.to_string()));
        set_clauses.push(format!("description = ?{}", params.len()));
    }

    // Always update timestamp
    set_clauses.push("updated_at = datetime('now')".to_string());

    // Add the id parameter last
    params.push(Box::new(id.to_string()));
    let id_idx = params.len();

    let sql = format!(
        "UPDATE designs SET {} WHERE id = ?{}",
        set_clauses.join(", "),
        id_idx,
    );

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let changes = conn
        .execute(&sql, param_refs.as_slice())
        .map_err(|e| DesignError::Database(e.to_string()))?;

    if changes == 0 {
        return Err(DesignError::NotFound(id.to_string()));
    }

    Ok(())
}

/// Deletes a design by ID.
pub fn delete_design(conn: &Connection, id: &str) -> Result<(), DesignError> {
    let changes = conn
        .execute("DELETE FROM designs WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| DesignError::Database(e.to_string()))?;

    if changes == 0 {
        return Err(DesignError::NotFound(id.to_string()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migration::run_migrations;
    use crate::designs::types::{
        CanvasSize, DesignElement, ElementData, Position, Size, TextElementData,
    };
    use crate::settings::defaults::seed_defaults_if_empty;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        seed_defaults_if_empty(&conn).unwrap();
        conn
    }

    fn sample_design() -> Design {
        Design {
            id: "design-uuid-1234".to_string(),
            name: "Test Alert".to_string(),
            design_type: DesignType::Alert,
            config: DesignTree {
                schema_version: 1,
                canvas: CanvasSize {
                    width: 1920,
                    height: 1080,
                },
                background_color: "#0A0D14".to_string(),
                elements: vec![DesignElement {
                    id: "elem-1".to_string(),
                    name: "Title".to_string(),
                    position: Position { x: 0.0, y: 0.0 },
                    size: Size {
                        width: 500.0,
                        height: 100.0,
                    },
                    rotation: 0.0,
                    opacity: 1.0,
                    visible: true,
                    locked: false,
                    layer_order: 0,
                    animation: None,
                    sound: None,
                    data: ElementData::Text(TextElementData {
                        text: "New Follower!".to_string(),
                        font_family: "Inter".to_string(),
                        font_size: 32.0,
                        font_weight: 700,
                        color: "#00E5FF".to_string(),
                        text_align: crate::designs::types::TextAlign::Center,
                        line_height: 1.4,
                        stroke: None,
                        shadow: None,
                    }),
                }],
            },
            thumbnail: Some("thumbnails/design-uuid-1234.png".to_string()),
            tags: vec!["alert".to_string(), "follower".to_string()],
            description: "Alert for new followers".to_string(),
            created_at: "2026-03-10 00:00:00".to_string(),
            updated_at: "2026-03-10 00:00:00".to_string(),
        }
    }

    fn sample_overlay() -> Design {
        Design {
            id: "design-uuid-5678".to_string(),
            name: "Stream Overlay".to_string(),
            design_type: DesignType::Overlay,
            config: DesignTree::default(),
            thumbnail: None,
            tags: vec!["overlay".to_string(), "stream".to_string()],
            description: "Main stream overlay".to_string(),
            created_at: "2026-03-09 00:00:00".to_string(),
            updated_at: "2026-03-09 00:00:00".to_string(),
        }
    }

    #[test]
    fn insert_and_get_round_trip() {
        let conn = setup_test_db();
        let design = sample_design();

        insert_design(&conn, &design).unwrap();
        let fetched = get_design_by_id(&conn, &design.id).unwrap().unwrap();

        assert_eq!(fetched.id, design.id);
        assert_eq!(fetched.name, design.name);
        assert_eq!(fetched.design_type, DesignType::Alert);
        assert_eq!(fetched.config.elements.len(), 1);
        assert_eq!(fetched.tags, vec!["alert", "follower"]);
        assert_eq!(fetched.description, "Alert for new followers");
        assert!(fetched.thumbnail.is_some());
    }

    #[test]
    fn get_nonexistent_returns_none() {
        let conn = setup_test_db();
        let result = get_design_by_id(&conn, "nonexistent-id").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn insert_duplicate_id_fails() {
        let conn = setup_test_db();
        let design = sample_design();

        insert_design(&conn, &design).unwrap();
        let result = insert_design(&conn, &design);
        assert!(matches!(result, Err(DesignError::Database(_))));
    }

    #[test]
    fn list_designs_returns_all() {
        let conn = setup_test_db();
        insert_design(&conn, &sample_design()).unwrap();
        insert_design(&conn, &sample_overlay()).unwrap();

        let results = list_designs(&conn, None, None, 50, 0).unwrap();
        assert_eq!(results.len(), 2);
        // Most recently updated first
        assert_eq!(results[0].id, "design-uuid-1234");
    }

    #[test]
    fn list_designs_filters_by_type() {
        let conn = setup_test_db();
        insert_design(&conn, &sample_design()).unwrap();
        insert_design(&conn, &sample_overlay()).unwrap();

        let results = list_designs(&conn, Some(&DesignType::Overlay), None, 50, 0).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].design_type, DesignType::Overlay);
    }

    #[test]
    fn list_designs_search_by_name() {
        let conn = setup_test_db();
        insert_design(&conn, &sample_design()).unwrap();
        insert_design(&conn, &sample_overlay()).unwrap();

        let results = list_designs(&conn, None, Some("Stream"), 50, 0).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Stream Overlay");
    }

    #[test]
    fn list_designs_search_by_tag() {
        let conn = setup_test_db();
        insert_design(&conn, &sample_design()).unwrap();
        insert_design(&conn, &sample_overlay()).unwrap();

        let results = list_designs(&conn, None, Some("follower"), 50, 0).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "design-uuid-1234");
    }

    #[test]
    fn list_designs_search_by_description() {
        let conn = setup_test_db();
        insert_design(&conn, &sample_design()).unwrap();
        insert_design(&conn, &sample_overlay()).unwrap();

        let results = list_designs(&conn, None, Some("main stream"), 50, 0).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "design-uuid-5678");
    }

    #[test]
    fn list_designs_pagination() {
        let conn = setup_test_db();
        insert_design(&conn, &sample_design()).unwrap();
        insert_design(&conn, &sample_overlay()).unwrap();

        let page1 = list_designs(&conn, None, None, 1, 0).unwrap();
        assert_eq!(page1.len(), 1);

        let page2 = list_designs(&conn, None, None, 1, 1).unwrap();
        assert_eq!(page2.len(), 1);
        assert_ne!(page1[0].id, page2[0].id);
    }

    #[test]
    fn count_designs_matches_list() {
        let conn = setup_test_db();
        insert_design(&conn, &sample_design()).unwrap();
        insert_design(&conn, &sample_overlay()).unwrap();

        assert_eq!(count_designs(&conn, None, None).unwrap(), 2);
        assert_eq!(
            count_designs(&conn, Some(&DesignType::Alert), None).unwrap(),
            1
        );
        assert_eq!(count_designs(&conn, None, Some("overlay")).unwrap(), 1);
    }

    #[test]
    fn update_design_name() {
        let conn = setup_test_db();
        insert_design(&conn, &sample_design()).unwrap();

        update_design(
            &conn,
            "design-uuid-1234",
            Some("Renamed Alert"),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let fetched = get_design_by_id(&conn, "design-uuid-1234")
            .unwrap()
            .unwrap();
        assert_eq!(fetched.name, "Renamed Alert");
    }

    #[test]
    fn update_design_config() {
        let conn = setup_test_db();
        insert_design(&conn, &sample_design()).unwrap();

        let new_config = DesignTree {
            background_color: "#FF0000".to_string(),
            ..DesignTree::default()
        };

        update_design(
            &conn,
            "design-uuid-1234",
            None,
            Some(&new_config),
            None,
            None,
            None,
        )
        .unwrap();

        let fetched = get_design_by_id(&conn, "design-uuid-1234")
            .unwrap()
            .unwrap();
        assert_eq!(fetched.config.background_color, "#FF0000");
        assert!(fetched.config.elements.is_empty());
    }

    #[test]
    fn update_design_thumbnail_clear() {
        let conn = setup_test_db();
        insert_design(&conn, &sample_design()).unwrap();

        // Clear thumbnail (set to NULL)
        update_design(
            &conn,
            "design-uuid-1234",
            None,
            None,
            Some(None),
            None,
            None,
        )
        .unwrap();

        let fetched = get_design_by_id(&conn, "design-uuid-1234")
            .unwrap()
            .unwrap();
        assert!(fetched.thumbnail.is_none());
    }

    #[test]
    fn update_design_tags() {
        let conn = setup_test_db();
        insert_design(&conn, &sample_design()).unwrap();

        let new_tags = vec!["updated".to_string(), "custom".to_string()];
        update_design(
            &conn,
            "design-uuid-1234",
            None,
            None,
            None,
            Some(&new_tags),
            None,
        )
        .unwrap();

        let fetched = get_design_by_id(&conn, "design-uuid-1234")
            .unwrap()
            .unwrap();
        assert_eq!(fetched.tags, vec!["updated", "custom"]);
    }

    #[test]
    fn update_nonexistent_returns_not_found() {
        let conn = setup_test_db();
        let result = update_design(
            &conn,
            "nonexistent-id",
            Some("Name"),
            None,
            None,
            None,
            None,
        );
        assert!(matches!(result, Err(DesignError::NotFound(_))));
    }

    #[test]
    fn delete_design_removes_record() {
        let conn = setup_test_db();
        let design = sample_design();
        insert_design(&conn, &design).unwrap();

        delete_design(&conn, &design.id).unwrap();
        assert!(get_design_by_id(&conn, &design.id).unwrap().is_none());
    }

    #[test]
    fn delete_nonexistent_returns_not_found() {
        let conn = setup_test_db();
        let result = delete_design(&conn, "nonexistent-id");
        assert!(matches!(result, Err(DesignError::NotFound(_))));
    }
}
