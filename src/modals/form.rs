use colored::Colorize;
use crossterm::{
    cursor::Hide,
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::enable_raw_mode,
};
use std::collections::HashMap;
use std::io;
use std::sync::Arc;

use crate::error::Result;
use crate::frame::BoxFrame;
use crate::keys::{KeyAction, KeyMap};
use crate::modals::confirm::ConfirmModal;
use crate::terminal::{clean_exit, get_content_width, is_terminal_too_small, wait_for_valid_size};

/// Semantic field type with built-in validation and formatting behaviors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormFieldType {
    /// Generic string text input.
    String,
    /// Numeric floating-point input (allows digits, '-', and at most one '.').
    Number,
    /// Whole integer input (allows digits and optional leading '-').
    Integer,
    /// Masked password input (replaces display characters with '*').
    Password,
}

pub type FieldValidator = Arc<dyn Fn(&str) -> std::result::Result<(), String> + Send + Sync>;
pub type FieldForceValidator = Arc<dyn Fn(&str, &str) -> String + Send + Sync>;
pub type FieldFormatter = Arc<dyn Fn(&str) -> String + Send + Sync>;

/// An individual field within a `FormModal`.
#[derive(Clone)]
pub struct FormField {
    pub key: String,
    pub label: String,
    pub field_type: FormFieldType,
    pub value: String,
    pub default_value: String,
    pub placeholder: String,
    pub cursor: usize,
    pub validator: Option<FieldValidator>,
    pub force_validator: Option<FieldForceValidator>,
    pub format_fn: Option<FieldFormatter>,
    pub error: Option<String>,
}

