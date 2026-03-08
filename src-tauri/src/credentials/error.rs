use crate::db::error::DbError;
use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum CredentialError {
    #[error("Keychain error: {0}")]
    Keychain(String),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Database error: {0}")]
    Database(#[from] DbError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Credential not found: {0}")]
    NotFound(String),
}

pub type CredResult<T> = Result<T, CredentialError>;
