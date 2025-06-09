# Hybrid Blockchain Framework

A modern blockchain framework for developing, deploying, and interacting with RISCV-based smart contracts on Ethereum.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)

## Overview

Hybrid Blockchain Framework is a complete toolkit for blockchain development, featuring:

- **RISCV Smart Contract Development**: Write contracts in Rust compiled to RISCV bytecode
- **Local Blockchain Node**: Run a development blockchain with RISCV VM support
- **Deployment Tools**: Deploy RISCV contracts with a simple command
- **Dual VM Integration**: Support for both RISCV VM (r55) and EVM in a single node
- **Hybrid Execution Environment**: Seamlessly switch between EVM and RISC-V execution
- **Rust Ecosystem Integration**: Extends Cargo to support blockchain development workflow

## Project Structure

The project is organized into the following directories:

- `bins/` - Executable binaries
  - `cargo-hybrid/` - CLI tool for RISCV contract development (Phase 1)
  - `hybrid-node/` - Hybrid blockchain node implementation based on RETH (Phase 2)
- `crates/` - Core library components
  - `compile/` - Contract compilation tooling for Rust to RISCV
  - `vm/` - Virtual machine for RISCV contract execution (r55)
  - `hybrid_evm/` - Integration between EVM and RISC-V execution
- `contracts/` - Smart contract templates
  - `bare/` - Minimal contract template
  - `erc20/` - ERC20 token implementation
  - `storage/` - Contract with storage examples
- `examples/` - Example projects demonstrating hybrid functionality

## Getting Started

### Prerequisites

- Rust toolchain (stable)
- Cargo package manager

### Installation

```
cargo install --path hybrid/bins/cargo-hybrid
cargo install --path hybrid/bins/hybrid-node
```

## Using cargo-hybrid (Phase 1)

The `cargo-hybrid` tool provides a complete workflow for RISCV-based smart contract development. It extends the Rust ecosystem to support blockchain development.

### Available Commands

| Command | Description |
|---------|-------------|
| `new` | Create a new smart contract project |
| `build` | Build a smart contract |
| `check` | Check if a smart contract compiles |
| `deploy` | Deploy a smart contract to the blockchain |
| `node` | Start a local development node |

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
- `--private-key KEY` - Private key to use for deployment (default key provided)
- `--encoded-args ARGS` - Constructor arguments (hex encoded, with or without 0x prefix)

Example with constructor arguments:
```
cargo hybrid deploy --encoded-args 0x000000000000000000000000f39fd6e51aad88f6f4ce6ab8827279cfffb92266
```

### Starting a Local Node

You can also start a local development node directly using cargo-hybrid:

```
cargo hybrid node
```

This starts the hybrid node in development mode.

## Running the Hybrid Node (Phase 2)

The Hybrid blockchain node is a RETH-based implementation that supports both EVM and RISCV VM (r55) in a unified execution environment.

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

### Architecture

The Hybrid Node is built on a modular architecture:

- Command-line interface for node control
- Core VM implementation supporting dual execution environments
- Custom payload builder for both EVM and RISC-V transactions
- JSON-RPC API compatible with standard Ethereum clients

## Core Components

### Compile Crate

The `compile` crate handles the compilation of Rust smart contracts to RISCV bytecode format required by the r55 VM.

### VM Crate

The `vm` crate provides the execution environment for both RISCV (r55) and EVM, allowing smart contracts to run on the Hybrid blockchain.

### Hybrid EVM Crate

The `hybrid_evm` crate implements the integration between the EVM and RISC-V execution environments, enabling seamless switching between both virtual machines.

## Development Workflow

### Two-Phase Development

#### Phase 1: Smart Contract Development
1. **Create a new contract** using `cargo hybrid new`
2. **Develop** your contract in Rust
3. **Test** your contract with `cargo hybrid check`
4. **Build** your contract to RISCV bytecode with `cargo hybrid build`
5. **Start a local node** with `cargo hybrid node` (or `hybrid-node start --dev`)
6. **Deploy** your RISCV contract with `cargo hybrid deploy`

#### Phase 2: Blockchain Node
1. **Start a local node** with `hybrid-node start --dev`
2. **Deploy and interact** with both EVM and RISCV-based contracts
3. **Configure** your node as needed using the available options
4. **Develop hybrid applications** that leverage both execution environments

### Troubleshooting

Common issues you may encounter:

- **Error: 'hybrid-node' command not found**  
  Make sure the hybrid-node executable is installed and available in your PATH.

- **Error: No compiled contracts found in 'out'**  
  Run `cargo hybrid build` before attempting to deploy.

- **Error: Failed to decode constructor arguments**  
  Make sure the constructor arguments are properly hex encoded.

## Acknowledgments

This project was inspired by and builds upon the work of the r55 team. The `hybrid_evm` crate and parts of the VM implementation were adapted from the r55 project, with modifications to support our hybrid execution environment.

- **r55 Project**: [https://github.com/r55-eth/r55](https://github.com/r55-eth/r55)

Special thanks to the r55 team for their pioneering work in bringing RISC-V execution to the Ethereum ecosystem.

## License

[MIT License](LICENSE)