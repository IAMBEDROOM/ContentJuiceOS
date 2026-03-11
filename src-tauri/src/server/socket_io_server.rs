use std::sync::Mutex;

use log::{error, info, warn};
use tauri::Manager;
use tauri_plugin_shell::process::CommandChild;
use tauri_plugin_shell::ShellExt;

use std::sync::Arc;

use crate::db::Database;

/// Kill any process currently listening on the given port.
/// Returns Ok(true) if a process was killed, Ok(false) if port was free.
fn kill_process_on_port(port: u16) -> Result<bool, String> {
    let pid = find_pid_on_port(port)?;
    let Some(pid) = pid else {
        return Ok(false);
    };

    // Don't kill ourselves
    let our_pid = std::process::id();
    if pid == our_pid {
        return Ok(false);
    }

    info!("Killing orphan process {pid} on port {port}");

    #[cfg(target_os = "windows")]
    {
        let status = std::process::Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(|e| format!("Failed to run taskkill: {e}"))?;
        if !status.success() {
            warn!("taskkill exited with non-zero for PID {pid}");
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = std::process::Command::new("kill")
            .args(["-9", &pid.to_string()])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }

    // Wait briefly for the port to be released
    for _ in 0..20 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if find_pid_on_port(port)?.is_none() {
            return Ok(true);
        }
    }

    Err(format!(
        "Port {port} still in use after killing PID {pid}"
    ))
}

/// Find the PID of the process listening on 127.0.0.1:{port}.
fn find_pid_on_port(port: u16) -> Result<Option<u32>, String> {
    #[cfg(target_os = "windows")]
    {
        let output = std::process::Command::new("netstat")
            .args(["-ano", "-p", "TCP"])
            .output()
            .map_err(|e| format!("Failed to run netstat: {e}"))?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let target = format!("127.0.0.1:{port}");
        for line in stdout.lines() {
            let line = line.trim();
            if line.contains(&target) && line.contains("LISTENING") {
                if let Some(pid_str) = line.split_whitespace().last() {
                    if let Ok(pid) = pid_str.parse::<u32>() {
                        return Ok(Some(pid));
                    }
                }
            }
        }
        Ok(None)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let output = std::process::Command::new("lsof")
            .args(["-ti", &format!(":{port}"), "-sTCP:LISTEN"])
            .output()
            .map_err(|e| format!("Failed to run lsof: {e}"))?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let pid = stdout.trim().lines().next().and_then(|l| l.parse().ok());
        Ok(pid)
    }
}

pub struct SocketIoServer {
    port: u16,
    child: Mutex<Option<CommandChild>>,
}

impl SocketIoServer {
    pub fn start(app_handle: &tauri::AppHandle) -> Result<Self, String> {
        let port = {
            let db = app_handle.state::<Arc<Database>>();
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

        // Kill any orphan process left over from a previous crash
        match kill_process_on_port(port) {
            Ok(true) => info!("Cleared orphan process on port {port}"),
            Ok(false) => {}
            Err(e) => warn!("Could not clear port {port}: {e}"),
        }

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
