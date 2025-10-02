---
description: Exploring how smart contract compilation is handled
---

# Compilation Pipeline

The compilation pipeline transforms Rust smart contracts into RISC-V bytecode that can execute on EVM-compatible blockchains. This system handles contract discovery, validation, dependency resolution, and multi-stage compilation to produce deployment-ready binaries.

> **Related Documentation:** For information about the runtime execution of compiled contracts, see [Hybrid VM Core](). For details about procedural macros used during compilation, see [Procedural Macros]().

## Pipeline Overview

The compilation pipeline orchestrates the transformation from Rust source code to executable RISC-V bytecode through a series of well-defined stages:

1. Developer writes Rust contract with `#[contract]` attribute
2. Contract discovery via `obtain_contract_by_path()`
3. Validation through `ContractWithDeps::try_from()`
4. Contract identifier extraction via `find_contract_ident()`
5. Runtime compilation via `compile_runtime()`
6. Deploy compilation via `compile_deploy()`
7. Bytecode combination with 0xFF prefix
8. Output generation as `.bin` file in `out/` directory

The main entry point `run_contract_compilation()` coordinates these stages, providing progress tracking and error handling throughout the process.

## Contract Discovery and Validation

The pipeline begins by locating and validating Rust projects that contain Hybrid Framework contracts.

### Discovery Process

The `obtain_contract_by_path()` function serves as the main discovery mechanism, performing the following validation steps:

1. **Directory Validation** - Checks if the provided path exists and is a directory
2. **Cargo.toml Detection** - Verifies the presence of a valid Cargo manifest
3. **Contract Structure Analysis** - Delegates to `ContractWithDeps::try_from()` for detailed validation

### Validation Requirements

The `ContractWithDeps::try_from()` method performs comprehensive validation of contract structure:

| Validation Step | Required Elements | Error Type |
|----------------|-------------------|------------|
| Package Structure | Valid Cargo.toml with package name | `ContractError::NotToml` |
| Features | `default`, `deploy`, `interface-only` | `ContractError::MissingFeatures` |
| Binary Targets | `runtime` and `deploy` bins with correct paths | `ContractError::MissingBinaries` |
| Dependencies | `hybrid-derive` and `hybrid-contract` | `ContractError::MissingDependencies` |
| Contract Dependencies | Interface-only contract dependencies | `ContractError::WrongPath` |

The validation process ensures that all required elements are present and correctly configured before proceeding to compilation.

## Contract Identifier Extraction

After validating the project structure, the pipeline extracts the contract identifier from the Rust source code.

The `find_contract_ident()` function performs the following steps:

1. Reads the `src/lib.rs` file into a string
2. Parses the file using the `syn` crate
3. Iterates through all items in the parsed file
4. Identifies implementation blocks (`Item::Impl`)
5. Checks for the `#[contract]` attribute via `has_contract_attribute()`
6. Extracts the type name using `extract_ident()`

This process uses Rust's `syn` crate to parse the source code and locate implementation blocks decorated with the `#[contract]` attribute.

## Two-Stage Compilation

The pipeline performs two distinct compilation stages to generate both runtime and deployment bytecode.

### Compilation Architecture

Both stages target the `riscv64imac-unknown-none-elf` architecture with no-std support, but differ in their feature flags and purpose:

#### Runtime Compilation

The `compile_runtime()` method builds the contract code that executes during normal function calls:

```bash
cargo +nightly-2025-01-07 build -r --lib -Z build-std=core,alloc \
    --target riscv64imac-unknown-none-elf --bin runtime
```

This generates a RISC-V ELF binary containing the contract's runtime logic without deployment-specific functionality.

#### Deploy Compilation

The `compile_deploy()` method builds the contract deployment code with the `deploy` feature enabled:

```bash
cargo +nightly-2025-01-07 build -r --lib -Z build-std=core,alloc \
    --target riscv64imac-unknown-none-elf --bin deploy --features deploy
```

This generates bytecode that includes constructor logic and deployment-time initialization.

### Key Compilation Parameters

- **Toolchain**: `nightly-2025-01-07` required for unstable features
- **Target**: `riscv64imac-unknown-none-elf` (bare-metal RISC-V)
- **Build Mode**: Release (`-r`) for optimized bytecode
- **Build Std**: Rebuilds `core` and `alloc` from source for the target
- **Binary Type**: Both compile as `--bin` targets (not libraries)

## Binary Generation and Prefixing

The final stage combines the compiled bytecodes and adds the EVM compatibility prefix.

The `compile_r55()` method orchestrates the two-stage compilation and produces the final deployment bytecode with the **0xFF prefix** that signals to the EVM that this is RISC-V bytecode rather than native EVM bytecode.

### Bytecode Format

```
[0xFF] + [deploy_bytecode] + [runtime_bytecode]
```

The 0xFF byte serves as a magic number indicating RISC-V bytecode format.

### Output Structure

The compilation pipeline generates the following output structure:

```
contract-root/
├── out/
│   └── {package-name}.bin    # Deployment-ready bytecode with 0xFF prefix
└── target/
    └── riscv64imac-unknown-none-elf/
        └── release/
            ├── runtime       # Runtime ELF binary
            └── deploy        # Deploy ELF binary
```

## Error Handling and Progress Tracking

The compilation pipeline provides comprehensive error reporting and visual feedback through the `ProgressBar` interface.

### Error Categories

| Error Type | Description | Recovery Action |
|-----------|-------------|-----------------|
| `ContractError::IoError` | File system operations failed | Check file permissions and disk space |
| `ContractError::NotToml` | Invalid Cargo.toml format | Fix TOML syntax and package configuration |
| `ContractError::MissingDependencies` | Required deps not found | Add `hybrid-derive` and `hybrid-contract` |
| `ContractError::MissingBinaries` | Binary targets misconfigured | Fix `[[bin]]` sections in Cargo.toml |
| `ContractError::MissingFeatures` | Required features not defined | Add `default`, `deploy`, `interface-only` features |

The pipeline uses `tracing` for detailed logging and provides colored terminal output for user feedback.

## Integration with Hybrid VM

The compiled bytecode integrates with the broader Hybrid Framework through the VM execution layer:

1. `.bin` file with 0xFF prefix is generated
2. `HybridEvm` loads the bytecode
3. `rvemu` RISC-V emulator executes the code
4. `hybrid-syscalls` bridge provides access to EVM functionality
5. Integration with EVM host environment for blockchain state access

The compiled RISC-V bytecode is designed to be loaded and executed by the `rvemu` RISC-V emulator within the Hybrid VM, with syscalls providing the bridge to the EVM host environment for blockchain state access.

## Source References

- `crates/hybrid-compile/src/lib.rs` - Main compilation entry points
- `crates/hybrid-compile/src/primitives.rs` - Core compilation logic and data structures
- `crates/hybrid-compile/src/utils.rs` - Discovery and validation utilities