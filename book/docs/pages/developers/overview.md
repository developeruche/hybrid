# Hybrid Framework

A comprehensive framework and node for developing, deploying, and interacting with smart contracts written in Rust and Solidity (including any language that compiles to EVM bytecode, such as Vyper, Yul, Huff, etc.)

## Overview

The Hybrid framework comprises a two-part system:

1. **Hybrid Node**: A standalone Ethereum node with native RISC-V contract execution, also enabling EVM execution using a `mini-evm-interpreter`.
2. **Cargo Hybrid**: A development tool that enables the development, deployment, testing, and interaction with smart contracts written in Rust, compiled to RISC-V IMAC 64-bit.

## Key Features

- **RISC-V Smart Contract Development**: Write contracts in Rust compiled to RISC-V bytecode
- **Local Blockchain Node**: Run a development blockchain with RISC-V VM support
- **Deployment Tools**: Deploy RISC-V contracts with simple commands
- **Dual VM Integration**: Support for both RISC-V VM and EVM in a single node
- **Hybrid Execution Environment**: Seamlessly switch between EVM and RISC-V execution, enabled by an EVM emulator

## Getting Started

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

### Creating a New Contract

```bash
cargo hybrid new [NAME] --template [TEMPLATE]
```

**Available templates:**
- `storage` (default) - Contract with storage examples
- `bare` - Minimal contract template
- `erc20` - ERC20 token implementation

**Example:**
```bash
cargo hybrid new my-token --template erc20
```

### Building Contracts

```bash
cd my-token
cargo hybrid build
```

This compiles your Rust smart contract to RISC-V bytecode and outputs the binary to the `out` directory.

**Options:**
- `--out DIR` - Specify a custom output directory (default: "out")

### Checking Contracts

To check if your contract compiles without generating output:

```bash
cargo hybrid check
```

### Deploying Contracts

```bash
cargo hybrid deploy --rpc [RPC_URL]
```

**Options:**
- `--out DIR` - Specify the directory containing compiled contracts (default: "out")
- `--rpc URL` - RPC endpoint to deploy to (default: "http://localhost:8545")
- `--private-key KEY` - Private key to use for deployment (default key provided)
- `--encoded-args ARGS` - Constructor arguments (hex encoded, with or without 0x prefix)

**Example with constructor arguments:**
```bash
cargo hybrid deploy --encoded-args 0x000000000000000000000000f39fd6e51aad88f6f4ce6ab8827279cfffb92266
```

### Starting a Local Node

You can start a local development node directly using cargo-hybrid:

```bash
cargo hybrid node
```

This starts the hybrid node in development mode.

## Running the Hybrid Node

The Hybrid blockchain node is a RETH-based implementation that supports both EVM and RISC-V VM (r55) in a unified execution environment.

### Starting the Node

```bash
hybrid-node start
```

**Options:**
- `--dev` - Run in development mode with additional debugging features

**Example:**
```bash
hybrid-node start --dev
```

### Viewing Node Configuration

```bash
hybrid-node config
```

## Architecture

The framework consists of several key components:

- **hybrid-derive**: Procedural macros for contract development (`#[contract]`, `#[storage]`, etc.)
- **hybrid-compile**: Rust-to-RISC-V compilation pipeline
- **hybrid-vm**: Virtual machine for executing RISC-V contracts in EVM context
- **hybrid-ethereum**: Custom Ethereum node implementation using Reth
- **rvemu**: RISC-V emulator implementation, a clone and modification of the [RISC-V emulator](https://github.com/r55-eth/rvemu)
- **cargo-hybrid**: Command-line interface for development workflow

## Development Workflow

1. **Create Project**: Use `cargo hybrid new` to scaffold a new contract
2. **Implement Logic**: Write contract in `src/lib.rs` using Hybrid macros
3. **Local Testing**: Use `cargo test` for unit tests
4. **Build**: Compile with `cargo hybrid build`
5. **Deploy**: Deploy with `cargo hybrid deploy`
6. **Interact**: Use standard Ethereum tools for interaction

### Contract Structure

Hybrid contracts require a specific project structure:

```
my_contract/
├── Cargo.toml          # Must include required features and dependencies
├── src/
│   └── lib.rs          # Contract implementation
└── out/                # Generated bytecode (after build)
    └── my_contract.bin
```

## Examples

### Basic Storage Contract

```rust
use hybrid_derive::{contract, storage};

#[storage]
pub struct Storage {
    value: u64,
}

#[contract]
impl Storage {
    pub fn new() -> Self {
        Self { value: 0 }
    }
    
    pub fn set_value(&mut self, new_value: u64) {
        self.value = new_value;
    }
    
    pub fn get_value(&self) -> u64 {
        self.value
    }
}
```

## Contributing

We welcome contributions! Please see our [contributing guidelines](CONTRIBUTING.md) for details.

### Development Setup

1. Clone the repository
2. Install prerequisites
3. Run tests: `cargo test`
4. Build: `cargo build`

## Acknowledgments

This project was inspired by and builds upon the work of the r55 team. The `hybrid_evm` crate and parts of the VM implementation were adapted from the r55 project, with modifications to support our hybrid execution environment.

- **r55 Project**: [https://github.com/r55-eth/r55](https://github.com/r55-eth/r55)

Special thanks to the r55 team for their pioneering work in bringing RISC-V execution to the Ethereum ecosystem.

## License

[MIT License](LICENSE)

## Disclaimer

**Note**: This is experimental technology. Use with caution in production environments.