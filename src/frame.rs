use colored::Colorize;
use crossterm::{
    cursor::{Hide, MoveTo},
    queue,
    terminal::{Clear, ClearType},
};
use std::io::{self, Write};

use crate::error::Result;
use crate::terminal::get_terminal_size;
use crate::text_flow::{truncate_ansi, wrap_button_items};
use crate::theme::{
    box_bottom, box_divider, box_title, box_top, is_utf8_supported, strip_ansi, visible_len,
    BORDER_COLOR, RESET,
};

/// Core layout engine that encapsulates all visual TUI components inside an airtight dynamic box.
#[derive(Debug, Clone)]
pub struct BoxFrame {
    pub width: usize,
    pub title: Option<(String, bool)>,
    pub show_title: bool,
    pub breadcrumbs: Option<String>,
    pub show_breadcrumbs: bool,
    pub lines: Vec<FrameLine>,
    pub footer_help: Option<String>,
    pub footer_button_items: Vec<String>,
    pub footer_separator: String,
    pub footer_center: bool,
    pub vertical_center: bool,
}

#[derive(Debug, Clone)]
pub enum FrameLine {
    Row(String),
    RawRow(String),
    Centered(String),
    Empty,
    Divider,
    DividerDimmed,
}

impl BoxFrame {
    /// Creates a new box frame with a specified desired width (clamped to terminal bounds).
    pub fn new(width: usize) -> Self {
        Self {
            width,
            title: None,
            show_title: true,
            breadcrumbs: None,
            show_breadcrumbs: true,
            lines: Vec::new(),
            footer_help: None,
            footer_button_items: Vec::new(),
            footer_separator: "|".to_string(),
            footer_center: true,
            vertical_center: true,
        }
    }

    /// Sets the modal frame title.
    pub fn with_title(mut self, title: impl Into<String>, is_error: bool) -> Self {
        self.title = Some((title.into(), is_error));
        self
    }

    /// Disables title rendering on this frame.
    pub fn without_title(mut self) -> Self {
        self.show_title = false;
        self
    }

    /// Sets explicit navigation breadcrumbs.
    pub fn with_breadcrumbs(mut self, breadcrumbs: impl Into<String>) -> Self {
        self.breadcrumbs = Some(breadcrumbs.into());
        self.show_breadcrumbs = true;
        self
    }

    /// Disables navigation breadcrumbs rendering on this frame.
    pub fn without_breadcrumbs(mut self) -> Self {
        self.show_breadcrumbs = false;
        self
    }

    /// Adds a standardized content row enclosed in vertical side borders `│ ... │`.
    pub fn row(&mut self, content: impl AsRef<str>) -> &mut Self {
        self.lines
            .push(FrameLine::Row(content.as_ref().to_string()));
        self
    }

    /// Adds a horizontally centered content row enclosed in vertical side borders `│ ... │`.
    pub fn centered_row(&mut self, content: impl AsRef<str>) -> &mut Self {
        self.lines
            .push(FrameLine::Centered(content.as_ref().to_string()));
        self
    }

    /// Adds an empty padded row `│       │`.
    pub fn empty_row(&mut self) -> &mut Self {
        self.lines.push(FrameLine::Empty);
        self
    }

    /// Adds an internal divider `├───────┤`.
    pub fn divider(&mut self) -> &mut Self {
        self.lines.push(FrameLine::Divider);
        self
    }

    /// Adds a dimmed internal divider.
    pub fn divider_dimmed(&mut self) -> &mut Self {
        self.lines.push(FrameLine::DividerDimmed);
        self
    }

    /// Sets the bottom footer shortcut help bar.
    pub fn footer(&mut self, help_text: impl Into<String>) -> &mut Self {
        self.footer_help = Some(help_text.into());
        self
    }

    /// Sets the bottom footer shortcut help bar from individual button texts, separator, and centering.
    pub fn footer_buttons(
        &mut self,
        buttons: impl IntoIterator<Item = impl Into<String>>,
        separator: impl Into<String>,
        center: bool,
    ) -> &mut Self {
        self.footer_button_items = buttons.into_iter().map(Into::into).collect();
        self.footer_separator = separator.into();
        self.footer_center = center;
        self
    }

    /// Sets the bottom footer shortcut help bar directly from a high-level `Shortcuts` collection.
    pub fn shortcuts(&mut self, shortcuts: &crate::shortcuts::Shortcuts) -> &mut Self {
        self.footer_button_items = shortcuts.to_button_items();
        self.footer_separator = "  |  ".to_string();
        self.footer_help = Some(shortcuts.to_footer_string());
        self
    }

