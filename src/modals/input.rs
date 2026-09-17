use std::sync::Arc;

use crate::error::Result;
use crate::keys::KeyMap;
use crate::modals::form::{FormField, FormModal};

/// Outcome returned by running an `InputModal`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputOutcome {
    Submitted(String),
    Cancelled,
}

pub type InputValidator = Arc<dyn Fn(&str) -> std::result::Result<(), String> + Send + Sync>;

/// A fully boxed, responsive single-field input modal built as a specialization of `FormModal`.
pub struct InputModal {
    pub title: String,
    pub prompt: String,
    pub default_val: Option<String>,
    pub placeholder: Option<String>,
    pub is_password: bool,
    pub breadcrumbs: Option<String>,
    pub show_breadcrumbs: bool,
    pub keymap: KeyMap,
    pub max_width: u16,
    pub validator: Option<InputValidator>,
}

impl InputModal {
    /// Creates a new `InputModal`.
    pub fn new(title: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            prompt: prompt.into(),
            default_val: None,
            placeholder: None,
            is_password: false,
            breadcrumbs: None,
            show_breadcrumbs: true,
            keymap: KeyMap::default(),
            max_width: 0,
            validator: None,
        }
    }

    /// Sets explicit navigation breadcrumbs.
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

    /// Sets default value for input.
    pub fn with_default(mut self, val: impl Into<String>) -> Self {
        self.default_val = Some(val.into());
        self
    }

    /// Sets placeholder shown when input buffer is empty.
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Sets whether input characters should be masked with `*`.
    pub fn with_password(mut self, is_pw: bool) -> Self {
        self.is_password = is_pw;
        self
    }

    /// Sets an input validator function.
    pub fn with_validator(
        mut self,
        v: impl Fn(&str) -> std::result::Result<(), String> + Send + Sync + 'static,
    ) -> Self {
        self.validator = Some(Arc::new(v));
        self
    }

    /// Sets maximum desired box content width.
    pub fn with_max_width(mut self, width: u16) -> Self {
        self.max_width = width;
        self
    }

    /// Runs the interactive input modal loop, delegating to `FormModal`.
    pub fn run(&self) -> Result<InputOutcome> {
        let mut field = if self.is_password {
            FormField::password("input", &self.prompt)
        } else {
            FormField::string("input", &self.prompt)
        };
        if let Some(ref d) = self.default_val {
            field = field.with_default(d);
        }
        if let Some(ref p) = self.placeholder {
            field = field.with_placeholder(p);
        }
        if let Some(ref v) = self.validator {
            let v_clone = Arc::clone(v);
            field = field.with_validator(move |val| v_clone(val));
        }

        let mut form = FormModal::new(&self.title)
            .with_field(field)
            .with_max_width(self.max_width)
            .with_confirm_on_cancel(false)
            .with_footer_help("[Enter] Submit  |  [Esc] Cancel");

        if let Some(ref b) = self.breadcrumbs {
            form = form.with_breadcrumbs(b);
        }
        if !self.show_breadcrumbs {
            form = form.without_breadcrumbs();
        }

        match form.run()? {
            Some(res) => Ok(InputOutcome::Submitted(res.get_string("input"))),
            None => Ok(InputOutcome::Cancelled),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::BoxFrame;
    use crate::keys::KeyHelpMode;
    use crate::theme::strip_ansi;

    #[test]
    fn test_input_modal_builder() {
        let modal = InputModal::new("INPUT PROMPT", "Enter value:")
            .with_default("default123")
            .with_placeholder("e.g. 25565")
            .with_password(true)
            .with_validator(|s| {
                if s.len() > 3 {
                    Ok(())
                } else {
                    Err("Too short".to_string())
                }
            });

        assert_eq!(modal.title, "INPUT PROMPT");
        assert_eq!(modal.prompt, "Enter value:");
        assert_eq!(modal.default_val.as_deref(), Some("default123"));
        assert_eq!(modal.placeholder.as_deref(), Some("e.g. 25565"));
        assert!(modal.is_password);
        assert!(modal.validator.is_some());
    }

    #[test]
    fn test_cursor_y_calculation() {
        let _g1 = crate::nav::NavGuard::enter("Dashboard");
        let _g2 = crate::nav::NavGuard::enter("Local Servers");
        let _g3 = crate::nav::NavGuard::enter("Create Server");

        let modal = InputModal::new("SERVER SETUP WIZARD (STEP 1/6)", "Enter server name:")
            .with_default("my-server");

        let width = 80;
        let mut frame = BoxFrame::new(width).with_title(&modal.title, false);
        for line in modal.prompt.lines() {
            frame.row(line);
        }
        frame.empty_row();
        frame.row(format!("> {}", "my-server"));
        frame.footer(modal.keymap.footer_help_text(KeyHelpMode::Input));

        let rendered = frame.render_to_string();

        let cursor_y = rendered.lines().enumerate().find_map(|(idx, line)| {
            let clean = strip_ansi(line);
            let inner = clean.trim_matches(|c| c == '│' || c == '|').trim_start();
            if inner.starts_with("> ") && idx >= 2 {
                Some(idx as u16)
            } else {
                None
            }
        });

        assert_eq!(cursor_y, Some(6));
    }
}
