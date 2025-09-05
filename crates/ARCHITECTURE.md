# Hybrid Framework Architecture

This document provides a comprehensive overview of the Hybrid blockchain framework architecture, focusing on the relationship between the core crates and how they enable RISC-V smart contracts to run on EVM-compatible blockchains.

## Overview

The Hybrid framework is a revolutionary approach to smart contract development that allows developers to write contracts in Rust, compile them to RISC-V bytecode, and execute them on EVM-compatible blockchains. This hybrid approach combines the safety and expressiveness of Rust with the ubiquity of the Ethereum ecosystem.

## Architecture Components

```
┌─────────────────────────────────────────────────────────────┐
│                    Hybrid Framework                         │
├─────────────────────────────────────────────────────────────┤
│  Smart Contract (Rust) + hybrid-derive macros              │
│  ┌─────────────────┐  ┌─────────────────────────────────┐   │
│  │ Contract Logic  │  │ Generated Code                  │   │
│  │ #[contract]     │  │ - Function selectors           │   │
│  │ #[storage]      │  │ - ABI encoding/decoding        │   │
│  │ #[derive(Error)]│  │ - Interface generation         │   │
│  │ #[derive(Event)]│  │ - Deployment code              │   │
│  └─────────────────┘  └─────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│              RISC-V Bytecode Compilation                    │
│                    (rustc + target)                         │
├─────────────────────────────────────────────────────────────┤
│                   Runtime Environment                       │
│  ┌─────────────────┐  ┌─────────────────────────────────┐   │
│  │ RISC-V Emulator │  │ Syscall Interface               │   │
│  │ (rvemu)         │  │ (hybrid-syscalls)               │   │
│  │                 │  │ - EVM opcode mapping            │   │
│  │                 │  │ - Type conversions              │   │
│  └─────────────────┘  └─────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                 EVM-Compatible Blockchain                   │
│              (Ethereum, Polygon, BSC, etc.)                 │
└─────────────────────────────────────────────────────────────┘
```

## Core Crates

### hybrid-derive

**Purpose**: Procedural macro crate for smart contract development

**Responsibilities**:
- Contract definition and code generation
- ABI-compatible type conversions
- Function selector computation
- Interface generation for contract interaction
- Storage layout management
- Event and error handling

**Key Features**:
- `#[contract]` - Main contract attribute for implementation blocks
- `#[storage]` - Persistent storage layout definition
- `#[derive(Error)]` - Custom error types with ABI encoding
- `#[derive(Event)]` - Blockchain event logging
- `#[interface]` - Type-safe contract interfaces
- `#[payable]` - Payment handling for functions

### hybrid-syscalls

**Purpose**: System call interface between RISC-V and EVM environments

**Responsibilities**:
- Define syscall opcodes and their EVM equivalents
- Provide type-safe syscall enum with conversions
- Enable bidirectional opcode/string/enum conversion
- Error handling for invalid syscalls

**Key Features**:
- Direct EVM opcode mapping (0x20 → Keccak256, 0x33 → Caller, etc.)
- String parsing and formatting
- Compile-time syscall validation
- No-std compatibility for embedded environments

## Data Flow

### Contract Development Workflow

1. **Development Phase**:
   ```rust
   use hybrid_derive::{contract, storage, Error, Event};
   
   #[storage]
   struct MyContract { /* storage fields */ }
   
   #[contract]
   impl MyContract {
       pub fn my_function(&mut self) -> Result<(), MyError> {
           // Contract logic using hybrid_contract runtime
       }
   }
   ```

2. **Compilation Phase**:
   - `hybrid-derive` macros expand to generate:
     - Function selectors (Keccak-256 of signature)
     - ABI encoding/decoding logic
     - Dispatch mechanism for contract calls
     - Interface definitions for external interaction

3. **Deployment Phase**:
   - Rust code compiles to RISC-V bytecode
   - Deployment bytecode includes constructor logic
   - Runtime bytecode handles contract execution

4. **Execution Phase**:
   - Contract calls trigger RISC-V emulation
   - Syscalls bridge RISC-V to EVM operations via `hybrid-syscalls`
   - Results encoded and returned to blockchain

### Syscall Translation Process

```
RISC-V Contract          hybrid-syscalls         EVM Blockchain
─────────────────        ───────────────         ──────────────
ecall (t0=0x33)    →     Syscall::Caller   →     CALLER opcode
                         u8::from(syscall)       
                         syscall.to_string()     
```

## Type System Integration

### Rust to Solidity Type Mapping

The framework provides automatic type conversion between Rust and Solidity:

