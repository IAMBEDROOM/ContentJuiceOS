use serde::Serialize;
use tauri::State;

use super::HttpServer;

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
