---
description: Explain the smart contract pipeline
---

## Smart Contract Development Pipeline

The Hybrid Framework enables Rust developers to write safe, efficient smart contracts using RISC-V architecture while maintaining full EVM compatibility. Contracts leverage Rust's type safety and performance characteristics to create robust blockchain applications.

**Key Features:**
- RISC-V smart contract execution on EVM chains
- Rust-based development with type safety
- Solidity-like storage abstractions
- Two-stage compilation process
- Comprehensive error handling


### Project Structure

#### Required Cargo.toml Configuration

Every Hybrid Framework contract must follow a specific project structure:

```toml
[package]
name = "my-contract"
version = "0.1.0"
edition = "2021"

[features]
default = []
deploy = []
interface-only = []

[[bin]]
name = "runtime"
path = "src/lib.rs"

[[bin]]
name = "deploy"
path = "src/lib.rs"
required-features = ["deploy"]

[dependencies]
hybrid-derive = { path = "../hybrid-derive" }
hybrid-contract = { path = "../hybrid-contract" }
```

**Required Features:**
- `default`: Standard contract features
- `deploy`: Deployment-specific code paths
- `interface-only`: Interface generation without implementation

**Binary Targets:**
- `runtime`: Handles normal contract function calls after deployment
- `deploy`: Executes during contract deployment and initialization

The contract discovery system validates these requirements automatically, performing comprehensive checks including feature validation, binary target validation, and dependency resolution.


### Development Workflow

The contract development process follows a structured workflow:

### Step-by-Step Process

1. **Initialize Project**
   ```bash
   cargo hybrid new my-contract
   ```

2. **Define Storage Structure**
   - Create storage struct with required fields
   - Use `Slot<T>` and `Mapping<K, V>` types

3. **Implement Contract Trait**
   - Add `#[contract]` attribute
   - Implement contract functions

4. **Validate Syntax**
   ```bash
   cargo hybrid check
   ```

5. **Build Contract**
   ```bash
   cargo hybrid build
   ```
   - Compiles runtime binary
   - Compiles deploy binary
   - Generates bytecode with 0xff prefix

6. **Deploy Contract**
   ```bash
   cargo hybrid deploy
   ```

7. **Test Contract Functions**
   - Verify deployment
   - Test contract interactions



### Contract Implementation

#### Basic Contract Structure

```rust
#![no_std]
#![no_main]

use hybrid_contract::*;
use hybrid_contract::hstd::{Slot, Mapping};

// Storage structure
struct MyTokenStorage {
    total_supply: Slot<U256>,
    balances: Mapping<Address, U256>,
    allowances: Mapping<Address, Mapping<Address, U256>>,
}

// Contract implementation
#[contract]
impl MyTokenStorage {
    fn call(&mut self) {
        let sig = msg_sig();
        match sig {
            [0xa9, 0x05, 0x9c, 0xbb] => self.balance_of(),
            [0x23, 0xb8, 0x72, 0xdd] => self.transfer(),
            _ => revert(),
        }
    }
}

// Entry point
#[entry]
fn main() -> ! {
    let mut contract = MyTokenStorage::default();
    contract.call();
}
```

### Key Components

- **Storage Structure**: Defines persistent contract state
- **#[contract] Attribute**: Marks the main contract implementation
- **Function Dispatch**: Routes calls based on function signatures
- **Entry Point**: Main function that initializes and runs the contract



### Runtime Library

The `hybrid-contract` crate provides essential runtime components for smart contract development.

### Storage Abstractions

The `hstd` module offers Solidity-like storage types:

**Slot&lt;T&gt;**
- Single storage slots for any ABI-encodable type
- Direct storage access for simple values

**Mapping&lt;K, V&gt;**
- Key-value mappings with automatic key derivation
- Supports nested mappings
- Efficient storage layout

Both abstractions use `sload` and `sstore` system calls for blockchain storage interaction.

### Environment Access

Access blockchain environment information through dedicated functions:

**Block Information**
```rust
block_number()    // Current block number
block_timestamp() // Block timestamp
base_fee()        // Base fee per gas
chain_id()        // Chain identifier
```

**Transaction Information**
```rust
gas_price()       // Transaction gas price
origin()          // Transaction originator
```

**Message Context**
```rust
msg_sender()      // Current caller address
msg_value()       // Value sent with call
msg_data()        // Call data
msg_sig()         // Function signature
```

### System Call Interface

Core system calls provide EVM compatibility:

| Function | EVM Opcode | Description |
|----------|-----------|-------------|
| `sload(key)` | SLOAD | Read from storage |
| `sstore(key, value)` | SSTORE | Write to storage |
| `msg_sender()` | CALLER | Get caller address |
| `msg_value()` | CALLVALUE | Get sent value |
| `keccak256(offset, size)` | KECCAK256 | Hash computation |
| `return_riscv(addr, size)` | RETURN | Return from contract |


## Compilation Pipeline

The Hybrid Framework uses a two-stage compilation process that generates specialized bytecode for each execution phase.

### Compilation Flow

1. **Runtime Compilation**
   - Compiles contract logic without deploy features
   - Optimized for post-deployment execution
   - Smaller bytecode size

2. **Deploy Compilation**
   - Includes deployment-specific initialization
   - Compiled with `deploy` feature flag
   - Executes only during contract creation

3. **Bytecode Generation**
   - Combines runtime and deploy binaries
   - Prepends 0xff prefix to signal RISC-V execution
   - Final format: `[0xff] + [deploy_bytecode]`

### Runtime vs Deploy Binary

The `deploy` feature flag controls compilation paths:

- **Runtime Binary**: Handles normal contract function calls after deployment
- **Deploy Binary**: Executes during contract deployment and initialization

This separation allows deployment-specific initialization logic without bloating the runtime binary.

### Bytecode Format

```
[0xff] + [deploy_bytecode]
```

The `0xff` prefix signals to the Hybrid VM that this is RISC-V bytecode rather than native EVM bytecode, triggering the appropriate execution path in the dual-VM system.


### Memory Layout

Hybrid contracts execute in a carefully designed memory environment that provides EVM compatibility while running on RISC-V.

#### Memory Organization

Contracts use a structured memory layout that mirrors EVM conventions:

- **Low Memory**: Reserved for EVM scratch space
- **Stack Region**: Function call stack
- **Heap Region**: Dynamic allocations
- **Storage Interface**: Maps to blockchain storage

The runtime uses a custom linker script to ensure proper memory layout and deterministic behavior across all contract executions.



### Error Handling

The runtime includes comprehensive error handling mechanisms to ensure contract safety and debuggability.

#### Panic Handler

- Converts Rust panics to contract reverts
- Includes panic messages in revert data
- Maintains blockchain determinism
- Improves debugging experience

#### Deterministic Allocation

- Bump allocator ensures consistent memory usage
- No system allocator dependencies
- Predictable gas costs
- Reproducible execution

#### System Call Safety

- All EVM interactions validated through system calls
- Type-safe interfaces prevent common errors
- Compile-time checks for storage access
- Runtime validation for dynamic operations



### Additional Resources

For more detailed information, refer to:

- **Compilation Pipeline**: Architecture and optimization details
- **Procedural Macros**: Code generation and macro expansion
- **Contract Runtime Library**: Complete API reference



### Getting Started

To begin developing with the Hybrid Framework:

1. Install the Hybrid toolchain
2. Create a new project: `cargo hybrid new my-contract`
3. Implement your contract logic
4. Build and deploy: `cargo hybrid build && cargo hybrid deploy`