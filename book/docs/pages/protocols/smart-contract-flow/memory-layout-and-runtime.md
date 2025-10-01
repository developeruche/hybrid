---
description: Explore RUST smart contracts and memory layout and runtime
---

# Hybrid Contract Runtime Library

The Contract Runtime Library (hybrid-contract) provides a comprehensive smart contract development framework for the Hybrid VM. This library enables developers to write Rust-based smart contracts that execute on RISC-V architecture while maintaining full EVM compatibility through system calls and ABI compliance.

## Core Architecture

The runtime library serves as the bridge between Rust contract code and the Hybrid VM's dual execution environment. It provides high-level abstractions while maintaining deterministic execution and EVM compatibility.

### Runtime Components

The library consists of several key layers:

- **Memory Layout**: Deterministic memory management with memory-mapped regions
- **System Interface**: RISC-V system calls for VM interaction
- **Storage Abstractions**: High-level storage primitives (`Slot<T>`, `Mapping<K,V>`)
- **Runtime Core**: Message context, transaction functions, and environment utilities
- **Contract Layer**: Entry point macro and Contract trait implementation

## Message Context Functions

The runtime provides access to EVM message context through a set of core functions that correspond to EVM opcodes.

| Function | EVM Opcode | Return Type | Description |
|----------|-----------|-------------|-------------|
| `msg_sender()` | CALLER | Address | Immediate caller address |
| `msg_value()` | CALLVALUE | U256 | Wei sent with call |
| `msg_sig()` | - | [u8; 4] | Function selector |
| `msg_data()` | CALLDATA | &'static [u8] | Complete calldata |

### Function Selector Dispatching

The `msg_sig()` function extracts the 4-byte function selector used for contract function routing:

```rust
// Example usage
let selector = msg_sig();
match selector {
    [0xa9, 0x05, 0x9c, 0xbb] => self.balance_of(), // balanceOf(address)
    [0x23, 0xb8, 0x72, 0xdd] => self.transfer(),   // transfer(address,uint256)
    _ => revert(), // Unknown function
}
```

## Storage System

The runtime provides two primary storage abstractions that map to EVM storage slots while maintaining Rust type safety.

### Storage Operations

| Function | EVM Opcode | Parameters | Description |
|----------|-----------|------------|-------------|
| `sload(key)` | SLOAD | U256 | Read from storage slot |
| `sstore(key, value)` | SSTORE | U256, U256 | Write to storage slot |
| `keccak256(offset, size)` | KECCAK256 | u64, u64 | Hash memory region |

### Storage Abstractions

**Slot&lt;T&gt;**: Single value storage with static position calculation

**Mapping&lt;K,V&gt;**: Key-value storage using `keccak256(key + slot)` for key derivation

**Nested Mappings**: Support for `Mapping<K, Mapping<K2,V>>` patterns

## Transaction Environment

The `tx` module provides access to transaction-level context that remains constant throughout the call chain.

### Transaction Context Functions

**`tx::gas_price()`**: Returns the gas price in wei (GASPRICE opcode)

**`tx::origin()`**: Returns the original EOA address (ORIGIN opcode)

### Security Note

The `tx::origin()` function provides access to the original transaction sender (EOA), but usage for authorization is discouraged due to security vulnerabilities in contract-to-contract call scenarios.

## System Call Interface

All runtime functions ultimately interface with the Hybrid VM through RISC-V system calls using inline assembly.

### System Call Pattern

The runtime uses a consistent pattern for system calls:

1. **Parameter marshalling**: Convert high-level types to u64 register values
2. **Assembly invocation**: Use `asm!("ecall")` with appropriate registers
3. **Result unmarshalling**: Convert returned register values back to types

Example flow:
```
Contract Code → Runtime Function → RISC-V Assembly (ecall) 
  → System Call Handler → Hybrid VM Host → Return values
```

## Contract Lifecycle

### Entry Point

Contracts use the `#[entry]` macro to define the entry point, which sets up the runtime environment and invokes the Contract trait's `call()` method.

### Contract Trait

Contracts must implement the `Contract` trait, which provides the main execution entry point for function dispatching based on `msg_sig()`.

## Memory Management

The runtime uses a custom bump allocator for deterministic memory allocation across different execution environments.

### Memory Layout

| Address Range | Purpose | Access |
|--------------|---------|--------|
| 0x8000_0000 | Calldata mapping | Read-only |
| 0x8000_0000 + 0 | Calldata length (8 bytes) | Read-only |
| 0x8000_0000 + 8 | Calldata content | Read-only |
| Heap | Bump allocated memory | Read-write |

### Calldata Access

Calldata is memory-mapped at address `CALLDATA_ADDRESS` (0x8000_0000), providing efficient read-only access without system calls.

## Error Handling

### Panic Handler

The runtime provides a comprehensive panic handling system that converts Rust panics into contract reverts:

1. Rust `panic!()` is triggered
2. Custom `#[panic_handler]` captures the panic
3. `IS_PANICKING` guard prevents recursion
4. Panic message is captured and converted to bytes
5. `revert_with_error(&[u8])` is called
6. System call to host reverts the transaction

### Revert Functions

| Function | Purpose | Parameters |
|----------|---------|------------|
| `revert()` | Simple revert with no data | None |
| `revert_with_error(data)` | Revert with error message | &[u8] |
| `return_riscv(addr, size)` | Normal return with data | u64, u64 |

## Address Conversion Utilities

The runtime includes utilities for converting between different address representations used in the RISC-V and EVM environments.

### Conversion Functions

**`__address_to_3u64()`**: Converts EVM Address (20 bytes) to 3x u64 register values

**`__3u64_to_address()`**: Converts 3x u64 register values back to EVM Address (20 bytes)

These utilities handle the conversion between 20-byte Ethereum addresses and the 3x u64 register format used for system calls.

