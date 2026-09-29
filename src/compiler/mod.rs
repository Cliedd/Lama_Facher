pub mod diagnostics;
pub mod java;
pub mod rust;

use crate::core::errors::{ForgeError, Result};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
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

    fn output_limit(self) -> u64 {
        match self {
            Self::Compile => 8 * 1024 * 1024,
            Self::Run => 1024 * 1024,
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
    let max_output = phase.output_limit();
    let workdir = command
        .get_current_dir()
        .map(Path::to_path_buf)
        .ok_or_else(|| ForgeError::Generic("A temporary work directory is required".into()))?;
    isolate_environment(command, phase, &workdir);
    let mut stdout = tempfile::tempfile_in(&workdir)?;
    let mut stderr = tempfile::tempfile_in(&workdir)?;
    command
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout.try_clone()?))
        .stderr(Stdio::from(stderr.try_clone()?));

    // A separate process group lets a timeout stop descendants as well as the direct child.
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let is_java = command.get_program().to_string_lossy().ends_with("java");
        // Compilers also write executables and class files; allow a larger per-file
        // ceiling there while polling their diagnostic streams at the lower limit.
        let max_file_size = if matches!(phase, Phase::Compile) {
            64 * 1024 * 1024
        } else {
            max_output
        };
        command.process_group(0);
        // Bound each output file even between polling iterations. Keep the compiler's
        // address space unrestricted: rustc and javac legitimately reserve large maps.
        unsafe {
            command.pre_exec(move || {
                let file_limit = libc::rlimit {
                    rlim_cur: max_file_size as libc::rlim_t,
                    rlim_max: max_file_size as libc::rlim_t,
                };
                if libc::setrlimit(libc::RLIMIT_FSIZE, &file_limit) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                if matches!(phase, Phase::Run) && !is_java {
                    let memory_limit = libc::rlimit {
                        rlim_cur: 512 * 1024 * 1024,
                        rlim_max: 512 * 1024 * 1024,
                    };
                    if libc::setrlimit(libc::RLIMIT_AS, &memory_limit) != 0 {
                        return Err(std::io::Error::last_os_error());
                    }
                }
                Ok(())
            });
        }
    }

    let mut child = command.spawn()?;
    let start = Instant::now();
    let status = loop {
        if output_size(&stdout, &stderr)? >= max_output {
            stop_child(&mut child);
            return Err(ForgeError::OutputLimit {
                phase: phase_name,
                program,
                limit_kb: max_output / 1024,
            });
        }
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
    if output_size(&stdout, &stderr)? >= max_output {
        return Err(ForgeError::OutputLimit {
            phase: phase_name,
            program,
            limit_kb: max_output / 1024,
        });
    }
    Ok(Output {
        status,
        stdout: read_output(&mut stdout)?,
        stderr: read_output(&mut stderr)?,
    })
}

fn output_size(stdout: &File, stderr: &File) -> Result<u64> {
    Ok(stdout
        .metadata()?
        .len()
        .saturating_add(stderr.metadata()?.len()))
}

fn isolate_environment(command: &mut Command, phase: Phase, workdir: &Path) {
    // Keep only tool discovery and runtime variables. Learner code can inspect its
    // environment, so it must not inherit tokens or credentials from the shell.
    let preserved: Vec<_> = [
        "PATH",
        "JAVA_HOME",
        "RUSTUP_TOOLCHAIN",
        "LD_LIBRARY_PATH",
        "DYLD_LIBRARY_PATH",
        "SystemRoot",
        "WINDIR",
        "PATHEXT",
    ]
    .into_iter()
    .filter_map(|name| std::env::var_os(name).map(|value| (name, value)))
    .collect();
    let rustup_home = std::env::var_os("RUSTUP_HOME")
        .or_else(|| std::env::var_os("HOME").map(|home| Path::new(&home).join(".rustup").into()));
    let cargo_home = std::env::var_os("CARGO_HOME")
        .or_else(|| std::env::var_os("HOME").map(|home| Path::new(&home).join(".cargo").into()));
    command.env_clear();
    for (name, value) in preserved {
        command.env(name, value);
    }
    if matches!(phase, Phase::Compile) {
        if let Some(value) = rustup_home {
            command.env("RUSTUP_HOME", value);
        }
        if let Some(value) = cargo_home {
            command.env("CARGO_HOME", value);
        }
    }
    command
        .env("HOME", workdir)
        .env("USERPROFILE", workdir)
        .env("TMPDIR", workdir)
        .env("TMP", workdir)
        .env("TEMP", workdir);
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
