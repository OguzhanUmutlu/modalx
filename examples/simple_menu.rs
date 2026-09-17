use modalx::prelude::*;

fn main() -> modalx::Result<()> {
    let mut selected = 0;

    let modal = SelectModal::new()
        .with_title("APPLICATION MENU", false)
        .with_raw_fields("Environment: Production | Region: us-east-1", " | ")
        .item("1", "Manage Databases")
        .item("2", "View Cluster Metrics")
        .item("3", "Deploy Applications")
        .item("4", "System Preferences")
        .item("q", "Exit Application");

    match modal.run(&mut selected)? {
        SelectOutcome::Selected(idx) => {
            println!("Selected entry index: {}", idx);
        }
        SelectOutcome::Cancelled => {
            println!("Action cancelled by user.");
        }
        _ => {}
    }

    Ok(())
}
