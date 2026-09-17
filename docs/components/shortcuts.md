# Shortcuts Bar

The `Shortcuts` module provides a declarative builder for composing standardized footer keybinding guides across frames and modals.

---

## Visual Presentation

A properly formatted shortcut bar provides clear, non-intrusive keyboard hints at the bottom of the bounding frame:

```
╭──────────────────────────────────────────────────────────────────────────────╮
│ ...                                                                          │
├──────────────────────────────────────────────────────────────────────────────┤
│  [↑/↓/j/k] Move  |  [Enter/→] Select  |  [Esc/←] Back  |  [q] Exit           │
╰──────────────────────────────────────────────────────────────────────────────╯
```

---

## Core Types

### `Shortcut`

A `Shortcut` pairs one or more key triggers with a concise action description:

```rust
pub struct Shortcut {
    pub keys: String,
    pub description: String,
}
```

When formatted, it automatically encloses the key triggers in square brackets:

```rust
let sc = Shortcut::new("Enter", "Submit");
assert_eq!(sc.to_string(), "[Enter] Submit");
```

#### Standard Constructors

`modalx` provides pre-built constructors for common terminal conventions:

| Method | Output String | Standard Meaning |
| :--- | :--- | :--- |
| `Shortcut::dismiss()` | `[Enter/Esc] Dismiss` | Close non-destructive alert or info box |
| `Shortcut::select()` | `[Enter/→] Select` | Choose currently focused menu option |
| `Shortcut::move_selection()`| `[↑/↓/j/k] Move` | Navigate list with arrows or vim keys |
| `Shortcut::move_arrows()` | `[↑/↓] Move` | Navigate list with arrow keys only |
| `Shortcut::back()` | `[Esc/←] Back` | Return to parent menu in navigation stack |
| `Shortcut::cancel()` | `[Esc] Cancel` | Abort active form or operation |
| `Shortcut::confirm()` | `[Enter] Confirm` | Confirm affirmative action |
| `Shortcut::save()` | `[Enter] Save` | Submit form and persist changes |
| `Shortcut::toggle()` | `[Space] Toggle` | Toggle focused checkbox or switch |
| `Shortcut::exit()` | `[q] Exit` | Cleanly terminate application |
| `Shortcut::scroll()` | `[↑/↓/PgUp/PgDn] Scroll` | Scroll multi-line viewport |
| `Shortcut::page()` | `[PgUp/PgDn] Page` | Jump between pages in paginated list |

---

## The `Shortcuts` Builder

The `Shortcuts` struct (aliased as `ShortcutBar`) manages a collection of shortcuts and formats them into a centered, pipe-delimited footer line.

```rust
use modalx::prelude::*;

let bar = Shortcuts::new()
    .add(Shortcut::move_selection())
    .add(Shortcut::select())
    .add(Shortcut::back())
    .add(Shortcut::exit());

assert_eq!(
    bar.to_footer_string(),
    "[↑/↓/j/k] Move  |  [Enter/→] Select  |  [Esc/←] Back  |  [q] Exit"
);
```

### Conditional Shortcut Inclusion

Modals frequently need to show or hide shortcuts based on runtime context (e.g. only showing `Scroll` if content overflows, or only showing `Page` if multiple pages exist).

`Shortcuts` provides conditional builder methods:

```rust
let can_scroll = content.lines().count() > viewport_height;
let is_root_menu = current_depth == 0;

let bar = Shortcuts::new()
    .add(Shortcut::dismiss())
    .scroll_if(can_scroll)
    .exit_if(is_root_menu)
    .add_if(!is_root_menu, Shortcut::back());
```

---

## Integration with `BoxFrame`

`BoxFrame` directly accepts `Shortcuts` when constructing a frame:

```rust
let shortcuts = Shortcuts::new()
    .add(Shortcut::save())
    .add(Shortcut::cancel());

let mut frame = BoxFrame::new(80);
frame.title("SETTINGS");
frame.row("Configure daemon listen port:");
frame.shortcuts(&shortcuts);

let rendered = frame.render(Some(24));
```
