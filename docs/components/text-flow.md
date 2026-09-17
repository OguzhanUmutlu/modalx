# Text Flow & Dynamic Reflow

The `text_flow` module implements text measurement, word wrapping, button layout, and delimited metadata reflow algorithms. It is strictly ANSI-aware, ensuring that colored text never breaks character-cell alignment.

---

## The ANSI Alignment Challenge

In terminal interfaces, text is frequently styled with ANSI Select Graphic Rendition (SGR) escape sequences (e.g. `\x1b[32m` for green, `\x1b[1m` for bold, `\x1b[0m` for reset).

If an algorithm uses standard `str::len()` or `char::count()`, ANSI codes inflate the measured length:

```rust
let text = "Online".green().bold().to_string();
// Visual characters displayed on screen: 6 ("Online")
// String byte length: 19 (includes \x1b[32m\x1b[1m...\x1b[0m)
```

Naive layout algorithms that allocate line padding based on byte length will under-pad lines, causing the right border (`│`) of the box frame to warp and misalign.

`modalx` resolves this with `visible_len()`:

```rust
pub fn visible_len(text: &str) -> usize {
    strip_ansi(text).chars().count()
}
```

Every width calculation across `BoxFrame` and `text_flow` strictly uses visible cell width.

---

## Greedy Metadata Reflow (`wrap_fields`)

When displaying multi-token metadata headers (e.g. `Host: ubuntu | RAM: 16GB | CPU: 12% | Status: OK`), narrow terminals cannot fit all tokens on a single line.

Instead of truncating the line with ellipses (`Host: ubuntu | RAM: 16...`), `wrap_fields` greedily packs tokens across multiple lines up to the target width:

```
Wide Terminal (100 columns):
│  Host: ubuntu-01 | RAM: 16 GB | CPU: 12.4% | Network: 1.2 Gbps | Status: OK  │

Narrow Terminal (60 columns):
│  Host: ubuntu-01 | RAM: 16 GB | CPU: 12.4%                                   │
│  Network: 1.2 Gbps | Status: OK                                              │
```

### Usage

```rust
use modalx::text_flow::wrap_fields;

let tokens = vec![
    "Host: production-cluster-01",
    "Region: us-east-1",
    "Nodes: 48",
    "Uptime: 42d",
    "Status: [NOMINAL]",
];

let wrapped_lines = wrap_fields(&tokens, " | ", 60);
for line in wrapped_lines {
    println!("{}", line);
}
```

---

## Button Row Wrapping & Centering (`wrap_button_items`)

When rendering action buttons (such as in `ConfirmModal` or `FormModal`), long button labels must wrap cleanly onto multiple rows while remaining centered horizontally:

```rust
use modalx::text_flow::wrap_button_items;

let buttons = vec![
    "[ Primary Action ]",
    "[ Secondary Option ]",
    "[ Cancel Operation ]",
];

// Wraps buttons into rows that fit within 50 columns, centering each row
let rows = wrap_button_items(&buttons, 50, 4);
```

---

## ANSI-Preserving Truncation (`truncate_ansi`)

When text must be cut off at a hard column limit, standard slicing can sever an escape code mid-sequence (e.g. `\x1b[32`), corrupting the terminal's color palette.

`truncate_ansi` cuts text at the exact visual character boundary and cleanly appends an ANSI reset sequence (`\x1b[0m`) before adding a dimmed ellipsis (`...`):

```rust
use modalx::text_flow::truncate_ansi;

let colored_line = "Processing large file archive...".cyan().bold().to_string();
let truncated = truncate_ansi(&colored_line, 18);
// Result visually renders 18 cells, cleanly resetting colors
```
