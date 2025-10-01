---
description: Hybrid Overview 
---

# Hybrid Framework Architecture Overview

The Hybrid Framework implements a novel dual-VM architecture that enables seamless execution of both RISC-V and EVM bytecode within a single blockchain environment. At its core, the framework features a **HybridEvm** orchestrator that manages interactions between a RISC-V emulator and a lightweight EVM interpreter, providing full compatibility with the Ethereum ecosystem while extending capabilities to RISC-V compiled smart contracts.

### Related Documentation

- **Hybrid VM Core** - Core virtual machine implementation
- **Mini EVM Interpreter** - Lightweight EVM interpreter details
- **RISC-V Emulation** - RISC-V emulation layer
- **Syscall Interface** - Host-guest communication protocol
- **State Serialization** - Context serialization mechanisms
- **Smart Contract Development** - Contract development workflow

### System Overview

The Hybrid Framework's architecture centers on executing EVM bytecode through a custom lightweight EVM interpreter running within a RISC-V emulation environment. This design enables RISC-V compiled smart contracts to interact seamlessly with the EVM ecosystem while maintaining compatibility with existing Ethereum tools and infrastructure.

### Key Benefits

- **Dual execution environments** - Support for both RISC-V and EVM bytecode
- **Ethereum compatibility** - Full interoperability with existing Ethereum tools
- **Flexible contract development** - Write contracts in RISC-V or traditional Solidity
- **Unified state management** - Coherent state across both execution environments

### Core Architecture Components


**Key responsibilities:**
- Serializing execution context for RISC-V environment
- Loading and initializing the mini-EVM interpreter binary
- Managing the RISC-V emulator lifecycle
- Deserializing execution results
- Handling syscall exceptions

**Core methods:**
- `serialize_input()` - Packages execution context for transfer
- `deserialize_output()` - Processes results from RISC-V environment
- `setup_from_mini_elf()` - Loads mini-EVM interpreter binary
- `run_interpreter()` - Main execution loop with exception handling

### 1. RISC-V Execution Environment

**Component:** `rvemu::Emulator`

The RISC-V emulator provides the execution environment for the mini-EVM interpreter. It implements a complete RISC-V CPU emulation with memory management and exception handling capabilities.

**Features:**
- Full RISC-V instruction set support
- Memory-mapped I/O for syscall communication
- Exception-based host interaction
- Register-based parameter passing

### 2. Mini EVM Interpreter

**Location:** `bins/mini-evm-interpreter/`

A lightweight EVM implementation compiled to RISC-V bytecode that handles EVM instruction execution within the RISC-V environment.

**Components:**
- `mini_instruction_table` - 256-entry opcode dispatch table
- `ext_opcode` module - Host interface wrappers
- Instruction implementations for all EVM opcodes

### 3. Syscall Interface

**Memory address:** `0xBEC00000` (MINI_EVM_SYSCALLS_MEM_ADDR)

The syscall interface provides the communication bridge between the RISC-V environment and the host EVM context. It uses a combination of register-based parameter passing and dedicated memory regions.

**Syscall IDs (10-20):**
- `HOST_BALANCE` (10) - Query account balance
- `HOST_SLOAD` (15) - Load from storage
- `HOST_SSTORE` (16) - Store to storage
- `HOST_BLOCK_NUMBER` (13) - Get block number
- Additional host functions (11-20)

**Register conventions:**
- `x5 (t0)` - Syscall ID
- `x10-x12` - Address parameters
- `x13-x16` - Key/value limbs
- `x31` - Output size indicator

### 5. Node Integration

**Entry point:** `bins/hybrid-node/src/main.rs`

The hybrid-node binary integrates the dual-VM system with Ethereum-compatible blockchain infrastructure through a custom Reth implementation.

**Integration layers:**
- `hybrid-ethereum` crate - Custom Reth implementation
- EVM context management via `ContextTr` trait
- Inspector support through `InspectorEvmTr`
- Block processing pipeline integration



### Execution Flow

### Context Transfer Process

