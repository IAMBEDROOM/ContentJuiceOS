use reqwest::header::HeaderMap;
use serde::Serialize;
use tokio::time::Instant;

pub use crate::types::Platform;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Priority {
    /// Chat messages, alerts, EventSub — uses 70% reserved pool + overflow into background
    Realtime,
    /// Analytics, cache refresh, metadata sync — uses 30% background pool only
    Background,
}

/// Feedback parsed from HTTP response headers to sync bucket state with server reality.
#[derive(Debug)]
#[allow(dead_code)]
pub struct RateLimitFeedback {
    pub remaining: Option<u32>,
    pub reset_at: Option<Instant>,
    pub limit: Option<u32>,
}

#[allow(dead_code)]
impl RateLimitFeedback {
    /// Parse standard rate limit headers (Twitch-style).
    ///
    /// Looks for:
    /// - `Ratelimit-Remaining` — tokens remaining
    /// - `Ratelimit-Reset` — Unix epoch timestamp when the bucket resets
    /// - `Ratelimit-Limit` — max tokens per window
    ///
    /// Gracefully returns None fields for missing/unparseable headers.
    pub fn from_response_headers(headers: &HeaderMap) -> Self {
        let remaining = headers
            .get("ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok());

        let reset_at = headers
            .get("ratelimit-reset")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .and_then(|epoch_secs| {
                let now_epoch = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .ok()?
                    .as_secs();
                if epoch_secs > now_epoch {
                    Some(Instant::now() + std::time::Duration::from_secs(epoch_secs - now_epoch))
                } else {
                    None
                }
            });

        let limit = headers
            .get("ratelimit-limit")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok());

        Self {
            remaining,
            reset_at,
            limit,
        }
    }
}

/// Snapshot of a platform's rate limit state, for frontend diagnostics.
#[derive(Debug, Serialize)]
pub struct RateLimitStatus {
    pub platform: Platform,
    pub available_tokens: u32,
    pub max_tokens: u32,
    pub daily_quota_used: Option<u32>,
    pub daily_quota_limit: Option<u32>,
}
