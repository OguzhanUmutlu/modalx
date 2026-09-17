use colored::Colorize;
use crossterm::{
    cursor::Hide,
    event::{self, Event},
    execute,
    terminal::enable_raw_mode,
};
use std::io;

use crate::error::Result;
use crate::frame::BoxFrame;
use crate::keys::{KeyAction, KeyMap};
use crate::section::{FieldSection, ModalSection, SelectItem};
use crate::terminal::{
    clean_exit, get_content_width, get_terminal_size, is_terminal_too_small, wait_for_valid_size,
};
use crate::text_flow::{wrap_delimited_string, wrap_words};
use crate::theme::strip_ansi;

/// Outcome returned by running a `SelectModal`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectOutcome {
    Selected(usize),
    Toggled(usize),
    ItemAction(char, usize),
    Cancelled,
}

/// A high-level menu modal with responsive layout, title, breadcrumbs, and shortcut support.
pub type MenuModal = SelectModal;

/// A fully boxed, responsive menu/button selection modal.
#[derive(Debug, Clone)]
pub struct SelectModal {
    pub title: Option<(String, bool)>,
    pub breadcrumbs: Option<String>,
    pub show_breadcrumbs: bool,
    pub sections: Vec<ModalSection>,
    pub header_rows: Vec<String>,
    pub entries: Vec<SelectItem>,
    pub allow_toggle: bool,
    pub allow_quit_on_q: bool,
    pub keymap: KeyMap,
    pub item_actions: Vec<char>,
    pub footer_help: Option<String>,
    pub shortcuts: Option<crate::shortcuts::Shortcuts>,
    pub max_width: u16,
    pub wrap_around: bool,
}

impl Default for SelectModal {
    fn default() -> Self {
        Self::new()
    }
}

impl SelectModal {
    /// Creates a new empty `SelectModal`.
    pub fn new() -> Self {
        Self {
            title: None,
            breadcrumbs: None,
            show_breadcrumbs: true,
            sections: Vec::new(),
            header_rows: Vec::new(),
            entries: Vec::new(),
            allow_toggle: false,
            allow_quit_on_q: true,
            keymap: KeyMap::menu_default(true),
            item_actions: Vec::new(),
            footer_help: None,
            shortcuts: None,
            max_width: 0,
            wrap_around: false,
        }
    }

    /// Creates a high-level `MenuModal` with title and automatic navigation breadcrumbs.
    pub fn menu(title: impl Into<String>) -> Self {
        Self::new().with_title(title, false)
    }

    /// Convenience constructor with title.
    pub fn with_name(title: impl Into<String>) -> Self {
        Self::new().with_title(title, false)
    }

    /// Adds a composable `ModalSection`.
    pub fn with_section(mut self, section: ModalSection) -> Self {
        self.sections.push(section);
        self
    }

    /// Adds multiple `ModalSection` items.
    pub fn with_sections(mut self, sections: impl IntoIterator<Item = ModalSection>) -> Self {
        self.sections.extend(sections);
        self
    }

    /// Adds a responsive field section from items and separator.
    pub fn with_fields<T: AsRef<str>>(mut self, fields: &[T], separator: &str) -> Self {
        let items = fields.iter().map(|f| f.as_ref().to_string()).collect();
        self.sections
            .push(ModalSection::Fields(FieldSection::with_items(
                items, separator,
            )));
        self
    }

    /// Adds a responsive field section from a raw delimited string.
    pub fn with_raw_fields(mut self, raw: impl Into<String>, separator: impl Into<String>) -> Self {
        self.sections
            .push(ModalSection::Fields(FieldSection::from_raw(raw, separator)));
        self
    }

    /// Creates a `SelectModal` by parsing a legacy dashboard header string and list of menu entries.
    pub fn from_legacy(header: &str, entries: &[SelectItem]) -> Self {
        let (parsed_title, parsed_rows) = parse_header_lines(header);
        let mut modal = Self::new().with_entries(entries.to_vec());
        if let Some(title) = parsed_title {
            modal = modal.with_title(title, false);
        }
        modal.header_rows = parsed_rows;
        modal
    }

    /// Sets modal title.
    pub fn with_title(mut self, title: impl Into<String>, is_error: bool) -> Self {
        self.title = Some((title.into(), is_error));
        self
    }

