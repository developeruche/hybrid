# Hybrid Framework Architecture

This document provides a comprehensive overview of the Hybrid blockchain framework architecture, detailing how the framework enables developers to write smart contracts in Rust that compile to RISC-V bytecode and execute on EVM-compatible blockchains.

## Overview

The Hybrid Framework is a comprehensive blockchain development stack that bridges Rust and Ethereum ecosystems. It allows developers to write smart contracts in Rust, compile them to RISC-V bytecode, and deploy them on any EVM-compatible blockchain. The framework includes a complete toolchain from development to deployment, including a custom Ethereum node implementation.

## Architecture Components

```
┌─────────────────────────────────────────────────────────────┐
│                    Developer Interface                       │
├─────────────────────────────────────────────────────────────┤
│  cargo-hybrid CLI  │        Contract Templates              │
│  (new, build,      │        (ERC20, Storage, etc.)          │
│   check, deploy,   │                                        │
│   node)            │                                        │
├─────────────────────────────────────────────────────────────┤
│                    Compilation Layer                        │
├─────────────────────────────────────────────────────────────┤
│  hybrid-compile    │  hybrid-derive     │  hybrid-contract  │
│  (Rust→RISC-V      │  (#[contract],     │  (Runtime libs,   │
│   compiler)        │   #[storage],      │   syscalls)       │
│                    │   #[Event], etc.)  │                   │
├─────────────────────────────────────────────────────────────┤
│                    Execution Layer                          │
├─────────────────────────────────────────────────────────────┤
│     hybrid-vm      │    hybrid-syscalls │      rvemu        │
│  (EVM integration, │  (RISC-V↔EVM       │  (RISC-V emulator │
│   contract exec)   │   syscall bridge)  │   implementation) │
├─────────────────────────────────────────────────────────────┤
│                    Node Layer                               │
├─────────────────────────────────────────────────────────────┤
│   hybrid-ethereum  │       hybrid-node  │      Reth         │
│  (Custom executor, │   (Standalone node │   (Ethereum       │
│   payload builder) │    binary)         │    client)        │
└─────────────────────────────────────────────────────────────┘
```

## Core Crates

### hybrid-derive

**Purpose**: Procedural macro crate providing contract development primitives

**Location**: `crates/hybrid-derive` (excluded from workspace for compilation reasons)

**Responsibilities**:
- Contract definition through `#[contract]` attribute
- Storage layout management with `#[storage]`
- Event definitions with `#[derive(Event)]`
- Error handling with `#[derive(Error)]`
- Function attribute macros (`#[payable]`, etc.)
- ABI-compatible type conversions and encoding
- Function selector computation (Keccak-256 based)

**Key Features**:
```rust
#[storage]
struct MyContract {
    balance: Mapping<Address, Slot<U256>>,
    owner: Slot<Address>,
}

#[contract]
impl MyContract {
    #[payable]
    pub fn transfer(&mut self, to: Address, amount: U256) -> Result<bool, Error> {
        // Contract logic here
    }
}
```

### hybrid-contract

**Purpose**: Runtime library providing contract execution environment

**Location**: `crates/hybrid-contract` (excluded from workspace)

**Responsibilities**:
- No-std runtime environment for contracts
- Standard library replacements (`hstd` module)
- System call wrappers and utilities
- Memory management for contract execution
- Integration with `hybrid-syscalls` for EVM operations

### hybrid-compile

**Purpose**: Rust-to-RISC-V compilation pipeline

**Location**: `crates/hybrid-compile`

**Responsibilities**:
- Contract project discovery and validation
- Dependency resolution and management
- Multi-stage compilation (runtime + deploy)
- Target: `riscv64imac-unknown-none-elf`
- Binary optimization and generation
- Integration with Rust's `build-std` feature

**Key Features**:
- Automatic contract validation
- Progress tracking during compilation
- Support for interface-only dependencies
- Error reporting and diagnostics

### hybrid-syscalls

