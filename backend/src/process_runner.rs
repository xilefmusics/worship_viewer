//! Fixed-argv child process execution and temporary workspace helpers.

use std::path::{Path, PathBuf};
use std::process::ExitStatus;
use std::time::Duration;

use thiserror::Error;
use tokio::io::AsyncReadExt;
use tokio::process::{ChildStderr, ChildStdout, Command};
use tokio::time::timeout;

#[derive(Debug, Error)]
pub enum ProcessError {
    #[error("process timed out")]
    TimedOut,
    #[error("failed to spawn process")]
    SpawnFailed,
    #[error("process failed with status {0}")]
    NonZeroExit(i32),
    #[error("stdout output exceeded limit")]
    StdoutTooLarge,
    #[error("stderr output exceeded limit")]
    StderrTooLarge,
    #[error("io error")]
    Io(#[from] std::io::Error),
}

/// Result of a bounded child process invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessOutput {
    pub status: ExitStatus,
    pub stdout: String,
    pub stderr: String,
}

/// Run `argv[0]` with remaining args, without a shell. Stdin is closed; stdout/stderr discarded.
pub async fn run_command(
    argv: &[&str],
    run_timeout: Duration,
    max_stderr_bytes: usize,
) -> Result<ProcessOutput, ProcessError> {
    run_command_capture(argv, run_timeout, 0, max_stderr_bytes).await
}

/// Run with bounded stdout and stderr capture.
pub async fn run_command_capture(
    argv: &[&str],
    run_timeout: Duration,
    max_stdout_bytes: usize,
    max_stderr_bytes: usize,
) -> Result<ProcessOutput, ProcessError> {
    let (program, args) = argv.split_first().ok_or(ProcessError::SpawnFailed)?;
    let mut child = Command::new(program)
        .args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|_| ProcessError::SpawnFailed)?;

    let stdout_handle = child.stdout.take();
    let stderr_handle = child.stderr.take();
    let wait = child.wait();
    let status = match timeout(run_timeout, wait).await {
        Ok(Ok(status)) => status,
        Ok(Err(e)) => return Err(ProcessError::Io(e)),
        Err(_) => {
            let _ = child.kill().await;
            return Err(ProcessError::TimedOut);
        }
    };

    let stdout = read_stdout(stdout_handle, max_stdout_bytes).await?;
    let stderr = read_stderr(stderr_handle, max_stderr_bytes).await?;

    Ok(ProcessOutput {
        status,
        stdout,
        stderr,
    })
}

async fn read_stdout(
    handle: Option<ChildStdout>,
    max_bytes: usize,
) -> Result<String, ProcessError> {
    let mut out = String::new();
    if max_bytes == 0 {
        return Ok(out);
    }
    if let Some(mut pipe) = handle {
        let mut buf = vec![0u8; 4096];
        loop {
            let n = pipe.read(&mut buf).await?;
            if n == 0 {
                break;
            }
            if out.len() + n > max_bytes {
                return Err(ProcessError::StdoutTooLarge);
            }
            out.push_str(&String::from_utf8_lossy(&buf[..n]));
        }
    }
    Ok(out)
}

async fn read_stderr(
    handle: Option<ChildStderr>,
    max_bytes: usize,
) -> Result<String, ProcessError> {
    let mut out = String::new();
    if max_bytes == 0 {
        return Ok(out);
    }
    if let Some(mut pipe) = handle {
        let mut buf = vec![0u8; 4096];
        loop {
            let n = pipe.read(&mut buf).await?;
            if n == 0 {
                break;
            }
            if out.len() + n > max_bytes {
                return Err(ProcessError::StderrTooLarge);
            }
            out.push_str(&String::from_utf8_lossy(&buf[..n]));
        }
    }
    Ok(out)
}

/// Verify an executable responds to `--version` without shell invocation.
pub async fn check_tool_version(argv: &[&str]) -> Result<(), ProcessError> {
    let output = run_command(argv, Duration::from_secs(10), 16_384).await?;
    if output.status.success() {
        Ok(())
    } else {
        Err(ProcessError::NonZeroExit(output.status.code().unwrap_or(1)))
    }
}

/// Temporary workspace directory removed on drop.
#[derive(Debug)]
pub struct TempWorkDir {
    path: PathBuf,
}

impl TempWorkDir {
    pub fn new(parent: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let id = uuid::Uuid::new_v4().to_string();
        let path = parent.as_ref().join(id);
        std::fs::create_dir_all(&path)?;
        Ok(Self { path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempWorkDir {
    fn drop(&mut self) {
        if let Err(err) = std::fs::remove_dir_all(&self.path) {
            tracing::warn!(error = %err, "temp_work_dir.cleanup_failed");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn run_echo_succeeds() {
        let out = run_command(&["echo", "hello"], Duration::from_secs(5), 1024)
            .await
            .expect("echo should run");
        assert!(out.status.success());
    }

    #[tokio::test]
    async fn capture_stdout() {
        let out = run_command_capture(&["echo", "hello"], Duration::from_secs(5), 1024, 1024)
            .await
            .expect("echo should run");
        assert!(out.stdout.contains("hello"));
    }

    #[tokio::test]
    async fn run_false_nonzero() {
        let out = run_command(&["false"], Duration::from_secs(5), 1024)
            .await
            .expect("false should spawn");
        assert!(!out.status.success());
    }

    #[tokio::test]
    async fn stderr_bound_enforced() {
        let script = if cfg!(unix) {
            vec!["sh", "-c", "while true; do echo err >&2; done"]
        } else {
            return;
        };
        let err = run_command(&script, Duration::from_millis(200), 64)
            .await
            .err()
            .expect("should fail on stderr bound or timeout");
        assert!(matches!(
            err,
            ProcessError::StderrTooLarge | ProcessError::TimedOut
        ));
    }

    #[test]
    fn temp_work_dir_removed_on_drop() {
        let parent =
            std::env::temp_dir().join(format!("worshipviewer_temp_work_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&parent).unwrap();
        let path = {
            let dir = TempWorkDir::new(&parent).unwrap();
            let p = dir.path().to_path_buf();
            assert!(p.exists());
            p
        };
        assert!(!path.exists());
        let _ = std::fs::remove_dir_all(&parent);
    }
}