    /// Sets navigation breadcrumbs.
    pub fn with_breadcrumbs(mut self, breadcrumbs: impl Into<String>) -> Self {
        self.breadcrumbs = Some(breadcrumbs.into());
        self.show_breadcrumbs = true;
        self
    }

    /// Disables navigation breadcrumbs rendering on this modal.
    pub fn without_breadcrumbs(mut self) -> Self {
        self.show_breadcrumbs = false;
        self
    }

    /// Adds a metadata content row inside the box above the menu entries.
    pub fn with_header_row(mut self, row: impl Into<String>) -> Self {
        self.header_rows.push(row.into());
        self
    }

    /// Sets all metadata header rows.
    pub fn with_header_rows(mut self, rows: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.header_rows = rows.into_iter().map(Into::into).collect();
        self
    }

    /// Sets menu entries.
    pub fn with_entries(mut self, entries: Vec<SelectItem>) -> Self {
        self.entries = entries;
        self
    }

    /// Adds a single menu entry.
    pub fn with_entry(mut self, entry: SelectItem) -> Self {
        self.entries.push(entry);
        self
    }

    /// Fluent helper to add an item.
    pub fn item(mut self, hotkey: impl Into<String>, label: impl Into<String>) -> Self {
        self.entries.push(SelectItem::new(hotkey, label));
        self
    }

    /// Fluent helper to add an item with a description.
    pub fn item_with_desc(
        mut self,
        hotkey: impl Into<String>,
        label: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        self.entries
            .push(SelectItem::new(hotkey, label).with_description(description));
        self
    }

    /// Enables or disables Spacebar toggle behavior.
    pub fn with_allow_toggle(mut self, allow: bool) -> Self {
        self.allow_toggle = allow;
        self
    }

    /// Configures whether pressing 'q' quits the application.
    pub fn with_allow_quit_on_q(mut self, allow: bool) -> Self {
        self.allow_quit_on_q = allow;
        self.keymap.allow_quit_on_q = allow;
        self
    }

    /// Sets a custom `KeyMap`.
    pub fn with_keymap(mut self, keymap: KeyMap) -> Self {
        self.keymap = keymap;
        self
    }

    /// Sets custom bottom footer help text.
    pub fn with_footer_help(mut self, help: impl Into<String>) -> Self {
        self.footer_help = Some(help.into());
        self
    }

    /// Sets custom bottom footer shortcuts.
    pub fn with_shortcuts(mut self, shortcuts: impl Into<crate::shortcuts::Shortcuts>) -> Self {
        self.shortcuts = Some(shortcuts.into());
        self
    }

    /// Registers a single item-scoped action hotkey (e.g. 'e' to edit highlighted item).
    pub fn with_item_action(mut self, ch: char) -> Self {
        self.item_actions.push(ch.to_ascii_lowercase());
        self
    }

    /// Registers multiple item-scoped action hotkeys.
    pub fn with_item_actions(mut self, chars: impl IntoIterator<Item = char>) -> Self {
        self.item_actions
            .extend(chars.into_iter().map(|c| c.to_ascii_lowercase()));
        self
    }

    /// Sets maximum desired box content width (defaults to 80).
    pub fn with_max_width(mut self, width: u16) -> Self {
        self.max_width = width;
        self
    }

    /// Sets whether arrow key navigation wraps around from top to bottom and bottom to top.
    pub fn with_wrap_around(mut self, wrap_around: bool) -> Self {
        self.wrap_around = wrap_around;
        self
    }

