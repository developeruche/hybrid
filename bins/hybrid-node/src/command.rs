//! CLI commands for the hybrid blockchain node.
use clap::{Parser, Subcommand};

/// Hybrid blockchain node
#[derive(Parser)]
#[clap(name = "hybrid-node", version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Option<Commands>,

    /// Run as development node with additional debugging features
    #[clap(long, global = true)]
    pub dev: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the blockchain node
    Start,
    /// Print node configuration
    Config,
}
