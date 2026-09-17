# Navigation Stack & Breadcrumbs

`modalx` includes a built-in breadcrumb navigation stack that tracks the user's location through nested menus and wizards. It automatically formats and centers navigation trails directly below modal titles.

---

## Visual Presentation

```
╭──────────────────────────────────────────────────────────────────────────────╮
│                           DATABASE CREDENTIALS                               │
│                   Dashboard › Servers › saga › Database                      │
├──────────────────────────────────────────────────────────────────────────────┤
│  ...                                                                         │
```

---

## The `NavGuard` RAII Guard

Rather than manually passing navigation path strings down function call hierarchies, `modalx` uses an RAII guard pattern:

```rust
use modalx::prelude::*;

fn main_menu() -> modalx::Result<()> {
    let _guard = NavGuard::new("Dashboard");
    servers_menu()?;
    Ok(())
}

fn servers_menu() -> modalx::Result<()> {
    let _guard = NavGuard::new("Servers");
    server_details_menu("saga")?;
    Ok(())
}

fn server_details_menu(name: &str) -> modalx::Result<()> {
    let _guard = NavGuard::new(name);
    // At this point, active breadcrumbs are:
    // Dashboard › Servers › saga
    database_menu()?;
    Ok(())
}
```

When a function returns or errors, `_guard` goes out of scope and drops, automatically popping its entry from the thread-local breadcrumb stack.

---

## Automatic Modal Integration

All standard modals (`SelectModal`, `FormModal`, `InfoModal`, `BoxFrame`) automatically inspect the active breadcrumb stack:

- If breadcrumbs exist, they are formatted using the standard delimiter (` › `) and rendered as a centered subtitle row directly beneath the title bar.
- If no breadcrumbs are active, the subtitle row is omitted with zero vertical space wasted.

---

## The Breadcrumb Squeeze Algorithm

On narrow terminal screens (or deep navigation trees), long breadcrumb trails could overflow the bounding box. `format_breadcrumbs` includes an adaptive squeeze algorithm:

```
Full Path:     Dashboard › Infrastructure › Production › Nodes › Worker-04
Squeezed Path: Dashboard › ... › Nodes › Worker-04
```

1. It measures the available character width inside the box frame.
2. If the full breadcrumb trail fits, it renders in full.
3. If it exceeds available width, intermediate breadcrumb nodes are collapsed into an ellipsis (`...`), preserving the root context (`Dashboard`) and the immediate active context (`Worker-04`).
