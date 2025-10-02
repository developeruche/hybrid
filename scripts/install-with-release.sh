#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
GITHUB_REPO="developeruche/hybrid"
GITHUB_RAW_URL="https://raw.githubusercontent.com/${GITHUB_REPO}/main/manual-releases"
INSTALL_DIR="${HOME}/.cargo/bin"
BINARIES=("cargo-hybrid" "hybrid-node")

echo -e "${GREEN}Hybrid Installation Script${NC}"
echo "======================================"
echo ""

# Ensure install directory exists
if [ ! -d "$INSTALL_DIR" ]; then
    echo -e "${YELLOW}Creating installation directory: $INSTALL_DIR${NC}"
    mkdir -p "$INSTALL_DIR"
fi

# Function to download and install a binary
install_binary() {
    local binary_name=$1
    local download_url="${GITHUB_RAW_URL}/${binary_name}"
    local install_path="${INSTALL_DIR}/${binary_name}"

    echo -e "${YELLOW}Downloading ${binary_name}...${NC}"

    if command -v curl &> /dev/null; then
        if curl -fsSL -o "$install_path" "$download_url"; then
            chmod +x "$install_path"
            echo -e "${GREEN}✓ Successfully installed ${binary_name}${NC}"
            return 0
        else
            echo -e "${RED}✗ Failed to download ${binary_name}${NC}"
            return 1
        fi
    elif command -v wget &> /dev/null; then
        if wget -q -O "$install_path" "$download_url"; then
            chmod +x "$install_path"
            echo -e "${GREEN}✓ Successfully installed ${binary_name}${NC}"
            return 0
        else
            echo -e "${RED}✗ Failed to download ${binary_name}${NC}"
            return 1
        fi
    else
        echo -e "${RED}Error: Neither curl nor wget is available. Please install one of them.${NC}"
        exit 1
    fi
}

# Install each binary
failed_installs=()
for binary in "${BINARIES[@]}"; do
    if ! install_binary "$binary"; then
        failed_installs+=("$binary")
    fi
    echo ""
done

# Summary
echo "======================================"
if [ ${#failed_installs[@]} -eq 0 ]; then
    echo -e "${GREEN}All binaries installed successfully!${NC}"
    echo ""
    echo "Installation directory: $INSTALL_DIR"
    echo "Installed binaries:"
    for binary in "${BINARIES[@]}"; do
        echo "  - $binary"
    done
    echo ""
    echo -e "${YELLOW}Note: Make sure ${INSTALL_DIR} is in your PATH${NC}"

    # Check if directory is in PATH
    if [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
        echo -e "${YELLOW}Warning: ${INSTALL_DIR} is not in your PATH${NC}"
        echo "Add the following line to your shell configuration file (~/.bashrc, ~/.zshrc, etc.):"
        echo "  export PATH=\"\$PATH:${INSTALL_DIR}\""
    fi

    exit 0
else
    echo -e "${RED}Some installations failed:${NC}"
    for binary in "${failed_installs[@]}"; do
        echo "  - $binary"
    done
    exit 1
fi
