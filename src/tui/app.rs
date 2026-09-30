use crate::compiler::diagnostics::Diagnostic;
use crate::core::errors::Result;
use crate::core::exercise::Exercise;
use crate::core::progress::{Status, UserProgress};
use crate::core::session::Session;
use crate::core::settings::Locale;
use crate::lang;
use crate::storage::local::LocalStorage;
use crate::tui::widgets::editor::EditorState;
use std::sync::mpsc::{self, Receiver, TryRecvError};

struct RunOutcome {
    compiled: crate::lang::traits::CompileOutput,
    run: Option<crate::lang::traits::RunOutput>,
    verified: bool,
}

pub struct PendingRun {
    receiver: Receiver<Result<RunOutcome>>,
    exercise: Exercise,
    code: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Locale,
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
    pub locale_selection: usize,
    pub selected_index: usize,
    pub progress: UserProgress,
    pub storage: LocalStorage,
    pub locale: Locale,
    pub editor_state: EditorState,
    pub diagnostics: Vec<Diagnostic>,
    pub current_output: String,
    pub feedback: String,
    pub revealed_hints: usize,
    pub has_run: bool,
    pub should_quit: bool,
    pub pending: Option<PendingRun>,
}

impl App {
    pub fn new() -> crate::core::errors::Result<Self> {
        let storage = LocalStorage::new()?;
        let locale = storage.load_settings()?.locale;
        let progress = storage.load_progress()?;
        let exercises = Exercise::load_all_from_dir(&crate::config::exercises_dir_for(locale))?;
        let first_run = !storage.settings_path().exists();
        Ok(Self {
            mode: if first_run {
                AppMode::Locale
            } else {
                AppMode::Home
            },
            previous_mode: AppMode::Home,
            exercises,
            language: None,
            home_selection: 0,
            locale_selection: usize::from(locale == Locale::En),
            selected_index: 0,
            progress,
            storage,
            locale,
            editor_state: EditorState::default(),
            diagnostics: Vec::new(),
            current_output: String::new(),
            feedback: String::new(),
            revealed_hints: 0,
            has_run: false,
            should_quit: false,
            pending: None,
        })
    }

    pub fn choose_locale(&mut self) -> crate::core::errors::Result<()> {
        let locale = if self.locale_selection == 1 {
            Locale::En
        } else {
            Locale::Fr
        };
        self.storage.save_locale(locale)?;
        self.locale = locale;
        self.exercises = Exercise::load_all_from_dir(&crate::config::exercises_dir_for(locale))?;
        self.mode = AppMode::Home;
        Ok(())
    }

    pub fn open_locale_picker(&mut self) {
        self.locale_selection = usize::from(self.locale == Locale::En);
        self.previous_mode = self.mode;
        self.mode = AppMode::Locale;
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
        if self.mode != AppMode::Editor || self.pending.is_some() {
            return;
        }
        let ex = match self.selected_exercise() {
            Some(e) => e.clone(),
            None => return,
        };
        self.has_run = true;
        self.diagnostics.clear();
        self.current_output.clear();
        let Some(adapter) = lang::get_adapter(&ex.language) else {
            self.feedback = format!("Langage non pris en charge : {}", ex.language);
            return;
        };
        let code = self.editor_state.get_content();
        let (sender, receiver) = mpsc::channel();
        let exercise = ex.clone();
        let source = code.clone();
        std::thread::spawn(move || {
            let result = (|| {
                let session = Session::new(exercise, adapter)?;
                let compiled = session.compile(&source)?;
                let (run, verified) = if compiled.success {
                    let run = session.run()?;
                    let verified = session.verify(&run);
                    (Some(run), verified)
                } else {
                    (None, false)
                };
                Ok(RunOutcome {
                    compiled,
                    run,
                    verified,
                })
            })();
            let _ = sender.send(result);
        });
        self.pending = Some(PendingRun {
            receiver,
            exercise: ex,
            code,
        });
        self.feedback = "Compilation et vérification en cours… F1 aide, Ctrl+Q quitter.".into();
    }

    pub fn poll_run(&mut self) {
        let result = match self
            .pending
            .as_ref()
            .map(|pending| pending.receiver.try_recv())
        {
            Some(Ok(result)) => result,
            Some(Err(TryRecvError::Disconnected)) => {
                self.pending = None;
                self.feedback = "La vérification s'est interrompue. Réessaie.".into();
                return;
            }
            _ => return,
        };
        let pending = self.pending.take().expect("received pending run");
        let ex = pending.exercise;
        let code = pending.code;
        match result {
            Ok(outcome) => {
                if let Some(adapter) = lang::get_adapter(&ex.language) {
                    self.diagnostics = adapter.parse_errors(&outcome.compiled);
                }
                if let Some(run) = outcome.run {
                    self.current_output = if run.stdout.is_empty() && run.stderr.is_empty() {
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
                    if run.success && outcome.verified {
                        self.progress.mark_completed(&ex.id, &code);
                        self.feedback = "Réussi ! Exercice terminé.".into();
                    } else if !run.success {
                        self.feedback = match run.exit_code {
                            Some(code) => format!(
                                "Programme arrêté avec le code {code}. Lis la sortie ci-dessous."
                            ),
                            None => {
                                "Programme interrompu par le système. Lis la sortie ci-dessous."
                                    .into()
                            }
                        };
                    } else if !expected_matches {
                        self.feedback = format!(
                            "Sortie différente.\nAttendu : {}\nObtenu : {}",
                            ex.expected_output.as_deref().unwrap_or("(aucune sortie)"),
                            if run.stdout.trim().is_empty() {
                                "(aucune sortie)"
                            } else {
                                run.stdout.trim()
                            }
                        );
                    } else {
                        self.feedback = "Le résultat principal est correct, mais un cas supplémentaire a échoué. Vérifie les autres valeurs possibles.".into();
                    }
                } else {
                    self.feedback = "Compilation échouée : consulte les diagnostics.".into();
                    self.current_output = outcome.compiled.stderr;
                }
            }
            Err(error) => self.feedback = format!("Vérification interrompue : {error}"),
        }
        if !matches!(self.progress.get_status(&ex.id), Status::Completed) {
            self.progress.save_draft(&ex.id, &code);
        }
        if let Err(error) = self.storage.save_progress(&self.progress) {
            self.feedback
                .push_str(&format!(" Sauvegarde impossible : {error}"));
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
