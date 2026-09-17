---
layout: home

hero:
  name: "modalx"
  text: "Box-Encapsulated Terminal UI Framework for Rust"
  tagline: "Declarative, responsive, zero-flicker modal dialogs and structured terminal interfaces."
  actions:
    - theme: brand
      text: Get Started
      link: /guide/getting-started
    - theme: alt
      text: Explore Modals
      link: /modals/overview
    - theme: alt
      text: View on GitHub
      link: https://github.com/OguzhanUmutlu/modalx

features:
  - title: Strict Box Geometry
    details: Complete enclosure within Unicode border matrices with automatic ASCII fallbacks. Text reflows cleanly without border breaking or visual overflow.
  - title: Zero-Flicker Terminal Engine
    details: Atomic in-memory double-buffering flushed in single stdout writes. Automatic RAII cleanup hooks for panics and abnormal termination.
  - title: Dynamic Metadata Reflow
    details: Delimited key-value metadata reflows greedily across multiple framed rows on narrow terminals rather than truncating with ellipses.
  - title: Comprehensive Modal Library
    details: Ready-to-use Select, Multi-Field Form, Readline Input, Confirm, Table, Info, and Async Waiting modals.
  - title: Context-Aware Shortcuts
    details: Dynamic shortcut bar builder that automatically evaluates viewport bounds and omits scroll hints when content fits without scrolling.
  - title: Responsive Viewport Guard
    details: Automatically detects terminals smaller than 60x14, presents a centered alert card, and cleanly pauses until resize events arrive.
---

```
╭──────────────────────────────────────────────────────────────────────────────╮
│                        CLOUD INFRASTRUCTURE DASHBOARD                        │
├──────────────────────────────────────────────────────────────────────────────┤
│  Host: Ubuntu-Workstation | RAM: 14.2 / 32.0 GB (44.3%) | Status: [HEALTHY]  │
│  Active Services: 18 | Alerts: 0 | Network: 1.2 Gbps                         │
├──────────────────────────────────────────────────────────────────────────────┤
│  > [1]   Virtual Machines                                                    │
│    [2]   Kubernetes Clusters                                                 │
│    [3]   Persistent Volumes                                                  │
│    [4]   Security Groups & Firewall                                          │
│    [5]   Audit & Access Logs                                                 │
│    [6]   Billing & Usage                                                     │
├──────────────────────────────────────────────────────────────────────────────┤
│  [↑/↓/j/k] Move  |  [Enter/→] Select  |  [Esc/←] Back  |  [q] Exit           │
╰──────────────────────────────────────────────────────────────────────────────╯
```
