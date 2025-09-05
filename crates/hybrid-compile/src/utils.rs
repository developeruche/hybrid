//! Utility functions used for the compile of contracts
use std::{fs, path::Path};
use syn::{Attribute, Item, ItemImpl};
use tracing::{error, warn};

use crate::primitives::{ContractError, ContractWithDeps};

/// Function that the path to a contract and returns a structured
/// contract and coresponding dependency
pub fn obtain_contract_by_path(path: &Path) -> Option<ContractWithDeps> {
    println!("Path: {}", path.to_str().unwrap());

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

// Check if attributes contain #[contract]
fn has_contract_attribute(attrs: &[Attribute]) -> bool {
    attrs
        .iter()
        .any(|attr| attr.path.segments.len() == 1 && attr.path.segments[0].ident == "contract")
}

// Extract the type name from its impl block
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
