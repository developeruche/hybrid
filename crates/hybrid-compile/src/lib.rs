//! Compiling RUST smart contract targetting RISCV
pub mod primitivies;
pub mod utils;

use colored::Colorize;
use indicatif::ProgressBar;
use primitivies::Contract;
use std::{fs, path::Path};
use tracing::info;
use utils::obtain_contract_by_path;

/// This is the function the main binary cli application would use to compile the contract
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
