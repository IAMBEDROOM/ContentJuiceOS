use std::sync::Arc;
use tokio::sync::Notify;
use tokio::time::Instant;

use super::config::PlatformRateConfig;
use super::types::{Priority, RateLimitFeedback};

/// Token bucket that enforces per-platform rate limits with a two-tier priority system.
///
/// The bucket divides its capacity into two pools:
/// - **Realtime pool** (70% by default): Reserved for chat, alerts, EventSub
/// - **Background pool** (30% by default): For analytics, cache refresh, metadata
///
/// Realtime requests can overflow into the background pool when their reserve is exhausted.
/// Background requests are strictly limited to their own pool.
#[allow(dead_code)]
pub struct TokenBucket {
    config: PlatformRateConfig,
    /// Current available tokens (can be fractional during refill calculations)
    available: f64,
    /// Last time tokens were refilled
    last_refill: Instant,
    /// Cumulative daily usage for quota-limited platforms (YouTube)
    daily_used: u32,
    /// When the daily quota resets (midnight Pacific for YouTube)
    daily_reset: Option<Instant>,
    /// Notifier to wake callers waiting for tokens
    notify: Arc<Notify>,
}

#[allow(dead_code)]
impl TokenBucket {
    pub fn new(config: PlatformRateConfig, notify: Arc<Notify>) -> Self {
        let available = config.max_tokens as f64;
        let daily_reset = config.daily_quota.map(|_| {
            // Set initial daily reset to 24h from now
            Instant::now() + std::time::Duration::from_secs(86400)
        });

        Self {
            config,
            available,
            last_refill: Instant::now(),
            daily_used: 0,
            daily_reset,
            notify,
        }
    }

    /// Attempt to consume `cost` tokens at the given priority. Returns `true` if acquired.
    pub fn try_acquire(&mut self, priority: Priority, cost: u32) -> bool {
        // Check daily quota first (YouTube)
        if let Some(quota) = self.config.daily_quota {
            if self.daily_used + cost > quota {
                return false;
            }
        }

        let cost_f = cost as f64;
        let available_for_priority = self.available_for(priority);

        if available_for_priority >= cost_f {
            self.available -= cost_f;
            self.daily_used += cost;
            true
        } else {
            false
        }
    }

    /// How many tokens are available for a given priority level.
    fn available_for(&self, priority: Priority) -> f64 {
        match priority {
            Priority::Realtime => {
                // Realtime can use ALL available tokens (reserve + overflow into background)
                self.available
            }
            Priority::Background => {
                // Background can only use tokens beyond the realtime reserve
                let realtime_reserved = self.config.realtime_capacity() as f64;
                (self.available - realtime_reserved).max(0.0)
            }
        }
    }

