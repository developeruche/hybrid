---
description: Simple Storage Contract Implementation
---

# Storage Contract

This example demonstrates a fundamental storage contract implementation on the Hybrid blockchain. The contract showcases basic state management, event emission, and read/write operations using Hybrid's Rust-based smart contract framework.

## Overview

The Storage contract provides a simple yet comprehensive example of persistent state management on Hybrid. It demonstrates how to store and retrieve data on-chain while maintaining compatibility with Ethereum's storage model. This contract serves as an excellent starting point for developers new to Hybrid development.

## Features

- **Simple State Management**: Store and retrieve a single U256 value
- **Event Emission**: Logs state changes for off-chain monitoring
- **Error Handling**: Demonstrates custom error types and handling
- **Constructor Initialization**: Shows proper contract initialization patterns
- **Gas Efficient**: Optimized for minimal gas consumption

Create a new storage contract using the Hybrid framework:

```bash
cargo hybrid new hybrid-storage --template storage
```

## Contract Architecture

### Storage Structure

```rust
#[storage]
pub struct Storage {
    storage_item: Slot<U256>,
}
```

The contract uses Hybrid's storage primitives:
- `Slot<U256>`: A single storage slot containing a 256-bit unsigned integer

### Events

The contract emits events to track state changes:

```rust
#[derive(Event)]
pub struct StorageSet {
    pub storage_item: U256,
}
```

### Error Types

Custom error handling provides clear feedback:

```rust
#[derive(Error)]
pub enum StorageError {
    FailedToSetStorage
}
```

## Core Functions

### Constructor

```rust
pub fn new(init_item: U256) -> Self
```

Initializes a new Storage contract with an initial value for the storage item.

**Parameters:**
- `init_item`: The initial U256 value to store in the contract

**Example:**
```rust
let storage = Storage::new(U256::from(42));
```

### State Modification

```rust
pub fn set_storage(&mut self, item: U256) -> Result<bool, StorageError>
```

Updates the stored value and emits an event to log the change.

**Parameters:**
- `item`: The new U256 value to store

**Returns:**
- `Ok(true)` on successful storage update
- `Err(StorageError)` if the operation fails

**Events Emitted:**
- `StorageSet` with the new storage value

### State Reading

```rust
pub fn read_item(&self) -> U256
```

Returns the currently stored value without modifying state.

**Returns:**
- The current U256 value stored in the contract

## Deployment Guide

### Prerequisites

Ensure you have the Hybrid development environment configured:

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

Navigate to the contract directory and compile:

```bash
cd contracts/storage
cargo +nightly-2025-01-07 build -r --lib -Z build-std=core,alloc --target riscv64imac-unknown-none-elf --bin runtime
```

This produces an optimized RISC-V binary suitable for deployment on the Hybrid network.

### Project Structure

```
storage/
├── .cargo/
│   └── config.toml          # Build configuration
├── src/
│   └── lib.rs              # Contract implementation
├── Cargo.toml              # Dependencies and metadata
└── hybrid-rust-rt.x        # Custom linker script
```

### Build Configuration

The `.cargo/config.toml` specifies the RISC-V compilation target:

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

### Basic Storage Operations

```rust
// Initialize contract with a starting value
let mut storage_contract = Storage::new(U256::from(100));

// Read the initial value
let initial_value = storage_contract.read_item();
assert_eq!(initial_value, U256::from(100));

// Update the stored value
let result = storage_contract.set_storage(U256::from(200));
assert!(result.is_ok());

// Verify the update
let updated_value = storage_contract.read_item();
assert_eq!(updated_value, U256::from(200));
```

### Event Monitoring

When the storage value is updated, the contract emits a `StorageSet` event:

```rust
// This operation will emit: StorageSet { storage_item: U256::from(42) }
storage_contract.set_storage(U256::from(42))?;
```

Off-chain applications can monitor these events to track contract state changes.

### Error Handling

```rust
match storage_contract.set_storage(new_value) {
    Ok(_) => println!("Storage updated successfully"),
    Err(StorageError::FailedToSetStorage) => println!("Failed to update storage"),
}
```

## Key Concepts Demonstrated

### 1. Storage Slots

The contract demonstrates how to use Hybrid's `Slot<T>` type for persistent storage:

```rust
storage_item: Slot<U256>
```

Storage slots provide persistent, on-chain storage that survives between function calls and transactions.

### 2. State Mutability

The contract shows the distinction between state-reading and state-modifying functions:

- `read_item(&self)`: Read-only access, doesn't modify state
- `set_storage(&mut self)`: Requires mutable reference to modify state

### 3. Event Emission

Events provide a way to log important state changes:

```rust
log::emit(StorageSet::new(item));
```

Events are essential for:
- Off-chain monitoring and indexing
- User interface updates
- Transaction history tracking

### 4. Constructor Patterns

The `new` function demonstrates proper contract initialization:

```rust
pub fn new(init_item: U256) -> Self {
    let mut storage = Storage::default();
    storage.storage_item.write(init_item);
    storage
}
```

## Comparison with Solidity

### Solidity Equivalent

```solidity
contract Storage {
    uint256 private storageItem;
    
    event StorageSet(uint256 storageItem);
    
    constructor(uint256 _initItem) {
        storageItem = _initItem;
    }
    
    function setStorage(uint256 _item) public returns (bool) {
        storageItem = _item;
        emit StorageSet(_item);
        return true;
    }
    
    function readItem() public view returns (uint256) {
        return storageItem;
    }
}
```

### Advantages of the Rust Implementation

1. **Memory Safety**: Rust prevents common vulnerabilities like buffer overflows
2. **Explicit Error Handling**: Result types make error cases visible and mandatory to handle
3. **Zero-Cost Abstractions**: Rust's optimizations can lead to lower gas costs
4. **Strong Typing**: Compile-time type checking prevents many runtime errors

## Testing Strategies

Consider testing the following scenarios:

1. **Initialization**: Verify the constructor sets the initial value correctly
2. **State Updates**: Test that `set_storage` updates the value and emits events
3. **State Persistence**: Confirm that stored values persist between function calls
4. **Error Conditions**: Test any potential failure modes
5. **Gas Usage**: Measure and optimize gas consumption for operations

## Advanced Patterns

This basic storage contract can be extended with:

1. **Access Controls**: Add owner-only modifications
2. **Multiple Storage Slots**: Store different data types
3. **Batch Operations**: Update multiple values in a single transaction
4. **Value Validation**: Add constraints on stored values
5. **Historical Tracking**: Maintain a history of value changes

## Conclusion

The Storage contract provides a foundational example of state management on Hybrid. While simple, it demonstrates core concepts that are essential for building more complex smart contracts. The pattern of initialization, state modification, and state reading forms the basis of most blockchain applications.

This example showcases Hybrid's ability to provide a familiar development experience while leveraging Rust's safety and performance advantages. Developers can build upon this foundation to create sophisticated decentralized applications with confidence in their code's reliability and efficiency.

For more complex examples and advanced patterns, explore the other contracts in the Hybrid examples collection, including the ERC20 token implementation.