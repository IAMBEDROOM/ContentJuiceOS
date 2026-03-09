use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use axum::extract::{Query, State};
use axum::response::Html;
use axum::routing::get;
use axum::{Json, Router};
use log::{error, info};
use serde::Serialize;
use tauri::Manager;
use http::{header, HeaderValue, Method};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

use crate::db::Database;
use crate::platform::kick::oauth::KickAuthState;
use crate::platform::twitch::oauth::{OAuthCallbackParams, TwitchAuthState};
use crate::platform::youtube::oauth::YouTubeAuthState;

#[derive(Clone)]
struct AppState {
    port: u16,
    socket_io_port: u16,
    auth_state: Arc<TwitchAuthState>,
    youtube_auth_state: Arc<YouTubeAuthState>,
    kick_auth_state: Arc<KickAuthState>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    port: u16,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ConfigResponse {
    http_port: u16,
    socket_io_port: u16,
}

pub struct HttpServer {
    port: u16,
    shutdown: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl HttpServer {
    pub fn start(
        app_handle: tauri::AppHandle,
        auth_state: Arc<TwitchAuthState>,
        youtube_auth_state: Arc<YouTubeAuthState>,
        kick_auth_state: Arc<KickAuthState>,
    ) -> Result<Self, String> {
        let (configured_port, socket_io_port) = {
            let db = app_handle.state::<Arc<Database>>();
            let conn = db.conn.lock().map_err(|e| e.to_string())?;

            let http_port: u16 = conn
                .query_row(
                    "SELECT value FROM settings WHERE key = 'server.httpPort'",
                    [],
                    |row| row.get::<_, String>(0),
                )
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(4848);

            let sio_port: u16 = conn
                .query_row(
                    "SELECT value FROM settings WHERE key = 'server.socketIoPort'",
                    [],
                    |row| row.get::<_, String>(0),
                )
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(4849);

            (http_port, sio_port)
        };

        let browser_sources_path = app_handle
            .path()
            .resource_dir()
            .map(|d| d.join("browser-sources"))
            .unwrap_or_else(|_| PathBuf::from("browser-sources"));

        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_flag = shutdown.clone();

        let (tx, rx) = std::sync::mpsc::channel::<Result<u16, String>>();

        let handle = thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
            rt.block_on(run_server(
                configured_port,
                socket_io_port,
                browser_sources_path,
                shutdown_flag,
                tx,
                auth_state,
                youtube_auth_state,
                kick_auth_state,
            ));
        });

        let bound_port = rx
            .recv_timeout(Duration::from_secs(5))
            .map_err(|_| "HTTP server failed to start within 5 seconds".to_string())?
            .map_err(|e| format!("HTTP server bind failed: {e}"))?;

