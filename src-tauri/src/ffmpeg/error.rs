use std::fmt;

#[derive(Debug)]
pub enum FfmpegError {
    /// FFmpeg process exited with a non-zero exit code
    ProcessFailed {
        exit_code: Option<i32>,
        stderr: String,
    },
    /// Could not spawn the sidecar process
    SpawnFailed(String),
    /// ffprobe output could not be parsed
    ProbeFailed(String),
    /// No job found with the given ID
    JobNotFound(String),
    /// Job is in a state that doesn't allow the requested operation
    InvalidJobState { job_id: String, state: String },
    /// Command builder validation failed
    InvalidCommand(String),
}

impl fmt::Display for FfmpegError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProcessFailed { exit_code, stderr } => {
                write!(
                    f,
                    "FFmpeg process failed (exit code: {exit_code:?}): {stderr}"
                )
            }
            Self::SpawnFailed(msg) => write!(f, "Failed to spawn FFmpeg: {msg}"),
            Self::ProbeFailed(msg) => write!(f, "ffprobe failed: {msg}"),
            Self::JobNotFound(id) => write!(f, "Job not found: {id}"),
            Self::InvalidJobState { job_id, state } => {
                write!(
                    f,
                    "Job {job_id} is in invalid state for this operation: {state}"
                )
            }
            Self::InvalidCommand(msg) => write!(f, "Invalid FFmpeg command: {msg}"),
        }
    }
}

impl std::error::Error for FfmpegError {}

impl From<FfmpegError> for String {
    fn from(e: FfmpegError) -> Self {
        e.to_string()
    }
}
