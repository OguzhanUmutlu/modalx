//! # modalx
//!
//! A responsive, declarative, box-encapsulated Terminal User Interface (TUI) and modal dialog engine for Rust.
//!
//! ## Highlights
//! - **Airtight Dynamic Box Framing**: All visual components are strictly encapsulated within dynamic Unicode box borders (`╭─╮`, `│ │`, `├─┤`, `╰─╯`).
//! - **Responsive Sizing & Terminal Protection**: Automatically detects narrow windows (< 60 cols × 14 rows), rendering a clean, centered "TERMINAL TOO SMALL" warning card that resumes execution upon window resize.
//! - **Flow Layout & Field Reflow**: Delimited metadata (e.g. `Branch: main | Target: Release | Status: Ready`) reflows greedily across multiple lines when terminal width is narrow, avoiding unsightly ellipsis truncation.
//! - **Composable Sections**: Build interfaces declaratively using `TitleSection`, `FieldSection`, `TextSection`, `MenuSection`, and `FooterSection`.
//! - **Batteries-Included Modals**:
//!   - [`SelectModal`]: Searchable, paginated keyboard-driven menus with hotkeys and vim navigation.
//!   - [`InputModal`]: Full readline text editing, cursor navigation, password masking, and real-time validators.
//!   - [`ConfirmModal`]: Two-button affirmative/negative prompts with danger styling and hotkey toggling.
//!   - [`InfoModal`]: Scrollable text and alert dialogs.
//!   - [`WaitingModal`]: In-place progress monitors with smooth animated braille spinners.
//!   - [`TableModal`]: Multi-column tabular data displays with column alignments and row selection.
//! - **Panic Safety**: Automatic terminal cleanup hook ensuring raw mode is disabled and cursor unhidden if an unexpected panic occurs.
//!
//! ## Quickstart
//!
//! ```no_run
//! use modalx::prelude::*;
//!
//! fn main() -> modalx::Result<()> {
//!     let mut selected = 0;
//!     let modal = SelectModal::new()
//!         .with_title("WORKSPACE CONTROLLER", false)
//!         .with_raw_fields("Project: nexus | Target: Release | Status: Ready", " | ")
//!         .item("1", "Run Build Pipeline")
//!         .item("2", "Interactive Test Runner")
//!         .item("3", "Deploy & Release");
//!
//!     match modal.run(&mut selected)? {
//!         SelectOutcome::Selected(idx) => println!("Selected item {}", idx),
//!         SelectOutcome::Cancelled => println!("User pressed Escape"),
//!         _ => {}
//!     }
//!
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod frame;
pub mod input;
pub mod keys;
pub mod modals;
pub mod nav;
pub mod section;
pub mod shortcuts;
pub mod terminal;
pub mod text_flow;
pub mod theme;

pub use error::{Result, TuiError};
pub use frame::{render_too_small, BoxFrame, FrameLine};
pub use input::{TextInput, TextInputAction};
pub use keys::{KeyAction, KeyHelpMode, KeyMap};
pub use modals::*;
pub use nav::{
    box_breadcrumbs, clear_root_breadcrumbs, format_breadcrumbs, get_breadcrumbs,
    set_root_breadcrumbs, NavGuard,
};
pub use section::{
    FieldSection, FooterSection, MenuEntry, MenuSection, ModalSection, SelectItem, TextSection,
    TitleSection,
};
pub use shortcuts::{Shortcut, ShortcutBar, Shortcuts};
pub use terminal::{
    clean_exit, get_content_width, get_terminal_size, init_terminal_panic_hook,
    is_terminal_too_small, restore_terminal, wait_for_constraints, wait_for_valid_size,
    AltScreenGuard, TerminalConstraints, TerminalGuard, MIN_TERM_HEIGHT, MIN_TERM_WIDTH,
};
pub use text_flow::{
    truncate_ansi, truncate_ellipsis, truncate_str, wrap_button_items, wrap_delimited_string,
    wrap_fields, wrap_words,
};
pub use theme::{
    box_bottom, box_divider, box_line, box_title, box_title_simple, box_top, is_utf8_supported,
    strip_ansi, visible_len,
};

/// Convenient prelude module re-exporting all primary traits, structs, and functions.
pub mod prelude {
    pub use crate::error::{Result, TuiError};
    pub use crate::frame::{render_too_small, BoxFrame, FrameLine};
    pub use crate::keys::{KeyAction, KeyHelpMode, KeyMap};
    pub use crate::modals::{
        confirm::{ConfirmModal, ConfirmOutcome},
        form::{FormField, FormFieldType, FormModal, FormResult},
        info::InfoModal,
        input::{InputModal, InputOutcome, InputValidator},
        select::{parse_header_lines, SelectModal, SelectOutcome},
        table::{ColumnAlign, TableColumn, TableModal, TableOutcome},
        waiting::{render_boxed_status, WaitingModal, SPINNER_FRAMES},
    };
    pub use crate::nav::NavGuard;
    pub use crate::section::{
        FieldSection, FooterSection, MenuEntry, MenuSection, ModalSection, SelectItem, TextSection,
        TitleSection,
    };
    pub use crate::shortcuts::{Shortcut, ShortcutBar, Shortcuts};
    pub use crate::terminal::{
        clean_exit, get_content_width, get_terminal_size, restore_terminal, wait_for_valid_size,
        AltScreenGuard, TerminalConstraints, TerminalGuard,
    };
    pub use crate::text_flow::{
        truncate_ansi, truncate_ellipsis, truncate_str, wrap_button_items, wrap_delimited_string,
        wrap_fields, wrap_words,
    };
    pub use crate::theme::{strip_ansi, visible_len};
}
