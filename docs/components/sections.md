# Declarative Sections

Complex terminal interfaces often combine multiple disparate visual blocks: title banners, status grids, item menus, and footer shortcut bars. `modalx` models these elements as composable `ModalSection` blocks.

---

## The `ModalSection` Architecture

A modal's content area is constructed from a sequence of strongly-typed sections:

```rust
pub enum ModalSection {
    Title(TitleSection),
    Fields(FieldSection),
    Text(TextSection),
    Menu(MenuSection),
    Footer(FooterSection),
}
```

Sections can be chained together in any order. The layout engine renders divider lines (`├─┤`) between distinct sections, maintaining structured visual encapsulation.

---

## Available Sections

### 1. `TitleSection`
Renders the top banner of the modal:

```rust
let title = TitleSection::new("INFRASTRUCTURE DASHBOARD")
    .with_main_menu(true)
    .with_subtitle("Active Region: us-east-1");
```

### 2. `FieldSection`
Renders key-value metadata with greedy line reflow:

```rust
let fields = FieldSection::new()
    .with_separator(" | ")
    .add_field("Host: worker-01")
    .add_field("RAM: 14.2 / 32 GB")
    .add_field("Status: [HEALTHY]");
```

When the terminal window narrows, `FieldSection` automatically distributes these fields across multiple neatly aligned rows rather than truncating them.

### 3. `TextSection`
Renders arbitrary multi-line explanatory or helper text:

```rust
let text = TextSection::new(
    "Select a virtual machine instance from the list below to inspect live \
     telemetry, restart background services, or view error logs."
);
```

### 4. `MenuSection`
Holds selectable list entries with hotkey mappings and descriptions:

```rust
let menu = MenuSection::new()
    .item("1", "Node Pools")
    .item("2", "Security Groups")
    .item("q", "Exit");
```

### 5. `FooterSection`
Renders the bottom keybinding bar:

```rust
let footer = FooterSection::from_shortcuts(
    Shortcuts::new()
        .add(Shortcut::move_selection())
        .add(Shortcut::select())
        .add(Shortcut::exit())
);
```

---

## Composing Custom Dashboards

```rust
use modalx::prelude::*;

fn main() -> modalx::Result<()> {
    let mut selected = 0;

    let modal = SelectModal::new()
        .with_title("KUBERNETES CLUSTER DASHBOARD", false)
        .with_section(ModalSection::Fields(
            FieldSection::new()
                .with_separator(" | ")
                .add_field("Cluster: prod-k8s-01")
                .add_field("Nodes: 24 Ready")
                .add_field("Pods: 142/150"),
        ))
        .with_section(ModalSection::Fields(
            FieldSection::new()
                .with_separator(" | ")
                .add_field("CPU: 38.4%")
                .add_field("Memory: 64.2%")
                .add_field("Network: 4.8 Gbps"),
        ))
        .item("1", "View Pod Status")
        .item("2", "Scale Deployments")
        .item("3", "Cluster Events & Warnings")
        .item("q", "Exit");

    modal.run(&mut selected)?;
    Ok(())
}
```