    /// Sets the bottom footer shortcut help bar directly from a high-level `Shortcuts` collection (builder style).
    pub fn with_shortcuts(mut self, shortcuts: impl Into<crate::shortcuts::Shortcuts>) -> Self {
        let sc = shortcuts.into();
        self.shortcuts(&sc);
        self
    }

    /// Configures whether this box frame should be rendered vertically centered in the terminal.
    pub fn with_vertical_center(mut self, center: bool) -> Self {
        self.vertical_center = center;
        self
    }

    /// Formats a single content string into a framed row strictly bounded by side borders `│ ... │`.
    pub fn format_row(content: &str, width: usize) -> String {
        let border_char = if is_utf8_supported() { "│" } else { "|" };
        let inner_width = width.saturating_sub(2); // subtract 2 for border chars
        if inner_width == 0 {
            return String::new();
        }

        let clean = strip_ansi(content);
        let char_count = visible_len(&clean);

        if char_count >= inner_width {
            let truncated = truncate_ansi(content, inner_width);
            let vis_len = visible_len(&truncated);
            let pad = inner_width.saturating_sub(vis_len);
            format!(
                "{}{}{}{}{}{}{}{}",
                BORDER_COLOR,
                border_char,
                RESET,
                truncated,
                " ".repeat(pad),
                BORDER_COLOR,
                border_char,
                RESET
            )
        } else {
            let padding = inner_width.saturating_sub(char_count);
            format!(
                "{}{}{}{}{}{}{}{}",
                BORDER_COLOR,
                border_char,
                RESET,
                content,
                " ".repeat(padding),
                BORDER_COLOR,
                border_char,
                RESET
            )
        }
    }

    /// Formats a row with custom internal padding (e.g. 2 spaces on left and right).
    pub fn format_padded_row(content: &str, width: usize, indent: usize) -> String {
        let border_char = if is_utf8_supported() { "│" } else { "|" };
        let inner_width = width.saturating_sub(2);
        if inner_width == 0 {
            return String::new();
        }

        let clean = strip_ansi(content);
        let total_content_len = visible_len(&clean) + indent;

        if total_content_len >= inner_width {
            let available = inner_width.saturating_sub(indent);
            let truncated = truncate_ansi(content, available);
            let vis_len = visible_len(&truncated);
            let pad = inner_width.saturating_sub(indent + vis_len);
            format!(
                "{}{}{}{}{}{}{}{}{}",
                BORDER_COLOR,
                border_char,
                RESET,
                " ".repeat(indent),
                truncated,
                " ".repeat(pad),
                BORDER_COLOR,
                border_char,
                RESET
            )
        } else {
            let pad = inner_width.saturating_sub(total_content_len);
            format!(
                "{}{}{}{}{}{}{}{}{}",
                BORDER_COLOR,
                border_char,
                RESET,
                " ".repeat(indent),
                content,
                " ".repeat(pad),
                BORDER_COLOR,
                border_char,
                RESET
            )
        }
    }

    /// Formats a single content string horizontally centered within the side borders `│ ... │`.
    pub fn format_centered_row(content: &str, width: usize) -> String {
        let border_char = if is_utf8_supported() { "│" } else { "|" };
        let inner_width = width.saturating_sub(2);
        if inner_width == 0 {
            return String::new();
        }

        let clean = strip_ansi(content);
        let char_count = visible_len(&clean);

        if char_count >= inner_width {
            let truncated = truncate_ansi(content, inner_width);
            let vis_len = visible_len(&truncated);
            let pad = inner_width.saturating_sub(vis_len);
            format!(
                "{}{}{}{}{}{}{}{}",
                BORDER_COLOR,
                border_char,
                RESET,
                truncated,
                " ".repeat(pad),
                BORDER_COLOR,
                border_char,
                RESET
            )
        } else {
            let pad_left = inner_width.saturating_sub(char_count) / 2;
            let pad_right = inner_width.saturating_sub(char_count + pad_left);
            format!(
                "{}{}{}{}{}{}{}{}{}",
                BORDER_COLOR,
                border_char,
                RESET,
                " ".repeat(pad_left),
                content,
                " ".repeat(pad_right),
                BORDER_COLOR,
                border_char,
                RESET
            )
        }
    }

