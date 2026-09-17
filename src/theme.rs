use colored::Colorize;

pub const BORDER_COLOR: &str = "\x1B[38;5;240m";
pub const RESET: &str = "\x1B[0m";
pub const ACCENT: &str = "\x1B[1;36m";
pub const BOLD_WHITE: &str = "\x1B[1;37m";
pub const DIM: &str = "\x1B[2m";

/// Returns true if the environment supports UTF-8 characters.
pub fn is_utf8_supported() -> bool {
    std::env::var("LANG")
        .or_else(|_| std::env::var("LC_ALL"))
        .map(|v| v.to_ascii_lowercase().contains("utf"))
        .unwrap_or(true)
}

/// Strips ANSI escape sequences for accurate visual length calculation.
pub fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_escape = false;
    for c in s.chars() {
        if c == '\x1B' {
            in_escape = true;
        } else if in_escape {
            if c.is_ascii_alphabetic() {
                in_escape = false;
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Calculates the visible character length of a string ignoring ANSI escapes.
pub fn visible_len(s: &str) -> usize {
    strip_ansi(s).chars().count()
}

/// Generates a top border: ╭────────────────╮ or +----------------+
pub fn box_top(width: usize) -> String {
    let w = width.saturating_sub(2);
    if is_utf8_supported() {
        format!("{}╭{}╮{}", BORDER_COLOR, "─".repeat(w), RESET)
    } else {
        format!("{}+{}+{}", BORDER_COLOR, "-".repeat(w), RESET)
    }
}

/// Generates a bottom border: ╰────────────────╯ or +----------------+
pub fn box_bottom(width: usize) -> String {
    let w = width.saturating_sub(2);
    if is_utf8_supported() {
        format!("{}╰{}╯{}", BORDER_COLOR, "─".repeat(w), RESET)
    } else {
        format!("{}+{}+{}", BORDER_COLOR, "-".repeat(w), RESET)
    }
}

/// Generates an internal divider: ├────────────────┤ or +----------------+
pub fn box_divider(width: usize) -> String {
    let w = width.saturating_sub(2);
    if is_utf8_supported() {
        format!("{}├{}┤{}", BORDER_COLOR, "─".repeat(w), RESET)
    } else {
        format!("{}+{}+{}", BORDER_COLOR, "-".repeat(w), RESET)
    }
}

/// Generates a centered title row with side borders: │           TITLE            │
pub fn box_title(title: &str, width: usize, is_error: bool) -> String {
    let inner_width = width.saturating_sub(2);
    let border_char = if is_utf8_supported() { "│" } else { "|" };

    let title_len = visible_len(title);

    let (pad_left, pad_right) = if title_len < inner_width {
        let remaining = inner_width - title_len;
        (remaining / 2, remaining - remaining / 2)
    } else {
        (0, 0)
    };

    let title_styled = if is_error {
        title.red().bold().to_string()
    } else {
        title.cyan().bold().to_string()
    };

    format!(
        "{}{}{}{}{}{}{}{}{}",
        BORDER_COLOR,
        border_char,
        RESET,
        " ".repeat(pad_left),
        title_styled,
        " ".repeat(pad_right),
        BORDER_COLOR,
        border_char,
        RESET
    )
}

/// Generates a single centered title row (kept for backward compatibility).
pub fn box_title_simple(title: &str, width: usize, is_error: bool) -> String {
    box_title(title, width, is_error)
}

/// Formats a line inside a box with left and right vertical borders
pub fn box_line(content: &str, width: usize) -> String {
    let inner_width = width.saturating_sub(2);
    let border_char = if is_utf8_supported() { "│" } else { "|" };

    let len = visible_len(content);
    let padding = inner_width.saturating_sub(len + 1);

    format!(
        "{}{}{} {}{}{}{}{}",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_ansi() {
        let raw = "\x1B[1;36mHello World\x1B[0m";
        assert_eq!(strip_ansi(raw), "Hello World");
        assert_eq!(visible_len(raw), 11);
    }

    #[test]
    fn test_box_borders() {
        let top = box_top(20);
        assert!(strip_ansi(&top).contains("╭"));
        assert!(strip_ansi(&top).contains("╮"));

        let bottom = box_bottom(20);
        assert!(strip_ansi(&bottom).contains("╰"));
        assert!(strip_ansi(&bottom).contains("╯"));

        let divider = box_divider(20);
        assert!(strip_ansi(&divider).contains("├"));
        assert!(strip_ansi(&divider).contains("┤"));
    }
}