        Ok(Self {
            port: bound_port,
            shutdown,
            handle: Some(handle),
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn stop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for HttpServer {
    fn drop(&mut self) {
        self.stop();
    }
}

async fn run_server(
    configured_port: u16,
    socket_io_port: u16,
    browser_sources_path: PathBuf,
    shutdown: Arc<AtomicBool>,
    tx: std::sync::mpsc::Sender<Result<u16, String>>,
    auth_state: Arc<TwitchAuthState>,
    youtube_auth_state: Arc<YouTubeAuthState>,
    kick_auth_state: Arc<KickAuthState>,
) {
    let listener = match bind_with_fallback(configured_port, socket_io_port).await {
        Ok(l) => l,
        Err(e) => {
            let _ = tx.send(Err(e));
            return;
        }
    };

    let bound_port = listener.local_addr().unwrap().port();
    let router = build_router(
        bound_port,
        socket_io_port,
        browser_sources_path,
        auth_state,
        youtube_auth_state,
        kick_auth_state,
    );

    info!("HTTP server listening on http://127.0.0.1:{}", bound_port);
    let _ = tx.send(Ok(bound_port));

    let _ = axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal(shutdown))
        .await;
}

async fn bind_with_fallback(
    configured_port: u16,
    socket_io_port: u16,
) -> Result<tokio::net::TcpListener, String> {
    for offset in 0..=20u16 {
        let port = configured_port.wrapping_add(offset);
        if port == socket_io_port {
            continue;
        }
        match tokio::net::TcpListener::bind(("127.0.0.1", port)).await {
            Ok(listener) => return Ok(listener),
            Err(_) => continue,
        }
    }
    Err(format!(
        "Failed to bind HTTP server on ports {}-{}",
        configured_port,
        configured_port + 20
    ))
}

/// Simple token-bucket rate limiter: 100 requests per 60 seconds.
async fn rate_limit_middleware(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::UNIX_EPOCH;

    static TOKENS: AtomicU64 = AtomicU64::new(100);
    static WINDOW_START: AtomicU64 = AtomicU64::new(0);

    let now = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let window = WINDOW_START.load(Ordering::Relaxed);
    if now >= window + 60 {
        TOKENS.store(100, Ordering::Relaxed);
        WINDOW_START.store(now, Ordering::Relaxed);
    }

    if TOKENS.load(Ordering::Relaxed) == 0 {
        return axum::response::Response::builder()
            .status(429)
            .header("Retry-After", "60")
            .body(axum::body::Body::from("Too Many Requests"))
            .unwrap();
    }

    TOKENS.fetch_sub(1, Ordering::Relaxed);
    next.run(request).await
}

/// Add security headers to all responses.
async fn security_headers_middleware(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let is_sensitive = request
        .uri()
        .path()
        .starts_with("/health")
        || request.uri().path().starts_with("/config")
        || request.uri().path().starts_with("/auth/");

    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "SAMEORIGIN".parse().unwrap());
    headers.insert("Referrer-Policy", "no-referrer".parse().unwrap());

    if is_sensitive {
        headers.insert("Cache-Control", "no-store".parse().unwrap());
    }

    response
}

fn build_cors_layer(port: u16) -> CorsLayer {
    let origins: Vec<HeaderValue> = [
        format!("http://127.0.0.1:{port}"),
        format!("http://localhost:{port}"),
        "http://localhost:1420".to_string(),  // Tauri dev server
        "tauri://localhost".to_string(),      // Tauri production (macOS/Linux)
        "https://tauri.localhost".to_string(), // Tauri production (Windows)
    ]
    .into_iter()
    .filter_map(|o| o.parse::<HeaderValue>().ok())
    .collect();

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET])
        .allow_headers([header::CONTENT_TYPE, header::ACCEPT])
}

fn build_router(
    port: u16,
    socket_io_port: u16,
    browser_sources_path: PathBuf,
    auth_state: Arc<TwitchAuthState>,
    youtube_auth_state: Arc<YouTubeAuthState>,
    kick_auth_state: Arc<KickAuthState>,
) -> Router {
    let state = AppState {
        port,
        socket_io_port,
        auth_state,
        youtube_auth_state,
        kick_auth_state,
    };
    Router::new()
        .route("/health", get(health_handler))
        .route("/config", get(config_handler))
        .route("/auth/callback/twitch", get(twitch_callback_handler))
        .route("/auth/callback/youtube", get(youtube_callback_handler))
        .route("/auth/callback/kick", get(kick_callback_handler))
        .nest_service("/browser-sources", ServeDir::new(browser_sources_path))
        .layer(build_cors_layer(port))
        .layer(axum::middleware::from_fn(rate_limit_middleware))
        .layer(axum::middleware::from_fn(security_headers_middleware))
        .with_state(state)
}

async fn health_handler(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        port: state.port,
    })
}

async fn config_handler(State(state): State<AppState>) -> Json<ConfigResponse> {
    Json(ConfigResponse {
        http_port: state.port,
        socket_io_port: state.socket_io_port,
    })
}

