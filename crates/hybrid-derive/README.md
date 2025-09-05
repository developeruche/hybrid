# hybrid-derive

A procedural macro crate for writing smart contracts in Rust that compile to RISC-V and interact with EVM-compatible blockchains.

## Overview

`hybrid-derive` provides a set of derive macros and attributes that enable developers to write smart contracts using familiar Rust syntax while automatically generating the necessary boilerplate for blockchain interaction, ABI encoding/decoding, and contract deployment.

## Features

- **Smart Contract Definition**: `#[contract]` attribute for defining contract logic
- **Error Handling**: `#[derive(Error)]` for custom error types with ABI encoding
- **Event Emission**: `#[derive(Event)]` for blockchain event logging
- **Contract Interfaces**: `#[interface]` for generating type-safe contract interfaces
- **Storage Management**: `#[storage]` for defining persistent contract storage
- **Payment Handling**: `#[payable]` attribute for functions that can receive payments
- **Automatic ABI Generation**: Function selectors and encoding/decoding generated automatically
- **Type Safety**: Full type safety between Rust types and Solidity ABI types

## Quick Start

Add `hybrid-derive` to your `Cargo.toml`:

```toml
[dependencies]
hybrid-derive = "0.1.0"
```

### Basic Contract Example

```rust
use hybrid_derive::{contract, storage, Error, Event};
use alloy_primitives::{Address, U256};

#[storage]
pub struct TokenStorage {
    pub balances: StorageMap<Address, U256>,
    pub total_supply: StorageValue<U256>,
}

#[derive(Error)]
pub enum TokenError {
    InsufficientBalance,
    InvalidAddress,
}

#[derive(Event)]
pub struct Transfer {
    #[indexed]
    pub from: Address,
    #[indexed] 
    pub to: Address,
    pub amount: U256,
}

#[contract]
impl TokenStorage {
    pub fn new(initial_supply: U256) -> Self {
        let mut storage = Self::default();
        storage.total_supply.set(initial_supply);
        storage
    }

    pub fn balance_of(&self, account: Address) -> U256 {
        self.balances.get(account).unwrap_or(U256::ZERO)
    }

    pub fn transfer(&mut self, to: Address, amount: U256) -> Result<(), TokenError> {
        let from = hybrid_contract::caller();
        let from_balance = self.balance_of(from);
        
        if from_balance < amount {
            return Err(TokenError::InsufficientBalance);
        }

        self.balances.set(from, from_balance - amount);
        let to_balance = self.balance_of(to);
        self.balances.set(to, to_balance + amount);

        emit!(Transfer, from, to, amount);
        Ok(())
    }

    #[payable]
    pub fn deposit(&mut self) {
        let caller = hybrid_contract::caller();
        let value = hybrid_contract::msg_value();
        let current_balance = self.balance_of(caller);
        self.balances.set(caller, current_balance + value);
    }
}
```

## Attributes and Macros

### `#[contract]`

The main attribute for defining smart contract implementation. Applied to an `impl` block, it:

- Generates function selectors for all public methods
- Creates dispatch logic for contract calls
- Handles ABI encoding/decoding of parameters and return values
- Generates deployment code when used with `#[storage]`

**Features:**
- Automatic method routing based on function selectors
- Support for payable and non-payable functions
- Error handling with custom error types
- Return value encoding

### `#[storage]`

Defines the persistent storage layout for a contract.

```rust
#[storage]
pub struct MyStorage {
    pub owner: Address,
    pub balance: U256,
    pub users: StorageMap<Address, User>,
}
```

**Features:**
- Automatic storage slot allocation
- Type-safe storage access
- Support for complex nested types
- Generates `default()` constructor

### `#[derive(Error)]`

Generates ABI-compatible error types that can be thrown by contract functions.

```rust
#[derive(Error)]
pub enum MyError {
    Unauthorized,
    InsufficientFunds(U256),
    InvalidAddress { provided: Address, expected: Address },
}
```

