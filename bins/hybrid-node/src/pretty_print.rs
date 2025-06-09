//! This holds print displays for the node
use colored::Colorize;
use std::time::Duration;

pub fn print_startup_banner(is_dev: bool) {
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

pub fn print_config(is_dev: bool) {
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
