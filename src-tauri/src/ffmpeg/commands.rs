use std::path::Path;
use std::sync::Arc;

use tauri::{AppHandle, State};

use crate::ffmpeg::probe;
use crate::ffmpeg::queue::FfmpegQueue;
use crate::ffmpeg::types::{JobPriority, JobState, JobStatus, MediaInfo};
use crate::user_error::UserFacingError;

/// Validate that a media file path is safe to use with FFmpeg.
fn validate_media_path(path: &str, must_exist: bool) -> Result<(), String> {
    if path.is_empty() {
        return Err("File path cannot be empty.".to_string());
    }

    // Reject null bytes (could truncate paths in C-level FFmpeg calls)
    if path.contains('\0') {
        return Err("File path contains invalid characters.".to_string());
    }

    let p = Path::new(path);

    // Require absolute paths to prevent ambiguity
    if !p.is_absolute() {
        return Err("File path must be absolute.".to_string());
    }

    if must_exist {
        if !p.exists() {
            return Err("Input file does not exist.".to_string());
        }
        if !p.is_file() {
            return Err("Input path is not a regular file.".to_string());
        }
    } else {
        // For output paths, verify parent directory exists
        if let Some(parent) = p.parent() {
            if !parent.exists() {
                return Err("Output directory does not exist.".to_string());
            }
        }
    }

    Ok(())
}

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
    // Validate paths before submitting
    validate_media_path(&input_path, true)?;
    validate_media_path(&output_path, false)?;

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
    queue.get_job(&job_id).await.map_user_err()
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
    queue.cancel_job(&job_id).await.map_user_err()
}

/// Run ffprobe on a file and return media information.
#[tauri::command]
pub async fn ffprobe_media_info(
    app_handle: AppHandle,
    file_path: String,
) -> Result<MediaInfo, String> {
    validate_media_path(&file_path, true)?;
    probe::probe_media(&app_handle, &file_path)
        .await
        .map_user_err()
}
