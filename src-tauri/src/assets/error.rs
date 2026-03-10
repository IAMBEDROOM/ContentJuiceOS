use std::fmt;

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
}

impl fmt::Display for AssetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceNotFound(path) => write!(f, "Source file not found: {path}"),
            Self::Io(err) => write!(f, "Asset I/O error: {err}"),
            Self::InvalidRoot(msg) => write!(f, "Invalid asset root: {msg}"),
            Self::InvalidFilename(msg) => write!(f, "Invalid filename: {msg}"),
            Self::SettingsError(msg) => write!(f, "Settings error: {msg}"),
        }
    }
}

impl std::error::Error for AssetError {}

impl From<std::io::Error> for AssetError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}
