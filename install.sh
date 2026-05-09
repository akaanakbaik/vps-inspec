#!/bin/bash

set -e

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║     VPS INSPECTOR PROFESSIONAL - ONE CLICK INSTALLER        ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

check_os() {
    echo -e "${BLUE}🔍 Checking operating system...${NC}"
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        echo -e "${GREEN}✓ Linux detected${NC}"
        if [[ -f /etc/os-release ]]; then
            . /etc/os-release
            echo -e "  Distribution: $NAME $VERSION"
        fi
    else
        echo -e "${RED}✗ This tool only supports Linux${NC}"
        exit 1
    fi
}

check_rust() {
    echo -e "${BLUE}🦀 Checking Rust installation...${NC}"
    if command -v rustc &> /dev/null; then
        RUST_VERSION=$(rustc --version | cut -d' ' -f2)
        echo -e "${GREEN}✓ Rust found: $RUST_VERSION${NC}"
    else
        echo -e "${YELLOW}⚠️ Rust not found. Installing...${NC}"
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
        echo -e "${GREEN}✓ Rust installed successfully${NC}"
    fi
}

check_dependencies() {
    echo -e "${BLUE}📦 Checking system dependencies...${NC}"
    
    if command -v apt &> /dev/null; then
        sudo apt update
        sudo apt install -y build-essential pkg-config libssl-dev git curl wget
    elif command -v yum &> /dev/null; then
        sudo yum groupinstall -y "Development Tools"
        sudo yum install -y openssl-devel git curl wget
    elif command -v dnf &> /dev/null; then
        sudo dnf groupinstall -y "Development Tools"
        sudo dnf install -y openssl-devel git curl wget
    elif command -v pacman &> /dev/null; then
        sudo pacman -S --noconfirm base-devel openssl git curl wget
    else
        echo -e "${RED}✗ Unsupported package manager${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✓ Dependencies installed${NC}"
}

clone_repo() {
    echo -e "${BLUE}📥 Cloning repository...${NC}"
    
    if [[ -d "vps-inspec" ]]; then
        echo -e "${YELLOW}⚠️ Directory exists. Updating...${NC}"
        cd vps-inspec
        git pull
    else
        git clone https://github.com/akaanakbaik/vps-inspec.git
        cd vps-inspec
    fi
    
    echo -e "${GREEN}✓ Repository ready${NC}"
}

build_project() {
    echo -e "${BLUE}🔨 Building project (this may take a few minutes)...${NC}"
    
    cargo build --release
    
    if [[ $? -eq 0 ]]; then
        echo -e "${GREEN}✓ Build successful${NC}"
    else
        echo -e "${RED}✗ Build failed${NC}"
        exit 1
    fi
}

install_binary() {
    echo -e "${BLUE}💾 Installing binary...${NC}"
    
    sudo cp target/release/vps-inspec /usr/local/bin/
    sudo chmod +x /usr/local/bin/vps-inspec
    
    if command -v vps-inspec &> /dev/null; then
        echo -e "${GREEN}✓ Binary installed to /usr/local/bin/vps-inspec${NC}"
    else
        echo -e "${RED}✗ Installation failed${NC}"
        exit 1
    fi
}

verify_installation() {
    echo -e "${BLUE}✅ Verifying installation...${NC}"
    
    VERSION=$(vps-inspec --version 2>/dev/null || echo "Unknown")
    echo -e "${GREEN}✓ VPS Inspector Professional $VERSION${NC}"
    
    if vps-inspec --help &> /dev/null; then
        echo -e "${GREEN}✓ Help menu accessible${NC}"
    else
        echo -e "${YELLOW}⚠️ Help menu check skipped${NC}"
    fi
}

show_completion() {
    echo ""
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║                    🎉 INSTALLATION COMPLETE!                 ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo ""
    echo -e "${GREEN}VPS Inspector Professional is now installed!${NC}"
    echo ""
    echo "Quick start:"
    echo "  $ vps-inspec"
    echo ""
    echo "For help:"
    echo "  $ vps-inspec --help"
    echo ""
    echo "To update later:"
    echo "  $ vps-inspec --update"
    echo "  or"
    echo "  $ cd ~/vps-inspec && git pull && cargo build --release && sudo cp target/release/vps-inspec /usr/local/bin/"
    echo ""
    echo -e "${BLUE}Website: https://github.com/akaanakbaik/vps-inspec${NC}"
    echo ""
}

main() {
    check_os
    check_rust
    check_dependencies
    clone_repo
    build_project
    install_binary
    verify_installation
    show_completion
}

main "$@"