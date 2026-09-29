use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, StatefulWidget, Widget},
};
use std::sync::OnceLock;
use syntect::{easy::HighlightLines, highlighting::ThemeSet, parsing::SyntaxSet};

#[derive(Debug, Clone, Default)]
pub struct EditorState {
    pub lines: Vec<String>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub scroll_row: usize,
    pub scroll_col: usize,
    pub language: String,
}

impl EditorState {
    pub fn new(content: &str, language: &str) -> Self {
        let mut lines: Vec<String> = content.split('\n').map(str::to_string).collect();
        if lines.is_empty() {
            lines.push(String::new());
        }
        Self {
            lines,
            language: language.to_string(),
            ..Self::default()
        }
    }

    pub fn get_content(&self) -> String {
        self.lines.join("\n")
    }

    fn current_len(&self) -> usize {
        self.lines[self.cursor_row].chars().count()
    }
    fn byte_index(line: &str, column: usize) -> usize {
        line.char_indices()
            .nth(column)
            .map(|(i, _)| i)
            .unwrap_or(line.len())
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) {
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        self.cursor_row = self.cursor_row.min(self.lines.len() - 1);
        self.cursor_col = self.cursor_col.min(self.current_len());
        match key.code {
            KeyCode::Char(c)
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                let line = &mut self.lines[self.cursor_row];
                line.insert(Self::byte_index(line, self.cursor_col), c);
                self.cursor_col += 1;
            }
            KeyCode::Tab => {
                let line = &mut self.lines[self.cursor_row];
                line.insert_str(Self::byte_index(line, self.cursor_col), "    ");
                self.cursor_col += 4;
            }
            KeyCode::Backspace => {
                if self.cursor_col > 0 {
                    let line = &mut self.lines[self.cursor_row];
                    let start = Self::byte_index(line, self.cursor_col - 1);
                    let end = Self::byte_index(line, self.cursor_col);
                    line.replace_range(start..end, "");
                    self.cursor_col -= 1;
                } else if self.cursor_row > 0 {
                    let current = self.lines.remove(self.cursor_row);
                    self.cursor_row -= 1;
                    self.cursor_col = self.current_len();
                    self.lines[self.cursor_row].push_str(&current);
                }
            }
            KeyCode::Delete => {
                if self.cursor_col < self.current_len() {
                    let line = &mut self.lines[self.cursor_row];
                    let start = Self::byte_index(line, self.cursor_col);
                    let end = Self::byte_index(line, self.cursor_col + 1);
                    line.replace_range(start..end, "");
                } else if self.cursor_row + 1 < self.lines.len() {
                    let next = self.lines.remove(self.cursor_row + 1);
                    self.lines[self.cursor_row].push_str(&next);
                }
            }
            KeyCode::Enter => {
                let line = &mut self.lines[self.cursor_row];
                let rest = line.split_off(Self::byte_index(line, self.cursor_col));
                let indent: String = line.chars().take_while(|c| c.is_whitespace()).collect();
                self.lines
                    .insert(self.cursor_row + 1, format!("{indent}{rest}"));
                self.cursor_row += 1;
                self.cursor_col = indent.chars().count();
            }
            KeyCode::Left => {
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                } else if self.cursor_row > 0 {
                    self.cursor_row -= 1;
                    self.cursor_col = self.current_len();
                }
            }
            KeyCode::Right => {
                if self.cursor_col < self.current_len() {
                    self.cursor_col += 1;
                } else if self.cursor_row + 1 < self.lines.len() {
                    self.cursor_row += 1;
                    self.cursor_col = 0;
                }
            }
            KeyCode::Up => {
                self.cursor_row = self.cursor_row.saturating_sub(1);
                self.cursor_col = self.cursor_col.min(self.current_len());
            }
            KeyCode::Down => {
                self.cursor_row = (self.cursor_row + 1).min(self.lines.len() - 1);
                self.cursor_col = self.cursor_col.min(self.current_len());
            }
            KeyCode::Home => self.cursor_col = 0,
            KeyCode::End => self.cursor_col = self.current_len(),
            KeyCode::PageUp => {
                self.cursor_row = self.cursor_row.saturating_sub(10);
                self.cursor_col = self.cursor_col.min(self.current_len());
            }
            KeyCode::PageDown => {
                self.cursor_row = (self.cursor_row + 10).min(self.lines.len() - 1);
                self.cursor_col = self.cursor_col.min(self.current_len());
            }
            _ => {}
        }
    }

    pub fn ensure_visible(&mut self, height: usize, width: usize) {
        if height == 0 || width == 0 {
            return;
        }
        if self.cursor_row < self.scroll_row {
            self.scroll_row = self.cursor_row;
        }
        if self.cursor_row >= self.scroll_row + height {
            self.scroll_row = self.cursor_row + 1 - height;
        }
        if self.cursor_col < self.scroll_col {
            self.scroll_col = self.cursor_col;
        }
        if self.cursor_col >= self.scroll_col + width {
            self.scroll_col = self.cursor_col + 1 - width;
        }
    }
}

