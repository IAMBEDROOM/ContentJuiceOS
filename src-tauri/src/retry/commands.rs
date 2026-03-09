use std::sync::Arc;

use tauri::State;

use super::types::{PlatformHealthStatus, QueueStats};
use super::RetryService;
use crate::rate_limiter::RateLimiterService;
use crate::types::Platform;

/// Get health status for a specific platform.
#[tauri::command]
pub async fn get_platform_health(
    platform: String,
    retry_service: State<'_, Arc<RetryService>>,
) -> Result<PlatformHealthStatus, String> {
    let platform =
        Platform::from_str(&platform).ok_or_else(|| format!("Unknown platform: {platform}"))?;
    Ok(retry_service.get_health_status(platform).await)
}

/// Get health status for all platforms.
#[tauri::command]
pub async fn get_all_platform_health(
    retry_service: State<'_, Arc<RetryService>>,
) -> Result<Vec<PlatformHealthStatus>, String> {
    Ok(retry_service.get_all_health_statuses().await)
}

/// Get action queue statistics for a specific platform.
#[tauri::command]
pub async fn get_action_queue_stats(
    platform: String,
    retry_service: State<'_, Arc<RetryService>>,
) -> Result<QueueStats, String> {
    let platform =
        Platform::from_str(&platform).ok_or_else(|| format!("Unknown platform: {platform}"))?;
    Ok(retry_service.get_queue_stats(platform).await)
}

/// Manually drain the action queue for a platform, retrying queued actions.
/// Returns the number of successfully executed actions.
#[tauri::command]
pub async fn drain_action_queue(
    platform: String,
    retry_service: State<'_, Arc<RetryService>>,
    rate_limiter: State<'_, Arc<RateLimiterService>>,
) -> Result<u32, String> {
    let platform =
        Platform::from_str(&platform).ok_or_else(|| format!("Unknown platform: {platform}"))?;
    Ok(retry_service.drain_queue(platform, &rate_limiter).await)
}
