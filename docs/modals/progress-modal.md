# ProgressModal

`ProgressModal` provides real-time progress visualization for streaming downloads, file transfers, and long-running batch operations. It renders within a boxed frame with sub-character smooth Unicode progress bars, braille spinners, exponential-smoothing speed telemetry, and estimated time of arrival (ETA).

---

## Visual Presentation

```
╭──────────────────────────────────────────────────────────────────────────────╮
│                         DOWNLOADING SERVER ASSETS                            │
├──────────────────────────────────────────────────────────────────────────────┤
│  ⠙ Downloading paper-1.21.4-187.jar                                          │
│                                                                              │
│    ████████████████████████████████████████▌              78.4%              │
│    38.4 MB / 49.0 MB  •  12.8 MB/s  •  ETA: 1s                               │
│                                                                              │
│    ✓ Query remote build manifest                                             │
│    • Stream artifact payload                                                 │
│      Validate SHA-256 integrity                                              │
├──────────────────────────────────────────────────────────────────────────────┤
│  [Esc] Cancel Download                                                       │
╰──────────────────────────────────────────────────────────────────────────────╯
```

---

## Basic Download Usage

```rust
use modalx::prelude::*;
use std::io;
use std::thread;
use std::time::Duration;

fn main() -> modalx::Result<()> {
    let mut stdout = io::stdout();
    let total_bytes: u64 = 50 * 1024 * 1024; // 50 MB
    let chunk_size: u64 = 1024 * 1024; // 1 MB

    let mut modal = ProgressModal::new(
        "DOWNLOADING ARTIFACT",
        "Fetching remote runtime package...",
        total_bytes,
    )
    .with_step("Check repository index", true)
    .with_step("Stream binary payload", false);

    modal.render_forced(&mut stdout)?;

    let mut downloaded = 0;
    while downloaded < total_bytes {
        downloaded = (downloaded + chunk_size).min(total_bytes);
        modal.update(downloaded);
        modal.render(&mut stdout)?;
        thread::sleep(Duration::from_millis(30));
    }

    modal.set_step_status(1, true);
    modal.finish("Package downloaded successfully!", &mut stdout)?;
    Ok(())
}
```

---

## Indeterminate Progress

For network streams where `Content-Length` is not provided by the server, use `ProgressModal::indeterminate`:

```rust
use modalx::prelude::*;
use std::io;

let mut stdout = io::stdout();
let mut modal = ProgressModal::indeterminate(
    "STREAMING DATA",
    "Receiving dynamic stream...",
);

// Stream bytes without a known total
modal.inc(4096);
modal.render(&mut stdout)?;
```

---

## Progress Bar Styles

`modalx` supports three bar rendering styles via `ProgressStyle`:

- `ProgressStyle::UnicodeSmooth` (Default): Uses 8-step fractional block elements (`▏`, `▎`, `▍`, `▌`, `▋`, `▊`, `▉`, `█`) for high-resolution progress.
- `ProgressStyle::UnicodeBlock`: Uses solid `█` and background `░` characters.
- `ProgressStyle::Ascii`: Fallback for non-UTF-8 terminals (`=` and `-`).

```rust
use modalx::prelude::*;

let modal = ProgressModal::new("TRANSFER", "Sending file...", 1000)
    .with_style(ProgressStyle::UnicodeSmooth);
```

---

## Configuration Options

| Method | Type | Description |
|---|---|---|
| `new(title, message, total)` | Constructor | Creates a new progress modal with title, status, and total items/bytes. |
| `indeterminate(title, message)` | Constructor | Creates an indeterminate modal (total = 0). |
| `with_step(label, completed)` | Builder | Appends a step to the execution checklist. |
| `with_max_width(u16)` | Builder | Sets the maximum width of the rendered frame (default: 76). |
| `with_style(ProgressStyle)` | Builder | Sets the progress bar character style. |
| `with_throttle_ms(u64)` | Builder | Limits rendering frequency to prevent terminal saturation (default: 35ms). |
| `with_shortcuts(Shortcuts)` | Builder | Customizes footer navigation keys. |
| `update(u64)` | Mutator | Sets current absolute position (e.g. downloaded bytes). |
| `inc(u64)` | Mutator | Increments current position by delta. |
| `set_step_status(usize, bool)` | Mutator | Updates step completion state by index. |
| `set_message(String)` | Mutator | Updates the current status description. |
| `render(&mut stdout)` | Runner | Redraws the modal if throttled time threshold has passed. |
| `render_forced(&mut stdout)` | Runner | Redraws the modal immediately, bypassing throttling. |
| `finish(message, &mut stdout)` | Runner | Displays final completed frame with 100% progress. |
