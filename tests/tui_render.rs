//! A deterministic visual smoke test: Ratatui draws into memory, so CI needs no terminal.
use forge::core::exercise::Exercise;
use forge::core::progress::UserProgress;
use forge::storage::local::LocalStorage;
use forge::tui::app::{App, AppMode};
use forge::tui::ui;
use forge::tui::widgets::editor::EditorState;
use ratatui::{backend::TestBackend, Terminal};

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

#[test]
fn home_list_and_editor_render_without_a_real_terminal() {
    let temp = tempfile::tempdir().unwrap();
    let mut app = App {
        mode: AppMode::Home,
        previous_mode: AppMode::Home,
        exercises: vec![
            exercise("java_01", "java", "Java intro"),
            exercise("rust_01", "rust", "Rust intro"),
        ],
        language: None,
        home_selection: 0,
        selected_index: 0,
        progress: UserProgress::default(),
        storage: LocalStorage::at(temp.path().to_path_buf()),
        editor_state: EditorState::default(),
        diagnostics: Vec::new(),
        current_output: String::new(),
        feedback: String::new(),
        revealed_hints: 0,
        has_run: false,
        should_quit: false,
    };
    let mut terminal = Terminal::new(TestBackend::new(100, 32)).unwrap();

    terminal.draw(|frame| ui::render(frame, &mut app)).unwrap();
    let home = screen(&terminal);
    assert!(home.contains("FORGE"));
    assert!(home.contains("JAVA"));
    assert!(home.contains("RUST"));

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
