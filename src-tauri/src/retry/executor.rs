use std::future::Future;
use std::time::Duration;

use rand::Rng;

use super::config::RetryConfig;
use super::types::ErrorClass;
use crate::platform::error::PlatformError;

/// Execute an async operation with exponential backoff and full jitter.
///
/// Uses the "Full Jitter" algorithm recommended by AWS:
///   cap = min(max_delay, base_delay * 2^attempt)
///   sleep = random_uniform(0, cap)
///
/// Returns `Ok(T)` on success, or `Err((last_error, attempts_made))` if all
/// retries are exhausted or a permanent error is encountered.
pub async fn retry_with_backoff<T, E, F, Fut>(
    config: &RetryConfig,
    classify: impl Fn(&E) -> ErrorClass,
    mut operation: F,
) -> Result<T, (E, u32)>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    let mut attempt = 0u32;

    loop {
        match operation().await {
            Ok(value) => return Ok(value),
            Err(err) => {
                attempt += 1;

                // Permanent errors are never retried
                if classify(&err) == ErrorClass::Permanent {
                    return Err((err, attempt));
                }

                // All retries exhausted
                if attempt > config.max_retries {
                    return Err((err, attempt));
                }

                // Full jitter backoff
                let cap = std::cmp::min(
                    config.max_delay,
                    config.base_delay.saturating_mul(1u32 << (attempt - 1)),
                );
                let sleep_ms = rand::thread_rng().gen_range(0..=cap.as_millis() as u64);
                tokio::time::sleep(Duration::from_millis(sleep_ms)).await;
            }
        }
    }
}

/// Classify a `PlatformError` for retry decision-making.
///
/// Transient errors (worth retrying):
///   - HTTP 429 (rate limited by server), 500, 502, 503, 504
///   - Connection-level errors (no status code)
///
/// Permanent errors (not retryable):
///   - HTTP 400, 401, 403, 404, 409
///   - OAuth/parse/credential/database/state errors
pub fn classify_platform_error(err: &PlatformError) -> ErrorClass {
    match err {
        PlatformError::HttpStatus { status, .. } => match *status {
            429 | 500 | 502 | 503 | 504 => ErrorClass::Transient,
            _ => ErrorClass::Permanent,
        },
        PlatformError::Http(_) => ErrorClass::Transient,
        PlatformError::OAuth(_)
        | PlatformError::Database(_)
        | PlatformError::Credential(_)
        | PlatformError::NotFound(_)
        | PlatformError::RateLimited(_)
        | PlatformError::InvalidState(_) => ErrorClass::Permanent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn succeeds_on_first_attempt() {
        let config = RetryConfig::default_for(crate::types::Platform::Twitch);
        let result = retry_with_backoff(
            &config,
            |_: &String| ErrorClass::Transient,
            || async { Ok::<&str, String>("success") },
        )
        .await;

        assert_eq!(result.unwrap(), "success");
    }

    #[tokio::test]
    async fn retries_transient_then_succeeds() {
        let config = RetryConfig {
            base_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(10),
            max_retries: 4,
            degraded_threshold: 3,
            down_threshold: 8,
            queue_stale_secs: 300,
            health_probe_interval: Duration::from_secs(30),
        };

        let call_count = Arc::new(AtomicU32::new(0));
        let cc = Arc::clone(&call_count);

        let result = retry_with_backoff(
            &config,
            |_: &String| ErrorClass::Transient,
            move || {
                let cc = Arc::clone(&cc);
                async move {
                    let n = cc.fetch_add(1, Ordering::SeqCst);
                    if n < 2 {
                        Err::<&str, String>("transient".into())
                    } else {
                        Ok("recovered")
                    }
                }
            },
        )
        .await;

        assert_eq!(result.unwrap(), "recovered");
        assert_eq!(call_count.load(Ordering::SeqCst), 3); // 2 failures + 1 success
    }

    #[tokio::test]
    async fn permanent_error_stops_immediately() {
        let config = RetryConfig {
            base_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(10),
            max_retries: 4,
            degraded_threshold: 3,
            down_threshold: 8,
            queue_stale_secs: 300,
            health_probe_interval: Duration::from_secs(30),
        };

        let call_count = Arc::new(AtomicU32::new(0));
        let cc = Arc::clone(&call_count);

        let result = retry_with_backoff(
            &config,
            |_: &String| ErrorClass::Permanent,
            move || {
                let cc = Arc::clone(&cc);
                async move {
                    cc.fetch_add(1, Ordering::SeqCst);
                    Err::<&str, String>("permanent".into())
                }
            },
        )
        .await;

        assert!(result.is_err());
        let (err, attempts) = result.unwrap_err();
        assert_eq!(err, "permanent");
        assert_eq!(attempts, 1); // Only one attempt, no retries
    }

    #[tokio::test]
    async fn exhausts_all_retries() {
        let config = RetryConfig {
            base_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(5),
            max_retries: 2,
            degraded_threshold: 3,
            down_threshold: 8,
            queue_stale_secs: 300,
            health_probe_interval: Duration::from_secs(30),
        };

        let call_count = Arc::new(AtomicU32::new(0));
        let cc = Arc::clone(&call_count);

        let result = retry_with_backoff(
            &config,
            |_: &String| ErrorClass::Transient,
            move || {
                let cc = Arc::clone(&cc);
                async move {
                    cc.fetch_add(1, Ordering::SeqCst);
                    Err::<&str, String>("still failing".into())
                }
            },
        )
        .await;

        assert!(result.is_err());
        let (_, attempts) = result.unwrap_err();
        assert_eq!(attempts, 3); // 1 initial + 2 retries
    }

    #[test]
    fn classify_http_status_codes() {
        // Transient
        assert_eq!(
            classify_platform_error(&PlatformError::HttpStatus {
                status: 429,
                body: String::new()
            }),
            ErrorClass::Transient
        );
        assert_eq!(
            classify_platform_error(&PlatformError::HttpStatus {
                status: 503,
                body: String::new()
            }),
            ErrorClass::Transient
        );
        assert_eq!(
            classify_platform_error(&PlatformError::HttpStatus {
                status: 500,
                body: String::new()
            }),
            ErrorClass::Transient
        );

        // Permanent
        assert_eq!(
            classify_platform_error(&PlatformError::HttpStatus {
                status: 401,
                body: String::new()
            }),
            ErrorClass::Permanent
        );
        assert_eq!(
            classify_platform_error(&PlatformError::HttpStatus {
                status: 403,
                body: String::new()
            }),
            ErrorClass::Permanent
        );
        assert_eq!(
            classify_platform_error(&PlatformError::HttpStatus {
                status: 404,
                body: String::new()
            }),
            ErrorClass::Permanent
        );
    }

    #[test]
    fn classify_connection_errors_as_transient() {
        assert_eq!(
            classify_platform_error(&PlatformError::Http("connection reset".into())),
            ErrorClass::Transient
        );
    }

    #[test]
    fn classify_oauth_errors_as_permanent() {
        assert_eq!(
            classify_platform_error(&PlatformError::OAuth("invalid grant".into())),
            ErrorClass::Permanent
        );
    }
}
