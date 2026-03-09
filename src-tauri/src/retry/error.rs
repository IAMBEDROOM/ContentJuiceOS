use std::fmt;

use crate::rate_limiter::error::RateLimiterError;

#[derive(Debug)]
pub enum RetryError {
    /// All retry attempts exhausted
    Exhausted {
        platform: String,
        attempts: u32,
        last_error: String,
    },
    /// Error classified as non-retryable
    Permanent { platform: String, error: String },
    /// Platform is Down; action was queued for later execution
    Queued { platform: String },
    /// Rate limiter denied the request
    RateLimited(RateLimiterError),
    /// Dequeued action was too old and discarded
    #[allow(dead_code)]
    Stale,
}

impl fmt::Display for RetryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exhausted {
                platform,
                attempts,
                last_error,
            } => write!(
                f,
                "All {attempts} retry attempts exhausted for {platform}: {last_error}"
            ),
            Self::Permanent { platform, error } => {
                write!(f, "Permanent error for {platform}: {error}")
            }
            Self::Queued { platform } => {
                write!(f, "Platform {platform} is down; action queued for retry")
            }
            Self::RateLimited(e) => write!(f, "Rate limited: {e}"),
            Self::Stale => write!(f, "Queued action expired (too old)"),
        }
    }
}

impl std::error::Error for RetryError {}

impl From<RateLimiterError> for RetryError {
    fn from(e: RateLimiterError) -> Self {
        Self::RateLimited(e)
    }
}
