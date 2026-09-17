//! High-level shortcut abstraction and dynamic footer bar builder for TUI frames and modals.

use std::fmt;

/// A single shortcut entry pairing key expression(s) with an action description.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shortcut {
    /// The formatted key text (e.g. "Enter/Esc", "↑/↓", "q", "Space", "Ctrl+C").
    pub keys: String,
    /// The action description (e.g. "Dismiss", "Select", "Back", "Move", "Toggle", "Exit").
    pub description: String,
}

impl Shortcut {
    /// Creates a new shortcut definition.
    pub fn new(keys: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            keys: keys.into(),
            description: description.into(),
        }
    }

    /// Convenience helper for Dismiss action (`[Enter/Esc] Dismiss`).
    pub fn dismiss() -> Self {
        Self::new("Enter/Esc", "Dismiss")
    }

    /// Convenience helper for Select action (`[Enter/→] Select`).
    pub fn select() -> Self {
        Self::new("Enter/→", "Select")
    }

    /// Convenience helper for Move action with vim keys (`[↑/↓/j/k] Move`).
    pub fn move_selection() -> Self {
        Self::new("↑/↓/j/k", "Move")
    }

    /// Convenience helper for Move action with arrows only (`[↑/↓] Move`).
    pub fn move_arrows() -> Self {
        Self::new("↑/↓", "Move")
    }

    /// Convenience helper for Back action (`[Esc/←] Back`).
    pub fn back() -> Self {
        Self::new("Esc/←", "Back")
    }

    /// Convenience helper for Cancel action (`[Esc] Cancel`).
    pub fn cancel() -> Self {
        Self::new("Esc", "Cancel")
    }

    /// Convenience helper for Confirm action (`[Enter] Confirm`).
    pub fn confirm() -> Self {
        Self::new("Enter", "Confirm")
    }

    /// Convenience helper for Save action (`[Enter] Save`).
    pub fn save() -> Self {
        Self::new("Enter", "Save")
    }

    /// Convenience helper for Toggle action (`[Space] Toggle`).
    pub fn toggle() -> Self {
        Self::new("Space", "Toggle")
    }

    /// Convenience helper for Exit action (`[q] Exit`).
    pub fn exit() -> Self {
        Self::new("q", "Exit")
    }

    /// Convenience helper for Scroll action (`[↑/↓/PgUp/PgDn] Scroll`).
    pub fn scroll() -> Self {
        Self::new("↑/↓/PgUp/PgDn", "Scroll")
    }

    /// Convenience helper for Page action (`[PgUp/PgDn] Page`).
    pub fn page() -> Self {
        Self::new("PgUp/PgDn", "Page")
    }

    /// Formats this shortcut into the bracketed label `[<keys>] <description>`.
    pub fn format(&self) -> String {
        format!("[{}] {}", self.keys, self.description)
    }
}

impl fmt::Display for Shortcut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.keys, self.description)
    }
}

/// A high-level collection of shortcuts rendered in the bottom footer of a modal or screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shortcuts {
    items: Vec<Shortcut>,
    separator: String,
}

impl Default for Shortcuts {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            separator: "  |  ".to_string(),
        }
    }
}

impl Shortcuts {
    /// Creates a new empty shortcuts collection with standard separator `"  |  "`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an empty shortcuts collection.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Sets custom separator between shortcuts (defaults to `"  |  "`).
    pub fn with_separator(mut self, separator: impl Into<String>) -> Self {
        self.separator = separator.into();
        self
    }

    /// Adds a shortcut with given key label and description.
    pub fn add(mut self, keys: impl Into<String>, description: impl Into<String>) -> Self {
        self.items.push(Shortcut::new(keys, description));
        self
    }

    /// Mutably pushes a shortcut with given key label and description.
    pub fn push(&mut self, keys: impl Into<String>, description: impl Into<String>) -> &mut Self {
        self.items.push(Shortcut::new(keys, description));
        self
    }

    /// Adds a preconstructed `Shortcut`.
    pub fn add_shortcut(mut self, shortcut: Shortcut) -> Self {
        self.items.push(shortcut);
        self
    }

    /// Mutably pushes a preconstructed `Shortcut`.
    pub fn push_shortcut(&mut self, shortcut: Shortcut) -> &mut Self {
        self.items.push(shortcut);
        self
    }

    /// Conditionally adds a shortcut if `condition` is true.
    pub fn add_if(
        mut self,
        condition: bool,
        keys: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        if condition {
            self.items.push(Shortcut::new(keys, description));
        }
        self
    }

    /// Semantic helper: adds Move selection (`[↑/↓/j/k] Move`).
    pub fn move_selection(self) -> Self {
        self.add_shortcut(Shortcut::move_selection())
    }

    /// Semantic helper: adds Move arrows (`[↑/↓] Move`).
    pub fn move_arrows(self) -> Self {
        self.add_shortcut(Shortcut::move_arrows())
    }

    /// Semantic helper: adds Select (`[Enter/→] Select`).
    pub fn select(self) -> Self {
        self.add_shortcut(Shortcut::select())
    }

    /// Semantic helper: adds Back (`[Esc/←] Back`).
    pub fn back(self) -> Self {
        self.add_shortcut(Shortcut::back())
    }

    /// Semantic helper: adds Cancel (`[Esc] Cancel`).
    pub fn cancel(self) -> Self {
        self.add_shortcut(Shortcut::cancel())
    }

    /// Semantic helper: adds Confirm (`[Enter] Confirm`).
    pub fn confirm(self) -> Self {
        self.add_shortcut(Shortcut::confirm())
    }

