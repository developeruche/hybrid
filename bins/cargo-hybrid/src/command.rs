//! holding command related structures
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[clap(name = "cargo-hybrid", bin_name = "cargo")]
#[clap(
    version,
    about = "Hybrid blockchain tools for smart contract developers"
)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Hybrid blockchain tools for smart contract developers
    #[clap(name = "hybrid")]
    Hybrid(HybridCommands),
}

#[derive(Args)]
pub struct HybridCommands {
    #[clap(subcommand)]
    pub command: HybridSubcommands,
}

#[derive(Subcommand)]
pub enum HybridSubcommands {
    /// Create a new smart contract project
    New(NewArgs),

    /// Build the smart contract
    Build(BuildArgs),

    /// Check if the smart contract compiles without updating the out directory
    Check,

    /// Deploy a smart contract to the blockchain
    Deploy(DeployArgs),

    /// Start the hybrid node in development mode
    Node,
}

#[derive(Args)]
pub struct NewArgs {
    /// Template to use for the new project
    #[clap(long, default_value = "storage")]
    pub template: String,

    /// Name of the project
    #[clap(default_value = "my-hybrid-contract")]
    pub name: String,
}

#[derive(Args)]
pub struct BuildArgs {
    /// Output directory for the compiled contract
    #[clap(long, default_value = "out")]
    pub out: String,
}

#[derive(Args)]
pub struct DeployArgs {
    /// Path to the output directory containing the compiled contract
    #[clap(long, default_value = "out")]
    pub out: String,

    /// RPC endpoint to deploy to
    #[clap(long, default_value = "http://127.0.0.1:8545")]
    pub rpc: String,

    /// Private key to use for deployment
    #[clap(
        long,
        default_value = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
    )]
    pub private_key: String,

    /// Constructor arguments (hex encoded, with or without 0x prefix)
    /// Example: --encoded-args 0000000000000000000000000000000000000000000000000000000000000020
    #[clap(long)]
    pub encoded_args: Option<String>,
}