    /// Add tokens based on elapsed time since last refill.
    pub fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);
        let interval = self.config.refill_interval.as_secs_f64();

        if interval > 0.0 {
            let fraction = elapsed.as_secs_f64() / interval;
            let tokens_to_add = fraction * self.config.refill_amount as f64;
            self.available = (self.available + tokens_to_add).min(self.config.max_tokens as f64);
        }

        self.last_refill = now;

        // Check daily quota reset
        if let Some(reset) = self.daily_reset {
            if now >= reset {
                self.daily_used = 0;
                self.daily_reset = Some(now + std::time::Duration::from_secs(86400));
            }
        }
    }

    /// Sync bucket state with actual server-reported limits.
    pub fn update_from_feedback(&mut self, feedback: RateLimitFeedback) {
        if let Some(remaining) = feedback.remaining {
            // Trust the server's remaining count — only adjust downward to be conservative,
            // or upward if server says we have more than we think.
            self.available = remaining as f64;
        }

        if let Some(limit) = feedback.limit {
            // Server told us the actual limit; update our config
            if limit != self.config.max_tokens {
                self.config.max_tokens = limit;
                self.config.refill_amount = limit;
            }
        }

        if let Some(reset_at) = feedback.reset_at {
            self.last_refill = Instant::now();
            // Schedule next refill at the server's reset time
            let until_reset = reset_at.duration_since(Instant::now());
            self.config.refill_interval = until_reset;
        }
    }

    /// Get a reference to the notify handle (used by the service to wake waiters).
    pub fn notify(&self) -> &Arc<Notify> {
        &self.notify
    }

    /// Current available tokens (floored to integer for display).
    pub fn available_tokens(&self) -> u32 {
        self.available.floor().max(0.0) as u32
    }

    /// Max tokens for this bucket.
    pub fn max_tokens(&self) -> u32 {
        self.config.max_tokens
    }

    /// Daily quota used (for YouTube).
    pub fn daily_used(&self) -> u32 {
        self.daily_used
    }

    /// Daily quota limit (for YouTube).
    pub fn daily_quota(&self) -> Option<u32> {
        self.config.daily_quota
    }

    /// Whether the daily quota is exhausted.
    pub fn is_daily_quota_exhausted(&self) -> bool {
        match self.config.daily_quota {
            Some(quota) => self.daily_used >= quota,
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn test_config() -> PlatformRateConfig {
        PlatformRateConfig {
            max_tokens: 100,
            refill_amount: 100,
            refill_interval: Duration::from_secs(60),
            realtime_reserve_ratio: 0.7,
            daily_quota: None,
        }
    }

    fn youtube_config() -> PlatformRateConfig {
        PlatformRateConfig {
            max_tokens: 50,
            refill_amount: 50,
            refill_interval: Duration::from_secs(60),
            realtime_reserve_ratio: 0.7,
            daily_quota: Some(100),
        }
    }

    #[test]
    fn bucket_starts_full() {
        let notify = Arc::new(Notify::new());
        let bucket = TokenBucket::new(test_config(), notify);
        assert_eq!(bucket.available_tokens(), 100);
    }

    #[test]
    fn realtime_can_use_all_tokens() {
        let notify = Arc::new(Notify::new());
        let mut bucket = TokenBucket::new(test_config(), notify);
        // Realtime can consume all 100 tokens
        assert!(bucket.try_acquire(Priority::Realtime, 100));
        assert_eq!(bucket.available_tokens(), 0);
    }

    #[test]
    fn background_limited_to_its_pool() {
        let notify = Arc::new(Notify::new());
        let mut bucket = TokenBucket::new(test_config(), notify);
        // Background pool = 100 - 70 = 30 tokens
        assert!(bucket.try_acquire(Priority::Background, 30));
        assert!(!bucket.try_acquire(Priority::Background, 1));
        // But realtime can still use the remaining 70
        assert!(bucket.try_acquire(Priority::Realtime, 70));
    }

    #[test]
    fn realtime_overflows_into_background_pool() {
        let notify = Arc::new(Notify::new());
        let mut bucket = TokenBucket::new(test_config(), notify);
        // Consume 80 tokens via realtime (70 reserve + 10 from background pool)
        assert!(bucket.try_acquire(Priority::Realtime, 80));
        assert_eq!(bucket.available_tokens(), 20);
        // Background now has only 20 available, but reserve is 70, so 20 - 70 = 0 for background
        assert!(!bucket.try_acquire(Priority::Background, 1));
    }

    #[test]
    fn refill_adds_tokens_proportionally() {
        let notify = Arc::new(Notify::new());
        let mut bucket = TokenBucket::new(test_config(), notify);
        bucket.try_acquire(Priority::Realtime, 100);
        assert_eq!(bucket.available_tokens(), 0);

        // Simulate time passing (half the refill interval)
        bucket.last_refill = Instant::now() - Duration::from_secs(30);
        bucket.refill();

        // Should have ~50 tokens (half of refill_amount=100)
        let tokens = bucket.available_tokens();
        assert!(tokens >= 49 && tokens <= 51, "Expected ~50, got {tokens}");
    }

    #[test]
    fn refill_caps_at_max() {
        let notify = Arc::new(Notify::new());
        let mut bucket = TokenBucket::new(test_config(), notify);
        // Already full, refill shouldn't exceed max
        bucket.last_refill = Instant::now() - Duration::from_secs(120);
        bucket.refill();
        assert_eq!(bucket.available_tokens(), 100);
    }

    #[test]
    fn daily_quota_tracking() {
        let notify = Arc::new(Notify::new());
        let mut bucket = TokenBucket::new(youtube_config(), notify);
        // Use 90 of 100 daily quota across multiple acquires
        for _ in 0..9 {
            assert!(bucket.try_acquire(Priority::Realtime, 10));
            // Refill per-minute tokens so we don't run out of those
            bucket.available = 50.0;
        }
        assert_eq!(bucket.daily_used(), 90);
        // Can still use 10 more
        assert!(bucket.try_acquire(Priority::Realtime, 10));
        // Now at 100/100 daily — should deny
        bucket.available = 50.0;
        assert!(!bucket.try_acquire(Priority::Realtime, 1));
        assert!(bucket.is_daily_quota_exhausted());
    }

    #[test]
    fn daily_quota_resets() {
        let notify = Arc::new(Notify::new());
        let mut bucket = TokenBucket::new(youtube_config(), notify);
        // Exhaust daily quota
        for _ in 0..10 {
            bucket.try_acquire(Priority::Realtime, 10);
            bucket.available = 50.0;
        }
        assert!(bucket.is_daily_quota_exhausted());

        // Simulate daily reset passing
        bucket.daily_reset = Some(Instant::now() - Duration::from_secs(1));
        bucket.refill();

        assert_eq!(bucket.daily_used(), 0);
        assert!(!bucket.is_daily_quota_exhausted());
    }

    #[test]
    fn feedback_updates_available() {
        let notify = Arc::new(Notify::new());
        let mut bucket = TokenBucket::new(test_config(), notify);
        bucket.try_acquire(Priority::Realtime, 50);
        assert_eq!(bucket.available_tokens(), 50);

        // Server says we actually have 80 remaining
        bucket.update_from_feedback(RateLimitFeedback {
            remaining: Some(80),
            reset_at: None,
            limit: None,
        });
        assert_eq!(bucket.available_tokens(), 80);
    }

    #[test]
    fn feedback_updates_limit() {
        let notify = Arc::new(Notify::new());
        let mut bucket = TokenBucket::new(test_config(), notify);
        assert_eq!(bucket.max_tokens(), 100);

        bucket.update_from_feedback(RateLimitFeedback {
            remaining: None,
            reset_at: None,
            limit: Some(120),
        });
        assert_eq!(bucket.max_tokens(), 120);
    }
}
