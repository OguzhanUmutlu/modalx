use colored::Colorize;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::theme::{is_utf8_supported, strip_ansi, BORDER_COLOR, RESET};

/// Action returned when a key event is processed by `TextInput`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextInputAction {
    /// User submitted the input (e.g. Enter pressed).
    Submit(String),
    /// Content or cursor changed.
    Changed,
    /// History navigation occurred.
    HistoryNavigated,
    /// Unhandled or ignored key event.
    None,
}

/// A responsive, stateful single-line text input engine with full readline editing,
/// command history navigation, unicode safety, and inside-the-box row rendering.
#[derive(Debug, Clone)]
pub struct TextInput {
    buffer: String,
    cursor: usize,
    history: Vec<String>,
    history_idx: Option<usize>,
    history_stash: String,
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}

impl TextInput {
    /// Creates a new empty `TextInput`.
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            cursor: 0,
            history: Vec::new(),
            history_idx: None,
            history_stash: String::new(),
        }
    }

    /// Creates a `TextInput` initialized with an initial value.
    pub fn with_value(val: impl Into<String>) -> Self {
        let buffer = val.into();
        let cursor = buffer.chars().count();
        Self {
            buffer,
            cursor,
            history: Vec::new(),
            history_idx: None,
            history_stash: String::new(),
        }
    }

    /// Attaches pre-populated command history.
    pub fn with_history(mut self, history: Vec<String>) -> Self {
        self.history = history;
        self
    }

    /// Returns the current raw string buffer.
    pub fn buffer(&self) -> &str {
        &self.buffer
    }

    /// Returns the current character cursor position.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Returns the history entries.
    pub fn history(&self) -> &[String] {
        &self.history
    }

    /// Replaces the current buffer and resets the cursor to the end.
    pub fn set_buffer(&mut self, val: impl Into<String>) {
        self.buffer = val.into();
        self.cursor = self.buffer.chars().count();
    }

    /// Inserts a character at the current cursor position.
    pub fn insert_char(&mut self, c: char) {
        let mut chars: Vec<char> = self.buffer.chars().collect();
        let cur = self.cursor.min(chars.len());
        chars.insert(cur, c);
        self.buffer = chars.into_iter().collect();
        self.cursor = cur + 1;
    }

    /// Inserts a string at the current cursor position.
    pub fn insert_str(&mut self, s: &str) {
        for c in s.chars() {
            self.insert_char(c);
        }
    }

    /// Deletes the character directly preceding the cursor.
    pub fn backspace(&mut self) {
        if self.cursor > 0 && !self.buffer.is_empty() {
            let mut chars: Vec<char> = self.buffer.chars().collect();
            let cur = self.cursor.min(chars.len());
            chars.remove(cur - 1);
            self.buffer = chars.into_iter().collect();
            self.cursor = cur - 1;
        }
    }

    /// Deletes the character directly under the cursor.
    pub fn delete(&mut self) {
        let mut chars: Vec<char> = self.buffer.chars().collect();
        if self.cursor < chars.len() {
            chars.remove(self.cursor);
            self.buffer = chars.into_iter().collect();
        }
    }

    /// Deletes the word preceding the cursor (Ctrl+Backspace / Ctrl+W / Alt+Backspace).
    pub fn delete_word(&mut self) {
        if self.cursor == 0 || self.buffer.is_empty() {
            return;
        }
        let chars: Vec<char> = self.buffer.chars().collect();
        let cur = self.cursor.min(chars.len());
        let before = &chars[..cur];

        let mut idx = cur;
        // Skip whitespace immediately before cursor
        while idx > 0 && before[idx - 1].is_whitespace() {
            idx -= 1;
        }
        // Skip word / non-whitespace characters
        while idx > 0 && !before[idx - 1].is_whitespace() {
            idx -= 1;
        }

        let mut candidate: String = chars[..idx].iter().collect();
        let after: String = chars[cur..].iter().collect();
        candidate.push_str(&after);

        self.buffer = candidate;
        self.cursor = idx;
    }

    /// Clears the entire buffer and resets the cursor.
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.cursor = 0;
    }

    /// Moves cursor left by one character.
    pub fn move_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    /// Moves cursor right by one character.
    pub fn move_right(&mut self) {
        let len = self.buffer.chars().count();
        if self.cursor < len {
            self.cursor += 1;
        }
    }

    /// Moves cursor to the start of the previous word.
    pub fn word_left(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let chars: Vec<char> = self.buffer.chars().collect();
        let cur = self.cursor.min(chars.len());
        let mut idx = cur;
        while idx > 0 && chars[idx - 1].is_whitespace() {
            idx -= 1;
        }
        while idx > 0 && !chars[idx - 1].is_whitespace() {
            idx -= 1;
        }
        self.cursor = idx;
    }

    /// Moves cursor to the start of the next word.
    pub fn word_right(&mut self) {
        let chars: Vec<char> = self.buffer.chars().collect();
        let len = chars.len();
        let mut idx = self.cursor.min(len);
        while idx < len && !chars[idx].is_whitespace() {
            idx += 1;
        }
        while idx < len && chars[idx].is_whitespace() {
            idx += 1;
        }
        self.cursor = idx.min(len);
    }

    /// Moves cursor to the beginning of the line.
    pub fn move_home(&mut self) {
        self.cursor = 0;
    }

    /// Moves cursor to the end of the line.
    pub fn move_end(&mut self) {
        self.cursor = self.buffer.chars().count();
    }

    /// Recalls previous command from history (Up arrow).
    pub fn history_prev(&mut self) -> bool {
        if self.history.is_empty() {
            return false;
        }
        match self.history_idx {
            None => {
                self.history_stash = self.buffer.clone();
                let last = self.history.len() - 1;
                self.history_idx = Some(last);
                self.buffer = self.history[last].clone();
                self.cursor = self.buffer.chars().count();
                true
            }
            Some(idx) if idx > 0 => {
                let prev = idx - 1;
                self.history_idx = Some(prev);
                self.buffer = self.history[prev].clone();
                self.cursor = self.buffer.chars().count();
                true
            }
            _ => false,
        }
    }

    /// Recalls next command from history (Down arrow).
    pub fn history_next(&mut self) -> bool {
        match self.history_idx {
            Some(idx) => {
                if idx + 1 < self.history.len() {
                    let next = idx + 1;
                    self.history_idx = Some(next);
                    self.buffer = self.history[next].clone();
                    self.cursor = self.buffer.chars().count();
                } else {
                    self.history_idx = None;
                    self.buffer = self.history_stash.clone();
                    self.cursor = self.buffer.chars().count();
                }
                true
            }
            None => false,
        }
    }

    /// Commits the current buffer to history and clears the buffer, returning the submitted text.
    pub fn commit(&mut self) -> String {
        let submitted = self.buffer.clone();
        let trimmed = submitted.trim();
        if !trimmed.is_empty() {
            // Avoid duplicate contiguous history entries
            if self.history.last().map(|s| s.as_str()) != Some(trimmed) {
                self.history.push(trimmed.to_string());
            }
        }
        self.buffer.clear();
        self.cursor = 0;
        self.history_idx = None;
        self.history_stash.clear();
        submitted
    }

    /// Processes a crossterm key event, returning the resulting action.
    pub fn handle_key(&mut self, event: &KeyEvent) -> TextInputAction {
        if event.kind != KeyEventKind::Press {
            return TextInputAction::None;
        }

        // 1. Modifiers + Keys
        if event.modifiers.contains(KeyModifiers::CONTROL) {
            match event.code {
                KeyCode::Char('w')
                | KeyCode::Char('W')
                | KeyCode::Backspace
                | KeyCode::Char('\x08')
                | KeyCode::Char('\x7f') => {
                    self.delete_word();
                    return TextInputAction::Changed;
                }
                KeyCode::Char('u') | KeyCode::Char('U') => {
                    self.clear();
                    return TextInputAction::Changed;
                }
                KeyCode::Char('a') | KeyCode::Char('A') => {
                    self.move_home();
                    return TextInputAction::Changed;
                }
                KeyCode::Char('e') | KeyCode::Char('E') => {
                    self.move_end();
                    return TextInputAction::Changed;
                }
                KeyCode::Left => {
                    self.word_left();
                    return TextInputAction::Changed;
                }
                KeyCode::Right => {
                    self.word_right();
                    return TextInputAction::Changed;
                }
                _ => return TextInputAction::None,
            }
        }

        if event.modifiers.contains(KeyModifiers::ALT) {
            match event.code {
                KeyCode::Backspace | KeyCode::Char('\x08') | KeyCode::Char('\x7f') => {
                    self.delete_word();
                    return TextInputAction::Changed;
                }
                KeyCode::Char('b') | KeyCode::Char('B') => {
                    self.word_left();
                    return TextInputAction::Changed;
                }
                KeyCode::Char('f') | KeyCode::Char('F') => {
                    self.word_right();
                    return TextInputAction::Changed;
                }
                _ => return TextInputAction::None,
            }
        }

        // 2. Navigation & Editing Keys
        match event.code {
            KeyCode::Enter => {
                let line = self.commit();
                TextInputAction::Submit(line)
            }
            KeyCode::Backspace => {
                self.backspace();
                TextInputAction::Changed
            }
            KeyCode::Delete => {
                self.delete();
                TextInputAction::Changed
            }
            KeyCode::Left => {
                self.move_left();
                TextInputAction::Changed
            }
            KeyCode::Right => {
                self.move_right();
                TextInputAction::Changed
            }
            KeyCode::Home => {
                self.move_home();
                TextInputAction::Changed
            }
            KeyCode::End => {
                self.move_end();
                TextInputAction::Changed
            }
            KeyCode::Up => {
                if self.history_prev() {
                    TextInputAction::HistoryNavigated
                } else {
                    TextInputAction::None
                }
            }
            KeyCode::Down => {
                if self.history_next() {
                    TextInputAction::HistoryNavigated
                } else {
                    TextInputAction::None
                }
            }
            KeyCode::Char(c) => {
                self.insert_char(c);
                TextInputAction::Changed
            }
            _ => TextInputAction::None,
        }
    }

    /// Renders the input line enclosed inside box borders `│ <prefix><text> │`
    /// and returns `(rendered_row, visual_cursor_x)`.
    ///
    /// The row adapts to available `inner_width` (the space between left and right `│`).
    /// If the text exceeds the available space, it horizontally scrolls so the cursor
    /// is always visible.
    pub fn render_box_row(&self, inner_width: usize, prefix: &str) -> (String, u16) {
        let border_char = if is_utf8_supported() { "│" } else { "|" };
        let prefix_clean = strip_ansi(prefix);
        let prefix_len = prefix_clean.chars().count();

        // 1 column left border + 1 padding space
        let base_x = 2; // X index where content starts after "│ "
        let available_text_len = inner_width.saturating_sub(prefix_len + 2).max(1);

        let chars: Vec<char> = self.buffer.chars().collect();
        let total_chars = chars.len();
        let cur = self.cursor.min(total_chars);

        let (display_text, visual_offset) = if total_chars > available_text_len {
            let start = cur.saturating_sub(available_text_len.saturating_sub(4));
            let end = (start + available_text_len).min(total_chars);
            let slice: String = chars[start..end].iter().collect();
            let offset = cur.saturating_sub(start);
            (slice, offset)
        } else {
            (self.buffer.clone(), cur)
        };

        let displayed_len = display_text.chars().count();
        let pad_len = inner_width.saturating_sub(1 + prefix_len + displayed_len + 1);

        let row = format!(
            "{}{}{} {}{}{}{}{}{}{}\x1B[K\r\n",
            BORDER_COLOR,
            border_char,
            RESET,
            prefix.cyan().bold(),
            display_text,
            " ".repeat(pad_len),
            " ",
            BORDER_COLOR,
            border_char,
            RESET,
        );

        let visual_cursor_x = (base_x + prefix_len + visual_offset) as u16;
        (row, visual_cursor_x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_input_insert_and_backspace() {
        let mut input = TextInput::new();
        input.insert_str("hello world");
        assert_eq!(input.buffer(), "hello world");
        assert_eq!(input.cursor(), 11);

        input.backspace();
        assert_eq!(input.buffer(), "hello worl");
        assert_eq!(input.cursor(), 10);
    }

    #[test]
    fn test_text_input_delete_word() {
        let mut input = TextInput::new();
        input.insert_str("server stop now");
        input.delete_word();
        assert_eq!(input.buffer(), "server stop ");
        assert_eq!(input.cursor(), 12);

        input.delete_word();
        assert_eq!(input.buffer(), "server ");
        assert_eq!(input.cursor(), 7);

        input.delete_word();
        assert_eq!(input.buffer(), "");
        assert_eq!(input.cursor(), 0);
    }

    #[test]
    fn test_text_input_word_navigation() {
        let mut input = TextInput::new();
        input.insert_str("one two three");
        assert_eq!(input.cursor(), 13);

        input.word_left();
        assert_eq!(input.cursor(), 8);

        input.word_left();
        assert_eq!(input.cursor(), 4);

        input.word_right();
        assert_eq!(input.cursor(), 8);
    }

    #[test]
    fn test_text_input_history() {
        let mut input = TextInput::new();
        input.insert_str("cmd1");
        assert_eq!(input.commit(), "cmd1");

        input.insert_str("cmd2");
        assert_eq!(input.commit(), "cmd2");

        assert!(input.history_prev());
        assert_eq!(input.buffer(), "cmd2");

        assert!(input.history_prev());
        assert_eq!(input.buffer(), "cmd1");

        assert!(input.history_next());
        assert_eq!(input.buffer(), "cmd2");

        assert!(input.history_next());
        assert_eq!(input.buffer(), "");
    }

    #[test]
    fn test_render_box_row() {
        let mut input = TextInput::new();
        input.insert_str("status");
        let (row, cursor_x) = input.render_box_row(40, "> ");
        assert!(row.contains("│"));
        assert!(row.contains("status"));
        // base_x (2) + prefix_len (2) + 6 = 10
        assert_eq!(cursor_x, 10);
    }
}
