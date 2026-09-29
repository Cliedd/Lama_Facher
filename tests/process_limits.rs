#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Output};
use std::time::{Duration, Instant};
use tempfile::TempDir;

fn exercise(root: &Path, language: &str) {
    let dir = root.join("exercises");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("loop.yaml"),
        format!("id: loop\ntitle: Loop\nlanguage: {language}\ndifficulty: Beginner\ndescription: Stop the loop\ntemplate: placeholder\nexpected_output: done\n"),
    )
    .unwrap();
}

fn forge(root: &Path, source: &Path, variables: &[(&str, &str)]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_forge"));
    command
        .args(["run", "loop", "--file"])
        .arg(source)
        .current_dir(root)
        .env("FORGE_HOME", root)
        .env("FORGE_PROGRESS_DIR", root.join("progress"));
    for (key, value) in variables {
        command.env(key, value);
    }
    command.output().unwrap()
}

#[test]
fn rust_program_timeout_is_reported_promptly() {
    let root = TempDir::new().unwrap();
    exercise(root.path(), "rust");
    let source = root.path().join("answer.rs");
    fs::write(&source, "fn main() { loop {} }").unwrap();
    let start = Instant::now();
    let output = forge(root.path(), &source, &[("FORGE_RUN_TIMEOUT_MS", "200")]);
    assert!(!output.status.success());
    assert!(start.elapsed() < Duration::from_secs(5));
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(
        error.contains("execution")
            && error.contains("200 ms")
            && error.contains("FORGE_RUN_TIMEOUT_MS"),
        "{error}"
    );
}

#[test]
fn rust_compiler_timeout_is_reported_promptly() {
    let root = TempDir::new().unwrap();
    exercise(root.path(), "rust");
    let source = root.path().join("answer.rs");
    fs::write(&source, "fn main() {}").unwrap();
    let bin = root.path().join("bin");
    fs::create_dir(&bin).unwrap();
    let mock = bin.join("rustc");
    fs::write(&mock, "#!/bin/sh\nsleep 30\n").unwrap();
    fs::set_permissions(&mock, fs::Permissions::from_mode(0o755)).unwrap();
    let path = format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let start = Instant::now();
    let output = forge(
        root.path(),
        &source,
        &[("FORGE_COMPILE_TIMEOUT_MS", "200"), ("PATH", &path)],
    );
    assert!(!output.status.success());
    assert!(start.elapsed() < Duration::from_secs(5));
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(
        error.contains("compilation")
            && error.contains("200 ms")
            && error.contains("FORGE_COMPILE_TIMEOUT_MS"),
        "{error}"
    );
}

#[test]
fn java_program_timeout_is_reported_when_jdk_is_available() {
    if Command::new("javac").arg("-version").output().is_err() {
        return;
    }
    let root = TempDir::new().unwrap();
    exercise(root.path(), "java");
    let source = root.path().join("Main.java");
    fs::write(
        &source,
        "public class Main { public static void main(String[] args) { while (true) {} } }",
    )
    .unwrap();
    let start = Instant::now();
    let output = forge(root.path(), &source, &[("FORGE_RUN_TIMEOUT_MS", "200")]);
    assert!(!output.status.success());
    assert!(start.elapsed() < Duration::from_secs(10));
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(
        error.contains("execution") && error.contains("FORGE_RUN_TIMEOUT_MS"),
        "{error}"
    );
}
