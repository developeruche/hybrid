//! Hybrid blockchain node binary
use clap::Parser;
use command::{Cli, Commands};
use eyre::Result;
use pretty_print::{print_config, print_startup_banner};
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};

mod command;
mod pretty_print;

/// Initialize the logger with a nice formatted output
fn init_logger() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,vm=debug,reth=info"));

    fmt::fmt().with_env_filter(filter).with_target(false).init();
}

async fn start_node(is_dev: bool) -> Result<()> {
    info!(
        "{} node...",
        if is_dev {
            "Starting development"
        } else {
            "Starting"
        }
    );

    // Run the node using the vm crate's run_node function
    vm::run_node(is_dev)
        .await
        .map_err(|e| eyre::eyre!("Node error: {}", e))?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), eyre::Error> {
    // Initialize the logger
    init_logger();

    // Parse command line arguments
    let cli = Cli::parse();
    let is_dev = cli.dev;

    match cli.command {
        Some(Commands::Start) | None => {
            print_startup_banner(is_dev);
            start_node(is_dev).await?;
        }
        Some(Commands::Config) => {
            print_config(is_dev);
        }
    }

    Ok(())
}
