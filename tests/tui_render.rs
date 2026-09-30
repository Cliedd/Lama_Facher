//! A deterministic visual smoke test: Ratatui draws into memory, so CI needs no terminal.
use forge::core::exercise::Exercise;
use forge::core::progress::UserProgress;
use forge::core::settings::Locale;
use forge::storage::local::LocalStorage;
use forge::tui::app::{App, AppMode};
use forge::tui::ui;
use forge::tui::widgets::editor::EditorState;
use ratatui::{backend::TestBackend, style::Color, Terminal};
use std::time::{Duration, Instant};

fn exercise(id: &str, language: &str, title: &str) -> Exercise {
    serde_yaml::from_str(&format!(
        "id: {id}\ntitle: {title}\nlanguage: {language}\ndifficulty: Beginner\ndescription: Print a greeting\ntemplate: 'fn main() {{}}'\nexpected_output: hello\nhints:\n  - Try a print statement\n"
    ))
    .unwrap()
}

fn screen(terminal: &Terminal<TestBackend>) -> String {
    terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>()
}

fn test_app() -> App {
    let temp = tempfile::tempdir().unwrap();
    App {
        mode: AppMode::Home,
        previous_mode: AppMode::Home,
        exercises: vec![
            exercise("java_01", "java", "Java intro"),
            exercise("rust_01", "rust", "Rust intro"),
        ],
        language: None,
        home_selection: 0,
        locale_selection: 0,
        selected_index: 0,
        progress: UserProgress::default(),
        storage: LocalStorage::at(temp.path().to_path_buf()),
        locale: Locale::Fr,
        editor_state: EditorState::default(),
        diagnostics: Vec::new(),
        current_output: String::new(),
        feedback: String::new(),
        revealed_hints: 0,
        has_run: false,
        should_quit: false,
        pending: None,
    }
}

#[test]
fn home_list_and_editor_render_without_a_real_terminal() {
    let mut app = test_app();
    let mut terminal = Terminal::new(TestBackend::new(100, 32)).unwrap();

    terminal.draw(|frame| ui::render(frame, &mut app)).unwrap();
    let home = screen(&terminal);
    assert!(home.contains("FORGE"));
    assert!(home.contains("JAVA"));
    assert!(home.contains("RUST"));
    assert!(terminal
        .backend()
        .buffer()
        .content
        .iter()
        .any(|cell| { cell.symbol() == "F" && cell.fg == Color::Cyan }));

    app.home_selection = 1;
    app.choose_language();
    terminal.draw(|frame| ui::render(frame, &mut app)).unwrap();
    let list = screen(&terminal);
    assert!(list.contains("Rust intro"));
    assert!(list.contains("Détails de l'exercice"));

    app.enter_editor();
    terminal.draw(|frame| ui::render(frame, &mut app)).unwrap();
    let editor = screen(&terminal);
    assert!(editor.contains("Objectif & indices"));
    assert!(editor.contains("Diagnostics"));
    assert!(editor.contains("Résultat"));
}

#[test]
fn editor_render_is_stable_across_sizes_and_shows_feedback() {
    let mut app = test_app();
    app.choose_language();
    app.enter_editor();
    app.feedback = "Exécution interrompue : sortie limitée".into();
    app.current_output = "La sortie a dépassé la limite".into();
    app.revealed_hints = 1;

    for (width, height) in [(100, 32), (60, 20), (30, 8)] {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|frame| ui::render(frame, &mut app)).unwrap();
        let first = screen(&terminal);
        terminal.draw(|frame| ui::render(frame, &mut app)).unwrap();
        assert_eq!(
            first,
            screen(&terminal),
            "unstable render at {width}x{height}"
        );
        if width < 40 {
            assert!(first.contains("Agrandis le terminal"));
        } else {
            assert!(first.contains("Indice 1/1"));
            assert!(first.contains("Exécution interrompue"));
        }
    }
}

#[test]
fn compilation_keeps_the_editor_responsive_until_result_arrives() {
    let temp = tempfile::tempdir().unwrap();
    let mut app = test_app();
    app.storage = LocalStorage::at(temp.path().to_path_buf());
    app.home_selection = 1;
    app.choose_language();
    app.enter_editor();
    app.editor_state = EditorState::new("fn main() { println!(\"hello\"); }", "rust");

    let start = Instant::now();
    app.compile_current();
    assert!(start.elapsed() < Duration::from_secs(1));
    assert!(app.pending.is_some());
    let mut terminal = Terminal::new(TestBackend::new(100, 32)).unwrap();
    terminal.draw(|frame| ui::render(frame, &mut app)).unwrap();
    assert!(screen(&terminal).contains("en cours"));

    while app.pending.is_some() && start.elapsed() < Duration::from_secs(10) {
        app.poll_run();
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(app.pending.is_none(), "the run never finished");
    assert!(app.feedback.contains("Réussi"), "{}", app.feedback);
}
