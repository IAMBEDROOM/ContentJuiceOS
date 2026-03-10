use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

use log::{error, info};
use tauri::AppHandle;
use tauri_plugin_shell::process::CommandChild;
use tokio::sync::Mutex;

use crate::ffmpeg::error::FfmpegError;
use crate::ffmpeg::executor;
use crate::ffmpeg::types::{FfmpegJob, FfmpegProgress, JobPriority, JobState, JobStatus};
use crate::server::emit::emit_socket_io_event;

type ChildHandleMap = Arc<Mutex<HashMap<String, Arc<Mutex<Option<CommandChild>>>>>>;

/// Manages a priority-based job queue for FFmpeg operations with
/// configurable concurrency and real-time Socket.IO progress reporting.
pub struct FfmpegQueue {
    jobs: Arc<Mutex<HashMap<String, FfmpegJob>>>,
    max_concurrent: u32,
    running_count: Arc<AtomicU32>,
    cancel_flags: Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>,
    child_handles: ChildHandleMap,
    socket_io_port: u16,
}

impl FfmpegQueue {
    pub fn new(max_concurrent: u32, socket_io_port: u16) -> Self {
        Self {
            jobs: Arc::new(Mutex::new(HashMap::new())),
            max_concurrent,
            running_count: Arc::new(AtomicU32::new(0)),
            cancel_flags: Arc::new(Mutex::new(HashMap::new())),
            child_handles: Arc::new(Mutex::new(HashMap::new())),
            socket_io_port,
        }
    }

    /// Submit a new FFmpeg job to the queue. Returns the job ID.
    pub async fn submit_job(
        &self,
        app_handle: AppHandle,
        command_args: Vec<String>,
        input_path: String,
        output_path: String,
        duration_ms: Option<u64>,
        priority: JobPriority,
    ) -> String {
        let job_id = uuid::Uuid::new_v4().to_string();
        let job = FfmpegJob {
            id: job_id.clone(),
            state: JobState::Queued,
            priority,
            command_args,
            input_path,
            output_path,
            duration_ms,
            progress: None,
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
            error: None,
        };

        {
            let mut jobs = self.jobs.lock().await;
            jobs.insert(job_id.clone(), job);
        }

        info!("FFmpeg job {job_id} submitted with priority {priority:?}");
        try_run_next(
            app_handle,
            self.jobs.clone(),
            self.max_concurrent,
            self.running_count.clone(),
            self.cancel_flags.clone(),
            self.child_handles.clone(),
            self.socket_io_port,
        )
        .await;
        job_id
    }

    /// Cancel a queued or running job.
    pub async fn cancel_job(&self, job_id: &str) -> Result<(), FfmpegError> {
        let mut jobs = self.jobs.lock().await;
        let job = jobs
            .get_mut(job_id)
            .ok_or_else(|| FfmpegError::JobNotFound(job_id.to_string()))?;

        match job.state {
            JobState::Queued => {
                job.state = JobState::Cancelled;
                job.completed_at = Some(chrono::Utc::now());
                info!("FFmpeg job {job_id} cancelled (was queued)");
                Ok(())
            }
            JobState::Running => {
                job.state = JobState::Cancelled;
                job.completed_at = Some(chrono::Utc::now());

                // Set cancel flag — executor will detect and kill the process
                let flags = self.cancel_flags.lock().await;
                if let Some(flag) = flags.get(job_id) {
                    flag.store(true, Ordering::Relaxed);
                }

                // Also kill the child process directly
                let handles = self.child_handles.lock().await;
                if let Some(handle) = handles.get(job_id) {
                    let mut child = handle.lock().await;
                    if let Some(c) = child.take() {
                        let _ = c.kill();
                    }
                }

                info!("FFmpeg job {job_id} cancelled (was running)");
                Ok(())
            }
            _ => Err(FfmpegError::InvalidJobState {
                job_id: job_id.to_string(),
                state: job.state.to_string(),
            }),
        }
    }

    /// Get the status of a specific job.
    pub async fn get_job(&self, job_id: &str) -> Result<JobStatus, FfmpegError> {
        let jobs = self.jobs.lock().await;
        jobs.get(job_id)
            .map(JobStatus::from)
            .ok_or_else(|| FfmpegError::JobNotFound(job_id.to_string()))
    }

    /// List all jobs, optionally filtered by state.
    pub async fn list_jobs(&self, state_filter: Option<JobState>) -> Vec<JobStatus> {
        let jobs = self.jobs.lock().await;
        jobs.values()
            .filter(|j| {
                state_filter
                    .as_ref()
                    .is_none_or(|filter| j.state == *filter)
            })
            .map(JobStatus::from)
            .collect()
    }
}

