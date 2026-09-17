# Cookbook: Interactive Setup Wizard

This cookbook demonstrates how to build a guided, multi-step application setup wizard using `modalx`. It chains `InputModal`, `SelectModal`, and `ConfirmModal` while automatically updating breadcrumb trails via `NavGuard`.

---

## What We Are Building

A 3-step project initialization workflow:
1. **Step 1**: Prompt for project name with real-time validation.
2. **Step 2**: Select target runtime environment from a menu.
3. **Step 3**: Review configuration and confirm initialization.

---

## Complete Implementation

```rust
use modalx::prelude::*;

#[derive(Debug)]
pub struct ProjectConfig {
    pub name: String,
    pub runtime: String,
}

fn main() -> modalx::Result<()> {
    // Preserve terminal buffer across all wizard steps
    let _alt = AltScreenGuard::enter();

    let _root_guard = NavGuard::new("Setup Wizard");

    // ---------------------------------------------------------
    // Step 1: Project Name
    // ---------------------------------------------------------
    let _step1_guard = NavGuard::new("Step 1: Name");
    let name_outcome = InputModal::new(
        "PROJECT SETUP WIZARD (1/3)",
        "Enter application name:"
    )
    .with_placeholder("my-microservice")
    .with_validator(|val| {
        let trimmed = val.trim();
        if trimmed.is_empty() {
            Err("Project name cannot be empty".to_string())
        } else if trimmed.contains(' ') {
            Err("Project name cannot contain spaces".to_string())
        } else if trimmed.chars().any(|c| !c.is_ascii_alphanumeric() && c != '-' && c != '_') {
            Err("Only alphanumeric characters, hyphens, and underscores are allowed".to_string())
        } else {
            Ok(())
        }
    })
    .run()?;

    let project_name = match name_outcome {
        InputOutcome::Submitted(name) => name,
        InputOutcome::Cancelled => {
            println!("Setup cancelled by user.");
            return Ok(());
        }
    };
    drop(_step1_guard);

    // ---------------------------------------------------------
    // Step 2: Runtime Selection
    // ---------------------------------------------------------
    let _step2_guard = NavGuard::new("Step 2: Runtime");
    let mut runtime_idx = 0;
    let runtimes = ["Rust (tokio + axum)", "Go (standard library)", "Node.js (TypeScript)", "Python (FastAPI)"];

    let runtime_outcome = SelectModal::new()
        .with_title("PROJECT SETUP WIZARD (2/3)", false)
        .with_header_row(format!("Target Project: {}", project_name))
        .item("1", runtimes[0])
        .item("2", runtimes[1])
        .item("3", runtimes[2])
        .item("4", runtimes[3])
        .run(&mut runtime_idx)?;

    if runtime_outcome == SelectOutcome::Cancelled {
        println!("Setup cancelled by user.");
        return Ok(());
    }
    let chosen_runtime = runtimes[runtime_idx].to_string();
    drop(_step2_guard);

    // ---------------------------------------------------------
    // Step 3: Confirmation
    // ---------------------------------------------------------
    let _step3_guard = NavGuard::new("Step 3: Confirm");
    let confirm_outcome = ConfirmModal::new(
        "PROJECT SETUP WIZARD (3/3)",
        format!("Initialize project '{}' with runtime '{}'?", project_name, chosen_runtime)
    )
    .with_detail("This will create scaffolding directories, git configuration, and dependency manifests.")
    .with_yes_label("Initialize Project")
    .with_no_label("Cancel")
    .default_yes(true)
    .run()?;

    if confirm_outcome == ConfirmOutcome::Confirmed {
        let config = ProjectConfig {
            name: project_name,
            runtime: chosen_runtime,
        };
        println!("Project successfully created: {:?}", config);
    } else {
        println!("Setup aborted.");
    }

    Ok(())
}
```

---

## Architectural Notes

1. **Alternate Screen Scoping**: `AltScreenGuard::enter()` is called once at the start of `main()`. This prevents the terminal from flickering back to the shell between wizard steps.
2. **Deterministic RAII Breadcrumbs**: Scoped `NavGuard` instances automatically update the breadcrumb header (`Setup Wizard › Step 1: Name`) and clean up when dropped.
3. **Early Return on Cancellation**: Every step checks for cancellation (`Cancelled`), allowing the user to press `Esc` at any point to exit cleanly.
