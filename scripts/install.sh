#!/bin/bash

# Hybrid Blockchain Framework Installation Script
# This script installs the Hybrid Blockchain Framework tools with a single command
# Usage: curl --proto '=https' --tlsv1.2 https://raw.githubusercontent.com/developeruche/hybrid/main/scripts/install.sh -sSf | sh

set -e

REPO_URL="https://github.com/developeruche/hybrid.git"
TEMP_DIR=$(mktemp -d)
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}┌───────────────────────────────────────────┐${NC}"
echo -e "${BLUE}│       Hybrid Blockchain Framework         │${NC}"
echo -e "${BLUE}│              Installer                    │${NC}"
echo -e "${BLUE}└───────────────────────────────────────────┘${NC}"
echo -e "This will install cargo-hybrid and hybrid-node tools"
echo ""

# Check for dependencies
echo -e "Checking dependencies..."

# Check if Git is installed
if ! command -v git &> /dev/null; then
    echo -e "${RED}Error: Git is required but not installed${NC}"
    echo "Please install Git and try again."
    exit 1
fi

# Check if Rust is installed
if ! command -v rustc &> /dev/null || ! command -v cargo &> /dev/null; then
    echo -e "${YELLOW}Rust and Cargo are required but not detected${NC}"
    echo "Would you like to install Rust now? (y/n)"
    read -r install_rust
    
    if [[ "$install_rust" =~ ^[Yy]$ ]]; then
        echo "Installing Rust..."
        curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh -s -- -y
        source "$HOME/.cargo/env"
    else
        echo -e "${RED}Rust is required to install Hybrid. Please install Rust and try again.${NC}"
        echo "You can install Rust by running: curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh"
        exit 1
    fi
fi

echo -e "${GREEN}✓ Rust is installed${NC}"

# Clone the repository
echo "Cloning Hybrid repository..."
git clone "$REPO_URL" "$TEMP_DIR/hybrid" --depth 1 || {
    echo -e "${RED}Failed to clone repository${NC}"
    exit 1
}

echo "Installing Hybrid tools..."
# Try to install with progress feedback
(
    echo -e "${BLUE}[1/2]${NC} Installing cargo-hybrid..."
    if cargo install --path "$TEMP_DIR/hybrid/bins/cargo-hybrid"; then
        echo -e "${GREEN}✓ cargo-hybrid installed successfully${NC}"
    else
        echo -e "${RED}Failed to install cargo-hybrid${NC}"
        exit 1
    fi

    echo -e "${BLUE}[2/2]${NC} Installing hybrid-node..."
    if cargo install --path "$TEMP_DIR/hybrid/bins/hybrid-node"; then
        echo -e "${GREEN}✓ hybrid-node installed successfully${NC}"
    else
        echo -e "${RED}Failed to install hybrid-node${NC}"
        exit 1
    fi
) || {
    echo -e "${RED}Installation failed${NC}"
    echo "Please check the error messages above and try again."
    exit 1
}

# Cleanup
echo "Cleaning up temporary files..."
rm -rf "$TEMP_DIR"
echo -e "${GREEN}✓ Cleanup complete${NC}"

echo ""
echo -e "${GREEN}┌───────────────────────────────────────────┐${NC}"
echo -e "${GREEN}│       Installation Complete! 🚀           │${NC}"
echo -e "${GREEN}└───────────────────────────────────────────┘${NC}"
echo ""
echo -e "You can now use the following commands:"
echo -e "  ${BLUE}cargo hybrid${NC} - For RISC-V smart contract development"
echo -e "  ${BLUE}hybrid-node${NC}  - To run a hybrid blockchain node"
echo ""
echo -e "${YELLOW}Getting started:${NC}"
echo -e "  1. Create a new smart contract: ${BLUE}cargo hybrid new my-contract${NC}"
echo -e "  2. Build your contract: ${BLUE}cd my-contract && cargo hybrid build${NC}"
echo -e "  3. Start a local node: ${BLUE}cargo hybrid node${NC}"
echo -e "  4. Deploy your contract: ${BLUE}cargo hybrid deploy${NC}"
echo ""
echo -e "Documentation: https://github.com/developeruche/hybrid#readme"
echo -e "Issues & Support: https://github.com/developeruche/hybrid/issues"
echo ""
echo -e "Enjoy building with Hybrid Blockchain Framework! 🎉"