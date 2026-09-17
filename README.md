# modalx

[![Crates.io](https://img.shields.io/crates/v/modalx.svg)](https://crates.io/crates/modalx)
[![Documentation](https://docs.rs/modalx/badge.svg)](https://docs.rs/modalx)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A responsive, declarative, box-encapsulated Terminal User Interface (TUI) and modal dialog engine for Rust, built directly on [crossterm](https://crates.io/crates/crossterm).

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

## Core Capabilities

- **Strict Unicode Box Encapsulation**: All visual content is contained within Unicode borders (`╭─╮`, `│ │`, `├─┤`, `╰─╯`) with automatic ASCII fallback. Boundaries never clip, overflow, or break terminal line layouts.
- **Responsive Terminal Protection**: Detects constrained viewports (< 60 columns x 14 rows), pauses execution with a centered warning card, and automatically resumes on resize.
- **Dynamic Greedy Reflow**: Delimited metadata tokens (e.g. `Host: ... | RAM: ... | Status: ...`) dynamically wrap across multiple framed lines rather than truncating with ellipses.
- **Full Modal Library**:
  - `SelectModal`: Searchable, paginated keyboard-driven menus with hotkeys and vim navigation.
  - `FormModal`: Multi-field forms with real-time validators, keystroke force-validators, masked password fields, integer/numeric constraints, and unsaved changes confirmation.
  - `InputModal`: Readline text editing with word jump, character deletion, password masking, and custom validation.
  - `ConfirmModal`: Affirmative/negative prompts with horizontal centering, default focus control, and danger mode styling.
  - `InfoModal`: Multi-line scrollable dialogs with contextual shortcut bars that omit scroll hints when content fits the viewport.
  - `TableModal`: Multi-column tabular data displays with column alignment (Left, Center, Right) and row selection.
  - `WaitingModal`: In-place progress monitors with smooth animated braille spinners and multi-step checklists.
- **Dynamic Shortcuts Engine**: Contextual shortcut bar (`Shortcuts`, `ShortcutBar`) with automatic deduction and formatted bracketed key hints.
- **RAII Terminal Safety**: Process-wide panic hook ensuring raw mode is disabled and cursor visibility is restored on unexpected exits.

---

## Installation

Add `modalx` to your `Cargo.toml`:

```toml
[dependencies]
modalx = "0.1"
```

Or install via Cargo CLI:

```bash
cargo add modalx
```

---

## Quickstart

### 1. Menu Selection

```rust
use modalx::prelude::*;

fn main() -> modalx::Result<()> {
    let mut selected = 0;

    let modal = SelectModal::new()
        .with_title("SYSTEM MANAGEMENT", false)
        .with_raw_fields("Cluster: us-east-1 | Nodes: 12 | Health: Nominal", " | ")
        .item("1", "Node Pools")
        .item("2", "Persistent Storage")
        .item("3", "Security Policies")
        .item("q", "Exit");

    match modal.run(&mut selected)? {
        SelectOutcome::Selected(idx) => println!("Selected item: {}", idx),
        SelectOutcome::Cancelled => println!("Selection cancelled."),
        _ => {}
    }

    Ok(())
}
```

### 2. Multi-Field Form with Validation

```rust
use modalx::prelude::*;

fn main() -> modalx::Result<()> {
    let modal = FormModal::new("DATABASE CONNECTION")
        .field(FormField::string("host", "Hostname").with_default("127.0.0.1"))
        .field(
            FormField::integer("port", "Port")
                .with_default("5432")
                .with_validator(|val| match val.parse::<u16>() {
                    Ok(p) if p > 0 => Ok(()),
                    _ => Err("Port must be between 1 and 65535".to_string()),
                }),
        )
        .field(FormField::string("user", "Username").with_default("postgres"))
        .field(FormField::password("password", "Password"));

    if let FormResult::Submitted(values) = modal.run()? {
        println!("Connecting to {}:{} as {}", values["host"], values["port"], values["user"]);
    }

    Ok(())
}
```

### 3. Destructive Confirmation Dialog

```rust
use modalx::prelude::*;

fn main() -> modalx::Result<()> {
    let outcome = ConfirmModal::new("PURGE VOLUME", "Permanently remove persistent volume 'vol-data-01'?")
        .danger(true)
        .with_detail("This operation cannot be undone. All database records will be erased.")
        .with_yes_label("Purge Volume")
        .with_no_label("Cancel")
        .default_yes(false)
        .run()?;

    if outcome == ConfirmOutcome::Confirmed {
        println!("Volume purged.");
    }

    Ok(())
}
```

---

## Examples

Run any of the included examples:

```bash
# Minimal menu selection
cargo run --example simple_menu

# Responsive dashboard with metadata reflow
cargo run --example dynamic_dashboard

# Multi-step wizard chaining inputs, menus, and confirms
cargo run --example interactive_wizard

# Structured data table viewer
cargo run --example data_table

# Multi-step animated progress spinner
cargo run --example progress_spinner
```

---

## Documentation

Comprehensive documentation, architecture guides, and cookbooks are available on the official documentation site.

---

## License

MIT (c) Oguzhan Umutlu
