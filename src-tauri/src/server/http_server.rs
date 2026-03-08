use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use log::info;
use serde::Serialize;
use tauri::Manager;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

use crate::db::Database;

#[derive(Clone)]
struct AppState {
    port: u16,
    socket_io_port: u16,
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
    pub fn start(app_handle: tauri::AppHandle) -> Result<Self, String> {
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
) {
    let listener = match bind_with_fallback(configured_port, socket_io_port).await {
        Ok(l) => l,
        Err(e) => {
            let _ = tx.send(Err(e));
            return;
        }
    };

    let bound_port = listener.local_addr().unwrap().port();
    let router = build_router(bound_port, socket_io_port, browser_sources_path);

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

fn build_router(port: u16, socket_io_port: u16, browser_sources_path: PathBuf) -> Router {
    let state = AppState {
        port,
        socket_io_port,
    };
    Router::new()
        .route("/health", get(health_handler))
        .route("/config", get(config_handler))
        .nest_service("/browser-sources", ServeDir::new(browser_sources_path))
        .layer(CorsLayer::permissive())
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
        build_router(4848, 4849, PathBuf::from("nonexistent"))
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

        let router = build_router(4848, 4849, tmp.clone());
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
}
