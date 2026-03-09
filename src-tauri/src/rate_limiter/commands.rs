use std::sync::Arc;

use tauri::State;

use super::types::{Platform, RateLimitStatus};
use super::RateLimiterService;

#[tauri::command]
pub async fn get_rate_limit_status(
    platform: String,
    rate_limiter: State<'_, Arc<RateLimiterService>>,
) -> Result<RateLimitStatus, String> {
    let platform = match platform.to_lowercase().as_str() {
        "twitch" => Platform::Twitch,
        "youtube" => Platform::YouTube,
        "kick" => Platform::Kick,
        other => return Err(format!("Unknown platform: {other}")),
    };

    Ok(rate_limiter.get_status(platform).await)
}