pub struct EditorWidget<'a> {
    block: Option<Block<'a>>,
}
impl<'a> EditorWidget<'a> {
    pub fn new() -> Self {
        Self { block: None }
    }
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl<'a> StatefulWidget for EditorWidget<'a> {
    type State = EditorState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let inner = match &self.block {
            Some(b) => {
                let inner = b.inner(area);
                b.clone().render(area, buf);
                inner
            }
            None => area,
        };
        if inner.width < 8 || inner.height == 0 {
            return;
        }
        let gutter = 6u16;
        let width = (inner.width - gutter) as usize;
        state.ensure_visible(inner.height as usize, width);
        static SYNTAX: OnceLock<SyntaxSet> = OnceLock::new();
        static THEMES: OnceLock<ThemeSet> = OnceLock::new();
        let syntaxes = SYNTAX.get_or_init(SyntaxSet::load_defaults_newlines);
        let themes = THEMES.get_or_init(ThemeSet::load_defaults);
        let extension = match state.language.as_str() {
            "rust" => "rs",
            "java" => "java",
            other => other,
        };
        let syntax = syntaxes
            .find_syntax_by_extension(extension)
            .unwrap_or_else(|| syntaxes.find_syntax_plain_text());
        let theme = &themes.themes["base16-ocean.dark"];
        let mut highlighter = HighlightLines::new(syntax, theme);
        for (row, line) in state.lines.iter().enumerate() {
            let ranges = highlighter
                .highlight_line(line, syntaxes)
                .unwrap_or_default();
            if row < state.scroll_row {
                continue;
            }
            if row >= state.scroll_row + inner.height as usize {
                break;
            }
            let y = inner.y + (row - state.scroll_row) as u16;
            let num = format!("{:>4} │", row + 1);
            buf.set_stringn(
                inner.x,
                y,
                num,
                gutter as usize,
                Style::default().fg(Color::DarkGray),
            );
            let mut column = 0usize;
            for (style, segment) in ranges {
                let color = Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
                for c in segment.chars() {
                    if column >= state.scroll_col && column < state.scroll_col + width {
                        let x = inner.x + gutter + (column - state.scroll_col) as u16;
                        buf.set_stringn(
                            x,
                            y,
                            c.to_string(),
                            (inner.x + inner.width - x) as usize,
                            Style::default().fg(color),
                        );
                    }
                    column += 1;
                }
            }
        }
        if state.cursor_row >= state.scroll_row
            && state.cursor_row < state.scroll_row + inner.height as usize
        {
            let x = inner.x + gutter + (state.cursor_col - state.scroll_col) as u16;
            let y = inner.y + (state.cursor_row - state.scroll_row) as u16;
            if x < inner.x + inner.width {
                buf.set_style(
                    Rect::new(x, y, 1, 1),
                    Style::default().bg(Color::Cyan).fg(Color::Black),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEvent, KeyModifiers};
    #[test]
    fn unicode_editing_and_scrolling() {
        let mut editor = EditorState::new("éclair\nfin", "rust");
        editor.handle_key_event(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
        editor.handle_key_event(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
        assert_eq!(editor.get_content(), "clair\nfin");
        editor.handle_key_event(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        editor.ensure_visible(1, 2);
        assert_eq!(editor.scroll_row, 1);
    }
}
