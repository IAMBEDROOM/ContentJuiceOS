use serde::{Deserialize, Serialize};

/// Priority level for queued FFmpeg jobs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum JobPriority {
    Low = 0,
    Normal = 1,
    High = 2,
}

/// Lifecycle state of an FFmpeg job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobState {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for JobState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Queued => write!(f, "Queued"),
            Self::Running => write!(f, "Running"),
            Self::Completed => write!(f, "Completed"),
            Self::Failed => write!(f, "Failed"),
            Self::Cancelled => write!(f, "Cancelled"),
        }
    }
}

/// Real-time progress data parsed from FFmpeg's `-progress pipe:1` output.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FfmpegProgress {
    pub job_id: String,
    pub frame: u64,
    pub fps: f64,
    pub bitrate_kbps: f64,
    pub time_ms: u64,
    pub speed: f64,
    pub percent: Option<f64>,
}

/// Internal representation of a queued/running FFmpeg job.
#[derive(Debug, Clone)]
pub struct FfmpegJob {
    pub id: String,
    pub state: JobState,
    pub priority: JobPriority,
    pub command_args: Vec<String>,
    pub input_path: String,
    pub output_path: String,
    pub duration_ms: Option<u64>,
    pub progress: Option<FfmpegProgress>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub error: Option<String>,
}

/// Serializable job status returned to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobStatus {
    pub id: String,
    pub state: JobState,
    pub priority: JobPriority,
    pub input_path: String,
    pub output_path: String,
    pub progress: Option<FfmpegProgress>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub error: Option<String>,
}

impl From<&FfmpegJob> for JobStatus {
    fn from(job: &FfmpegJob) -> Self {
        Self {
            id: job.id.clone(),
            state: job.state.clone(),
            priority: job.priority,
            input_path: job.input_path.clone(),
            output_path: job.output_path.clone(),
            progress: job.progress.clone(),
            created_at: job.created_at.to_rfc3339(),
            started_at: job.started_at.map(|t| t.to_rfc3339()),
            completed_at: job.completed_at.map(|t| t.to_rfc3339()),
            error: job.error.clone(),
        }
    }
}

/// Media information returned by ffprobe.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaInfo {
    pub format_name: String,
    pub duration_ms: Option<u64>,
    pub size_bytes: Option<u64>,
    pub bit_rate: Option<u64>,
    pub streams: Vec<StreamInfo>,
}

/// Individual stream information from ffprobe.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamInfo {
    pub index: u32,
    pub codec_type: String,
    pub codec_name: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub frame_rate: Option<f64>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u32>,
    pub bit_rate: Option<u64>,
}
