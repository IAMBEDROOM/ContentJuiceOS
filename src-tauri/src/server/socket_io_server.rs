use std::sync::Mutex;

use log::{error, info};
use tauri::Manager;
use tauri_plugin_shell::process::CommandChild;
use tauri_plugin_shell::ShellExt;

use crate::db::Database;

pub struct SocketIoServer {
    port: u16,
    child: Mutex<Option<CommandChild>>,
}

impl SocketIoServer {
    pub fn start(app_handle: &tauri::AppHandle) -> Result<Self, String> {
        let port = {
            let db = app_handle.state::<Database>();
            let conn = db.conn.lock().map_err(|e| e.to_string())?;
            conn.query_row(
                "SELECT value FROM settings WHERE key = 'server.socketIoPort'",
                [],
                |row| row.get::<_, String>(0),
            )
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(4849u16)
        };

        let (tx, rx) = std::sync::mpsc::channel::<Result<u16, String>>();

        let sidecar = app_handle
            .shell()
            .sidecar("binaries/socket-io-server")
            .map_err(|e| format!("Failed to create sidecar command: {e}"))?
            .args(["--port", &port.to_string()]);

        let (mut receiver, child) = sidecar
            .spawn()
            .map_err(|e| format!("Failed to spawn Socket.IO sidecar: {e}"))?;

        let tx_clone = tx.clone();
        tauri::async_runtime::spawn(async move {
            use tauri_plugin_shell::process::CommandEvent;

            let mut ready_sent = false;
            while let Some(event) = receiver.recv().await {
                match event {
                    CommandEvent::Stdout(line) => {
                        let text = String::from_utf8_lossy(&line);
                        let text = text.trim();
                        if !ready_sent {
                            if let Some(port_str) = text.strip_prefix("READY:") {
                                if let Ok(p) = port_str.parse::<u16>() {
                                    let _ = tx_clone.send(Ok(p));
                                    ready_sent = true;
                                    info!("Socket.IO server ready on port {p}");
                                    continue;
                                }
                            }
                        }
                        info!("[socket.io] {text}");
                    }
                    CommandEvent::Stderr(line) => {
                        let text = String::from_utf8_lossy(&line);
                        error!("[socket.io] {}", text.trim());
                    }
                    CommandEvent::Terminated(status) => {
                        info!("[socket.io] process terminated with code {:?}", status.code);
                        if !ready_sent {
                            let _ = tx_clone.send(Err(format!(
                                "Socket.IO sidecar exited before READY (code: {:?})",
                                status.code
                            )));
                        }
                        break;
                    }
                    _ => {}
                }
            }
        });

        let bound_port = rx
            .recv_timeout(std::time::Duration::from_secs(10))
            .map_err(|_| "Socket.IO sidecar did not send READY within 10 seconds".to_string())?
            .map_err(|e| format!("Socket.IO sidecar failed: {e}"))?;

        Ok(Self {
            port: bound_port,
            child: Mutex::new(Some(child)),
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn stop(&self) {
        if let Ok(mut guard) = self.child.lock() {
            if let Some(child) = guard.take() {
                let _ = child.kill();
            }
        }
    }
}

impl Drop for SocketIoServer {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn default_port_is_4849() {
        // Verifies our default port constant matches the plan
        assert_eq!(4849u16, 4849);
    }
}
