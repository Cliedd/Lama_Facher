use crate::compiler::diagnostics::{Diagnostic, Severity};
use crate::compiler::{run_command, Phase};
use crate::core::errors::{ForgeError, Result};
use crate::lang::traits::{CompileOutput, RunOutput};
use regex::Regex;
use std::path::Path;
use std::process::Command;

pub struct JavaCompiler;

impl JavaCompiler {
    pub fn new() -> Self {
        Self
    }

    pub fn compile(&self, code: &str, workdir: &Path) -> Result<CompileOutput> {
        let file_path = workdir.join("Main.java");
        std::fs::write(&file_path, code)?;

        let output = run_command(
            Command::new("javac")
                .arg("-d")
                .arg(workdir)
                .arg(&file_path)
                .current_dir(workdir),
            Phase::Compile,
        );

        match output {
            Ok(out) => {
                let success = out.status.success();
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();

                Ok(CompileOutput {
                    success,
                    stdout,
                    stderr,
                    executable: if success {
                        Some(workdir.join("Main.class"))
                    } else {
                        None
                    },
                })
            }
            Err(ForgeError::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => Err(
                ForgeError::Toolchain("Failed to execute javac. Is JDK installed?".into()),
            ),
            Err(e) => Err(e),
        }
    }

    pub fn run(&self, workdir: &Path) -> Result<RunOutput> {
        let output = run_command(
            Command::new("java")
                .arg("-Xmx256m")
                .arg("-XX:MaxMetaspaceSize=128m")
                .arg(format!("-Djava.io.tmpdir={}", workdir.display()))
                .arg("-cp")
                .arg(workdir)
                .arg("Main")
                .current_dir(workdir),
            Phase::Run,
        );

        match output {
            Ok(out) => Ok(RunOutput {
                success: out.status.success(),
                stdout: String::from_utf8_lossy(&out.stdout).to_string(),
                stderr: String::from_utf8_lossy(&out.stderr).to_string(),
                exit_code: out.status.code(),
            }),
            Err(ForgeError::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => Err(
                ForgeError::Toolchain("Failed to execute java. Is JDK installed?".into()),
            ),
            Err(e) => Err(e),
        }
    }

    pub fn parse_errors(&self, output: &CompileOutput) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let re = Regex::new(r"Main\.java:(\d+):\s*(error|warning):\s*(.+)").unwrap();

        for line in output.stderr.lines() {
            if let Some(caps) = re.captures(line) {
                let line_num: usize = caps.get(1).map_or(1, |m| m.as_str().parse().unwrap_or(1));
                let kind = caps.get(2).map_or("error", |m| m.as_str());
                let msg = caps.get(3).map_or("", |m| m.as_str()).to_string();

                let severity = if kind == "warning" {
                    Severity::Warning
                } else {
                    Severity::Error
                };

                let mut diag = Diagnostic::new(severity, line_num, 1, msg.clone());

                // Add smart beginner-friendly suggestions
                if msg.contains("cannot find symbol") {
                    diag.suggestion =
                        Some("Check variable/method spelling or missing imports.".into());
                } else if msg.contains("';' expected") {
                    diag.suggestion =
                        Some("You forgot a semicolon ';' at the end of the statement.".into());
                } else if msg.contains("reached end of file while parsing") {
                    diag.suggestion = Some("Check for missing closing braces '}'.".into());
                } else if msg.contains("incompatible types") {
                    diag.suggestion = Some(
                        "Type mismatch: ensure variable types match the assigned value.".into(),
                    );
                }

                diagnostics.push(diag);
            }
        }

        diagnostics
    }
}
