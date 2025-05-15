# Hybrid Blockchain Framework

A modern blockchain framework for developing, deploying, and interacting with RISCV-based smart contracts on Ethereum.

## Overview

Hybrid Blockchain Framework is a complete toolkit for blockchain development, featuring:

- **RISCV Smart Contract Development**: Write contracts in Rust compiled to RISCV bytecode
- **Local Blockchain Node**: Run a development blockchain with RISCV VM support
- **Deployment Tools**: Deploy RISCV contracts with a simple command
- **Dual VM Integration**: Support for both RISCV VM (r55) and EVM in a single node

## Project Structure

The project is organized into the following directories:

- `bins/` - Executable binaries
  - `cargo-hybrid/` - CLI tool for RISCV contract development (Phase 1)
  - `node/` - Hybrid blockchain node implementation based on RETH (Phase 2)
- `crates/` - Core library components
  - `compile/` - Contract compilation tooling for Rust to RISCV
  - `vm/` - Virtual machine for RISCV contract execution (r55)
- `contracts/` - Smart contract templates
  - `bare/` - Minimal contract template
  - `erc20/` - ERC20 token implementation
  - `storage/` - Contract with storage examples

## Getting Started

### Prerequisites

- Rust toolchain (stable)
- Cargo package manager

### Installation

```
cargo install --path hybrid/bins/cargo-hybrid
cargo install --path hybrid/bins/node
```

## Using cargo-hybrid (Phase 1)

The `cargo-hybrid` tool provides a complete workflow for RISCV-based smart contract development.

### Creating a New Contract

```
cargo hybrid new [NAME] --template [TEMPLATE]
```

Available templates:
- `storage` (default) - Contract with storage examples
- `bare` - Minimal contract template
- `erc20` - ERC20 token implementation

Example:
```
cargo hybrid new my-token --template erc20
```

### Building Contracts

```
cd my-token
cargo hybrid build
```

This compiles your Rust smart contract to RISCV bytecode and outputs the binary to the `out` directory.

Options:
- `--out DIR` - Specify a custom output directory (default: "out")

### Checking Contracts

To check if your contract compiles without generating output:

```
cargo hybrid check
```

### Deploying Contracts

```
cargo hybrid deploy --rpc [RPC_URL]
```

Options:
- `--out DIR` - Specify the directory containing compiled contracts (default: "out")
- `--rpc URL` - RPC endpoint to deploy to (default: "http://localhost:8545")

## Running the Hybrid Node (Phase 2)

The Hybrid blockchain node is a RETH-based implementation that will eventually support both EVM and RISCV VM (r55).

> **Note:** Currently, the node implementation only supports EVM. RISCV VM (r55) integration is under active development.

### Starting the Node

```
hybrid-node start
```

Options:
- `--dev` - Run in development mode with additional debugging features

Example:
```
hybrid-node start --dev
```

### Viewing Node Configuration

```
hybrid-node config
```

## Core Components

### Compile Crate

The `compile` crate handles the compilation of Rust smart contracts to RISCV bytecode format required by the r55 VM.

### VM Crate

The `vm` crate provides the RISCV virtual machine environment (r55) for executing smart contracts on the Hybrid blockchain alongside the traditional EVM.

## Development Workflow

### Two-Phase Development

#### Phase 1: Smart Contract Development
1. **Create a new contract** using `cargo hybrid new`
2. **Develop** your contract in Rust
3. **Test** your contract with `cargo hybrid check`
4. **Build** your contract to RISCV bytecode with `cargo hybrid build`
5. **Deploy** your RISCV contract with `cargo hybrid deploy`

#### Phase 2: Blockchain Node
1. **Start a local node** with `hybrid-node start --dev`
2. **Deploy and interact** with both EVM and RISCV-based contracts (once r55 support is complete)
3. **Configure** your node as needed using the available options

## License

[MIT License](LICENSE)