---
description: Explain the syscall interface that makes this all possible
---

# RISC-V Syscall Interface Documentation

## Overview

The syscall interface system enables RISC-V smart contracts to interact with the EVM host environment in the Hybrid Framework. This interface provides a bridge between RISC-V execution contexts and EVM blockchain state operations, including storage access, account queries, and block information retrieval.

**Related Documentation:**
- RISC-V Emulation
- Mini EVM Interpreter


## Architecture Overview

The syscall interface operates as a bidirectional communication layer between RISC-V contracts running in the `rvemu` emulator and the EVM host environment managed by `hybrid-vm`. This system allows RISC-V smart contracts to access blockchain state and invoke EVM operations through a standardized syscall mechanism.

### System Call Flow

```
┌─────────────────────────┐
│  RISC-V Contract        │
│  Rust Smart Contract    │
└───────────┬─────────────┘
            │
            │ ecall instruction
            │ t0 = syscall_id
            │ a0-a6 = arguments
            ▼
┌─────────────────────────┐
│  Syscall Processing     │
│  (execution/mod.rs)     │
└───────────┬─────────────┘
            │
     ┌──────┴──────┐
     │             │
     ▼             ▼
┌─────────┐   ┌─────────────┐
│  HOST_* │   │ EVM Opcode  │
│  ops    │   │  Wrappers   │
└────┬────┘   └──────┬──────┘
     │               │
     └───────┬───────┘
             ▼
    ┌─────────────────┐
    │  Shared Memory  │
    │  0xBEC00000     │
    └────────┬────────┘
             │
             ▼
    ┌─────────────────┐
    │  EVM Host       │
    │  Environment    │
    └────────┬────────┘
             │
             ▼
    ┌─────────────────┐
    │ Blockchain State│
    └─────────────────┘
```

**Key Components:**
- **RISC-V Contract Execution**: Smart contracts written in Rust compiled to RISC-V
- **Syscall Processing Layer**: Routes syscalls to appropriate handlers
- **Host Interface Layer**: Manages communication with EVM host
- **Shared Memory**: `MINI_EVM_SYSCALLS_MEM_ADDR` at `0xBEC00000`
- **EVM Host Environment**: Provides access to blockchain state


## Syscall Categories

The syscall interface supports several categories of operations:

| Category | Syscall IDs | Purpose | Implementation |
|----------|-------------|---------|----------------|
| Host State Queries | 10-14, 19 | Account and block information | `ext_syscalls.rs` |
| Storage Operations | 15-16 | Persistent storage access | Direct host calls |
| Transient Storage | 17-18 | EIP-1153 temporary storage | Register-based I/O |
| Contract Operations | 20 | Self-destruct functionality | Serialized host calls |
| EVM Standard Opcodes | 0x20-0xFF | Standard EVM operations | `execution/mod.rs` |

### Host Operations (IDs 10-20)

| ID | Syscall | Function | Description |
|----|---------|----------|-------------|
| 10 | `HOST_BALANCE` | `host_balance()` | Query account balance |
| 11 | `HOST_LOAD_ACCOUNT_CODE` | `host_load_account_code()` | Load contract bytecode |
| 12 | `HOST_LOAD_ACCOUNT_CODE_HASH` | `host_load_account_code_hash()` | Get code hash |
| 13 | `HOST_BLOCK_NUMBER` | `host_block_number()` | Current block number |
| 14 | `HOST_BLOCK_HASH` | `host_block_hash()` | Block hash by number |
| 15 | `HOST_SLOAD` | `host_sload()` | Load from storage |
| 16 | `HOST_SSTORE` | `host_sstore()` | Store to storage |
| 17 | `HOST_TLOAD` | `host_tload()` | Load from transient storage |
| 18 | `HOST_TSTORE` | `host_tstore()` | Store to transient storage |
| 19 | `HOST_LOAD_ACCOUNT_DELEGATED` | `host_load_account_delegated()` | Query delegated account |
| 20 | `HOST_SELFDESTRUCT` | `host_selfdestruct()` | Self-destruct contract |

### EVM Opcodes (IDs 0x20-0xFF)

