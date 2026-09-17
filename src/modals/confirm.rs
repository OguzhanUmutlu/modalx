use colored::Colorize;
use crossterm::{
    cursor::Hide,
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::io;

use crate::error::Result;
use crate::frame::BoxFrame;
use crate::keys::{KeyAction, KeyMap};
use crate::terminal::{clean_exit, get_content_width, is_terminal_too_small, wait_for_valid_size};
use crate::text_flow::wrap_words;

/// Outcome returned by running a `ConfirmModal`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmOutcome {
    Confirmed,
    Cancelled,
}

/// A fully boxed, responsive confirmation modal.
#[derive(Debug, Clone)]
pub struct ConfirmModal {
    pub title: String,
    pub message: String,
    pub details: Vec<String>,
    pub is_danger: bool,
    pub center_message: bool,
    pub yes_label: String,
    pub no_label: String,
    pub default_yes: bool,
    pub keymap: KeyMap,
    pub max_width: u16,
    pub shortcuts: Option<crate::shortcuts::Shortcuts>,
}

impl ConfirmModal {
    /// Creates a new `ConfirmModal`.
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            details: Vec::new(),
            is_danger: false,
            center_message: true,
            yes_label: "Yes".to_string(),
            no_label: "No".to_string(),
            default_yes: false,
            keymap: KeyMap::default(),
            max_width: 0,
            shortcuts: None,
        }
    }

    /// Sets whether this is a dangerous destructive action (styles borders and button red).
    pub fn danger(mut self, is_danger: bool) -> Self {
        self.is_danger = is_danger;
        self
    }

    /// Fluent convenience helper to mark this confirmation as danger.
    pub fn as_danger(self) -> Self {
        self.danger(true)
    }

    /// Sets whether the message prompt and details should be centered horizontally (defaults to true).
    pub fn center_message(mut self, center: bool) -> Self {
        self.center_message = center;
        self
    }

    /// Fluent alias for `center_message`.
    pub fn with_centered_message(self, center: bool) -> Self {
        self.center_message(center)
    }

    /// Adds a detail line explaining the action consequences.
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.details.push(detail.into());
        self
    }

    /// Sets the label for the affirmative action button.
    pub fn with_yes_label(mut self, label: impl Into<String>) -> Self {
        self.yes_label = label.into();
        self
    }

    /// Sets the label for the negative action button.
    pub fn with_no_label(mut self, label: impl Into<String>) -> Self {
        self.no_label = label.into();
        self
    }

    /// Sets whether Yes is selected by default (defaults to false for safety).
    pub fn default_yes(mut self, default_yes: bool) -> Self {
        self.default_yes = default_yes;
        self
    }

    /// Sets maximum desired box content width.
    pub fn with_max_width(mut self, width: u16) -> Self {
        self.max_width = width;
        self
    }

    /// Sets custom bottom footer shortcuts.
    pub fn with_shortcuts(mut self, shortcuts: impl Into<crate::shortcuts::Shortcuts>) -> Self {
        self.shortcuts = Some(shortcuts.into());
        self
    }

    /// Runs the interactive confirmation dialog loop.
    pub fn run(&self) -> Result<ConfirmOutcome> {
        let mut stdout = io::stdout();
        enable_raw_mode()?;
        let _ = execute!(stdout, Hide);

        let mut selected_yes = self.default_yes;

        let res = (|| -> Result<ConfirmOutcome> {
            loop {
                if is_terminal_too_small() {
                    wait_for_valid_size(&mut stdout)?;
                    continue;
                }

                let width = get_content_width(self.max_width);
                let inner_width = width.saturating_sub(6);

                let mut frame = BoxFrame::new(width);
                frame.title = Some((self.title.clone(), self.is_danger));
                frame.show_breadcrumbs = false;

                for raw_line in self.message.lines() {
                    for line in wrap_words(raw_line, inner_width) {
                        if self.center_message {
                            frame.centered_row(line.white().bold().to_string());
                        } else {
                            frame.row(line.white().bold().to_string());
                        }
                    }
                }

                if !self.details.is_empty() {
                    frame.empty_row();
                    for d in &self.details {
                        for line in wrap_words(d, inner_width) {
                            if self.center_message {
                                frame.centered_row(line.dimmed().to_string());
                            } else {
                                frame.row(line.dimmed().to_string());
                            }
                        }
                    }
                }

                frame.empty_row();

                // Format interactive buttons
                let yes_btn = if selected_yes {
                    if self.is_danger {
                        format!(" \x1B[7;31m  {}  \x1B[0m ", self.yes_label)
                    } else {
                        format!(" \x1B[7;36m  {}  \x1B[0m ", self.yes_label)
                    }
                } else {
                    format!(" [ {} ] ", self.yes_label)
                };

                let no_btn = if !selected_yes {
                    format!(" \x1B[7;37m  {}  \x1B[0m ", self.no_label)
                } else {
                    format!(" [ {} ] ", self.no_label)
                };

                let raw_combined = format!("{}      {}", yes_btn, no_btn);
                frame.centered_row(raw_combined);

                let shortcuts = self.shortcuts.clone().unwrap_or_else(|| {
                    crate::shortcuts::Shortcuts::new()
                        .add("←/→/Tab", "Select")
                        .add("Enter/y/n", "Confirm")
                        .cancel()
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
                            KeyAction::Cancel => {
                                return Ok(ConfirmOutcome::Cancelled);
                            }
                            KeyAction::Left | KeyAction::Right => {
                                selected_yes = !selected_yes;
                            }
                            KeyAction::Submit => {
                                return if selected_yes {
                                    Ok(ConfirmOutcome::Confirmed)
                                } else {
                                    Ok(ConfirmOutcome::Cancelled)
                                };
                            }
                            _ => match key.code {
                                KeyCode::Tab => {
                                    selected_yes = !selected_yes;
                                }
                                KeyCode::Char('y') | KeyCode::Char('Y') => {
                                    return Ok(ConfirmOutcome::Confirmed);
                                }
                                KeyCode::Char('n') | KeyCode::Char('N') => {
                                    return Ok(ConfirmOutcome::Cancelled);
                                }
                                _ => {}
                            },
                        }
                    }
                    _ => {}
                }
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confirm_modal_builder() {
        let modal = ConfirmModal::new("DELETE SERVER", "Are you sure?")
            .danger(true)
            .with_detail("This cannot be undone")
            .with_yes_label("Delete Forever")
            .with_no_label("Keep Safe")
            .default_yes(true);

        assert_eq!(modal.title, "DELETE SERVER");
        assert_eq!(modal.message, "Are you sure?");
        assert!(modal.is_danger);
        assert_eq!(modal.details, vec!["This cannot be undone".to_string()]);
        assert_eq!(modal.yes_label, "Delete Forever");
        assert_eq!(modal.no_label, "Keep Safe");
        assert!(modal.default_yes);
        assert!(modal.center_message);

        let uncentered = modal.center_message(false);
        assert!(!uncentered.center_message);
    }
}
