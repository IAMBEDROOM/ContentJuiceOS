use std::fmt;

use crate::db::error::DbError;

#[derive(Debug)]
#[allow(dead_code)]
pub enum CacheError {
    Database(DbError),
    Serialization(String),
    NotFound {
        cache_type: String,
        cache_key: String,
    },
    InvalidType(String),
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(e) => write!(f, "Cache database error: {e}"),
            Self::Serialization(msg) => write!(f, "Cache serialization error: {msg}"),
            Self::NotFound {
                cache_type,
                cache_key,
            } => {
                write!(
                    f,
                    "Cache entry not found: type={cache_type}, key={cache_key}"
                )
            }
            Self::InvalidType(t) => write!(f, "Invalid cache type: {t}"),
        }
    }
}

impl std::error::Error for CacheError {}

impl From<DbError> for CacheError {
    fn from(e: DbError) -> Self {
        Self::Database(e)
    }
}

impl From<serde_json::Error> for CacheError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serialization(e.to_string())
    }
}

pub type CacheResult<T> = Result<T, CacheError>;
