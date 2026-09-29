pub mod diagnostics;
pub mod java;
pub mod rust;

use crate::core::errors::{ForgeError, Result};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
pub(crate) enum Phase {
    Compile,
    Run,
}

impl Phase {
    fn settings(self) -> (&'static str, &'static str, u64) {
        match self {
            Self::Compile => ("compilation", "FORGE_COMPILE_TIMEOUT_MS", 30_000),
            Self::Run => ("execution", "FORGE_RUN_TIMEOUT_MS", 5_000),
        }
    }
}

fn timeout(phase: Phase) -> Result<Duration> {
    let (_, variable, default_ms) = phase.settings();
    let millis = match std::env::var(variable) {
        Ok(value) => value
            .parse::<u64>()
            .ok()
            .filter(|ms| *ms > 0)
            .ok_or_else(|| {
                ForgeError::Generic(format!(
                    "{variable} must be a positive number of milliseconds"
                ))
            })?,
        Err(std::env::VarError::NotPresent) => default_ms,
        Err(_) => {
            return Err(ForgeError::Generic(format!(
                "{variable} must be valid UTF-8"
            )))
        }
    };
    Ok(Duration::from_millis(millis))
}

pub(crate) fn run_command(command: &mut Command, phase: Phase) -> Result<Output> {
    let limit = timeout(phase)?;
    let (phase_name, variable, _) = phase.settings();
    let program = command.get_program().to_string_lossy().into_owned();
    let mut stdout = tempfile::tempfile()?;
    let mut stderr = tempfile::tempfile()?;
    command
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout.try_clone()?))
        .stderr(Stdio::from(stderr.try_clone()?));

    // A separate process group lets a timeout stop descendants as well as the direct child.
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }

    let mut child = command.spawn()?;
    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if start.elapsed() >= limit => {
                stop_child(&mut child);
                return Err(ForgeError::Timeout {
                    phase: phase_name,
                    program,
                    timeout_ms: limit.as_millis() as u64,
                    variable,
                });
            }
            Ok(None) => std::thread::sleep(
                Duration::from_millis(10).min(limit.saturating_sub(start.elapsed())),
            ),
            Err(error) => {
                stop_child(&mut child);
                return Err(error.into());
            }
        }
    };
    Ok(Output {
        status,
        stdout: read_output(&mut stdout)?,
        stderr: read_output(&mut stderr)?,
    })
}

fn read_output(file: &mut File) -> Result<Vec<u8>> {
    file.seek(SeekFrom::Start(0))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn stop_child(child: &mut std::process::Child) {
    #[cfg(unix)]
    unsafe {
        unsafe extern "C" {
            fn kill(pid: i32, signal: i32) -> i32;
        }
        // The child is the leader of the process group created above.
        kill(-(child.id() as i32), 9);
    }
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/F", "/T", "/PID", &child.id().to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    let _ = child.kill();
    let _ = child.wait();
}
