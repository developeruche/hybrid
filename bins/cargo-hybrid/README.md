# Cargo Hybrid

A CLI tool for developing, building, and deploying Rust-based smart contracts on the Hybrid blockchain.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)

## Overview

Cargo Hybrid extends the Rust ecosystem to support blockchain development by providing tools for:

- Creating new smart contract projects from templates
- Building and compiling smart contracts
- Checking contracts for errors
- Deploying contracts to the Hybrid blockchain
- Starting a local development node

## Installation

### Prerequisites

- Rust and Cargo (install via [rustup](https://rustup.rs/))
- `hybrid-node` for local development (must be in your PATH)

### Install cargo-hybrid

```bash
cargo install cargo-hybrid
```

## Usage

Cargo Hybrid is used as a cargo subcommand:

```bash
cargo hybrid [COMMAND]
```

### Available Commands

| Command | Description |
|---------|-------------|
| `new` | Create a new smart contract project |
| `build` | Build a smart contract |
| `check` | Check if a smart contract compiles |
| `deploy` | Deploy a smart contract to the blockchain |
| `node` | Start a local development node |

## Command Details

### Creating a New Project

```bash
cargo hybrid new [NAME] [--template TEMPLATE]
```

Creates a new smart contract project based on a template.

**Options:**
- `--template` - Template to use for the new project (default: "storage")
- `NAME` - Name of the project (default: "my-hybrid-contract")

**Available Templates:**
- `storage` - A basic contract with storage functionality
- `bare` - A minimal contract with no additional features
- `erc20` - A token contract implementing the ERC-20 standard

**Example:**
```bash
cargo hybrid new my-token-contract --template erc20
```

### Building a Contract

```bash
cargo hybrid build [--out OUT]
```

Builds the smart contract and outputs the compiled files to the specified directory.

**Options:**
- `--out` - Output directory for the compiled contract (default: "out")

**Example:**
```bash
cargo hybrid build --out build
```

### Checking a Contract

```bash
cargo hybrid check
```

Checks if the smart contract compiles without updating the output directory.

**Example:**
```bash
cargo hybrid check
```

### Deploying a Contract

```bash
cargo hybrid deploy [--out OUT] [--rpc RPC] [--private-key KEY] [--encoded-args ARGS]
```

Deploys a smart contract to the blockchain.

**Options:**
- `--out` - Path to the output directory containing the compiled contract (default: "out")
- `--rpc` - RPC endpoint to deploy to (default: "http://127.0.0.1:8545")
- `--private-key` - Private key to use for deployment (default key provided)
- `--encoded-args` - Constructor arguments (hex encoded, with or without 0x prefix)

**Example:**
```bash
cargo hybrid deploy --rpc http://testnet.hybrid.io:8545
```

With constructor arguments:
```bash
cargo hybrid deploy --encoded-args 0x000000000000000000000000f39fd6e51aad88f6f4ce6ab8827279cfffb92266
```

### Starting a Local Node

```bash
cargo hybrid node
```

Starts the hybrid node in development mode.

**Example:**
```bash
cargo hybrid node
```

## Development Workflow

A typical development workflow might look like:

1. Create a new contract:
   ```bash
   cargo hybrid new my-contract
   cd my-contract
   ```

2. Edit your contract code in `src/lib.rs`

3. Check that your contract compiles:
   ```bash
   cargo hybrid check
   ```

4. Build your contract:
   ```bash
   cargo hybrid build
   ```

5. Start a local development node:
   ```bash
   cargo hybrid node
   ```

6. In another terminal, deploy your contract:
   ```bash
   cargo hybrid deploy
   ```

## Troubleshooting

### Common Issues

- **Error: 'hybrid-node' command not found**  
  Make sure the hybrid-node executable is installed and available in your PATH.

- **Error: No compiled contracts found in 'out'**  
  Run `cargo hybrid build` before attempting to deploy.

- **Error: Failed to decode constructor arguments**  
  Make sure the constructor arguments are properly hex encoded.

## Project Structure

The cargo-hybrid tool is organized as follows:

```
cargo-hybrid/
├── src/
│   ├── command.rs       # Command-line argument definitions
│   ├── handlers.rs      # Implementation of command handlers
│   ├── main.rs          # Entry point for the CLI
│   └── utils.rs         # Utility functions for deployment and logging
├── Cargo.toml           # Dependencies and package configuration
└── README.md            # This documentation
```

## Contributing

Contributions are welcome! Here's how you can contribute:

1. Fork the repository
2. Create a new branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Commit your changes (`git commit -m 'Add some amazing feature'`)
5. Push to the branch (`git push origin feature/amazing-feature`)
6. Open a Pull Request

Please make sure your code follows the project's style guidelines and includes appropriate tests.

## License

This project is licensed under the MIT License - see the LICENSE file for details.