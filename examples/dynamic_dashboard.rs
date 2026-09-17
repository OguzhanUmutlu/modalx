use modalx::prelude::*;

fn main() -> modalx::Result<()> {
    let mut selected = 0;

    let fields = FieldSection::new()
        .with_separator(" | ")
        .add_field("Host: Ubuntu-Workstation")
        .add_field("RAM: 14.2 / 32.0 GB (44.3%)")
        .add_field("Status: [HEALTHY]")
        .add_field("Uptime: 14d 6h 32m");

    let modal = SelectModal::new()
        .with_title("CLOUD INFRASTRUCTURE DASHBOARD", false)
        .with_section(ModalSection::Fields(fields))
        .with_section(ModalSection::Fields(
            FieldSection::new()
                .with_separator(" | ")
                .add_field("Active Services: 18")
                .add_field("Alerts: 0")
                .add_field("Network: 1.2 Gbps"),
        ))
        .item("1", "Virtual Machines")
        .item("2", "Kubernetes Clusters")
        .item("3", "Persistent Volumes")
        .item("4", "Security Groups & Firewall")
        .item("5", "Audit & Access Logs")
        .item("6", "Billing & Usage");

    match modal.run(&mut selected)? {
        SelectOutcome::Selected(idx) => {
            println!("Navigating to section index: {}", idx);
        }
        SelectOutcome::Cancelled => {
            println!("Dashboard closed.");
        }
        _ => {}
    }

    Ok(())
}
