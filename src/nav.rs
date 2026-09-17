use super::theme::{is_utf8_supported, strip_ansi, BORDER_COLOR, RESET};
use colored::Colorize;
use std::cell::RefCell;

thread_local! {
    static NAV_STACK: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    static ROOT_CRUMBS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// Configures root prefix breadcrumbs (e.g. for remote node or context presentation mode).
pub fn set_root_breadcrumbs(crumbs: &[&str]) {
    ROOT_CRUMBS.with(|rc| {
        let mut r = rc.borrow_mut();
        r.clear();
        for c in crumbs {
            r.push(c.to_string());
        }
    });
}

/// Clears root prefix breadcrumbs.
pub fn clear_root_breadcrumbs() {
    ROOT_CRUMBS.with(|rc| {
        rc.borrow_mut().clear();
    });
}

/// RAII Guard that manages navigation breadcrumbs on screen headers.
pub struct NavGuard;

impl NavGuard {
    pub fn enter(title: impl Into<String>) -> Self {
        NAV_STACK.with(|stack| {
            stack.borrow_mut().push(title.into());
        });
        NavGuard
    }

    pub fn depth() -> usize {
        NAV_STACK.with(|stack| stack.borrow().len())
    }

    pub fn current() -> Option<String> {
        let crumbs = get_breadcrumbs();
        crumbs.last().cloned()
    }

    pub fn is_root() -> bool {
        Self::depth() <= 1
    }

    pub fn crumbs() -> Vec<String> {
        get_breadcrumbs()
    }

    pub fn has_crumbs() -> bool {
        get_breadcrumbs().len() > 1
    }

    pub fn format(max_width: usize) -> Option<String> {
        let crumbs = get_breadcrumbs();
        if crumbs.len() <= 1 {
            None
        } else {
            Some(format_breadcrumbs(&crumbs, max_width))
        }
    }
}

impl Drop for NavGuard {
    fn drop(&mut self) {
        NAV_STACK.with(|stack| {
            stack.borrow_mut().pop();
        });
    }
}

/// Returns a copy of the current navigation path.
pub fn get_breadcrumbs() -> Vec<String> {
    let mut res = ROOT_CRUMBS.with(|rc| rc.borrow().clone());
    NAV_STACK.with(|stack| {
        res.extend(stack.borrow().iter().cloned());
    });
    res
}

/// Formats breadcrumb items into a single line, squeezing intermediate items if needed.
pub fn format_breadcrumbs(crumbs: &[String], max_width: usize) -> String {
    if crumbs.is_empty() {
        return String::new();
    }

    let sep = if is_utf8_supported() { " › " } else { " > " };

    // Full path
    let full = crumbs.join(sep);
    if full.chars().count() <= max_width {
        return full;
    }

    // Try keeping first, ellipsis, and last two
    if crumbs.len() >= 4 {
        let candidate = format!(
            "{}{}{}{}{}{}{}",
            crumbs[0],
            sep,
            "...",
            sep,
            crumbs[crumbs.len() - 2],
            sep,
            crumbs[crumbs.len() - 1]
        );
        if candidate.chars().count() <= max_width {
            return candidate;
        }
    }

    // Try keeping first, ellipsis, and last one
    if crumbs.len() >= 3 {
        let candidate = format!(
            "{}{}{}{}{}",
            crumbs[0],
            sep,
            "...",
            sep,
            crumbs[crumbs.len() - 1]
        );
        if candidate.chars().count() <= max_width {
            return candidate;
        }
    }

    // Try keeping ellipsis and last one
    let last = crumbs.last().map(|s| s.as_str()).unwrap_or("");
    let last_candidate = format!("...{}{}", sep, last);
    if last_candidate.chars().count() <= max_width {
        return last_candidate;
    }

    // Truncate the last item if even that doesn't fit
    if max_width > 3 {
        let truncated: String = last.chars().take(max_width.saturating_sub(3)).collect();
        format!("{}...", truncated)
    } else {
        last.chars().take(max_width).collect()
    }
}

/// Generates a bordered breadcrumb line if more than 1 navigation level exists.
pub fn box_breadcrumbs(width: usize) -> Option<String> {
    let crumbs = get_breadcrumbs();
    if crumbs.len() <= 1 {
        return None;
    }

    let inner_width = width.saturating_sub(2);
    let raw_formatted = format_breadcrumbs(&crumbs, inner_width.saturating_sub(4));
    let clean = strip_ansi(&raw_formatted);
    let len = clean.chars().count();

    let (pad_left, pad_right) = if len < inner_width {
        let rem = inner_width - len;
        (rem / 2, rem - rem / 2)
    } else {
        (0, 0)
    };

    let border_char = if is_utf8_supported() { "│" } else { "|" };
    let styled_text = raw_formatted.dimmed().to_string();

    Some(format!(
        "{}{}{}{}{}{}{}{}{}",
        BORDER_COLOR,
        border_char,
        RESET,
        " ".repeat(pad_left),
        styled_text,
        " ".repeat(pad_right),
        BORDER_COLOR,
        border_char,
        RESET,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nav_guard_stack() {
        {
            let _n1 = NavGuard::enter("Dashboard");
            assert_eq!(get_breadcrumbs(), vec!["Dashboard"]);
            {
                let _n2 = NavGuard::enter("Local Servers");
                assert_eq!(get_breadcrumbs(), vec!["Dashboard", "Local Servers"]);
                {
                    let _n3 = NavGuard::enter("my-server");
                    assert_eq!(
                        get_breadcrumbs(),
                        vec!["Dashboard", "Local Servers", "my-server"]
                    );
                }
                assert_eq!(get_breadcrumbs(), vec!["Dashboard", "Local Servers"]);
            }
            assert_eq!(get_breadcrumbs(), vec!["Dashboard"]);
        }
        assert!(get_breadcrumbs().is_empty());
    }

    #[test]
    fn test_format_breadcrumbs_squeezing() {
        let crumbs = vec![
            "Dashboard".to_string(),
            "Backup Systems".to_string(),
            "Local Storage".to_string(),
            "Default Storage".to_string(),
        ];

        let full = format_breadcrumbs(&crumbs, 100);
        assert!(full.contains("Backup Systems"));
        assert!(full.contains("Default Storage"));

        let squeezed = format_breadcrumbs(&crumbs, 45);
        assert!(squeezed.contains("..."));
        assert!(squeezed.contains("Dashboard"));
        assert!(squeezed.contains("Default Storage"));
        assert!(squeezed.chars().count() <= 45);

        let narrow = format_breadcrumbs(&crumbs, 20);
        assert!(narrow.chars().count() <= 20);
    }
}
