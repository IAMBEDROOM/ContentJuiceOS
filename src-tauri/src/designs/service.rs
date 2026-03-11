use chrono::Utc;
use uuid::Uuid;

use crate::db::Database;

use super::error::DesignError;
use super::repository;
use super::types::{Design, DesignListResponse, DesignTree, DesignType};

/// Creates a new design with a generated UUID and timestamps.
pub fn create_design(
    db: &Database,
    name: String,
    design_type: DesignType,
    config: Option<DesignTree>,
    description: Option<String>,
    tags: Option<Vec<String>>,
) -> Result<Design, DesignError> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(DesignError::Validation(
            "Design name cannot be empty.".to_string(),
        ));
    }

    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let design = Design {
        id: Uuid::new_v4().to_string(),
        name,
        design_type,
        config: config.unwrap_or_default(),
        thumbnail: None,
        tags: tags.unwrap_or_default(),
        description: description.unwrap_or_default(),
        created_at: now.clone(),
        updated_at: now,
    };

    let conn = db
        .conn
        .lock()
        .map_err(|e| DesignError::Database(format!("Failed to lock database: {e}")))?;

    repository::insert_design(&conn, &design)?;

    Ok(design)
}

/// Retrieves a single design by ID.
pub fn get_design(db: &Database, id: &str) -> Result<Design, DesignError> {
    let conn = db
        .conn
        .lock()
        .map_err(|e| DesignError::Database(format!("Failed to lock database: {e}")))?;

    repository::get_design_by_id(&conn, id)?.ok_or_else(|| DesignError::NotFound(id.to_string()))
}

/// Lists designs with optional filtering, search, and pagination.
pub fn list_designs(
    db: &Database,
    type_filter: Option<&DesignType>,
    search: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<DesignListResponse, DesignError> {
    let conn = db
        .conn
        .lock()
        .map_err(|e| DesignError::Database(format!("Failed to lock database: {e}")))?;

    let designs = repository::list_designs(&conn, type_filter, search, limit, offset)?;
    let total = repository::count_designs(&conn, type_filter, search)?;

    Ok(DesignListResponse { designs, total })
}

/// Updates specific fields of a design, returning the updated record.
pub fn update_design(
    db: &Database,
    id: &str,
    name: Option<&str>,
    config: Option<&DesignTree>,
    thumbnail: Option<Option<&str>>,
    tags: Option<&Vec<String>>,
    description: Option<&str>,
) -> Result<Design, DesignError> {
    let conn = db
        .conn
        .lock()
        .map_err(|e| DesignError::Database(format!("Failed to lock database: {e}")))?;

    repository::update_design(&conn, id, name, config, thumbnail, tags, description)?;

    repository::get_design_by_id(&conn, id)?.ok_or_else(|| DesignError::NotFound(id.to_string()))
}

/// Deletes a design by ID.
pub fn delete_design(db: &Database, id: &str) -> Result<(), DesignError> {
    let conn = db
        .conn
        .lock()
        .map_err(|e| DesignError::Database(format!("Failed to lock database: {e}")))?;

    repository::delete_design(&conn, id)
}

/// Duplicates an existing design with a new ID and "(Copy)" suffix.
pub fn duplicate_design(db: &Database, id: &str) -> Result<Design, DesignError> {
    let conn = db
        .conn
        .lock()
        .map_err(|e| DesignError::Database(format!("Failed to lock database: {e}")))?;

    let original = repository::get_design_by_id(&conn, id)?
        .ok_or_else(|| DesignError::NotFound(id.to_string()))?;

    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let copy = Design {
        id: Uuid::new_v4().to_string(),
        name: format!("{} (Copy)", original.name),
        design_type: original.design_type,
        config: original.config,
        thumbnail: None,
        tags: original.tags,
        description: original.description,
        created_at: now.clone(),
        updated_at: now,
    };

    repository::insert_design(&conn, &copy)?;

    Ok(copy)
}
