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
use crate::terminal::{
    clean_exit, get_content_width, get_terminal_size, is_terminal_too_small, wait_for_valid_size,
};
use crate::text_flow::truncate_ellipsis;
use crate::theme::strip_ansi;

/// Alignment options for table columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColumnAlign {
    #[default]
    Left,
    Center,
    Right,
}

/// Column specification for `TableModal`.
#[derive(Debug, Clone)]
pub struct TableColumn {
    pub header: String,
    pub width: usize,
    pub align: ColumnAlign,
    pub align_right: bool,
}

impl TableColumn {
    pub fn new(header: impl Into<String>, width: usize) -> Self {
        Self {
            header: header.into(),
            width,
            align: ColumnAlign::Left,
            align_right: false,
        }
    }

    pub fn with_align(mut self, align: ColumnAlign) -> Self {
        self.align = align;
        if align == ColumnAlign::Right {
            self.align_right = true;
        }
        self
    }

    pub fn right_aligned(mut self) -> Self {
        self.align = ColumnAlign::Right;
        self.align_right = true;
        self
    }
}

/// Outcome returned by running a `TableModal`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TableOutcome {
    Selected(usize),
    Cancelled,
}

/// A fully boxed, scrollable tabular data viewer and row selection modal.
#[derive(Debug, Clone)]
pub struct TableModal {
    pub title: String,
    pub columns: Vec<TableColumn>,
    pub rows: Vec<Vec<String>>,
    pub selectable: bool,
    pub keymap: KeyMap,
    pub max_width: u16,
    pub footer_help: Option<String>,
    pub shortcuts: Option<crate::shortcuts::Shortcuts>,
}

