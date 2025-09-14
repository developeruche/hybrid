# Mini EVM Interpreter

A lightweight, hybrid EVM (Ethereum Virtual Machine) interpreter designed to run in a no-std environment with RISC-V architecture support. This interpreter provides a minimal but functional implementation of EVM bytecode execution optimized for constrained environments.

## Overview

The Mini EVM Interpreter is built as a hybrid contract that can execute Ethereum bytecode in resource-constrained environments. It supports a comprehensive subset of EVM opcodes through a custom instruction table and operates with serialized input/output for state management.

## Features

- **No-std Environment**: Designed to run without standard library dependencies
- **RISC-V Architecture**: Optimized for RISC-V processors
- **Comprehensive Opcode Support**: Implements most EVM opcodes including:
  - Arithmetic operations (ADD, SUB, MUL, DIV, etc.)
  - Bitwise operations (AND, OR, XOR, shifts, etc.)
  - Stack operations (PUSH, POP, DUP, SWAP)
  - Memory operations (MLOAD, MSTORE, MCOPY)
  - Storage operations (SLOAD, SSTORE, TLOAD, TSTORE)
  - Control flow (JUMP, JUMPI, CALL, RETURN)
  - System operations (CREATE, SELFDESTRUCT, LOG)
  - EOF (Ethereum Object Format) support
- **Hybrid Contract Integration**: Seamless integration with hybrid contract environments
- **Memory-Efficient**: Optimized for minimal memory footprint
- **Serializable State**: Input/output via binary serialization

## Architecture

### Core Components

1. **Main Library (`lib.rs`)**
   - Entry point for EVM execution
   - Context setup and management
   - Input/output coordination

2. **Instruction Table (`instruction_table.rs`)**
   - Maps EVM opcodes to implementation functions
   - Supports 200+ EVM instructions
   - Organized by functional categories

3. **Utilities (`utils.rs`)**
   - Memory management functions
   - Serialization/deserialization
   - Debug utilities
   - I/O operations

### Memory Model

The interpreter operates with a specific memory layout:

```
+------------------+
| Input Data       |  <- CALLDATA_ADDRESS
| - Length Header  |     (8 bytes: data length)
| - Serialized:    |
|   * Interpreter  |
|   * BlockEnv     |
|   * TxEnv        |
+------------------+
| ...              |
+------------------+
| Debug Region     |  <- CALLDATA_ADDRESS + 1GB - 2000
| - Debug Output   |     (for debugging purposes)
+------------------+
```

### Data Format

#### Input Format
```
[interpreter_len: u64][block_len: u64][tx_len: u64][interpreter_data][block_data][tx_data]
```

#### Output Format
```
[interpreter_len: u64][block_len: u64][tx_len: u64][action_len: u64]
[interpreter_data][block_data][tx_data][action_data]
```

## Build Instructions

The project requires Rust nightly with specific build configurations for the RISC-V target:

```bash
cargo +nightly-2025-01-07 build -r --lib -Z build-std=core,alloc --target riscv64imac-unknown-none-elf --bin runtime
```

### Build Requirements

- **Rust Toolchain**: `nightly-2025-01-07`
- **Target**: `riscv64imac-unknown-none-elf`
- **Features**: 
  - `core` and `alloc` standard libraries
  - Custom build-std configuration
  - Release optimization (`-r`)

### Build Targets

The project provides two build targets:

- **`runtime`**: Standard execution binary
- **`deploy`**: Deployment binary (requires `deploy` feature)

## Usage

### Basic Execution Flow

1. **Input Preparation**: Serialize interpreter state, block environment, and transaction environment
2. **Execution**: Load the binary and provide serialized input via memory
3. **Output Retrieval**: Read serialized execution results from memory

### Integration Example

```rust
// This is conceptual - actual integration depends on your hybrid contract environment
let input_data = serialize_execution_context(&interpreter, &block_env, &tx_env);
let output_data = execute_mini_evm(input_data);
let (updated_interpreter, block_env, tx_env, action) = deserialize_output(&output_data);
```

## Supported EVM Instructions

### Arithmetic Operations (0x01-0x0B)
- `ADD`, `MUL`, `SUB`, `DIV`, `SDIV`
- `MOD`, `SMOD`, `ADDMOD`, `MULMOD`
- `EXP`, `SIGNEXTEND`

### Comparison & Bitwise (0x10-0x1D)
- `LT`, `GT`, `SLT`, `SGT`, `EQ`, `ISZERO`
- `AND`, `OR`, `XOR`, `NOT`, `BYTE`
- `SHL`, `SHR`, `SAR`

### Cryptographic (0x20)
- `KECCAK256`

