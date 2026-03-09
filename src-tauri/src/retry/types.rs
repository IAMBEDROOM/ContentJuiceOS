use serde::Serialize;

/// Health state for a platform's API availability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthState {
    Healthy,
    Degraded,
    Down,
}

/// Snapshot of a platform's health for frontend display.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformHealthStatus {
    pub platform: String,
    pub state: HealthState,
    pub consecutive_failures: u32,
    pub last_success: Option<String>,
    pub last_failure: Option<String>,
    pub last_error_message: Option<String>,
    pub queued_actions: usize,
}

/// Queue statistics for a platform.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueStats {
    pub platform: String,
    pub pending_count: usize,
    pub oldest_age_secs: Option<f64>,
}

/// Classification of an error for retry decision-making.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorClass {
    /// Transient errors (network blips, 5xx, 429) — worth retrying
    Transient,
    /// Permanent errors (4xx auth failures, parse errors) — do not retry
    Permanent,
}