    /// Renders the entire encapsulated frame to a string with CRLF line endings.
    pub fn render_to_string(&self) -> String {
        let mut out = String::new();
        let w = self.width;

        // 1. Top border
        out.push_str(&box_top(w));
        out.push_str("\r\n");

        // 2. Title & Breadcrumbs
        let effective_title = if self.show_title {
            self.title.clone().or_else(|| {
                let crumbs = crate::nav::get_breadcrumbs();
                if crumbs.len() > 1 {
                    crumbs.last().map(|c| (c.to_uppercase(), false))
                } else {
                    None
                }
            })
        } else {
            None
        };

        if let Some((ref title, is_error)) = effective_title {
            out.push_str(&box_title(title, w, is_error));
            out.push_str("\r\n");

            if self.show_breadcrumbs && !is_error {
                let bc_text = self.breadcrumbs.clone().or_else(|| {
                    let crumbs = crate::nav::get_breadcrumbs();
                    if crumbs.len() > 1 {
                        let inner_width = w.saturating_sub(2);
                        Some(crate::nav::format_breadcrumbs(
                            &crumbs,
                            inner_width.saturating_sub(4),
                        ))
                    } else {
                        None
                    }
                });

                if let Some(ref bc) = bc_text {
                    let inner_w = w.saturating_sub(2);
                    let clean = strip_ansi(bc);
                    let len = visible_len(&clean);
                    let (pad_l, pad_r) = if len < inner_w {
                        let rem = inner_w - len;
                        (rem / 2, rem - rem / 2)
                    } else {
                        (0, 0)
                    };
                    let b_char = if is_utf8_supported() { "│" } else { "|" };
                    let styled_bc = bc.dimmed().to_string();
                    let bc_line = format!(
                        "{}{}{}{}{}{}{}{}{}\r\n",
                        BORDER_COLOR,
                        b_char,
                        RESET,
                        " ".repeat(pad_l),
                        styled_bc,
                        " ".repeat(pad_r),
                        BORDER_COLOR,
                        b_char,
                        RESET
                    );
                    out.push_str(&bc_line);
                }
            }

            out.push_str(&box_divider(w));
            out.push_str("\r\n");
        }

        // 3. Body lines
        for line in &self.lines {
            match line {
                FrameLine::Row(content) => {
                    out.push_str(&Self::format_padded_row(content, w, 2));
                    out.push_str("\r\n");
                }
                FrameLine::Centered(content) => {
                    out.push_str(&Self::format_centered_row(content, w));
                    out.push_str("\r\n");
                }
                FrameLine::RawRow(content) => {
                    out.push_str(&Self::format_row(content, w));
                    out.push_str("\r\n");
                }
                FrameLine::Empty => {
                    out.push_str(&Self::format_row("", w));
                    out.push_str("\r\n");
                }
                FrameLine::Divider => {
                    out.push_str(&box_divider(w));
                    out.push_str("\r\n");
                }
                FrameLine::DividerDimmed => {
                    out.push_str(&format!("{}\r\n", box_divider(w).dimmed()));
                }
            }
        }

        // 4. Footer & Bottom border
        let footer_items = if !self.footer_button_items.is_empty() {
            Some(self.footer_button_items.clone())
        } else {
            self.footer_help.as_ref().map(|help| {
                if help.contains('|') {
                    help.split('|')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<_>>()
                } else {
                    vec![help.clone()]
                }
            })
        };

        if let Some(items) = footer_items {
            if !items.is_empty() {
                out.push_str(&box_divider(w));
                out.push_str("\r\n");

                let inner_w = w.saturating_sub(2);
                let wrapped_lines =
                    wrap_button_items(&items, &self.footer_separator, inner_w, self.footer_center);

                for w_line in wrapped_lines {
                    let styled_help = format!("{}{}{}", "\x1B[90m", w_line, RESET);
                    out.push_str(&Self::format_row(&styled_help, w));
                    out.push_str("\r\n");
                }
            }
        }

        out.push_str(&box_bottom(w));
        out.push_str("\r\n");

        out
    }

    /// Renders this frame directly into stdout, vertically centering the box in the terminal window.
    pub fn render(&self, stdout: &mut io::Stdout) -> Result<()> {
        let (_, term_h) = get_terminal_size();
        let output = self.render_to_string();
        let lines: Vec<&str> = output.lines().collect();
        let box_height = lines.len() as u16;

        let top_padding = if self.vertical_center && term_h > box_height {
            (term_h - box_height) / 2
        } else {
            0
        };

        queue!(stdout, Hide, MoveTo(0, 0))?;
        for _ in 0..top_padding {
            write!(stdout, "\x1B[K\r\n")?;
        }
        for line in lines {
            write!(stdout, "{}\x1B[K\r\n", line)?;
        }
        queue!(stdout, Clear(ClearType::FromCursorDown), Hide)?;
        stdout.flush()?;
        Ok(())
    }
}

