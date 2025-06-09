# Hybrid Node

A blockchain node implementation that integrates both EVM (Ethereum Virtual Machine) and RISC-V execution environments.

## Overview

Hybrid Node is an experimental blockchain node that extends the traditional Ethereum execution environment by adding support for RISC-V architecture. This allows for:

- Running traditional EVM smart contracts
- Executing RISC-V binaries on-chain
- Creating hybrid applications that leverage both environments

## Features

- **Dual Execution Environment**: Seamlessly switch between EVM and RISC-V execution
- **Development Mode**: Enhanced debugging capabilities with `--dev` flag
- **JSON-RPC API**: Standard Ethereum API for interacting with the node
- **Hybrid Payload Builder**: Custom payload building for both execution environments

## Installation

### Prerequisites

- Rust 2021 edition or later
- Cargo build tools

### Building from Source

```bash
# Clone the repository
git clone <repository-url>
cd hybrid

# Build the node
cargo build --release -p hybrid-node
```

## Usage

### Starting the Node

```bash
# Start in normal mode
./target/release/hybrid-node start

# Start in development mode with additional debugging
./target/release/hybrid-node start --dev
```

### Command Line Options

- `start`: Start the blockchain node (default if no command is provided)
- `config`: Print the current node configuration
- `--dev`: Run as a development node with additional debugging features

## Architecture

The Hybrid Node is built on a modular architecture:

- `command.rs`: Defines the CLI interface
- `main.rs`: Entry point that initializes and starts the node
- `vm` crate: Core execution environment implementation
- `hybrid_evm` crate: Integration between EVM and RISC-V execution

## Development

### Project Structure

```
hybrid/
├── bins/
│   └── hybrid-node/      # Main node binary
├── crates/
│   ├── vm/               # Core VM implementation
│   ├── hybrid_evm/       # Hybrid execution environment
│   └── compile/          # Compilation tools
└── contracts/            # Example contracts
```

## License

[Insert license information here]

## Contributing

[Insert contribution guidelines here]