pub mod commands;
pub mod config;
pub mod error;
pub mod executor;
pub mod health;
pub mod queue;
pub mod types;

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use log::{info, warn};

use config::RetryConfig;
use error::RetryError;
use executor::{classify_platform_error, retry_with_backoff};
use health::HealthTracker;
use queue::{ActionQueue, QueuedAction};
use types::HealthState;

use crate::platform::error::PlatformError;
use crate::rate_limiter::types::Priority;
use crate::rate_limiter::RateLimiterService;
use crate::types::Platform;

/// Central retry service composing health tracking, action queuing, and retry execution.
///
/// Registered as `Arc<RetryService>` in Tauri managed state. All platform API calls
/// should go through `execute()` to get automatic retries, health tracking, and
/// rate limit integration.
pub struct RetryService {
    health: HealthTracker,
    queue: ActionQueue,
    configs: HashMap<Platform, RetryConfig>,
    /// Tracks whether a health probe is already running per platform.
    probe_running: HashMap<Platform, AtomicBool>,
}

impl RetryService {
    pub fn new() -> Self {
        let mut configs = HashMap::new();
        let mut probe_running = HashMap::new();
        for &platform in Platform::all() {
            configs.insert(platform, RetryConfig::default_for(platform));
            probe_running.insert(platform, AtomicBool::new(false));
        }

        Self {
            health: HealthTracker::new(),
            queue: ActionQueue::new(),
            configs,
            probe_running,
        }
    }

    /// Execute an async operation with retry, rate limiting, and health tracking.
    ///
    /// Flow:
    /// 1. Check if platform is Down + request is Background → return Queued error
    /// 2. Acquire rate limit token
    /// 3. Run operation with exponential backoff
    /// 4. On success: record healthy, return result
    /// 5. On failure: record failure, maybe start probe, return error
    pub async fn execute<T, F, Fut>(
        self: &Arc<Self>,
        platform: Platform,
        priority: Priority,
        rate_limiter: &RateLimiterService,
        mut operation: F,
    ) -> Result<T, RetryError>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, PlatformError>>,
    {
        let config = self
            .configs
            .get(&platform)
            .expect("all platforms configured");

        // If platform is Down and this is a background task, don't even try
        let current_state = self.health.get_state(platform).await;
        if current_state == HealthState::Down && matches!(priority, Priority::Background) {
            return Err(RetryError::Queued {
                platform: platform.to_string(),
            });
        }

        // Acquire rate limit token
        rate_limiter.acquire(platform, priority).await?;

        // Execute with retries
        match retry_with_backoff(config, classify_platform_error, &mut operation).await {
            Ok(value) => {
                self.health.record_success(platform).await;
                Ok(value)
            }
            Err((err, attempts)) => {
                let err_msg = err.to_string();
                let new_state = self.health.record_failure(platform, &err_msg, config).await;

                // If we just transitioned to Down, start a health probe
                if new_state == HealthState::Down {
                    self.maybe_start_probe(platform, rate_limiter);
                }

                // Determine appropriate error
                if executor::classify_platform_error(&err) == types::ErrorClass::Permanent {
                    Err(RetryError::Permanent {
                        platform: platform.to_string(),
                        error: err_msg,
                    })
                } else {
                    Err(RetryError::Exhausted {
                        platform: platform.to_string(),
                        attempts,
                        last_error: err_msg,
                    })
                }
            }
        }
    }

    /// Execute a fire-and-forget operation, queuing it if the platform is Down.
    ///
    /// Unlike `execute()`, this accepts operations that return `Result<(), PlatformError>`
    /// and can enqueue them for later retry when the platform recovers.
    #[allow(dead_code)]
    pub async fn execute_or_queue(
        self: &Arc<Self>,
        platform: Platform,
        priority: Priority,
        rate_limiter: &RateLimiterService,
        description: String,
        operation: impl FnOnce() -> Pin<Box<dyn Future<Output = Result<(), PlatformError>> + Send>>
            + Send
            + 'static,
    ) -> Result<(), RetryError> {
        let current_state = self.health.get_state(platform).await;

        if current_state == HealthState::Down {
            // Queue instead of executing
            let action = QueuedAction {
                id: uuid::Uuid::new_v4().to_string(),
                description,
                created_at: std::time::Instant::now(),
                operation: Box::new(operation),
            };
            self.queue.enqueue(platform, action).await;
            return Err(RetryError::Queued {
                platform: platform.to_string(),
            });
        }

        let config = self
            .configs
            .get(&platform)
            .expect("all platforms configured");

        // Acquire rate limit token
        rate_limiter.acquire(platform, priority).await?;

        // Execute the FnOnce operation directly
        let result = (operation)().await;

        match result {
            Ok(()) => {
                self.health.record_success(platform).await;
                Ok(())
            }
            Err(err) => {
                let err_msg = err.to_string();
                let new_state = self.health.record_failure(platform, &err_msg, config).await;

                if new_state == HealthState::Down {
                    self.maybe_start_probe(platform, rate_limiter);
                }

                if executor::classify_platform_error(&err) == types::ErrorClass::Permanent {
                    Err(RetryError::Permanent {
                        platform: platform.to_string(),
                        error: err_msg,
                    })
                } else {
                    Err(RetryError::Exhausted {
                        platform: platform.to_string(),
                        attempts: 1,
                        last_error: err_msg,
                    })
                }
            }
        }
    }

