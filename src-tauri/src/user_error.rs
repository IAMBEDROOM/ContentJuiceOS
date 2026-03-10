//! Converts internal errors into safe, user-facing messages.
//!
//! All Tauri command handlers should use `.map_err(user_error)` or the
//! `UserFacingError` trait instead of `.map_err(|e| e.to_string())` so that
//! internal details (SQL errors, HTTP response bodies, file paths) are logged
//! server-side but never exposed to the frontend.

use std::fmt;

use log::error;

use crate::assets::error::AssetError;
use crate::cache::error::CacheError;
use crate::credentials::error::CredentialError;
use crate::db::error::DbError;
use crate::ffmpeg::error::FfmpegError;
use crate::platform::error::PlatformError;
use crate::retry::error::RetryError;

/// Extension trait: convert any error into a user-safe string while logging details.
pub trait UserFacingError<T> {
    fn map_user_err(self) -> Result<T, String>;
}

impl<T, E: Into<UserError>> UserFacingError<T> for Result<T, E> {
    fn map_user_err(self) -> Result<T, String> {
        self.map_err(|e| {
            let ue: UserError = e.into();
            error!("[internal] {}", ue.internal);
            ue.message
        })
    }
}

/// Wraps an internal error description and a safe user-facing message.
struct UserError {
    message: String,
    internal: String,
}

impl UserError {
    fn new(message: impl Into<String>, internal: impl fmt::Display) -> Self {
        Self {
            message: message.into(),
            internal: internal.to_string(),
        }
    }
}

// ── AssetError ──────────────────────────────────────────────────────────

impl From<AssetError> for UserError {
    fn from(e: AssetError) -> Self {
        let msg = match &e {
            AssetError::SourceNotFound(_) => "The selected file could not be found.".to_string(),
            AssetError::Io(_) => "A file system error occurred while managing assets.".to_string(),
            AssetError::InvalidRoot(_) => "The media directory setting is invalid.".to_string(),
            AssetError::InvalidFilename(_) => "The file has an invalid name.".to_string(),
            AssetError::SettingsError(_) => "Could not read asset settings.".to_string(),
            AssetError::FormatNotSupported(_) => {
                "This file format is not supported.".to_string()
            }
            AssetError::FileTooLarge { .. } => {
                "The file exceeds the maximum allowed size.".to_string()
            }
            AssetError::MetadataExtraction(_) => {
                "Could not read file metadata. The file may be corrupted.".to_string()
            }
            AssetError::Database(_) => {
                "A database error occurred while managing the asset.".to_string()
            }
            AssetError::NotFound(id) => format!("Asset not found: {id}"),
            AssetError::AssetInUse { .. } => {
                "This asset is in use by other items. Use force-delete to remove it anyway."
                    .to_string()
            }
            AssetError::DeleteBlocked(_) => {
                "This asset cannot be deleted because it is a project's source video. Delete the project first."
                    .to_string()
            }
        };
        UserError::new(msg, e)
    }
}

// ── PlatformError ───────────────────────────────────────────────────────

impl From<PlatformError> for UserError {
    fn from(e: PlatformError) -> Self {
        let msg = match &e {
            PlatformError::Database(_) => "An internal database error occurred.".to_string(),
            PlatformError::NotFound(what) => format!("Not found: {what}"),
            PlatformError::OAuth(_) => {
                "Authentication failed. Please try reconnecting your account.".to_string()
            }
            PlatformError::Http(_) => {
                "Could not reach the platform. Check your internet connection.".to_string()
            }
            PlatformError::HttpStatus { status, .. } => match *status {
                401 | 403 => "Authentication expired. Please reconnect your account.".to_string(),
                429 => "Too many requests. Please try again shortly.".to_string(),
                s if (500..600).contains(&s) => {
                    "The platform is experiencing issues. Please try again later.".to_string()
                }
                _ => "An unexpected platform error occurred.".to_string(),
            },
            PlatformError::InvalidState(_) => {
                "Operation is not valid in the current state.".to_string()
            }
            PlatformError::Credential(_) => {
                "Could not access stored credentials. Please try reconnecting.".to_string()
            }
            PlatformError::RateLimited(_) => {
                "Too many requests. Please try again shortly.".to_string()
            }
        };
        UserError::new(msg, e)
    }
}

// ── FfmpegError ─────────────────────────────────────────────────────────

