---
description: Deep dive into the hybrid virtual machine
---

# Hybrid VM Core Technical Documentation

## Overview

The Hybrid VM Core serves as the central orchestrator for a dual-VM execution environment in the Hybrid Framework. It manages the execution of both RISC-V bytecode (compiled from Rust smart contracts) and traditional EVM bytecode, providing seamless interoperability between the two execution environments through a unified interface.

## Architecture Overview

The Hybrid VM Core implements a dual-execution architecture where a single orchestrator routes execution to either the RISC-V emulator or the mini-EVM interpreter based on bytecode inspection.

```
┌─────────────────────────────────────────────────────────────┐
│                      Hybrid VM Core                          │
│                                                               │
│  ┌──────────────────┐              ┌────────────────────┐   │
│  │  EVM Execution   │              │  RISC-V Execution  │   │
│  │      Path        │              │       Path         │   │
│  │                  │              │                    │   │
│  │  0xFF prefix ──► │              │  Standard EVM      │   │
│  │                  │              │                    │   │
│  │  EVM Context     │              │  rvemu::Emulator   │   │
│  └──────────────────┘              └────────────────────┘   │
│                                                               │
│  ┌───────────────────────────────────────────────────────┐  │
│  │               HybridEvm (Orchestrator)                 │  │
│  │                 Bytecode Detection                     │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                               │
│  ┌──────────────────┐              ┌────────────────────┐   │
│  │  Mini-EVM        │              │  Syscall Interface │   │
│  │  Interpreter     │◄────────────►│                    │   │
│  └──────────────────┘              └────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
              ┌─────────────────────┐
              │   Host Environment   │
              │   Blockchain State   │
              └─────────────────────┘
```

**Source References:**
- `crates/hybrid-vm/src/evm.rs:46-62`
- `crates/hybrid-vm/src/frame.rs:44-56`
- `crates/hybrid-vm/src/execution/mod.rs:34-48`

## HybridEvm Orchestrator

The `HybridEvm` struct serves as the main orchestrator, wrapping the standard EVM implementation while providing dual-VM capabilities. It implements the `EvmTr` trait to integrate seamlessly with the reth EVM framework.

### Structure

```rust
HybridEvm<CTX: ContextTr, INSP: Inspector>
├── Wrapped EVM Components
│   ├── Interpreter
│   ├── EvmData
│   ├── EthInstructions
│   └── EthPrecompiles
└── Context Management
    ├── InterpreterAction
    └── InterpreterResult
```

### Key Methods

| Method | Purpose | Source |
|--------|---------|--------|
| `new()` | Creates new HybridEvm instance | `crates/hybrid-vm/src/evm.rs:52-61` |
| `run_interpreter()` | Main execution entry point | `crates/hybrid-vm/src/evm.rs:84-368` |
| `with_inspector()` | Inspector management | `crates/hybrid-vm/src/evm.rs:412-421` |

**Source References:**
- `crates/hybrid-vm/src/evm.rs:46-62`
- `crates/hybrid-vm/src/evm.rs:64-83`

## Execution Flow

The execution flow begins with bytecode analysis to determine the appropriate execution path. The system uses a `0xFF` prefix to identify RISC-V bytecode.

```
Contract Call
     │
     ▼
Bytecode Analysis
     │
     ├─── First byte == 0xFF? ───► RISC-V Execution Path
     │                                  │
     │                                  ├─► setup_from_elf()
     │                                  ├─► rvemu.estart()
     │                                  ├─► Handle Syscalls
     │                                  └─► Return/Revert
     │
     └─── Standard bytecode ────► EVM Execution Path
                                      │
                                      ├─► serialize_input()
                                      ├─► setup_from_mini_elf()
                                      ├─► Mini-EVM Execution
                                      ├─► Host Syscall Bridge
                                      └─► Return/Revert
```

**Source References:**
- `crates/hybrid-vm/src/frame.rs:44-56`
- `crates/hybrid-vm/src/hybrid_execute.rs:32-75`
- `crates/hybrid-vm/src/evm.rs:123-148`

## Bytecode Detection and Routing

The system uses a simple but effective bytecode detection mechanism in the frame handler:

```rust
// Pseudocode representation
match contract_bytecode.split_first() {
    Some((0xFF, rest)) => {
        // RISC-V execution path
        run_hybrid_interpreter()
        setup_from_elf(rest)
    }
    _ => {
        // Standard EVM execution
        Frame::run()
    }
}
```

The detection logic extracts the first byte and routes accordingly. For RISC-V contracts, the remaining bytecode after the `0xFF` prefix is passed to the ELF loader.

**Source Reference:** `crates/hybrid-vm/src/frame.rs:44-56`

## RISC-V Execution Path

When RISC-V bytecode is detected, the system initializes a RISC-V emulator and begins execution with syscall handling:

```
setup_from_elf()
     │
     ▼
rvemu::Emulator
     │
     ▼
emulator.estart()
     │
     ├─── Ok(_) ────────────────► Continue Execution
     │
     ├─── Exception::           ┌─────────────────────┐
     │    EnvironmentCall   ────►│  Syscall Dispatch   │
     │    FromMMode              │                     │
     │                           │ • context.balance() │
     │                           │ • sload/sstore()    │
     │                           │ • block_number()    │
     │                           │ • execute_create()  │
     │                           │ • execute_call()    │
     │                           └─────────────────────┘
     │                                    │
     │                                    ├─► Return/Revert syscalls
     │                                    └─► Syscall enum (0x20-0xFF)
     │
     └─── Err(other) ───────────► Execution Error
```

