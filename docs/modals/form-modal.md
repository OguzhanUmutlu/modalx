# FormModal

`FormModal` provides a multi-field data entry form. It features real-time validation, keystroke-level input filtering, masked password fields, semantic field types (integer, numeric, string, password), and accidental cancellation protection.

---

## Visual Presentation

```
╭──────────────────────────────────────────────────────────────────────────────╮
│                    MICROSERVICE GATEWAY CREDENTIALS                          │
│             Settings › Cluster Connection › prod-gateway › Edit              │
├──────────────────────────────────────────────────────────────────────────────┤
│  Configure API gateway connection parameters and credentials:                │
│                                                                              │
│  Label:                                                                      │
│  [ prod-gateway-primary                                       ]              │
│                                                                              │
│  Host:                                                                       │
│  [ 10.0.12.45                                                 ]              │
│                                                                              │
│  Port:                                                                       │
│  [ 8080                                                       ]              │
│                                                                              │
│  Password / Secret Token:                                                    │
│  [ ******************                                         ]              │
│                                                                              │
│                       [ Save ]         [ Cancel ]                            │
├──────────────────────────────────────────────────────────────────────────────┤
│  [Tab/Shift+Tab] Next/Prev  |  [↑/↓] Fields  |  [Enter] Confirm  |  [Esc]    │
╰──────────────────────────────────────────────────────────────────────────────╯
```

---

## Basic Usage

```rust
use modalx::prelude::*;

fn main() -> modalx::Result<()> {
    let modal = FormModal::new("HOST CONFIGURATION")
        .with_header("Specify network details for remote host:")
        .field(FormField::string("label", "Label").with_default("my-server"))
        .field(FormField::string("host", "Hostname or IP").with_placeholder("e.g. 192.168.1.50"))
        .field(
            FormField::integer("port", "Port")
                .with_default("22")
                .with_validator(|val| match val.parse::<u16>() {
                    Ok(p) if p > 0 => Ok(()),
                    _ => Err("Port must be between 1 and 65535".to_string()),
                }),
        )
        .field(FormField::string("username", "SSH Username").with_default("root"))
        .field(FormField::password("password", "Password / Passphrase"));

    match modal.run()? {
        FormResult::Submitted(values) => {
            println!("Host: {}:{}", values["host"], values["port"]);
            println!("User: {}", values["username"]);
            // Passwords retain their raw unmasked value in the output map
            println!("Password length: {} characters", values["password"].len());
        }
        FormResult::Cancelled => {
            println!("Form cancelled.");
        }
    }

    Ok(())
}
```

---

## Semantic Field Types

`modalx` provides specialized constructors for common field categories via `FormFieldType`:

### 1. String (`FormField::string`)
General-purpose text input. Supports arbitrary Unicode characters and standard text editing.

### 2. Integer (`FormField::integer`)
Whole integer input. Automatically attaches a `force_validator` that rejects non-numeric keystrokes in real time (allowing an optional leading `-`). Users physically cannot type letters or invalid characters into an integer field.

### 3. Number (`FormField::number`)
Floating-point numeric input. Allows digits, an optional leading `-`, and at most one decimal point (`.`).

### 4. Password (`FormField::password`)
Sensitive input. Attaches a display formatter that renders each character as an asterisk (`*`) while preserving the raw string in the field value.

---

## Validation Pipeline

`FormModal` features a two-tiered validation pipeline:

### 1. Force Validators (`FieldForceValidator`)
A force validator intercepts keystrokes before they enter the field buffer:

$$\text{force\_validator}: (\text{old\_val}, \text{new\_val}) \to \text{filtered\_val}$$

If the user attempts to paste or type invalid characters, the force validator rejects the modification and retains the previous valid string.

### 2. Form Validators (`FieldValidator`)
A validator verifies the semantic correctness of the entire field value:

$$\text{validator}: (\&str) \to \text{Result<(), String>}$$

When a validator returns `Err(message)`, the modal displays a highlighted error banner directly underneath the field and prevents form submission until the issue is resolved.

```rust
FormField::string("email", "Email Address")
    .with_validator(|val| {
        if val.contains('@') && val.contains('.') {
            Ok(())
        } else {
            Err("Please enter a valid email address".to_string())
        }
    })
```

---

## Navigation & Unsaved Changes Guard

- **Cycling Focus**: Pressing `Tab` or `Down` advances focus to the next field. When the last field is reached, focus moves to the `[ Save ]` and `[ Cancel ]` buttons.
- **Reverse Cycling**: Pressing `Shift+Tab` or `Up` moves focus backwards.
- **Field Advancement**: Pressing `Enter` in an active text field automatically advances cursor focus to the next field.
- **Escape Protection**: If the user modifies any field and presses `Esc`, `FormModal` intercepts the event and prompts a `ConfirmModal` ("Discard unsaved changes?"). This prevents losing extensive configuration data to an accidental keystroke.