/// Renders a responsive, centered warning box when terminal dimensions fall below the minimum threshold.
pub fn render_too_small(term_cols: u16, term_rows: u16, min_cols: u16, min_rows: u16) -> String {
    let card_width = (term_cols as usize).saturating_sub(4).clamp(28, 52);
    let border_char = if is_utf8_supported() { "│" } else { "|" };

    let mut out = String::new();

    // Compute top vertical padding to center the card
    let card_height = 8usize;
    let v_pad = (term_rows as usize).saturating_sub(card_height) / 2;
    for _ in 0..v_pad {
        out.push_str("\r\n");
    }

    let h_pad = (term_cols as usize).saturating_sub(card_width) / 2;
    let h_space = " ".repeat(h_pad);

    // Card Top
    out.push_str(&h_space);
    out.push_str(&box_top(card_width));
    out.push_str("\r\n");

    // Card Title
    let title = "TERMINAL TOO SMALL".red().bold().to_string();
    let clean_title = "TERMINAL TOO SMALL";
    let title_pad = card_width.saturating_sub(2 + clean_title.len()) / 2;
    let title_pad_right = card_width.saturating_sub(2 + clean_title.len() + title_pad);
    out.push_str(&h_space);
    out.push_str(&format!(
        "{}{}{}{}{}{}{}{}{}\r\n",
        BORDER_COLOR,
        border_char,
        RESET,
        " ".repeat(title_pad),
        title,
        " ".repeat(title_pad_right),
        BORDER_COLOR,
        border_char,
        RESET
    ));

    // Card Divider
    out.push_str(&h_space);
    out.push_str(&box_divider(card_width));
    out.push_str("\r\n");

    // Card Body
    let line1 = format!("Current Window:    {} x {} cols/rows", term_cols, term_rows);
    let line2 = format!("Minimum Required:  {} x {} cols/rows", min_cols, min_rows);
    let line3 = "Please resize your terminal window";
    let line4 = "or zoom out to continue.";

    for line in &[line1.as_str(), line2.as_str(), "", line3, line4] {
        out.push_str(&h_space);
        out.push_str(&BoxFrame::format_padded_row(line, card_width, 2));
        out.push_str("\r\n");
    }

    // Card Bottom
    out.push_str(&h_space);
    out.push_str(&box_bottom(card_width));
    out.push_str("\r\n");

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_frame_format_row_padding() {
        let width = 40;
        let formatted = BoxFrame::format_row("Hello World", width);
        let clean = strip_ansi(&formatted);
        assert_eq!(clean.chars().count(), width);
        assert!(clean.starts_with('│') || clean.starts_with('|'));
        assert!(clean.ends_with('│') || clean.ends_with('|'));
    }

    #[test]
    fn test_box_frame_render_contains_borders() {
        let mut frame = BoxFrame::new(60).with_title("TEST TITLE", false);
        frame.row("Row content 1");
        frame.empty_row();
        frame.row("Row content 2");
        frame.footer("Footer help shortcuts");

        let rendered = frame.render_to_string();
        assert!(rendered.contains("TEST TITLE"));
        assert!(rendered.contains("Row content 1"));
        assert!(rendered.contains("Footer help shortcuts"));
    }

    #[test]
    fn test_render_too_small_dimensions() {
        let small_view = render_too_small(45, 10, 60, 14);
        assert!(small_view.contains("TERMINAL TOO SMALL"));
        assert!(small_view.contains("45 x 10"));
        assert!(small_view.contains("60 x 14"));
    }

    #[test]
    fn test_box_frame_format_centered_row() {
        let width = 40;
        let formatted = BoxFrame::format_centered_row("Centered", width);
        let clean = strip_ansi(&formatted);
        assert_eq!(clean.chars().count(), width);
        assert!(clean.starts_with('│') || clean.starts_with('|'));
        assert!(clean.ends_with('│') || clean.ends_with('|'));
        // "Centered" has length 8. Inner width is 38.
        // Left padding = (38 - 8) / 2 = 15. Right padding = 15.
        let trimmed_l = clean.trim_start_matches(['│', '|']);
        assert!(trimmed_l.starts_with(&" ".repeat(15)));
    }

    #[test]
    fn test_box_frame_auto_title_from_nav() {
        let _g1 = crate::nav::NavGuard::enter("First Level");
        let _g2 = crate::nav::NavGuard::enter("Second Level");

        let frame = BoxFrame::new(60);
        let rendered = frame.render_to_string();
        assert!(rendered.contains("SECOND LEVEL"));
        assert!(
            rendered.contains("First Level › Second Level")
                || rendered.contains("First Level > Second Level")
        );
    }
}