async fn twitch_callback_handler(
    Query(params): Query<OAuthCallbackParams>,
    State(state): State<AppState>,
) -> Html<String> {
    let (title, message, accent_color) = if params.error.is_some() {
        let error_desc = params
            .error_description
            .as_deref()
            .unwrap_or("Authorization was denied or an error occurred.");
        let _ = state
            .auth_state
            .complete_pending(Err(error_desc.to_string()))
            .await;
        (
            "Authorization Failed",
            format!("Twitch authorization was not completed: {error_desc}"),
            "#FF007F", // Hyper Magenta
        )
    } else {
        if let Err(e) = state.auth_state.complete_pending(Ok(params)).await {
            error!("Failed to complete OAuth flow: {e}");
            return Html(callback_html(
                "Authorization Error",
                "An internal error occurred. Please try again.",
                "#FF007F",
            ));
        }
        (
            "Authorization Successful",
            "You have been connected to Twitch! You can close this tab and return to ContentJuiceOS."
                .to_string(),
            "#00E5FF", // Electric Cyan
        )
    };

    Html(callback_html(title, &message, accent_color))
}

async fn youtube_callback_handler(
    Query(params): Query<OAuthCallbackParams>,
    State(state): State<AppState>,
) -> Html<String> {
    let (title, message, accent_color) = if params.error.is_some() {
        let error_desc = params
            .error_description
            .as_deref()
            .unwrap_or("Authorization was denied or an error occurred.");
        let _ = state
            .youtube_auth_state
            .complete_pending(Err(error_desc.to_string()))
            .await;
        (
            "Authorization Failed",
            format!("YouTube authorization was not completed: {error_desc}"),
            "#FF007F", // Hyper Magenta
        )
    } else {
        if let Err(e) = state.youtube_auth_state.complete_pending(Ok(params)).await {
            error!("Failed to complete YouTube OAuth flow: {e}");
            return Html(callback_html(
                "Authorization Error",
                "An internal error occurred. Please try again.",
                "#FF007F",
            ));
        }
        (
            "Authorization Successful",
            "You have been connected to YouTube! You can close this tab and return to ContentJuiceOS."
                .to_string(),
            "#00E5FF", // Electric Cyan
        )
    };

    Html(callback_html(title, &message, accent_color))
}

async fn kick_callback_handler(
    Query(params): Query<OAuthCallbackParams>,
    State(state): State<AppState>,
) -> Html<String> {
    let (title, message, accent_color) = if params.error.is_some() {
        let error_desc = params
            .error_description
            .as_deref()
            .unwrap_or("Authorization was denied or an error occurred.");
        let _ = state
            .kick_auth_state
            .complete_pending(Err(error_desc.to_string()))
            .await;
        (
            "Authorization Failed",
            format!("Kick authorization was not completed: {error_desc}"),
            "#FF007F", // Hyper Magenta
        )
    } else {
        if let Err(e) = state.kick_auth_state.complete_pending(Ok(params)).await {
            error!("Failed to complete Kick OAuth flow: {e}");
            return Html(callback_html(
                "Authorization Error",
                "An internal error occurred. Please try again.",
                "#FF007F",
            ));
        }
        (
            "Authorization Successful",
            "You have been connected to Kick! You can close this tab and return to ContentJuiceOS."
                .to_string(),
            "#00E5FF", // Electric Cyan
        )
    };

    Html(callback_html(title, &message, accent_color))
}

fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

