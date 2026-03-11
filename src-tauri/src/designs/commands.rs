use std::sync::Arc;

use tauri::State;

use crate::db::Database;
use crate::user_error::UserFacingError;

use super::service;
use super::types::{Design, DesignListResponse, DesignTree, DesignType};

#[tauri::command]
pub fn create_design(
    name: String,
    design_type: String,
    config: Option<DesignTree>,
    description: Option<String>,
    tags: Option<Vec<String>>,
    database: State<'_, Arc<Database>>,
) -> Result<Design, String> {
    let dt = parse_design_type_str(&design_type);
    service::create_design(&database, name, dt, config, description, tags).map_user_err()
}

#[tauri::command]
pub fn get_design(id: String, database: State<'_, Arc<Database>>) -> Result<Design, String> {
    service::get_design(&database, &id).map_user_err()
}

#[tauri::command]
pub fn list_designs(
    type_filter: Option<String>,
    search: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
    database: State<'_, Arc<Database>>,
) -> Result<DesignListResponse, String> {
    let design_type = type_filter.as_deref().map(parse_design_type_str);
    let limit = limit.unwrap_or(50);
    let offset = offset.unwrap_or(0);

    service::list_designs(
        &database,
        design_type.as_ref(),
        search.as_deref(),
        limit,
        offset,
    )
    .map_user_err()
}

#[tauri::command]
pub fn update_design(
    id: String,
    name: Option<String>,
    config: Option<DesignTree>,
    thumbnail: Option<String>,
    tags: Option<Vec<String>>,
    description: Option<String>,
    database: State<'_, Arc<Database>>,
) -> Result<Design, String> {
    // thumbnail semantics:
    // - None (not provided)    → don't change
    // - Some("")               → clear (set to NULL)
    // - Some(val)              → set to val
    let thumb = thumbnail
        .as_ref()
        .map(|s| if s.is_empty() { None } else { Some(s.as_str()) });

    service::update_design(
        &database,
        &id,
        name.as_deref(),
        config.as_ref(),
        thumb,
        tags.as_ref(),
        description.as_deref(),
    )
    .map_user_err()
}

#[tauri::command]
pub fn delete_design(id: String, database: State<'_, Arc<Database>>) -> Result<(), String> {
    service::delete_design(&database, &id).map_user_err()
}

#[tauri::command]
pub fn duplicate_design(id: String, database: State<'_, Arc<Database>>) -> Result<Design, String> {
    service::duplicate_design(&database, &id).map_user_err()
}

fn parse_design_type_str(s: &str) -> DesignType {
    match s {
        "alert" => DesignType::Alert,
        "overlay" => DesignType::Overlay,
        "scene" => DesignType::Scene,
        "stinger" => DesignType::Stinger,
        "panel" => DesignType::Panel,
        _ => DesignType::Alert,
    }
}
