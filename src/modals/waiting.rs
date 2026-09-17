use colored::Colorize;
use std::io;

use crate::error::Result;
use crate::frame::BoxFrame;
use crate::terminal::get_content_width;

pub const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// A boxed modal displaying in-place progress or background operation status.
#[derive(Debug, Clone)]
pub struct WaitingModal {
    pub title: String,
    pub message: String,
    pub steps: Vec<(String, bool)>,
    pub max_width: u16,
    pub shortcuts: Option<crate::shortcuts::Shortcuts>,
    pub progress: Option<crate::modals::progress::ProgressBar>,
    pub cancellable: bool,
}

impl WaitingModal {
    /// Creates a new `WaitingModal`.
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            steps: Vec::new(),
            max_width: 0,
            shortcuts: None,
            progress: None,
            cancellable: false,
        }
    }

    /// Adds an embedded progress bar.
    pub fn with_progress(mut self, current: u64, total: u64) -> Self {
        let mut pb = crate::modals::progress::ProgressBar::new(total);
        pb.set_position(current);
        self.progress = Some(pb);
        self
    }

    /// Updates current progress if a progress bar is attached.
    pub fn update_progress(&mut self, current: u64) {
        if let Some(ref mut pb) = self.progress {
            pb.set_position(current);
        }
    }

    /// Adds a sub-step showing its completion state.
    pub fn with_step(mut self, step: impl Into<String>, completed: bool) -> Self {
        self.steps.push((step.into(), completed));
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

    /// Sets whether this modal can be cancelled by the user pressing Escape.
    pub fn with_cancellable(mut self, cancellable: bool) -> Self {
        self.cancellable = cancellable;
        self
    }

    /// Checks whether the user pressed the Escape key to request cancellation (non-blocking).
    pub fn check_cancelled(&self) -> Result<bool> {
        use crossterm::event::{self, Event, KeyCode, KeyEventKind};
        use std::time::Duration;

        while event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Esc {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// Renders a single frame of the modal with a specific spinner animation frame.
    pub fn render_spinner(&self, spinner_idx: usize, stdout: &mut io::Stdout) -> Result<()> {
        let width = get_content_width(self.max_width);
        let spinner_char = SPINNER_FRAMES[spinner_idx % SPINNER_FRAMES.len()];

        let mut frame = BoxFrame::new(width);
        frame.title = Some((self.title.clone(), false));

        let main_line = format!(
            "{} {}",
            spinner_char.cyan().bold(),
            self.message.white().bold()
        );
        frame.row(main_line);

        if let Some(ref pb) = self.progress {
            let inner_width = width.saturating_sub(4);
            let bar_width = inner_width.saturating_sub(10).clamp(20, 48);
            let bar_visual = pb.render_bar(bar_width, spinner_idx);
            frame.empty_row();
            if pb.is_indeterminate() {
                frame.row(format!("  {}", bar_visual));
                frame.row(format!("  {} • {}", pb.format_counts().cyan(), pb.format_speed().dimmed()));
            } else {
                frame.row(format!("  {} {:>5.1}%", bar_visual, pb.percent()));
                frame.row(format!("  {} • {} • {}", pb.format_counts().cyan(), pb.format_speed().dimmed(), pb.format_eta().yellow()));
            }
        }

        if !self.steps.is_empty() {
            frame.empty_row();
            for (step, completed) in &self.steps {
                if *completed {
                    frame.row(format!("  {} {}", "✓".green().bold(), step));
                } else {
                    frame.row(format!("  {} {}", "•".dimmed(), step.dimmed()));
                }
            }
        }

        if let Some(ref sc) = self.shortcuts {
            frame.shortcuts(sc);
        } else if self.cancellable {
            frame.footer("[Esc] Cancel  |  Please wait...".to_string());
        } else {
            frame.footer("Please wait...".to_string());
        }
        frame.render(stdout)
    }

    /// Renders static progress information without an active spinner.
    pub fn render_static(&self, stdout: &mut io::Stdout) -> Result<()> {
        let width = get_content_width(self.max_width);

        let mut frame = BoxFrame::new(width);
        frame.title = Some((self.title.clone(), false));

        frame.row(self.message.clone());

        if let Some(ref pb) = self.progress {
            let inner_width = width.saturating_sub(4);
            let bar_width = inner_width.saturating_sub(10).clamp(20, 48);
            let bar_visual = pb.render_bar(bar_width, 0);
            frame.empty_row();
            if pb.is_indeterminate() {
                frame.row(format!("  {}", bar_visual));
                frame.row(format!("  {} • {}", pb.format_counts().cyan(), pb.format_speed().dimmed()));
            } else {
                frame.row(format!("  {} {:>5.1}%", bar_visual, pb.percent()));
                frame.row(format!("  {} • {} • {}", pb.format_counts().cyan(), pb.format_speed().dimmed(), pb.format_eta().yellow()));
            }
        }

        if !self.steps.is_empty() {
            frame.empty_row();
            for (step, completed) in &self.steps {
                if *completed {
                    frame.row(format!("  {} {}", "✓".green().bold(), step));
                } else {
                    frame.row(format!("  {} {}", "•".dimmed(), step.dimmed()));
                }
            }
        }

        if let Some(ref sc) = self.shortcuts {
            frame.shortcuts(sc);
        } else if self.cancellable {
            frame.footer("[Esc] Cancel  |  Please wait...".to_string());
        } else {
            frame.footer("Please wait...".to_string());
        }
        frame.render(stdout)
    }
}

/// Standalone helper rendering an in-place boxed status panel from a list of text lines.
pub fn render_boxed_status<S: AsRef<str>>(title: &str, lines: &[S]) -> Result<()> {
    let mut stdout = io::stdout();
    let width = get_content_width(80);

    let mut frame = BoxFrame::new(width);
    frame.title = Some((title.to_string(), false));

    for line in lines {
        frame.row(line.as_ref());
    }

    frame.render(&mut stdout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_waiting_modal_builder() {
        let modal = WaitingModal::new("RESTORING BACKUP", "Unpacking archive...")
            .with_step("Verify hash", true)
            .with_step("Decompressing tar.zst", false)
            .with_max_width(90);

        assert_eq!(modal.title, "RESTORING BACKUP");
        assert_eq!(modal.message, "Unpacking archive...");
        assert_eq!(modal.steps.len(), 2);
        assert_eq!(modal.steps[0].0, "Verify hash");
        assert!(modal.steps[0].1);
        assert_eq!(modal.steps[1].0, "Decompressing tar.zst");
        assert!(!modal.steps[1].1);
        assert_eq!(modal.max_width, 90);
    }
}
