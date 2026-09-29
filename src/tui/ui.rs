use crate::core::progress::Status;
use crate::tui::app::{App, AppMode};
use crate::tui::widgets::diagnostic::DiagnosticWidget;
use crate::tui::widgets::editor::EditorWidget;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

const ACCENT: Color = Color::Cyan;
const MUTED: Color = Color::DarkGray;

fn panel(title: &str) -> Block<'_> {
    Block::default()
        .title(format!(" {title} "))
        .title_style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(MUTED))
}

pub fn render(f: &mut Frame, app: &mut App) {
    if f.area().width < 40 || f.area().height < 12 {
        f.render_widget(
            Paragraph::new("Agrandis le terminal (40 colonnes × 12 lignes minimum).")
                .block(panel("Forge"))
                .wrap(Wrap { trim: true }),
            f.area(),
        );
        return;
    }
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());
    let language = app.language.as_deref().unwrap_or("Choisir un langage");
    let completed = app
        .exercises
        .iter()
        .filter(|e| app.progress.get_status(&e.id) == Status::Completed)
        .count();
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            "  FORGE  ",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(
                "  {language}  •  {completed}/{} terminés",
                app.exercises.len()
            ),
            Style::default().fg(Color::White),
        ),
    ]))
    .block(panel("Apprendre en pratiquant"));
    f.render_widget(header, chunks[0]);
    match app.mode {
        AppMode::Home => render_home(f, app, chunks[1]),
        AppMode::List => render_list(f, app, chunks[1]),
        AppMode::Editor => render_editor(f, app, chunks[1]),
        AppMode::Help => render_help(f, chunks[1]),
    }
    let shortcuts = match app.mode {
        AppMode::Home => " ↑↓ choisir   Entrée ouvrir   F1 aide   q quitter ",
        AppMode::List => " ↑↓/j k parcourir   Entrée ouvrir   Esc accueil   F1 aide ",
        AppMode::Editor => " Ctrl+R exécuter   Ctrl+S sauver   F2 indice   F1 aide   Esc retour ",
        AppMode::Help => " Esc revenir   q quitter ",
    };
    f.render_widget(
        Paragraph::new(shortcuts).style(Style::default().bg(Color::Blue).fg(Color::White)),
        chunks[2],
    );
}

fn render_home(f: &mut Frame, app: &App, area: Rect) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(9), Constraint::Min(0)])
        .split(area);
    f.render_widget(Paragraph::new("Bienvenue dans Forge !\n\n1. Choisis Java ou Rust.  2. Lis la mini-leçon et l'objectif.  3. Modifie le code de départ.\n4. Lance avec Ctrl+R, lis le résultat, puis avance à ton rythme. Tes brouillons sont sauvegardés.")
        .block(panel("Bienvenue")).wrap(Wrap { trim: true }), sections[0]);
    let languages = app.languages();
    let items: Vec<ListItem> = languages
        .iter()
        .enumerate()
        .map(|(i, language)| {
            let total = app
                .exercises
                .iter()
                .filter(|e| &e.language == language)
                .count();
            let done = app
                .exercises
                .iter()
                .filter(|e| {
                    &e.language == language && app.progress.get_status(&e.id) == Status::Completed
                })
                .count();
            let marker = if i == app.home_selection { "▸" } else { " " };
            ListItem::new(format!(
                " {marker} {:<12} {done}/{total} exercices terminés",
                language.to_uppercase()
            ))
            .style(if i == app.home_selection {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            })
        })
        .collect();
    let message = if items.is_empty() {
        "Aucun exercice trouvé. Vérifie le dossier exercises."
    } else {
        "Choisis ton parcours (Entrée pour continuer)"
    };
    let mut list = items;
    list.push(ListItem::new(""));
    list.push(ListItem::new(
        "Commandes utiles : forge progress · forge doctor · forge help",
    ));
    f.render_widget(List::new(list).block(panel(message)), sections[1]);
}

