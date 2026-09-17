#!/usr/bin/env bash
# ==============================================================================
# modalx Quickstart & Installation Script
# https://modalx.oguzhanumutlu.com
# ==============================================================================

set -euo pipefail

CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BOLD='\033[1m'
DIM='\033[2m'
NC='\033[0m'

echo -e "${CYAN}${BOLD}╭──────────────────────────────────────────────────────────────────╮${NC}"
echo -e "${CYAN}${BOLD}│                             modalx                               │${NC}"
echo -e "${CYAN}${BOLD}│       Terminal Modal & TUI Dialog Framework for Rust             │${NC}"
echo -e "${CYAN}${BOLD}╰──────────────────────────────────────────────────────────────────╯${NC}"
echo ""

if ! command -v cargo >/dev/null 2>&1; then
    echo -e "${RED}Error:${NC} Cargo/Rust is not installed on this system."
    echo -e "Please install Rust first using rustup:"
    echo -e "  ${BOLD}curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh${NC}"
    exit 1
fi

RUST_VER="$(rustc --version | awk '{print $2}')"
echo -e "${GREEN}✓${NC} Found Rust ${BOLD}v${RUST_VER}${NC}"

if [ -f "Cargo.toml" ]; then
    echo -e "${BLUE}==>${NC} Existing Cargo workspace detected."
    echo -e "Adding ${BOLD}modalx = \"0.1.0\"${NC} to your project..."
    cargo add modalx
    echo -e "${GREEN}✓ modalx dependency added successfully!${NC}"
    echo ""
    echo -e "Add this to your ${BOLD}src/main.rs${NC}:"
    echo ""
    echo -e "${DIM}use modalx::prelude::*;${NC}"
    echo -e "${DIM}fn main() -> modalx::Result<()> {${NC}"
    echo -e "${DIM}    let mut sel = 0;${NC}"
    echo -e "${DIM}    let modal = SelectModal::menu(\"MY DASHBOARD\")${NC}"
    echo -e "${DIM}        .item(\"1\", \"First Option\")${NC}"
    echo -e "${DIM}        .item(\"2\", \"Second Option\");${NC}"
    echo -e "${DIM}    modal.run(&mut sel)?;${NC}"
    echo -e "${DIM}    Ok(())${NC}"
    echo -e "${DIM}}${NC}"
else
    TARGET_DIR="${1:-modalx-starter}"
    echo -e "${BLUE}==>${NC} Scaffolding starter project in ${BOLD}./${TARGET_DIR}${NC}..."
    cargo new --bin "${TARGET_DIR}"
    cd "${TARGET_DIR}"
    cargo add modalx
    cargo add colored

    cat << 'RUST_EOF' > src/main.rs
use modalx::prelude::*;

fn main() -> modalx::Result<()> {
    let _guard = AltScreenGuard::enter();
    let mut selected = 0;

    loop {
        let modal = SelectModal::menu("MODALX QUICKSTART")
            .with_header_row("Welcome to modalx - Box-encapsulated TUI framework")
            .item("1", "Counter Example")
            .item("2", "Configuration Form")
            .item("3", "Documentation (https://modalx.oguzhanumutlu.com)")
            .item("0", "Exit");

        match modal.run(&mut selected)? {
            SelectOutcome::Selected(0) => {
                show_modal_message(
                    "COUNTER DEMO",
                    &["Counter demonstration selected.", "Press Enter to return."],
                    false,
                )?;
            }
            SelectOutcome::Selected(1) => {
                let form = FormModal::new("CONFIGURATION")
                    .field(FormField::text("Project Name", "my-service"))
                    .field(FormField::integer("Port", 8080));
                let _ = form.run();
            }
            SelectOutcome::Selected(2) => {
                show_modal_message(
                    "DOCUMENTATION",
                    &[
                        "Visit the official documentation at:",
                        "https://modalx.oguzhanumutlu.com",
                    ],
                    false,
                )?;
            }
            SelectOutcome::Selected(3) | SelectOutcome::Cancelled => {
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
RUST_EOF

    echo -e "${GREEN}✓ Starter project created in ./${TARGET_DIR}${NC}"
    echo ""
    echo -e "To run your modalx application:"
    echo -e "  ${BOLD}cd ${TARGET_DIR} && cargo run${NC}"
fi

echo ""
echo -e "Documentation: ${CYAN}${BOLD}https://modalx.oguzhanumutlu.com${NC}"
echo -e "Crates.io:     ${CYAN}${BOLD}https://crates.io/crates/modalx${NC}"
echo -e "Repository:    ${CYAN}${BOLD}https://github.com/OguzhanUmutlu/modalx${NC}"
