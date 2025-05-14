//! Hybrid blockchain node binary
use clap::{Parser, Subcommand};
use colored::Colorize;
use eyre::Result;
use std::time::Duration;
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};

/// Hybrid blockchain node
#[derive(Parser)]
#[clap(name = "hybrid-node", version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,

    /// Run as development node with additional debugging features
    #[clap(long, global = true)]
    dev: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the blockchain node
    Start,

    /// Print node configuration
    Config,
}

/// Initialize the logger with a nice formatted output
fn init_logger() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,vm=debug,reth=info"));

    fmt::fmt().with_env_filter(filter).with_target(false).init();
}

#[tokio::main]
async fn main() -> Result<()> {
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

fn print_startup_banner(is_dev: bool) {
    let node_type = if is_dev {
        "DEVELOPMENT".yellow()
    } else {
        "PRODUCTION".bright_blue()
    };
    println!(
        "\n{}",
        "╔═════════════════════════════════════════════╗".bright_cyan()
    );
    println!(
        "{} {}  {}",
        "║".bright_cyan(),
        " HYBRID BLOCKCHAIN NODE ".bold(),
        "║".bright_cyan()
    );
    println!(
        "{} {}      {}",
        "║".bright_cyan(),
        node_type,
        "║".bright_cyan()
    );
    println!(
        "{}",
        "╚═════════════════════════════════════════════╝".bright_cyan()
    );
    println!();

    // Small delay for visual effect
    std::thread::sleep(Duration::from_millis(100));
}

fn print_config(is_dev: bool) {
    println!("\n{}", "HYBRID NODE CONFIGURATION".bold());
    println!("-------------------------");
    println!(
        "Mode: {}",
        if is_dev {
            "Development".yellow()
        } else {
            "Production".bright_blue()
        }
    );
    println!("Chain: Mainnet");
    println!("HTTP RPC: Enabled");
    println!("WebSocket RPC: Disabled");
    println!("-------------------------\n");
}
