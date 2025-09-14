# Hybrid Compile API Documentation

This document provides comprehensive API documentation for the `hybrid-compile` crate, covering all public functions, types, and their usage patterns.

## Table of Contents

- [Public API](#public-api)
- [Core Types](#core-types)
- [Error Types](#error-types)
- [Utility Functions](#utility-functions)
- [Usage Patterns](#usage-patterns)
- [Examples](#examples)

## Public API

### Main Functions

#### `run_contract_compilation`

```rust
pub fn run_contract_compilation(
    contract_root: &Path,
    is_check: bool,
    pb: ProgressBar,
    out: String,
) -> Result<(), anyhow::Error>
```

The primary entry point for contract compilation. Orchestrates the entire compilation pipeline from contract discovery to binary generation.

**Parameters:**
- `contract_root: &Path` - Path to the contract's root directory (must contain `Cargo.toml`)
- `is_check: bool` - If `true`, only validates syntax without generating binaries
- `pb: ProgressBar` - Progress bar instance for visual feedback during compilation
- `out: String` - Name of the output directory (created relative to contract root)

**Returns:**
- `Ok(())` - Compilation completed successfully
- `Err(anyhow::Error)` - Compilation failed with detailed error information

**Behavior:**
- Creates output directory if it doesn't exist
- Discovers and validates contract structure
- Performs two-stage compilation (runtime + deploy)
- Generates deployment-ready RISC-V bytecode with `0xff` prefix

**Example:**
```rust
use hybrid_compile::run_contract_compilation;
use indicatif::ProgressBar;
use std::path::Path;

let contract_path = Path::new("./my-contract");
let progress_bar = ProgressBar::new(100);

run_contract_compilation(contract_path, false, progress_bar, "build".to_string())?;
```

## Core Types

### `Contract`

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Contract {
    pub path: PathBuf,
    pub name: ContractName,
}
```

Represents a compiled smart contract with its essential metadata.

**Fields:**
- `path: PathBuf` - Absolute path to the contract's root directory
- `name: ContractName` - Contract identification information

**Methods:**

#### `path_str(&self) -> Result<&str, anyhow::Error>`

Converts the contract's path to a string representation.

**Returns:**
- `Ok(&str)` - String slice of the path
- `Err(anyhow::Error)` - Path contains invalid UTF-8 characters

#### `compile_r55(&self) -> Result<Vec<u8>, anyhow::Error>`

Compiles the contract to RISC-V bytecode for deployment.

**Returns:**
- `Ok(Vec<u8>)` - Deployment-ready bytecode with `0xff` prefix
- `Err(anyhow::Error)` - Compilation failed

**Process:**
1. Compiles runtime binary (`cargo build --bin runtime`)
2. Compiles deploy binary (`cargo build --bin deploy --features deploy`)
3. Combines bytecodes with `0xff` prefix

### `ContractWithDeps`

```rust
#[derive(Debug, Clone)]
pub struct ContractWithDeps {
    pub path: PathBuf,
    pub name: ContractName,
    pub deps: Vec<Contract>,
}
```

Represents a contract along with its resolved dependencies.

**Fields:**
- `path: PathBuf` - Absolute path to contract root
- `name: ContractName` - Contract identification
- `deps: Vec<Contract>` - List of contract dependencies with "interface-only" feature

**Trait Implementations:**

#### `TryFrom<&PathBuf> for ContractWithDeps`

```rust
impl TryFrom<&PathBuf> for ContractWithDeps {
    type Error = ContractError;
    
    fn try_from(cargo_toml_path: &PathBuf) -> Result<Self, Self::Error>
}
```

Parses a `Cargo.toml` file and validates contract structure.

**Validation Steps:**
1. Parse TOML format and extract package name
2. Validate required features: `default`, `deploy`, `interface-only`
3. Validate binary targets: `runtime` and `deploy` with correct configuration
4. Validate required dependencies: `hybrid-derive` and `hybrid-contract`
5. Resolve contract dependencies with `interface-only` feature

**Errors:**
- `ContractError::NotToml` - Invalid TOML or missing package info
- `ContractError::MissingFeatures` - Required features not defined
- `ContractError::MissingBinaries` - Binary targets incorrectly configured
- `ContractError::MissingDependencies` - Required dependencies missing
- `ContractError::WrongPath` - Dependency path cannot be resolved

#### `From<ContractWithDeps> for Contract`

Converts `ContractWithDeps` to `Contract`, discarding dependency information.

### `ContractName`

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct ContractName {
    pub package: String,
    pub ident: String,
}
```

Contract identification information extracted from metadata and source code.

**Fields:**
- `package: String` - Cargo package name from `Cargo.toml`
- `ident: String` - Contract identifier from `#[contract]` implementation

## Error Types

### `ContractError`

```rust
#[derive(Debug)]
pub enum ContractError {
    IoError(std::io::Error),
    NotToml,
    MissingDependencies,
    MissingBinaries,
    MissingFeatures,
    WrongPath,
    CyclicDependency,
}
```

Comprehensive error type covering all contract validation failures.

**Variants:**
- `IoError(std::io::Error)` - File system operation failed
- `NotToml` - Invalid or malformed `Cargo.toml`
- `MissingDependencies` - Required dependencies not found
- `MissingBinaries` - Binary targets not properly configured
- `MissingFeatures` - Required Cargo features missing
- `WrongPath` - Invalid dependency path
- `CyclicDependency` - Circular dependency detected (future use)

**Trait Implementations:**
- `From<std::io::Error> for ContractError` - Automatic conversion from I/O errors

## Utility Functions

### `obtain_contract_by_path`

```rust
pub fn obtain_contract_by_path(path: &Path) -> Option<ContractWithDeps>
```

Discovers and validates a Hybrid Framework smart contract at the specified path.

**Parameters:**
- `path: &Path` - Path to the contract project root directory

**Returns:**
- `Some(ContractWithDeps)` - Valid contract found and validated
- `None` - No valid contract found or validation failed

**Validation Process:**
1. Verify path is a directory
2. Locate and validate `Cargo.toml`
3. Parse contract structure using `ContractWithDeps::try_from`
4. Extract contract identifier from `src/lib.rs`
5. Resolve dependencies (placeholder for future enhancement)

**Error Handling:**
- Logs errors at appropriate levels (`error!`, `warn!`)
- Returns `None` for all failure cases
- Provides detailed error messages in logs

### `find_contract_ident`

```rust
pub fn find_contract_ident(file_path: &Path) -> Result<String, anyhow::Error>
```

Extracts the contract identifier from Rust source code containing `#[contract]` implementation.

**Parameters:**
- `file_path: &Path` - Path to Rust source file (typically `src/lib.rs`)

**Returns:**
- `Ok(String)` - Contract identifier from `#[contract]` impl block
- `Err(anyhow::Error)` - File parsing failed or no contract found

**Process:**
1. Read and parse Rust source using `syn::parse_file`
2. Traverse AST looking for `impl` blocks
3. Check for `#[contract]` attribute on impl blocks
4. Extract type identifier from impl target

**Supported Patterns:**
```rust
#[contract]
impl SimpleContract { /* ... */ }           // → "SimpleContract"

#[contract]
impl path::to::Contract { /* ... */ }       // → "Contract"

#[contract]
impl GenericContract<T> { /* ... */ }       // → "GenericContract"
```

### Helper Functions

#### `has_contract_attribute`

```rust
fn has_contract_attribute(attrs: &[Attribute]) -> bool
```

Private helper that checks if attributes contain `#[contract]`.

#### `extract_ident`

```rust
fn extract_ident(item_impl: &ItemImpl) -> Option<String>
```

Private helper that extracts type identifier from impl block.

## Usage Patterns

### Basic Compilation

```rust
use hybrid_compile::run_contract_compilation;
use indicatif::ProgressBar;
use std::path::Path;

// Simple compilation
let path = Path::new("./my-contract");
let pb = ProgressBar::new(100);
run_contract_compilation(path, false, pb, "out".to_string())?;
```

### Syntax Check Only

```rust
// Check without generating binaries
run_contract_compilation(path, true, pb, "check".to_string())?;
```

### Contract Discovery

```rust
use hybrid_compile::utils::obtain_contract_by_path;

if let Some(contract) = obtain_contract_by_path(path) {
    println!("Found: {} ({})", contract.name.ident, contract.name.package);
    println!("Dependencies: {}", contract.deps.len());
}
```

### Error Handling

```rust
match run_contract_compilation(path, false, pb, "out".to_string()) {
    Ok(()) => println!("Success!"),
    Err(e) => {
        if e.to_string().contains("missing dependency") {
            eprintln!("Add hybrid-derive and hybrid-contract to Cargo.toml");
        } else {
            eprintln!("Compilation error: {}", e);
        }
    }
}
```

## Examples

### CLI Tool Integration

```rust
use clap::Parser;
use hybrid_compile::run_contract_compilation;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    path: String,
    #[arg(short, long)]
    check: bool,
}

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    let pb = ProgressBar::new(100);
    
    run_contract_compilation(
        Path::new(&args.path),
        args.check,
        pb,
        "build".to_string(),
    )
}
```

### Batch Processing

```rust
use hybrid_compile::utils::obtain_contract_by_path;

fn discover_all_contracts(base_path: &Path) -> Vec<ContractWithDeps> {
    let mut contracts = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(base_path) {
        for entry in entries.flatten() {
            if let Some(contract) = obtain_contract_by_path(&entry.path()) {
                contracts.push(contract);
            }
        }
    }
    
    contracts
}
```

### Build System Integration

```rust
// build.rs
use hybrid_compile::run_contract_compilation;

fn main() {
    let contracts_dir = Path::new("contracts");
    
    for entry in std::fs::read_dir(contracts_dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            let pb = ProgressBar::hidden();
            run_contract_compilation(&path, false, pb, "target/contracts".to_string())
                .expect("Contract compilation failed");
        }
    }
    
    println!("cargo:rerun-if-changed=contracts/");
}
```

## Requirements

### Toolchain Requirements

- Rust nightly toolchain: `nightly-2025-01-07`
- RISC-V target: `riscv64imac-unknown-none-elf`
- Build-std support for `core` and `alloc`

### Contract Structure Requirements

#### Cargo.toml
```toml
[package]
name = "my-contract"
version = "0.1.0"
edition = "2021"

[features]
default = []
deploy = []
interface-only = []

[dependencies]
hybrid-derive = { path = "../hybrid-derive" }
hybrid-contract = { path = "../hybrid-contract" }

[[bin]]
name = "runtime"
path = "src/lib.rs"

[[bin]]
name = "deploy" 
path = "src/lib.rs"
required-features = ["deploy"]
```

#### Source Code
```rust
// src/lib.rs
#![no_std]
#![no_main]

use hybrid_derive::{contract, storage};

#[storage]
struct MyContract {
    value: u64,
}

#[contract]
impl MyContract {
    pub fn new() -> Self {
        Self { value: 0 }
    }
}
```

## Performance Considerations

- **Compilation Time**: Initial compilation can be slow due to RISC-V target
- **Binary Size**: RISC-V binaries are larger than native EVM bytecode
- **Memory Usage**: Requires sufficient memory for Rust compilation
- **Disk Space**: Generated artifacts require significant storage

## Limitations

- Contract inheritance not fully supported
- Complex dependency graphs may cause issues
- Requires specific nightly toolchain version
- Limited to `no_std` environment
- Circular dependencies not detected/handled

## Future Enhancements

- Full contract inheritance support
- Dependency cycle detection
- Incremental compilation
- Build caching
- Multi-target support
- Custom optimization levels