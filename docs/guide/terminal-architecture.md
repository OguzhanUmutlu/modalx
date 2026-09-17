# Terminal Architecture & Lifecycle

`modalx` interacts directly with terminal emulators via the virtual terminal sequence protocol. Understanding the underlying terminal pipeline helps in writing predictable, clean CLI applications.

---

## The Terminal Mode Pipeline

Operating systems typically maintain terminals in **canonical mode** (often referred to as "cooked mode"). In canonical mode:
- Input is line-buffered by the operating system kernel. Characters typed by the user are held until the user presses Enter.
- The operating system handles character echo and backspacing automatically.
- Control signals (e.g. `Ctrl+C`, `Ctrl+Z`) generate OS interrupt signals (`SIGINT`, `SIGTSTP`).

For interactive modals, `modalx` switches the terminal into **raw mode**:
- Input is delivered byte-by-byte with zero kernel buffering. Keystrokes are intercepted the microsecond they occur.
- Automatic echo is disabled, giving the modal full control over what is drawn on screen.
- Line endings and carriage returns are managed explicitly.

---

## RAII Resource Guards

Raw terminal mode has one major risk: if the program exits abnormally or panics while raw mode is active, the user's shell remains in raw mode. Keystrokes will not echo, Enter will not advance lines properly, and the cursor may remain invisible.

`modalx` prevents this through deterministic RAII (Resource Acquisition Is Initialization) guards and panic hooks.

### `TerminalGuard`

`TerminalGuard` encapsulates the full lifecycle of a terminal modal session:

```rust
pub struct TerminalGuard {
    alt_screen: bool,
}

impl TerminalGuard {
    pub fn enter() -> Result<Self> {
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), cursor::Hide, terminal::EnterAlternateScreen)?;
        Ok(Self { alt_screen: true })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), cursor::Show, terminal::LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}
```

When the guard goes out of scope (whether through a normal return, an error `?`, or stack unwinding), its `Drop` implementation executes automatically, restoring the canonical terminal state.

### `AltScreenGuard`

When you want to run a series of modals without clearing the terminal between every step, you can instantiate an `AltScreenGuard` at the top of your program:

```rust
fn main() -> modalx::Result<()> {
    let _alt = AltScreenGuard::enter();

    // All subsequent modals render within the alternate screen
    step_one_modal()?;
    step_two_modal()?;

    Ok(())
} // When _alt drops, the original shell viewport is restored perfectly
```

### Global Panic Hook

In the event of an unhandled panic, `modalx` ensures the terminal is cleanly reset before printing the panic backtrace:

```rust
modalx::terminal::init_terminal_panic_hook();
```

Internally, this installs a custom panic hook that issues `terminal::disable_raw_mode()` and `cursor::Show` to standard error before delegating to the default panic handler.

---

## Zero-Flicker Double Buffering

Terminal visual tearing and flickering occurs when an application clears the screen and prints lines sequentially across multiple system calls. The terminal emulator renders intermediate states between writes, causing visible flashing.

`modalx` resolves this through atomic in-memory double buffering:

```
+-------------------------------------------------------------+
|                     In-Memory Buffer                        |
|                                                             |
|   1. Construct outer top border                             |
|   2. Compute and format centered header row                 |
|   3. Format content sections and reflow metadata lines      |
|   4. Add active button elements and shortcut bar            |
|   5. Construct outer bottom border                          |
+-------------------------------------------------------------+
                              |
                     Single Atomic Write
                              |
                              v
+-------------------------------------------------------------+
|             Standard Output (crossterm / stdout)            |
|                                                             |
|   MoveTo(0, 0) -> Write Entire Buffer -> Flush Output       |
+-------------------------------------------------------------+
```

### Rendering Mechanics

1. During each frame, `BoxFrame` allocates a single `String` buffer with capacity pre-computed based on terminal height and width.
2. The entire boxed layout (borders, headers, content lines, padding, footers) is written into this memory buffer.
3. The renderer executes:
   ```rust
   let mut stdout = io::stdout();
   execute!(stdout, cursor::MoveTo(0, 0))?;
   stdout.write_all(buffer.as_bytes())?;
   stdout.flush()?;
   ```
4. The terminal emulator receives the entire frame in a single packet, rendering the screen update instantaneously with zero visual flicker.

---

## Event Draining

When transitioning between modal screens (for example, when an affirmative confirmation modal succeeds and the next step begins), keystrokes typed rapidly by the user could leak into the subsequent modal.

`modalx` provides an event draining helper that discards pending input events before entering an interactive loop:

```rust
while event::poll(Duration::from_millis(0))? {
    let _ = event::read()?;
}
```

This ensures that each modal starts with a clean input buffer.
