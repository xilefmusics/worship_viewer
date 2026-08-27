//! FFmpeg/ffprobe-backed audio/video probing and transcoding.

use std::path::{Path, PathBuf};
use std::time::Duration;

use async_trait::async_trait;
use serde::Deserialize;
use shared::MediaAssetKind;

use crate::error::AppError;
use crate::process_runner::{ProcessError, ProcessOutput, TempWorkDir, run_command_capture};

pub const MAX_PROBE_STDOUT_BYTES: usize = 65_536;
pub const MAX_TOOL_STDERR_BYTES: usize = 8_192;
pub const MAX_VIDEO_WIDTH: u32 = 7680;
pub const MAX_VIDEO_HEIGHT: u32 = 4320;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeResult {
    pub duration_ms: u64,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub has_video: bool,
    pub has_audio: bool,
}

#[derive(Debug)]
pub struct TranscodeResult {
    pub output_path: PathBuf,
    pub duration_ms: u64,
    pub width: Option<u32>,
    pub height: Option<u32>,
    _work: TempWorkDir,
}

impl TranscodeResult {
    pub fn new(
        work: TempWorkDir,
        output_path: PathBuf,
        duration_ms: u64,
        width: Option<u32>,
        height: Option<u32>,
    ) -> Self {
        Self {
            output_path,
            duration_ms,
            width,
            height,
            _work: work,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AvProcessFailure {
    InputInvalid,
    InputUnsupported,
    TimedOut,
    Failed,
}

impl AvProcessFailure {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InputInvalid => "media_input_invalid",
            Self::InputUnsupported => "media_input_unsupported",
            Self::TimedOut => "media_processing_timeout",
            Self::Failed => "media_processing_failed",
        }
    }

    pub fn detail(&self) -> &'static str {
        match self {
            Self::InputInvalid => "The uploaded file could not be read or is malformed.",
            Self::InputUnsupported => "The uploaded file format is not supported.",
            Self::TimedOut => "Processing took too long.",
            Self::Failed => "Processing failed.",
        }
    }
}

#[async_trait]
pub trait AvProcessor: Send + Sync {
    async fn probe_input(
        &self,
        input: &Path,
        kind: MediaAssetKind,
    ) -> Result<ProbeResult, AvProcessFailure>;

    async fn transcode(
        &self,
        input: &Path,
        kind: MediaAssetKind,
        work_parent: &Path,
    ) -> Result<TranscodeResult, AvProcessFailure>;
}

#[derive(Clone)]
pub struct FfmpegAvProcessor {
    pub ffmpeg_path: String,
    pub ffprobe_path: String,
    pub timeout: Duration,
}

impl FfmpegAvProcessor {
    pub fn map_process_error(err: ProcessError) -> AvProcessFailure {
        match err {
            ProcessError::TimedOut => AvProcessFailure::TimedOut,
            ProcessError::SpawnFailed | ProcessError::Io(_) => AvProcessFailure::Failed,
            ProcessError::NonZeroExit(_)
            | ProcessError::StderrTooLarge
            | ProcessError::StdoutTooLarge => AvProcessFailure::Failed,
        }
    }

    async fn run_tool(&self, argv: &[&str]) -> Result<ProcessOutput, AvProcessFailure> {
        run_command_capture(
            argv,
            self.timeout,
            MAX_PROBE_STDOUT_BYTES,
            MAX_TOOL_STDERR_BYTES,
        )
        .await
        .map_err(Self::map_process_error)
    }

    fn parse_probe_json(
        stdout: &str,
        kind: MediaAssetKind,
    ) -> Result<ProbeResult, AvProcessFailure> {
        #[derive(Deserialize)]
        struct ProbeFormat {
            duration: Option<String>,
        }
        #[derive(Deserialize)]
        struct ProbeStream {
            codec_type: Option<String>,
            width: Option<u32>,
            height: Option<u32>,
        }
        #[derive(Deserialize)]
        struct ProbeDoc {
            format: Option<ProbeFormat>,
            streams: Option<Vec<ProbeStream>>,
        }

        let doc: ProbeDoc =
            serde_json::from_str(stdout).map_err(|_| AvProcessFailure::InputInvalid)?;
        let streams = doc.streams.unwrap_or_default();
        let has_video = streams
            .iter()
            .any(|s| s.codec_type.as_deref() == Some("video"));
        let has_audio = streams
            .iter()
            .any(|s| s.codec_type.as_deref() == Some("audio"));
        match kind {
            MediaAssetKind::Video if !has_video => return Err(AvProcessFailure::InputUnsupported),
            MediaAssetKind::Audio if !has_audio => return Err(AvProcessFailure::InputUnsupported),
            _ => {}
        }
        let video_stream = streams
            .iter()
            .find(|s| s.codec_type.as_deref() == Some("video"));
        let width = video_stream.and_then(|s| s.width);
        let height = video_stream.and_then(|s| s.height);
        if let (Some(w), Some(h)) = (width, height)
            && (w > MAX_VIDEO_WIDTH || h > MAX_VIDEO_HEIGHT)
        {
            return Err(AvProcessFailure::InputUnsupported);
        }
        let duration_secs = doc
            .format
            .and_then(|f| f.duration)
            .and_then(|d| d.parse::<f64>().ok())
            .filter(|d| *d > 0.0)
            .unwrap_or(0.0);
        if duration_secs <= 0.0 {
            return Err(AvProcessFailure::InputInvalid);
        }
        let duration_ms = (duration_secs * 1000.0).round() as u64;
        Ok(ProbeResult {
            duration_ms,
            width,
            height,
            has_video,
            has_audio,
        })
    }
}

