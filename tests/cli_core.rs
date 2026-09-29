use forge::core::exercise::Exercise;
use forge::core::progress::Status;
use forge::storage::local::LocalStorage;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn fixture(root: &TempDir, expected: &str) {
    let dir = root.path().join("exercises");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("lesson.yaml"), format!(
        "id: demo_01\ntitle: First step\nlanguage: rust\ndifficulty: Beginner\ndescription: Print hello\ntemplate: 'fn main() {{ println!(\"hello\"); }}'\nexpected_output: '{expected}'\nhints:\n  - Use println!\n"
    )).unwrap();
}

fn forge(root: &TempDir, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_forge"))
        .args(args)
        .current_dir(root.path())
        .env("FORGE_HOME", root.path())
        .env("FORGE_PROGRESS_DIR", root.path().join("progress"))
        .output()
        .unwrap()
}

#[test]
fn invalid_catalog_is_reported() {
    let root = TempDir::new().unwrap();
    let dir = root.path().join("exercises");
    fs::create_dir(&dir).unwrap();
    fs::write(dir.join("bad.yaml"), "id: [not an id").unwrap();
    let err = Exercise::load_all_from_dir(&dir).unwrap_err().to_string();
    assert!(err.contains("bad.yaml"), "{err}");
    assert!(err.contains("YAML"), "{err}");
}

#[test]
fn duplicate_ids_are_rejected() {
    let root = TempDir::new().unwrap();
    fixture(&root, "hello");
    fs::copy(
        root.path().join("exercises/lesson.yaml"),
        root.path().join("exercises/copy.yaml"),
    )
    .unwrap();
    let err = Exercise::load_all_from_dir(&root.path().join("exercises"))
        .unwrap_err()
        .to_string();
    assert!(err.contains("Duplicate exercise ID"), "{err}");
}

#[test]
fn cli_discovers_catalog_and_reports_failures() {
    let root = TempDir::new().unwrap();
    fixture(&root, "hello");
    let list = forge(&root, &["list", "--language", "rust"]);
    assert!(
        list.status.success(),
        "{}",
        String::from_utf8_lossy(&list.stderr)
    );
    assert!(String::from_utf8_lossy(&list.stdout).contains("demo_01"));

    let show = forge(&root, &["show", "demo_01"]);
    assert!(show.status.success());
    assert!(String::from_utf8_lossy(&show.stdout).contains("Use println!"));

    let invalid = forge(&root, &["lesson", "python"]);
    assert!(!invalid.status.success());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("Unknown language"));

    let source = root.path().join("answer.rs");
    fs::write(&source, "fn main() { println!(\"wrong\"); }").unwrap();
    let failed = forge(
        &root,
        &["run", "demo_01", "--file", source.to_str().unwrap()],
    );
    assert!(!failed.status.success());
    let storage = LocalStorage::at(root.path().join("progress"));
    let progress = storage.load_progress().unwrap();
    assert_eq!(progress.get_status("demo_01"), Status::InProgress);
    assert_eq!(progress.exercises["demo_01"].attempts, 1);

    fs::write(&source, "fn main() { println!(\"hello\"); }").unwrap();
    let passed = forge(
        &root,
        &["run", "demo_01", "--file", source.to_str().unwrap()],
    );
    assert!(
        passed.status.success(),
        "{}",
        String::from_utf8_lossy(&passed.stderr)
    );
    let progress = storage.load_progress().unwrap();
    assert_eq!(progress.get_status("demo_01"), Status::Completed);
    assert_eq!(progress.exercises["demo_01"].attempts, 2);

    let summary = forge(&root, &["progress"]);
    assert!(String::from_utf8_lossy(&summary.stdout).contains("Completed: 1/1"));
}

#[test]
fn test_command_validates_starter_catalog() {
    let root = TempDir::new().unwrap();
    fixture(&root, "other");
    let output = forge(&root, &["test"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed, 0 failed"));
}

#[test]
fn progress_export_and_reset_preserve_only_requested_records() {
    let root = TempDir::new().unwrap();
    let storage = LocalStorage::at(root.path().join("progress"));
    let mut progress = storage.load_progress().unwrap();
    progress.record_failed_attempt("rust_01", "line,\"quoted\"\nnext");
    progress.mark_completed("java_01", "class Main {}\n");
    storage.save_progress(&progress).unwrap();

    let json = forge(&root, &["progress", "export"]);
    assert!(json.status.success());
    let exported: forge::core::progress::UserProgress =
        serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(exported.exercises["rust_01"].attempts, 1);
    assert_eq!(exported.exercises["java_01"].status, Status::Completed);
    assert_eq!(
        exported.exercises["rust_01"].last_code,
        progress.exercises["rust_01"].last_code
    );

    let csv = forge(&root, &["progress", "export", "--format", "csv"]);
    assert!(csv.status.success());
    let csv = String::from_utf8(csv.stdout).unwrap();
    assert!(csv.starts_with("exercise_id,status,attempts,last_code\r\n"));
    assert!(csv.find("java_01").unwrap() < csv.find("rust_01").unwrap());
    assert!(csv.contains("\"line,\"\"quoted\"\"\nnext\""));

    let refused = forge(&root, &["progress", "reset"]);
    assert!(!refused.status.success());
    assert_eq!(storage.load_progress().unwrap().exercises.len(), 2);

    let one = forge(&root, &["progress", "reset", "--exercise", "rust_01"]);
    assert!(one.status.success());
    let remaining = storage.load_progress().unwrap();
    assert!(!remaining.exercises.contains_key("rust_01"));
    assert_eq!(remaining.exercises["java_01"].status, Status::Completed);

    let missing = forge(&root, &["progress", "reset", "--exercise", "rust_01"]);
    assert!(!missing.status.success());

    let all = forge(&root, &["progress", "reset", "--yes"]);
    assert!(all.status.success());
    assert!(!storage.progress_path().exists());
    assert!(storage.load_progress().unwrap().exercises.is_empty());

    let empty = forge(&root, &["progress", "export"]);
    assert!(empty.status.success());
    assert!(String::from_utf8_lossy(&empty.stdout).contains("\"exercises\": {}"));
}