1. **Serialization** - Host serializes BlockEnv, TxEnv, and interpreter state
2. **Binary loading** - Mini-EVM interpreter ELF loaded into RISC-V memory
3. **Emulator start** - RISC-V emulator begins execution
4. **Instruction processing** - EVM opcodes executed via instruction table
5. **Host interaction** - Syscalls (ecall) trigger when host functions needed
6. **Result return** - Execution completes with result in register x31
7. **Deserialization** - Host processes and integrates results

### Host Interaction Model

When the mini-EVM interpreter needs to interact with blockchain state or environment data:

1. Interpreter triggers `EnvironmentCallFromMMode` exception
2. Host examines syscall ID in register t0 (x5)
3. Host dispatches to appropriate function (balance, sload, sstore, etc.)
4. Result written to memory at `0xBEC00000`
5. Execution returns to RISC-V environment
6. Interpreter continues processing



### Memory Architecture

### RISC-V Memory Space Layout

```
0x80000000    DRAM_BASE (Main program memory)
...
0xBEC00000    MINI_EVM_SYSCALLS_MEM_ADDR (Last 20MB)
              - Syscall communication region
              - Interpreter output region
```

The dedicated syscall memory region enables efficient data transfer between the RISC-V environment and the host without complex serialization overhead for host function results.



### EVM Instruction Processing

The mini-EVM interpreter implements all 256 EVM opcodes through a dispatch table architecture:

### Instruction Categories

- **Arithmetic** - ADD, MUL, SUB, DIV, MOD, etc.
- **Stack operations** - PUSH, POP, DUP, SWAP
- **Memory operations** - MLOAD, MSTORE, MCOPY
- **Storage operations** - SLOAD, SSTORE (via syscalls)
- **Control flow** - JUMP, JUMPI, CALL, RETURN
- **Host interface** - BALANCE, EXTCODE*, etc. (via syscalls)

Instructions requiring blockchain state access are implemented through the syscall interface, while pure computational operations execute directly within the RISC-V environment.



### State Management

### Serialization Components

The system maintains execution state coherency through:

- **Input serialization** - Complete execution context packaging
- **Output deserialization** - Result processing and integration
- **Embedded interpreter** - Mini-EVM binary included in host binary
- **Dynamic loading** - Interpreter loaded on-demand per execution

This architecture ensures that state transitions within the RISC-V environment are properly reflected in the broader blockchain state.



## Technical Specifications

- **Target architecture:** RISC-V (RV64)
- **EVM compatibility:** Full Ethereum opcode support
- **Syscall region:** 20MB dedicated memory
- **Base address:** 0x80000000 (DRAM)
- **Syscall address:** 0xBEC00000

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────┐
│                    Hybrid Node                          │
│  ┌────────────────────────────────────────────────┐     │
│  │         hybrid-ethereum (Custom Reth)          │     │
│  │                                                │     │
│  │  ┌──────────────────────────────────────────┐  │     │
│  │  │          HybridEvm Orchestrator          │  │     │
│  │  │                                          │  │     │
│  │  │  ┌────────────────────────────────────┐  │  │     │
│  │  │  │   RISC-V Emulator (rvemu)          │  │  │     │
│  │  │  │                                    │  │  │     │
│  │  │  │  ┌──────────────────────────────┐  │  │  │     │
│  │  │  │  │  mini-evm-interpreter (ELF)  │  │  │  │     │
│  │  │  │  │                              │  │  │  │     │
│  │  │  │  │  • Instruction Table (256)   │  │  │  │     │
│  │  │  │  │  • Syscall Interface         │  │  │  │     │
│  │  │  │  │  • EVM Opcode Execution      │  │  │  │     │
│  │  │  │  └──────────────────────────────┘  │  │  │     │
│  │  │  │         ↕ (ecall/exception)        │  │  │     │
│  │  │  └────────────────────────────────────┘  │  │     │
│  │  │                                          │  │     │
│  │  │  Host Functions (Balance, Storage, etc.) │  │     │
│  │  └──────────────────────────────────────────┘  │     │
│  └────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────┘
```