use std::collections::HashMap;

use chrono::{DateTime, Utc};
use tokio::sync::Mutex;

use super::config::RetryConfig;
use super::types::{HealthState, PlatformHealthStatus};
use crate::types::Platform;

/// Internal per-platform health tracking data.
struct PlatformHealth {
    state: HealthState,
    consecutive_failures: u32,
    last_success: Option<DateTime<Utc>>,
    last_failure: Option<DateTime<Utc>>,
    last_error_message: Option<String>,
}

impl PlatformHealth {
    fn new() -> Self {
        Self {
            state: HealthState::Healthy,
            consecutive_failures: 0,
            last_success: None,
            last_failure: None,
            last_error_message: None,
        }
    }
}

/// Tracks the health state of each platform using a simple state machine.
///
/// State transitions:
///   Healthy → Degraded (after `degraded_threshold` consecutive failures)
///   Degraded → Down (after `down_threshold` consecutive failures)
///   Any → Healthy (on any success, resets failure counter)
pub struct HealthTracker {
    states: Mutex<HashMap<Platform, PlatformHealth>>,
}

impl HealthTracker {
    pub fn new() -> Self {
        let mut states = HashMap::new();
        for &platform in Platform::all() {
            states.insert(platform, PlatformHealth::new());
        }
        Self {
            states: Mutex::new(states),
        }
    }

    /// Record a successful API call. Resets the platform to Healthy.
    pub async fn record_success(&self, platform: Platform) {
        let mut states = self.states.lock().await;
        let health = states.entry(platform).or_insert_with(PlatformHealth::new);
        health.state = HealthState::Healthy;
        health.consecutive_failures = 0;
        health.last_success = Some(Utc::now());
        health.last_error_message = None;
    }

    /// Record a failed API call. Returns the new health state after transition.
    pub async fn record_failure(
        &self,
        platform: Platform,
        error_msg: &str,
        config: &RetryConfig,
    ) -> HealthState {
        let mut states = self.states.lock().await;
        let health = states.entry(platform).or_insert_with(PlatformHealth::new);

        health.consecutive_failures += 1;
        health.last_failure = Some(Utc::now());
        health.last_error_message = Some(error_msg.to_string());

        // Transition state based on thresholds
        if health.consecutive_failures >= config.down_threshold {
            health.state = HealthState::Down;
        } else if health.consecutive_failures >= config.degraded_threshold {
            health.state = HealthState::Degraded;
        }

        health.state
    }

    /// Get the current health state for a platform.
    pub async fn get_state(&self, platform: Platform) -> HealthState {
        let states = self.states.lock().await;
        states
            .get(&platform)
            .map(|h| h.state)
            .unwrap_or(HealthState::Healthy)
    }

    /// Get a full status snapshot for a platform (for frontend/commands).
    pub async fn get_status(
        &self,
        platform: Platform,
        queued_actions: usize,
    ) -> PlatformHealthStatus {
        let states = self.states.lock().await;
        let health = states.get(&platform);

        match health {
            Some(h) => PlatformHealthStatus {
                platform: platform.as_str().to_string(),
                state: h.state,
                consecutive_failures: h.consecutive_failures,
                last_success: h.last_success.map(|dt| dt.to_rfc3339()),
                last_failure: h.last_failure.map(|dt| dt.to_rfc3339()),
                last_error_message: h.last_error_message.clone(),
                queued_actions,
            },
            None => PlatformHealthStatus {
                platform: platform.as_str().to_string(),
                state: HealthState::Healthy,
                consecutive_failures: 0,
                last_success: None,
                last_failure: None,
                last_error_message: None,
                queued_actions: 0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> RetryConfig {
        RetryConfig {
            base_delay: std::time::Duration::from_millis(100),
            max_delay: std::time::Duration::from_secs(1),
            max_retries: 3,
            degraded_threshold: 3,
            down_threshold: 8,
            queue_stale_secs: 300,
            health_probe_interval: std::time::Duration::from_secs(30),
        }
    }

    #[tokio::test]
    async fn starts_healthy() {
        let tracker = HealthTracker::new();
        assert_eq!(
            tracker.get_state(Platform::Twitch).await,
            HealthState::Healthy
        );
    }

    #[tokio::test]
    async fn transitions_to_degraded() {
        let tracker = HealthTracker::new();
        let config = test_config();

        for i in 0..3 {
            let state = tracker
                .record_failure(Platform::Twitch, &format!("error {i}"), &config)
                .await;
            if i < 2 {
                assert_eq!(state, HealthState::Healthy);
            } else {
                assert_eq!(state, HealthState::Degraded);
            }
        }
    }

    #[tokio::test]
    async fn transitions_to_down() {
        let tracker = HealthTracker::new();
        let config = test_config();

        for _ in 0..8 {
            tracker
                .record_failure(Platform::Twitch, "error", &config)
                .await;
        }

        assert_eq!(tracker.get_state(Platform::Twitch).await, HealthState::Down);
    }

    #[tokio::test]
    async fn success_resets_to_healthy() {
        let tracker = HealthTracker::new();
        let config = test_config();

        // Drive to Down
        for _ in 0..8 {
            tracker
                .record_failure(Platform::Twitch, "error", &config)
                .await;
        }
        assert_eq!(tracker.get_state(Platform::Twitch).await, HealthState::Down);

        // Single success resets everything
        tracker.record_success(Platform::Twitch).await;
        assert_eq!(
            tracker.get_state(Platform::Twitch).await,
            HealthState::Healthy
        );

        // Verify failure count was reset
        let status = tracker.get_status(Platform::Twitch, 0).await;
        assert_eq!(status.consecutive_failures, 0);
        assert!(status.last_success.is_some());
    }

    #[tokio::test]
    async fn platforms_are_independent() {
        let tracker = HealthTracker::new();
        let config = test_config();

        // Degrade Twitch
        for _ in 0..5 {
            tracker
                .record_failure(Platform::Twitch, "error", &config)
                .await;
        }

        // YouTube should still be healthy
        assert_eq!(
            tracker.get_state(Platform::YouTube).await,
            HealthState::Healthy
        );
        assert_eq!(
            tracker.get_state(Platform::Twitch).await,
            HealthState::Degraded
        );
    }

    #[tokio::test]
    async fn status_snapshot_includes_error_details() {
        let tracker = HealthTracker::new();
        let config = test_config();

        tracker
            .record_failure(Platform::Kick, "connection refused", &config)
            .await;

        let status = tracker.get_status(Platform::Kick, 5).await;
        assert_eq!(status.platform, "kick");
        assert_eq!(status.consecutive_failures, 1);
        assert_eq!(
            status.last_error_message.as_deref(),
            Some("connection refused")
        );
        assert!(status.last_failure.is_some());
        assert_eq!(status.queued_actions, 5);
    }
}
