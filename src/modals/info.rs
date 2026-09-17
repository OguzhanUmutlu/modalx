use colored::Colorize;
use crossterm::{
    cursor::Hide,
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::io;

use crate::error::Result;
use crate::frame::BoxFrame;
use crate::keys::{KeyAction, KeyMap};
use crate::shortcuts::Shortcuts;
use crate::terminal::{
    clean_exit, get_content_width, get_terminal_size, is_terminal_too_small, wait_for_valid_size,
};

/// A fully boxed, scrollable message/alert modal.
pub struct InfoModal {
    pub title: String,
    pub lines: Vec<String>,
    pub is_error: bool,
    pub keymap: KeyMap,
    pub max_width: u16,
    pub shortcuts: Option<Shortcuts>,
}

impl InfoModal {
    /// Creates a new `InfoModal`.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            lines: Vec::new(),
            is_error: false,
            keymap: KeyMap::default(),
            max_width: 0,
            shortcuts: None,
        }
    }

    /// Sets content lines.
    pub fn with_lines(mut self, lines: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.lines = lines.into_iter().map(Into::into).collect();
        self
    }

    /// Adds a single line to the content.
    pub fn with_line(mut self, line: impl Into<String>) -> Self {
        self.lines.push(line.into());
        self
    }

    /// Sets whether this modal represents an error dialog.
    pub fn with_error(mut self, is_error: bool) -> Self {
        self.is_error = is_error;
        self
    }

    /// Sets whether this modal represents an error dialog (alias for `with_error`).
    #[allow(clippy::wrong_self_convention)]
    pub fn is_error(self, is_error: bool) -> Self {
        self.with_error(is_error)
    }

    /// Sets maximum desired box content width.
    pub fn with_max_width(mut self, width: u16) -> Self {
        self.max_width = width;
        self
    }

    /// Sets a custom `KeyMap`.
    pub fn with_keymap(mut self, keymap: KeyMap) -> Self {
        self.keymap = keymap;
        self
    }

    /// Sets custom bottom footer shortcuts.
    pub fn with_shortcuts(mut self, shortcuts: impl Into<Shortcuts>) -> Self {
        self.shortcuts = Some(shortcuts.into());
        self
    }

    /// Runs the interactive scrollable modal loop.
    pub fn run(&self) -> Result<()> {
        let mut stdout = io::stdout();
        enable_raw_mode()?;
        let _ = execute!(stdout, Hide);

        let mut scroll_offset: usize = 0;

        let res = (|| -> Result<()> {
            loop {
                if is_terminal_too_small() {
                    wait_for_valid_size(&mut stdout)?;
                    continue;
                }

                let (_, term_h) = get_terminal_size();
                let width = get_content_width(self.max_width);

                let overhead = 1 + 2 + 3; // top (1) + title/divider (2) + footer/divider/bottom (3)
                let viewport_size = (term_h as usize).saturating_sub(overhead).max(4);

                if scroll_offset + viewport_size > self.lines.len() {
                    scroll_offset = self.lines.len().saturating_sub(viewport_size);
                }

                let mut frame = BoxFrame::new(width);
                frame.title = Some((self.title.clone(), self.is_error));

                if scroll_offset > 0 {
                    frame.row(
                        format!("[▲ {} lines above]", scroll_offset)
                            .dimmed()
                            .to_string(),
                    );
                }

                let end_idx = self.lines.len().min(scroll_offset + viewport_size);
                for line in &self.lines[scroll_offset..end_idx] {
                    frame.row(line);
                }

                if end_idx < self.lines.len() {
                    frame.row(
                        format!("[▼ {} lines below]", self.lines.len() - end_idx)
                            .dimmed()
                            .to_string(),
                    );
                }

                let can_scroll = self.lines.len() > viewport_size;
                let shortcuts = self.shortcuts.clone().unwrap_or_else(|| {
                    Shortcuts::new()
                        .scroll_if(can_scroll)
                        .dismiss()
                        .exit_if(self.keymap.allow_quit_on_q)
                });
                frame.shortcuts(&shortcuts);
                frame.render(&mut stdout)?;

                match event::read()? {
                    Event::Resize(..) => continue,
                    Event::Key(key) => {
                        let action = self.keymap.resolve(&key);
                        match action {
                            KeyAction::Quit => {
                                clean_exit();
                            }
                            KeyAction::Up => {
                                scroll_offset = scroll_offset.saturating_sub(1);
                            }
                            KeyAction::Down => {
                                if scroll_offset + viewport_size < self.lines.len() {
                                    scroll_offset += 1;
                                }
                            }
                            KeyAction::PageUp => {
                                scroll_offset = scroll_offset.saturating_sub(viewport_size);
                            }
                            KeyAction::PageDown => {
                                scroll_offset = (scroll_offset + viewport_size)
                                    .min(self.lines.len().saturating_sub(viewport_size));
                            }
                            KeyAction::Home => {
                                scroll_offset = 0;
                            }
                            KeyAction::End => {
                                scroll_offset = self.lines.len().saturating_sub(viewport_size);
                            }
                            KeyAction::Submit
                            | KeyAction::Cancel
                            | KeyAction::Toggle
                            | KeyAction::Left
                            | KeyAction::Right => {
                                break;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            Ok(())
        })();

        while event::poll(std::time::Duration::from_millis(0)).unwrap_or(false) {
            let _ = event::read();
        }

        if !crate::terminal::is_alt_screen_active() {
            let _ = disable_raw_mode();
        }
        res
    }
}