### Environment Information (0x30-0x4A)
- Address: `ADDRESS`, `BALANCE`, `ORIGIN`, `CALLER`, `CALLVALUE`
- Calldata: `CALLDATALOAD`, `CALLDATASIZE`, `CALLDATACOPY`
- Code: `CODESIZE`, `CODECOPY`, `EXTCODESIZE`, `EXTCODECOPY`, `EXTCODEHASH`
- Block: `BLOCKHASH`, `COINBASE`, `TIMESTAMP`, `NUMBER`, `DIFFICULTY`, `GASLIMIT`
- Advanced: `CHAINID`, `SELFBALANCE`, `BASEFEE`, `BLOBHASH`, `BLOBBASEFEE`

### Stack & Memory (0x50-0x5E)
- Stack: `POP`, `PUSH0-PUSH32`, `DUP1-DUP16`, `SWAP1-SWAP16`
- Memory: `MLOAD`, `MSTORE`, `MSTORE8`, `MSIZE`, `MCOPY`
- Storage: `SLOAD`, `SSTORE`, `TLOAD`, `TSTORE`
- Control: `JUMP`, `JUMPI`, `PC`, `GAS`, `JUMPDEST`

### Logging (0xA0-0xA4)
- `LOG0`, `LOG1`, `LOG2`, `LOG3`, `LOG4`

### EOF Operations (0xD0-0xEF)
- Data: `DATALOAD`, `DATALOADN`, `DATASIZE`, `DATACOPY`
- Control: `RJUMP`, `RJUMPI`, `RJUMPV`, `CALLF`, `RETF`, `JUMPF`
- Stack: `DUPN`, `SWAPN`, `EXCHANGE`
- Contracts: `EOFCREATE`, `RETURNCONTRACT`

### System Operations (0xF0-0xFF)
- Creation: `CREATE`, `CREATE2`, `EOFCREATE`
- Calls: `CALL`, `CALLCODE`, `DELEGATECALL`, `STATICCALL`
- Advanced: `EXTCALL`, `EXTDELEGATECALL`, `EXTSTATICCALL`
- Termination: `RETURN`, `REVERT`, `INVALID`, `SELFDESTRUCT`

## Configuration

### Chain Configuration
The interpreter is configured for Ethereum mainnet (Chain ID: 1) by default. This can be modified in the `CHAIN_ID` constant in `lib.rs`.

### Memory Configuration
Memory addresses and layout are defined in the `utils.rs` module and can be adjusted based on your hybrid contract environment requirements.

## Dependencies

### Core Dependencies
- **ext-revm**: Extended REVM interpreter for EVM execution
- **hybrid-contract**: Hybrid contract framework
- **bincode**: Binary serialization
- **alloy-core**: Ethereum core types

### Development Dependencies
- **serde**: Serialization framework (with no-std support)
- **alloc**: Allocation library for no-std environments

## Safety Considerations

This interpreter performs several unsafe operations:

1. **Raw Memory Access**: Direct memory operations at specific addresses
2. **Inline Assembly**: RISC-V assembly for register manipulation
3. **No Bounds Checking**: Optimized memory operations without safety checks

### Usage Guidelines

- Ensure proper memory layout in the host environment
- Validate input data before execution
- Handle potential panics in deserialization
- Consider timeout mechanisms for infinite loops
- Implement proper error handling for production use

## Development

### Code Structure

```
src/
├── lib.rs              # Main entry point and execution logic
├── utils.rs            # Utilities for I/O and memory management
└── instruction_table.rs # EVM instruction implementations
```

### Adding New Instructions

1. Add the opcode mapping in `instruction_table.rs`
2. Import the required instruction implementation
3. Update documentation with the new instruction

### Debugging

The interpreter includes debug utilities:

```rust
unsafe {
    debug_println(); // Writes "Hello, world!" to debug region
    debug_println_dyn_data(b"Custom debug message");
}
```

## Testing

Currently, testing requires integration with a hybrid contract environment. Unit tests for individual components can be added with careful consideration of the no-std constraints.

## License

This project is part of the hybrid contract ecosystem. Please refer to the main project license for usage terms.

## Contributing

When contributing to this project:

1. Maintain no-std compatibility
2. Follow existing documentation patterns
3. Test on RISC-V target architecture
4. Update instruction documentation for new opcodes
5. Ensure memory safety in unsafe code blocks

## Performance Considerations

- **Optimized for Size**: Built with `opt-level = "z"` for minimal binary size
- **LTO Enabled**: Link-time optimization for better performance
- **No Standard Library**: Reduced runtime overhead
- **Direct Memory Access**: Minimal allocation overhead
- **Efficient Serialization**: Binary format for fast I/O

## Troubleshooting

### Build Issues
- Ensure you're using the exact nightly version: `nightly-2025-01-07`
- Verify RISC-V target is installed: `rustup target add riscv64imac-unknown-none-elf`
- Check that build-std is available for your toolchain

### Runtime Issues
- Verify input data format matches expected serialization
- Check memory layout compatibility with host environment
- Ensure sufficient memory allocation for execution
- Validate that required host functions are implemented

### Debug Output
Use the debug utilities to trace execution:
- `debug_println()` for simple debugging
- `debug_println_dyn_data()` for custom debug messages
- Check debug memory region at `CALLDATA_ADDRESS + 1GB - 2000`