impl From<FfmpegError> for UserError {
    fn from(e: FfmpegError) -> Self {
        let msg = match &e {
            FfmpegError::ProcessFailed { .. } => {
                "Media processing failed. The input file may be corrupted or unsupported."
                    .to_string()
            }
            FfmpegError::SpawnFailed(_) => {
                "Could not start the media processor. Please restart the app.".to_string()
            }
            FfmpegError::ProbeFailed(_) => {
                "Could not read media file information. Check that the file is valid.".to_string()
            }
            FfmpegError::JobNotFound(id) => format!("Media job not found: {id}"),
            FfmpegError::InvalidJobState { .. } => {
                "This media job cannot be modified in its current state.".to_string()
            }
            FfmpegError::InvalidCommand(_) => "Invalid media processing command.".to_string(),
        };
        UserError::new(msg, e)
    }
}

// ── DbError ─────────────────────────────────────────────────────────────

impl From<DbError> for UserError {
    fn from(e: DbError) -> Self {
        let msg = match &e {
            DbError::Backup(_) => "Backup operation failed.".to_string(),
            DbError::Settings(_) => "Could not access settings.".to_string(),
            _ => "An internal database error occurred.".to_string(),
        };
        UserError::new(msg, e)
    }
}

// ── CredentialError ─────────────────────────────────────────────────────

impl From<CredentialError> for UserError {
    fn from(e: CredentialError) -> Self {
        let msg = match &e {
            CredentialError::NotFound(_) => "Credential not found.".to_string(),
            _ => "Could not access stored credentials.".to_string(),
        };
        UserError::new(msg, e)
    }
}

// ── CacheError ──────────────────────────────────────────────────────────

impl From<CacheError> for UserError {
    fn from(e: CacheError) -> Self {
        let msg = match &e {
            CacheError::NotFound { .. } => "Cached data not found.".to_string(),
            _ => "A cache error occurred.".to_string(),
        };
        UserError::new(msg, e)
    }
}

// ── RetryError ──────────────────────────────────────────────────────────

impl From<RetryError> for UserError {
    fn from(e: RetryError) -> Self {
        let msg = match &e {
            RetryError::Exhausted { platform, .. } => {
                format!(
                    "Could not reach {platform} after multiple attempts. Please try again later."
                )
            }
            RetryError::Permanent { platform, .. } => {
                format!(
                    "A permanent error occurred with {platform}. Please reconnect your account."
                )
            }
            RetryError::Queued { platform } => {
                format!("{platform} is temporarily unavailable. Your action has been queued.")
            }
            RetryError::RateLimited(_) => {
                "Too many requests. Please try again shortly.".to_string()
            }
            RetryError::Stale => "The queued action expired and was discarded.".to_string(),
        };
        UserError::new(msg, e)
    }
}

// ── Mutex PoisonError (from .lock()) ────────────────────────────────────

impl<T> From<std::sync::PoisonError<T>> for UserError {
    fn from(e: std::sync::PoisonError<T>) -> Self {
        UserError::new(
            "An internal error occurred. Please restart the app.",
            format!("Mutex poisoned: {e}"),
        )
    }
}

// ── tauri::Error (from path resolution) ─────────────────────────────────

impl From<tauri::Error> for UserError {
    fn from(e: tauri::Error) -> Self {
        UserError::new("Could not resolve application paths.", e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_http_status_hides_body() {
        let err = PlatformError::HttpStatus {
            status: 500,
            body: "Internal server error: secret DB connection string".to_string(),
        };
        let ue: UserError = err.into();
        assert!(!ue.message.contains("secret"));
        assert!(!ue.message.contains("DB connection"));
        assert!(ue.message.contains("experiencing issues"));
    }

    #[test]
    fn ffmpeg_process_failed_hides_stderr() {
        let err = FfmpegError::ProcessFailed {
            exit_code: Some(1),
            stderr: "/home/user/.secret/file.mp4: No such file".to_string(),
        };
        let ue: UserError = err.into();
        assert!(!ue.message.contains(".secret"));
        assert!(!ue.message.contains("/home"));
    }

    #[test]
    fn credential_error_hides_internals() {
        let err = CredentialError::Keychain("keyring::Error: No password found".to_string());
        let ue: UserError = err.into();
        assert!(!ue.message.contains("keyring"));
        assert!(!ue.message.contains("password"));
    }
}
