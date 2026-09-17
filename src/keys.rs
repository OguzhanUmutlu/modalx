use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

/// Semantic user input action resolved from low-level terminal key events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyAction {
    /// Move cursor / selection upward
    Up,
    /// Move cursor / selection downward
    Down,
    /// Page up viewport
    PageUp,
    /// Page down viewport
    PageDown,
    /// Jump to the first item / line
    Home,
    /// Jump to the last item / line
    End,
    /// Move cursor left or collapse
    Left,
    /// Move cursor right or expand
    Right,
    /// Confirm, enter, or activate selection
    Submit,
    /// Cancel, escape, or navigate back
    Cancel,
    /// Toggle item state (e.g. checkbox with Space)
    Toggle,
    /// Remove character before cursor
    Backspace,
    /// Remove character under / after cursor
    Delete,
    /// Jump cursor backward by one word
    WordLeft,
    /// Jump cursor forward by one word
    WordRight,
    /// Delete word backward (e.g. Ctrl+W)
    DeleteWord,
    /// Clear current line / buffer (e.g. Ctrl+U)
    ClearInput,
    /// Direct alphanumeric hotkey pressed
    Hotkey(char),
    /// Search / filter action (e.g. '/')
    Search,
    /// Refresh data view (e.g. 'r' or F5)
    Refresh,
    /// Cleanly quit application (e.g. Ctrl+C or 'q')
    Quit,
    /// Custom named action
    Custom(String),
    /// Ignored or unrecognized key event
    None,
}

/// Contextual mode used to generate dynamic footer help text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyHelpMode {
    Menu,
    MenuWithToggle,
    Input,
    Info,
    Waiting,
    Confirm,
    Table,
}

/// Configurable key mapping engine translating raw crossterm key events into `KeyAction`.
#[derive(Debug, Clone)]
pub struct KeyMap {
    pub up: Vec<KeyCode>,
    pub down: Vec<KeyCode>,
    pub page_up: Vec<KeyCode>,
    pub page_down: Vec<KeyCode>,
    pub home: Vec<KeyCode>,
    pub end: Vec<KeyCode>,
    pub left: Vec<KeyCode>,
    pub right: Vec<KeyCode>,
    pub submit: Vec<KeyCode>,
    pub cancel: Vec<KeyCode>,
    pub toggle: Vec<KeyCode>,
    pub delete: Vec<KeyCode>,
    pub backspace: Vec<KeyCode>,
    pub allow_quit_on_q: bool,
    pub custom: Vec<(KeyCode, KeyModifiers, KeyAction)>,
}

impl Default for KeyMap {
    fn default() -> Self {
        Self {
            up: vec![KeyCode::Up, KeyCode::Char('k')],
            down: vec![KeyCode::Down, KeyCode::Char('j')],
            page_up: vec![KeyCode::PageUp],
            page_down: vec![KeyCode::PageDown],
            home: vec![KeyCode::Home],
            end: vec![KeyCode::End],
            left: vec![KeyCode::Left],
            right: vec![KeyCode::Right],
            submit: vec![KeyCode::Enter],
            cancel: vec![KeyCode::Esc],
            toggle: vec![KeyCode::Char(' ')],
            delete: vec![KeyCode::Delete],
            backspace: vec![KeyCode::Backspace],
            allow_quit_on_q: true,
            custom: Vec::new(),
        }
    }
}

impl KeyMap {
    /// Creates a keymap specifically tuned for menu selection.
    pub fn menu_default(allow_quit_on_q: bool) -> Self {
        Self {
            allow_quit_on_q,
            right: vec![KeyCode::Right],
            left: vec![KeyCode::Left],
            ..Default::default()
        }
    }

    /// Adds a custom keybinding override.
    pub fn with_custom(
        mut self,
        code: KeyCode,
        modifiers: KeyModifiers,
        action: KeyAction,
    ) -> Self {
        self.custom.push((code, modifiers, action));
        self
    }

