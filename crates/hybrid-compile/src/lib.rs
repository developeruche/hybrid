//! # Hybrid Compile
//!
//! A Rust compiler crate for building RISC-V smart contracts that run on EVM-compatible blockchains.
//!
//! This crate provides the core compilation infrastructure for the Hybrid Framework, enabling
//! developers to write smart contracts in Rust and compile them to RISC-V bytecode that can
//! be executed on any EVM-compatible blockchain.
//!
//! ## Key Features
//!
//! - **Rust-to-RISC-V Compilation**: Compiles Rust smart contracts to optimized RISC-V bytecode
//! - **Contract Discovery**: Automatically discovers and validates contract projects
//! - **Dependency Resolution**: Handles contract dependencies and interface-only imports
//! - **Multi-stage Compilation**: Separates runtime and deployment compilation stages
//! - **Progress Tracking**: Provides visual feedback during compilation
//!
//! ## Architecture
//!
//! The crate is organized into three main modules:
//! - [`primitives`]: Core data structures and compilation logic
//! - [`utils`]: Utility functions for contract discovery and parsing
//!
//! ## Usage
//!
//! The primary entry point is [`run_contract_compilation`], which orchestrates the entire
//! compilation process:
//!
//! ```rust,no_run
//! use hybrid_compile::run_contract_compilation;
//! use std::path::Path;
//! use indicatif::ProgressBar;
//!
//! let contract_root = Path::new("./my-contract");
//! let progress_bar = ProgressBar::new(100);
//! let output_dir = "out".to_string();
//!
//! // Compile the contract
//! run_contract_compilation(contract_root, false, progress_bar, output_dir)?;
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! ## Contract Requirements
//!
//! For a Rust project to be recognized as a valid Hybrid contract, it must:
//!
//! 1. Have a valid `Cargo.toml` with required features: `default`, `deploy`, `interface-only`
//! 2. Define required binary targets: `runtime` and `deploy`
//! 3. Include required dependencies: `hybrid-derive` and `hybrid-contract`
//! 4. Have a `src/lib.rs` with a `#[contract]` implementation
//!
//! ## Compilation Process
//!
//! The compilation process consists of two stages:
//!
//! 1. **Runtime Compilation**: Generates bytecode for normal contract execution
//! 2. **Deploy Compilation**: Generates bytecode for contract deployment
//!
//! Both stages target `riscv64imac-unknown-none-elf` and use Rust's `build-std` feature
//! for no-std compatibility.
//!
//! ## Error Handling
//!
//! All functions return `Result<(), anyhow::Error>` for comprehensive error reporting.
//! Common errors include missing dependencies, invalid contract structure, and compilation failures.

pub mod primitives;
pub mod utils;

use colored::Colorize;
use indicatif::ProgressBar;
use primitives::Contract;
use std::{fs, path::Path};
use tracing::info;
use utils::obtain_contract_by_path;

/// Compiles a Rust smart contract to RISC-V bytecode for deployment on EVM-compatible blockchains.
///
/// This is the main entry point for contract compilation, handling the entire pipeline from
/// contract discovery to binary generation. The function performs validation, dependency
/// resolution, and multi-stage compilation to produce deployment-ready bytecode.
///
/// # Arguments
/// # Errors
///
/// This function can return various errors:
/// - Contract not found or invalid structure
/// - Missing required dependencies or features
/// - Compilation failures (syntax errors, type errors, etc.)
/// - File system errors during binary generation
pub fn run_contract_compilation(
    contract_root: &Path,
    is_check: bool,
    pb: ProgressBar,
    out: String,
) -> Result<(), anyhow::Error> {
    let output_dir = contract_root.join("out");
    fs::create_dir_all(&output_dir)?;

    let contract: Contract = obtain_contract_by_path(contract_root)
        .ok_or(anyhow::anyhow!("contract fetch by path error"))?
        .into();

    info!("Compiling contract: {}", contract.name.ident);

    let deploy_bytecode = contract.compile_r55()?;
    let deploy_path = output_dir.join(format!("{}.bin", contract.name.package));

    if is_check {
        pb.finish_with_message("Contract check completed successfully!".green().to_string());
        println!("\n✅ {}\n", "Contract syntax check passed!".green().bold());
    } else {
        fs::write(deploy_path, deploy_bytecode)?;
        pb.finish_with_message("Contract build completed successfully!".green().to_string());
        println!(
            "\n✅ {} to {}\n",
            "Contract built successfully".green().bold(),
            out.cyan()
        );
    }

    Ok(())
}

pub fn run_contract_compilation_runtime(
    contract_root: &Path,
    is_check: bool,
    pb: ProgressBar,
    out: String,
) -> Result<(), anyhow::Error> {
    let output_dir = contract_root.join("out");
    fs::create_dir_all(&output_dir)?;

    let contract: Contract = obtain_contract_by_path(contract_root)
        .ok_or(anyhow::anyhow!("contract fetch by path error"))?
        .into();

    info!("Compiling contract: {}", contract.name.ident);

    let runtime_bytecode = contract.compile_runtime()?;
    let deploy_path = output_dir.join(format!("{}.bin", contract.name.package));

    if is_check {
        pb.finish_with_message("Contract check completed successfully!".green().to_string());
        println!("\n✅ {}\n", "Contract syntax check passed!".green().bold());
    } else {
        fs::write(deploy_path, runtime_bytecode)?;
        pb.finish_with_message("Contract build completed successfully!".green().to_string());
        println!(
            "\n✅ {} to {}\n",
            "Contract built successfully".green().bold(),
            out.cyan()
        );
    }

    Ok(())
}
