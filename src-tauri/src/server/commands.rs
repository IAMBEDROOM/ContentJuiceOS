use serde::Serialize;
use tauri::State;

use super::{HttpServer, SocketIoServer};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    pub port: u16,
    pub base_url: String,
}

#[tauri::command]
pub fn get_server_info(server: State<'_, HttpServer>) -> ServerInfo {
    let port = server.port();
    ServerInfo {
        port,
        base_url: format!("http://localhost:{}", port),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SocketIoInfo {
    pub port: u16,
    pub base_url: String,
    pub namespaces: Vec<String>,
}

#[tauri::command]
pub fn get_socket_io_info(server: State<'_, SocketIoServer>) -> SocketIoInfo {
    let port = server.port();
    SocketIoInfo {
        port,
        base_url: format!("http://localhost:{}", port),
        namespaces: vec!["/overlays".to_string(), "/control".to_string()],
    }
}