| Opcode | ID | Syscall | Description |
|--------|-----|---------|-------------|
| `KECCAK256` | 0x20 | `Syscall::Keccak256` | Compute Keccak-256 hash |
| `BALANCE` | 0x31 | `Syscall::Balance` | Get account balance |
| `SLOAD` | 0x54 | `Syscall::SLoad` | Load storage slot |
| `SSTORE` | 0x55 | `Syscall::SStore` | Store to storage slot |
| `CREATE` | 0xF0 | `Syscall::Create` | Create new contract |
| `CALL` | 0xF1 | `Syscall::Call` | Call another contract |
| `RETURN` | 0xF3 | `Syscall::Return` | Return from execution |
| `REVERT` | 0xFD | `Syscall::Revert` | Revert state changes |


## Memory Management

### Memory Layout

The syscall interface uses a dedicated memory region for data exchange:

- **Base Address**: `MINI_EVM_SYSCALLS_MEM_ADDR = 0xBEC00000`
- **Location**: Last 20MB of address space
- **Serialization Format**: `bincode` with legacy configuration
- **Data Flow**: Input parameters → shared memory → host processing → output results

### Memory Usage Pattern

```
┌──────────────┐
│  EVM Host    │
└──────┬───────┘
       │ 1. Write serialized input
       ▼
┌──────────────────────┐
│  Shared Memory       │
│  0xBEC00000          │
└──────┬───────────────┘
       │ 2. ecall with input_size
       ▼
┌──────────────────────┐
│  RISC-V Contract     │
│  Read input data     │
│  Process syscall     │
└──────┬───────────────┘
       │ 3. Write serialized output
       ▼
┌──────────────────────┐
│  Shared Memory       │
│  0xBEC00000          │
└──────┬───────────────┘
       │ 4. Return output_size
       ▼
┌──────────────────────┐
│  EVM Host            │
│  Read output data    │
└──────────────────────┘
```


## Implementation Details

### Host Syscall Functions

The `ext_syscalls.rs` module implements host-specific syscalls using shared memory communication.

#### Example: SSTORE Implementation

```rust
pub fn host_sstore(address: Address, index: U256, value: U256) 
    -> Option<StateLoad<SStoreResult>> {
    // Serialize input parameters
    let input_serialized = serialize_sstore_input(address, index, value);
    let input_size = input_serialized.len();
    
    // Write to shared memory
    unsafe {
        let dest = slice_from_raw_parts_mut(
            MINI_EVM_SYSCALLS_MEM_ADDR, 
            input_serialized.len()
        );
        dest.copy_from_slice(&input_serialized);
    }
    
    // Make syscall via ecall instruction
    let mut output_size;
    unsafe {
        asm!(
            "ecall",
            in("a0") input_size,
            lateout("a0") output_size,
            in("t0") mini_evm_syscalls_ids::HOST_SSTORE
        );
    }
    
    // Read result from shared memory
    let out_serialized = unsafe { 
        slice_from_raw_parts(MINI_EVM_SYSCALLS_MEM_ADDR, output_size) 
    };
    
    bincode::serde::decode_from_slice(
        out_serialized, 
        bincode::config::legacy()
    ).unwrap().0
}
```

### EVM Opcode Wrappers

The `ext_opcode.rs` module provides EVM opcode implementations.

#### Example: BALANCE Opcode

```rust
pub fn balance<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    _host: &mut H,
) {
    popn_top!([], top, interpreter);
    let address = top.into_address();
    
    // Call host_balance syscall
    let Some(balance) = host_balance(address) else {
        interpreter.control.set_instruction_result(
            InstructionResult::FatalExternalError
        );
        return;
    };
    
    // Calculate gas cost based on EVM specification
    gas!(interpreter, 
        if spec_id.is_enabled_in(BERLIN) {
            warm_cold_cost(balance.is_cold)
        } else if spec_id.is_enabled_in(ISTANBUL) {
            700
        } else if spec_id.is_enabled_in(TANGERINE) {
            400
        } else {
            20
        }
    );
    
    *top = balance.data;
}
```

### RISC-V Execution Loop

The main execution loop in `hybrid-vm` processes syscalls:

