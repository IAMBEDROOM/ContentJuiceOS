use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use log::{debug, error, info};
use tauri::AppHandle;
use tauri_plugin_shell::process::CommandChild;
use tauri_plugin_shell::ShellExt;
use tokio::sync::{mpsc, Mutex};

use crate::ffmpeg::error::FfmpegError;
use crate::ffmpeg::types::FfmpegProgress;

/// Execute an FFmpeg command as a sidecar process, parsing progress output
/// and supporting cancellation.
///
/// Progress is reported via the `progress_tx` channel. The `cancel_flag`
/// can be set to `true` to kill the process mid-run. The `CommandChild`
/// is stored in `child_handle` so external code can also kill it.
pub async fn execute_ffmpeg(
    app_handle: &AppHandle,
    job_id: String,
    args: Vec<String>,
    duration_ms: Option<u64>,
    progress_tx: mpsc::Sender<FfmpegProgress>,
    cancel_flag: Arc<AtomicBool>,
    child_handle: Arc<Mutex<Option<CommandChild>>>,
) -> Result<(), FfmpegError> {
    let sidecar = app_handle
        .shell()
        .sidecar("binaries/ffmpeg")
        .map_err(|e| FfmpegError::SpawnFailed(format!("Failed to create FFmpeg sidecar: {e}")))?
        .args(&args);

    let (mut receiver, child) = sidecar
        .spawn()
        .map_err(|e| FfmpegError::SpawnFailed(format!("Failed to spawn FFmpeg: {e}")))?;

    // Store child so external code (cancel) can kill it
    {
        let mut handle = child_handle.lock().await;
        *handle = Some(child);
    }

    info!("FFmpeg job {job_id} started");

    let mut current_progress = FfmpegProgress {
        job_id: job_id.clone(),
        ..Default::default()
    };
    let mut stderr_buf = String::new();

    use tauri_plugin_shell::process::CommandEvent;
    loop {
        // Check cancellation
        if cancel_flag.load(Ordering::Relaxed) {
            info!("FFmpeg job {job_id} cancelled, killing process");
            let mut handle = child_handle.lock().await;
            if let Some(child) = handle.take() {
                let _ = child.kill();
            }
            return Err(FfmpegError::ProcessFailed {
                exit_code: None,
                stderr: "Cancelled by user".into(),
            });
        }

        let event = tokio::select! {
            event = receiver.recv() => event,
            // Poll cancellation every 100ms even if no output
            _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => continue,
        };

        match event {
            Some(CommandEvent::Stdout(line)) => {
                let text = String::from_utf8_lossy(&line);
                let text = text.trim();
                if text.is_empty() {
                    continue;
                }
                debug!("[ffmpeg:{job_id}] stdout: {text}");

                if let Some((key, value)) = text.split_once('=') {
                    parse_progress_field(
                        key.trim(),
                        value.trim(),
                        &mut current_progress,
                        duration_ms,
                    );

                    // "progress=continue" or "progress=end" marks the end of a block
                    if key.trim() == "progress" {
                        let _ = progress_tx.send(current_progress.clone()).await;
                    }
                }
            }
            Some(CommandEvent::Stderr(line)) => {
                let text = String::from_utf8_lossy(&line);
                debug!("[ffmpeg:{job_id}] stderr: {}", text.trim());
                stderr_buf.push_str(&text);
                // Cap stderr buffer to prevent unbounded memory use
                if stderr_buf.len() > 8192 {
                    stderr_buf = stderr_buf[stderr_buf.len() - 4096..].to_string();
                }
            }
            Some(CommandEvent::Terminated(status)) => {
                info!(
                    "FFmpeg job {job_id} terminated with code {:?}",
                    status.code
                );
                // Clear child handle
                let mut handle = child_handle.lock().await;
                *handle = None;

                if status.code == Some(0) {
                    return Ok(());
                } else {
                    return Err(FfmpegError::ProcessFailed {
                        exit_code: status.code,
                        stderr: if stderr_buf.is_empty() {
                            format!("Process exited with code {:?}", status.code)
                        } else {
                            stderr_buf
                        },
                    });
                }
            }
            Some(_) => {}
            None => {
                error!("FFmpeg job {job_id}: event channel closed unexpectedly");
                return Err(FfmpegError::ProcessFailed {
                    exit_code: None,
                    stderr: "Event channel closed unexpectedly".into(),
                });
            }
        }
    }
}

/// Parse a single key=value line from FFmpeg's `-progress pipe:1` output.
fn parse_progress_field(
    key: &str,
    value: &str,
    progress: &mut FfmpegProgress,
    duration_ms: Option<u64>,
) {
    match key {
        "frame" => {
            progress.frame = value.parse().unwrap_or(0);
        }
        "fps" => {
            progress.fps = value.parse().unwrap_or(0.0);
        }
        "bitrate" => {
            // Format: "1234.5kbits/s" or "N/A"
            progress.bitrate_kbps = value
                .trim_end_matches("kbits/s")
                .parse()
                .unwrap_or(0.0);
        }
        "out_time_ms" => {
            // FFmpeg reports this in microseconds despite the name
            let microseconds: u64 = value.parse().unwrap_or(0);
            progress.time_ms = microseconds / 1000;

            // Compute percent if we know total duration
            if let Some(total) = duration_ms {
                if total > 0 {
                    let pct = (progress.time_ms as f64 / total as f64 * 100.0).min(100.0);
                    progress.percent = Some(pct);
                }
            }
        }
        "speed" => {
            // Format: "2.00x" or "N/A"
            progress.speed = value.trim_end_matches('x').parse().unwrap_or(0.0);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_progress_fields() {
        let mut p = FfmpegProgress::default();
        let duration = Some(10_000u64); // 10 seconds

        parse_progress_field("frame", "120", &mut p, duration);
        assert_eq!(p.frame, 120);

        parse_progress_field("fps", "30.00", &mut p, duration);
        assert!((p.fps - 30.0).abs() < 0.01);

        parse_progress_field("bitrate", "1234.5kbits/s", &mut p, duration);
        assert!((p.bitrate_kbps - 1234.5).abs() < 0.01);

        // out_time_ms is actually in microseconds
        parse_progress_field("out_time_ms", "5000000", &mut p, duration);
        assert_eq!(p.time_ms, 5000); // 5 seconds in ms
        assert!((p.percent.unwrap() - 50.0).abs() < 0.01);

        parse_progress_field("speed", "2.00x", &mut p, duration);
        assert!((p.speed - 2.0).abs() < 0.01);
    }

    #[test]
    fn parse_na_values() {
        let mut p = FfmpegProgress::default();
        parse_progress_field("bitrate", "N/A", &mut p, None);
        assert!((p.bitrate_kbps - 0.0).abs() < 0.01);

        parse_progress_field("speed", "N/A", &mut p, None);
        assert!((p.speed - 0.0).abs() < 0.01);
    }
}
