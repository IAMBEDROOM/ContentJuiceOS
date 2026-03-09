#![allow(dead_code)]

use crate::ffmpeg::error::FfmpegError;

/// Type-safe builder for constructing FFmpeg command-line arguments.
///
/// Produces a `Vec<String>` suitable for passing to the FFmpeg sidecar.
/// Automatically prepends `-progress pipe:1 -nostats` so the executor
/// can parse real-time progress output.
#[derive(Debug, Clone, Default)]
pub struct FfmpegCommandBuilder {
    inputs: Vec<InputSpec>,
    outputs: Vec<OutputSpec>,
    video_codec: Option<String>,
    audio_codec: Option<String>,
    video_filters: Vec<String>,
    audio_filters: Vec<String>,
    format: Option<String>,
    overwrite: bool,
    custom_args: Vec<String>,
}

#[derive(Debug, Clone)]
struct InputSpec {
    path: String,
    pre_input_opts: Vec<String>,
}

#[derive(Debug, Clone)]
struct OutputSpec {
    path: String,
    output_opts: Vec<String>,
}

impl FfmpegCommandBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an input file.
    pub fn input(mut self, path: impl Into<String>) -> Self {
        self.inputs.push(InputSpec {
            path: path.into(),
            pre_input_opts: Vec::new(),
        });
        self
    }

    /// Add an input file with pre-input options (e.g., `-ss 10`).
    pub fn input_with_opts(
        mut self,
        path: impl Into<String>,
        opts: Vec<String>,
    ) -> Self {
        self.inputs.push(InputSpec {
            path: path.into(),
            pre_input_opts: opts,
        });
        self
    }

    /// Add an output file.
    pub fn output(mut self, path: impl Into<String>) -> Self {
        self.outputs.push(OutputSpec {
            path: path.into(),
            output_opts: Vec::new(),
        });
        self
    }

    /// Add an output file with output-specific options (e.g., `-b:a 192k`).
    pub fn output_with_opts(
        mut self,
        path: impl Into<String>,
        opts: Vec<String>,
    ) -> Self {
        self.outputs.push(OutputSpec {
            path: path.into(),
            output_opts: opts,
        });
        self
    }

    /// Set the video codec (e.g., `libx264`, `copy`).
    pub fn video_codec(mut self, codec: impl Into<String>) -> Self {
        self.video_codec = Some(codec.into());
        self
    }

    /// Set the audio codec (e.g., `aac`, `libmp3lame`, `copy`).
    pub fn audio_codec(mut self, codec: impl Into<String>) -> Self {
        self.audio_codec = Some(codec.into());
        self
    }

    /// Add a video filter (e.g., `scale=1920:1080`).
    pub fn video_filter(mut self, filter: impl Into<String>) -> Self {
        self.video_filters.push(filter.into());
        self
    }

    /// Add an audio filter (e.g., `volume=2.0`).
    pub fn audio_filter(mut self, filter: impl Into<String>) -> Self {
        self.audio_filters.push(filter.into());
        self
    }

    /// Set the output format (e.g., `mp4`, `mp3`, `wav`).
    pub fn format(mut self, fmt: impl Into<String>) -> Self {
        self.format = Some(fmt.into());
        self
    }

    /// Whether to overwrite output files without asking (`-y`).
    pub fn overwrite(mut self, yes: bool) -> Self {
        self.overwrite = yes;
        self
    }

    /// Add arbitrary custom arguments.
    pub fn custom_args(mut self, args: Vec<String>) -> Self {
        self.custom_args.extend(args);
        self
    }

    /// Build the final argument vector. Returns an error if no input or output is specified.
    pub fn build(self) -> Result<Vec<String>, FfmpegError> {
        if self.inputs.is_empty() {
            return Err(FfmpegError::InvalidCommand(
                "At least one input is required".into(),
            ));
        }
        if self.outputs.is_empty() {
            return Err(FfmpegError::InvalidCommand(
                "At least one output is required".into(),
            ));
        }

        let mut args: Vec<String> = Vec::new();

        // Progress output to stdout for parsing
        args.extend(["-progress".into(), "pipe:1".into(), "-nostats".into()]);

        // Overwrite flag must come before inputs
        if self.overwrite {
            args.push("-y".into());
        }

        // Inputs (with pre-input options)
        for input in &self.inputs {
            args.extend(input.pre_input_opts.clone());
            args.extend(["-i".into(), input.path.clone()]);
        }

        // Codecs
        if let Some(ref vc) = self.video_codec {
            args.extend(["-c:v".into(), vc.clone()]);
        }
        if let Some(ref ac) = self.audio_codec {
            args.extend(["-c:a".into(), ac.clone()]);
        }

        // Filters
        if !self.video_filters.is_empty() {
            args.extend(["-vf".into(), self.video_filters.join(",")]);
        }
        if !self.audio_filters.is_empty() {
            args.extend(["-af".into(), self.audio_filters.join(",")]);
        }

        // Format
        if let Some(ref fmt) = self.format {
            args.extend(["-f".into(), fmt.clone()]);
        }

        // Custom args
        args.extend(self.custom_args);

        // Outputs (with per-output options before each output path)
        for output in &self.outputs {
            args.extend(output.output_opts.clone());
            args.push(output.path.clone());
        }

        Ok(args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_requires_input() {
        let result = FfmpegCommandBuilder::new().output("out.mp4").build();
        assert!(result.is_err());
    }

    #[test]
    fn build_requires_output() {
        let result = FfmpegCommandBuilder::new().input("in.mp4").build();
        assert!(result.is_err());
    }

    #[test]
    fn basic_transcode() {
        let args = FfmpegCommandBuilder::new()
            .input("input.wav")
            .audio_codec("libmp3lame")
            .overwrite(true)
            .output("output.mp3")
            .build()
            .unwrap();

        assert_eq!(
            args,
            vec![
                "-progress", "pipe:1", "-nostats", "-y",
                "-i", "input.wav",
                "-c:a", "libmp3lame",
                "output.mp3",
            ]
        );
    }

    #[test]
    fn complex_command_with_filters() {
        let args = FfmpegCommandBuilder::new()
            .input("input.mp4")
            .video_codec("libx264")
            .audio_codec("aac")
            .video_filter("scale=1280:720")
            .overwrite(true)
            .output("output.mp4")
            .build()
            .unwrap();

        assert!(args.contains(&"-c:v".to_string()));
        assert!(args.contains(&"libx264".to_string()));
        assert!(args.contains(&"-vf".to_string()));
        assert!(args.contains(&"scale=1280:720".to_string()));
    }

    #[test]
    fn pre_input_opts() {
        let args = FfmpegCommandBuilder::new()
            .input_with_opts("input.mp4", vec!["-ss".into(), "10".into()])
            .output("output.mp4")
            .build()
            .unwrap();

        let ss_pos = args.iter().position(|a| a == "-ss").unwrap();
        let i_pos = args.iter().position(|a| a == "-i").unwrap();
        assert!(ss_pos < i_pos, "-ss should come before -i");
    }
}
