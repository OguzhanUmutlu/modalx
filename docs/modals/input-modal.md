# InputModal

`InputModal` provides a focused, single-line text prompt dialog. It includes full readline editing capabilities, cursor positioning, word deletion shortcuts, dynamic validation, and optional password masking.

---

## Visual Presentation

```
╭────────────────────────────────────────────────────────────╮
│                    NEW SERVICE SETUP                       │
├────────────────────────────────────────────────────────────┤
│  Enter a name for the new backend service:                 │
│                                                            │
│  [ auth-service-2026_                                    ] │
│                                                            │
│  ✗ Service name cannot contain uppercase characters        │
├────────────────────────────────────────────────────────────┤
│  [Enter] Submit  |  [Esc] Cancel  |  [Ctrl+W] Delete Word  │
╰────────────────────────────────────────────────────────────╯
```

---

## Basic Usage

```rust
use modalx::prelude::*;

fn main() -> modalx::Result<()> {
    let outcome = InputModal::new("PROJECT INITIALIZATION", "Enter project name:")
        .with_placeholder("my-service")
        .with_validator(|input| {
            let trimmed = input.trim();
            if trimmed.is_empty() {
                Err("Project name cannot be empty".to_string())
            } else if trimmed.contains(' ') {
                Err("Project name cannot contain spaces".to_string())
            } else {
                Ok(())
            }
        })
        .run()?;

    match outcome {
        InputOutcome::Submitted(name) => {
            println!("Creating project: {}", name);
        }
        InputOutcome::Cancelled => {
            println!("Setup cancelled.");
        }
    }

    Ok(())
}
```

---

## Readline Editing Keybindings

`InputModal` provides full standard Unix readline text editing behavior:

| Key Binding | Action |
| :--- | :--- |
| `Left` / `Right` | Move cursor one character left or right |
| `Home` / `Ctrl+A` | Move cursor to the beginning of the text line |
| `End` / `Ctrl+E` | Move cursor to the end of the text line |
| `Ctrl+Left` | Jump cursor back by one word |
| `Ctrl+Right` | Jump cursor forward by one word |
| `Backspace` | Delete character immediately preceding cursor |
| `Delete` | Delete character immediately under cursor |
| `Ctrl+W` | Delete preceding word up to whitespace or delimiter |
| `Ctrl+U` | Clear the entire input buffer |
| `Enter` | Validate and submit current input buffer |
| `Esc` | Cancel prompt and return `InputOutcome::Cancelled` |

---

## Real-Time Validation

Validators are closures conforming to:

```rust
Fn(&str) -> std::result::Result<(), String>
```

When a validator returns `Err(msg)`:
- The error message is rendered directly below the input field inside a formatted error container.
- Pressing `Enter` will not submit the modal; the cursor remains active so the user can fix the input immediately.
- As the user types characters that satisfy the validation rules, the error banner disappears automatically.

---

## Sensitive Input Masking

To collect credentials or API tokens with character masking:

```rust
let outcome = InputModal::new("AUTHENTICATION", "Enter API secret key:")
    .with_password(true)
    .run()?;
```

When password mode is enabled, characters are displayed on screen as asterisks (`*`), but the submitted `String` contains the raw plaintext value.
