# API Reference Index

This page provides a reference index of all primary modules, types, functions, and constants exported by `modalx`.

---

## Crate Modules

| Module | Description |
| :--- | :--- |
| [`modalx::error`](#error) | Custom error types and Result aliases |
| [`modalx::frame`](#frame) | BoxFrame engine, line buffers, and geometry |
| [`modalx::keys`](#keys) | Keyboard mappings, action translations, and shortcuts |
| [`modalx::modals`](#modals) | Built-in modal implementations (Select, Form, Input, Confirm, Table, Info, Waiting) |
| [`modalx::nav`](#nav) | Breadcrumb navigation stack and RAII NavGuard |
| [`modalx::section`](#section) | Composable screen sections (Title, Fields, Text, Menu, Footer) |
| [`modalx::shortcuts`](#shortcuts) | High-level shortcut abstraction and dynamic footer bar builder |
| [`modalx::terminal`](#terminal) | Terminal lifecycle, raw mode guards, and panic hooks |
| [`modalx::text_flow`](#text_flow) | Greedy field reflow, word wrapping, and ANSI-aware string utilities |
| [`modalx::theme`](#theme) | Border characters, ANSI stripping, and visual length calculations |

---

## Primary Types

### `modals::SelectModal`
```rust
pub struct SelectModal { ... }

impl SelectModal {
    pub fn new() -> Self;
    pub fn with_title(self, title: impl Into<String>, is_main_menu: bool) -> Self;
    pub fn with_header_row(self, row: impl Into<String>) -> Self;
    pub fn with_raw_fields(self, raw: &str, delimiter: &str) -> Self;
    pub fn with_section(self, section: ModalSection) -> Self;
    pub fn item(self, key: impl Into<String>, label: impl Into<String>) -> Self;
    pub fn run(&self, selected: &mut usize) -> Result<SelectOutcome>;
}
```

### `modals::FormModal`
```rust
pub struct FormModal { ... }

impl FormModal {
    pub fn new(title: impl Into<String>) -> Self;
    pub fn with_header(self, header: impl Into<String>) -> Self;
    pub fn field(self, field: FormField) -> Self;
    pub fn run(mut self) -> Result<FormResult>;
}

pub enum FormResult {
    Submitted(HashMap<String, String>),
    Cancelled,
}
```

### `modals::FormField`
```rust
pub struct FormField {
    pub key: String,
    pub label: String,
    pub field_type: FormFieldType,
    pub value: String,
    pub default_value: String,
    pub placeholder: String,
    ...
}

impl FormField {
    pub fn string(key: impl Into<String>, label: impl Into<String>) -> Self;
    pub fn integer(key: impl Into<String>, label: impl Into<String>) -> Self;
    pub fn number(key: impl Into<String>, label: impl Into<String>) -> Self;
    pub fn password(key: impl Into<String>, label: impl Into<String>) -> Self;
    pub fn with_default(self, val: impl Into<String>) -> Self;
    pub fn with_placeholder(self, ph: impl Into<String>) -> Self;
    pub fn with_validator(self, f: impl Fn(&str) -> std::result::Result<(), String> + Send + Sync + 'static) -> Self;
    pub fn with_force_validator(self, f: impl Fn(&str, &str) -> String + Send + Sync + 'static) -> Self;
}
```

### `modals::InputModal`
```rust
pub struct InputModal { ... }

impl InputModal {
    pub fn new(title: impl Into<String>, prompt: impl Into<String>) -> Self;
    pub fn with_placeholder(self, placeholder: impl Into<String>) -> Self;
    pub fn with_default(self, default_value: impl Into<String>) -> Self;
    pub fn with_password(self, is_password: bool) -> Self;
    pub fn with_validator(self, validator: impl Fn(&str) -> std::result::Result<(), String> + 'static) -> Self;
    pub fn run(self) -> Result<InputOutcome>;
}

pub enum InputOutcome {
    Submitted(String),
    Cancelled,
}
```

### `modals::ConfirmModal`
```rust
pub struct ConfirmModal { ... }

impl ConfirmModal {
    pub fn new(title: impl Into<String>, question: impl Into<String>) -> Self;
    pub fn with_detail(self, detail: impl Into<String>) -> Self;
    pub fn with_yes_label(self, label: impl Into<String>) -> Self;
    pub fn with_no_label(self, label: impl Into<String>) -> Self;
    pub fn default_yes(self, yes: bool) -> Self;
    pub fn danger(self, danger: bool) -> Self;
    pub fn run(self) -> Result<ConfirmOutcome>;
}

pub enum ConfirmOutcome {
    Confirmed,
    Cancelled,
}
```

### `modals::InfoModal`
```rust
pub struct InfoModal { ... }

impl InfoModal {
    pub fn new(title: impl Into<String>, lines: Vec<String>) -> Self;
    pub fn with_shortcuts(self, shortcuts: impl Into<Shortcuts>) -> Self;
    pub fn run(self) -> Result<()>;
}
```

### `modals::TableModal`
```rust
pub struct TableModal { ... }

impl TableModal {
    pub fn new(title: impl Into<String>) -> Self;
    pub fn with_columns(self, cols: Vec<TableColumn>) -> Self;
    pub fn with_rows(self, rows: Vec<Vec<String>>) -> Self;
    pub fn with_selectable(self, selectable: bool) -> Self;
    pub fn run(self, selected_row: &mut usize) -> Result<TableOutcome>;
}

pub enum TableOutcome {
    Selected(usize),
    Cancelled,
}
```

### `modals::WaitingModal`
```rust
pub struct WaitingModal { ... }

impl WaitingModal {
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self;
    pub fn with_step(self, step: impl Into<String>, completed: bool) -> Self;
    pub fn render_spinner<W: io::Write>(&self, frame_idx: usize, writer: &mut W) -> Result<()>;
    pub fn render_static<W: io::Write>(&self, writer: &mut W) -> Result<()>;
}

pub const SPINNER_FRAMES: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
```

---

## Shortcuts Engine

### `shortcuts::Shortcut`
```rust
pub struct Shortcut {
    pub keys: String,
    pub description: String,
}

impl Shortcut {
    pub fn new(keys: impl Into<String>, description: impl Into<String>) -> Self;
    pub fn dismiss() -> Self;
    pub fn select() -> Self;
    pub fn move_selection() -> Self;
    pub fn move_arrows() -> Self;
    pub fn back() -> Self;
    pub fn cancel() -> Self;
    pub fn confirm() -> Self;
    pub fn save() -> Self;
    pub fn toggle() -> Self;
    pub fn exit() -> Self;
    pub fn scroll() -> Self;
    pub fn page() -> Self;
}
```

### `shortcuts::Shortcuts`
```rust
pub struct Shortcuts { ... }

impl Shortcuts {
    pub fn new() -> Self;
    pub fn add(mut self, shortcut: Shortcut) -> Self;
    pub fn push(&mut self, shortcut: Shortcut) -> &mut Self;
    pub fn add_if(self, condition: bool, shortcut: Shortcut) -> Self;
    pub fn scroll_if(self, condition: bool) -> Self;
    pub fn exit_if(self, condition: bool) -> Self;
    pub fn page_if(self, condition: bool) -> Self;
    pub fn to_footer_string(&self) -> String;
    pub fn to_button_items(&self) -> Vec<String>;
}
```

---

## Terminal & Lifecycle

### RAII Guards
```rust
pub struct TerminalGuard { ... }
impl TerminalGuard {
    pub fn enter() -> Result<Self>;
}

pub struct AltScreenGuard { ... }
impl AltScreenGuard {
    pub fn enter() -> Self;
}

pub struct NavGuard { ... }
impl NavGuard {
    pub fn new(name: impl Into<String>) -> Self;
}
```

### Terminal Functions
```rust
pub fn init_terminal_panic_hook();
pub fn get_terminal_size() -> (u16, u16);
pub fn is_terminal_too_small(width: u16, height: u16) -> bool;
pub fn wait_for_valid_size() -> Result<(u16, u16)>;
pub fn clean_exit() -> !;
```

---

## Text Flow & Formatting

```rust
pub fn visible_len(text: &str) -> usize;
pub fn strip_ansi(text: &str) -> String;
pub fn truncate_ansi(text: &str, max_len: usize) -> String;
pub fn wrap_words(text: &str, width: usize) -> Vec<String>;
pub fn wrap_fields(fields: &[&str], delimiter: &str, width: usize) -> Vec<String>;
pub fn wrap_button_items(items: &[&str], width: usize, spacing: usize) -> Vec<String>;
```
