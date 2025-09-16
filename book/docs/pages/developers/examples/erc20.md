---
description: ERC20 Token Contract Implementation
---

# ERC20 Token Contract

This example demonstrates how to implement a fully-featured ERC20 token contract on the Hybrid blockchain. The contract showcases Hybrid's Ethereum compatibility while leveraging Rust's safety features and performance optimizations.

## Overview

The ERC20 implementation provides all standard token functionality including transfers, approvals, and minting capabilities. Built with Hybrid's Rust-based smart contract framework, it demonstrates how traditional Ethereum contracts can be seamlessly ported to Hybrid while maintaining full backward compatibility.

## Features

- **Standard ERC20 Compliance**: Implements all required ERC20 functions
- **Ownership Management**: Built-in owner controls for administrative functions
- **Event Emission**: Comprehensive event logging for all state changes
- **Error Handling**: Robust error handling with custom error types
- **Gas Optimized**: Leverages Rust's efficiency for optimal gas consumption

Creating this ERC20 token contract using the hybrid framework is relatively straightforward.

```bash
cargo hybrid new hybrid-erc20 --template erc20
```

## Contract Architecture

### Storage Structure

```rust
#[storage]
pub struct ERC20 {
    total_supply: Slot<U256>,
    balance_of: Mapping<Address, Slot<U256>>,
    allowance_of: Mapping<Address, Mapping<Address, Slot<U256>>>,
    owner: Slot<Address>,
}
```

The contract uses Hybrid's storage primitives:
- `Slot<T>`: For single-value storage
- `Mapping<K, V>`: For key-value storage mappings

### Events

The contract emits standard ERC20 events:

```rust
#[derive(Event)]
pub struct Transfer {
    #[indexed]
    pub from: Address,
    #[indexed]
    pub to: Address,
    pub amount: U256,
}

#[derive(Event)]
pub struct Approval {
    #[indexed]
    pub owner: Address,
    #[indexed]
    pub spender: Address,
    pub amount: U256,
}
```

### Error Types

Custom error handling provides clear feedback:

```rust
#[derive(Error)]
pub enum ERC20Error {
    OnlyOwner,
    InsufficientBalance(U256),
    InsufficientAllowance(U256),
    SelfApproval,
    SelfTransfer,
    ZeroAmount,
    ZeroAddress,
}
```

## Core Functions

### Constructor

```rust
pub fn new(owner: Address) -> Self
```

Initializes a new ERC20 token contract with the specified owner address.

### Minting

```rust
pub fn mint(&mut self, to: Address, amount: U256) -> Result<bool, ERC20Error>
```

Creates new tokens and assigns them to the specified address. Only callable by the contract owner.

**Requirements:**
- Caller must be the contract owner
- Amount must be greater than zero
- Recipient address must not be zero

### Token Transfers

```rust
pub fn transfer(&mut self, to: Address, amount: U256) -> Result<bool, ERC20Error>
```

Transfers tokens from the caller's account to the specified recipient.

```rust
pub fn transfer_from(&mut self, from: Address, to: Address, amount: U256) -> Result<bool, ERC20Error>
```

Transfers tokens on behalf of another address (requires prior approval).

### Approvals

```rust
pub fn approve(&mut self, spender: Address, amount: U256) -> Result<bool, ERC20Error>
```

Approves another address to spend tokens on behalf of the caller.

### View Functions

```rust
pub fn balance_of(&self, owner: Address) -> U256
pub fn allowance(&self, owner: Address, spender: Address) -> U256
pub fn total_supply(&self) -> U256
pub fn owner(&self) -> Address
```

## Deployment Guide

### Prerequisites

Ensure you have the Hybrid development environment set up:

1. **Rust Nightly Toolchain**
   ```bash
   rustup install nightly-2025-01-07
   rustup component add rust-src --toolchain nightly-2025-01-07
   ```

2. **RISC-V Target**
   ```bash
   rustup target add riscv64imac-unknown-none-elf --toolchain nightly-2025-01-07
   ```

### Building the Contract

Navigate to the contract directory and build:

```bash
cd contracts/erc20
cargo +nightly-2025-01-07 build -r --lib -Z build-std=core,alloc --target riscv64imac-unknown-none-elf --bin runtime
```

This generates the optimized RISC-V binary ready for deployment on Hybrid.

### Project Structure

```
erc20/
├── .cargo/
│   └── config.toml          # Build configuration
├── src/
│   └── lib.rs              # Contract implementation
├── Cargo.toml              # Dependencies and metadata
├── hybrid-rust-rt.x        # Linker script
└── README.md               # Build instructions
```

### Configuration

The `.cargo/config.toml` configures the build for the RISC-V target:

```toml
[target.riscv64imac-unknown-none-elf]
rustflags = [
  "-C", "link-arg=-T./hybrid-rust-rt.x",
  "-C", "llvm-args=--inline-threshold=275"
]

[build]
target = "riscv64imac-unknown-none-elf"
```

## Usage Examples

### Basic Token Operations

```rust
// Deploy contract with owner
let mut token = ERC20::new(owner_address);

// Mint initial supply
token.mint(owner_address, U256::from(1_000_000))?;

// Transfer tokens
token.transfer(recipient_address, U256::from(100))?;

// Approve spending
token.approve(spender_address, U256::from(50))?;

// Transfer on behalf
token.transfer_from(owner_address, recipient_address, U256::from(25))?;
```

### Reading Token Data

```rust
// Check balance
let balance = token.balance_of(user_address);

// Check allowance
let allowance = token.allowance(owner_address, spender_address);

// Get total supply
let supply = token.total_supply();
```

## Security Considerations

1. **Owner Privileges**: The contract owner has minting privileges. Consider implementing timelock or multisig controls for production deployments.

2. **Integer Overflow**: Rust's built-in overflow protection and U256 arithmetic provide safety against common vulnerabilities.

3. **Zero Address Checks**: The contract prevents transfers to the zero address, protecting against token burns.

4. **Self-Transfer Prevention**: Prevents wasteful self-transfers that would only consume gas.

## Differences from Solidity ERC20

1. **Memory Safety**: Rust's ownership system prevents common bugs like buffer overflows and use-after-free errors.

2. **Explicit Error Handling**: Result types make error cases explicit and must be handled.

3. **Gas Efficiency**: Rust's zero-cost abstractions and optimized compilation can result in lower gas costs.

4. **Type Safety**: Strong typing prevents many runtime errors common in dynamic languages.

## Testing

The contract can be tested using Hybrid's testing framework. Consider testing:

- All standard ERC20 functionality
- Edge cases (zero amounts, zero addresses)
- Owner-only functions
- Event emission
- Error conditions

## Conclusion

This ERC20 implementation demonstrates Hybrid's capability to run Ethereum-compatible smart contracts while leveraging Rust's safety and performance benefits. The contract maintains full ERC20 compliance while providing enhanced security through Rust's type system and memory safety guarantees.

For more examples and advanced patterns, explore the other contracts in the Hybrid examples collection.