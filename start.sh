#!/bin/bash
# ============================================================
#  VPS Inspector Professional — Smart Launcher
#  Usage: bash start.sh [--update] [--build] [--help]
# ============================================================

set -euo pipefail

# ── Colours ────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
DIM='\033[2m'
NC='\033[0m'

# ── Constants ───────────────────────────────────────────────
BINARY_NAME="vps-inspec"
UPDATER_NAME="vps-inspec-update"
RELEASE_DIR="target/release"
INSTALL_PATH="/usr/local/bin/${BINARY_NAME}"
REPO_URL="https://github.com/akaanakbaik/vps-inspec.git"

# ── Helpers ─────────────────────────────────────────────────
banner() {
    echo ""
    echo -e "${CYAN}${BOLD}╔══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}${BOLD}║       🔍  VPS INSPECTOR PROFESSIONAL  v1.0               ║${NC}"
    echo -e "${CYAN}${BOLD}║           Smart Launcher — start.sh                      ║${NC}"
    echo -e "${CYAN}${BOLD}╚══════════════════════════════════════════════════════════╝${NC}"
    echo ""
}

info()    { echo -e "${BLUE}ℹ  $*${NC}"; }
success() { echo -e "${GREEN}✔  $*${NC}"; }
warn()    { echo -e "${YELLOW}⚠  $*${NC}"; }
error()   { echo -e "${RED}✘  $*${NC}"; exit 1; }
step()    { echo -e "${BOLD}▶  $*${NC}"; }

usage() {
    echo -e "${BOLD}Usage:${NC}"
    echo "  bash start.sh           # Launch VPS Inspector (auto-build if needed)"
    echo "  bash start.sh --update  # Pull latest changes & rebuild, then launch"
    echo "  bash start.sh --build   # Force rebuild without launching"
    echo "  bash start.sh --help    # Show this help message"
    echo ""
}

# ── Checks ──────────────────────────────────────────────────
check_os() {
    if [[ "$OSTYPE" != "linux-gnu"* ]]; then
        error "VPS Inspector supports Linux only."
    fi
}

check_rust() {
    if ! command -v cargo &>/dev/null; then
        warn "Rust/Cargo not found. Installing via rustup..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --quiet
        # shellcheck source=/dev/null
        source "$HOME/.cargo/env"
    fi
    RUST_VER=$(rustc --version 2>/dev/null | awk '{print $2}')
    success "Rust ${RUST_VER} ready"
}

check_deps() {
    local missing=()
    for cmd in git curl pkg-config; do
        command -v "$cmd" &>/dev/null || missing+=("$cmd")
    done

    if [[ ${#missing[@]} -gt 0 ]]; then
        warn "Missing system packages: ${missing[*]}"
        if command -v apt-get &>/dev/null; then
            sudo apt-get update -qq && sudo apt-get install -y build-essential pkg-config libssl-dev git curl
        elif command -v dnf &>/dev/null; then
            sudo dnf install -y gcc openssl-devel pkgconf git curl
        elif command -v yum &>/dev/null; then
            sudo yum install -y gcc openssl-devel pkgconfig git curl
        elif command -v pacman &>/dev/null; then
            sudo pacman -Sy --noconfirm base-devel openssl git curl
        else
            error "Cannot auto-install dependencies. Install manually: ${missing[*]}"
        fi
    fi
    success "System dependencies OK"
}

# ── Core Actions ─────────────────────────────────────────────
build_binary() {
    step "Building ${BINARY_NAME} (release mode)..."
    if cargo build --release --bin "${BINARY_NAME}" 2>&1; then
        success "Build complete → ${RELEASE_DIR}/${BINARY_NAME}"
    else
        error "Build failed. Check the output above for details."
    fi
}

do_update() {
    step "Pulling latest changes from origin..."
    if git pull --ff-only origin "$(git rev-parse --abbrev-ref HEAD)" 2>&1; then
        success "Repository updated"
    else
        warn "git pull failed — you may have local changes. Continuing with current code."
    fi
    build_binary
}

launch() {
    local binary=""

    # Priority: system-wide install → local release build → cargo run
    if command -v "${BINARY_NAME}" &>/dev/null; then
        binary=$(command -v "${BINARY_NAME}")
    elif [[ -x "${RELEASE_DIR}/${BINARY_NAME}" ]]; then
        binary="${RELEASE_DIR}/${BINARY_NAME}"
    else
        warn "No compiled binary found. Building now..."
        check_rust
        check_deps
        build_binary
        binary="${RELEASE_DIR}/${BINARY_NAME}"
    fi

    echo ""
    echo -e "${DIM}Launching: ${binary}${NC}"
    echo -e "${DIM}────────────────────────────────────────────────────────────${NC}"
    echo ""
    exec "${binary}"
}

# ── Entry Point ──────────────────────────────────────────────
main() {
    banner
    check_os

    local mode="launch"
    for arg in "$@"; do
        case "$arg" in
            --update) mode="update" ;;
            --build)  mode="build"  ;;
            --help|-h) usage; exit 0 ;;
            *) warn "Unknown option: $arg"; usage; exit 1 ;;
        esac
    done

    case "$mode" in
        update)
            check_rust
            check_deps
            do_update
            launch
            ;;
        build)
            check_rust
            check_deps
            build_binary
            success "You can now run: bash start.sh"
            ;;
        launch)
            launch
            ;;
    esac
}

main "$@"
