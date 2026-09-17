# ConfirmModal

`ConfirmModal` provides a binary affirmative/negative confirmation prompt. It features horizontal centering of text and action buttons, danger styling for irreversible operations, configurable default focus, and direct keyboard shortcuts.

---

## Visual Presentation

```
╭────────────────────────────────────────────────────────────╮
│                       CONFIRMATION                         │
├────────────────────────────────────────────────────────────┤
│                                                            │
│     Terminate the background daemon worker process?        │
│                                                            │
│       Active task jobs will continue running               │
│       in the background without health monitoring.         │
│                                                            │
│               [ Yes, Terminate ]     [ Cancel ]            │
│                                                            │
├────────────────────────────────────────────────────────────┤
│  [←/→/Tab] Select  |  [Enter] Confirm  |  [y/n]  |  [Esc]  │
╰────────────────────────────────────────────────────────────╯
```

---

## Basic Usage

```rust
use modalx::prelude::*;

fn main() -> modalx::Result<()> {
    let outcome = ConfirmModal::new(
        "TERMINATE WORKER",
        "Are you sure you want to stop background worker 'worker-01'?"
    )
    .with_detail("In-flight tasks will complete before shutdown.")
    .with_yes_label("Stop Worker")
    .with_no_label("Keep Running")
    .default_yes(true)
    .run()?;

    if outcome == ConfirmOutcome::Confirmed {
        println!("Stopping worker...");
    } else {
        println!("Worker remains running.");
    }

    Ok(())
}
```

---

## Danger Mode for Destructive Actions

When an action is irreversible (such as deleting databases, purging volumes, or dropping tables), enabling `.danger(true)` modifies the visual presentation:
- The affirmative button is highlighted in vivid red.
- Focus defaults safely to the negative button (`[ Cancel ]`).
- Pressing `y` directly requires conscious confirmation.

```rust
let outcome = ConfirmModal::new(
    "PURGE VOLUME",
    "Permanently remove persistent volume 'vol-data-01'?"
)
.danger(true)
.with_detail("This action cannot be undone. All database records will be erased.")
.with_yes_label("Purge Volume")
.with_no_label("Cancel")
.default_yes(false) // Safe default
.run()?;
```

---

## Keyboard Controls

`ConfirmModal` allows rapid keyboard navigation:

- **Arrow Keys & Tab**: `Left`, `Right`, `Tab`, `Shift+Tab`, or `h`/`l` toggle focus between the affirmative and negative buttons.
- **Direct Hotkeys**:
  - Pressing `y` or `Y` immediately selects affirmative and confirms.
  - Pressing `n` or `N` immediately selects negative and cancels.
- **Enter & Space**: Activates whichever button currently holds focus.
- **Esc**: Immediately cancels and returns `ConfirmOutcome::Cancelled`.
