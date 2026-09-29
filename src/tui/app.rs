use crate::compiler::diagnostics::Diagnostic;
use crate::core::exercise::Exercise;
use crate::core::progress::{Status, UserProgress};
use crate::lang;
use crate::storage::local::LocalStorage;
use crate::tui::widgets::editor::EditorState;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Home,
    List,
    Editor,
    Help,
}

pub struct App {
    pub mode: AppMode,
    pub previous_mode: AppMode,
    pub exercises: Vec<Exercise>,
    pub language: Option<String>,
    pub home_selection: usize,
    pub selected_index: usize,
    pub progress: UserProgress,
    pub storage: LocalStorage,
    pub editor_state: EditorState,
    pub diagnostics: Vec<Diagnostic>,
    pub current_output: String,
    pub feedback: String,
    pub revealed_hints: usize,
    pub has_run: bool,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> crate::core::errors::Result<Self> {
        let storage = LocalStorage::new()?;
        let progress = storage.load_progress()?;
        let exercises = Exercise::load_all_from_dir(&crate::config::exercises_dir())?;
        Ok(Self {
            mode: AppMode::Home,
            previous_mode: AppMode::Home,
            exercises,
            language: None,
            home_selection: 0,
            selected_index: 0,
            progress,
            storage,
            editor_state: EditorState::default(),
            diagnostics: Vec::new(),
            current_output: String::new(),
            feedback: String::new(),
            revealed_hints: 0,
            has_run: false,
            should_quit: false,
        })
    }

    pub fn languages(&self) -> Vec<String> {
        let mut languages: Vec<String> =
            self.exercises.iter().map(|e| e.language.clone()).collect();
        languages.sort();
        languages.dedup();
        languages
    }

    pub fn visible_exercises(&self) -> Vec<&Exercise> {
        self.exercises
            .iter()
            .filter(|e| {
                self.language
                    .as_ref()
                    .is_none_or(|lang| &e.language == lang)
            })
            .collect()
    }

    pub fn selected_exercise(&self) -> Option<&Exercise> {
        self.visible_exercises().get(self.selected_index).copied()
    }

    pub fn choose_language(&mut self) {
        if let Some(language) = self.languages().get(self.home_selection).cloned() {
            self.language = Some(language);
            self.selected_index = 0;
            self.mode = AppMode::List;
        }
    }

    pub fn move_selection(&mut self, down: bool) {
        let len = self.visible_exercises().len();
        if len == 0 {
            return;
        }
        self.selected_index = if down {
            (self.selected_index + 1).min(len - 1)
        } else {
            self.selected_index.saturating_sub(1)
        };
    }

    pub fn enter_editor(&mut self) {
        if let Some(ex) = self.selected_exercise() {
            let id = ex.id.clone();
            let lang = ex.language.clone();
            let content = self
                .progress
                .exercises
                .get(&id)
                .and_then(|p| p.last_code.clone())
                .unwrap_or_else(|| ex.template.clone());
            self.editor_state = EditorState::new(&content, &lang);
            self.mode = AppMode::Editor;
            self.diagnostics.clear();
            self.current_output.clear();
            self.feedback.clear();
            self.revealed_hints = 0;
            self.has_run = false;
        }
    }

    pub fn reveal_hint(&mut self) {
        if let Some(count) = self.selected_exercise().map(|ex| ex.hints.len()) {
            self.revealed_hints = (self.revealed_hints + 1).min(count);
            if count == 0 {
                self.feedback = "Aucun indice pour cet exercice.".into();
            }
        }
    }

    pub fn show_help(&mut self) {
        self.previous_mode = self.mode;
        self.mode = AppMode::Help;
    }

    pub fn compile_current(&mut self) {
        if self.mode != AppMode::Editor {
            return;
        }
        let ex = match self.selected_exercise() {
            Some(e) => e.clone(),
            None => return,
        };
        self.has_run = true;
        self.diagnostics.clear();
        self.current_output.clear();
        self.feedback = "Compilation...".into();
        if let Some(adapter) = lang::get_adapter(&ex.language) {
            let code = self.editor_state.get_content();
            let workdir = match tempfile::tempdir() {
                Ok(d) => d,
                Err(e) => {
                    self.feedback = format!("Espace temporaire indisponible : {e}");
                    return;
                }
            };
            match adapter.compile(&code, workdir.path()) {
                Ok(output) => {
                    self.diagnostics = adapter.parse_errors(&output);
                    if output.success {
                        match adapter.run(workdir.path()) {
                            Ok(run) => {
                                self.current_output =
                                    if run.stdout.is_empty() && run.stderr.is_empty() {
                                        "(aucune sortie)".into()
                                    } else if run.stderr.is_empty() {
                                        run.stdout.clone()
                                    } else {
                                        format!("{}\n{}", run.stdout, run.stderr)
                                    };
                                let expected_matches = ex
                                    .expected_output
                                    .as_ref()
                                    .is_none_or(|expected| run.stdout.trim() == expected.trim());
                                if run.success && expected_matches {
                                    self.progress.mark_completed(&ex.id, &code);
                                    self.feedback = "Réussi ! Exercice terminé.".into();
                                } else if !run.success {
                                    self.feedback = format!(
                                        "Le programme a échoué (code {:?}).",
                                        run.exit_code
                                    );
                                } else {
                                    self.feedback = format!(
                                        "Sortie différente. Attendu : {}",
                                        ex.expected_output.as_deref().unwrap_or("")
                                    );
                                }
                            }
                            Err(e) => self.feedback = format!("Exécution impossible : {e}"),
                        }
                    } else {
                        self.feedback = "Compilation échouée : consulte les diagnostics.".into();
                        self.current_output = output.stderr;
                    }
                }
                Err(e) => self.feedback = format!("Compilation impossible : {e}"),
            }
            if !matches!(self.progress.get_status(&ex.id), Status::Completed) {
                self.progress.save_draft(&ex.id, &code);
            }
            if let Err(e) = self.storage.save_progress(&self.progress) {
                self.feedback
                    .push_str(&format!(" Sauvegarde impossible : {e}"));
            }
        } else {
            self.feedback = format!("Langage non pris en charge : {}", ex.language);
        }
    }

    pub fn save_draft(&mut self) {
        if self.mode != AppMode::Editor {
            return;
        }
        if let Some(ex) = self.selected_exercise() {
            let id = ex.id.clone();
            let code = self.editor_state.get_content();
            self.progress.save_draft(&id, &code);
            self.feedback = match self.storage.save_progress(&self.progress) {
                Ok(()) => "Brouillon sauvegardé.".into(),
                Err(e) => format!("Sauvegarde impossible : {e}"),
            };
        }
    }
}
