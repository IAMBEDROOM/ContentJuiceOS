use std::sync::Arc;

use tauri::State;

use crate::credentials::store::CredentialManager;
use crate::credentials::types::CredentialKind;
use crate::db::Database;
use crate::user_error::UserFacingError;

use super::repository;
use super::types::PlatformConnection;

#[tauri::command]
pub fn get_platform_connections(
    db: State<'_, Arc<Database>>,
) -> Result<Vec<PlatformConnection>, String> {
    let conn = db.conn.lock().map_user_err()?;
    repository::list_connections(&conn).map_user_err()
}

#[tauri::command]
pub fn get_platform_connection(
    id: String,
    db: State<'_, Arc<Database>>,
) -> Result<Option<PlatformConnection>, String> {
    let conn = db.conn.lock().map_user_err()?;
    repository::get_connection(&conn, &id).map_user_err()
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
        .map_user_err()?;

    // Update status in DB
    let conn = db.conn.lock().map_user_err()?;
    repository::update_connection_status(&conn, &id, "disconnected").map_user_err()
}
