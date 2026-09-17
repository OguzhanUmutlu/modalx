# Introduction

`modalx` is a specialized Rust framework designed for building structured, modal-driven terminal user interfaces without the overhead of full-screen widget graphs or immediate-mode layout engines.

It operates on a straightforward principle: terminal dialogs, forms, menus, and inspectors should be modular, self-contained units that execute synchronously, capture user input cleanly, and return deterministic outcome enums to the caller.

---

## The Problem Space

Modern terminal applications typically face an awkward choice between two extremes:

1. **Lightweight prompt libraries** (e.g. basic readline or inline prompt tools). These work well for linear scripts, but break down when an application requires structured dashboards, multi-field forms with validation, framed tables, or multi-step configuration wizards. Content scrolls haphazardly through the terminal buffer, leaving messy visual artifacts.
2. **Full-screen immediate-mode TUI engines**. While powerful for persistent dashboards (e.g. system monitors), they require setting up full render loops, complex layout trees, state machines, manual focus tracking, and custom event dispatching even when the developer simply wants to prompt the user for database credentials or show a confirmation dialog.

`modalx` occupies the middle ground: providing the visual fidelity, clean borders, and responsive framing of a full TUI engine with the call-site simplicity of a modal dialog.

---

## Core Architectural Pillars

### 1. Synchronous, Encapsulated Execution

Every modal in `modalx` is a self-contained state machine. Calling `.run()` on a modal:

- Enters raw terminal mode and captures terminal focus.
- Renders the frame to an alternate screen buffer or inline canvas using double-buffered atomic writes.
- Drives an internal event loop handling keyboard navigation, text input, scrolling, and window resizing.
- Restores the previous terminal state upon completion.
- Returns a strongly-typed outcome enum (`SelectOutcome`, `FormResult`, `ConfirmOutcome`, etc.).

```rust
// The caller simply awaits the outcome
let outcome = ConfirmModal::new("CONFIRM ACTION", "Proceed with database migration?")
    .run()?;

match outcome {
    ConfirmOutcome::Confirmed => apply_migration()?,
    ConfirmOutcome::Cancelled => println!("Migration aborted."),
}
```

### 2. Strict Box Geometry

In `modalx`, content never floats unconstrained. Visual components are strictly enclosed inside a `BoxFrame` bounded by Unicode borders (`╭─╮`, `│ │`, `├─┤`, `╰─╯`).

The layout engine dynamically computes character-cell dimensions based on terminal width and height, applying padding, truncation, and alignment so that lines never wrap unexpectedly or disrupt outer borders. Non-UTF8 terminals fall back to standard ASCII borders (`+--+`, `|  |`).

### 3. Responsive Text and Metadata Reflow

Rather than truncating lines with ellipses (`Host: web-01 | RAM: 16GB...`) when a terminal window narrows, `modalx` features a greedy token reflow algorithm (`wrap_fields`). Delimited tokens are measured in visual character cells (ignoring ANSI color sequences) and packed onto multiple neatly indented lines within the bounding box.

### 4. Zero-Flicker Double Buffering

Terminal flickering is caused by sequential cursor repositioning and multi-part `write` syscalls. `modalx` eliminates flicker by formatting the entire frame into an in-memory ANSI string buffer before emitting a single atomic write to standard output followed by an immediate flush.

### 5. RAII Terminal Protection

Unexpected process termination or panics while in raw terminal mode can corrupt the user's terminal session (leaving the cursor hidden and disabling echo). `modalx` installs a global panic hook via `init_terminal_panic_hook()` and uses RAII guards (`TerminalGuard`, `AltScreenGuard`) ensuring the terminal is always reset to canonical mode.