| Rust Type | Solidity ABI | hybrid-derive | hybrid-syscalls |
|-----------|-------------|---------------|-----------------|
| `Address` | `address` | ✓ Auto-convert | ✓ Address passing |
| `U256` | `uint256` | ✓ Encoding/decoding | ✓ Value handling |
| `bool` | `bool` | ✓ Direct mapping | N/A |
| `String` | `string` | ✓ UTF-8 handling | N/A |
| `Vec<T>` | `T[]` | ✓ Dynamic arrays | N/A |
| `[T; N]` | `T[N]` | ✓ Fixed arrays | N/A |

### Function Selector Generation

```rust
// hybrid-derive automatically generates:
// transfer(address,uint256) → 0xa9059cbb
pub fn transfer(&mut self, to: Address, amount: U256) -> bool {
    // Implementation
}
```

## Runtime Architecture

### Contract Execution Model

1. **Call Initiation**: EVM transaction calls contract address
2. **Bytecode Loading**: RISC-V bytecode loaded into emulator
3. **Syscall Handling**: System calls translated via `hybrid-syscalls`
4. **State Mutations**: Storage operations mapped to EVM SSTORE/SLOAD
5. **Return/Revert**: Execution results returned to EVM

### Memory Model

```
RISC-V Memory Space:
┌─────────────────┐ ← High addresses
│   Stack         │
├─────────────────┤
│   Heap          │
├─────────────────┤
│   Contract Code │
├─────────────────┤
│   ABI Data      │
└─────────────────┘ ← Low addresses

EVM Storage Mapping:
Storage Slot 0 → Contract field 0
Storage Slot 1 → Contract field 1
...
```

## Security Model

### Isolation Boundaries

1. **Contract Isolation**: Each contract runs in isolated RISC-V environment
2. **Syscall Validation**: Only whitelisted syscalls allowed
3. **Memory Safety**: Rust ownership system prevents memory vulnerabilities
4. **Type Safety**: Compile-time guarantees for ABI compatibility

### Attack Surface Reduction

- **No Dynamic Loading**: All code compiled statically
- **Limited Syscall Surface**: Only EVM-equivalent operations exposed
- **Deterministic Execution**: RISC-V execution is deterministic
- **Formal Verification**: Rust's type system enables formal analysis

## Performance Characteristics

### Compilation Time
- **hybrid-derive**: Compile-time code generation (no runtime overhead)
- **hybrid-syscalls**: Zero-cost abstractions for syscall handling

### Runtime Performance
- **RISC-V Emulation**: ~10-100x slower than native execution
- **Syscall Overhead**: Minimal translation cost
- **Memory Usage**: Comparable to native smart contracts

### Gas Consumption
- **Additional Overhead**: RISC-V emulation adds gas cost
- **Optimization Opportunities**: Potential for specialized opcodes
- **Trade-offs**: Developer productivity vs execution efficiency

## Deployment Models

### Single Contract Deployment
```
Contract.rs → RISC-V bytecode → EVM deployment
```

### Multi-Contract Systems
```
Interface definitions shared via hybrid-derive
Contract A ←→ Contract B (typed interfaces)
```

### Upgrade Patterns
```
Proxy Contract (EVM) → Implementation Contract (RISC-V)
Upgradeable via proxy pattern
```

## Integration Points

### With Existing EVM Ecosystem

1. **Web3 Libraries**: Standard JSON-RPC interface
2. **Development Tools**: Compatible with Hardhat, Foundry
3. **Block Explorers**: Standard transaction/event formats
4. **Wallets**: No changes required for user interaction

### With Rust Ecosystem

1. **Cargo**: Standard Rust package management
2. **Testing**: `cargo test` for unit testing
3. **Documentation**: `cargo doc` for API documentation
4. **Linting**: `clippy` for code quality

## Future Architecture Evolution

### Planned Enhancements

1. **Native Opcodes**: Custom EVM opcodes for RISC-V operations
2. **JIT Compilation**: Just-in-time compilation for hot paths
3. **Parallel Execution**: Multi-threaded contract execution
4. **Advanced Type System**: More sophisticated type mappings

### Backward Compatibility

- **Syscall Stability**: Syscall interface versioning
- **ABI Compatibility**: Maintained across framework versions
- **Migration Tools**: Automated contract migration utilities

## Conclusion

The Hybrid framework represents a novel approach to smart contract development that bridges two different computational paradigms. By leveraging `hybrid-derive` for high-level contract abstractions and `hybrid-syscalls` for low-level system integration, developers can write smart contracts in Rust while maintaining full compatibility with the EVM ecosystem.

The architecture prioritizes:
- **Developer Experience**: Familiar Rust development environment
- **Type Safety**: Compile-time correctness guarantees
- **EVM Compatibility**: Seamless integration with existing infrastructure
- **Performance**: Optimized execution paths where possible
- **Security**: Reduced attack surface through safe system design

This hybrid approach opens new possibilities for smart contract development while leveraging the strengths of both the Rust and Ethereum ecosystems.