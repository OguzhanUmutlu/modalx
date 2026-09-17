# Getting Started

This guide walks through adding `modalx` to a Rust project, configuring terminal lifecycle handling, and running your first interactive modal dialog.

---

## Installation

Add `modalx` to your `Cargo.toml` dependencies:

```toml
[dependencies]
modalx = "0.1"
```

Or use the Cargo CLI:

```bash
cargo add modalx
```

### System Requirements
- **Rust Edition**: 2021 or later.
- **Platforms**: Linux, macOS, Windows (Windows Terminal, PowerShell, or cmd.exe with ANSI support).
- **Core Dependencies**: `crossterm` 0.29, `colored` 3.0, `thiserror` 2.0. No heavy external C libraries or windowing runtimes are required.

---

## The Prelude

For convenience, `modalx` provides a prelude module that exports all essential types, traits, and modal constructors:

```rust
use modalx::prelude::*;
```

The prelude re-exports:
- Modal structs: `SelectModal`, `FormModal`, `InputModal`, `ConfirmModal`, `InfoModal`, `TableModal`, `WaitingModal`.
- Outcome types: `SelectOutcome`, `FormResult`, `InputOutcome`, `ConfirmOutcome`, `TableOutcome`.
- Layout and styling types: `BoxFrame`, `Shortcuts`, `Shortcut`, `FieldSection`, `TitleSection`, `NavGuard`.
- Terminal guards: `TerminalGuard`, `AltScreenGuard`.
- Error types: `Result`, `TuiError`.

---

## First Modal Program

Below is a complete, standalone program demonstrating a menu modal that runs in the terminal's alternate screen buffer:

```rust
use modalx::prelude::*;

fn main() -> modalx::Result<()> {
    // Optional: Switch to alternate screen to preserve shell history
    let _alt = AltScreenGuard::enter();

    let mut selected_index = 0;

    let modal = SelectModal::new()
        .with_title("APPLICATION CONTROL PANEL", false)
        .with_raw_fields("Environment: Production | Region: us-west-2", " | ")
        .item("1", "Inspect Microservices")
        .item("2", "View Ingress Logs")
        .item("3", "Cluster Autoscaler Configuration")
        .item("4", "Restart Gateway Pods")
        .item("q", "Exit");

    match modal.run(&mut selected_index)? {
        SelectOutcome::Selected(idx) => {
            println!("User selected entry at index {}", idx);
        }
        SelectOutcome::Cancelled => {
            println!("User cancelled the prompt (pressed Esc or q).");
        }
        _ => {}
    }

    Ok(())
}
```

When run, the screen clears, switches to the alternate screen, renders a centered box frame with title and status metadata, and captures keypresses until an option is selected or dismissed. Upon exiting, the original shell contents are restored intact.

---

## Error Handling

All modal operations return `modalx::Result<T>`, where `Result<T>` is an alias for `std::result::Result<T, modalx::TuiError>`.

### Error Variants

```rust
#[derive(Debug, thiserror::Error)]
pub enum TuiError {
    #[error("Terminal I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Terminal window too small ({width}x{height}, required: {min_width}x{min_height})")]
    TerminalTooSmall {
        width: u16,
        height: u16,
        min_width: u16,
        min_height: u16,
    },

    #[error("Modal execution cancelled by user")]
    Cancelled,

    #[error("{0}")]
    Other(String),
}
```

### Propagating with `anyhow` or Custom Errors

Because `TuiError` implements `std::error::Error`, it integrates seamlessly with `anyhow` or `thiserror` in application code:

```rust
// In custom application error enum:
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("TUI interface error: {0}")]
    Tui(#[from] modalx::TuiError),

    #[error("Network failure: {0}")]
    Network(String),
}
```
