//! # Utilities
//!
//! Utility functions for contract discovery, validation, and source code analysis.
//!
//! This module provides the essential functionality for discovering and parsing Rust
//! smart contract projects within the Hybrid Framework. It handles the complex task
//! of analyzing contract source code to extract metadata and validate structure.
//!
//! ## Key Functions
//!
//! - [`obtain_contract_by_path`]: Main entry point for contract discovery
//! - [`find_contract_ident`]: Extracts contract identifier from Rust source code
//!
//! ## Contract Discovery Process
//!
//! The contract discovery process follows these steps:
//! 1. **Path Validation**: Ensures the provided path is a valid directory
//! 2. **Cargo.toml Detection**: Looks for and validates the Cargo.toml file
//! 3. **Structure Validation**: Validates contract structure and dependencies
//! 4. **Identifier Extraction**: Parses Rust source to find the contract identifier
//! 5. **Dependency Resolution**: Resolves contract dependencies (currently limited)
//!
//! ## Source Code Analysis
//!
//! The module uses `syn` (Rust's parsing library) to analyze contract source code
//! and extract the contract identifier from `#[contract]` attribute implementations.
use std::{fs, path::Path};
use syn::{Attribute, Item, ItemImpl};
use tracing::{error, warn};

use crate::primitives::{ContractError, ContractWithDeps};

/// Discovers and validates a Hybrid Framework smart contract at the specified path.
///
/// This function serves as the main entry point for contract discovery and validation.
/// It performs comprehensive checks to ensure the project meets all requirements for
/// a Hybrid Framework smart contract, then extracts the contract identifier and
/// resolves dependencies.
///
/// # Contract Requirements
///
/// For a project to be recognized as a valid contract, it must:
/// 1. Be a directory containing a `Cargo.toml` file
/// 2. Have required Cargo features: `default`, `deploy`, `interface-only`
/// 3. Define required binary targets: `runtime` and `deploy`
/// 4. Include required dependencies: `hybrid-derive` and `hybrid-contract`
/// 5. Have a `src/lib.rs` file with a `#[contract]` implementation
///
/// # Arguments
///
/// * `path` - Path to the contract project root directory
///
/// # Returns
///
/// Returns `Some(ContractWithDeps)` if a valid contract is found and validated,
/// or `None` if the path doesn't contain a valid contract or validation fails.
pub fn obtain_contract_by_path(path: &Path) -> Option<ContractWithDeps> {
    if !path.is_dir() {
        // currently smart contract have to a lib create hereby dir
        return None;
    }

    let cargo_path = path.join("Cargo.toml");
    if !cargo_path.exists() {
        return None;
    }

    // Try to parse as R55 contract
    match ContractWithDeps::try_from(&cargo_path) {
        Ok(mut contract) => {
            let lib_path = contract.path.join("src").join("lib.rs");
            let ident = match find_contract_ident(&lib_path) {
                Ok(ident) => ident,
                Err(e) => {
                    error!(
                        "Unable to find contract identifier at {:?}: {:?}",
                        lib_path, e
                    );
                    return None;
                }
            };
            contract.name.ident = ident;

            for _dep in &mut contract.deps {
                // TODO: Contract with another depending contract is not supported at the moment ion other words, inheritance is not allowed at the moment
            }

            return Some(contract);
        }
        Err(ContractError::MissingDependencies) => {
            error!("Hybrid missing dependency");

            return None;
        }
        Err(ContractError::MissingBinaries) => {
            error!("Hybrid missing binary");

            return None;
        }
        Err(ContractError::MissingFeatures) => {
            error!("Hybrid missing feature");

            return None;
        }
        Err(e) => {
            warn!(
                "Error parsing potential contract at {:?}: {:?}",
                cargo_path, e
            );

            None
        }
    }
}

/// Extracts the contract identifier from a Rust source file containing a `#[contract]` implementation.
///
/// This function parses Rust source code using the `syn` crate to locate implementation blocks
/// decorated with the `#[contract]` attribute and extracts the contract's type identifier.
/// This identifier is used throughout the compilation process and becomes part of the
/// contract's ABI interface.
///
/// # How It Works
///
/// 1. **File Parsing**: Reads and parses the Rust source file using `syn::parse_file`
/// 2. **AST Traversal**: Iterates through all top-level items in the syntax tree
/// 3. **Impl Block Detection**: Identifies `impl` blocks (implementation blocks)
/// 4. **Attribute Checking**: Looks for the `#[contract]` attribute on impl blocks
/// 5. **Type Extraction**: Extracts the type name from the impl block's target type
///
/// # Arguments
///
/// * `file_path` - Path to the Rust source file to analyze (typically `src/lib.rs`)
///
/// # Returns
///
/// Returns the contract identifier as a `String` on success, or an error if:
/// - The file cannot be read or parsed
/// - No `#[contract]` implementation is found
/// - The contract type cannot be extracted
pub fn find_contract_ident(file_path: &Path) -> Result<String, anyhow::Error> {
    // Read and parse the file content
    let content = fs::read_to_string(file_path)?;
    let file = syn::parse_file(&content)?;

    // Look for impl blocks with #[contract] attribute
    for item in file.items {
        if let Item::Impl(item_impl) = item {
            // Check if this impl block has the #[contract] attribute
            if has_contract_attribute(&item_impl.attrs) {
                // Extract the type name from the impl block
                if let Some(ident) = extract_ident(&item_impl) {
                    return Ok(ident);
                }
            }
        }
    }

    anyhow::bail!("No contract implementation found in file: {:?}", file_path)
}

/// Checks if a list of attributes contains the `#[contract]` attribute.
///
/// This helper function examines attribute syntax nodes to determine if any of them
/// represent the `#[contract]` attribute that marks a contract implementation.
///
/// # Arguments
///
/// * `attrs` - Slice of `syn::Attribute` nodes to examine
fn has_contract_attribute(attrs: &[Attribute]) -> bool {
    attrs
        .iter()
        .any(|attr| attr.path.segments.len() == 1 && attr.path.segments[0].ident == "contract")
}

/// Extracts the type identifier from a Rust impl block.
///
/// This function analyzes the target type of an `impl` block and extracts its
/// identifier for use as the contract name. It handles simple type paths and
/// returns the final component as the type name.
///
/// # Arguments
///
/// * `item_impl` - The `syn::ItemImpl` node representing an impl block
///
/// # Returns
///
/// Returns `Some(String)` containing the type identifier if extraction succeeds,
/// or `None` if the type structure is too complex or unsupported.
fn extract_ident(item_impl: &ItemImpl) -> Option<String> {
    match &*item_impl.self_ty {
        syn::Type::Path(type_path) if !type_path.path.segments.is_empty() => {
            // Get the last segment of the path (the type name)
            let segment = type_path.path.segments.last().unwrap();
            Some(segment.ident.to_string())
        }
        _ => None,
    }
}
