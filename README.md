# modalx

[![Crates.io](https://img.shields.io/crates/v/modalx.svg)](https://crates.io/crates/modalx)
[![Documentation](https://docs.rs/modalx/badge.svg)](https://docs.rs/modalx)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A responsive, declarative, box-encapsulated Terminal User Interface (TUI) and modal dialog engine for Rust built on top of [crossterm](https://crates.io/crates/crossterm).

```
╭──────────────────────────────────────────────────────────────────────────────╮
│                        CLOUD INFRASTRUCTURE DASHBOARD                        │
├──────────────────────────────────────────────────────────────────────────────┤
│  Host: Ubuntu-Workstation | RAM: 14.2 / 32.0 GB (44.3%) | Status: [HEALTHY]  │
│  Active Services: 18 | Alerts: 0 | Network: 1.2 Gbps                         │
├──────────────────────────────────────────────────────────────────────────────┤
│  > [1]   Virtual Machines                                                    │
│    [2]   Kubernetes Clusters                                                 │
│    [3]   Persistent Volumes                                                  │
│    [4]   Security Groups & Firewall                                          │
│    [5]   Audit & Access Logs                                                 │
│    [6]   Billing & Usage                                                     │
├──────────────────────────────────────────────────────────────────────────────┤
│  [↑/↓/j/k] Move  |  [Enter/→] Select  |  [Esc/←] Back  |  [q] Exit           │
╰──────────────────────────────────────────────────────────────────────────────╯
```

---

## Highlights

- 📦 **Airtight Dynamic Box Framing**: All visual components are strictly enclosed inside Unicode borders (`╭─╮`, `│ │`, `├─┤`, `╰─╯`). Text never overflows, warps, or breaks borders.
- 📐 **Responsive Terminal Protection**: Automatically detects narrow or short windows (< 60 cols × 14 rows), rendering a clean, centered "TERMINAL TOO SMALL" warning that safely resumes execution when resized.
- 🔄 **Greedy Flow Layout & Responsive Field Reflow**: Delimited metadata (e.g. `Host: ... | RAM: ... | Daemon: ...`) reflows greedily across multiple lines when terminal width is constrained, eliminating ugly truncation (`...`).
- 🧩 **Declarative Section Composition**: Construct screens using composable sections (`TitleSection`, `FieldSection`, `TextSection`, `MenuSection`, `CustomSection`, `DividerSection`, `FooterSection`).
- 🧰 **Batteries-Included Modals**:
  - **`SelectModal`**: Keyboard-navigable list with real-time search filtering, hotkeys (`1-9`, `a-z`), aliases, vim bindings (`j`/`k`), and viewport pagination.
  - **`InputModal`**: Readline editor supporting cursor movement, word jumps (`Ctrl+Left`/`Right`), deletion (`Ctrl+W`/`Ctrl+U`), password masking, and real-time custom validation.
  - **`ConfirmModal`**: Two-button affirmative/negative prompts (`[ Yes ] / [ No ]`), danger styling, and Tab/arrow/Y/N toggling.
  - **`InfoModal`**: Scrollable text and alert dialog with PageUp/PageDown, Up/Down, and Home/End.
  - **`WaitingModal`**: In-place progress monitors with smooth animated braille spinners (`⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏`).
  - **`TableModal`**: Structured multi-column tabular data with custom alignments (Left, Center, Right) and row selection.
- 🧭 **Hierarchical Navigation Breadcrumbs**: Built-in RAII breadcrumb stack (`NavGuard`) that automatically formats and centers navigation trails (`Dashboard › Servers › My Server`) directly below modal titles.
- 🛡️ **Panic Safety & RAII Alternate Screen**: Includes a process-wide panic hook ensuring the cursor is unhidden and raw mode is cleanly disabled even on unexpected panics, keeping the user's terminal pristine.

---

## Installation

Add `modalx` to your `Cargo.toml`:

```toml
[dependencies]
modalx = "1.0"
```

Or install via `cargo add`:

```bash
cargo add modalx
```

---

## Quickstart

### 1. Minimal Menu Selection

```rust
use modalx::prelude::*;

fn main() -> modalx::Result<()> {
    let mut selected = 0;

    let modal = SelectModal::new()
        .with_title("APPLICATION MENU", false)
        .with_raw_fields("Environment: Production | Region: us-east-1", " | ")
        .item("1", "Manage Databases")
        .item("2", "View Cluster Metrics")
        .item("3", "Deploy Applications")
        .item("q", "Exit Application");

    match modal.run(&mut selected)? {
        SelectOutcome::Selected(idx) => println!("Selected entry: {}", idx),
        SelectOutcome::Cancelled => println!("User pressed Escape"),
        _ => {}
    }

    Ok(())
}
```

### 2. Interactive Input with Validation

```rust
use modalx::prelude::*;

fn main() -> modalx::Result<()> {
    let outcome = InputModal::new("USER REGISTRATION", "Enter your username:")
        .with_placeholder("john_doe")
        .with_validator(|val| {
            if val.len() < 3 {
                Err("Username must be at least 3 characters".to_string())
            } else if val.contains(' ') {
                Err("Username cannot contain spaces".to_string())
            } else {
                Ok(())
            }
        })
        .run()?;

    if let InputOutcome::Submitted(username) = outcome {
        println!("Registered user: {}", username);
    }

    Ok(())
}
```

### 3. Destructive Confirmation Dialog

```rust
use modalx::prelude::*;

fn main() -> modalx::Result<()> {
    let outcome = ConfirmModal::new("DROP DATABASE", "Are you sure you want to drop 'production_db'?")
        .danger(true)
        .with_detail("This action is irreversible and all data will be permanently deleted.")
        .with_yes_label("Drop Database")
        .with_no_label("Cancel")
        .default_yes(false)
        .run()?;

    if outcome == ConfirmOutcome::Confirmed {
        println!("Database dropped.");
    }

    Ok(())
}
```

---

## Composable Sections & Responsive Field Reflow

Build complex dashboards by composing sections. Delimited metadata reflows automatically across lines to fit narrow terminal windows:

```rust
use modalx::prelude::*;

let fields = FieldSection::new()
    .with_separator(" | ")
    .add_field("Host: Ubuntu")
    .add_field("RAM: 14.2 / 32.0 GB (44.3%)")
    .add_field("Status: [ONLINE]");

let modal = SelectModal::new()
    .with_title("INFRASTRUCTURE DASHBOARD", false)
    .with_section(ModalSection::Fields(fields))
    .item("1", "Server Control")
    .item("2", "Backup Manager");
```

---

## Examples

Run any of the included examples to see `modalx` in action:

```bash
# Minimal menu
cargo run --example simple_menu

# Responsive dashboard with dynamic metadata reflow
cargo run --example dynamic_dashboard

# Multi-step wizard chaining inputs, menus, and confirms
cargo run --example interactive_wizard

# Structured data table viewer
cargo run --example data_table

# Multi-step animated spinner
cargo run --example progress_spinner
```

---

## License

MIT © Oguzhan Umutlu
