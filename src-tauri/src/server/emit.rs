use log::warn;
use reqwest::Client;
use serde_json::Value;

/// Emit an event to the Socket.IO sidecar server via its HTTP endpoint.
///
/// This is fire-and-forget: errors are logged but never propagated,
/// so callers (like the FFmpeg queue) don't fail due to Socket.IO issues.
pub async fn emit_socket_io_event(port: u16, namespace: &str, event: &str, data: &Value) {
    let url = format!("http://127.0.0.1:{port}/emit");
    let payload = serde_json::json!({
        "namespace": namespace,
        "event": event,
        "data": data,
    });

    let result = Client::new().post(&url).json(&payload).send().await;

    if let Err(e) = result {
        warn!("Failed to emit Socket.IO event {event} on {namespace}: {e}");
    }
}