impl FormField {
    /// Creates a generic string input field.
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self::string(key, label)
    }

    /// Creates a string field.
    pub fn string(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            field_type: FormFieldType::String,
            value: String::new(),
            default_value: String::new(),
            placeholder: String::new(),
            cursor: 0,
            validator: None,
            force_validator: None,
            format_fn: None,
            error: None,
        }
    }

    /// Creates an integer number field with automatic integer force validation.
    pub fn integer(key: impl Into<String>, label: impl Into<String>) -> Self {
        let mut field = Self::string(key, label);
        field.field_type = FormFieldType::Integer;
        field.force_validator = Some(Arc::new(|_old: &str, new: &str| {
            if new.is_empty() || new == "-" {
                return new.to_string();
            }
            if new.chars().all(|c| c.is_ascii_digit())
                || (new.starts_with('-') && new[1..].chars().all(|c| c.is_ascii_digit()))
            {
                new.to_string()
            } else {
                _old.to_string()
            }
        }));
        field
    }

    /// Creates a numeric floating-point field with automatic decimal force validation.
    pub fn number(key: impl Into<String>, label: impl Into<String>) -> Self {
        let mut field = Self::string(key, label);
        field.field_type = FormFieldType::Number;
        field.force_validator = Some(Arc::new(|_old: &str, new: &str| {
            if new.is_empty() || new == "-" || new == "." || new == "-." {
                return new.to_string();
            }
            let dot_count = new.chars().filter(|&c| c == '.').count();
            if dot_count <= 1 {
                let without_sign = new.strip_prefix('-').unwrap_or(new);
                if without_sign.chars().all(|c| c.is_ascii_digit() || c == '.') {
                    return new.to_string();
                }
            }
            _old.to_string()
        }));
        field
    }

    /// Creates a password field with automatic masking.
    pub fn password(key: impl Into<String>, label: impl Into<String>) -> Self {
        let mut field = Self::string(key, label);
        field.field_type = FormFieldType::Password;
        field.format_fn = Some(Arc::new(|s: &str| "*".repeat(s.chars().count())));
        field
    }

    /// Sets default value for the field.
    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        let d = default.into();
        self.value = d.clone();
        self.cursor = d.chars().count();
        self.default_value = d;
        self
    }

    /// Sets a placeholder displayed in gray when the field is empty.
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Sets an input validator that produces an error message and blocks submission.
    pub fn with_validator(
        mut self,
        val_fn: impl Fn(&str) -> std::result::Result<(), String> + Send + Sync + 'static,
    ) -> Self {
        self.validator = Some(Arc::new(val_fn));
        self
    }

    /// Sets a force validator: `(old_str, new_str) -> accepted_str`.
    pub fn with_force_validator(
        mut self,
        force_fn: impl Fn(&str, &str) -> String + Send + Sync + 'static,
    ) -> Self {
        self.force_validator = Some(Arc::new(force_fn));
        self
    }

    /// Sets a custom formatting function: `(current_str) -> display_str`.
    pub fn with_format(mut self, fmt_fn: impl Fn(&str) -> String + Send + Sync + 'static) -> Self {
        self.format_fn = Some(Arc::new(fmt_fn));
        self
    }

    /// Inserts a character at the current cursor position, applying force validator.
    pub fn insert_char(&mut self, ch: char) {
        let mut candidate = self.value.clone();
        let char_idx = self
            .value
            .char_indices()
            .nth(self.cursor)
            .map(|(i, _)| i)
            .unwrap_or(self.value.len());
        candidate.insert(char_idx, ch);

        let accepted = if let Some(ref fv) = self.force_validator {
            fv(&self.value, &candidate)
        } else {
            candidate
        };

        if accepted != self.value {
            self.value = accepted;
            self.cursor = (self.cursor + 1).min(self.value.chars().count());
            self.error = None;
        }
    }

    /// Deletes the character preceding the cursor.
    pub fn backspace(&mut self) {
        if self.cursor > 0 && !self.value.is_empty() {
            let mut candidate = self.value.clone();
            let prev_idx = self.cursor - 1;
            let byte_idx = candidate
                .char_indices()
                .nth(prev_idx)
                .map(|(i, _)| i)
                .unwrap_or(0);
            candidate.remove(byte_idx);

            let accepted = if let Some(ref fv) = self.force_validator {
                fv(&self.value, &candidate)
            } else {
                candidate
            };

            self.value = accepted;
            self.cursor = prev_idx.min(self.value.chars().count());
            self.error = None;
        }
    }

    /// Deletes the character directly under the cursor.
    pub fn delete(&mut self) {
        if self.cursor < self.value.chars().count() {
            let mut candidate = self.value.clone();
            let byte_idx = candidate
                .char_indices()
                .nth(self.cursor)
                .map(|(i, _)| i)
                .unwrap_or(0);
            candidate.remove(byte_idx);

            let accepted = if let Some(ref fv) = self.force_validator {
                fv(&self.value, &candidate)
            } else {
                candidate
            };

            self.value = accepted;
            self.error = None;
        }
    }

    /// Formats the field's current value for rendering on screen.
    pub fn display_value(&self) -> String {
        if let Some(ref fmt) = self.format_fn {
            fmt(&self.value)
        } else {
            self.value.clone()
        }
    }

    /// Checks if the current value has been modified from its default value.
    pub fn is_modified(&self) -> bool {
        self.value != self.default_value
    }
}

/// Collection of field values returned upon successful form submission.
#[derive(Debug, Clone, Default)]
pub struct FormResult {
    pub values: HashMap<String, String>,
}

impl FormResult {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(|s| s.as_str())
    }

    pub fn get_string(&self, key: &str) -> String {
        self.values.get(key).cloned().unwrap_or_default()
    }

    pub fn get_u16(&self, key: &str) -> Option<u16> {
        self.values.get(key).and_then(|s| s.trim().parse().ok())
    }

    pub fn get_u32(&self, key: &str) -> Option<u32> {
        self.values.get(key).and_then(|s| s.trim().parse().ok())
    }

    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.values.get(key).and_then(|s| s.trim().parse().ok())
    }

    pub fn get_f64(&self, key: &str) -> Option<f64> {
        self.values.get(key).and_then(|s| s.trim().parse().ok())
    }

    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.values.insert(key.into(), value.into());
    }
}

