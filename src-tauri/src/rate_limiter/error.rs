use super::types::Platform;
use std::fmt;

#[derive(Debug)]
#[allow(dead_code)]
pub enum RateLimiterError {
    /// Timed out waiting 30s for a rate limit permit
    Timeout(Platform),
    /// YouTube daily quota has been exhausted
    QuotaExhausted(Platform),
}

impl fmt::Display for RateLimiterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Timeout(p) => write!(f, "Rate limit timeout for {p:?}: waited 30s for permit"),
            Self::QuotaExhausted(p) => write!(f, "Daily quota exhausted for {p:?}"),
        }
    }
}

impl std::error::Error for RateLimiterError {}
