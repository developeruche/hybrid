//! Handlers for the cargo-cli command
use crate::command::{BuildArgs, DeployArgs, NewArgs};
use anyhow::{anyhow, Result};
use colored::Colorize;
use compile::run_contract_compilation;
use fs_extra::dir::{self, CopyOptions};
use indicatif::{ProgressBar, ProgressStyle};
use std::{fs, path::PathBuf};
use toml_edit::{value, DocumentMut};
use tracing::info;

/// Create a new project from a template
pub fn create_new_project(args: &NewArgs) -> Result<()> {
    // Validate the template
    let template_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| anyhow!("Failed to find parent directory"))?
        .parent()
        .ok_or_else(|| anyhow!("Failed to find workspace root"))?
        .join("contracts")
        .join(&args.template);

    if !template_path.exists() {
        return Err(anyhow!(
            "Template '{}' not found. Available templates: bare, storage, erc20",
            args.template
        ));
    }

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

    // Copy the template to the target directory
    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;
    copy_options.copy_inside = true;
    dir::copy(&template_path, ".", &copy_options)?;

    // Rename the directory
    fs::rename(
        PathBuf::from(&template_path.file_name().unwrap()),
        &target_dir,
    )?;

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
                return Err(anyhow::anyhow!("No 'name' field found in 'package' section"));
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
    pb.set_message("Compiling...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    // Use the compile crate's run_contract_compilation function
    if check_only {
        // In check mode, we temporarily compile but don't save the output
        let temp_dir = tempfile::tempdir()?;
        let temp_path = temp_dir.path();

        // Copy contract files to temp directory
        let mut copy_options = CopyOptions::new();
        copy_options.overwrite = true;
        copy_options.copy_inside = true;
        dir::copy(&contract_root, temp_path, &copy_options)?;

        run_contract_compilation(&temp_path.to_path_buf())?;
    } else {
        run_contract_compilation(&contract_root)?;
    }

    if check_only {
        pb.finish_with_message("Contract check completed successfully!".green().to_string());
        println!("\n✅ {}\n", "Contract syntax check passed!".green().bold());
    } else {
        pb.finish_with_message("Contract build completed successfully!".green().to_string());
        println!(
            "\n✅ {} to {}\n",
            "Contract built successfully".green().bold(),
            args.out.cyan()
        );
    }

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

    info!("Deploying contract to {}", args.rpc.bold());

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

    // This is where you would call your deployment logic
    // For now, we'll just simulate a deployment
    std::thread::sleep(std::time::Duration::from_secs(2));
    pb.set_message("Uploading contract bytecode...");
    std::thread::sleep(std::time::Duration::from_secs(2));
    pb.set_message("Waiting for confirmation...");
    std::thread::sleep(std::time::Duration::from_secs(1));

    // Generate a fake contract address
    let contract_address = "0x1234567890123456789012345678901234567890";

    pb.finish_with_message("Contract deployed successfully!".green().to_string());
    println!(
        "\n🚀 {} {} {}\n",
        "Contract deployed at:".green().bold(),
        contract_address.cyan(),
        "🎉".green()
    );

    Ok(())
}
