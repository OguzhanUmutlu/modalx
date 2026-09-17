# Modals Overview

`modalx` includes a suite of built-in modal dialogs tailored for standard terminal workflows. Every modal implements a consistent builder pattern and returns a strongly-typed outcome enum.

---

## Built-In Modals Reference

| Modal | Primary Use Case | Output Type | Key Navigation Features |
| :--- | :--- | :--- | :--- |
| **`SelectModal`** | Menus, options lists, resource selectors | `SelectOutcome` | Arrows, vim keys (`j`/`k`), numeric hotkeys (`1-9`), search filtering |
| **`FormModal`** | Multi-field records, credentials, network settings | `FormResult` | `Tab`/`Shift+Tab` cycling, real-time validators, force-validators, masked passwords |
| **`InputModal`** | Single-value prompt (names, paths, URLs) | `InputOutcome` | Full readline editing, word deletion (`Ctrl+W`), real-time error banner |
| **`ConfirmModal`** | Binary choices, destructive action gates | `ConfirmOutcome` | Centered text flow, `y`/`n` shortcuts, danger mode, default focus toggle |
| **`InfoModal`** | Log inspections, release notes, ping results | `Result<()>` | Dynamic contextual footer, scrolling indicators (`▲`/`▼`), PageUp/Down |
| **`TableModal`** | Tabular datasets, process lists, metrics | `TableOutcome` | Fixed/proportional column widths, Left/Center/Right alignments, row cursor |
| **`WaitingModal`** | Long-running operations, migrations, backups | `Result<()>` | Animated braille spinner, multi-step checklist state, sync/async drivers |

---

## The Modal Lifecycle

All modals follow a three-stage lifecycle:

```
+-------------------------------------------------------------+
| 1. Configuration (Builder Pattern)                          |
|    let modal = SelectModal::new()                           |
|        .with_title("TITLE", false)                          |
|        .item("1", "First Option");                          |
+-------------------------------------------------------------+
                              |
                              v
+-------------------------------------------------------------+
| 2. Interactive Execution Loop                               |
|    let outcome = modal.run(&mut selected_idx)?;             |
|    - Enables raw mode and enters double-buffered canvas     |
|    - Polls keyboard events and handles terminal resize      |
|    - Cleans up upon selection, cancellation, or exit        |
+-------------------------------------------------------------+
                              |
                              v
+-------------------------------------------------------------+
| 3. Outcome Evaluation                                       |
|    match outcome {                                          |
|        SelectOutcome::Selected(idx) => { ... }              |
|        SelectOutcome::Cancelled => { ... }                  |
|    }                                                        |
+-------------------------------------------------------------+
```

---

## Common Key Conventions Across Modals

To ensure an intuitive experience for terminal users, all modals share standard key conventions:

- **Cancel / Back**: `Esc` uniformly dismisses or navigates back.
- **Confirm / Submit**: `Enter` confirms the focused button, submits a form, or selects an item.
- **Quit**: Where applicable, `q` or `Ctrl+C` terminates cleanly.
- **Navigation**: Both arrow keys (`↑`, `↓`, `←`, `→`) and vim bindings (`k`, `j`, `h`, `l`) are accepted in list-based modals.