fn render_list(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(44), Constraint::Percentage(56)])
        .split(area);
    let visible = app.visible_exercises();
    let done = visible
        .iter()
        .filter(|e| app.progress.get_status(&e.id) == Status::Completed)
        .count();
    let items: Vec<ListItem> = visible
        .iter()
        .map(|ex| {
            let (marker, color) = match app.progress.get_status(&ex.id) {
                Status::Completed => ("✓", Color::Green),
                Status::InProgress => ("•", Color::Yellow),
                Status::NotStarted => ("○", Color::Gray),
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!(" {marker} "), Style::default().fg(color)),
                Span::raw(&ex.title),
            ]))
        })
        .collect();
    let mut state = ListState::default();
    if !visible.is_empty() {
        state.select(Some(app.selected_index.min(visible.len() - 1)));
    }
    f.render_stateful_widget(
        List::new(items)
            .block(panel(&format!("Exercices  {done}/{}", visible.len())))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▸"),
        chunks[0],
        &mut state,
    );
    if let Some(ex) = app.selected_exercise() {
        let status = match app.progress.get_status(&ex.id) {
            Status::Completed => "Terminé",
            Status::InProgress => "En cours",
            Status::NotStarted => "À commencer",
        };
        let details = format!("{}\n\n{}  •  {}  •  {}\n\nMini-cours : {}\n\nObjectif : {}\n\nAstuce : {}\n\n{} indice(s) disponibles\n\nEntrée pour ouvrir l'éditeur", ex.title, ex.language.to_uppercase(), ex.difficulty, status, ex.lesson, ex.description, ex.tip, ex.hints.len());
        f.render_widget(
            Paragraph::new(details)
                .block(panel("Détails de l'exercice"))
                .wrap(Wrap { trim: true }),
            chunks[1],
        );
    }
}

fn render_editor(f: &mut Frame, app: &mut App, area: Rect) {
    let ex = app.selected_exercise().cloned();
    let upper = if area.height < 18 { 4 } else { 9 };
    let lower = if area.height < 15 { 4 } else { 8 };
    let parts = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(upper),
            Constraint::Min(3),
            Constraint::Length(lower),
        ])
        .split(area);
    if let Some(ex) = &ex {
        let hint = if app.revealed_hints == 0 {
            "F2 pour révéler un indice".to_string()
        } else {
            ex.hints.get(app.revealed_hints - 1).map_or_else(
                || "Aucun indice disponible".to_string(),
                |hint| format!("Indice {}/{} : {hint}", app.revealed_hints, ex.hints.len()),
            )
        };
        let text = if area.height < 18 {
            format!("{} · {}\n{}", ex.title, ex.description, hint)
        } else {
            format!(
                "{} · {}\n{}\nLeçon : {}\nAstuce : {}\n{}",
                ex.title, ex.difficulty, ex.description, ex.lesson, ex.tip, hint
            )
        };
        f.render_widget(
            Paragraph::new(text)
                .block(panel("Objectif & indices"))
                .wrap(Wrap { trim: true }),
            parts[0],
        );
    }
    let title = format!(
        "Éditeur  {}:{}",
        app.editor_state.cursor_row + 1,
        app.editor_state.cursor_col + 1
    );
    f.render_stateful_widget(
        EditorWidget::new().block(panel(&title)),
        parts[1],
        &mut app.editor_state,
    );
    let bottom = Layout::default()
        .direction(if area.width < 80 {
            Direction::Vertical
        } else {
            Direction::Horizontal
        })
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(parts[2]);
    f.render_widget(
        DiagnosticWidget::new(&app.diagnostics)
            .has_run(app.has_run)
            .block(panel("Diagnostics")),
        bottom[0],
    );
    let output = if app.feedback.is_empty() {
        "Ctrl+R pour compiler et exécuter".to_string()
    } else {
        format!("{}\n\n{}", app.feedback, app.current_output)
    };
    f.render_widget(
        Paragraph::new(output)
            .block(panel("Résultat"))
            .wrap(Wrap { trim: false }),
        bottom[1],
    );
}

fn render_help(f: &mut Frame, area: Rect) {
    let text = "Accueil : ↑/↓ pour choisir Java ou Rust, Entrée pour ouvrir.\n\nExercices : ↑/↓ ou j/k pour parcourir, Entrée pour éditer, Esc pour l'accueil. Les symboles ✓, • et ○ indiquent terminé, en cours et à commencer.\n\nÉditeur : Ctrl+R pour compiler et exécuter, Ctrl+S pour sauvegarder, F2 pour révéler un indice, Esc pour revenir. Flèches, Home, End, Suppr, Retour arrière, Tab et Entrée permettent d'éditer.\n\nF1 affiche cette aide ; Esc la ferme. Ctrl+Q quitte Forge.";
    f.render_widget(
        Paragraph::new(text)
            .block(panel("Aide & raccourcis"))
            .wrap(Wrap { trim: true }),
        area,
    );
}
