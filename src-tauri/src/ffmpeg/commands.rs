use std::sync::Arc;

use tauri::{AppHandle, State};

use crate::ffmpeg::probe;
use crate::ffmpeg::queue::FfmpegQueue;
use crate::ffmpeg::types::{JobPriority, JobState, JobStatus, MediaInfo};

/// Submit an FFmpeg job to the processing queue.
///
/// If `duration_ms` is not provided, automatically probes the input file
/// to determine duration (needed for progress percentage calculation).
#[tauri::command]
pub async fn ffmpeg_submit_job(
    app_handle: AppHandle,
    queue: State<'_, Arc<FfmpegQueue>>,
    args: Vec<String>,
    input_path: String,
    output_path: String,
    duration_ms: Option<u64>,
    priority: Option<JobPriority>,
) -> Result<String, String> {
    // Auto-probe duration if not provided
    let duration = match duration_ms {
        Some(d) => Some(d),
        None => {
            match probe::probe_media(&app_handle, &input_path).await {
                Ok(info) => info.duration_ms,
                Err(e) => {
                    log::warn!("Could not probe input for duration: {e}");
                    None
                }
            }
        }
    };

    let job_id = queue
        .submit_job(
            app_handle,
            args,
            input_path,
            output_path,
            duration,
            priority.unwrap_or(JobPriority::Normal),
        )
        .await;

    Ok(job_id)
}

/// Get the status of a specific FFmpeg job.
#[tauri::command]
pub async fn ffmpeg_get_job(
    queue: State<'_, Arc<FfmpegQueue>>,
    job_id: String,
) -> Result<JobStatus, String> {
    queue.get_job(&job_id).await.map_err(|e| e.to_string())
}

/// List all FFmpeg jobs, optionally filtered by state.
#[tauri::command]
pub async fn ffmpeg_list_jobs(
    queue: State<'_, Arc<FfmpegQueue>>,
    state: Option<JobState>,
) -> Result<Vec<JobStatus>, String> {
    Ok(queue.list_jobs(state).await)
}

/// Cancel a queued or running FFmpeg job.
#[tauri::command]
pub async fn ffmpeg_cancel_job(
    queue: State<'_, Arc<FfmpegQueue>>,
    job_id: String,
) -> Result<(), String> {
    queue.cancel_job(&job_id).await.map_err(|e| e.to_string())
}

/// Run ffprobe on a file and return media information.
#[tauri::command]
pub async fn ffprobe_media_info(
    app_handle: AppHandle,
    file_path: String,
) -> Result<MediaInfo, String> {
    probe::probe_media(&app_handle, &file_path)
        .await
        .map_err(|e| e.to_string())
}
