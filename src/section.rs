use super::keys::{KeyHelpMode, KeyMap};
use super::text_flow::{wrap_fields, wrap_words};

/// Represents an item in a menu or selectable list with a hotkey, label, and optional aliases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectItem {
    pub hotkey: String,
    pub label: String,
    pub description: Option<String>,
    pub aliases: Vec<String>,
}

impl SelectItem {
    pub fn new(hotkey: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            hotkey: hotkey.into(),
            label: label.into(),
            description: None,
            aliases: Vec::new(),
        }
    }

    /// Creates a selectable menu button without a hotkey/shortcut.
    pub fn button(label: impl Into<String>) -> Self {
        Self {
            hotkey: String::new(),
            label: label.into(),
            description: None,
            aliases: Vec::new(),
        }
    }

    /// Creates a selectable menu item with an optional hotkey.
    pub fn with_optional_hotkey(hotkey: Option<impl Into<String>>, label: impl Into<String>) -> Self {
        Self {
            hotkey: hotkey.map(Into::into).unwrap_or_default(),
            label: label.into(),
            description: None,
            aliases: Vec::new(),
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_aliases(mut self, aliases: &[&str]) -> Self {
        self.aliases = aliases
            .iter()
            .map(|s| s.to_string())
            .filter(|s| {
                if s.len() == 1
                    && s.chars()
                        .next()
                        .map(|c| c.is_ascii_digit())
                        .unwrap_or(false)
                    && s != &self.hotkey
                {
                    return false;
                }
                if s.eq_ignore_ascii_case("q") && !self.hotkey.eq_ignore_ascii_case("q") {
                    return false;
                }
                true
            })
            .collect();
        self
    }
}

/// Backward compatibility alias.
pub type MenuEntry = SelectItem;

/// Represents a distinct visual or interactive section in a modal.
#[derive(Debug, Clone)]
pub enum ModalSection {
    /// Title header section with centered title and optional navigation breadcrumbs
    Title(TitleSection),
    /// Responsive field group with delimiter and smart multi-row wrapping
    Fields(FieldSection),
    /// Word-wrapped descriptive paragraph or text lines
    Text(TextSection),
    /// Selectable menu or button list
    Menu(MenuSection),
    /// Custom pre-formatted lines inside the frame
    Custom(Vec<String>),
    /// Horizontal box divider line (`├──────┤`)
    Divider { dimmed: bool },
    /// Bottom footer shortcut help bar
    Footer(Box<FooterSection>),
}

/// Header title section with navigation breadcrumbs.
#[derive(Debug, Clone)]
pub struct TitleSection {
    pub title: String,
    pub breadcrumbs: Option<String>,
    pub show_breadcrumbs: bool,
    pub is_error: bool,
}

impl TitleSection {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            breadcrumbs: None,
            show_breadcrumbs: true,
            is_error: false,
        }
    }

    pub fn with_breadcrumbs(mut self, breadcrumbs: impl Into<String>) -> Self {
        self.breadcrumbs = Some(breadcrumbs.into());
        self.show_breadcrumbs = true;
        self
    }

    pub fn without_breadcrumbs(mut self) -> Self {
        self.show_breadcrumbs = false;
        self
    }

    pub fn with_error(mut self, is_error: bool) -> Self {
        self.is_error = is_error;
        self
    }
}

/// Metadata field container that automatically wraps items across multiple rows
/// when the available terminal width is constrained.
#[derive(Debug, Clone)]
pub struct FieldSection {
    pub items: Vec<String>,
    pub separator: String,
}

impl Default for FieldSection {
    fn default() -> Self {
        Self::new()
    }
}

