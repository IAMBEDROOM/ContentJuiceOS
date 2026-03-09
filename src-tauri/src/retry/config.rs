use std::time::Duration;

use crate::types::Platform;

/// Configuration for retry behavior, per platform.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Initial backoff delay before first retry
    pub base_delay: Duration,
    /// Maximum backoff delay cap
    pub max_delay: Duration,
    /// Maximum number of retries (total attempts = max_retries + 1)
    pub max_retries: u32,
    /// Consecutive failures to enter Degraded state
    pub degraded_threshold: u32,
    /// Consecutive failures to enter Down state
    pub down_threshold: u32,
    /// Seconds after which a queued action is considered stale
    pub queue_stale_secs: u64,
    /// Interval between health probe attempts when a platform is Down
    pub health_probe_interval: Duration,
}

impl RetryConfig {
    /// Returns the default retry configuration for a given platform.
    ///
    /// YouTube is more conservative due to its stricter daily quota system.
    pub fn default_for(platform: Platform) -> Self {
        match platform {
            Platform::Twitch => Self {
                base_delay: Duration::from_millis(500),
                max_delay: Duration::from_secs(30),
                max_retries: 4,
                degraded_threshold: 3,
                down_threshold: 8,
                queue_stale_secs: 300,
                health_probe_interval: Duration::from_secs(30),
            },
            Platform::YouTube => Self {
                base_delay: Duration::from_millis(1000),
                max_delay: Duration::from_secs(60),
                max_retries: 3,
                degraded_threshold: 3,
                down_threshold: 8,
                queue_stale_secs: 300,
                health_probe_interval: Duration::from_secs(30),
            },
            Platform::Kick => Self {
                base_delay: Duration::from_millis(500),
                max_delay: Duration::from_secs(30),
                max_retries: 4,
                degraded_threshold: 3,
                down_threshold: 8,
                queue_stale_secs: 300,
                health_probe_interval: Duration::from_secs(30),
            },
        }
    }
}