The execution environment provides a comprehensive syscall interface mapping EVM opcodes to RISC-V system calls.

**Source References:**
- `crates/hybrid-vm/src/execution/mod.rs:65-501`
- `crates/hybrid-syscalls/src/lib.rs:77-118`

## Mini-EVM Integration

For EVM bytecode execution within RISC-V contracts, the system embeds a mini-EVM interpreter that runs inside the RISC-V emulator:

```
Mini-EVM Interpreter
     │
     ├─► serialize_input()
     │
     ├─► setup_from_mini_elf()
     │
     ├─► Embedded Mini-EVM Binary
     │        │
     │        ├─► RISC-V Execution
     │        │
     │        └─► Host Syscalls (10-20)
     │                 │
     │                 ├─► HOST_BALANCE (10)
     │                 ├─► HOST_LOAD_ACCOUNT_CODE (11)
     │                 ├─► HOST_LOAD_ACCOUNT_CODE_HASH (12)
     │                 ├─► HOST_BLOCK_NUMBER (13)
     │                 ├─► HOST_BLOCK_HASH (14)
     │                 ├─► HOST_SLOAD (15)
     │                 ├─► HOST_SSTORE (16)
     │                 ├─► HOST_TLOAD (17)
     │                 └─► HOST_TSTORE (18)
     │
     └─► deserialize_output()
```

The mini-EVM uses a dedicated memory region for syscall communication and implements a comprehensive instruction table for EVM compatibility.

**Source References:**
- `crates/hybrid-vm/src/evm.rs:123-148`
- `crates/hybrid-vm/src/evm.rs:32-44`
- `bins/mini-evm-interpreter/src/instruction_table.rs:142-353`

## Syscall Interface

The syscall interface provides the bridge between RISC-V execution and the host EVM environment. It uses a structured approach with defined syscall IDs and parameter passing conventions.

### RISC-V Syscall Interface

```
RISC-V Contract
     │
     ├─► ecall instruction
     │
     ├─► t0 register (syscall ID)
     │
     ├─► Parameter Registers:
     │   • a0 (x10)
     │   • a1 (x11)
     │   • a2 (x12)
     │   • a3 (x13)
     │   • a4 (x14)
     │   • a5 (x15)
     │
     ▼
Syscall Dispatch ──► Host Function Call ──► Return via Registers
```

### Mini-EVM Syscall Interface

```
Mini-EVM
     │
     ├─► Syscall ID (10-20)
     │
     ├─► MINI_EVM_SYSCALLS_MEM_ADDR
     │
     ├─► bincode serialization
     │
     ▼
Host Operation
```

The syscall system supports both direct parameter passing (for RISC-V) and memory-based communication (for mini-EVM), with automatic serialization and deserialization.

**Source References:**
- `crates/hybrid-vm/src/execution/mod.rs:69-489`
- `crates/hybrid-vm/src/evm.rs:154-357`

## Memory Management

The system uses distinct memory regions for different execution contexts:

| Memory Region | Base Address | Purpose | Source |
|--------------|--------------|---------|--------|
| DRAM | `0x8000_0000` | RISC-V program memory | `crates/rvemu/src/bus.rs:45` |
| Mini-EVM Syscalls | `0xBEC00000` | EVM-Host communication | `crates/hybrid-vm/src/evm.rs:30` |
| Return Data | `0x8000_0000` | Interpreter output | `crates/hybrid-vm/src/evm.rs:143` |

The memory layout ensures isolation between execution contexts while providing efficient communication channels through designated regions.

**Source References:**
- `crates/hybrid-vm/src/evm.rs:28-30`
- `crates/rvemu/src/bus.rs:44-47`
- `crates/hybrid-vm/src/execution/helper.rs:53-76`

## Key Features

1. **Transparent Bytecode Routing**: Automatic detection and routing based on bytecode prefix
2. **Dual Execution Environments**: Support for both RISC-V and EVM execution
3. **Comprehensive Syscall Bridge**: Full mapping between RISC-V syscalls and EVM operations
4. **Embedded Mini-EVM**: EVM interpreter running within RISC-V context
5. **Efficient Memory Management**: Isolated memory regions with structured communication
6. **Host Integration**: Seamless integration with the reth EVM framework

## Usage Example

```rust
// Create a new HybridEvm instance
let hybrid_evm = HybridEvm::new(context, inspector);

// Execute contract (automatically routed based on bytecode)
let result = hybrid_evm.run_interpreter(
    contract_address,
    bytecode,
    input_data,
    is_static
);

// Handle result
match result {
    InterpreterResult::Return { output, .. } => {
        // Handle successful execution
    }
    InterpreterResult::Revert { output, .. } => {
        // Handle revert
    }
    _ => {
        // Handle other cases
    }
}
```

## Performance Considerations

- **Bytecode Detection**: O(1) operation using simple prefix check
- **Memory Isolation**: Minimal overhead through well-defined memory regions
- **Syscall Overhead**: Optimized parameter passing for both execution paths
- **Context Switching**: Efficient switching between RISC-V and EVM contexts

## Future Enhancements

- Extended syscall interface for additional EVM opcodes
- Performance optimizations for frequent syscall patterns
- Enhanced debugging and tracing capabilities
- Support for additional execution environments