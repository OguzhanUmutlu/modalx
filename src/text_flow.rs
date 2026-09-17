use super::theme::{strip_ansi, visible_len};

/// Truncates a string to at most `max_chars` characters, safely handling UTF-8 boundaries.
pub fn truncate_str(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        None => s,
        Some((idx, _)) => &s[..idx],
    }
}

/// Truncates a string to at most `max_chars` characters, appending a gray ellipsis ("...") if truncated.
pub fn truncate_ellipsis(s: &str, max_chars: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_chars {
        s.to_string()
    } else {
        let keep_chars = max_chars.saturating_sub(3);
        let truncated = truncate_str(s, keep_chars);
        format!("{}\x1B[90m...\x1B[0m", truncated)
    }
}

/// Truncates a string that may contain ANSI escape sequences to at most `max_visible` visible characters,
/// preserving active ANSI formatting and appending an ellipsis in gray (`\x1B[90m...\x1B[0m`).
pub fn truncate_ansi(content: &str, max_visible: usize) -> String {
    let clean = strip_ansi(content);
    if clean.chars().count() <= max_visible {
        return content.to_string();
    }

    let keep_visible = max_visible.saturating_sub(3);
    let mut result = String::new();
    let mut visible_count = 0;
    let mut in_escape = false;

    for ch in content.chars() {
        if ch == '\x1B' {
            in_escape = true;
            result.push(ch);
            continue;
        }

        if in_escape {
            result.push(ch);
            if ch == 'm' || ch == 'H' || ch == 'J' || ch == 'K' {
                in_escape = false;
            }
            continue;
        }

        if visible_count < keep_visible {
            result.push(ch);
            visible_count += 1;
        } else {
            break;
        }
    }

    result.push_str("\x1B[90m...\x1B[0m");
    result
}

/// Wraps a slice of button/action texts into multiple framed lines, separated by `separator`,
/// and optionally horizontally centering each line within `available_width`.
pub fn wrap_button_items<T: AsRef<str>>(
    items: &[T],
    separator: &str,
    available_width: usize,
    center: bool,
) -> Vec<String> {
    if items.is_empty() {
        return Vec::new();
    }
    if available_width == 0 {
        return vec![items
            .iter()
            .map(|i| i.as_ref())
            .collect::<Vec<_>>()
            .join(separator)];
    }

    let sep_formatted = if separator.trim() == "|" {
        "  |  "
    } else {
        separator
    };
    let sep_len = visible_len(sep_formatted);

    let mut raw_lines: Vec<String> = Vec::new();
    let mut current_line = String::new();
    let mut current_len = 0usize;

    for item in items {
        let text = item.as_ref().trim();
        if text.is_empty() {
            continue;
        }
        let t_len = visible_len(text);

        if current_line.is_empty() {
            current_line.push_str(text);
            current_len = t_len;
        } else if current_len + sep_len + t_len <= available_width {
            current_line.push_str(sep_formatted);
            current_line.push_str(text);
            current_len += sep_len + t_len;
        } else {
            raw_lines.push(current_line);
            current_line = text.to_string();
            current_len = t_len;
        }
    }

    if !current_line.is_empty() {
        raw_lines.push(current_line);
    }

    if !center {
        return raw_lines;
    }

    raw_lines
        .into_iter()
        .map(|line| {
            let line_len = visible_len(&line);
            if line_len < available_width {
                let total_pad = available_width - line_len;
                let pad_left = total_pad / 2;
                let pad_right = total_pad - pad_left;
                format!("{}{}{}", " ".repeat(pad_left), line, " ".repeat(pad_right))
            } else {
                line
            }
        })
        .collect()
}

