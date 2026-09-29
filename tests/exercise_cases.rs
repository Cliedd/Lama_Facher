use forge::core::exercise::Exercise;
use forge::core::session::Session;
use forge::lang;
use std::fs;
use tempfile::TempDir;

fn grade(exercise: Exercise, source: &str) -> bool {
    let adapter = lang::get_adapter(&exercise.language).unwrap();
    let session = Session::new(exercise, adapter).unwrap();
    assert!(session.compile(source).unwrap().success);
    session.verify(&session.run().unwrap())
}

#[test]
fn old_yaml_remains_valid_without_extra_cases() {
    let exercise: Exercise = serde_yaml::from_str(
        "id: old\ntitle: Old\nlanguage: rust\ndifficulty: Beginner\ndescription: Old catalog\ntemplate: 'fn main() {}'\nexpected_output: ok\n",
    )
    .unwrap();
    assert!(exercise.test_cases.is_empty());
    assert!(grade(exercise, "fn main() { println!(\"ok\"); }"));
}

#[test]
fn rust_boundary_cases_reject_a_fixed_answer() {
    let exercise =
        Exercise::load_from_file(std::path::Path::new("exercises/rust/04_condition.yaml")).unwrap();
    assert!(exercise.test_cases.len() >= 2);
    let fixed = "fn main() { let age = 16; println!(\"5\"); }";
    assert!(!grade(exercise.clone(), fixed));
    let solution = "fn main() { let age = 16; let price = if age < 18 { 5 } else { 10 }; println!(\"{price}\"); }";
    assert!(grade(exercise, solution));
}

#[test]
fn java_boundary_cases_reject_a_fixed_answer() {
    let exercise =
        Exercise::load_from_file(std::path::Path::new("exercises/java/04_condition.yaml")).unwrap();
    assert!(exercise.test_cases.len() >= 2);
    let fixed = "public class Main { public static void main(String[] args) { int age = 16; System.out.println(5); } }";
    assert!(!grade(exercise.clone(), fixed));
    let solution = "public class Main { public static void main(String[] args) { int age = 16; int price = age < 18 ? 5 : 10; System.out.println(price); } }";
    assert!(grade(exercise, solution));
}

#[test]
fn catalog_rejects_a_case_without_a_unique_anchor() {
    let root = TempDir::new().unwrap();
    fs::write(
        root.path().join("bad.yaml"),
        "id: bad\ntitle: Bad\nlanguage: rust\ndifficulty: Beginner\ndescription: Test\ntemplate: 'fn main() {}'\nexpected_output: ok\ntest_cases:\n  - name: boundary\n    replace: 'let x = 1;'\n    with: 'let x = 2;'\n    expected_output: ok\n",
    )
    .unwrap();
    let error = Exercise::load_all_from_dir(root.path())
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("replacement must occur exactly once"),
        "{error}"
    );
}

#[test]
fn failed_recompile_cannot_run_an_earlier_solution() {
    let exercise: Exercise = serde_yaml::from_str(
        "id: old\ntitle: Old\nlanguage: rust\ndifficulty: Beginner\ndescription: Old catalog\ntemplate: 'fn main() {}'\nexpected_output: ok\n",
    )
    .unwrap();
    let adapter = lang::get_adapter("rust").unwrap();
    let session = Session::new(exercise, adapter).unwrap();
    assert!(session.run().is_err());
    assert!(
        session
            .compile("fn main() { println!(\"ok\"); }")
            .unwrap()
            .success
    );
    let earlier = session.run().unwrap();
    assert!(earlier.success);
    assert!(
        !session
            .compile("fn main() { this will not compile }")
            .unwrap()
            .success
    );
    assert!(session.run().is_err());
    assert!(!session.verify(&earlier));
}