**Features:**
- Automatic error selector generation (4-byte signatures)
- ABI encoding/decoding implementation
- Debug trait implementation
- Support for unit, tuple, and struct variants

### `#[derive(Event)]`

Creates events that can be emitted to the blockchain for logging and monitoring.

```rust
#[derive(Event)]
pub struct Transfer {
    #[indexed]
    pub from: Address,
    #[indexed]
    pub to: Address,
    pub amount: U256,
}
```

**Features:**
- Automatic topic generation for indexed fields (up to 3 indexed fields)
- ABI encoding for event data
- Topic 0 generation (event signature hash)
- Integration with `emit!` macro

### `#[interface]`

Generates type-safe interfaces for calling other contracts.

```rust
#[interface]
trait IERC20 {
    fn balance_of(&self, account: Address) -> U256;
    fn transfer(&mut self, to: Address, amount: U256) -> bool;
}
```

**Options:**
- `#[interface("camelCase")]` - Converts snake_case method names to camelCase for compatibility

**Features:**
- Generates struct with contract address
- Type-safe method calls with automatic ABI encoding
- Support for both view and state-changing calls
- Builder pattern for contract instantiation

### `#[payable]`

Marks functions that can receive native token payments.

```rust
#[payable]
pub fn deposit(&mut self) {
    let amount = hybrid_contract::msg_value();
    // Handle deposit logic
}
```

Without this attribute, functions will revert if called with a non-zero value.

## Type Mapping

`hybrid-derive` automatically maps Rust types to Solidity ABI types:

| Rust Type | Solidity Type | Notes |
|-----------|---------------|-------|
| `bool` | `bool` | Boolean values |
| `Address` | `address` | 20-byte addresses |
| `U256`, `U128`, etc. | `uint256`, `uint128`, etc. | Unsigned integers |
| `I256`, `I128`, etc. | `int256`, `int128`, etc. | Signed integers |
| `String` | `string` | Dynamic strings |
| `Bytes` | `bytes` | Dynamic byte arrays |
| `B32`, `B20`, etc. | `bytes32`, `bytes20`, etc. | Fixed-size byte arrays |
| `Vec<T>` | `T[]` | Dynamic arrays |
| `[T; N]` | `T[N]` | Fixed-size arrays |
| `(T1, T2, ...)` | `(T1, T2, ...)` | Tuples |

## Return Types and Error Handling

Methods can return different types to control success/failure behavior:

```rust
// Direct return - wraps in Option, None on failure
pub fn get_balance(&self) -> U256 { ... }

// Option return - Some/None for success/failure  
pub fn try_get_balance(&self) -> Option<U256> { ... }

// Result return - Ok/Err with custom error types
pub fn transfer(&mut self, to: Address, amount: U256) -> Result<(), TransferError> { ... }
```

## Build Features

The crate supports different build modes through Cargo features:

- **Default**: Generates full contract implementation with entry point
- **`interface-only`**: Only generates interface definitions for external use
- **`deploy`**: Generates deployment/initialization code

## Advanced Usage

### Custom Function Selectors

Function selectors are automatically generated from method signatures, but you can verify them:

```rust
use hybrid_derive::helpers::generate_fn_selector;

// Generates selector for "transfer(address,uint256)"
let selector = generate_fn_selector(&method_info, None);
```

### Interface Naming Styles

Convert Rust naming conventions to Solidity conventions:

```rust
#[interface("camelCase")]
trait MyInterface {
    fn get_user_balance(&self, user: Address) -> U256; // becomes getUserBalance
}
```

### Event Emission

Use the `emit!` macro to emit events:

```rust
emit!(Transfer, from_addr, to_addr, amount);
emit!(Approval, owner, spender, allowance);
```

## Examples

See the `/examples` directory for complete contract implementations:

- **ERC20 Token**: Standard fungible token implementation
- **Multi-signature Wallet**: Wallet requiring multiple signatures
- **NFT Contract**: Non-fungible token with metadata
- **Governance Contract**: Voting and proposal system

## Development

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Documentation

```bash
cargo doc --open
```

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.