    /// Drain the action queue for a platform, retrying each action.
    /// Returns the number of successfully executed actions.
    pub async fn drain_queue(&self, platform: Platform, rate_limiter: &RateLimiterService) -> u32 {
        let config = self
            .configs
            .get(&platform)
            .expect("all platforms configured");
        let actions = self.queue.drain(platform, config.queue_stale_secs).await;

        let total = actions.len();
        let mut successes = 0u32;

        for action in actions {
            // Acquire rate limit for each action
            if rate_limiter
                .acquire(platform, Priority::Background)
                .await
                .is_err()
            {
                warn!(
                    "Rate limit hit while draining queue for {}, stopping",
                    platform
                );
                break;
            }

            match (action.operation)().await {
                Ok(()) => {
                    successes += 1;
                    self.health.record_success(platform).await;
                }
                Err(err) => {
                    warn!(
                        "Queued action '{}' failed during drain for {}: {}",
                        action.description, platform, err
                    );
                    self.health
                        .record_failure(platform, &err.to_string(), config)
                        .await;
                }
            }
        }

        info!(
            "Drained queue for {}: {}/{} actions succeeded",
            platform, successes, total
        );
        successes
    }

    /// Get the health status for a platform, including queue count.
    pub async fn get_health_status(&self, platform: Platform) -> types::PlatformHealthStatus {
        let queued = self.queue.count(platform).await;
        self.health.get_status(platform, queued).await
    }

    /// Get health statuses for all platforms.
    pub async fn get_all_health_statuses(&self) -> Vec<types::PlatformHealthStatus> {
        let mut statuses = Vec::new();
        for &platform in Platform::all() {
            statuses.push(self.get_health_status(platform).await);
        }
        statuses
    }

    /// Get queue statistics for a platform.
    pub async fn get_queue_stats(&self, platform: Platform) -> types::QueueStats {
        self.queue.stats(platform).await
    }

    /// Start a background health probe if one isn't already running.
    fn maybe_start_probe(self: &Arc<Self>, platform: Platform, _rate_limiter: &RateLimiterService) {
        let probe_flag = self
            .probe_running
            .get(&platform)
            .expect("all platforms configured");

        // Only start if no probe is currently running
        if probe_flag
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return;
        }

        info!("Starting health probe for {}", platform);

        let service = Arc::clone(self);

        // We need to capture the probe interval from config
        let probe_interval = self
            .configs
            .get(&platform)
            .expect("all platforms configured")
            .health_probe_interval;

        tauri::async_runtime::spawn(async move {
            let mut interval = tokio::time::interval(probe_interval);
            // Skip the first immediate tick
            interval.tick().await;

            loop {
                interval.tick().await;

                // Check if platform is still Down
                let state = service.health.get_state(platform).await;
                if state != HealthState::Down {
                    info!("Health probe for {} exiting — no longer Down", platform);
                    break;
                }

                // Probe a lightweight public endpoint
                let probe_url = match platform {
                    Platform::Twitch => "https://id.twitch.tv/oauth2/validate",
                    Platform::YouTube => "https://www.googleapis.com/youtube/v3",
                    Platform::Kick => "https://kick.com/api/v2/categories",
                };

                let probe_result = reqwest::Client::new()
                    .get(probe_url)
                    .timeout(std::time::Duration::from_secs(10))
                    .send()
                    .await;

                match probe_result {
                    Ok(resp) if resp.status().is_server_error() => {
                        warn!(
                            "Health probe for {} returned {}, still Down",
                            platform,
                            resp.status()
                        );
                    }
                    Ok(_) => {
                        info!(
                            "Health probe for {} succeeded — platform is reachable",
                            platform
                        );
                        service.health.record_success(platform).await;

                        // Drain the queue (we can't easily get the rate_limiter ref here,
                        // so we just mark healthy and let the next real request trigger drain)
                        info!(
                            "Platform {} recovered. Queued actions will be drained on next request.",
                            platform
                        );
                        break;
                    }
                    Err(e) => {
                        warn!("Health probe for {} failed: {}", platform, e);
                    }
                }
            }

            // Clear the probe running flag
            if let Some(flag) = service.probe_running.get(&platform) {
                flag.store(false, Ordering::SeqCst);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn new_service_all_platforms_healthy() {
        let service = RetryService::new();
        for &platform in Platform::all() {
            let status = service.get_health_status(platform).await;
            assert_eq!(status.state, HealthState::Healthy);
            assert_eq!(status.consecutive_failures, 0);
            assert_eq!(status.queued_actions, 0);
        }
    }

    #[tokio::test]
    async fn execute_success_tracks_health() {
        let service = Arc::new(RetryService::new());
        let rate_limiter = RateLimiterService::new();

        let result = service
            .execute(
                Platform::Twitch,
                Priority::Realtime,
                &rate_limiter,
                || async { Ok::<&str, PlatformError>("success") },
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");

        let status = service.get_health_status(Platform::Twitch).await;
        assert_eq!(status.state, HealthState::Healthy);
        assert!(status.last_success.is_some());
    }

    #[tokio::test]
    async fn execute_permanent_error_no_retry() {
        let service = Arc::new(RetryService::new());
        let rate_limiter = RateLimiterService::new();

        let result: Result<(), RetryError> = service
            .execute(
                Platform::Twitch,
                Priority::Realtime,
                &rate_limiter,
                || async { Err::<(), PlatformError>(PlatformError::OAuth("invalid grant".into())) },
            )
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            RetryError::Permanent { platform, error } => {
                assert_eq!(platform, "twitch");
                assert!(error.contains("invalid grant"));
            }
            other => panic!("Expected Permanent, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn get_all_health_statuses_returns_all_platforms() {
        let service = RetryService::new();
        let statuses = service.get_all_health_statuses().await;
        assert_eq!(statuses.len(), 3);

        let platforms: Vec<&str> = statuses.iter().map(|s| s.platform.as_str()).collect();
        assert!(platforms.contains(&"twitch"));
        assert!(platforms.contains(&"youtube"));
        assert!(platforms.contains(&"kick"));
    }
}
