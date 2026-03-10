use std::fmt;

use super::types::AssetReference;

/// Errors that can occur during asset storage operations.
#[derive(Debug)]
#[allow(dead_code)]
pub enum AssetError {
    /// Source file does not exist at the given path
    SourceNotFound(String),
    /// Filesystem I/O failure
    Io(std::io::Error),
    /// The configured media directory is invalid (e.g. relative path)
    InvalidRoot(String),
    /// File has no valid filename or extension
    InvalidFilename(String),
    /// Could not read settings from the database
    SettingsError(String),
    /// File extension is not in the allowed formats list
    FormatNotSupported(String),
    /// File exceeds the maximum allowed size for its asset type
    FileTooLarge { limit_bytes: u64, actual_bytes: u64 },
    /// Failed to extract metadata via ffprobe
    MetadataExtraction(String),
    /// Database operation failed during asset insert/query
    Database(String),
    /// Asset ID does not exist in the database
    NotFound(String),
    /// Asset is referenced by other entities and force was not set
    AssetInUse {
        asset_id: String,
        references: Vec<AssetReference>,
    },
    /// Deletion blocked by a foreign key constraint (e.g. project source video)
    DeleteBlocked(String),
}

impl fmt::Display for AssetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceNotFound(path) => write!(f, "Source file not found: {path}"),
            Self::Io(err) => write!(f, "Asset I/O error: {err}"),
            Self::InvalidRoot(msg) => write!(f, "Invalid asset root: {msg}"),
            Self::InvalidFilename(msg) => write!(f, "Invalid filename: {msg}"),
            Self::SettingsError(msg) => write!(f, "Settings error: {msg}"),
            Self::FormatNotSupported(ext) => write!(f, "Format not supported: {ext}"),
            Self::FileTooLarge {
                limit_bytes,
                actual_bytes,
            } => write!(
                f,
                "File too large: {actual_bytes} bytes exceeds limit of {limit_bytes} bytes"
            ),
            Self::MetadataExtraction(msg) => write!(f, "Metadata extraction failed: {msg}"),
            Self::Database(msg) => write!(f, "Asset database error: {msg}"),
            Self::NotFound(id) => write!(f, "Asset not found: {id}"),
            Self::AssetInUse {
                asset_id,
                references,
            } => {
                write!(
                    f,
                    "Asset {asset_id} is in use by {} reference(s)",
                    references.len()
                )
            }
            Self::DeleteBlocked(msg) => write!(f, "Asset deletion blocked: {msg}"),
        }
    }
}

impl std::error::Error for AssetError {}

impl From<std::io::Error> for AssetError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}
