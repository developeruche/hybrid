//! cargo-hybrid cli
mod command;
mod handlers;
mod utils;

use anyhow::Result;
use clap::Parser;
use command::{BuildArgs, Cli, Commands, HybridSubcommands};
use handlers::{build_contract, create_new_project, deploy_contract, start_node};
use utils::init_logger;

fn main() -> Result<()> {
    // Initialize the logger
    init_logger();

    // Parse the command line arguments
    let cli = Cli::parse();

    // Handle the command
    match cli.command {
        Some(Commands::Hybrid(hybrid_commands)) => match hybrid_commands.command {
            HybridSubcommands::New(args) => create_new_project(&args)?,
            HybridSubcommands::Build(args) => build_contract(&args, false)?,
            HybridSubcommands::Check => build_contract(
                &BuildArgs {
                    out: "out".to_string(),
                    bytecode_type: "deploy".to_string(),
                },
                true,
            )?,
            HybridSubcommands::Deploy(args) => deploy_contract(&args)?,
            HybridSubcommands::Node => start_node()?,
        },
        None => {
            println!("Usage: cargo hybrid <COMMAND>");
            println!("\nFor more information try 'cargo hybrid --help'");
        }
    }

    Ok(())
}
