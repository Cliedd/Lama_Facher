use crate::compiler::diagnostics::{Diagnostic, Severity};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Widget},
};

pub struct DiagnosticWidget<'a> {
    block: Option<Block<'a>>,
    diagnostics: &'a [Diagnostic],
    has_run: bool,
}

impl<'a> DiagnosticWidget<'a> {
    pub fn new(diagnostics: &'a [Diagnostic]) -> Self {
        Self {
            block: None,
            diagnostics,
            has_run: false,
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn has_run(mut self, has_run: bool) -> Self {
        self.has_run = has_run;
        self
    }
}

impl<'a> Widget for DiagnosticWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner_area = match &self.block {
            Some(b) => {
                let inner = b.inner(area);
                b.clone().render(area, buf);
                inner
            }
            None => area,
        };

        if inner_area.width == 0 || inner_area.height == 0 {
            return;
        }
        if self.diagnostics.is_empty() {
            buf.set_string(
                inner_area.x,
                inner_area.y,
                if self.has_run {
                    "Aucun problème détecté"
                } else {
                    "Lance le code pour voir les diagnostics"
                },
                Style::default().fg(if self.has_run {
                    Color::Green
                } else {
                    Color::Gray
                }),
            );
            return;
        }

        let mut current_y = inner_area.y;
        for diag in self.diagnostics {
            if current_y >= inner_area.y + inner_area.height {
                break;
            }
            let color = match diag.severity {
                Severity::Error => Color::Red,
                Severity::Warning => Color::Yellow,
                Severity::Info => Color::Blue,
                Severity::Hint => Color::Cyan,
            };

            let prefix = match diag.severity {
                Severity::Error => "ERROR",
                Severity::Warning => "WARN ",
                Severity::Info => "INFO ",
                Severity::Hint => "HINT ",
            };

            let header = format!(
                "[{}] Line {}:{}: {}",
                prefix, diag.line, diag.column, diag.message
            );
            buf.set_stringn(
                inner_area.x,
                current_y,
                &header,
                inner_area.width as usize,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            );
            current_y += 1;

            if let Some(suggestion) = &diag.suggestion {
                let suggestion_str = format!("  💡 {}", suggestion);
                if current_y >= inner_area.y + inner_area.height {
                    break;
                }
                buf.set_stringn(
                    inner_area.x,
                    current_y,
                    &suggestion_str,
                    inner_area.width as usize,
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::ITALIC),
                );
                current_y += 1;
            }

            if current_y >= inner_area.y + inner_area.height {
                break;
            }
        }
    }
}
