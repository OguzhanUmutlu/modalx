use modalx::prelude::*;

fn main() -> modalx::Result<()> {
    let _alt = AltScreenGuard::enter();

    // Step 1: Input Modal
    let name_outcome =
        InputModal::new("PROJECT SETUP WIZARD (STEP 1/3)", "Enter application name:")
            .with_placeholder("my-awesome-app")
            .with_validator(|val| {
                if val.trim().is_empty() {
                    Err("Application name cannot be empty".to_string())
                } else if val.contains(' ') {
                    Err("Application name cannot contain spaces".to_string())
                } else {
                    Ok(())
                }
            })
            .run()?;

    let app_name = match name_outcome {
        InputOutcome::Submitted(name) => name,
        InputOutcome::Cancelled => {
            println!("Setup cancelled.");
            return Ok(());
        }
    };

    // Step 2: Select Modal
    let mut runtime_idx = 0;
    let runtime_outcome = SelectModal::new()
        .with_title("PROJECT SETUP WIZARD (STEP 2/3)", false)
        .with_header_row(format!("Target Name: {}", app_name))
        .item("1", "Rust (Cargo binary)")
        .item("2", "Node.js (TypeScript)")
        .item("3", "Go (Standard modules)")
        .item("4", "Python (FastAPI)")
        .run(&mut runtime_idx)?;

    if runtime_outcome == SelectOutcome::Cancelled {
        println!("Setup cancelled.");
        return Ok(());
    }

    // Step 3: Confirmation Modal
    let confirm = ConfirmModal::new(
        "PROJECT SETUP WIZARD (STEP 3/3)",
        format!(
            "Initialize project '{}' in the current directory?",
            app_name
        ),
    )
    .with_detail("This will create scaffolding directories and configuration files.")
    .with_yes_label("Create Project")
    .with_no_label("Cancel")
    .default_yes(true)
    .run()?;

    if confirm == ConfirmOutcome::Confirmed {
        println!("Project '{}' successfully initialized!", app_name);
    } else {
        println!("Setup aborted.");
    }

    Ok(())
}
