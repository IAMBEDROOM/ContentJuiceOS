use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Settings error: {0}")]
    Settings(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type DbResult<T> = Result<T, DbError>;
