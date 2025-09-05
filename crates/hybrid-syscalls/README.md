# hybrid-syscalls

A system call interface library that bridges RISC-V execution environments with EVM-compatible blockchain operations.

## Overview

`hybrid-syscalls` defines the system call interface used by the Hybrid blockchain framework to enable RISC-V compiled smart contracts to interact with EVM-compatible blockchain environments. It provides a unified abstraction layer that maps RISC-V system calls to their corresponding EVM opcodes.

## Features

- **EVM Opcode Mapping**: Direct mapping between RISC-V syscalls and EVM opcodes
- **Type Safety**: Rust enums with compile-time guarantees for syscall validity
- **Bidirectional Conversion**: Convert between opcodes, strings, and enum variants
- **Error Handling**: Comprehensive error types for invalid opcodes and parsing failures
- **No-std Compatible**: Works in embedded and blockchain environments

## Quick Start

Add `hybrid-syscalls` to your `Cargo.toml`:

```toml
[dependencies]
hybrid-syscalls = "0.1.0"
```

### Basic Usage

```rust
use hybrid_syscalls::Syscall;
use std::str::FromStr;

// Convert from opcode
let syscall = Syscall::try_from(0x20).unwrap(); // Keccak256
assert_eq!(syscall, Syscall::Keccak256);

// Convert to opcode
let opcode: u8 = Syscall::Caller.into();
assert_eq!(opcode, 0x33);

// Parse from string
let syscall = Syscall::from_str("sload").unwrap();
assert_eq!(syscall, Syscall::SLoad);

// Display as string
println!("{}", Syscall::Call); // prints "call"
```

## Supported System Calls

The library provides comprehensive coverage of EVM opcodes and blockchain operations:

### Cryptographic Operations
- **`Keccak256` (0x20)**: Compute Keccak-256 hash
  - Args: memory offset, size
  - Returns: 32-byte hash

### Environment Information
- **`Origin` (0x32)**: Get transaction origin address
- **`Caller` (0x33)**: Get message sender address  
- **`CallValue` (0x34)**: Get value sent with current call
- **`GasPrice` (0x3A)**: Get transaction gas price
- **`Timestamp` (0x42)**: Get current block timestamp
- **`Number` (0x43)**: Get current block number
- **`GasLimit` (0x45)**: Get current block gas limit
- **`ChainId` (0x46)**: Get current chain ID
- **`BaseFee` (0x48)**: Get current block base fee

### Memory and Data Operations
- **`ReturnDataSize` (0x3D)**: Get size of return data from last call
- **`ReturnDataCopy` (0x3E)**: Copy return data to memory
  - Args: memory offset, return data offset, size

### Storage Operations
- **`SLoad` (0x54)**: Load value from contract storage
  - Args: 256-bit storage key
  - Returns: 256-bit storage value
- **`SStore` (0x55)**: Store value in contract storage
  - Args: 256-bit storage key, 256-bit storage value

### Contract Operations
- **`Create` (0xF0)**: Create new contract
  - Args: value, calldata offset, calldata size
  - Returns: new contract address
- **`Call` (0xF1)**: Call another contract
  - Args: address, value, calldata offset, calldata size
- **`StaticCall` (0xFA)**: Static call to another contract
  - Args: address, calldata offset, calldata size

### Control Flow
- **`Return` (0xF3)**: Return data and halt execution
  - Args: data offset, data size
- **`Revert` (0xFD)**: Revert transaction and halt execution

### Event Logging
- **`Log` (0xA0)**: Emit log entry
  - Args: data offset, data size, topics

### RISC-V Extensions
- **`ReturnCreateAddress` (0x01)**: Get address of contract created in current transaction

## Architecture

### Syscall Enum

The core `Syscall` enum represents all supported system calls:

```rust
#[repr(u8)]
pub enum Syscall {
    Keccak256 = 0x20,
    Origin = 0x32,
    Caller = 0x33,
    // ... more syscalls
}
```

### Conversions

The library provides multiple conversion traits:

```rust
// From opcode to syscall
impl TryFrom<u8> for Syscall { ... }

// From syscall to opcode  
impl From<Syscall> for u8 { ... }

// From string to syscall
impl FromStr for Syscall { ... }

// From syscall to string
impl Display for Syscall { ... }
```

### Error Handling

Comprehensive error handling for invalid operations:

```rust
#[derive(Debug, Error)]
pub enum Error {
    #[error("Unknown syscall opcode: {0}")]
    UnknownOpcode(u8),
    
    #[error("Parse error for syscall string. Input: {input}")]
    ParseError { input: Cow<'static, str> },
}
```

## Usage in Smart Contracts

When writing smart contracts with the Hybrid framework, syscalls are typically invoked through higher-level APIs:

```rust
use hybrid_contract::*;

#[contract]
impl MyContract {
    pub fn get_caller(&self) -> Address {
        // Internally uses Syscall::Caller
        caller()
    }
    
    pub fn store_value(&mut self, key: U256, value: U256) {
        // Internally uses Syscall::SStore  
        sstore(key, value);
    }
    
    pub fn load_value(&self, key: U256) -> U256 {
        // Internally uses Syscall::SLoad
        sload(key)
    }
}
```

## EVM Compatibility

The syscall opcodes directly correspond to EVM opcodes, ensuring compatibility:

| Syscall | EVM Opcode | Description |
|---------|------------|-------------|
| `Keccak256` | `SHA3` (0x20) | Keccak-256 hash function |
| `Origin` | `ORIGIN` (0x32) | Transaction origin |
| `Caller` | `CALLER` (0x33) | Message sender |
| `CallValue` | `CALLVALUE` (0x34) | Wei sent with message |
| `SLoad` | `SLOAD` (0x54) | Load from storage |
| `SStore` | `SSTORE` (0x55) | Store to storage |
| `Call` | `CALL` (0xF1) | Message call |
| `Return` | `RETURN` (0xF3) | Halt and return data |

## Implementation Details

### Macro Generation

The syscalls are defined using a declarative macro that generates all the boilerplate:

```rust
syscalls!(
    (0x20, Keccak256, "keccak256"),
    (0x33, Caller, "caller"),
    // ...
);
```

This generates:
- The `Syscall` enum with proper discriminants
- `Display` implementation for string conversion
- `FromStr` implementation for parsing
- `From<Syscall>` and `TryFrom<u8>` implementations

### Memory Layout

The syscall enum uses `#[repr(u8)]` to ensure the discriminants match the EVM opcodes exactly, enabling zero-cost conversion between the enum and raw opcodes.

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_conversion() {
        assert_eq!(Syscall::try_from(0x20).unwrap(), Syscall::Keccak256);
        assert_eq!(u8::from(Syscall::Caller), 0x33);
    }

    #[test]  
    fn test_string_conversion() {
        assert_eq!(Syscall::from_str("sload").unwrap(), Syscall::SLoad);
        assert_eq!(Syscall::Call.to_string(), "call");
    }

    #[test]
    fn test_invalid_opcode() {
        assert!(Syscall::try_from(0xFF).is_err());
    }
}
```

## Contributing

When adding new syscalls:

1. Add the syscall to the `syscalls!` macro with the appropriate opcode
2. Ensure the opcode doesn't conflict with existing ones
3. Add documentation for the syscall's parameters and behavior
4. Add tests for the new syscall
5. Update this README with the new syscall information

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.