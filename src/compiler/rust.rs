use crate::compiler::diagnostics::{Diagnostic, Severity};
use crate::core::errors::{ForgeError, Result};
use crate::lang::traits::{CompileOutput, RunOutput};
use serde_json::Value;
use std::path::Path;
use std::process::Command;

pub struct RustCompiler;

impl RustCompiler {
    pub fn new() -> Self {
        Self
    }

    pub fn compile(&self, code: &str, workdir: &Path) -> Result<CompileOutput> {
        let file_path = workdir.join("main.rs");
        std::fs::write(&file_path, code)?;

        let exe_path = workdir.join("main_bin");

        let output = Command::new("rustc")
            .arg("--error-format=json")
            .arg("-o")
            .arg(&exe_path)
            .arg(&file_path)
            .output();

        match output {
            Ok(out) => {
                let success = out.status.success();
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();

                Ok(CompileOutput {
                    success,
                    stdout,
                    stderr,
                    executable: if success { Some(exe_path) } else { None },
                })
            }
            Err(e) => Err(ForgeError::Toolchain(format!(
                "Failed to execute rustc: {e}. Is Rust installed?"
            ))),
        }
    }

    pub fn run(&self, workdir: &Path) -> Result<RunOutput> {
        let exe_path = workdir.join("main_bin");
        let output = Command::new(&exe_path).output();

        match output {
            Ok(out) => Ok(RunOutput {
                success: out.status.success(),
                stdout: String::from_utf8_lossy(&out.stdout).to_string(),
                stderr: String::from_utf8_lossy(&out.stderr).to_string(),
                exit_code: out.status.code(),
            }),
            Err(e) => Err(ForgeError::Toolchain(format!("Failed to run binary: {e}"))),
        }
    }

    pub fn parse_errors(&self, output: &CompileOutput) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for line in output.stderr.lines() {
            if let Ok(v) = serde_json::from_str::<Value>(line) {
                if v["reason"] == "compiler-message" || v["level"].is_string() {
                    let level = v["level"].as_str().unwrap_or("error");
                    let message = v["message"].as_str().unwrap_or("").to_string();

                    let severity = match level {
                        "warning" => Severity::Warning,
                        "note" | "help" => Severity::Hint,
                        _ => Severity::Error,
                    };

                    let mut line_num = 1;
                    let mut col_num = 1;
                    let mut suggestion = None;

                    if let Some(spans) = v["spans"].as_array() {
                        if let Some(primary) = spans.iter().find(|s| s["is_primary"] == true) {
                            line_num = primary["line_start"].as_u64().unwrap_or(1) as usize;
                            col_num = primary["column_start"].as_u64().unwrap_or(1) as usize;
                            if let Some(label) = primary["label"].as_str() {
                                suggestion = Some(label.to_string());
                            }
                        }
                    }

                    if suggestion.is_none() {
                        if let Some(children) = v["children"].as_array() {
                            for child in children {
                                if child["level"] == "help" {
                                    if let Some(msg) = child["message"].as_str() {
                                        suggestion = Some(format!("Help: {msg}"));
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    if !message.is_empty() {
                        let mut diag = Diagnostic::new(severity, line_num, col_num, message);
                        diag.suggestion = suggestion;
                        diagnostics.push(diag);
                    }
                }
            }
        }

        diagnostics
    }
}
