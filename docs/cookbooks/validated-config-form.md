# Cookbook: Validated Configuration Form

This cookbook demonstrates how to build an enterprise-grade configuration form using `FormModal`. It features real-time validation for network ports and IP addresses, keystroke-level filtering, masked password fields, and exit protection.

---

## What We Are Building

A remote database connection dialog collecting:
- Server Label (`String`, required, alphanumeric + hyphens).
- IP Address or Hostname (`String`, validated for valid format).
- Database Port (`Integer`, strictly numeric keystrokes, validated range 1..65535).
- Database Name (`String`, default `production`).
- Master Password (`Password`, masked input).

---

## Complete Implementation

```rust
use modalx::prelude::*;
use std::net::IpAddr;

#[derive(Debug)]
pub struct DatabaseCredentials {
    pub label: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub password: String,
}

fn main() -> modalx::Result<()> {
    let _alt = AltScreenGuard::enter();
    let _nav = NavGuard::new("Database Settings");

    let form = FormModal::new("CONFIGURE DATABASE CONNECTION")
        .with_header("Enter primary PostgreSQL connection details:")
        .field(
            FormField::string("label", "Connection Label")
                .with_default("prod-db-primary")
                .with_validator(|val| {
                    if val.trim().is_empty() {
                        Err("Label cannot be empty".to_string())
                    } else {
                        Ok(())
                    }
                }),
        )
        .field(
            FormField::string("host", "Host IP or FQDN")
                .with_placeholder("10.0.0.15 or db.internal")
                .with_validator(|val| {
                    let trimmed = val.trim();
                    if trimmed.is_empty() {
                        return Err("Host address is required".to_string());
                    }
                    // Validate as IP address or valid domain
                    if trimmed.parse::<IpAddr>().is_err() && !trimmed.contains('.') {
                        return Err("Must be a valid IPv4/IPv6 address or qualified domain name".to_string());
                    }
                    Ok(())
                }),
        )
        .field(
            FormField::integer("port", "Port")
                .with_default("5432")
                .with_validator(|val| match val.parse::<u16>() {
                    Ok(p) if p > 0 => Ok(()),
                    _ => Err("Port must be an integer between 1 and 65535".to_string()),
                }),
        )
        .field(
            FormField::string("database", "Database Name")
                .with_default("app_production"),
        )
        .field(
            FormField::password("password", "Master Password")
                .with_validator(|val| {
                    if val.len() < 8 {
                        Err("Password must contain at least 8 characters".to_string())
                    } else {
                        Ok(())
                    }
                }),
        );

    match form.run()? {
        FormResult::Submitted(values) => {
            let credentials = DatabaseCredentials {
                label: values["label"].clone(),
                host: values["host"].clone(),
                port: values["port"].parse::<u16>().unwrap_or(5432),
                database: values["database"].clone(),
                password: values["password"].clone(),
            };

            println!("Database connection configured successfully:");
            println!("  Label:    {}", credentials.label);
            println!("  Endpoint: {}:{}", credentials.host, credentials.port);
            println!("  Database: {}", credentials.database);
            println!("  Secret:   {} characters", credentials.password.len());
        }
        FormResult::Cancelled => {
            println!("Configuration cancelled.");
        }
    }

    Ok(())
}
```

---

## Key Validations in Action

1. **Keystroke Filtering**: Because the `port` field is defined using `FormField::integer`, non-digit keys (letters, symbols) are silently blocked at the keystroke level by the built-in `force_validator`.
2. **Dynamic Error Banners**: If a user enters a port number exceeding 65535 (e.g. `70000`), the validator immediately renders:
   ```
   ✗ Port must be an integer between 1 and 65535
   ```
   and prevents form confirmation until corrected.
3. **Password Confidentiality**: Keystrokes in the password field render as `*` on screen, ensuring bystanders cannot view the password, while the submitted map receives the intact raw string.
4. **Unsaved Edits Confirmation**: If any field was modified and the user presses `Esc`, a confirmation dialog safeguards against accidental input loss.
