use forge::core::exercise::Exercise;
use forge::core::session::Session;
use forge::lang::get_adapter;
use std::path::PathBuf;
use std::process::Command;

fn catalog_file(language: &str, file: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("exercises")
        .join(language)
        .join(file)
}

#[test]
fn sixty_exercises_load_and_extra_cases_have_unique_names() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("exercises");
    let catalog = Exercise::load_all_from_dir(&root).unwrap();
    assert_eq!(catalog.len(), 60);
    assert_eq!(
        catalog
            .iter()
            .filter(|exercise| !exercise.test_cases.is_empty())
            .count(),
        16
    );
    for exercise in catalog {
        let mut names = std::collections::HashSet::new();
        for case in exercise.test_cases {
            assert!(names.insert(case.name));
        }
    }
}

fn assert_function_cases(language: &str, file: &str, correct: &str, fixed: &str) {
    let exercise = Exercise::load_from_file(&catalog_file(language, file)).unwrap();
    assert!(exercise.test_cases.len() >= 3);
    let adapter = get_adapter(language).unwrap();
    let session = Session::new(exercise.clone(), adapter.clone()).unwrap();
    assert!(session.compile(correct).unwrap().success);
    assert!(session.verify(&session.run().unwrap()));
    let session = Session::new(exercise, adapter).unwrap();
    assert!(session.compile(fixed).unwrap().success);
    assert!(!session.verify(&session.run().unwrap()));
}

#[test]
fn function_cases_check_zero_negative_and_larger_arguments() {
    assert_function_cases(
        "rust",
        "06_function.yaml",
        "fn double_value(value: i32) -> i32 { value * 2 } fn main() { println!(\"{}\", double_value(7)); }",
        "fn double_value(_value: i32) -> i32 { 14 } fn main() { println!(\"{}\", double_value(7)); }",
    );
    if Command::new("javac").arg("-version").output().is_ok() {
        assert_function_cases(
            "java",
            "06_method.yaml",
            "public class Main { static int doubleValue(int value) { return value * 2; } public static void main(String[] args) { System.out.println(doubleValue(7)); } }",
            "public class Main { static int doubleValue(int value) { return 14; } public static void main(String[] args) { System.out.println(doubleValue(7)); } }",
        );
    }
}
