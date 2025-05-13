//! Compiling RUST smart contract targetting RISCV
use std::{fs, path::Path};

use utils::obtain_contract_by_path;
pub mod primitivies;
pub mod utils;

pub fn run_contract_compilation(contract_root: &Path) -> Result<(), anyhow::Error> {
    let output_dir = contract_root.join("out");
    fs::create_dir_all(&output_dir)?;

    let contract = obtain_contract_by_path(contract_root);

    Ok(())
}
