# InfoModal

`InfoModal` displays multi-line text messages, diagnostic inspection results, logs, or release notes. It features smooth scrolling, dynamic scroll indicators, and context-aware shortcut bars that automatically omit scroll hints when content fits the terminal window without scrolling.

---

## Visual Presentation

```
╭──────────────────────────────────────────────────────────────────────────────╮
│                        HEALTH CHECK REPORT                                   │
│              Cluster › us-east-1 › Ingress Controller › Status               │
├──────────────────────────────────────────────────────────────────────────────┤
│  Endpoint: https://api.internal.net/v1/health                                │
│  Status: 200 OK (Nominal)                                                    │
│  Latency: 14.2ms                                                             │
│                                                                              │
│  TLS Certificate: Valid (expires in 84 days)                                 │
│  TLS Issuer: Let's Encrypt Authority X3                                      │
├──────────────────────────────────────────────────────────────────────────────┤
│  [Enter/Esc] Dismiss  |  [q] Exit                                            │
╰──────────────────────────────────────────────────────────────────────────────╯
```

---

## Basic Usage

```rust
use modalx::prelude::*;

fn main() -> modalx::Result<()> {
    let output_lines = vec![
        "Endpoint: https://api.internal.net/v1/health".to_string(),
        "Status: 200 OK (Nominal)".to_string(),
        "Latency: 14.2ms".to_string(),
        "".to_string(),
        "TLS Certificate: Valid (expires in 84 days)".to_string(),
        "TLS Issuer: Let's Encrypt Authority X3".to_string(),
    ];

    InfoModal::new("HEALTH CHECK REPORT", output_lines)
        .run()?;

    Ok(())
}
```

---

## Dynamic Context-Aware Footers

In standard terminal dialogs, footer keybinding guides often display a static hint such as `[↑/↓/PgUp/PgDn] Scroll`, even when the entire dialog consists of only 3 lines that fit completely inside the window.

`InfoModal` dynamically assesses viewport capacity:

```rust
let can_scroll = self.lines.len() > viewport_size;

let shortcuts = if can_scroll {
    Shortcuts::new()
        .add(Shortcut::dismiss())
        .add(Shortcut::scroll())
        .add(Shortcut::exit())
} else {
    // Scroll hint is omitted because content fits entirely
    Shortcuts::new()
        .add(Shortcut::dismiss())
        .add(Shortcut::exit())
};
```

When content fits within the window:
```
[Enter/Esc] Dismiss  |  [q] Exit
```

When content overflows and scrolling is required:
```
[Enter/Esc] Dismiss  |  [↑/↓/PgUp/PgDn] Scroll  |  [q] Exit
```

---

## Viewport Scrolling & Navigation

When viewing long logs or extensive outputs:

| Key Binding | Navigation Effect |
| :--- | :--- |
| `Down` / `j` | Scroll viewport down by 1 line |
| `Up` / `k` | Scroll viewport up by 1 line |
| `PageDown` / `Space` | Scroll viewport down by one page (`viewport_height - 2`) |
| `PageUp` | Scroll viewport up by one page |
| `Home` / `g` | Jump to the beginning of the content |
| `End` / `G` | Jump to the end of the content |
| `Enter` / `Esc` / `q` | Dismiss the dialog and return `Ok(())` |

When scrolled, subtle boundary indicators appear in the right border (`▲` at the top when lines exist above the viewport, and `▼` at the bottom when lines exist below) to indicate scroll progress to the user.