/// Free function that returns an explicitly boxed Send future to break the
/// recursive type inference cycle (this function spawns a task that calls itself).
fn try_run_next(
    app_handle: AppHandle,
    jobs: Arc<Mutex<HashMap<String, FfmpegJob>>>,
    max_concurrent: u32,
    running_count: Arc<AtomicU32>,
    cancel_flags: Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>,
    child_handles: ChildHandleMap,
    socket_io_port: u16,
) -> Pin<Box<dyn Future<Output = ()> + Send>> {
    Box::pin(async move {
        let current = running_count.load(Ordering::Relaxed);
        if current >= max_concurrent {
            return;
        }

        // Find highest-priority queued job
        let next_job_id = {
            let jobs_guard = jobs.lock().await;
            jobs_guard
                .values()
                .filter(|j| j.state == JobState::Queued)
                .max_by_key(|j| (j.priority, std::cmp::Reverse(j.created_at)))
                .map(|j| j.id.clone())
        };

        let job_id = match next_job_id {
            Some(id) => id,
            None => return,
        };

        // Transition to Running
        let (args, duration_ms) = {
            let mut jobs_guard = jobs.lock().await;
            let job = match jobs_guard.get_mut(&job_id) {
                Some(j) => j,
                None => return,
            };
            job.state = JobState::Running;
            job.started_at = Some(chrono::Utc::now());
            (job.command_args.clone(), job.duration_ms)
        };

        running_count.fetch_add(1, Ordering::Relaxed);

        // Set up cancellation and child handle
        let cancel_flag = Arc::new(AtomicBool::new(false));
        let child_handle: Arc<Mutex<Option<CommandChild>>> = Arc::new(Mutex::new(None));

        {
            let mut flags = cancel_flags.lock().await;
            flags.insert(job_id.clone(), cancel_flag.clone());
        }
        {
            let mut handles = child_handles.lock().await;
            handles.insert(job_id.clone(), child_handle.clone());
        }

        // Clone Arcs for the spawned task
        let task_jobs = jobs.clone();
        let task_running = running_count.clone();
        let task_cancel_flags = cancel_flags.clone();
        let task_child_handles = child_handles.clone();
        let job_id_clone = job_id.clone();

        // Clone Arcs again for the recursive try_run_next call
        let next_jobs = jobs;
        let next_running = running_count;
        let next_cancel = cancel_flags;
        let next_children = child_handles;

        tauri::async_runtime::spawn(async move {
            let (progress_tx, mut progress_rx) = tokio::sync::mpsc::channel::<FfmpegProgress>(32);

            // Spawn progress forwarder to Socket.IO
            let fwd_job_id = job_id_clone.clone();
            let fwd_jobs = task_jobs.clone();
            let fwd_port = socket_io_port;
            let progress_forwarder = tauri::async_runtime::spawn(async move {
                while let Some(progress) = progress_rx.recv().await {
                    // Update stored progress
                    {
                        let mut jobs_guard = fwd_jobs.lock().await;
                        if let Some(job) = jobs_guard.get_mut(&fwd_job_id) {
                            job.progress = Some(progress.clone());
                        }
                    }

                    // Emit to Socket.IO
                    let data = serde_json::to_value(&progress).unwrap_or_default();
                    emit_socket_io_event(fwd_port, "/control", "ffmpeg:progress", &data).await;
                }
            });

            // Execute FFmpeg
            let result = executor::execute_ffmpeg(
                &app_handle,
                job_id_clone.clone(),
                args,
                duration_ms,
                progress_tx,
                cancel_flag,
                child_handle,
            )
            .await;

            // Wait for progress forwarder to drain
            let _ = progress_forwarder.await;

            // Update job state
            {
                let mut jobs_guard = task_jobs.lock().await;
                if let Some(job) = jobs_guard.get_mut(&job_id_clone) {
                    match &result {
                        Ok(()) => {
                            job.state = JobState::Completed;
                            job.completed_at = Some(chrono::Utc::now());
                            info!("FFmpeg job {} completed", job_id_clone);

                            let data = serde_json::json!({
                                "jobId": job_id_clone,
                                "outputPath": job.output_path,
                            });
                            emit_socket_io_event(
                                socket_io_port,
                                "/control",
                                "ffmpeg:complete",
                                &data,
                            )
                            .await;
                        }
                        Err(e) => {
                            // Check if it was a cancellation
                            if job.state != JobState::Cancelled {
                                job.state = JobState::Failed;
                                job.error = Some(e.to_string());
                                job.completed_at = Some(chrono::Utc::now());
                                error!("FFmpeg job {} failed: {e}", job_id_clone);

                                let data = serde_json::json!({
                                    "jobId": job_id_clone,
                                    "error": e.to_string(),
                                });
                                emit_socket_io_event(
                                    socket_io_port,
                                    "/control",
                                    "ffmpeg:error",
                                    &data,
                                )
                                .await;
                            }
                        }
                    }
                }
            }

            // Clean up
            {
                let mut flags = task_cancel_flags.lock().await;
                flags.remove(&job_id_clone);
            }
            {
                let mut handles = task_child_handles.lock().await;
                handles.remove(&job_id_clone);
            }
            task_running.fetch_sub(1, Ordering::Relaxed);

            // Try to run next queued job (spawn as independent task to avoid Send issues)
            tauri::async_runtime::spawn(try_run_next(
                app_handle,
                next_jobs,
                max_concurrent,
                next_running,
                next_cancel,
                next_children,
                socket_io_port,
            ));
        });
    }) // close Box::pin(async move {
}