**Purpose**: System call interface bridging RISC-V and EVM environments

**Location**: `crates/hybrid-syscalls`

**Responsibilities**:
- Define syscall opcodes and EVM opcode mappings
- Type-safe syscall enumeration with bidirectional conversion
- No-std compatibility for embedded environments
- Error handling for invalid syscalls

**Key Mappings**:
```rust
// Direct EVM opcode mappings
0x20 → Keccak256
0x33 → Caller
0x34 → CallValue
0x35 → CallDataLoad
0x51 → MLoad
0x52 → MStore
// ... and more
```

### hybrid-vm

**Purpose**: Virtual machine executing RISC-V contracts within EVM context

**Location**: `crates/hybrid-vm`

**Responsibilities**:
- Integration with Reth's execution framework
- RISC-V contract execution using `rvemu`
- EVM state management and storage operations
- Syscall translation between RISC-V and EVM
- Contract call dispatch and result handling
- Memory management and sandboxing

**Architecture**:
```rust
pub mod api;           // VM interface definitions
pub mod eth_hybrid;    // Ethereum-specific integration
pub mod evm;          // EVM execution context
pub mod execution;    // Contract execution logic
pub mod frame;        // Execution frame management
pub mod handler;      // Call/syscall handling
pub mod setup;        // VM initialization
```

### hybrid-ethereum

**Purpose**: Custom Ethereum node implementation with RISC-V contract support

**Location**: `crates/hybrid-ethereum`

**Dependencies**: Built on Reth (Ethereum client implementation)

**Responsibilities**:
- Custom executor implementation (`HybridExecutorBuilder`)
- Payload builder for block construction
- Integration with existing Ethereum infrastructure
- RPC server with standard Ethereum JSON-RPC API
- Support for both EVM and RISC-V contract execution

**Features**:
- Full Ethereum node compatibility
- Dev mode for testing and development
- Standard Ethereum tooling integration (web3, hardhat, foundry)

### rvemu

**Purpose**: RISC-V emulator implementation

**Location**: `crates/rvemu`

**Responsibilities**:
- Complete RISC-V instruction set implementation
- Memory management (DRAM, ROM)
- CPU state management and registers
- Interrupt and exception handling
- Bus architecture for device communication

**Components**:
```rust
pub mod bus;        // Memory bus implementation
pub mod cpu;        // CPU core and instruction execution
pub mod csr;        // Control and Status Registers
pub mod devices;    // Device implementations
pub mod dram;       // Dynamic RAM implementation
pub mod emulator;   // Main emulator interface
pub mod exception;  // Exception handling
pub mod interrupt;  // Interrupt management
pub mod rom;        // Read-only memory
```

## Binary Tools

### cargo-hybrid

**Purpose**: Command-line interface for contract development

**Location**: `bins/cargo-hybrid`

**Commands**:
- `cargo hybrid new <name>` - Create new contract project
- `cargo hybrid build` - Compile contract to RISC-V bytecode
- `cargo hybrid check` - Syntax and type checking
- `cargo hybrid deploy` - Deploy contract to blockchain
- `cargo hybrid node` - Start development node

**Features**:
- Project scaffolding with templates
- Integration with Alloy for blockchain interaction
- Progress tracking and colored output
- Template management and initialization

### hybrid-node

**Purpose**: Standalone Ethereum node with RISC-V support

**Location**: `bins/hybrid-node`

**Responsibilities**:
- Standalone node operation
- Development and production modes
- Full Ethereum compatibility
- RISC-V contract execution capability

## Data Flow

### Contract Development Workflow

1. **Project Creation**:
   ```bash
   cargo hybrid new my_contract
   cd my_contract
   ```

2. **Contract Implementation**:
   ```rust
   #![no_std]
   #![no_main]
   
   use hybrid_derive::{contract, storage, Event, Error};
   
   #[storage]
   struct MyContract {
       // Storage fields
   }
   
   #[contract]
   impl MyContract {
       pub fn new() -> Self {
           // Constructor logic
       }
       
       pub fn my_function(&mut self) -> Result<(), Error> {
           // Contract logic
       }
   }
   ```

