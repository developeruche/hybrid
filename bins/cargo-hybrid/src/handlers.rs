//! Handlers for the cargo-cli command
use crate::command::{BuildArgs, DeployArgs, NewArgs};
use crate::utils::deploy_riscv_bytecode;
use alloy::primitives::hex;
use anyhow::{anyhow, Result};
use colored::Colorize;
use hybrid_compile::run_contract_compilation;
use include_dir::{include_dir, Dir};
use indicatif::{ProgressBar, ProgressStyle};
use std::{fs, path::PathBuf, process::Command};
use toml_edit::{value, DocumentMut};
use tracing::info;

// Include the templates directory at compile time
static TEMPLATES_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../../contracts");

/// Create a new project from a template
pub fn create_new_project(args: &NewArgs) -> Result<()> {
    // Validate the template
    let template_dir = match TEMPLATES_DIR.get_dir(&args.template) {
        Some(dir) => dir,
        None => {
            // Get available templates for the error message
            let available_templates: Vec<String> = TEMPLATES_DIR
                .dirs()
                .map(|dir| {
                    dir.path()
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string()
                })
                .collect();

            return Err(anyhow!(
                "Template '{}' not found. Available templates: {}",
                args.template,
                available_templates.join(", ")
            ));
        }
    };

    // Create the target directory
    let target_dir = PathBuf::from(&args.name);
    if target_dir.exists() {
        return Err(anyhow!("Directory '{}' already exists", args.name));
    }

    info!(
        "Creating new project: {} (template: {})",
        args.name.bold(),
        args.template.bold()
    );

    // Set up the progress bar
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Copying template files...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    // Create the target directory
    fs::create_dir_all(&target_dir)?;

    // Helper function to recursively extract files
    fn extract_recursively(
        dir: &include_dir::Dir,
        target_dir: &PathBuf,
        template_name: &str,
    ) -> Result<()> {
        // Process all files in the current directory
        for file in dir.files() {
            let rel_path = file.path().to_string_lossy();
            // Get path relative to the template root
            let file_path = rel_path.trim_start_matches(&format!("{}/", template_name));
            let target_file_path = target_dir.join(file_path);

            // Create parent directories if needed
            if let Some(parent) = target_file_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::write(target_file_path, file.contents())?;
        }

        // Process all subdirectories
        for subdir in dir.dirs() {
            let rel_path = subdir.path().to_string_lossy();
            // Get path relative to the template root
            let dir_path = rel_path.trim_start_matches(&format!("{}/", template_name));
            let target_subdir_path = target_dir.join(dir_path);

            // Create the subdirectory
            fs::create_dir_all(&target_subdir_path)?;

            // Recursively extract its contents
            extract_recursively(subdir, target_dir, template_name)?;
        }

        Ok(())
    }

    // Extract all template files
    extract_recursively(template_dir, &target_dir, &args.template)?;

    // Update the project name in Cargo.toml
    let cargo_toml_path = target_dir.join("Cargo.toml");
    if cargo_toml_path.exists() {
        let cargo_toml = fs::read_to_string(&cargo_toml_path)?;
        let mut doc = cargo_toml.parse::<DocumentMut>()?;

        // Update the package name
        if let Some(package) = doc.get_mut("package") {
            if let Some(name) = package.get_mut("name") {
                *name = value(&args.name); // Set the new name
            } else {
                return Err(anyhow::anyhow!(
                    "No 'name' field found in 'package' section"
                ));
            }
        } else {
            return Err(anyhow::anyhow!("No 'package' section found in Cargo.toml"));
        }

        fs::write(&cargo_toml_path, doc.to_string())?;
    }

    pb.finish_with_message(format!(
        "Project '{}' created successfully!",
        args.name.green()
    ));

    println!(
        "\n🎉 {} {}\n",
        "New project created:".green().bold(),
        args.name.cyan()
    );
    println!("To get started:");
    println!("  cd {}", args.name);
    println!("  cargo hybrid build");
    println!("\nHappy coding! 🚀\n");

    Ok(())
}

