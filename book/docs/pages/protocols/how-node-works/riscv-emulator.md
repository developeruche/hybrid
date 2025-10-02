---
description: Exploring the RISCV emulator
---

# RISC-V Emulation Subsystem

The RISC-V emulation subsystem provides a complete RISC-V processor emulator that executes RISC-V bytecode compiled from Rust smart contracts within the Hybrid Framework. This emulator serves as one half of the dual-VM architecture, enabling Rust contracts to run alongside traditional EVM bytecode.


## Architecture Overview

The RISC-V emulation is implemented in the `rvemu` crate and integrated into the hybrid VM through the `hybrid-vm` crate's setup module. The emulator provides a complete RISC-V RV64 processor implementation with support for multiple instruction sets and virtual devices.

### System Components

```
┌─────────────────────────────────────────────────────┐
│              Execution Context                       │
├─────────────────────────────────────────────────────┤
│  RISC-V Emulator (rvemu)                            │
│  ├── CPU                                             │
│  │   ├── XRegisters (32 integer registers)          │
│  │   ├── FRegisters (32 float registers)            │
│  │   ├── Program Counter                            │
│  │   └── State (CSRs)                               │
│  ├── Bus                                             │
│  └── DRAM                                            │
├─────────────────────────────────────────────────────┤
│  Device Emulation                                    │
│  ├── Virtio Block Device                            │
│  ├── UART                                            │
│  ├── PLIC (Platform-Level Interrupt Controller)     │
│  └── CLINT (Core-Local Interrupt Controller)        │
└─────────────────────────────────────────────────────┘
```

**Source References:**
- `crates/hybrid-vm/src/setup/mod.rs` (lines 1-74)
- `crates/rvemu/src/cpu.rs` (lines 212-262)


## ELF Loading and Memory Setup

The emulator initialization process involves loading ELF (Executable and Linkable Format) binaries compiled from Rust smart contracts into the RISC-V emulator's memory space.

### Setup Functions

The setup module provides two primary functions for different memory configurations:

- **`setup_from_elf`** - Allocates 1MB of memory for contract execution
- **`setup_from_mini_elf`** - Allocates 5MB of memory for larger contracts

Both functions follow the same initialization pattern:

```
ELF Binary Data ──┐
                  ├──> goblin::elf::Elf::parse
Contract Call Data┘
                      │
                      ├──> Allocate Memory
                      │
                      ├──> Copy Call Data to Memory
                      │
                      ├──> load_sections
                      │
                      └──> Emulator::new
                              ├── initialize_dram
                              └── initialize_pc
```

The call data is stored at the beginning of memory with an 8-byte length prefix, followed by the actual data. The ELF sections are then loaded into their appropriate memory locations based on the program headers.

**Source References:**
- `crates/hybrid-vm/src/setup/mod.rs` (lines 5-50, 52-73)

### Memory Layout

The `load_sections` function processes ELF program headers of type `PT_LOAD` and maps them into the emulator's memory space. Virtual addresses must fall within the DRAM boundaries (`DRAM_BASE` to `DRAM_BASE + DRAM_SIZE`).

| Memory Region | Purpose | Size |
|--------------|---------|------|
| Call Data Length | 8-byte call data size | 8 bytes |
| Call Data | Contract input parameters | Variable |
| Program Sections | ELF loadable segments | Variable |
| Stack Space | Runtime stack | Remaining DRAM |


## CPU Emulation

The core RISC-V processor emulation is implemented in the `Cpu` struct, which provides a complete RV64IMAFDC processor implementation with support for machine, supervisor, and user privilege modes.

### CPU Components

The CPU comprises four main subsystems:

1. **Register Files**
   - XRegisters: 32 64-bit integer registers
   - FRegisters: 32 64-bit floating-point registers
   - Program Counter
   - State: Control Status Registers (CSRs)

2. **Execution Engine**
   - Instruction fetch
   - Decode and execute
   - Support for both standard (32-bit) and compressed (16-bit) instructions

3. **Memory Management**
   - Virtual memory translation (SV39 paging)
   - Memory read/write operations
   - DRAM boundary checking

4. **Interrupt Handling**
   - Pending interrupt checks
   - Device increment operations
   - Exception/interrupt processing

**Source References:**
- `crates/rvemu/src/cpu.rs` (lines 212-262, 666-732)

### Register Implementation

#### Integer Registers (XRegisters)

The emulator implements 32 64-bit integer registers following RISC-V ABI naming conventions:

- Register `x0` is hardwired to zero (reads always return 0, writes are ignored)
- Registers `x1-x31` support full read/write operations
- Stack pointer initialized to `DRAM_BASE + DRAM_SIZE`
- Argument registers set for bootloader compatibility

#### Floating-Point Registers (FRegisters)

32 64-bit floating-point registers supporting IEEE 754 operations for single and double-precision arithmetic.

**Source References:**
- `crates/rvemu/src/cpu.rs` (lines 68-108, 143-166)

### Instruction Execution

The CPU supports both standard 32-bit RISC-V instructions and compressed 16-bit instructions through a fetch-decode-execute cycle:

```
┌──────────────┐
│ Fetch        │
│ HALFWORD     │
└──────┬───────┘
       │
       ├─── Check inst & 0b11
       │
       ├─── [0,1,2] ──> execute_compressed ──┐
       │                                      │
       └─── [3] ──> Fetch WORD                │
                         │                    │
                         ├─> execute_general ─┤
                         │                    │
                         └────────────────────┴──> Update PC
```

