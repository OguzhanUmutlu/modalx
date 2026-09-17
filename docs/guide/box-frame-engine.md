# BoxFrame Layout Engine

The `BoxFrame` layout engine is the geometric foundation of `modalx`. It guarantees that visual content is strictly bound within structured borders, horizontally and vertically centered, and responsive across varying terminal dimensions.

---

## Geometric Dimensions and Width Clamping

Terminal windows vary dramatically from 80x24 retro VT100 dimensions to expansive ultra-wide monitors. A fixed width looks cramped on wide screens or overflows on narrow ones; a naive 100% width stretches reading lines excessively.

`BoxFrame` applies an adaptive bounding formula:

```rust
pub fn get_content_width(
    term_width: u16,
    percent: f32,
    min_width: u16,
    max_width: u16,
) -> usize {
    let raw = (term_width as f32 * percent).round() as u16;
    let clamped = raw.clamp(min_width, max_width);
    clamped.min(term_width.saturating_sub(4)) as usize
}
```

### Width Characteristics
- **Standard Modals**: Scale proportionally to roughly 80% to 90% of terminal width, clamped between a minimum of 60 columns and a maximum of 140 columns.
- **Compact Dialogs** (e.g. `ConfirmModal`): Use tighter maximum bounds (typically 64 to 76 columns) so focused confirmation questions do not feel sparse.
- **Safety Margin**: Always reserves at least 4 columns (2 character margin on left and right) so outer borders do not collide with the physical edges of the terminal window.

---

## Border Characters and Theme Sets

`BoxFrame` uses rounded Unicode box-drawing characters by default, with automatic detection and fallback to standard ASCII when UTF-8 terminal support is not present.

| Component | Unicode UTF-8 | ASCII Fallback |
| :--- | :--- | :--- |
| Top-Left Corner | `╭` | `+` |
| Top-Right Corner | `╮` | `+` |
| Horizontal Border | `─` | `-` |
| Vertical Border | `│` | `\|` |
| Left Divider Junction | `├` | `+` |
| Right Divider Junction | `┤` | `+` |
| Bottom-Left Corner | `╰` | `+` |
| Bottom-Right Corner | `╯` | `+` |

### Internal Line Formatting

Every line inside a `BoxFrame` is constructed with strict cell accounting:

```
│  <left_padding><content><right_padding>  │
```

Visual width is computed using `visible_len()`, which strips ANSI SGR formatting sequences (colors, bold, underline) before measuring character length. This prevents color codes from artificially inflating calculated widths and breaking border alignment.

---

## Vertical Centering

By default, standard terminal tools render content starting at row 0 (top-left). For dialogs, vertical centering dramatically improves readability and user focus.

`BoxFrame` achieves vertical centering by calculating:

$$\text{top\_offset} = \left\lfloor \frac{\text{terminal\_rows} - \text{frame\_height}}{2} \right\rfloor$$

```
+-------------------------------------------------------------+
|                      Terminal Window                        |
|                                                             |
|                    (top offset empty rows)                  |
|                                                             |
|           ╭──────────────────────────────────────╮          |
|           │             BOX FRAME                │          |
|           │         Vertically Centered          │          |
|           ╰──────────────────────────────────────╯          |
|                                                             |
|                   (bottom offset empty rows)                |
|                                                             |
+-------------------------------------------------------------+
```

When vertical centering is active, rows outside the box are cleared using terminal line-clearing sequences (`\x1b[2K`), preventing visual debris from previous screen contents.

---

## Viewport Constraints and Protection

When a user resizes their terminal window below viable limits (for example, shrinking the window below 60 columns or 14 rows), rendering a full modal would cause clipping and illegibility.

`modalx` enforces minimum constraints:
- `MIN_TERM_WIDTH`: 60 columns.
- `MIN_TERM_HEIGHT`: 14 rows.

### The "Terminal Too Small" Guard

If the viewport drops below these thresholds, `is_terminal_too_small(width, height)` triggers. The modal suspends normal rendering and draws a compact warning card:

```
╭──────────────────────────────────────────────────────────╮
│                    TERMINAL TOO SMALL                    │
├──────────────────────────────────────────────────────────┤
│  Current: 52x11  |  Required: 60x14                      │
│                                                          │
│  Please resize your terminal window to continue...       │
╰──────────────────────────────────────────────────────────╯
```

The modal enters `wait_for_valid_size()`:
1. It listens exclusively for `Event::Resize(w, h)` events.
2. It blocks user interaction and pauses background animations.
3. As soon as the terminal is expanded to or above 60x14, execution resumes immediately, and the full modal frame is cleanly redrawn.