/// Start the hybrid node in development mode
pub fn start_node() -> Result<()> {
    info!("Starting hybrid node in development mode...");

    println!(
        "🚀 {} {}",
        "Starting Hybrid Node".green().bold(),
        "in development mode".cyan()
    );

    // Check if hybrid-node is installed
    let status = Command::new("which").arg("hybrid-node").status();

    if status.is_err() || !status.unwrap().success() {
        return Err(anyhow!("'hybrid-node' command not found. Please make sure it's installed and available in your PATH."));
    }

    // Execute the hybrid-node command with the --dev flag
    let child = Command::new("hybrid-node").arg("--dev").spawn()?;

    // Print a message about how to stop the node
    println!(
        "\n💡 {} {}",
        "Node is running.".green(),
        "Press Ctrl+C to stop the node."
    );

    // Wait for the command to complete
    let status = child.wait_with_output()?;

    if !status.status.success() {
        return Err(anyhow!(
            "Node exited with an error. Status code: {:?}",
            status.status.code()
        ));
    }

    Ok(())
}

/// Build the smart contract
pub fn build_contract(args: &BuildArgs, check_only: bool) -> Result<()> {
    // Get the current directory as the contract root
    let contract_root = std::env::current_dir()?;

    // Check if this is a valid contract directory
    let cargo_toml = contract_root.join("Cargo.toml");
    if !cargo_toml.exists() {
        return Err(anyhow!(
            "Not a valid Hybrid contract directory. Cargo.toml not found."
        ));
    }

    // Create the output directory if it doesn't exist and not in check-only mode
    let output_dir = contract_root.join(&args.out);
    if !check_only && !output_dir.exists() {
        fs::create_dir_all(&output_dir)?;
    }

    info!(
        "{} contract...",
        if check_only { "Checking" } else { "Building" }
    );

    // Set up the progress bar
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    // Use the compile crate's run_contract_compilation function
    run_contract_compilation(&contract_root, check_only, pb, args.out.clone())?;

    Ok(())
}

/// Deploy the smart contract
pub fn deploy_contract(args: &DeployArgs) -> Result<()> {
    // Check if the output directory exists
    let contract_root = std::env::current_dir()?;
    let output_dir = contract_root.join(&args.out);

    if !output_dir.exists() {
        return Err(anyhow!(
            "Output directory '{}' not found. Run 'cargo hybrid build' first.",
            args.out
        ));
    }

    // Find the compiled binary files
    let bin_files: Vec<_> = fs::read_dir(&output_dir)?
        .filter_map(Result::ok)
        .filter(|entry| {
            let path = entry.path();
            path.is_file() && path.extension().map_or(false, |ext| ext == "bin")
        })
        .collect();

    if bin_files.is_empty() {
        return Err(anyhow!(
            "No compiled contracts found in '{}'. Run 'cargo hybrid build' first.",
            args.out
        ));
    }

    // Get the contract name from the binary file
    let bin_path = &bin_files[0].path();
    let contract_name = bin_path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown");

    info!(
        "Deploying contract '{}' to {}",
        contract_name.bold(),
        args.rpc.bold()
    );

    // Set up the progress bar
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Connecting to network...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    // Read the bytecode from the binary file
    pb.set_message(format!("Reading bytecode for '{}'...", contract_name));
    let bytecode = fs::read(bin_path)?;

    pb.set_message("Deploying contract to the blockchain...");

    // Parse encoded arguments if provided
    let encoded_args = match &args.encoded_args {
        Some(hex_args) => {
            // Remove 0x prefix if present
            let clean_hex = hex_args.trim_start_matches("0x");

            // Parse hex string to bytes
            Some(
                hex::decode(clean_hex)
                    .map_err(|e| anyhow!("Failed to decode constructor arguments: {}", e))?,
            )
        }
        None => None,
    };

    // Run the deployment logic using tokio runtime
    let rt = tokio::runtime::Runtime::new()?;
    let contract_address = rt.block_on(async {
        deploy_riscv_bytecode(&args.rpc, &args.private_key, bytecode, encoded_args).await
    })?;

    pb.finish_with_message("Contract deployed successfully!".green().to_string());
    println!(
        "\n🚀 {} {} {}\n",
        "Contract deployed at:".green().bold(),
        contract_address.to_string().cyan(),
        "🎉".green()
    );

    Ok(())
}