The execution engine uses separate handlers for different instruction formats, ensuring proper support for the RISC-V instruction set architecture.

**Source References:**
- `crates/rvemu/src/cpu.rs` (lines 666-697, 734-1175)


## Device Emulation

The RISC-V emulator includes several virtual devices to provide I/O capabilities and system services for running contracts.

### Virtio Block Device

The Virtio block device implementation provides disk I/O functionality following the VirtIO specification. This device enables contracts to perform persistent storage operations.

#### Register Layout

- **MAGIC**: `0x74726976` - VirtIO magic value
- **DEVICE_ID**: Block device identifier
- **STATUS**: Device status register
- **CONFIG**: Configuration space

#### Queue Operations

The device supports standard block operations through virtqueues:

- **Descriptor Table**: Scatter-gather buffer descriptors
- **Available Ring**: Guest-to-device queue
- **Used Ring**: Device-to-guest completion queue

The device supports DMA (Direct Memory Access) for efficient data transfer between contract memory and virtual disk storage through the `disk_access` operation.

**Source References:**
- `crates/rvemu/src/devices/virtio_blk.rs` (lines 269-287, 525-621)

### Control and Status Registers (CSRs)

The CSR implementation provides the full RISC-V privileged architecture register set, enabling proper privilege mode transitions and system control.

| CSR Category | Examples | Purpose |
|--------------|----------|---------|
| Machine-level | MSTATUS, MTVEC, MEPC | Machine mode control |
| Supervisor-level | SSTATUS, STVEC, SEPC | Supervisor mode control |
| User-level | USTATUS, UTVEC, FCSR | User mode and floating-point |

The `State` struct manages all CSR operations with proper masking for supervisor-level register views.

**Source References:**
- `crates/rvemu/src/csr.rs` (lines 172-231, 238-283)


## Integration with Hybrid VM

The RISC-V emulator integrates with the hybrid VM system through the setup functions and provides the execution environment for Rust smart contracts.

### Contract Execution Flow

```
┌────────────┐                    ┌──────────┐
│ Hybrid VM  │                    │ Emulator │
└─────┬──────┘                    └────┬─────┘
      │                                │
      ├─ setup_from_elf ──────────────>│
      │  (Load contract ELF + call data)
      │                                │
      │<──── Initialize emulator ──────┤
      │      (with memory)             │
      │                                │
      ├─ Start execution ─────────────>│
      │                                │
      │         [Contract Execution Loop]
      │                                │
      │                    Fetch and execute
      │                    instructions ──┐
      │                                   │
      │<─── System call ──────────────────┤
      │     (for host interaction)        │
      │                                   │
      ├─ Handle host operations          │
      │                                   │
      ├─ Return result ──────────────────>│
      │                                   │
      │                    Resume execution
      │                                   │
      │<─── Contract completion ──────────┤
      │                                   │
```

The emulator executes until the contract completes or encounters a system call that requires host environment interaction. System calls are handled through the syscall interface, which bridges between the RISC-V execution context and the EVM host environment.

**Source References:**
- `crates/hybrid-vm/src/setup/mod.rs` (lines 5-26)
- `crates/rvemu/src/cpu.rs` (lines 318-402)

---

## Memory Management

The emulator provides virtual memory support through SV39 paging when running in supervisor mode. This enables proper memory isolation and supports the Linux-like execution environment expected by compiled Rust contracts.

### SV39 Paging

The translation process follows the RISC-V specification for three-level page tables:

- **Level 2**: VPN[2] - Virtual Page Number bits [38:30]
- **Level 1**: VPN[1] - Virtual Page Number bits [29:21]
- **Level 0**: VPN[0] - Virtual Page Number bits [20:12]
- **Page Offset**: Physical address bits [11:0]

The system includes proper exception handling for:
- Page faults
- Access violations
- Invalid page table entries

**Source References:**
- `crates/rvemu/src/cpu.rs` (lines 404-579, 581-639)


## Implementation Details

### Key Crates

- **`rvemu`**: Core RISC-V emulator implementation
- **`hybrid-vm`**: Integration layer connecting RISC-V emulation with the hybrid VM framework

### Supported Instruction Sets

- **RV64I**: Base integer instruction set (64-bit)
- **M**: Integer multiplication and division
- **A**: Atomic instructions
- **F**: Single-precision floating-point
- **D**: Double-precision floating-point
- **C**: Compressed instructions (16-bit)

### Privilege Modes

The emulator supports all three RISC-V privilege levels:

1. **Machine Mode (M)**: Highest privilege level
2. **Supervisor Mode (S)**: Operating system kernel level
3. **User Mode (U)**: Application level


## Usage Example

```rust
use hybrid_vm::setup::setup_from_elf;

// Load and initialize RISC-V emulator with contract
let elf_binary = include_bytes!("contract.elf");
let call_data = vec![/* contract input parameters */];

let mut emulator = setup_from_elf(elf_binary, &call_data)?;

// Execute contract
loop {
    match emulator.cpu.execute() {
        Ok(continuation) => {
            // Handle execution state
        }
        Err(exception) => {
            // Handle exceptions or syscalls
            break;
        }
    }
}
```


## Performance Considerations

- Use `setup_from_elf` (1MB) for smaller contracts to minimize memory overhead
- Use `setup_from_mini_elf` (5MB) for larger contracts requiring more memory
- The emulator supports efficient execution through compressed instruction support
- Virtio devices use DMA for optimized I/O operations