    /// Executes the interactive menu event loop, updating `selected_idx`.
    pub fn run(&self, selected_idx: &mut usize) -> Result<SelectOutcome> {
        let mut stdout = io::stdout();
        enable_raw_mode()?;
        let _ = execute!(stdout, Hide);

        if self.entries.is_empty() || *selected_idx >= self.entries.len() {
            *selected_idx = 0;
        }

        let mut scroll_offset: usize = 0;

        loop {
            if is_terminal_too_small() {
                wait_for_valid_size(&mut stdout)?;
                continue;
            }

            let (_, term_h) = get_terminal_size();
            let width = get_content_width(self.max_width);
            let available_content_width = width.saturating_sub(6);

            // Dynamically build metadata rows from both sections and legacy header_rows,
            // wrapping any field groups or delimited lines to fit within available_content_width.
            let mut rendered_metadata_rows: Vec<String> = Vec::new();

            for sec in &self.sections {
                match sec {
                    ModalSection::Fields(ref fs) => {
                        rendered_metadata_rows.extend(fs.render(available_content_width));
                    }
                    ModalSection::Text(ref ts) => {
                        rendered_metadata_rows.extend(ts.render(available_content_width));
                    }
                    ModalSection::Custom(ref lines) => {
                        rendered_metadata_rows.extend(lines.clone());
                    }
                    _ => {}
                }
            }

            for row in &self.header_rows {
                if row.contains(" | ") {
                    rendered_metadata_rows.extend(wrap_delimited_string(
                        row,
                        " | ",
                        available_content_width,
                    ));
                } else if strip_ansi(row).chars().count() > available_content_width {
                    rendered_metadata_rows.extend(wrap_words(row, available_content_width));
                } else {
                    rendered_metadata_rows.push(row.clone());
                }
            }

            let effective_title = self.title.clone().or_else(|| {
                let crumbs = crate::nav::get_breadcrumbs();
                if crumbs.len() > 1 {
                    crumbs.last().map(|c| (c.to_uppercase(), false))
                } else {
                    None
                }
            });

            // Overhead: top border (1) + title/divider (2 if present) + breadcrumbs (1 if present)
            // + rendered metadata rows + divider (1 if metadata present) + footer/bottom (3)
            let overhead = 1
                + if effective_title.is_some() { 2 } else { 0 }
                + if self.show_breadcrumbs
                    && (self.breadcrumbs.is_some() || crate::nav::NavGuard::has_crumbs())
                {
                    1
                } else {
                    0
                }
                + rendered_metadata_rows.len()
                + if !rendered_metadata_rows.is_empty() {
                    1
                } else {
                    0
                }
                + 3;

            let viewport_size = (term_h as usize).saturating_sub(overhead).max(3);

            // Keep selected index visible
            if *selected_idx < scroll_offset {
                scroll_offset = *selected_idx;
            } else if *selected_idx >= scroll_offset + viewport_size {
                scroll_offset = *selected_idx - viewport_size + 1;
            }
            if scroll_offset + viewport_size > self.entries.len() {
                scroll_offset = self.entries.len().saturating_sub(viewport_size);
            }

            // Build BoxFrame
            let mut frame = BoxFrame::new(width);
            if let Some((ref title, is_err)) = effective_title {
                frame.title = Some((title.clone(), is_err));
            }
            if let Some(ref bc) = self.breadcrumbs {
                frame.breadcrumbs = Some(bc.clone());
            }
            frame.show_breadcrumbs = self.show_breadcrumbs;

            for row in &rendered_metadata_rows {
                frame.row(row);
            }
            if !rendered_metadata_rows.is_empty() {
                frame.divider();
            }

            if scroll_offset > 0 {
                frame.row(
                    format!("[▲ {} more items above]", scroll_offset)
                        .dimmed()
                        .to_string(),
                );
            }

            let end_idx = self.entries.len().min(scroll_offset + viewport_size);
            for (local_i, entry) in self.entries[scroll_offset..end_idx].iter().enumerate() {
                let abs_i = scroll_offset + local_i;
                let badge = format!("[{}]", entry.hotkey);
                let row_str = if abs_i == *selected_idx {
                    format!(
                        "> {:<5} {}",
                        badge.cyan().bold(),
                        entry.label.white().bold()
                    )
                } else {
                    format!("  {:<5} {}", badge.cyan(), entry.label)
                };
                frame.row(row_str);
            }

            if end_idx < self.entries.len() {
                frame.row(
                    format!("[▼ {} more items below]", self.entries.len() - end_idx)
                        .dimmed()
                        .to_string(),
                );
            }

            let shortcuts = self.shortcuts.clone().unwrap_or_else(|| {
                if let Some(ref custom_footer) = self.footer_help {
                    crate::shortcuts::Shortcuts::from(custom_footer.as_str())
                } else {
                    let mut sc = crate::shortcuts::Shortcuts::new().move_selection();
                    if self.allow_toggle {
                        sc = sc.toggle().confirm().cancel();
                    } else {
                        sc = sc.select().back();
                    }
                    sc.exit_if(self.allow_quit_on_q)
                }
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
                            if !self.entries.is_empty() {
                                if *selected_idx > 0 {
                                    *selected_idx -= 1;
                                } else if self.wrap_around {
                                    *selected_idx = self.entries.len().saturating_sub(1);
                                }
                            }
                        }
                        KeyAction::Down => {
                            if !self.entries.is_empty() {
                                if *selected_idx + 1 < self.entries.len() {
                                    *selected_idx += 1;
                                } else if self.wrap_around {
                                    *selected_idx = 0;
                                }
                            }
                        }
                        KeyAction::PageUp => {
                            *selected_idx = selected_idx.saturating_sub(viewport_size);
                        }
                        KeyAction::PageDown => {
                            *selected_idx = (*selected_idx + viewport_size)
                                .min(self.entries.len().saturating_sub(1));
                        }
                        KeyAction::Home => {
                            *selected_idx = 0;
                        }
                        KeyAction::End => {
                            *selected_idx = self.entries.len().saturating_sub(1);
                        }
                        KeyAction::Submit | KeyAction::Right => {
                            if !self.entries.is_empty() {
                                return Ok(SelectOutcome::Selected(*selected_idx));
                            }
                        }
                        KeyAction::Toggle if self.allow_toggle => {
                            if !self.entries.is_empty() {
                                return Ok(SelectOutcome::Toggled(*selected_idx));
                            }
                        }
                        KeyAction::Cancel | KeyAction::Left => {
                            return Ok(SelectOutcome::Cancelled);
                        }
                        KeyAction::Hotkey(c) => {
                            let c_str = c.to_ascii_lowercase().to_string();
                            for (idx, entry) in self.entries.iter().enumerate() {
                                if entry.hotkey.eq_ignore_ascii_case(&c_str)
                                    || entry.aliases.iter().any(|a| a.eq_ignore_ascii_case(&c_str))
                                {
                                    *selected_idx = idx;
                                    return Ok(SelectOutcome::Selected(idx));
                                }
                            }
                            if self
                                .item_actions
                                .iter()
                                .any(|&a| a.eq_ignore_ascii_case(&c))
                                && !self.entries.is_empty()
                            {
                                return Ok(SelectOutcome::ItemAction(
                                    c.to_ascii_lowercase(),
                                    *selected_idx,
                                ));
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}

/// Parses legacy header strings into a clean title and individual metadata rows.
pub fn parse_header_lines(header: &str) -> (Option<String>, Vec<String>) {
    let mut title = None;
    let mut rows = Vec::new();

    let lines: Vec<&str> = header.lines().collect();
    let total_lines = lines.len();

    for raw_line in lines {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let clean = strip_ansi(trimmed);
        let clean_trimmed = clean.trim();

        // 1. Skip top borders
        if (clean_trimmed.starts_with('╭')
            || clean_trimmed.starts_with('┌')
            || clean_trimmed.starts_with('+'))
            && (clean_trimmed.ends_with('╮')
                || clean_trimmed.ends_with('┐')
                || clean_trimmed.ends_with('+'))
            && clean_trimmed.chars().all(|c| {
                c == '╭' || c == '╮' || c == '┌' || c == '┐' || c == '─' || c == '-' || c == '+'
            })
        {
            continue;
        }

        // 2. Skip internal dividers
        if (clean_trimmed.starts_with('├') || clean_trimmed.starts_with('+'))
            && (clean_trimmed.ends_with('┤') || clean_trimmed.ends_with('+'))
            && clean_trimmed
                .chars()
                .all(|c| c == '├' || c == '┤' || c == '─' || c == '-' || c == '+')
        {
            continue;
        }

        // 3. Skip bottom borders
        if (clean_trimmed.starts_with('╰')
            || clean_trimmed.starts_with('└')
            || clean_trimmed.starts_with('+'))
            && (clean_trimmed.ends_with('╯')
                || clean_trimmed.ends_with('┘')
                || clean_trimmed.ends_with('+'))
            && clean_trimmed.chars().all(|c| {
                c == '╰' || c == '╯' || c == '└' || c == '┘' || c == '─' || c == '-' || c == '+'
            })
        {
            continue;
        }

        // 4. Check for framed title: starts and ends with │ or |
        if (clean_trimmed.starts_with('│') || clean_trimmed.starts_with('|'))
            && (clean_trimmed.ends_with('│') || clean_trimmed.ends_with('|'))
        {
            let inner = clean_trimmed.trim_matches(|c| c == '│' || c == '|').trim();
            // Defensive: ignore breadcrumb lines that might be inside legacy frames
            if inner.contains(" › ") || inner.contains(" > ") {
                continue;
            }
            if title.is_none() && !inner.is_empty() {
                title = Some(inner.to_string());
                continue;
            }
        }

        // 5. If single line header with no borders, treat it as the title
        if title.is_none() && total_lines == 1 {
            title = Some(clean_trimmed.to_string());
            continue;
        }

        // 6. Otherwise, treat as content row
        let clean_inner = clean_trimmed.trim_matches(|c| c == '│' || c == '|').trim();
        // Skip breadcrumbs so they are NEVER rendered as a content row inside the box
        if clean_inner.contains(" › ") || clean_inner.contains(" > ") {
            continue;
        }

        let content = if (clean_trimmed.starts_with('│') || clean_trimmed.starts_with('|'))
            && (clean_trimmed.ends_with('│') || clean_trimmed.ends_with('|'))
        {
            clean_inner.to_string()
        } else {
            raw_line.trim().to_string()
        };

        if !content.is_empty() {
            rows.push(content);
        }
    }

    (title, rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_header_lines_legacy() {
        let header = "╭───╮\r\n│ DASHBOARD │\r\n├───┤\r\n Host: Ubuntu | RAM: 8GB\r\n Registered: 1\r\n├───┤";
        let (title, rows) = parse_header_lines(header);
        assert_eq!(title.as_deref(), Some("DASHBOARD"));
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], "Host: Ubuntu | RAM: 8GB");
        assert_eq!(rows[1], "Registered: 1");
    }

    #[test]
    fn test_parse_single_line_title() {
        let header = "Select Server Difficulty:";
        let (title, rows) = parse_header_lines(header);
        assert_eq!(title.as_deref(), Some("Select Server Difficulty:"));
        assert!(rows.is_empty());
    }

    #[test]
    fn test_parse_header_lines_filters_breadcrumbs() {
        let header = "╭───────────────────────────────────────────────────╮\r\n│                   LOCAL SERVERS                   │\r\n│             Dashboard › Local Servers             │\r\n├───────────────────────────────────────────────────┤\r\n Manage local servers on this host (Total: 1).\r\n├───────────────────────────────────────────────────┤";
        let (title, rows) = parse_header_lines(header);
        assert_eq!(title.as_deref(), Some("LOCAL SERVERS"));
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0], "Manage local servers on this host (Total: 1).");
    }

    #[test]
    fn test_select_modal_with_fields_builder() {
        let modal = SelectModal::new()
            .with_title("TEST DASHBOARD", false)
            .with_fields(&["Host: Ubuntu", "RAM: 16GB", "Daemon: [ONLINE]"], " | ")
            .with_fields(&["Servers: 3", "Running: 1"], " | ")
            .with_entry(SelectItem::new("1", "Local Servers"));

        assert_eq!(modal.sections.len(), 2);
        assert_eq!(modal.entries.len(), 1);
    }

    #[test]
    fn test_menu_modal_builder() {
        let modal = MenuModal::menu("OFFLINE ACTIONS")
            .with_header_row("Host: root@127.0.0.1:22")
            .with_entry(SelectItem::new("a", "Authentication"));

        assert_eq!(
            modal.title.as_ref().map(|(t, _)| t.as_str()),
            Some("OFFLINE ACTIONS")
        );
        assert_eq!(
            modal.header_rows,
            vec!["Host: root@127.0.0.1:22".to_string()]
        );
        assert_eq!(modal.entries.len(), 1);
    }

    #[test]
    fn test_select_modal_wrap_around() {
        let modal_default = SelectModal::new();
        assert!(!modal_default.wrap_around);

        let modal_wrapped = SelectModal::new().with_wrap_around(true);
        assert!(modal_wrapped.wrap_around);
    }
}