impl TableModal {
    /// Creates a new `TableModal`.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            columns: Vec::new(),
            rows: Vec::new(),
            selectable: true,
            keymap: KeyMap::default(),
            max_width: 0,
            footer_help: None,
            shortcuts: None,
        }
    }

    /// Adds a column to the table.
    pub fn with_column(mut self, column: TableColumn) -> Self {
        self.columns.push(column);
        self
    }

    /// Sets the list of columns.
    pub fn with_columns(mut self, columns: Vec<TableColumn>) -> Self {
        self.columns = columns;
        self
    }

    /// Sets all rows in the table.
    pub fn with_rows(mut self, rows: Vec<Vec<String>>) -> Self {
        self.rows = rows;
        self
    }

    /// Appends a single row to the table.
    pub fn with_row(mut self, row: Vec<String>) -> Self {
        self.rows.push(row);
        self
    }

    /// Configures whether individual rows can be selected.
    pub fn with_selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    /// Sets maximum desired box content width.
    pub fn with_max_width(mut self, width: u16) -> Self {
        self.max_width = width;
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

    /// Runs the interactive table viewer and returns the selected row index if confirmed.
    pub fn run(&self, selected_idx: &mut usize) -> Result<TableOutcome> {
        let mut stdout = io::stdout();
        enable_raw_mode()?;
        let _ = execute!(stdout, Hide);

        if self.rows.is_empty() || *selected_idx >= self.rows.len() {
            *selected_idx = 0;
        }

        let mut scroll_offset: usize = 0;

        let res = (|| -> Result<TableOutcome> {
            loop {
                if is_terminal_too_small() {
                    wait_for_valid_size(&mut stdout)?;
                    continue;
                }

                let (_, term_h) = get_terminal_size();
                let width = get_content_width(self.max_width);

                // Overhead: top (1) + title/divider (2) + table header/divider (2) + footer (2) + bottom (1) = 8
                let overhead = 8;
                let viewport_size = (term_h as usize).saturating_sub(overhead).max(3);

                if *selected_idx < scroll_offset {
                    scroll_offset = *selected_idx;
                } else if *selected_idx >= scroll_offset + viewport_size {
                    scroll_offset = *selected_idx - viewport_size + 1;
                }
                if scroll_offset + viewport_size > self.rows.len() {
                    scroll_offset = self.rows.len().saturating_sub(viewport_size);
                }

                let mut frame = BoxFrame::new(width);
                frame.title = Some((self.title.clone(), false));

                // 1. Table Header Row
                let mut header_parts = Vec::new();
                if self.selectable {
                    header_parts.push("   ".to_string());
                }

                for col in &self.columns {
                    let h_clean = strip_ansi(&col.header);
                    let w = col.width;
                    if h_clean.len() >= w {
                        header_parts.push(col.header.clone());
                    } else if col.align_right {
                        let pad = w - h_clean.len();
                        header_parts.push(format!(
                            "{}{}",
                            " ".repeat(pad),
                            col.header.cyan().bold()
                        ));
                    } else {
                        let pad = w - h_clean.len();
                        header_parts.push(format!(
                            "{}{}",
                            col.header.cyan().bold(),
                            " ".repeat(pad)
                        ));
                    }
                }
                frame.row(header_parts.join(" "));
                frame.divider_dimmed();

                // 2. Data Rows
                if scroll_offset > 0 {
                    frame.row(
                        format!("[▲ {} rows above]", scroll_offset)
                            .dimmed()
                            .to_string(),
                    );
                }

                let end_idx = self.rows.len().min(scroll_offset + viewport_size);
                for (local_i, row) in self.rows[scroll_offset..end_idx].iter().enumerate() {
                    let abs_i = scroll_offset + local_i;
                    let is_active = self.selectable && abs_i == *selected_idx;

                    let mut row_parts = Vec::new();
                    if self.selectable {
                        if is_active {
                            row_parts.push(">  ".cyan().bold().to_string());
                        } else {
                            row_parts.push("   ".to_string());
                        }
                    }

                    for (col_i, cell) in row.iter().enumerate() {
                        let col_w = self.columns.get(col_i).map(|c| c.width).unwrap_or(12);
                        let align_right = self
                            .columns
                            .get(col_i)
                            .map(|c| c.align_right)
                            .unwrap_or(false);
                        let clean_cell = strip_ansi(cell);
                        let cell_len = clean_cell.chars().count();

                        let formatted_cell = if cell_len >= col_w {
                            let keep = col_w.saturating_sub(1);
                            truncate_ellipsis(&clean_cell, keep)
                        } else if align_right {
                            let pad = col_w.saturating_sub(cell_len);
                            format!("{}{}", " ".repeat(pad), cell)
                        } else {
                            let pad = col_w.saturating_sub(cell_len);
                            format!("{}{}", cell, " ".repeat(pad))
                        };

                        if is_active {
                            row_parts.push(formatted_cell.white().bold().to_string());
                        } else {
                            row_parts.push(formatted_cell);
                        }
                    }

                    frame.row(row_parts.join(" "));
                }

                if end_idx < self.rows.len() {
                    frame.row(
                        format!("[▼ {} rows below]", self.rows.len() - end_idx)
                            .dimmed()
                            .to_string(),
                    );
                }

                let shortcuts = self.shortcuts.clone().unwrap_or_else(|| {
                    if let Some(ref custom_footer) = self.footer_help {
                        crate::shortcuts::Shortcuts::from(custom_footer.as_str())
                    } else {
                        let mut sc = crate::shortcuts::Shortcuts::new();
                        if self.selectable {
                            sc = sc.add("↑/↓", "Select Row");
                        }
                        if self.rows.len() > viewport_size {
                            sc = sc.page();
                        }
                        if self.selectable {
                            sc = sc.add("Enter", "Open");
                        }
                        sc = sc.back();
                        sc.exit_if(self.keymap.allow_quit_on_q)
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
                                if !self.rows.is_empty() {
                                    if *selected_idx > 0 {
                                        *selected_idx -= 1;
                                    } else {
                                        *selected_idx = self.rows.len().saturating_sub(1);
                                    }
                                }
                            }
                            KeyAction::Down => {
                                if !self.rows.is_empty() {
                                    if *selected_idx + 1 < self.rows.len() {
                                        *selected_idx += 1;
                                    } else {
                                        *selected_idx = 0;
                                    }
                                }
                            }
                            KeyAction::PageUp => {
                                *selected_idx = selected_idx.saturating_sub(viewport_size);
                            }
                            KeyAction::PageDown => {
                                *selected_idx = (*selected_idx + viewport_size)
                                    .min(self.rows.len().saturating_sub(viewport_size));
                            }
                            KeyAction::Home => {
                                *selected_idx = 0;
                            }
                            KeyAction::End => {
                                *selected_idx = self.rows.len().saturating_sub(1);
                            }
                            KeyAction::Submit if self.selectable => {
                                if !self.rows.is_empty() {
                                    return Ok(TableOutcome::Selected(*selected_idx));
                                }
                            }
                            KeyAction::Cancel => {
                                return Ok(TableOutcome::Cancelled);
                            }
                            _ => {}
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
    fn test_table_modal_builder() {
        let modal = TableModal::new("SERVER INSTANCES")
            .with_columns(vec![
                TableColumn::new("Name", 16),
                TableColumn::new("Software", 12),
                TableColumn::new("Port", 6).right_aligned(),
            ])
            .with_row(vec!["vanilla-1".into(), "vanilla".into(), "25565".into()])
            .with_row(vec!["paper-1".into(), "paper".into(), "25566".into()])
            .with_selectable(true);

        assert_eq!(modal.title, "SERVER INSTANCES");
        assert_eq!(modal.columns.len(), 3);
        assert_eq!(modal.columns[0].header, "Name");
        assert_eq!(modal.columns[1].header, "Software");
        assert!(modal.columns[2].align_right);
        assert_eq!(modal.rows.len(), 2);
        assert!(modal.selectable);
    }
}
