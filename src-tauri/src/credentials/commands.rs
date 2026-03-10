use tauri::State;

use crate::user_error::UserFacingError;

use super::store::CredentialManager;
use super::types::{CredentialKind, OAuthTokens};

#[tauri::command]
pub fn store_credential(
    kind: CredentialKind,
    value: String,
    cred_manager: State<'_, CredentialManager>,
) -> Result<(), String> {
    cred_manager.store_credential(&kind, &value).map_user_err()
}

#[tauri::command]
pub fn get_credential(
    kind: CredentialKind,
    cred_manager: State<'_, CredentialManager>,
) -> Result<Option<String>, String> {
    cred_manager.get_credential(&kind).map_user_err()
}

#[tauri::command]
pub fn delete_credential(
    kind: CredentialKind,
    cred_manager: State<'_, CredentialManager>,
) -> Result<(), String> {
    cred_manager.delete_credential(&kind).map_user_err()
}

#[tauri::command]
pub fn has_credential(
    kind: CredentialKind,
    cred_manager: State<'_, CredentialManager>,
) -> Result<bool, String> {
    cred_manager.has_credential(&kind).map_user_err()
}

#[tauri::command]
pub fn get_credential_backend(
    cred_manager: State<'_, CredentialManager>,
) -> Result<String, String> {
    match cred_manager.backend() {
        super::types::CredentialBackend::Keychain => Ok("keychain".to_string()),
        super::types::CredentialBackend::EncryptedSqlite => Ok("encrypted_sqlite".to_string()),
    }
}

#[tauri::command]
pub fn store_platform_tokens(
    connection_id: String,
    access_token: String,
    refresh_token: Option<String>,
    expires_at: Option<String>,
    cred_manager: State<'_, CredentialManager>,
) -> Result<(), String> {
    let tokens = OAuthTokens {
        access_token,
        refresh_token,
        token_expires_at: expires_at,
    };
    cred_manager
        .store_platform_tokens(&connection_id, &tokens)
        .map_user_err()
}

#[tauri::command]
pub fn get_platform_tokens(
    connection_id: String,
    cred_manager: State<'_, CredentialManager>,
) -> Result<Option<OAuthTokens>, String> {
    cred_manager
        .get_platform_tokens(&connection_id)
        .map_user_err()
}
