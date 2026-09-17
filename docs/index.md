---
layout: home

hero:
  name: "modalx"
  text: "Box-Encapsulated Terminal UI Framework for Rust"
  tagline: "Declarative, responsive, zero-flicker modal dialogs and structured terminal interfaces."
  image:
    src: /logo.svg
    alt: modalx logo
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
│                     WORKSPACE & PIPELINE CONTROLLER                          │
├──────────────────────────────────────────────────────────────────────────────┤
│  Project: nexus-core | Branch: feat/async-worker | Target: x86_64-musl       │
│  Environment: Staging | Health: Nominal (99.9%) | Active Workers: 8          │
├──────────────────────────────────────────────────────────────────────────────┤
│  > [1]   Run Build & Verification Pipeline                                   │
│    [2]   Interactive Test Suite (51 passed)                                  │
│    [3]   Database Schema Migrations                                          │
│    [4]   Deploy Staging Canary Artifact                                      │
│    [5]   Inspect Telemetry & Real-Time Logs                                  │
│    [6]   Workspace Configuration & Secrets                                   │
├──────────────────────────────────────────────────────────────────────────────┤
│  [↑/↓/j/k] Navigate  |  [Enter] Execute  |  [Esc] Back  |  [q] Quit          │
╰──────────────────────────────────────────────────────────────────────────────╯
```
