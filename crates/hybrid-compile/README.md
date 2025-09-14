# hybrid-compile

[![Crates.io](https://img.shields.io/crates/v/hybrid-compile.svg)](https://crates.io/crates/hybrid-compile)
[![Documentation](https://docs.rs/hybrid-compile/badge.svg)](https://docs.rs/hybrid-compile)
[![License](https://img.shields.io/crates/l/hybrid-compile.svg)](https://github.com/hybrid-blockchain/hybrid/blob/main/LICENSE)

A Rust compiler crate for building RISC-V smart contracts that run on EVM-compatible blockchains. This crate is part of the [Hybrid Framework](../../README.md), which enables developers to write smart contracts in Rust while maintaining full compatibility with the Ethereum ecosystem.

## Overview

`hybrid-compile` is responsible for compiling Rust smart contracts to RISC-V bytecode that can be executed on EVM-compatible blockchains. It handles the entire compilation pipeline from source code analysis to binary generation, including dependency management and contract validation.

## Features

- 🦀 **Rust-to-RISC-V Compilation**: Compile Rust smart contracts to RISC-V bytecode
- 📦 **Dependency Management**: Handle contract dependencies and inheritance
- ✅ **Contract Validation**: Validate contract structure and required features
- 🔧 **Build Configuration**: Support for different build modes (check, build)
- 📊 **Progress Tracking**: Built-in progress bars for compilation status
- 🎯 **Target Optimization**: Optimized builds for the `riscv64imac-unknown-none-elf` target

## Architecture

The crate is organized into three main modules:

```
hybrid-compile/
├── src/
│   ├── lib.rs          # Main compilation orchestration
│   ├── primitives.rs   # Core data structures and compilation logic
│   └── utils.rs        # Utility functions for contract discovery and parsing
```

### Core Components

1. **Contract Discovery**: Automatically finds and validates Rust contract projects
2. **Dependency Resolution**: Resolves and validates contract dependencies
3. **Compilation Pipeline**: Orchestrates the multi-stage compilation process
4. **Binary Generation**: Produces deployment-ready RISC-V bytecode

## Usage

### Basic Compilation

```rust
use hybrid_compile::run_contract_compilation;
use std::path::Path;
use indicatif::ProgressBar;

let contract_root = Path::new("./my-contract");
let progress_bar = ProgressBar::new(100);
let output_dir = "out".to_string();

// Compile the contract
run_contract_compilation(contract_root, false, progress_bar, output_dir)?;
```

### Syntax Check Only

```rust
// Just check syntax without generating binaries
run_contract_compilation(contract_root, true, progress_bar, output_dir)?;
```

## Contract Requirements

For a Rust project to be recognized as a valid Hybrid contract, it must meet specific requirements:

### Cargo.toml Structure

```toml
[package]
name = "my-contract"
version = "0.1.0"
edition = "2021"

# Required features
[features]
default = []
deploy = []
interface-only = []

# Required binaries
[[bin]]
name = "runtime"
path = "src/lib.rs"

[[bin]]
name = "deploy"
path = "src/lib.rs"
required-features = ["deploy"]

# Required dependencies
[dependencies]
hybrid-derive = { path = "../hybrid-derive" }
hybrid-contract = { path = "../hybrid-contract" }

# Optional contract dependencies (with interface-only feature)
[dependencies.other-contract]
path = "../other-contract"
features = ["interface-only"]
```

## Compilation Process

The compilation process consists of two main stages:

### 1. Runtime Compilation

Compiles the contract runtime code that executes during normal contract calls:

```bash
cargo +nightly-2025-01-07 build -r --lib -Z build-std=core,alloc \
    --target riscv64imac-unknown-none-elf --bin runtime
```

### 2. Deploy Compilation

Compiles the deployment code that runs during contract deployment:

```bash
cargo +nightly-2025-01-07 build -r --lib -Z build-std=core,alloc \
    --target riscv64imac-unknown-none-elf --bin deploy --features deploy
```

The final bytecode includes a `0xff` prefix and combines both runtime and deployment code.

## Error Handling

The crate provides comprehensive error handling for common compilation issues:

- **`ContractError::IoError`**: File system operation errors
- **`ContractError::NotToml`**: Invalid Cargo.toml format
- **`ContractError::MissingDependencies`**: Required dependencies not found
- **`ContractError::MissingBinaries`**: Required binary targets not configured
- **`ContractError::MissingFeatures`**: Required Cargo features not defined
- **`ContractError::WrongPath`**: Invalid dependency path
- **`ContractError::CyclicDependency`**: Circular dependency detected

## Development Setup

### Prerequisites

1. **Rust Nightly**: Install the specific nightly toolchain:
```bash
rustup install nightly-2025-01-07
rustup component add rust-src --toolchain nightly-2025-01-07
```

2. **RISC-V Target**: Add the RISC-V target:
```bash
rustup target add riscv64imac-unknown-none-elf --toolchain nightly-2025-01-07
```

### Building from Source

```bash
# Clone the repository
git clone https://github.com/hybrid-blockchain/hybrid.git
cd hybrid/crates/hybrid-compile

# Build the crate
cargo build --release

# Run tests
cargo test

# Generate documentation
cargo doc --open
```

## Contributing

Contributions are welcome! Please read our [contributing guidelines](../../CONTRIBUTING.md) before submitting PRs.

## License

This project is licensed under the [MIT License](../../LICENSE) - see the LICENSE file for details.