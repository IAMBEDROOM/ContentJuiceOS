use super::types::Platform;
use std::time::Duration;

/// Per-platform rate limit configuration.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PlatformRateConfig {
    /// Maximum tokens the bucket can hold (also the refill ceiling)
    pub max_tokens: u32,
    /// How many tokens are added each refill cycle
    pub refill_amount: u32,
    /// How often the refill occurs
    pub refill_interval: Duration,
    /// Fraction of capacity reserved for Realtime priority (0.0–1.0)
    pub realtime_reserve_ratio: f32,
    /// Optional daily quota cap (YouTube: 10,000 units/day)
    pub daily_quota: Option<u32>,
}

#[allow(dead_code)]
impl PlatformRateConfig {
    /// Returns the default rate config for a given platform.
    ///
    /// Values are based on documented API limits:
    /// - Twitch: 800 requests per 60 seconds
    /// - YouTube: 50 requests per 60 seconds, 10,000 daily quota units
    /// - Kick: 60 requests per 60 seconds (conservative estimate)
    pub fn default_for(platform: Platform) -> Self {
        match platform {
            Platform::Twitch => Self {
                max_tokens: 800,
                refill_amount: 800,
                refill_interval: Duration::from_secs(60),
                realtime_reserve_ratio: 0.7,
                daily_quota: None,
            },
            Platform::YouTube => Self {
                max_tokens: 50,
                refill_amount: 50,
                refill_interval: Duration::from_secs(60),
                realtime_reserve_ratio: 0.7,
                daily_quota: Some(10_000),
            },
            Platform::Kick => Self {
                max_tokens: 60,
                refill_amount: 60,
                refill_interval: Duration::from_secs(60),
                realtime_reserve_ratio: 0.7,
                daily_quota: None,
            },
        }
    }

    /// Number of tokens reserved for Realtime priority.
    pub fn realtime_capacity(&self) -> u32 {
        (self.max_tokens as f32 * self.realtime_reserve_ratio).floor() as u32
    }

    /// Number of tokens available for Background priority.
    pub fn background_capacity(&self) -> u32 {
        self.max_tokens - self.realtime_capacity()
    }
}