impl FieldSection {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            separator: " | ".to_string(),
        }
    }

    pub fn with_items(items: Vec<String>, separator: impl Into<String>) -> Self {
        Self {
            items,
            separator: separator.into(),
        }
    }

    /// Creates a field section from a raw delimited single line (e.g. `"Host: Ubuntu | RAM: 8GB"`).
    pub fn from_raw(raw: impl Into<String>, separator: impl Into<String>) -> Self {
        let sep = separator.into();
        let r = raw.into();
        let parts: Vec<String> = r
            .split(&sep)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        Self {
            items: parts,
            separator: sep,
        }
    }

    /// Sets the separator string between fields (default: `" | "`).
    pub fn with_separator(mut self, separator: impl Into<String>) -> Self {
        self.separator = separator.into();
        self
    }

    /// Appends an individual field item (fluent builder).
    pub fn add_field(mut self, item: impl Into<String>) -> Self {
        self.items.push(item.into());
        self
    }

    /// Appends an individual field item (mutable reference).
    pub fn add_item(&mut self, item: impl Into<String>) -> &mut Self {
        self.items.push(item.into());
        self
    }

    /// Renders field items into wrapped rows bounded by `available_width`.
    pub fn render(&self, available_width: usize) -> Vec<String> {
        wrap_fields(&self.items, &self.separator, available_width)
    }
}

/// Text section for paragraph or descriptive content.
#[derive(Debug, Clone)]
pub struct TextSection {
    pub text: String,
}

impl TextSection {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    pub fn render(&self, available_width: usize) -> Vec<String> {
        wrap_words(&self.text, available_width)
    }
}

/// Menu list section.
#[derive(Debug, Clone)]
pub struct MenuSection {
    pub entries: Vec<SelectItem>,
    pub allow_toggle: bool,
}

impl MenuSection {
    pub fn new(entries: Vec<SelectItem>) -> Self {
        Self {
            entries,
            allow_toggle: false,
        }
    }

    pub fn with_allow_toggle(mut self, allow: bool) -> Self {
        self.allow_toggle = allow;
        self
    }
}

/// Footer shortcut help section.
#[derive(Debug, Clone)]
pub struct FooterSection {
    pub keymap: KeyMap,
    pub mode: KeyHelpMode,
    pub custom_help: Option<String>,
}

impl FooterSection {
    pub fn new(mode: KeyHelpMode) -> Self {
        Self {
            keymap: KeyMap::default(),
            mode,
            custom_help: None,
        }
    }

    pub fn with_keymap(mut self, keymap: KeyMap) -> Self {
        self.keymap = keymap;
        self
    }

    pub fn with_custom_help(mut self, help: impl Into<String>) -> Self {
        self.custom_help = Some(help.into());
        self
    }

    pub fn render(&self) -> String {
        if let Some(ref custom) = self.custom_help {
            custom.clone()
        } else {
            self.keymap.footer_help_text(self.mode)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_section_from_raw() {
        let sec = FieldSection::from_raw("Host: Ubuntu | RAM: 16GB | Daemon: [ONLINE]", " | ");
        assert_eq!(sec.items.len(), 3);
        assert_eq!(sec.items[0], "Host: Ubuntu");
        assert_eq!(sec.items[1], "RAM: 16GB");
        assert_eq!(sec.items[2], "Daemon: [ONLINE]");

        let rendered = sec.render(80);
        assert_eq!(rendered.len(), 1);
        assert_eq!(rendered[0], "Host: Ubuntu | RAM: 16GB | Daemon: [ONLINE]");

        let narrow_rendered = sec.render(20);
        assert_eq!(narrow_rendered.len(), 3);
        assert_eq!(narrow_rendered[0], "Host: Ubuntu");
        assert_eq!(narrow_rendered[1], "RAM: 16GB");
        assert_eq!(narrow_rendered[2], "Daemon: [ONLINE]");
    }

    #[test]
    fn test_title_section_builder() {
        let title = TitleSection::new("DASHBOARD")
            .with_breadcrumbs("Dashboard › Servers")
            .with_error(false);
        assert_eq!(title.title, "DASHBOARD");
        assert_eq!(title.breadcrumbs.as_deref(), Some("Dashboard › Servers"));
        assert!(!title.is_error);
    }
}
