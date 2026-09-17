# ConfirmModal

`ConfirmModal` provides a binary affirmative/negative confirmation prompt. It features horizontal centering of text and action buttons, danger styling for irreversible operations, configurable default focus, and direct keyboard shortcuts.

---

## Visual Presentation

```
╭────────────────────────────────────────────────────────────╮
│                       CONFIRMATION                         │
├────────────────────────────────────────────────────────────┤
│                                                            │
│     Terminate the Craft background daemon process?         │
│                                                            │
│       Active server instances will continue running        │
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
        "STOP SERVER",
        "Are you sure you want to stop 'survival_prod'?"
    )
    .with_detail("Connected players will be safely disconnected.")
    .with_yes_label("Stop Server")
    .with_no_label("Keep Running")
    .default_yes(true)
    .run()?;

    if outcome == ConfirmOutcome::Confirmed {
        println!("Stopping server...");
    } else {
        println!("Server remains running.");
    }

    Ok(())
}
```

---

## Danger Mode for Destructive Actions

When an action is irreversible (such as deleting databases, purging backups, or formatting storage pools), enabling `.danger(true)` modifies the visual presentation:
- The affirmative button is highlighted in vivid red.
- Focus defaults safely to the negative button (`[ Cancel ]`).
- Pressing `y` directly requires conscious confirmation.

```rust
let outcome = ConfirmModal::new(
    "DELETE ALL BACKUPS",
    "Permanently delete all 24 local backups for 'lobby_server'?"
)
.danger(true)
.with_detail("This action cannot be undone. All compressed archives will be purged.")
.with_yes_label("Delete All Backups")
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