3. **Compilation Process**:
   ```bash
   cargo hybrid build
   ```
   - `hybrid-compile` discovers and validates contract
   - Compiles Rust code to RISC-V bytecode
   - Generates both runtime and deployment bytecode
   - Outputs binary to `out/` directory

4. **Deployment**:
   ```bash
   cargo hybrid deploy --rpc-url <url> --private-key <key>
   ```
   - Deploys bytecode to target blockchain
   - Returns contract address and transaction hash

### Contract Execution Flow

1. **Transaction Initiation**: Standard Ethereum transaction sent to contract address
2. **Node Processing**: `hybrid-ethereum` receives transaction via JSON-RPC
3. **Executor Dispatch**: `HybridExecutorBuilder` determines execution type
4. **VM Initialization**: `hybrid-vm` initializes RISC-V execution environment
5. **Bytecode Loading**: Contract bytecode loaded into `rvemu`
6. **Execution**: RISC-V instructions executed with syscall translation
7. **State Updates**: Storage operations mapped to EVM SSTORE/SLOAD
8. **Result Return**: Execution results encoded and returned

### Syscall Translation Process

```
RISC-V Contract Code
        ↓
   Syscall Invocation (hybrid-contract)
        ↓
   Syscall Enum (hybrid-syscalls)
        ↓
   EVM Operation Mapping (hybrid-vm)
        ↓
   Reth EVM Execution (hybrid-ethereum)
```

## Type System Integration

### Rust to Solidity ABI Mapping

| Rust Type | Solidity Type | Encoding | Notes |
|-----------|---------------|----------|-------|
| `Address` | `address` | 20 bytes | Ethereum address |
| `U256` | `uint256` | 32 bytes | Unsigned integer |
| `bool` | `bool` | 1 byte (padded) | Boolean value |
| `String` | `string` | Dynamic | UTF-8 encoded |
| `Vec<T>` | `T[]` | Dynamic array | Length-prefixed |
| `[T; N]` | `T[N]` | Fixed array | Static size |
| Custom types | `struct` | Packed encoding | Via derive macros |

### Storage Layout

```rust
#[storage]
struct Contract {
    field1: Slot<U256>,              // Storage slot 0
    field2: Mapping<Address, Slot<U256>>, // Keccak-based mapping
    field3: Slot<bool>,              // Storage slot 1
}
```

Storage slots are automatically assigned and optimized by `hybrid-derive`.

## Security Model

### Isolation Boundaries

1. **Process Isolation**: Each contract runs in isolated RISC-V environment
2. **Memory Safety**: Rust ownership system prevents memory vulnerabilities  
3. **Syscall Whitelisting**: Only EVM-equivalent operations permitted
4. **Type Safety**: Compile-time ABI compatibility guarantees
5. **Deterministic Execution**: RISC-V execution is deterministic and reproducible

### Attack Surface Analysis

**Reduced Attack Vectors**:
- No dynamic loading or code injection
- Limited syscall surface area
- Memory-safe language (Rust)
- Formal type system verification

**Potential Risks**:
- RISC-V emulator bugs
- Syscall translation errors
- Resource exhaustion attacks
- Gas model accuracy

## Performance Characteristics

### Compilation Performance
- **Cold compilation**: ~10-30 seconds for simple contracts
- **Incremental builds**: ~2-5 seconds with cached dependencies
- **Binary size**: ~10-50KB for typical contracts

### Runtime Performance
- **RISC-V overhead**: ~10-100x slower than native EVM
- **Syscall translation**: Minimal additional overhead
- **Memory usage**: Comparable to EVM contracts
- **Gas consumption**: Higher due to emulation overhead