fn callback_html(title: &str, message: &str, accent_color: &str) -> String {
    let title = html_escape(title);
    let message = html_escape(message);
    let accent_color = html_escape(accent_color);
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>ContentJuiceOS — {title}</title>
    <style>
        body {{
            margin: 0;
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
            background-color: #0A0D14;
            color: #E6EDF3;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        }}
        .card {{
            text-align: center;
            padding: 48px;
            background-color: #151A26;
            border-radius: 12px;
            border: 1px solid {accent_color}33;
            max-width: 420px;
        }}
        h1 {{
            color: {accent_color};
            font-size: 1.5rem;
            margin-bottom: 16px;
        }}
        p {{
            color: #E6EDF3;
            line-height: 1.6;
            opacity: 0.85;
        }}
    </style>
</head>
<body>
    <div class="card">
        <h1>{title}</h1>
        <p>{message}</p>
    </div>
</body>
</html>"#
    )
}

async fn shutdown_signal(shutdown: Arc<AtomicBool>) {
    let mut interval = tokio::time::interval(Duration::from_millis(500));
    loop {
        interval.tick().await;
        if shutdown.load(Ordering::Relaxed) {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use http::Request;
    use std::io::Write as _;
    use tower::ServiceExt;

    fn test_router() -> Router {
        build_router(
            4848,
            4849,
            PathBuf::from("nonexistent"),
            Arc::new(TwitchAuthState::new()),
            Arc::new(YouTubeAuthState::new()),
            Arc::new(KickAuthState::new()),
        )
    }

    #[tokio::test]
    async fn health_endpoint_returns_ok() {
        let router = test_router();
        let request = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), 200);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "ok");
        assert_eq!(json["port"], 4848);
    }

    #[tokio::test]
    async fn unknown_route_returns_404() {
        let router = test_router();
        let request = Request::builder()
            .uri("/nonexistent")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), 404);
    }

    #[tokio::test]
    async fn config_endpoint_returns_ports() {
        let router = test_router();
        let request = Request::builder()
            .uri("/config")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), 200);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["httpPort"], 4848);
        assert_eq!(json["socketIoPort"], 4849);
    }

    #[tokio::test]
    async fn browser_sources_serves_static_files() {
        let tmp = std::env::temp_dir().join("cjos_test_browser_sources");
        let _ = std::fs::create_dir_all(&tmp);
        let test_file = tmp.join("test.html");
        let mut f = std::fs::File::create(&test_file).unwrap();
        f.write_all(b"<h1>test</h1>").unwrap();

        let router = build_router(
            4848,
            4849,
            tmp.clone(),
            Arc::new(TwitchAuthState::new()),
            Arc::new(YouTubeAuthState::new()),
            Arc::new(KickAuthState::new()),
        );
        let request = Request::builder()
            .uri("/browser-sources/test.html")
            .body(Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        assert_eq!(response.status(), 200);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(body.as_ref(), b"<h1>test</h1>");

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[tokio::test]
    async fn port_fallback_skips_occupied_port() {
        let blocker = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let occupied_port = blocker.local_addr().unwrap().port();

        let listener = bind_with_fallback(occupied_port, 0).await.unwrap();
        let bound_port = listener.local_addr().unwrap().port();
        assert_eq!(bound_port, occupied_port + 1);
    }

    #[tokio::test]
    async fn port_fallback_skips_socket_io_port() {
        let blocker = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let occupied_port = blocker.local_addr().unwrap().port();
        let socket_io_port = occupied_port + 1;

        let listener = bind_with_fallback(occupied_port, socket_io_port)
            .await
            .unwrap();
        let bound_port = listener.local_addr().unwrap().port();
        assert_eq!(bound_port, occupied_port + 2);
    }

    #[tokio::test]
    async fn port_fallback_exhaustion_returns_error() {
        // Bind 21 consecutive ports starting from an ephemeral port
        let first = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let start_port = first.local_addr().unwrap().port();

        let mut blockers = vec![first];
        for offset in 1..=20u16 {
            if let Ok(l) = std::net::TcpListener::bind(("127.0.0.1", start_port + offset)) {
                blockers.push(l);
            }
        }

        let result = bind_with_fallback(start_port, 0).await;
        assert!(result.is_err());

        drop(blockers);
    }

    #[test]
    fn html_escape_prevents_xss() {
        let malicious = "<script>alert('xss')</script>";
        let escaped = html_escape(malicious);
        assert!(!escaped.contains('<'));
        assert!(!escaped.contains('>'));
        assert!(escaped.contains("&lt;script&gt;"));
    }

    #[test]
    fn callback_html_escapes_all_params() {
        let html = callback_html(
            "<b>title</b>",
            "<img src=x onerror=alert(1)>",
            "red; background: url(evil)",
        );
        assert!(!html.contains("<b>title</b>"));
        assert!(!html.contains("<img src=x"));
        assert!(html.contains("&lt;b&gt;title&lt;/b&gt;"));
        assert!(html.contains("&lt;img src=x"));
    }
}
