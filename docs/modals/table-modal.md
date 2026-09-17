# TableModal

`TableModal` provides a structured, multi-column tabular data browser. It supports custom column alignments (Left, Center, Right), column width allocations, interactive row selection, and viewport pagination.

---

## Visual Presentation

```
╭──────────────────────────────────────────────────────────────────────────────╮
│                           ACTIVE PROCESS INSPECTOR                           │
├──────────────────────────────────────────────────────────────────────────────┤
│     PID  Process Name               Memory (MB)     CPU (%)  Status          │
├──────────────────────────────────────────────────────────────────────────────┤
│    1042  systemd                           48.2         0.1  Running         │
│  > 2180  dockerd                          312.5         1.4  Running         │
│    3401  postgres                         524.8         2.1  Running         │
│    4892  redis-server                      84.1         0.3  Running         │
│    5110  nginx                             32.0         0.2  Running         │
│    6720  worker-pool                       41.6         0.0  Running         │
├──────────────────────────────────────────────────────────────────────────────┤
│  [↑/↓/j/k] Move  |  [Enter] Select Row  |  [Esc] Dismiss                     │
╰──────────────────────────────────────────────────────────────────────────────╯
```

---

## Basic Usage

```rust
use modalx::prelude::*;

fn main() -> modalx::Result<()> {
    let mut selected_row = 0;

    let columns = vec![
        TableColumn::new("PID", 8).right_aligned(),
        TableColumn::new("Process Name", 24),
        TableColumn::new("Memory (MB)", 14).right_aligned(),
        TableColumn::new("CPU (%)", 10).right_aligned(),
        TableColumn::new("Status", 12),
    ];

    let rows = vec![
        vec!["1042".into(), "systemd".into(), "48.2".into(), "0.1".into(), "Running".into()],
        vec!["2180".into(), "dockerd".into(), "312.5".into(), "1.4".into(), "Running".into()],
        vec!["3401".into(), "postgres".into(), "524.8".into(), "2.1".into(), "Running".into()],
        vec!["4892".into(), "redis-server".into(), "84.1".into(), "0.3".into(), "Running".into()],
    ];

    let modal = TableModal::new("ACTIVE PROCESS INSPECTOR")
        .with_columns(columns)
        .with_rows(rows)
        .with_selectable(true);

    match modal.run(&mut selected_row)? {
        TableOutcome::Selected(row_idx) => {
            println!("Selected process at row {}", row_idx);
        }
        TableOutcome::Cancelled => {
            println!("Inspector dismissed.");
        }
    }

    Ok(())
}
```

---

## Column Alignment & Width Accounting

Each column is configured through `TableColumn`:

```rust
pub struct TableColumn {
    pub header: String,
    pub width: usize,
    pub align: ColumnAlign,
}
```

### Alignment Modes

- **Left Aligned** (`TableColumn::new(header, width)`): Best for strings, names, identifiers, and file paths. Text is left-justified and padded with spaces to the right.
- **Right Aligned** (`.right_aligned()`): Essential for numbers, currency, timestamps, and percentages, ensuring decimal points and digits align visually down the column.
- **Center Aligned** (`.center_aligned()`): Suitable for status badges (e.g. `[OK]`, `RUNNING`, `PAUSED`).

---

## Selection Modes

`TableModal` can be used in two modes:

1. **Selectable Mode** (`.with_selectable(true)`):
   - A selection cursor (`> `) highlights the active row in bold or cyan.
   - Up and down arrows navigate rows.
   - Pressing `Enter` returns `TableOutcome::Selected(usize)` with the chosen row index.
2. **Read-Only Mode** (`.with_selectable(false)`):
   - Used for purely informational tables.
   - No selection cursor is rendered.
   - Pressing `Enter`, `Space`, or `Esc` dismisses the table.
