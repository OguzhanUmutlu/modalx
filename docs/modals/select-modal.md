# SelectModal & MenuModal

`SelectModal` provides an interactive, keyboard-driven selection menu. It supports hotkeys, real-time substring filtering, vim navigation, pagination for long lists, and declarative metadata header sections.

---

## Visual Presentation

```
╭──────────────────────────────────────────────────────────────────────────────╮
│                        INFRASTRUCTURE MANAGEMENT                             │
│                           Dashboard › Clusters                               │
├──────────────────────────────────────────────────────────────────────────────┤
│  Region: us-east-1 | Healthy Nodes: 48/48 | Active Alerts: 0                 │
├──────────────────────────────────────────────────────────────────────────────┤
│  > [1]   Production Kubernetes Cluster (k8s-prod-us-east)                    │
│    [2]   Staging Kubernetes Cluster    (k8s-stage-us-east)                   │
│    [3]   Analytics Spark Cluster       (spark-analytics-01)                  │
│    [4]   Edge Ingress Gateway Nodes    (ingress-edge-pool)                   │
│    [n]   Provision New Cluster                                               │
│    [q]   Return to Main Dashboard                                            │
├──────────────────────────────────────────────────────────────────────────────┤
│  [↑/↓/j/k] Move  |  [Enter/→] Select  |  [Esc/←] Back  |  [/] Filter  | [q]   │
╰──────────────────────────────────────────────────────────────────────────────╯
```

---

## Basic Usage

```rust
use modalx::prelude::*;

fn main() -> modalx::Result<()> {
    let mut current_selection = 0;

    let modal = SelectModal::new()
        .with_title("SERVER SELECTION", false)
        .with_header_row("Select an active host to open SSH session:")
        .item("1", "app-server-01 (Ubuntu 24.04)")
        .item("2", "db-primary-01 (Debian 12)")
        .item("3", "cache-redis-01 (Alpine 3.20)")
        .item("q", "Exit");

    match modal.run(&mut current_selection)? {
        SelectOutcome::Selected(0) => connect_ssh("app-server-01")?,
        SelectOutcome::Selected(1) => connect_ssh("db-primary-01")?,
        SelectOutcome::Selected(2) => connect_ssh("cache-redis-01")?,
        SelectOutcome::Selected(3) | SelectOutcome::Cancelled => {
            println!("Operation cancelled.");
        }
        _ => {}
    }

    Ok(())
}
```

---

## Configuration Options

### 1. Title and Header Configuration
- `.with_title(title, is_main_menu)`: Sets the top title bar. Setting `is_main_menu: true` enables double borders (`╔═╗`) on compatible themes.
- `.with_header_row(line)`: Adds an unselectable text header directly below the title.
- `.with_raw_fields(raw_text, delimiter)`: Adds delimited metadata tokens (e.g. `Host: ubuntu | RAM: 16GB`) that automatically reflow on narrow terminals.

### 2. Item Definitions
Items can be added using the fluent `.item()` helper:

```rust
// .item(key_label, description)
modal.item("1", "First Option")
     .item("2", "Second Option")
     .item("n", "Create New Resource")
     .item("q", "Back");
```

Pressing the corresponding key (`1`, `2`, `n`, `q`) immediately triggers selection without requiring arrow navigation followed by Enter.

### 3. Real-Time Search Filtering
If the modal contains more than 8 items, pressing `/` or typing characters enters search mode:
- Items are dynamically filtered against the search substring in real time.
- The item cursor automatically resets to the first matching entry.
- Pressing `Backspace` removes characters from the search query.
- Pressing `Esc` clears the active filter.

### 4. Viewport Pagination
For long lists (e.g. 50+ items), `SelectModal` automatically restricts the visible list to fit the terminal height:
- `PageUp` / `PageDown` scroll through pages of items.
- Up and down arrows smoothly scroll the viewport when the cursor reaches the boundary.

---

## Return Outcomes

The `.run(&mut selected_index)` method returns `modalx::Result<SelectOutcome>`:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectOutcome {
    /// An item was selected by index (0-based according to original item list).
    Selected(usize),
    /// The user dismissed the modal with Escape or a designated back key.
    Cancelled,
    /// A custom hotkey triggered an application-specific action.
    Custom(String),
}
```

Notice that `selected_index` is passed as a mutable reference (`&mut selected_index`). `SelectModal` updates this variable to reflect the user's final selection, allowing you to persist cursor position across menu transitions.
