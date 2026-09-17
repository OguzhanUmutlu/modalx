# WaitingModal

`WaitingModal` provides in-place progress monitoring for background tasks, installations, network transfers, and database migrations. It features smooth animated braille spinners and multi-step progress checklists.

---

## Visual Presentation

```
╭──────────────────────────────────────────────────────────────────────────────╮
│                         APPLYING SYSTEM UPGRADE                              │
├──────────────────────────────────────────────────────────────────────────────┤
│  ⠋ Downloading updated binary packages...                                    │
│                                                                              │
│    ✓ Verify digital signature                                                │
│    ✓ Create configuration backup                                             │
│    ⠋ Download binary assets                                                  │
│      Apply database migrations                                               │
├──────────────────────────────────────────────────────────────────────────────┤
│  [Esc] Cancel Operation                                                      │
╰──────────────────────────────────────────────────────────────────────────────╯
```

---

## Synchronous Step-by-Step Execution

```rust
use modalx::prelude::*;
use std::thread;
use std::time::Duration;

fn main() -> modalx::Result<()> {
    let _alt = AltScreenGuard::enter();
    let mut stdout = std::io::stdout();

    let mut modal = WaitingModal::new("SYSTEM UPGRADE", "Applying system updates...")
        .with_step("Verify digital signature", true)
        .with_step("Backup configuration", true)
        .with_step("Download binary assets", false)
        .with_step("Run database migrations", false);

    // Animate spinner during download step
    for frame in 0..40 {
        modal.render_spinner(frame, &mut stdout)?;
        thread::sleep(Duration::from_millis(80));
    }

    // Mark third step as complete
    modal.steps[2].1 = true;

    // Animate spinner during migration step
    for frame in 40..80 {
        modal.render_spinner(frame, &mut stdout)?;
        thread::sleep(Duration::from_millis(80));
    }

    modal.steps[3].1 = true;
    modal.message = "Upgrade completed successfully!".to_string();
    modal.render_static(&mut stdout)?;
    thread::sleep(Duration::from_secs(1));

    Ok(())
}
```

---

## The Braille Spinner Cycle

`WaitingModal` uses a smooth, 10-frame high-resolution Unicode Braille spinner:

```rust
pub const SPINNER_FRAMES: &[char] = &[
    '⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'
];
```

Because braille characters occupy a single character cell, the spinner animates continuously without shifting the trailing message text horizontally.

---

## Asynchronous Task Integration (Tokio)

For async runtimes (such as `tokio`), `WaitingModal` can be driven from an async task loop using an `mpsc` channel for status updates:

```rust
use modalx::prelude::*;
use std::time::Duration;
use tokio::sync::mpsc;

pub enum ProgressUpdate {
    Message(String),
    StepCompleted(usize),
    Finished,
}

pub async fn run_task_with_spinner(
    mut rx: mpsc::Receiver<ProgressUpdate>,
) -> modalx::Result<()> {
    let mut stdout = std::io::stdout();
    let mut modal = WaitingModal::new("DEPLOYING SERVICE", "Initializing...")
        .with_step("Provision container", false)
        .with_step("Attach volumes", false)
        .with_step("Start health check", false);

    let mut frame = 0;
    loop {
        // Check for incoming progress updates without blocking
        while let Ok(update) = rx.try_recv() {
            match update {
                ProgressUpdate::Message(msg) => modal.message = msg,
                ProgressUpdate::StepCompleted(idx) => {
                    if let Some(step) = modal.steps.get_mut(idx) {
                        step.1 = true;
                    }
                }
                ProgressUpdate::Finished => {
                    modal.render_static(&mut stdout)?;
                    return Ok(());
                }
            }
        }

        modal.render_spinner(frame, &mut stdout)?;
        frame = frame.wrapping_add(1);
        tokio::time::sleep(Duration::from_millis(80)).await;
    }
}
```

---

## Static Rendering with `render_boxed_status`

When you want to display a stationary boxed status card without spinner animation (for example, during long batch outputs), use `render_boxed_status`:

```rust
render_boxed_status(
    "MAINTENANCE MODE",
    "Server is currently undergoing scheduled maintenance. Please wait...",
    &mut stdout,
)?;
```
