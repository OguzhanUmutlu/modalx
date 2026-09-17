use crossterm::{
    cursor::{Hide, Show},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use std::io::{self, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Once;

use crate::error::Result;

pub const MIN_TERM_WIDTH: u16 = 60;
pub const MIN_TERM_HEIGHT: u16 = 14;

static ALT_SCREEN_DEPTH: AtomicUsize = AtomicUsize::new(0);

/// Terminal sizing constraints for modalx operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalConstraints {
    pub min_width: u16,
    pub min_height: u16,
}

impl Default for TerminalConstraints {
    fn default() -> Self {
        Self {
            min_width: MIN_TERM_WIDTH,
            min_height: MIN_TERM_HEIGHT,
        }
    }
}

/// Returns current terminal dimensions (width, height), defaulting to (80, 24) on error.
pub fn get_terminal_size() -> (u16, u16) {
    size().unwrap_or((80, 24))
}

/// Returns true if the terminal window is smaller than the default minimum dimensions.
pub fn is_terminal_too_small() -> bool {
    let (w, h) = get_terminal_size();
    w < MIN_TERM_WIDTH || h < MIN_TERM_HEIGHT
}

/// Cleanly resets the terminal out of alternate screen and raw mode, then exits the process.
pub fn clean_exit() -> ! {
    restore_terminal();
    std::process::exit(0);
}

/// Restores normal terminal state: disables raw mode, leaves alternate screen, and unhides the cursor.
pub fn restore_terminal() {
    ALT_SCREEN_DEPTH.store(0, Ordering::SeqCst);
    let _ = disable_raw_mode();
    let _ = execute!(io::stdout(), LeaveAlternateScreen, Show);
    let _ = io::stdout().flush();
}

/// Re-entrant RAII guard for the terminal alternate screen.
/// Ensures nested submenus and dialogs do not exit alternate screen prematurely.
#[derive(Debug)]
pub struct AltScreenGuard;

/// Returns true if an alternate screen guard is currently active.
pub fn is_alt_screen_active() -> bool {
    ALT_SCREEN_DEPTH.load(Ordering::SeqCst) > 0
}

/// Re-asserts the alternate screen buffer if an active AltScreenGuard exists.
/// This should be called after returning from external processes, child PTY sessions,
/// or SSH connections that may have emitted LeaveAlternateScreen or corrupted terminal modes.
pub fn restore_alt_screen_if_active() {
    if is_alt_screen_active() {
        let _ = execute!(
            io::stdout(),
            EnterAlternateScreen,
            Hide,
            Clear(ClearType::All),
            crossterm::cursor::MoveTo(0, 0)
        );
        let _ = io::stdout().flush();
    }
}

impl AltScreenGuard {
    pub fn enter() -> Self {
        init_terminal_panic_hook();
        if ALT_SCREEN_DEPTH.fetch_add(1, Ordering::SeqCst) == 0 {
            let _ = execute!(io::stdout(), EnterAlternateScreen, Hide);
        }
        AltScreenGuard
    }
}

impl Drop for AltScreenGuard {
    fn drop(&mut self) {
        if ALT_SCREEN_DEPTH.fetch_sub(1, Ordering::SeqCst) == 1 {
            let _ = disable_raw_mode();
            let _ = execute!(io::stdout(), LeaveAlternateScreen, Show);
        }
    }
}

/// Combined RAII guard that manages both alternate screen and raw mode.
#[derive(Debug)]
pub struct TerminalGuard {
    _alt: AltScreenGuard,
}

impl TerminalGuard {
    pub fn enter() -> Result<Self> {
        let guard = AltScreenGuard::enter();
        enable_raw_mode()?;
        Ok(Self { _alt: guard })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}

/// If the terminal window is smaller than `constraints`, enters a blocking event loop
/// rendering the centered "TERMINAL TOO SMALL" screen until the user resizes the terminal
/// to valid dimensions or presses 'q'/Ctrl+C to quit.
pub fn wait_for_valid_size(stdout: &mut io::Stdout) -> Result<(u16, u16)> {
    wait_for_constraints(stdout, TerminalConstraints::default())
}

/// Enforces specific terminal size constraints, rendering a warning until valid.
pub fn wait_for_constraints(
    stdout: &mut io::Stdout,
    constraints: TerminalConstraints,
) -> Result<(u16, u16)> {
    let (mut w, mut h) = get_terminal_size();
    if w >= constraints.min_width && h >= constraints.min_height {
        return Ok((w, h));
    }

    loop {
        execute!(stdout, crossterm::cursor::MoveTo(0, 0))?;
        let output =
            crate::frame::render_too_small(w, h, constraints.min_width, constraints.min_height);
        for line in output.lines() {
            print!("{}\x1B[K\r\n", line);
        }
        execute!(stdout, Clear(ClearType::FromCursorDown))?;
        stdout.flush()?;

        match crossterm::event::read()? {
            crossterm::event::Event::Resize(new_w, new_h) => {
                w = new_w;
                h = new_h;
                if w >= constraints.min_width && h >= constraints.min_height {
                    return Ok((w, h));
                }
            }
            crossterm::event::Event::Key(key) => {
                if key.kind == crossterm::event::KeyEventKind::Press
                    && ((key
                        .modifiers
                        .contains(crossterm::event::KeyModifiers::CONTROL)
                        && (key.code == crossterm::event::KeyCode::Char('c')
                            || key.code == crossterm::event::KeyCode::Char('C')))
                        || key.code == crossterm::event::KeyCode::Char('\x03')
                        || key.code == crossterm::event::KeyCode::Char('q')
                        || key.code == crossterm::event::KeyCode::Char('Q'))
                {
                    clean_exit();
                }
                let (cur_w, cur_h) = get_terminal_size();
                w = cur_w;
                h = cur_h;
                if w >= constraints.min_width && h >= constraints.min_height {
                    return Ok((w, h));
                }
            }
            _ => {}
        }
    }
}

/// Calculates a responsive content width dynamically adapting to the terminal window size.
/// Stretches to fill the terminal window with a clean 2-column margin.
pub fn get_content_width(max_desired: u16) -> usize {
    let (term_w, _) = get_terminal_size();
    let available = (term_w as usize).saturating_sub(2);

    if max_desired == 0 || max_desired <= 80 {
        available.max(MIN_TERM_WIDTH as usize)
    } else {
        available
            .min(max_desired as usize)
            .max(MIN_TERM_WIDTH as usize)
    }
}

/// Registers a process-wide panic hook ensuring raw mode is disabled and the cursor is restored
/// if an unhandled panic occurs while inside the alternate screen.
pub fn init_terminal_panic_hook() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let original_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            let _ = disable_raw_mode();
            let _ = execute!(io::stdout(), LeaveAlternateScreen, Show);
            original_hook(panic_info);
        }));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_width_clamping() {
        let width = get_content_width(80);
        assert!(width >= MIN_TERM_WIDTH as usize);
    }

    #[test]
    fn test_alt_screen_guard_nesting_and_restoration() {
        assert!(!is_alt_screen_active());
        {
            let _g1 = AltScreenGuard::enter();
            assert!(is_alt_screen_active());
            {
                let _g2 = AltScreenGuard::enter();
                assert!(is_alt_screen_active());
                restore_alt_screen_if_active();
                assert!(is_alt_screen_active());
            }
            assert!(is_alt_screen_active());
        }
        assert!(!is_alt_screen_active());
    }
}
