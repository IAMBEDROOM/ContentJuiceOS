use std::sync::Arc;

use tauri::State;

use crate::credentials::store::CredentialManager;
use crate::credentials::types::CredentialKind;
use crate::db::Database;

use super::repository;
use super::types::PlatformConnection;

#[tauri::command]
pub fn get_platform_connections(
    db: State<'_, Arc<Database>>,
) -> Result<Vec<PlatformConnection>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    repository::list_connections(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_platform_connection(
    id: String,
    db: State<'_, Arc<Database>>,
) -> Result<Option<PlatformConnection>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    repository::get_connection(&conn, &id).map_err(|e| e.to_string())
}

/// Disconnect a platform — sets status to 'disconnected' and deletes stored tokens.
#[tauri::command]
pub fn disconnect_platform(
    id: String,
    db: State<'_, Arc<Database>>,
    cred_manager: State<'_, CredentialManager>,
) -> Result<(), String> {
    // Delete tokens from credential store
    let kind = CredentialKind::PlatformToken {
        connection_id: id.clone(),
    };
    cred_manager
        .delete_credential(&kind)
        .map_err(|e| e.to_string())?;

    // Update status in DB
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    repository::update_connection_status(&conn, &id, "disconnected").map_err(|e| e.to_string())
}
