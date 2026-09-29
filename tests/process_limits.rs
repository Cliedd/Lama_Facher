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
fn excessive_output_is_stopped_and_reported() {
    let root = TempDir::new().unwrap();
    exercise(root.path(), "rust");
    let source = root.path().join("answer.rs");
    fs::write(
        &source,
        "fn main() { loop { println!(\"{}\", \"x\".repeat(4096)); } }",
    )
    .unwrap();
    let start = Instant::now();
    let output = forge(root.path(), &source, &[]);
    assert!(!output.status.success());
    assert!(start.elapsed() < Duration::from_secs(8));
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(
        error.contains("1024 KiB") && error.contains("output"),
        "{error}"
    );
}

#[test]
fn combined_stdout_and_stderr_share_the_output_limit() {
    let root = TempDir::new().unwrap();
    exercise(root.path(), "rust");
    let source = root.path().join("answer.rs");
    fs::write(
        &source,
        "fn main() { use std::io::Write; let bytes = vec![b'x'; 600 * 1024]; std::io::stdout().write_all(&bytes).unwrap(); std::io::stderr().write_all(&bytes).unwrap(); }",
    )
    .unwrap();
    let output = forge(root.path(), &source, &[]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("1024 KiB"));
}

#[test]
fn learner_program_does_not_inherit_secrets_and_uses_a_temporary_directory() {
    let root = TempDir::new().unwrap();
    exercise(root.path(), "rust");
    let source = root.path().join("answer.rs");
    fs::write(
        &source,
        "fn main() { assert!(std::env::var_os(\"FORGE_TEST_SECRET\").is_none()); let path = std::env::temp_dir().join(\"learner-probe\"); std::fs::write(&path, b\"ok\").unwrap(); assert_eq!(std::env::current_dir().unwrap(), path.parent().unwrap()); println!(\"done\"); }",
    )
    .unwrap();
    let output = forge(
        root.path(),
        &source,
        &[("FORGE_TEST_SECRET", "private-value")],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!root.path().join("learner-probe").exists());
}

#[test]
fn learner_file_creation_is_bounded() {
    let root = TempDir::new().unwrap();
    exercise(root.path(), "rust");
    let source = root.path().join("answer.rs");
    fs::write(
        &source,
        "fn main() { use std::io::Write; let mut f = std::fs::File::create(\"big.bin\").unwrap(); let bytes = vec![0u8; 2 * 1024 * 1024]; if f.write_all(&bytes).is_err() { println!(\"done\"); } }",
    )
    .unwrap();
    let output = forge(root.path(), &source, &[]);
    assert!(!output.status.success());
    assert!(!root.path().join("big.bin").exists());
}

#[test]
fn relative_files_are_written_in_the_temporary_workdir() {
    let root = TempDir::new().unwrap();
    exercise(root.path(), "rust");
    let source = root.path().join("answer.rs");
    fs::write(
        &source,
        "fn main() { std::fs::write(\"probe.txt\", \"x\").unwrap(); println!(\"done\"); }",
    )
    .unwrap();
    let output = forge(root.path(), &source, &[]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!root.path().join("probe.txt").exists());
}

#[cfg(target_os = "linux")]
#[test]
fn native_program_cannot_reserve_more_than_the_memory_limit() {
    let root = TempDir::new().unwrap();
    exercise(root.path(), "rust");
    let source = root.path().join("answer.rs");
    fs::write(&source, "fn main() { let mut data: Vec<u8> = Vec::new(); if data.try_reserve_exact(700 * 1024 * 1024).is_err() { println!(\"done\"); } else { println!(\"unlimited\"); } }").unwrap();
    let output = forge(root.path(), &source, &[]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
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
