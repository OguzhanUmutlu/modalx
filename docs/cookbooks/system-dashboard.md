# Cookbook: Cloud Infrastructure Dashboard

This cookbook demonstrates how to build an administrative dashboard displaying cluster metrics, service health, and actionable navigation menus using `modalx`'s declarative section system.

---

## What We Are Building

A multi-section dashboard displaying:
- Top banner with cluster identity and breadcrumbs.
- Two responsive metadata grids (hardware telemetry and container counts).
- An actionable menu navigating to sub-inspectors.
- Standardized footer shortcut bar.

---

## Complete Implementation

```rust
use modalx::prelude::*;

fn main() -> modalx::Result<()> {
    let _alt = AltScreenGuard::enter();
    let _nav = NavGuard::new("Dashboard");

    loop {
        let mut selected = 0;

        let hardware_fields = FieldSection::new()
            .with_separator(" | ")
            .add_field("Cluster: k8s-us-east-prod")
            .add_field("Nodes: 32 Ready")
            .add_field("CPU: 42.8%")
            .add_field("RAM: 128.4 / 256.0 GB (50.1%)");

        let service_fields = FieldSection::new()
            .with_separator(" | ")
            .add_field("Ingress: 4.2 Gbps")
            .add_field("Active Pods: 284")
            .add_field("Failed Deployments: 0")
            .add_field("Health: [NOMINAL]");

        let modal = SelectModal::new()
            .with_title("INFRASTRUCTURE CONTROL PLANE", true)
            .with_section(ModalSection::Fields(hardware_fields))
            .with_section(ModalSection::Fields(service_fields))
            .item("1", "Node Pool Status & Autoscaling")
            .item("2", "Deployments & Replicasets")
            .item("3", "Ingress Gateways & TLS Certificates")
            .item("4", "Cluster Events & Audit Logs")
            .item("5", "Run Diagnostic Health Check")
            .item("q", "Exit Dashboard");

        match modal.run(&mut selected)? {
            SelectOutcome::Selected(0) => inspect_nodes()?,
            SelectOutcome::Selected(1) => inspect_deployments()?,
            SelectOutcome::Selected(4) => run_health_check()?,
            SelectOutcome::Selected(5) | SelectOutcome::Cancelled => {
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

fn inspect_nodes() -> modalx::Result<()> {
    let _nav = NavGuard::new("Nodes");
    let columns = vec![
        TableColumn::new("Node Name", 24),
        TableColumn::new("Status", 12),
        TableColumn::new("CPU (%)", 10).right_aligned(),
        TableColumn::new("RAM (%)", 10).right_aligned(),
    ];
    let rows = vec![
        vec!["worker-pool-01-a".into(), "Ready".into(), "38.2".into(), "48.1".into()],
        vec!["worker-pool-01-b".into(), "Ready".into(), "44.0".into(), "52.3".into()],
        vec!["worker-pool-01-c".into(), "Ready".into(), "41.5".into(), "49.0".into()],
    ];
    let mut selected = 0;
    TableModal::new("NODE POOL STATUS")
        .with_columns(columns)
        .with_rows(rows)
        .run(&mut selected)?;
    Ok(())
}

fn inspect_deployments() -> modalx::Result<()> {
    let _nav = NavGuard::new("Deployments");
    let lines = vec![
        "Deployment: api-gateway          Replicas: 8/8    Status: UpToDate".to_string(),
        "Deployment: auth-service         Replicas: 4/4    Status: UpToDate".to_string(),
        "Deployment: order-processor      Replicas: 12/12  Status: UpToDate".to_string(),
        "Deployment: notification-worker  Replicas: 2/2    Status: UpToDate".to_string(),
    ];
    InfoModal::new("DEPLOYMENT OVERVIEW", lines).run()?;
    Ok(())
}

fn run_health_check() -> modalx::Result<()> {
    let _nav = NavGuard::new("Diagnostics");
    let mut stdout = std::io::stdout();
    let mut modal = WaitingModal::new("DIAGNOSTIC HEALTH CHECK", "Querying cluster endpoints...")
        .with_step("Verify API server reachability", false)
        .with_step("Check etcd quorum and leader election", false)
        .with_step("Audit kubelet node status", false);

    for i in 0..30 {
        modal.render_spinner(i, &mut stdout)?;
        std::thread::sleep(std::time::Duration::from_millis(60));
    }
    modal.steps[0].1 = true;

    for i in 30..60 {
        modal.render_spinner(i, &mut stdout)?;
        std::thread::sleep(std::time::Duration::from_millis(60));
    }
    modal.steps[1].1 = true;

    for i in 60..90 {
        modal.render_spinner(i, &mut stdout)?;
        std::thread::sleep(std::time::Duration::from_millis(60));
    }
    modal.steps[2].1 = true;
    modal.message = "All diagnostic checks passed successfully!".to_string();
    modal.render_static(&mut stdout)?;
    std::thread::sleep(std::time::Duration::from_millis(800));
    Ok(())
}
```
