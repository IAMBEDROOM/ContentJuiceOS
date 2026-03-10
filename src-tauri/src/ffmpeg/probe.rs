use log::debug;
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;

use crate::ffmpeg::error::FfmpegError;
use crate::ffmpeg::types::{MediaInfo, StreamInfo};

/// Run ffprobe on a file and return parsed media information.
pub async fn probe_media(
    app_handle: &AppHandle,
    file_path: &str,
) -> Result<MediaInfo, FfmpegError> {
    let sidecar = app_handle
        .shell()
        .sidecar("binaries/ffprobe")
        .map_err(|e| FfmpegError::SpawnFailed(format!("Failed to create ffprobe sidecar: {e}")))?
        .args([
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
            file_path,
        ]);

    let output = sidecar
        .output()
        .await
        .map_err(|e| FfmpegError::SpawnFailed(format!("Failed to spawn ffprobe: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(FfmpegError::ProbeFailed(format!(
            "ffprobe exited with code {:?}: {stderr}",
            output.status.code()
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    debug!("ffprobe output: {stdout}");

    let json: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|e| FfmpegError::ProbeFailed(format!("Failed to parse ffprobe JSON: {e}")))?;

    parse_probe_output(&json)
}

fn parse_probe_output(json: &serde_json::Value) -> Result<MediaInfo, FfmpegError> {
    let format = json
        .get("format")
        .ok_or_else(|| FfmpegError::ProbeFailed("Missing 'format' in ffprobe output".into()))?;

    let format_name = format
        .get("format_name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let duration_ms = format
        .get("duration")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<f64>().ok())
        .map(|secs| (secs * 1000.0) as u64);

    let size_bytes = format
        .get("size")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<u64>().ok());

    let bit_rate = format
        .get("bit_rate")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<u64>().ok());

    let streams = json
        .get("streams")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(parse_stream).collect())
        .unwrap_or_default();

    Ok(MediaInfo {
        format_name,
        duration_ms,
        size_bytes,
        bit_rate,
        streams,
    })
}

fn parse_stream(stream: &serde_json::Value) -> Option<StreamInfo> {
    let codec_type = stream.get("codec_type")?.as_str()?.to_string();
    let codec_name = stream
        .get("codec_name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let index = stream.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as u32;

    let width = stream
        .get("width")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);
    let height = stream
        .get("height")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);

    let frame_rate = stream
        .get("r_frame_rate")
        .and_then(|v| v.as_str())
        .and_then(parse_frame_rate);

    let sample_rate = stream
        .get("sample_rate")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<u32>().ok());

    let channels = stream
        .get("channels")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);

    let bit_rate = stream
        .get("bit_rate")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<u64>().ok());

    Some(StreamInfo {
        index,
        codec_type,
        codec_name,
        width,
        height,
        frame_rate,
        sample_rate,
        channels,
        bit_rate,
    })
}

/// Parse ffprobe frame rate strings like "30/1" or "24000/1001" into f64.
fn parse_frame_rate(s: &str) -> Option<f64> {
    if let Some((num, den)) = s.split_once('/') {
        let n: f64 = num.parse().ok()?;
        let d: f64 = den.parse().ok()?;
        if d > 0.0 {
            return Some(n / d);
        }
    }
    s.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_frame_rate_fraction() {
        assert!((parse_frame_rate("30/1").unwrap() - 30.0).abs() < 0.001);
        assert!((parse_frame_rate("24000/1001").unwrap() - 23.976).abs() < 0.01);
    }

    #[test]
    fn parse_frame_rate_plain() {
        assert!((parse_frame_rate("29.97").unwrap() - 29.97).abs() < 0.001);
    }

    #[test]
    fn parse_probe_output_full() {
        let json: serde_json::Value = serde_json::json!({
            "format": {
                "format_name": "mov,mp4,m4a,3gp,3g2,mj2",
                "duration": "120.500",
                "size": "5000000",
                "bit_rate": "332000"
            },
            "streams": [
                {
                    "index": 0,
                    "codec_type": "video",
                    "codec_name": "h264",
                    "width": 1920,
                    "height": 1080,
                    "r_frame_rate": "30/1"
                },
                {
                    "index": 1,
                    "codec_type": "audio",
                    "codec_name": "aac",
                    "sample_rate": "48000",
                    "channels": 2,
                    "bit_rate": "128000"
                }
            ]
        });

        let info = parse_probe_output(&json).unwrap();
        assert_eq!(info.format_name, "mov,mp4,m4a,3gp,3g2,mj2");
        assert_eq!(info.duration_ms, Some(120500));
        assert_eq!(info.size_bytes, Some(5000000));
        assert_eq!(info.streams.len(), 2);
        assert_eq!(info.streams[0].codec_type, "video");
        assert_eq!(info.streams[0].width, Some(1920));
        assert_eq!(info.streams[1].codec_type, "audio");
        assert_eq!(info.streams[1].channels, Some(2));
    }
}