### Optimization Strategies
- **Rust optimization**: `-Oz` flag for size optimization
- **LTO**: Link-time optimization enabled
- **Dead code elimination**: Automatic unused code removal
- **Future**: JIT compilation for hot code paths

## Integration Points

### Ethereum Ecosystem Compatibility

**Development Tools**:
- **Hardhat**: Full compatibility via JSON-RPC
- **Foundry**: Standard testing and deployment
- **Remix**: Browser-based development
- **Web3 libraries**: No changes required

**Infrastructure**:
- **Block explorers**: Standard transaction formats
- **Wallets**: No user-facing changes
- **Monitoring**: Standard Ethereum metrics
- **Oracles**: Standard integration patterns

### Rust Ecosystem Integration

**Development**:
- **Cargo**: Standard Rust package management
- **Testing**: `cargo test` for unit testing
- **Documentation**: `cargo doc` for API docs
- **Linting**: `clippy` for code quality

**Libraries**:
- **Alloy**: Blockchain interaction
- **Serde**: Serialization support
- **no-std**: Embedded environment compatibility

## Deployment Architectures

### Single Contract Deployment
```
Contract.rs → hybrid-compile → RISC-V bytecode → EVM deployment
```

### Multi-Contract Systems
```rust
// Interface sharing between contracts
#[interface]
trait TokenInterface {
    fn transfer(&mut self, to: Address, amount: U256) -> bool;
}

// Cross-contract calls with type safety
let token: TokenInterface = contract_at(token_address);
token.transfer(recipient, amount)?;
```

### Upgrade Patterns
```solidity
// Proxy pattern compatibility
contract Proxy {
    address implementation;
    
    fallback() external {
        // Delegate to RISC-V implementation
        assembly { 
            // Standard proxy delegation
        }
    }
}
```

## Error Handling and Debugging

### Compile-Time Errors
- **Syntax errors**: Standard Rust compiler diagnostics
- **Type errors**: Rust type system enforcement
- **ABI errors**: Custom diagnostics from `hybrid-derive`
- **Dependency errors**: Cargo resolution failures

### Runtime Errors
- **Contract reverts**: Rust `Result` types mapped to EVM reverts
- **VM errors**: RISC-V emulation failures
- **Syscall errors**: Invalid or unsupported operations
- **Gas errors**: Standard Ethereum out-of-gas handling

### Debugging Support
- **Local testing**: `cargo test` with mock environments
- **Tracing**: Structured logging throughout execution
- **State inspection**: Debug modes for contract state
- **Gas profiling**: Detailed gas consumption analysis

## Future Roadmap

### Near-term Enhancements
1. **String storage**: Native string type support in contracts
2. **Advanced types**: More sophisticated Rust type mappings
3. **Debugging tools**: Source-level debugging support
4. **Performance**: JIT compilation for hot paths

### Long-term Vision
1. **Native opcodes**: Custom EVM opcodes for RISC-V operations
2. **Parallel execution**: Multi-threaded contract execution
3. **Formal verification**: Integration with verification tools
4. **Cross-chain**: Support for other blockchain platforms

### Ecosystem Development
1. **Library ecosystem**: Standard contract libraries
2. **Developer tooling**: IDE integration and plugins
3. **Educational resources**: Documentation and tutorials
4. **Community**: Developer community and governance

## Conclusion

The Hybrid Framework represents a comprehensive solution for blockchain development that bridges Rust and Ethereum ecosystems. Through its layered architecture—from high-level contract abstractions to low-level RISC-V emulation—the framework provides:

- **Developer Productivity**: Familiar Rust development environment
- **Type Safety**: Compile-time correctness guarantees  
- **Ecosystem Compatibility**: Seamless Ethereum integration
- **Full Toolchain**: Complete development-to-deployment pipeline
- **Performance**: Optimized execution with future enhancement potential
- **Security**: Multiple isolation layers and memory safety

This architecture enables a new paradigm for smart contract development while maintaining full compatibility with existing Ethereum infrastructure and tooling.