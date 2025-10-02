---
description: Deep dive into the mini evm interpreter
---

# Mini EVM Interpreter Technical Documentation

## Overview

The Mini EVM Interpreter is a lightweight, no-std EVM bytecode execution engine designed specifically for the Hybrid Framework's dual-VM architecture. It enables execution of Ethereum bytecode within RISC-V environments through serialized state management and custom opcode implementations.

**Related Documentation:**
- Hybrid VM Core - Dual-VM orchestration system
- Syscall Interface - RISC-V and EVM environment bridge
- State Serialization - State serialization mechanisms

## Architecture Overview

The Mini EVM Interpreter operates as a self-contained binary that executes EVM bytecode in constrained environments. It interfaces with the broader Hybrid Framework through memory-based communication and serialized state exchange.

### System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│ Mini EVM Interpreter Binary                                  │
├─────────────────────────────────────────────────────────────┤
│  main() [lib.rs:119]                                         │
│  ├─ read_input() [utils.rs:57]                              │
│  ├─ write_output() [utils.rs:143]                           │
│  └─ mini_instruction_table() [instruction_table.rs]         │
├─────────────────────────────────────────────────────────────┤
│ Memory Layout                                                │
│  ├─ CALLDATA_ADDRESS (Input/Output Region)                  │
│  └─ CALLDATA_ADDRESS + 1GB - 2000 (Debug Region)           │
├─────────────────────────────────────────────────────────────┤
│ Serialized State                                             │
│  ├─ ext_revm::Interpreter                                   │
│  ├─ ext_revm::BlockEnv                                      │
│  ├─ ext_revm::TxEnv                                         │
│  └─ ext_revm::Context                                       │
└─────────────────────────────────────────────────────────────┘
```

### Input Format

```
[interpreter_len: u64][block_len: u64][tx_len: u64]
[interpreter_data: bincode][block_data: bincode][tx_data: bincode]
```

### Output Format

```
[interpreter_len: u64][block_len: u64][tx_len: u64][action_len: u64]
[interpreter_data: bincode][block_data: bincode][tx_data: bincode][action_data: bincode]
```

## Execution Flow

The interpreter follows a sequential execution model:

1. **Input Phase**: Host environment writes serialized input to `CALLDATA_ADDRESS`
2. **Binary Execution**: Mini EVM Interpreter binary is invoked
3. **Deserialization**: `copy_from_mem()` reads input byte slice, `deserialize_input()` parses data
4. **Context Setup**: Creates execution context with Interpreter, BlockEnv, and TxEnv
5. **Execution**: `interpreter.run_plain(mini_instruction_table(), context)` executes bytecode
6. **Serialization**: `serialize_output()` packages InterpreterAction result
7. **Output Phase**: `write_to_memory()` writes result and sets t6 register with output length
8. **Result Retrieval**: Host environment reads serialized output from memory

## Core Components

### Main Entry Point

The interpreter's main function serves as the execution entry point, marked with `#[hybrid_contract::entry]`. It orchestrates the complete EVM execution cycle:

| Phase | Function | Description |
|-------|----------|-------------|
| Input | `read_input()` | Deserializes interpreter state, block environment, and transaction environment from memory |
| Setup | Context creation | Configures EVM execution context with chain ID and state journal |
| Execution | `interpreter.run_plain()` | Executes bytecode using custom instruction table |
| Output | `write_output()` | Serializes execution results back to memory |

The function configures a `CfgEnv` with `CHAIN_ID = 1` (Ethereum mainnet) and creates a `Context` with an `EmptyDB` database backend for stateless execution.

**Source:** `bins/mini-evm-interpreter/src/lib.rs:118-154`

### Memory Management System

The interpreter operates with a specific memory layout optimized for RISC-V environments:

#### Memory Layout

