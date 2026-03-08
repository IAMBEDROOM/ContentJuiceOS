use std::fmt;

use crate::credentials::error::CredentialError;
use crate::db::error::DbError;

#[derive(Debug)]
#[allow(dead_code)]
pub enum PlatformError {
    Database(DbError),
    NotFound(String),
    OAuth(String),
    Http(String),
    InvalidState(String),
    Credential(CredentialError),
}

impl fmt::Display for PlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(e) => write!(f, "Database error: {e}"),
            Self::NotFound(msg) => write!(f, "Not found: {msg}"),
            Self::OAuth(msg) => write!(f, "OAuth error: {msg}"),
            Self::Http(msg) => write!(f, "HTTP error: {msg}"),
            Self::InvalidState(msg) => write!(f, "Invalid state: {msg}"),
            Self::Credential(e) => write!(f, "Credential error: {e}"),
        }
    }
}

impl std::error::Error for PlatformError {}

impl From<reqwest::Error> for PlatformError {
    fn from(e: reqwest::Error) -> Self {
        Self::Http(e.to_string())
    }
}

impl From<DbError> for PlatformError {
    fn from(e: DbError) -> Self {
        Self::Database(e)
    }
}

impl From<CredentialError> for PlatformError {
    fn from(e: CredentialError) -> Self {
        Self::Credential(e)
    }
}

pub type PlatformResult<T> = Result<T, PlatformError>;
