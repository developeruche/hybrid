---
description: Guide developers through the installation process
---

# Installation Guide

In this installation flow, there are two main installation steps:

1. **Step One**: Install Hybrid node (`hybrid-node`).
2. **Step Two**: Install Hybrid compile (`cargo-hybrid`).

Before proceeding, ensure you have the necessary prerequisites installed.

### Prerequisites

#### macOS
```sh
brew tap riscv-software-src/riscv
brew install riscv-gnu-toolchain gettext
rustup target add x86_64-unknown-linux-gnu
```

#### General Requirements
- Rust toolchain (stable)
- Cargo package manager

### Installation

#### Method 1: One-line Installation Script (Recommended)

```bash
curl --proto '=https' --tlsv1.2 https://raw.githubusercontent.com/developeruche/hybrid/main/scripts/install.sh -sSf | sh
```

This script will:
1. Check for required dependencies (Git, Rust)
2. Clone the repository
3. Install both cargo-hybrid and hybrid-node
4. Clean up temporary files

#### Method 2: Manual Installation

If you prefer to install manually:

```bash
# Clone the repository
git clone https://github.com/developeruche/hybrid.git

# Install the binaries
cargo install --path hybrid/bins/cargo-hybrid
cargo install --path hybrid/bins/hybrid-node

# Optional: Remove the cloned repository if no longer needed
rm -rf hybrid
```

## Using cargo-hybrid

The `cargo-hybrid` tool provides a complete workflow for RISC-V-based smart contract development, extending the Rust ecosystem to support blockchain development.

### Available Commands

| Command | Description |
|---------|-------------|
| `new` | Create a new smart contract project |
| `build` | Build a smart contract |
| `check` | Check if a smart contract compiles |
| `deploy` | Deploy a smart contract to the blockchain |
| `node` | Start a local development node |