```
CALLDATA_ADDRESS (0x80000000)
├─ Length Header [8 bytes: u64]
└─ Serialized Data [Variable length]

Debug Address (CALLDATA_ADDRESS + 1GB - 2000)
└─ Debug Output [Variable length]
```

#### Memory Operations

- `copy_from_mem()` - Reads data from memory regions (`utils.rs:188`)
- `write_to_memory()` - Writes data to memory regions (`utils.rs:219`)
- `debug_println()` - Writes debug output (`utils.rs:79`)

The memory system uses unsafe operations for direct memory access without bounds checking, optimized for performance in the constrained environment.

**Sources:** `bins/mini-evm-interpreter/src/utils.rs:188-222`, `bins/mini-evm-interpreter/hybrid-rust-rt.x:1-27`

## Serialization Framework

The interpreter uses a custom binary serialization format for state exchange.

### Input Data Format

```rust
[interpreter_len: u64][block_len: u64][tx_len: u64]
[interpreter_data: bincode][block_data: bincode][tx_data: bincode]
```

### Output Data Format

```rust
[interpreter_len: u64][block_len: u64][tx_len: u64][action_len: u64]
[interpreter_data: bincode][block_data: bincode][tx_data: bincode][action_data: bincode]
```

### Data Validation

The serialization system includes validation for data integrity and length consistency:

```rust
// Length validation in deserialize_input
if data.len() < 24 { panic!("Data too short for headers"); }
let expected_len = si_len + sb_len + st_len + 24;
if data.len() != expected_len { panic!("Data length mismatch"); }
```

**Sources:** `bins/mini-evm-interpreter/src/utils.rs:265-306`, `bins/mini-evm-interpreter/src/utils.rs:346-376`

## EVM Integration

### External REVM Integration

The interpreter integrates with a custom fork of REVM through the `ext-revm` dependency:

| Component | Type | Purpose |
|-----------|------|---------|
| Interpreter | `ext_revm::interpreter::Interpreter` | EVM bytecode execution engine |
| BlockEnv | `ext_revm::context::BlockEnv` | Block-level execution environment |
| TxEnv | `ext_revm::context::TxEnv` | Transaction-level execution environment |
| Context | `ext_revm::Context` | Complete execution context with journaled state |
| InterpreterAction | `ext_revm::interpreter::InterpreterAction` | Execution result wrapper |

The integration uses a `Journal` with `EmptyDB` for stateless execution, suitable for the hybrid contract environment where state management is handled externally.

**Sources:** `bins/mini-evm-interpreter/src/lib.rs:57-61`, `bins/mini-evm-interpreter/Cargo.toml:24`

### Instruction Table Architecture

The interpreter uses a custom instruction table (`mini_instruction_table()`) that maps EVM opcodes to implementation functions. The architecture supports comprehensive EVM opcode coverage including:

- **Arithmetic operations**: ADD, SUB, MUL, DIV, etc.
- **Bitwise operations**: AND, OR, XOR, shifts
- **Stack operations**: PUSH, POP, DUP, SWAP
- **Memory operations**: MLOAD, MSTORE, MCOPY
- **Storage operations**: SLOAD, SSTORE
- **Control flow**: JUMP, JUMPI, CALL, RETURN
- **System operations**: CREATE, SELFDESTRUCT, LOG

**Sources:** `bins/mini-evm-interpreter/src/lib.rs:64`, `bins/mini-evm-interpreter/README.md:122-162`

## Build Configuration

### RISC-V Target Configuration

The interpreter is specifically configured for RISC-V architecture with no-std constraints:

| Configuration | Value | Purpose |
|--------------|-------|---------|
| Target | `riscv64imac-unknown-none-elf` | 64-bit RISC-V with multiply/atomic extensions |
| Build Mode | `no_std`, `no_main` | Minimal runtime for embedded environments |
| Optimization | `opt-level = "z"`, `lto = true` | Size-optimized binary with link-time optimization |
| Toolchain | `nightly-2025-01-07` | Specific nightly for build-std support |