/// Formats and wraps a slice of discrete field items into one or more lines,
/// separating them with `delimiter` while ensuring no line exceeds `available_width`.
///
/// If a single item cannot fit on the remaining line space, it is moved to a new line.
/// If a single item by itself exceeds `available_width`, it is placed on its own line.
///
/// # Example
/// ```rust
/// use modalx::text_flow::wrap_fields;
///
/// let fields = vec![
///     "Host: Ubuntu".to_string(),
///     "RAM: 4.7 / 22.7 GB (20.5%)".to_string(),
///     "Daemon: [ONLINE]".to_string(),
/// ];
/// let lines = wrap_fields(&fields, " | ", 50);
/// assert_eq!(lines.len(), 2);
/// ```
pub fn wrap_fields(fields: &[String], delimiter: &str, available_width: usize) -> Vec<String> {
    if fields.is_empty() {
        return Vec::new();
    }
    if available_width == 0 {
        return vec![fields.join(delimiter)];
    }

    let delim_len = visible_len(delimiter);
    let mut lines: Vec<String> = Vec::new();
    let mut current_line = String::new();
    let mut current_len = 0usize;

    for field in fields {
        let flen = visible_len(field);

        if current_line.is_empty() {
            current_line.push_str(field);
            current_len = flen;
        } else {
            let needed = current_len + delim_len + flen;
            if needed <= available_width {
                current_line.push_str(delimiter);
                current_line.push_str(field);
                current_len = needed;
            } else {
                lines.push(current_line);
                current_line = field.clone();
                current_len = flen;
            }
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

/// Splits an existing delimited string (e.g. `"Host: Ubuntu | RAM: 16 GB | Status: OK"`)
/// by `delimiter`, trims whitespace around each component, and wraps them into lines
/// that fit within `available_width`.
pub fn wrap_delimited_string(
    content: &str,
    delimiter: &str,
    available_width: usize,
) -> Vec<String> {
    if content.is_empty() {
        return Vec::new();
    }
    if !content.contains(delimiter) || available_width == 0 {
        return vec![content.to_string()];
    }

    let fields: Vec<String> = content
        .split(delimiter)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if fields.is_empty() {
        return vec![content.to_string()];
    }

    wrap_fields(&fields, delimiter, available_width)
}

/// Wraps arbitrary prose into multiple lines fitting within `max_width`,
/// preserving ANSI escape sequences across word boundaries.
pub fn wrap_words(text: &str, max_width: usize) -> Vec<String> {
    if text.is_empty() {
        return vec![String::new()];
    }
    if max_width == 0 {
        return vec![text.to_string()];
    }

    let mut lines: Vec<String> = Vec::new();
    for raw_line in text.lines() {
        if raw_line.is_empty() {
            lines.push(String::new());
            continue;
        }

        let words: Vec<&str> = raw_line.split_whitespace().collect();
        let mut current_line = String::new();
        let mut current_len = 0usize;

        for word in words {
            let w_len = visible_len(word);
            if current_line.is_empty() {
                current_line.push_str(word);
                current_len = w_len;
            } else if current_len + 1 + w_len <= max_width {
                current_line.push(' ');
                current_line.push_str(word);
                current_len += 1 + w_len;
            } else {
                lines.push(current_line);
                current_line = word.to_string();
                current_len = w_len;
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate() {
        assert_eq!(truncate_str("hello world", 5), "hello");
        assert_eq!(strip_ansi(&truncate_ellipsis("hello world", 8)), "hello...");
        assert_eq!(truncate_ellipsis("hi", 8), "hi");
    }

    #[test]
    fn test_truncate_ansi_preserves_colors_and_grays_ellipsis() {
        let content = "\x1B[36mhello\x1B[0m world long text";
        let truncated = truncate_ansi(content, 10);
        assert_eq!(strip_ansi(&truncated), "hello w...");
        assert!(truncated.contains("\x1B[90m...\x1B[0m"));
    }

    #[test]
    fn test_wrap_button_items_wrapping_and_centering() {
        let buttons = vec![
            "[↑/↓/j/k] Move",
            "[Enter/→] Connect",
            "[e] Edit Offline",
            "[Esc/←] Back",
        ];

        // Wide terminal: all on 1 centered line
        let wide = wrap_button_items(&buttons, "|", 80, true);
        assert_eq!(wide.len(), 1);
        assert!(wide[0].contains("[↑/↓/j/k] Move"));
        assert!(wide[0].contains("[Esc/←] Back"));
        assert_eq!(visible_len(&wide[0]), 80);

        // Narrow terminal: wraps into 2 centered lines
        let narrow = wrap_button_items(&buttons, "|", 45, true);
        assert_eq!(narrow.len(), 2);
        assert_eq!(visible_len(&narrow[0]), 45);
        assert_eq!(visible_len(&narrow[1]), 45);
    }

    #[test]
    fn test_wrap_fields_single_line() {
        let fields = vec![
            "Host: Ubuntu".to_string(),
            "RAM: 4.7 GB".to_string(),
            "Daemon: [ONLINE]".to_string(),
        ];
        let lines = wrap_fields(&fields, " | ", 80);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "Host: Ubuntu | RAM: 4.7 GB | Daemon: [ONLINE]");
    }

    #[test]
    fn test_wrap_fields_multiline() {
        let fields = vec![
            "Host: Ubuntu".to_string(),
            "RAM: 4.7 / 22.7 GB (20.5%)".to_string(),
            "Daemon: [ONLINE]".to_string(),
        ];
        let lines = wrap_fields(&fields, " | ", 45);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "Host: Ubuntu | RAM: 4.7 / 22.7 GB (20.5%)");
        assert_eq!(lines[1], "Daemon: [ONLINE]");
    }

    #[test]
    fn test_wrap_words() {
        let text = "The quick brown fox jumps over the lazy dog";
        let lines = wrap_words(text, 15);
        assert!(lines.len() > 1);
        for line in &lines {
            assert!(visible_len(line) <= 15);
        }
    }
}