/// A fully boxed, responsive multi-field interactive form modal.
pub struct FormModal {
    pub title: String,
    pub breadcrumbs: Option<String>,
    pub show_breadcrumbs: bool,
    pub header_rows: Vec<String>,
    pub fields: Vec<FormField>,
    pub focused_idx: usize,
    pub max_width: u16,
    pub confirm_on_cancel: bool,
    pub footer_help: Option<String>,
    pub shortcuts: Option<crate::shortcuts::Shortcuts>,
}

impl FormModal {
    /// Creates a new `FormModal` with given title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            breadcrumbs: None,
            show_breadcrumbs: true,
            header_rows: Vec::new(),
            fields: Vec::new(),
            focused_idx: 0,
            max_width: 0,
            confirm_on_cancel: true,
            footer_help: None,
            shortcuts: None,
        }
    }

    /// Sets explicit navigation breadcrumbs.
    pub fn with_breadcrumbs(mut self, breadcrumbs: impl Into<String>) -> Self {
        self.breadcrumbs = Some(breadcrumbs.into());
        self.show_breadcrumbs = true;
        self
    }

    /// Disables breadcrumb rendering.
    pub fn without_breadcrumbs(mut self) -> Self {
        self.show_breadcrumbs = false;
        self
    }

    /// Adds a descriptive header row above the form fields.
    pub fn with_header_row(mut self, row: impl Into<String>) -> Self {
        self.header_rows.push(row.into());
        self
    }

    /// Adds multiple descriptive header rows above the form fields.
    pub fn with_header_rows(mut self, rows: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.header_rows.extend(rows.into_iter().map(Into::into));
        self
    }

    /// Adds a form field to this modal.
    pub fn with_field(mut self, field: FormField) -> Self {
        self.fields.push(field);
        self
    }

    /// Adds multiple form fields to this modal.
    pub fn with_fields(mut self, fields: impl IntoIterator<Item = FormField>) -> Self {
        self.fields.extend(fields);
        self
    }

    /// Sets maximum box width.
    pub fn with_max_width(mut self, width: u16) -> Self {
        self.max_width = width;
        self
    }

    /// Configures whether pressing Esc prompts for discard confirmation if modified.
    pub fn with_confirm_on_cancel(mut self, confirm: bool) -> Self {
        self.confirm_on_cancel = confirm;
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

    /// Checks if any field has unsaved changes compared to default values.
    pub fn is_modified(&self) -> bool {
        self.fields.iter().any(|f| f.is_modified())
    }

    /// Validates all fields; returns `Some(FormResult)` if valid, or sets field errors and moves focus.
    pub fn validate_and_collect(&mut self) -> Option<FormResult> {
        let mut first_error_idx = None;
        let mut result = FormResult::new();

        for (idx, field) in self.fields.iter_mut().enumerate() {
            if let Some(ref val_fn) = field.validator {
                if let Err(err_msg) = val_fn(&field.value) {
                    field.error = Some(err_msg);
                    if first_error_idx.is_none() {
                        first_error_idx = Some(idx);
                    }
                } else {
                    field.error = None;
                }
            } else {
                field.error = None;
            }
            result.insert(field.key.clone(), field.value.clone());
        }

        if let Some(err_idx) = first_error_idx {
            self.focused_idx = err_idx;
            None
        } else {
            Some(result)
        }
    }

    /// Runs the interactive form modal loop.
    pub fn run(&mut self) -> Result<Option<FormResult>> {
        let mut stdout = io::stdout();
        enable_raw_mode()?;
        let _ = execute!(stdout, Hide);

        if self.focused_idx >= self.fields.len() {
            self.focused_idx = 0;
        }

        let keymap = KeyMap::default();

        loop {
            if is_terminal_too_small() {
                wait_for_valid_size(&mut stdout)?;
                continue;
            }

            let width = get_content_width(self.max_width);
            let mut frame = BoxFrame::new(width);
            frame.title = Some((self.title.clone(), false));
            frame.breadcrumbs = self.breadcrumbs.clone();
            frame.show_breadcrumbs = self.show_breadcrumbs;

            // 3. Header rows
            if !self.header_rows.is_empty() {
                for row in &self.header_rows {
                    frame.row(row);
                }
                frame.divider();
            }

            // 4. Render Form Fields
            for (idx, field) in self.fields.iter().enumerate() {
                let is_focused = idx == self.focused_idx;

                // Field Label
                let trimmed_label = field.label.trim_end_matches(':').trim();
                let label_str = if is_focused {
                    format!("{}:", trimmed_label).cyan().bold().to_string()
                } else {
                    format!("{}:", trimmed_label).dimmed().to_string()
                };
                frame.row(label_str);

                // Field Input Line
                let prefix = if is_focused {
                    "> ".cyan().bold().to_string()
                } else {
                    "  ".to_string()
                };
                let input_line = if field.value.is_empty() {
                    if !field.placeholder.is_empty() {
                        format!("{}{}", prefix, field.placeholder.dimmed())
                    } else if is_focused {
                        format!("{}{}", prefix, " ".reversed())
                    } else {
                        format!("{}{}", prefix, "")
                    }
                } else {
                    let displayed = field.display_value();
                    if is_focused {
                        let char_count = displayed.chars().count();
                        let cur = field.cursor.min(char_count);
                        let before: String = displayed.chars().take(cur).collect();
                        let at_char = displayed.chars().nth(cur).unwrap_or(' ');
                        let after: String = displayed.chars().skip(cur + 1).collect();
                        format!(
                            "{}{}{}{}",
                            prefix,
                            before,
                            at_char.to_string().reversed(),
                            after
                        )
                    } else {
                        format!("{}{}", prefix, displayed)
                    }
                };

                frame.row(input_line);

                // Error line if any
                if let Some(ref err) = field.error {
                    frame.row(format!("  ✗ {}", err).red().bold().to_string());
                }

                // Divider or spacing between fields
                if idx + 1 < self.fields.len() {
                    frame.empty_row();
                }
            }

            // 5. Footer Help
            let shortcuts = self.shortcuts.clone().unwrap_or_else(|| {
                if let Some(ref custom_footer) = self.footer_help {
                    crate::shortcuts::Shortcuts::from(custom_footer.as_str())
                } else {
                    crate::shortcuts::Shortcuts::new()
                        .add("Tab/↓", "Next")
                        .add("Shift+Tab/↑", "Prev")
                        .save()
                        .cancel()
                }
            });
            frame.shortcuts(&shortcuts);

            frame.render(&mut stdout)?;

            match event::read()? {
                Event::Resize(..) => continue,
                Event::Key(key) => {
                    let action = keymap.resolve(&key);

                    // Check Shift+Tab for reverse navigation
                    if key.code == KeyCode::BackTab
                        || (key.modifiers.contains(KeyModifiers::SHIFT) && key.code == KeyCode::Tab)
                    {
                        if self.focused_idx > 0 {
                            self.focused_idx -= 1;
                        } else {
                            self.focused_idx = self.fields.len().saturating_sub(1);
                        }
                        continue;
                    }

                    if key.code == KeyCode::Tab {
                        if self.focused_idx + 1 < self.fields.len() {
                            self.focused_idx += 1;
                        } else {
                            self.focused_idx = 0;
                        }
                        continue;
                    }

                    match action {
                        KeyAction::Quit => {
                            clean_exit();
                        }
                        KeyAction::Up => {
                            if self.focused_idx > 0 {
                                self.focused_idx -= 1;
                            } else {
                                self.focused_idx = self.fields.len().saturating_sub(1);
                            }
                        }
                        KeyAction::Down => {
                            if self.focused_idx + 1 < self.fields.len() {
                                self.focused_idx += 1;
                            } else {
                                self.focused_idx = 0;
                            }
                        }
                        KeyAction::Left => {
                            if let Some(f) = self.fields.get_mut(self.focused_idx) {
                                if f.cursor > 0 {
                                    f.cursor -= 1;
                                }
                            }
                        }
                        KeyAction::Right => {
                            if let Some(f) = self.fields.get_mut(self.focused_idx) {
                                if f.cursor < f.value.chars().count() {
                                    f.cursor += 1;
                                }
                            }
                        }
                        KeyAction::Home => {
                            if let Some(f) = self.fields.get_mut(self.focused_idx) {
                                f.cursor = 0;
                            }
                        }
                        KeyAction::End => {
                            if let Some(f) = self.fields.get_mut(self.focused_idx) {
                                f.cursor = f.value.chars().count();
                            }
                        }
                        KeyAction::Backspace => {
                            if let Some(f) = self.fields.get_mut(self.focused_idx) {
                                f.backspace();
                            }
                        }
                        KeyAction::Delete => {
                            if let Some(f) = self.fields.get_mut(self.focused_idx) {
                                f.delete();
                            }
                        }
                        KeyAction::ClearInput => {
                            if let Some(f) = self.fields.get_mut(self.focused_idx) {
                                f.value.clear();
                                f.cursor = 0;
                                f.error = None;
                            }
                        }
                        KeyAction::Submit => {
                            if self.focused_idx + 1 < self.fields.len() {
                                self.focused_idx += 1;
                            } else if let Some(res) = self.validate_and_collect() {
                                return Ok(Some(res));
                            }
                        }
                        KeyAction::Cancel => {
                            if self.confirm_on_cancel && self.is_modified() {
                                let discard = ConfirmModal::new(
                                    "DISCARD CHANGES",
                                    "You have unsaved changes. Are you sure you want to discard them?",
                                )
                                .default_yes(false)
                                .run()?;

                                if discard == crate::modals::confirm::ConfirmOutcome::Confirmed {
                                    return Ok(None);
                                }
                            } else {
                                return Ok(None);
                            }
                        }
                        KeyAction::Hotkey(c) => {
                            if let Some(f) = self.fields.get_mut(self.focused_idx) {
                                f.insert_char(c);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_form_field_integer_force_validator() {
        let mut field = FormField::integer("port", "Port").with_default("22");
        assert_eq!(field.value, "22");

        // Allowed digits
        field.insert_char('2');
        assert_eq!(field.value, "222");

        // Disallowed letters
        field.insert_char('a');
        assert_eq!(field.value, "222");

        field.insert_char('X');
        assert_eq!(field.value, "222");
    }

    #[test]
    fn test_form_field_number_force_validator() {
        let mut field = FormField::number("price", "Price").with_default("10");

        field.insert_char('.');
        field.insert_char('5');
        assert_eq!(field.value, "10.5");

        // Second decimal point is rejected
        field.insert_char('.');
        assert_eq!(field.value, "10.5");

        // Letters rejected
        field.insert_char('z');
        assert_eq!(field.value, "10.5");
    }

    #[test]
    fn test_form_field_password_masking() {
        let mut field = FormField::password("pw", "Password").with_default("secret");
        assert_eq!(field.display_value(), "******");

        field.insert_char('1');
        assert_eq!(field.display_value(), "*******");
        assert_eq!(field.value, "secret1");
    }

    #[test]
    fn test_form_modal_validation() {
        let mut form = FormModal::new("TEST FORM")
            .with_field(
                FormField::string("alias", "Alias")
                    .with_default("my-host")
                    .with_validator(|s| {
                        if s.is_empty() {
                            Err("Alias cannot be empty".to_string())
                        } else {
                            Ok(())
                        }
                    }),
            )
            .with_field(
                FormField::integer("port", "Port")
                    .with_default("99999")
                    .with_validator(|s| match s.parse::<u16>() {
                        Ok(_) => Ok(()),
                        Err(_) => Err("Invalid port (1-65535)".to_string()),
                    }),
            );

        // First attempt: port 99999 is invalid u16
        assert!(form.validate_and_collect().is_none());
        assert_eq!(form.focused_idx, 1);
        assert_eq!(
            form.fields[1].error.as_deref(),
            Some("Invalid port (1-65535)")
        );

        // Fix port
        form.fields[1].value = "22".to_string();
        let res = form.validate_and_collect().expect("should validate");
        assert_eq!(res.get_string("alias"), "my-host");
        assert_eq!(res.get_u16("port"), Some(22));
    }
}