### Linker Configuration

The linker configuration defines specific memory regions:

```
CALL_DATA    : ORIGIN = 0x80000000, LENGTH = 1M
STACK        : ORIGIN = 0x80100000, LENGTH = 2M  
REST_OF_RAM  : ORIGIN = 0x80300000, LENGTH = 1021M
```

**Sources:** `bins/mini-evm-interpreter/Cargo.toml:35-37`, `bins/mini-evm-interpreter/.cargo/config.toml:1-9`, `bins/mini-evm-interpreter/hybrid-rust-rt.x:3-26`

## Build Process

The interpreter supports two build targets:

### Runtime Binary

Standard execution binary:

```bash
cargo +nightly-2025-01-07 build -r --lib \
  -Z build-std=core,alloc \
  --target riscv64imac-unknown-none-elf \
  --bin runtime
```

### Deploy Binary

Deployment-specific binary with deploy feature:

```bash
cargo +nightly-2025-01-07 build -r --lib \
  -Z build-std=core,alloc \
  --target riscv64imac-unknown-none-elf \
  --bin deploy \
  --features deploy
```

**Sources:** `bins/mini-evm-interpreter/Cargo.toml:26-33`, `bins/mini-evm-interpreter/README.md:83-102`

## Debug and Development Support

### Debug Utilities

The interpreter includes debug utilities for development and troubleshooting:

#### Debug Functions

- `debug_println()` - Outputs static 'Hello, world!' message
- `debug_println_dyn_data()` - Outputs custom byte data

#### Debug Memory

Both functions write directly to a designated debug memory region at `CALLDATA_ADDRESS + 1GB - 2000` without bounds checking, providing minimal overhead debugging capabilities.

**Source:** `bins/mini-evm-interpreter/src/utils.rs:79-110`

### Address Conversion Utilities

The interpreter includes utilities for converting between different address formats:

- `__3u64_to_address()` - Converts three 64-bit limbs to 20-byte Ethereum address
- `__address_to_3u64()` - Converts Ethereum address to three 64-bit limbs
- `serialize_sstore_input()` - Serializes storage operation parameters

These utilities support integration with external systems that may use different address representations.

**Sources:** `bins/mini-evm-interpreter/src/utils.rs:400-416`, `bins/mini-evm-interpreter/src/utils.rs:378-398`

## Integration with Hybrid Framework

The Mini EVM Interpreter operates as a standalone binary within the Hybrid Framework's dual-VM system. It communicates with the host environment through:

- **Memory-based I/O**: All communication occurs through designated memory regions
- **Register Communication**: Uses RISC-V t6 register to signal output length
- **Binary Interface**: No function call interface - purely binary execution
- **Serialized State**: All state exchange through binary serialization

This design enables the interpreter to function as a black-box EVM execution engine that can be invoked by the hybrid VM orchestrator for executing EVM bytecode within RISC-V smart contracts.

**Sources:** `bins/mini-evm-interpreter/src/utils.rs:143-161`, `bins/mini-evm-interpreter/src/lib.rs:152-154`

---

## Quick Reference

### Memory Addresses

- **CALLDATA_ADDRESS**: `0x80000000` (Input/Output region)
- **Debug Address**: `CALLDATA_ADDRESS + 1GB - 2000`
- **Stack**: `0x80100000` (2MB)
- **Additional RAM**: `0x80300000` (1021MB)

### Key Constants

- **CHAIN_ID**: `1` (Ethereum mainnet)
- **Database**: `EmptyDB` (stateless execution)

### File Locations

- Main entry: `bins/mini-evm-interpreter/src/lib.rs`
- Utilities: `bins/mini-evm-interpreter/src/utils.rs`
- Linker script: `bins/mini-evm-interpreter/hybrid-rust-rt.x`
- Configuration: `bins/mini-evm-interpreter/.cargo/config.toml`