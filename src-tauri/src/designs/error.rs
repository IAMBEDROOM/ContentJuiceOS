use std::fmt;

/// Errors that can occur during design CRUD operations.
#[derive(Debug)]
pub enum DesignError {
    /// SQLite operation failed
    Database(String),
    /// Design ID not found
    NotFound(String),
    /// JSON serialization/deserialization failed
    Serialization(String),
    /// Validation failure (empty name, invalid type, etc.)
    Validation(String),
}

impl fmt::Display for DesignError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(msg) => write!(f, "Design database error: {msg}"),
            Self::NotFound(id) => write!(f, "Design not found: {id}"),
            Self::Serialization(msg) => write!(f, "Design serialization error: {msg}"),
            Self::Validation(msg) => write!(f, "Design validation error: {msg}"),
        }
    }
}

impl std::error::Error for DesignError {}