#[async_trait]
impl AvProcessor for FfmpegAvProcessor {
    async fn probe_input(
        &self,
        input: &Path,
        kind: MediaAssetKind,
    ) -> Result<ProbeResult, AvProcessFailure> {
        let input_str = input.to_string_lossy();
        let output = self
            .run_tool(&[
                self.ffprobe_path.as_str(),
                "-v",
                "error",
                "-print_format",
                "json",
                "-show_format",
                "-show_streams",
                input_str.as_ref(),
            ])
            .await?;
        if !output.status.success() {
            return Err(AvProcessFailure::InputInvalid);
        }
        Self::parse_probe_json(&output.stdout, kind)
    }

    async fn transcode(
        &self,
        input: &Path,
        kind: MediaAssetKind,
        work_parent: &Path,
    ) -> Result<TranscodeResult, AvProcessFailure> {
        let work = TempWorkDir::new(work_parent).map_err(|_| AvProcessFailure::Failed)?;
        let input_str = input.to_string_lossy();
        let output_path = match kind {
            MediaAssetKind::Video => {
                let out = work.path().join("output.mp4");
                let out_str = out.to_string_lossy();
                let probe = self.probe_input(input, kind).await?;
                let mut argv = vec![
                    self.ffmpeg_path.as_str(),
                    "-nostdin",
                    "-y",
                    "-i",
                    input_str.as_ref(),
                    "-c:v",
                    "libx264",
                    "-preset",
                    "veryfast",
                    "-pix_fmt",
                    "yuv420p",
                    "-movflags",
                    "+faststart",
                ];
                if probe.has_audio {
                    argv.extend(["-c:a", "aac", "-b:a", "192k"]);
                } else {
                    argv.push("-an");
                }
                argv.push(out_str.as_ref());
                let output = self.run_tool(&argv).await?;
                if !output.status.success() {
                    return Err(AvProcessFailure::Failed);
                }
                out
            }
            MediaAssetKind::Audio => {
                let out = work.path().join("output.m4a");
                let out_str = out.to_string_lossy();
                let argv = [
                    self.ffmpeg_path.as_str(),
                    "-nostdin",
                    "-y",
                    "-i",
                    input_str.as_ref(),
                    "-vn",
                    "-c:a",
                    "aac",
                    "-b:a",
                    "192k",
                    "-f",
                    "ipod",
                    out_str.as_ref(),
                ];
                let output = self.run_tool(&argv).await?;
                if !output.status.success() {
                    return Err(AvProcessFailure::Failed);
                }
                out
            }
            _ => return Err(AvProcessFailure::InputUnsupported),
        };

        let probe = self.probe_input(&output_path, kind).await?;
        Ok(TranscodeResult::new(
            work,
            output_path,
            probe.duration_ms,
            probe.width,
            probe.height,
        ))
    }
}

pub fn app_error_from_failure(failure: AvProcessFailure) -> AppError {
    AppError::media_processing(failure.code(), failure.detail())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_probe_rejects_missing_audio() {
        let json = r#"{"format":{"duration":"1.0"},"streams":[{"codec_type":"video","width":640,"height":360}]}"#;
        assert_eq!(
            FfmpegAvProcessor::parse_probe_json(json, MediaAssetKind::Audio),
            Err(AvProcessFailure::InputUnsupported)
        );
    }

    #[test]
    fn parse_probe_rejects_absurd_dimensions() {
        let json = r#"{"format":{"duration":"1.0"},"streams":[{"codec_type":"video","width":9000,"height":5000}]}"#;
        assert_eq!(
            FfmpegAvProcessor::parse_probe_json(json, MediaAssetKind::Video),
            Err(AvProcessFailure::InputUnsupported)
        );
    }

    #[test]
    fn parse_probe_rejects_malformed_json() {
        assert_eq!(
            FfmpegAvProcessor::parse_probe_json("{not json", MediaAssetKind::Video),
            Err(AvProcessFailure::InputInvalid)
        );
    }

    #[test]
    fn parse_probe_accepts_video() {
        let json = r#"{"format":{"duration":"2.5"},"streams":[{"codec_type":"video","width":640,"height":360},{"codec_type":"audio"}]}"#;
        let probe = FfmpegAvProcessor::parse_probe_json(json, MediaAssetKind::Video).unwrap();
        assert_eq!(probe.duration_ms, 2500);
        assert_eq!(probe.width, Some(640));
        assert_eq!(probe.height, Some(360));
    }
}
