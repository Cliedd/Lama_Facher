use forge::compiler::diagnostics::{Diagnostic, Severity};
use forge::core::exercise::Exercise;
use forge::core::progress::{Status, UserProgress};

#[test]
fn test_exercise_struct_parsing() {
    let yaml = r#"
id: "test_01"
title: "Test Exercise"
language: "rust"
difficulty: "Beginner"
description: "Testing parsing"
template: "fn main() {}"
expected_output: "ok"
hints:
  - "hint 1"
"#;
    let ex: Exercise = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(ex.id, "test_01");
    assert_eq!(ex.language, "rust");
    assert_eq!(ex.expected_output, Some("ok".to_string()));
}

#[test]
fn test_progress_tracking() {
    let mut progress = UserProgress::default();
    assert_eq!(progress.get_status("ex1"), Status::NotStarted);

    progress.save_draft("ex1", "code1");
    assert_eq!(progress.get_status("ex1"), Status::InProgress);

    progress.mark_completed("ex1", "code2");
    assert_eq!(progress.get_status("ex1"), Status::Completed);
}

#[test]
fn test_diagnostic_creation() {
    let diag = Diagnostic::new(Severity::Error, 10, 5, "Missing semicolon".into());
    assert!(matches!(diag.severity, Severity::Error));
    assert_eq!(diag.line, 10);
    assert_eq!(diag.column, 5);
}