```rust
match run_result {
    Err(Exception::EnvironmentCallFromMMode) => {
        // Read syscall ID from t0 register
        let t0: u64 = emu.cpu.xregs.read(5);
        
        let Ok(syscall) = Syscall::try_from(t0 as u8) else {
            return return_revert(
                interpreter, 
                interpreter.control.gas.spent()
            );
        };
        
        match syscall {
            Syscall::Balance => {
                // Read address from registers a0-a2
                let address_1 = emu.cpu.xregs.read(10);
                let address_2 = emu.cpu.xregs.read(11);
                let address_3 = emu.cpu.xregs.read(12);
                
                let address = __3u64_to_address(
                    address_1, address_2, address_3
                );
                
                match host.balance(address) {
                    Some(state_load) => {
                        // Write result to registers a0-a3
                        let limbs = state_load.data.as_limbs();
                        emu.cpu.xregs.write(10, limbs[0]);
                        emu.cpu.xregs.write(11, limbs[1]);
                        emu.cpu.xregs.write(12, limbs[2]);
                        emu.cpu.xregs.write(13, limbs[3]);
                    }
                    _ => return return_revert(
                        interpreter, 
                        interpreter.control.gas.spent()
                    ),
                }
            }
            // ... other syscalls
        }
    }
}
```


## Register Conventions

The syscall interface follows RISC-V calling conventions:

| Register | RISC-V Name | Purpose | Usage |
|----------|-------------|---------|-------|
| `t0` | `x5` | Syscall ID | Contains the syscall identifier |
| `a0` | `x10` | Argument 0 / Return 0 | First argument or return value |
| `a1` | `x11` | Argument 1 / Return 1 | Second argument or return value |
| `a2` | `x12` | Argument 2 / Return 2 | Third argument or return value |
| `a3` | `x13` | Argument 3 / Return 3 | Fourth argument or return value |
| `a4` | `x14` | Argument 4 | Fifth argument |
| `a5` | `x15` | Argument 5 | Sixth argument |
| `a6` | `x16` | Argument 6 | Seventh argument |

### Complex Data Type Encoding

**U256 Values:**
- Split across 4 registers (`a0-a3`)
- Each register contains a 64-bit limb
- Little-endian ordering

**Addresses:**
- Split across 3 registers (`a0-a2`)
- Zero-padded to fit 160-bit address
- Remaining bits unused

**Variable-Length Data:**
- Size passed in `a0`
- Data stored in shared memory at `0xBEC00000`
- Read/write using memory address


## Error Handling

The syscall interface implements comprehensive error handling:

### Error Types

| Error Type | Cause | Result |
|------------|-------|--------|
| **Invalid Syscall ID** | Unknown syscall identifier | Contract reversion |
| **Memory Access Error** | Invalid memory operation | `FatalExternalError` |
| **Host Operation Failure** | Failed state operation | Contract revert |
| **Serialization Error** | Invalid data format | Panic or revert |

### Error Flow

```
Syscall Invoked
    ↓
Validate Syscall ID ──→ Invalid ──→ Revert Contract
    ↓ Valid
Execute Operation ──→ Failure ──→ FatalExternalError/Revert
    ↓ Success
Return Result
```

### State Consistency

The system ensures that any failure in the syscall interface results in a clean revert of the RISC-V contract execution, maintaining blockchain state consistency. No partial state changes are committed when a syscall fails.


## Usage Examples

### Calling a Syscall from RISC-V Contract

```rust
// Example: Query account balance
use hybrid_syscalls::*;

fn get_balance(address: Address) -> U256 {
    // This will trigger a syscall via the ecall instruction
    let balance = host_balance(address).expect("Balance query failed");
    balance.data
}
```

### Storage Operations

```rust
// Store a value
let key = U256::from(1);
let value = U256::from(42);
host_sstore(contract_address, key, value);

// Load a value
let stored_value = host_sload(contract_address, key)
    .expect("Storage load failed");
```

### Transient Storage (EIP-1153)

```rust
// Store temporary data (cleared at transaction end)
host_tstore(key, value);

// Load temporary data
let temp_value = host_tload(key);
```


## Source Code References

- **Syscall Definitions**: `crates/hybrid-syscalls/src/lib.rs`
- **Host Syscall Implementation**: `bins/mini-evm-interpreter/src/ext_syscalls.rs`
- **EVM Opcode Wrappers**: `bins/mini-evm-interpreter/src/ext_opcode.rs`
- **Execution Loop**: `crates/hybrid-vm/src/execution/mod.rs`