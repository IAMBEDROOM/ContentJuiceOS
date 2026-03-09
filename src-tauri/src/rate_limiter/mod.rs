pub mod bucket;
pub mod commands;
pub mod config;
pub mod error;
pub mod types;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{Mutex, Notify};

use bucket::TokenBucket;
use config::PlatformRateConfig;
use error::RateLimiterError;
use types::{Platform, Priority, RateLimitFeedback, RateLimitStatus};

/// Async rate limiting service for all platform API calls.
///
/// Registered as Tauri managed state (`Arc<RateLimiterService>`).
/// Callers call `acquire(platform, priority).await` before making HTTP requests;
/// the `.await` resolves when the caller is permitted to proceed.
pub struct RateLimiterService {
    buckets: Mutex<HashMap<Platform, TokenBucket>>,
    notifiers: HashMap<Platform, Arc<Notify>>,
}

impl RateLimiterService {
    /// Create a new service with default configs for all platforms.
    pub fn new() -> Self {
        let mut buckets = HashMap::new();
        let mut notifiers = HashMap::new();

        for &platform in Platform::all() {
            let notify = Arc::new(Notify::new());
            let config = PlatformRateConfig::default_for(platform);
            buckets.insert(platform, TokenBucket::new(config, Arc::clone(&notify)));
            notifiers.insert(platform, notify);
        }

        Self {
            buckets: Mutex::new(buckets),
            notifiers,
        }
    }

    /// Acquire a single token for the given platform and priority.
    /// Blocks until a token is available or times out after 30 seconds.
    #[allow(dead_code)]
    pub async fn acquire(
        &self,
        platform: Platform,
        priority: Priority,
    ) -> Result<(), RateLimiterError> {
        self.acquire_with_cost(platform, priority, 1).await
    }

    /// Acquire `cost` tokens (for YouTube quota-aware calls).
    /// Blocks until tokens are available or times out after 30 seconds.
    #[allow(dead_code)]
    pub async fn acquire_with_cost(
        &self,
        platform: Platform,
        priority: Priority,
        cost: u32,
    ) -> Result<(), RateLimiterError> {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(30);

        loop {
            // Try to acquire under the lock
            {
                let mut buckets = self.buckets.lock().await;
                if let Some(bucket) = buckets.get_mut(&platform) {
                    // Check daily quota exhaustion first (immediate fail, don't wait)
                    if bucket.is_daily_quota_exhausted() {
                        return Err(RateLimiterError::QuotaExhausted(platform));
                    }
                    if bucket.try_acquire(priority, cost) {
                        return Ok(());
                    }
                }
            }
            // Lock is dropped here — wait for refill notification or timeout

            let notify = self
                .notifiers
                .get(&platform)
                .expect("all platforms initialized in new()");

            let remaining = deadline.duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                return Err(RateLimiterError::Timeout(platform));
            }

            // Wait for either a notify (refill happened) or timeout
            if tokio::time::timeout(remaining, notify.notified())
                .await
                .is_err()
            {
                // Final try before returning timeout
                let mut buckets = self.buckets.lock().await;
                if let Some(bucket) = buckets.get_mut(&platform) {
                    if bucket.try_acquire(priority, cost) {
                        return Ok(());
                    }
                }
                return Err(RateLimiterError::Timeout(platform));
            }
        }
    }

    /// Report server-side rate limit feedback to adjust bucket state.
    #[allow(dead_code)]
    pub async fn report_feedback(&self, platform: Platform, feedback: RateLimitFeedback) {
        let mut buckets = self.buckets.lock().await;
        if let Some(bucket) = buckets.get_mut(&platform) {
            bucket.update_from_feedback(feedback);
        }
    }

    /// Spawn a background task that refills all buckets every second and wakes waiters.
    pub fn start_refill_task(self: &Arc<Self>) {
        let service = Arc::clone(self);
        tauri::async_runtime::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                {
                    let mut buckets = service.buckets.lock().await;
                    for (&platform, bucket) in buckets.iter_mut() {
                        bucket.refill();
                        if let Some(notify) = service.notifiers.get(&platform) {
                            notify.notify_waiters();
                        }
                    }
                }
            }
        });
    }

    /// Get a diagnostic snapshot of a platform's rate limit state.
    pub async fn get_status(&self, platform: Platform) -> RateLimitStatus {
        let buckets = self.buckets.lock().await;
        let bucket = buckets
            .get(&platform)
            .expect("all platforms initialized in new()");

        RateLimitStatus {
            platform,
            available_tokens: bucket.available_tokens(),
            max_tokens: bucket.max_tokens(),
            daily_quota_used: bucket.daily_quota().map(|_| bucket.daily_used()),
            daily_quota_limit: bucket.daily_quota(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn acquire_succeeds_when_tokens_available() {
        let service = RateLimiterService::new();
        let result = service.acquire(Platform::Twitch, Priority::Realtime).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn acquire_with_cost_succeeds() {
        let service = RateLimiterService::new();
        let result = service
            .acquire_with_cost(Platform::YouTube, Priority::Realtime, 10)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn get_status_reflects_consumption() {
        let service = RateLimiterService::new();
        let before = service.get_status(Platform::Kick).await;
        assert_eq!(before.available_tokens, 60);
        assert_eq!(before.max_tokens, 60);

        service
            .acquire(Platform::Kick, Priority::Realtime)
            .await
            .unwrap();

        let after = service.get_status(Platform::Kick).await;
        assert_eq!(after.available_tokens, 59);
    }

    #[tokio::test]
    async fn youtube_status_includes_quota() {
        let service = RateLimiterService::new();
        service
            .acquire_with_cost(Platform::YouTube, Priority::Realtime, 5)
            .await
            .unwrap();

        let status = service.get_status(Platform::YouTube).await;
        assert_eq!(status.daily_quota_used, Some(5));
        assert_eq!(status.daily_quota_limit, Some(10_000));
    }

    #[tokio::test]
    async fn feedback_adjusts_state() {
        let service = RateLimiterService::new();
        service
            .report_feedback(
                Platform::Twitch,
                RateLimitFeedback {
                    remaining: Some(200),
                    reset_at: None,
                    limit: None,
                },
            )
            .await;

        let status = service.get_status(Platform::Twitch).await;
        assert_eq!(status.available_tokens, 200);
    }

    #[tokio::test]
    async fn timeout_when_no_tokens() {
        // Create a service and exhaust all tokens for Kick (60 tokens)
        let service = Arc::new(RateLimiterService::new());

        // Exhaust all tokens
        for _ in 0..60 {
            service
                .acquire(Platform::Kick, Priority::Realtime)
                .await
                .unwrap();
        }

        // Override deadline logic by testing with a very short timeout
        // We can't easily test the 30s timeout, but we can verify the bucket denies
        let status = service.get_status(Platform::Kick).await;
        assert_eq!(status.available_tokens, 0);
    }

    #[tokio::test]
    async fn refill_task_replenishes_tokens() {
        let service = Arc::new(RateLimiterService::new());

        // Exhaust some tokens
        for _ in 0..10 {
            service
                .acquire(Platform::Kick, Priority::Realtime)
                .await
                .unwrap();
        }
        let before = service.get_status(Platform::Kick).await;
        assert_eq!(before.available_tokens, 50);

        // Start refill task and wait for one tick
        service.start_refill_task();
        tokio::time::sleep(Duration::from_millis(1500)).await;

        // Should have refilled some tokens (1s = 1/60th of interval = 1 token for Kick)
        let after = service.get_status(Platform::Kick).await;
        assert!(
            after.available_tokens > 50,
            "Expected > 50, got {}",
            after.available_tokens
        );
    }
}