    /// Resolves a raw terminal `KeyEvent` into a semantic `KeyAction`.
    pub fn resolve(&self, event: &KeyEvent) -> KeyAction {
        if event.kind != KeyEventKind::Press {
            return KeyAction::None;
        }

        // 1. Process termination (Ctrl+C, Ctrl+D)
        if (event.modifiers.contains(KeyModifiers::CONTROL)
            && (event.code == KeyCode::Char('c')
                || event.code == KeyCode::Char('C')
                || event.code == KeyCode::Char('d')))
            || event.code == KeyCode::Char('\x03')
            || event.code == KeyCode::Char('\x04')
        {
            return KeyAction::Quit;
        }

        // 2. Custom registered bindings
        for (c, m, act) in &self.custom {
            if event.code == *c && event.modifiers == *m {
                return act.clone();
            }
        }

        // 3. Readline modifier combos (Ctrl+W, Ctrl+Backspace, Alt+Backspace, Ctrl+U, Ctrl+Left, Ctrl+Right, Alt+B, Alt+F)
        if event.modifiers.contains(KeyModifiers::CONTROL) {
            match event.code {
                KeyCode::Char('w')
                | KeyCode::Char('W')
                | KeyCode::Backspace
                | KeyCode::Char('\x08')
                | KeyCode::Char('\x7f') => return KeyAction::DeleteWord,
                KeyCode::Char('u') | KeyCode::Char('U') => return KeyAction::ClearInput,
                KeyCode::Left => return KeyAction::WordLeft,
                KeyCode::Right => return KeyAction::WordRight,
                _ => {}
            }
        }

        if event.modifiers.contains(KeyModifiers::ALT) {
            match event.code {
                KeyCode::Backspace | KeyCode::Char('\x08') | KeyCode::Char('\x7f') => {
                    return KeyAction::DeleteWord
                }
                KeyCode::Char('b') | KeyCode::Char('B') => return KeyAction::WordLeft,
                KeyCode::Char('f') | KeyCode::Char('F') => return KeyAction::WordRight,
                _ => {}
            }
        }

        // 4. Global quit shortcut on 'q' if enabled and no modifiers
        if self.allow_quit_on_q
            && event.modifiers.is_empty()
            && (event.code == KeyCode::Char('q') || event.code == KeyCode::Char('Q'))
        {
            return KeyAction::Quit;
        }

        // 5. Standard action mappings
        if self.up.contains(&event.code) {
            return KeyAction::Up;
        }
        if self.down.contains(&event.code) {
            return KeyAction::Down;
        }
        if self.page_up.contains(&event.code) {
            return KeyAction::PageUp;
        }
        if self.page_down.contains(&event.code) {
            return KeyAction::PageDown;
        }
        if self.home.contains(&event.code) {
            return KeyAction::Home;
        }
        if self.end.contains(&event.code) {
            return KeyAction::End;
        }
        if self.submit.contains(&event.code) {
            return KeyAction::Submit;
        }
        if self.cancel.contains(&event.code) {
            return KeyAction::Cancel;
        }
        if self.toggle.contains(&event.code) {
            return KeyAction::Toggle;
        }
        if self.delete.contains(&event.code) {
            return KeyAction::Delete;
        }
        if self.backspace.contains(&event.code) {
            return KeyAction::Backspace;
        }
        if self.left.contains(&event.code) {
            return KeyAction::Left;
        }
        if self.right.contains(&event.code) {
            return KeyAction::Right;
        }

        // 6. Direct alphanumeric hotkey
        if let KeyCode::Char(c) = event.code {
            if event.modifiers.is_empty() || event.modifiers == KeyModifiers::SHIFT {
                return KeyAction::Hotkey(c);
            }
        }

        KeyAction::None
    }

