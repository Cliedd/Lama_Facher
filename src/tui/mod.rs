pub mod app;
pub mod ui;
pub mod widgets;

use crate::core::path_manager::PathManager;
use app::{App, AppMode};
use crossterm::{
    cursor::Show,
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

struct RestoreTerminal;

impl Drop for RestoreTerminal {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            io::stdout(),
            Show,
            LeaveAlternateScreen,
            DisableMouseCapture
        );
    }
}

pub fn run_tui() -> crate::core::errors::Result<()> {
    PathManager::ensure_toolchain_paths();
    let mut app = App::new()?;
    enable_raw_mode()?;
    let restore = RestoreTerminal;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = main_loop(&mut terminal, &mut app);
    drop(terminal);
    drop(restore);
    result
}

fn main_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> crate::core::errors::Result<()> {
    loop {
        app.poll_run();
        terminal.draw(|f| ui::render(f, app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                if key.code == KeyCode::F(1) {
                    if app.mode == AppMode::Help {
                        app.mode = app.previous_mode;
                    } else {
                        app.show_help();
                    }
                    continue;
                }
                if key.code == KeyCode::F(2) && app.mode == AppMode::Editor {
                    app.reveal_hint();
                    continue;
                }
                if key.code == KeyCode::Char('q') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    app.should_quit = true;
                    break;
                }

                match app.mode {
                    AppMode::Locale => match key.code {
                        KeyCode::Char('q') => app.should_quit = true,
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.locale_selection = app.locale_selection.saturating_sub(1)
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            app.locale_selection = (app.locale_selection + 1).min(1)
                        }
                        KeyCode::Enter => {
                            app.choose_locale()?;
                        }
                        KeyCode::Esc if app.previous_mode != AppMode::Locale => {
                            app.mode = app.previous_mode;
                        }
                        _ => {}
                    },
                    AppMode::Home => match key.code {
                        KeyCode::Char('q') => app.should_quit = true,
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.home_selection = app.home_selection.saturating_sub(1)
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            app.home_selection = (app.home_selection + 1)
                                .min(app.languages().len().saturating_sub(1))
                        }
                        KeyCode::Enter => app.choose_language(),
                        KeyCode::Char('h') | KeyCode::Char('?') => app.show_help(),
                        KeyCode::Char('l') => app.open_locale_picker(),
                        _ => {}
                    },
                    AppMode::List => match key.code {
                        KeyCode::Char('j') | KeyCode::Down => app.move_selection(true),
                        KeyCode::Char('k') | KeyCode::Up => app.move_selection(false),
                        KeyCode::Enter | KeyCode::Char('e') => {
                            app.enter_editor();
                        }
                        KeyCode::Char('h') | KeyCode::Char('?') => app.show_help(),
                        KeyCode::Esc | KeyCode::Char('b') => app.mode = AppMode::Home,
                        KeyCode::Char('l') => app.open_locale_picker(),
                        KeyCode::Char('q') => app.should_quit = true,
                        _ => {}
                    },
                    AppMode::Editor => {
                        if app.pending.is_some() {
                            continue;
                        } else if key.code == KeyCode::Esc {
                            app.save_draft();
                            app.mode = AppMode::List;
                        } else if key.code == KeyCode::Char('r')
                            && key.modifiers.contains(KeyModifiers::CONTROL)
                        {
                            app.feedback = "Compilation et vérification en cours…".into();
                            terminal.draw(|f| ui::render(f, app))?;
                            app.compile_current();
                        } else if key.code == KeyCode::Char('s')
                            && key.modifiers.contains(KeyModifiers::CONTROL)
                        {
                            app.save_draft();
                        } else {
                            app.editor_state.handle_key_event(key);
                        }
                    }
                    AppMode::Help => match key.code {
                        KeyCode::Esc | KeyCode::Enter | KeyCode::Char('h') | KeyCode::Char('?') => {
                            app.mode = app.previous_mode
                        }
                        KeyCode::Char('q') => app.should_quit = true,
                        _ => {}
                    },
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