    /// Semantic helper: adds Dismiss (`[Enter/Esc] Dismiss`).
    pub fn dismiss(self) -> Self {
        self.add_shortcut(Shortcut::dismiss())
    }

    /// Semantic helper: adds Save (`[Enter] Save`).
    pub fn save(self) -> Self {
        self.add_shortcut(Shortcut::save())
    }

    /// Semantic helper: adds Toggle (`[Space] Toggle`).
    pub fn toggle(self) -> Self {
        self.add_shortcut(Shortcut::toggle())
    }

    /// Semantic helper: adds Exit (`[q] Exit`).
    pub fn exit(self) -> Self {
        self.add_shortcut(Shortcut::exit())
    }

    /// Semantic helper: adds Exit (`[q] Exit`) conditionally.
    pub fn exit_if(self, condition: bool) -> Self {
        if condition {
            self.exit()
        } else {
            self
        }
    }

    /// Semantic helper: adds Scroll (`[↑/↓/PgUp/PgDn] Scroll`).
    pub fn scroll(self) -> Self {
        self.add_shortcut(Shortcut::scroll())
    }

    /// Semantic helper: adds Scroll (`[↑/↓/PgUp/PgDn] Scroll`) conditionally.
    pub fn scroll_if(self, condition: bool) -> Self {
        if condition {
            self.scroll()
        } else {
            self
        }
    }

    /// Semantic helper: adds Page (`[PgUp/PgDn] Page`).
    pub fn page(self) -> Self {
        self.add_shortcut(Shortcut::page())
    }

    /// Semantic helper: adds Page (`[PgUp/PgDn] Page`) conditionally.
    pub fn page_if(self, condition: bool) -> Self {
        if condition {
            self.page()
        } else {
            self
        }
    }

    /// Returns the list of shortcuts.
    pub fn items(&self) -> &[Shortcut] {
        &self.items
    }

    /// Returns individual formatted bracketed labels (e.g. `["[Enter/Esc] Dismiss", "[q] Exit"]`).
    pub fn to_button_items(&self) -> Vec<String> {
        self.items.iter().map(|s| s.format()).collect()
    }

    /// Returns the combined footer string joined by separator (e.g. `[Enter/Esc] Dismiss  |  [q] Exit`).
    pub fn to_footer_string(&self) -> String {
        self.to_button_items().join(&self.separator)
    }

    /// Returns true if no shortcuts are defined.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns count of shortcuts defined.
    pub fn len(&self) -> usize {
        self.items.len()
    }
}

impl fmt::Display for Shortcuts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_footer_string())
    }
}

impl From<Vec<Shortcut>> for Shortcuts {
    fn from(items: Vec<Shortcut>) -> Self {
        Self {
            items,
            separator: "  |  ".to_string(),
        }
    }
}

impl FromIterator<Shortcut> for Shortcuts {
    fn from_iter<I: IntoIterator<Item = Shortcut>>(iter: I) -> Self {
        Self {
            items: iter.into_iter().collect(),
            separator: "  |  ".to_string(),
        }
    }
}

impl From<&str> for Shortcuts {
    fn from(s: &str) -> Self {
        let mut sc = Shortcuts::new();
        for part in s.split('|') {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Some(start) = trimmed.find('[') {
                if let Some(end) = trimmed.find(']') {
                    let keys = &trimmed[start + 1..end];
                    let desc = trimmed[end + 1..].trim();
                    sc.items.push(Shortcut::new(keys, desc));
                    continue;
                }
            }
            sc.items.push(Shortcut::new("", trimmed));
        }
        sc
    }
}

impl From<String> for Shortcuts {
    fn from(s: String) -> Self {
        Shortcuts::from(s.as_str())
    }
}

/// Type alias for `Shortcuts`.
pub type ShortcutBar = Shortcuts;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcut_format() {
        let s = Shortcut::new("Enter/Esc", "Dismiss");
        assert_eq!(s.format(), "[Enter/Esc] Dismiss");
        assert_eq!(s.to_string(), "[Enter/Esc] Dismiss");
    }

    #[test]
    fn test_shortcuts_builder_and_display() {
        let sc = Shortcuts::new().dismiss().exit();
        assert_eq!(sc.len(), 2);
        assert_eq!(sc.to_footer_string(), "[Enter/Esc] Dismiss  |  [q] Exit");
        assert_eq!(sc.to_string(), "[Enter/Esc] Dismiss  |  [q] Exit");
        assert_eq!(
            sc.to_button_items(),
            vec!["[Enter/Esc] Dismiss", "[q] Exit"]
        );
    }

    #[test]
    fn test_conditional_shortcuts() {
        let sc_no_scroll = Shortcuts::new().scroll_if(false).dismiss().exit_if(true);
        assert_eq!(
            sc_no_scroll.to_footer_string(),
            "[Enter/Esc] Dismiss  |  [q] Exit"
        );

        let sc_with_scroll = Shortcuts::new().scroll_if(true).dismiss().exit_if(false);
        assert_eq!(
            sc_with_scroll.to_footer_string(),
            "[↑/↓/PgUp/PgDn] Scroll  |  [Enter/Esc] Dismiss"
        );
    }

    #[test]
    fn test_from_str_parsing() {
        let raw = "[↑/↓] Move  |  [Enter] Select  |  [Esc] Back";
        let sc = Shortcuts::from(raw);
        assert_eq!(sc.len(), 3);
        assert_eq!(sc.items()[0].keys, "↑/↓");
        assert_eq!(sc.items()[0].description, "Move");
        assert_eq!(sc.items()[1].keys, "Enter");
        assert_eq!(sc.items()[1].description, "Select");
        assert_eq!(sc.items()[2].keys, "Esc");
        assert_eq!(sc.items()[2].description, "Back");
    }
}