    /// Formats a clean, standard footer shortcut help string.
    pub fn footer_help_text(&self, mode: KeyHelpMode) -> String {
        let q_part = if self.allow_quit_on_q {
            "  |  [q] Exit"
        } else {
            ""
        };

        match mode {
            KeyHelpMode::Menu => {
                format!(
                    "[↑/↓/j/k] Move  |  [Enter/→] Select  |  [Esc/←] Back{}",
                    q_part
                )
            }
            KeyHelpMode::MenuWithToggle => {
                format!(
                    "[↑/↓] Move  |  [Space] Toggle  |  [Enter] Confirm  |  [Esc] Cancel{}",
                    q_part
                )
            }
            KeyHelpMode::Input => {
                "[Enter] Confirm  |  [Esc] Cancel  |  [Ctrl+W] Delete Word".to_string()
            }
            KeyHelpMode::Info => {
                format!("[↑/↓/PgUp/PgDn] Scroll  |  [Enter/Esc] Dismiss{}", q_part)
            }
            KeyHelpMode::Waiting => "[Esc] Cancel  |  [Ctrl+C] Abort".to_string(),
            KeyHelpMode::Confirm => {
                "[←/→/Tab] Select  |  [Enter/y/n] Confirm  |  [Esc] Cancel".to_string()
            }
            KeyHelpMode::Table => {
                format!(
                    "[↑/↓] Select Row  |  [PgUp/PgDn] Page  |  [Enter] Open  |  [Esc] Back{}",
                    q_part
                )
            }
        }
    }

    /// Returns the semantic footer shortcut help items as a vector of individual button descriptions.
    pub fn footer_help_items(&self, mode: KeyHelpMode) -> Vec<String> {
        let mut items = match mode {
            KeyHelpMode::Menu => vec![
                "[↑/↓/j/k] Move".to_string(),
                "[Enter/→] Select".to_string(),
                "[Esc/←] Back".to_string(),
            ],
            KeyHelpMode::MenuWithToggle => vec![
                "[↑/↓] Move".to_string(),
                "[Space] Toggle".to_string(),
                "[Enter] Confirm".to_string(),
                "[Esc] Cancel".to_string(),
            ],
            KeyHelpMode::Input => vec![
                "[Enter] Confirm".to_string(),
                "[Esc] Cancel".to_string(),
                "[Ctrl+W] Delete Word".to_string(),
            ],
            KeyHelpMode::Info => vec![
                "[↑/↓/PgUp/PgDn] Scroll".to_string(),
                "[Enter/Esc] Dismiss".to_string(),
            ],
            KeyHelpMode::Waiting => vec!["[Esc] Cancel".to_string(), "[Ctrl+C] Abort".to_string()],
            KeyHelpMode::Confirm => vec![
                "[←/→/Tab] Select".to_string(),
                "[Enter/y/n] Confirm".to_string(),
                "[Esc] Cancel".to_string(),
            ],
            KeyHelpMode::Table => vec![
                "[↑/↓] Select Row".to_string(),
                "[PgUp/PgDn] Page".to_string(),
                "[Enter] Open".to_string(),
                "[Esc] Back".to_string(),
            ],
        };

        if self.allow_quit_on_q
            && (mode == KeyHelpMode::Menu
                || mode == KeyHelpMode::MenuWithToggle
                || mode == KeyHelpMode::Info
                || mode == KeyHelpMode::Table)
        {
            items.push("[q] Exit".to_string());
        }

        items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_mapping() {
        let km = KeyMap::default();

        let up_event = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        assert_eq!(km.resolve(&up_event), KeyAction::Up);

        let k_event = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        assert_eq!(km.resolve(&k_event), KeyAction::Up);

        let enter_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(km.resolve(&enter_event), KeyAction::Submit);

        let esc_event = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(km.resolve(&esc_event), KeyAction::Cancel);

        let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(km.resolve(&ctrl_c), KeyAction::Quit);

        let hotkey_1 = KeyEvent::new(KeyCode::Char('1'), KeyModifiers::NONE);
        assert_eq!(km.resolve(&hotkey_1), KeyAction::Hotkey('1'));

        let ctrl_w = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL);
        assert_eq!(km.resolve(&ctrl_w), KeyAction::DeleteWord);
    }

    #[test]
    fn test_footer_help_text() {
        let km = KeyMap::default();
        let menu_help = km.footer_help_text(KeyHelpMode::Menu);
        assert!(menu_help.contains("Move"));
        assert!(menu_help.contains("Select"));
        assert!(menu_help.contains("Back"));
        assert!(menu_help.contains("Exit"));
    }
}
