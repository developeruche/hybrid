//! # Primitives
//!
//! Core data structures and compilation logic for the Hybrid Framework contract compiler.
//!
//! This module defines the fundamental types used throughout the compilation process,
//! including contract representation, error handling, and the compilation pipeline itself.
//!
//! ## Key Components
//!
//! - [`Contract`]: Represents a compiled contract with its metadata
//! - [`ContractWithDeps`]: Represents a contract with its dependencies
//! - [`ContractName`]: Contract identification information
//! - [`ContractError`]: Comprehensive error types for compilation failures
//!
//! ## Compilation Pipeline
//!
//! The compilation process follows these stages:
//! 1. **Discovery**: Parse `Cargo.toml` and validate contract structure
//! 2. **Runtime Compilation**: Generate bytecode for normal contract execution
//! 3. **Deploy Compilation**: Generate bytecode for contract deployment
//! 4. **Binary Generation**: Combine and prefix bytecodes for deployment
//!
//! ## Target Architecture
//!
//! All compilation targets the `riscv64imac-unknown-none-elf` architecture with:
//! - No standard library (`no_std`)
//! - Core and alloc only (`-Z build-std=core,alloc`)
//! - Optimized release builds (`-r`)
//! - Specific nightly toolchain (`nightly-2025-01-07`)
use std::{fmt, fs, io::Read, path::PathBuf, process::Command};
use toml::Value;
use tracing::{debug, error, info};

/// Errors that can occur during contract compilation and validation.
///
/// This enum covers all possible failure modes during the contract compilation process,
/// from file system errors to structural validation failures.
#[derive(Debug)]
pub enum ContractError {
    /// File system operation failed (read, write, directory access, etc.)
    IoError(std::io::Error),

    /// Invalid or malformed `Cargo.toml` file
    NotToml,

    /// Required dependencies (`hybrid-derive`, `hybrid-contract`) are missing
    MissingDependencies,

    /// Required binary targets (`runtime`, `deploy`) are not properly configured
    MissingBinaries,

    /// Required Cargo features (`default`, `deploy`, `interface-only`) are missing
    MissingFeatures,

    /// Dependency path is invalid or cannot be resolved
    WrongPath,

    /// Circular dependency detected in contract dependency graph
    CyclicDependency,
}

/// Contract identification information extracted from source code and metadata.
///
/// Contains both the Cargo package name and the actual contract identifier
/// found in the Rust source code via the `#[contract]` attribute.
#[derive(Debug, Clone, PartialEq)]
pub struct ContractName {
    /// Package name from `Cargo.toml` (e.g., "my-token-contract")
    pub package: String,

    /// Contract identifier from `#[contract]` impl block (e.g., "MyToken")
    pub ident: String,
}

/// Represents a compiled smart contract with its essential metadata.
///
/// This is the core representation of a contract after successful compilation,
/// containing the file system location and naming information.
#[derive(Debug, Clone, PartialEq)]
pub struct Contract {
    /// Absolute path to the contract's root directory (containing Cargo.toml)
    pub path: PathBuf,

    /// Contract identification information (package name and contract identifier)
    pub name: ContractName,
}

/// Represents a contract along with its resolved dependencies.
///
/// This structure is used during the contract discovery and validation phase,
/// before compilation begins. It includes dependency information that helps
/// validate the contract dependency graph and detect circular dependencies.
#[derive(Debug, Clone)]
pub struct ContractWithDeps {
    /// Absolute path to the contract's root directory (containing Cargo.toml)
    pub path: PathBuf,

    /// Contract identification information (package name and contract identifier)
    pub name: ContractName,

    /// List of contract dependencies with "interface-only" feature enabled
    pub deps: Vec<Contract>,
}

// Implement Display for Contract
impl fmt::Display for Contract {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name.ident)
    }
}

// Implement Display for ContractWithDeps
impl fmt::Display for ContractWithDeps {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.deps.is_empty() {
            write!(f, "{}", self.name.ident)
        } else {
            write!(f, "{} with deps: [", self.name.ident)?;
            for (i, dep) in self.deps.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", dep.name.ident)?;
            }
            write!(f, "]")
        }
    }
}

impl From<ContractWithDeps> for Contract {
    fn from(value: ContractWithDeps) -> Contract {
        Contract {
            name: value.name,
            path: value.path,
        }
    }
}

impl From<std::io::Error> for ContractError {
    fn from(value: std::io::Error) -> Self {
        ContractError::IoError(value)
    }
}

impl TryFrom<&PathBuf> for ContractWithDeps {
    type Error = ContractError;

    /// Attempts to parse a `Cargo.toml` file and construct a valid contract representation.
    ///
    /// This function performs comprehensive validation to ensure the project meets
    /// all requirements for a Hybrid Framework smart contract:
    ///
    /// 1. **Package Validation**: Ensures valid package name exists
    /// 2. **Feature Validation**: Checks for required features (`default`, `deploy`, `interface-only`)
    /// 3. **Binary Validation**: Validates required binary targets (`runtime`, `deploy`)
    /// 4. **Dependency Validation**: Ensures required dependencies are present
    /// 5. **Contract Dependencies**: Resolves other contracts used as dependencies
    ///
    /// # Arguments
    ///
    /// * `cargo_toml_path` - Path to the `Cargo.toml` file to parse
    ///
    /// # Returns
    ///
    /// Returns a `ContractWithDeps` instance on success, or a specific `ContractError`
    /// describing what validation failed.
    ///
    /// # Errors
    ///
    /// - `ContractError::IoError`: File read or path resolution failed
    /// - `ContractError::NotToml`: Invalid TOML format or missing package info
    /// - `ContractError::MissingFeatures`: Required Cargo features not defined
    /// - `ContractError::MissingBinaries`: Required binary targets not configured
    /// - `ContractError::MissingDependencies`: Required dependencies not found
    /// - `ContractError::WrongPath`: Contract dependency path invalid
    fn try_from(cargo_toml_path: &PathBuf) -> Result<Self, Self::Error> {
        let parent_dir = cargo_toml_path.parent().ok_or(ContractError::NotToml)?;
        let content = fs::read_to_string(cargo_toml_path)?;
        let cargo_toml = content
            .parse::<Value>()
            .map_err(|_| ContractError::NotToml)?;

        // Get package name
        let name = cargo_toml
            .get("package")
            .and_then(|f| f.get("name"))
            .ok_or(ContractError::NotToml)?
            .as_str()
            .ok_or(ContractError::NotToml)?
            .to_string();

        // Check for required features
        let has_features = match &cargo_toml.get("features") {
            Some(Value::Table(feat)) => {
                feat.contains_key("default")
                    && feat.contains_key("deploy")
                    && feat.contains_key("interface-only")
            }
            _ => false,
        };

        if !has_features {
            return Err(ContractError::MissingFeatures);
        }

        // Check for required binaries
        let has_required_bins = match &cargo_toml.get("bin") {
            Some(Value::Array(bins)) => {
                let mut has_runtime = false;
                let mut has_deploy = false;

                for bin in bins {
                    if let Value::Table(bin_table) = bin {
                        if let Some(Value::String(name)) = bin_table.get("name") {
                            if name == "runtime"
                                && bin_table.get("path").and_then(|p| p.as_str())
                                    == Some("src/lib.rs")
                            {
                                has_runtime = true;
                            } else if name == "deploy"
                                && bin_table.get("path").and_then(|p| p.as_str())
                                    == Some("src/lib.rs")
                                && bin_table
                                    .get("required-features")
                                    .map(|f| match f {
                                        Value::String(s) => s == "deploy",
                                        Value::Array(arr) => {
                                            arr.contains(&Value::String("deploy".to_string()))
                                        }
                                        _ => false,
                                    })
                                    .unwrap_or(false)
                            {
                                has_deploy = true;
                            }
                        }
                    }
                }

                has_runtime && has_deploy
            }
            _ => false,
        };

        if !has_required_bins {
            return Err(ContractError::MissingBinaries);
        }

        // Get package dependencies
        let mut contract_deps = Vec::new();
        if let Some(Value::Table(deps)) = cargo_toml.get("dependencies") {
            // Ensure required dependencies
            if !(deps.contains_key("hybrid-derive") && deps.contains_key("hybrid-contract")) {
                return Err(ContractError::MissingDependencies);
            }

            for (name, dep) in deps {
                if let Value::Table(dep_table) = dep {
                    // Ensure "interface-only" feature
                    let has_interface_only = match dep_table.get("features") {
                        Some(Value::Array(features)) => {
                            features.contains(&Value::String("interface-only".to_string()))
                        }
                        _ => false,
                    };

                    if !has_interface_only {
                        continue;
                    }

                    // Ensure local path
                    if let Some(Value::String(rel_path)) = dep_table.get("path") {
                        let path = parent_dir
                            .join(rel_path)
                            .canonicalize()
                            .map_err(|_| ContractError::WrongPath)?;
                        contract_deps.push(Contract {
                            name: ContractName {
                                ident: String::new(),
                                package: name.to_owned(),
                            },
                            path,
                        });
                    }
                }
            }
        }

        let contract = Self {
            name: ContractName {
                ident: String::new(),
                package: name,
            },
            deps: contract_deps,
            path: parent_dir.to_owned(),
        };

        Ok(contract)
    }
}

impl Contract {
    /// Converts the contract's path to a string representation.
    ///
    /// # Returns
    ///
    /// Returns a string slice of the path on success, or an error if the path
    /// contains invalid UTF-8 characters.
    ///
    /// # Errors
    ///
    /// Returns an error if the path cannot be converted to a valid UTF-8 string,
    /// which can happen with certain file system encodings.
    pub fn path_str(&self) -> Result<&str, anyhow::Error> {
        self.path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Failed to convert path to string {:?}", self.path))
    }

    /// Compiles the contract to RISC-V bytecode suitable for deployment on EVM-compatible blockchains.
    ///
    /// This is the main compilation orchestrator that performs a two-stage compilation:
    /// 1. **Runtime Compilation**: Generates bytecode for normal contract execution
    /// 2. **Deploy Compilation**: Generates bytecode for contract deployment/constructor
    ///
    /// The final bytecode includes a `0xff` prefix byte that signals to the EVM that
    /// this is a RISC-V contract rather than native EVM bytecode.
    ///
    /// # Returns
    ///
    /// Returns the complete deployment bytecode as a byte vector on success.
    /// The bytecode format is: `[0xff] + [deploy_bytecode]`
    ///
    /// # Errors
    ///
    /// Returns an error if either compilation stage fails, including:
    /// - Rust compilation errors (syntax, type errors, etc.)
    /// - Missing toolchain or target
    /// - File system errors during binary generation
    ///
    /// # Target Architecture
    ///
    /// Both compilation stages target `riscv64imac-unknown-none-elf` with:
    /// - No standard library support (`no_std`)
    /// - Core and alloc only (`build-std=core,alloc`)
    /// - Release optimization (`-r`)
    /// - Specific nightly toolchain (`nightly-2025-01-07`)
    pub fn compile_r55(&self) -> Result<Vec<u8>, anyhow::Error> {
        // First compile runtime
        self.compile_runtime()?;

        // Then compile deployment code
        let bytecode = self.compile_deploy()?;
        let mut prefixed_bytecode = vec![0xff]; // Add the 0xff prefix
        prefixed_bytecode.extend_from_slice(&bytecode);

        Ok(prefixed_bytecode)
    }

    /// Compiles the runtime binary for normal contract execution.
    ///
    /// The runtime binary handles all regular contract function calls after deployment.
    /// It's compiled without the "deploy" feature, so deployment-specific code is excluded.
    ///
    /// # Compilation Command
    ///
    /// Executes the equivalent of:
    /// ```bash
    /// cargo +nightly-2025-01-07 build -r --lib -Z build-std=core,alloc \
    ///     --target riscv64imac-unknown-none-elf --bin runtime
    /// ```
    ///
    /// # Returns
    ///
    /// Returns the compiled runtime bytecode as a byte vector.
    ///
    /// # Errors
    ///
    /// Returns an error if compilation fails or the generated binary cannot be read.
    fn compile_runtime(&self) -> Result<Vec<u8>, anyhow::Error> {
        debug!("Compiling runtime: {}", self.name.package);

        let path = self.path_str()?;
        let status = Command::new("cargo")
            .arg("+nightly-2025-01-07")
            .arg("build")
            .arg("-r")
            .arg("--lib")
            .arg("-Z")
            .arg("build-std=core,alloc")
            .arg("--target")
            .arg("riscv64imac-unknown-none-elf")
            .arg("--bin")
            .arg("runtime")
            .current_dir(path)
            .status()
            .expect("Failed to execute cargo command");

        if !status.success() {
            error!("Cargo command failed with status: {}", status);
            std::process::exit(1);
        } else {
            info!("Cargo command completed successfully");
        }

        let path = format!(
            "{}/target/riscv64imac-unknown-none-elf/release/runtime",
            path
        );
        let mut file = match fs::File::open(path) {
            Ok(file) => file,
            Err(e) => {
                anyhow::bail!("Failed to open file: {}", e);
            }
        };

        // Read the file contents into a vector.
        let mut bytecode = Vec::new();
        if let Err(e) = file.read_to_end(&mut bytecode) {
            anyhow::bail!("Failed to read file: {}", e);
        }

        Ok(bytecode)
    }

    /// Compiles the deployment binary for contract initialization.
    ///
    /// The deployment binary handles contract construction and initialization logic.
    /// It's compiled with the "deploy" feature enabled, which may include constructor
    /// code and deployment-specific functionality.
    ///
    /// # Prerequisites
    ///
    /// This function requires that `compile_runtime()` has been called first to ensure
    /// all necessary compilation artifacts are available.
    ///
    /// # Compilation Command
    ///
    /// Executes the equivalent of:
    /// ```bash
    /// cargo +nightly-2025-01-07 build -r --lib -Z build-std=core,alloc \
    ///     --target riscv64imac-unknown-none-elf --bin deploy --features deploy
    /// ```
    ///
    /// # Returns
    ///
    /// Returns the compiled deployment bytecode as a byte vector.
    ///
    /// # Errors
    ///
    /// Returns an error if compilation fails or the generated binary cannot be read.
    fn compile_deploy(&self) -> Result<Vec<u8>, anyhow::Error> {
        debug!("Compiling deploy: {}", self.name.package);

        let path = self.path_str()?;
        let status = Command::new("cargo")
            .arg("+nightly-2025-01-07")
            .arg("build")
            .arg("-r")
            .arg("--lib")
            .arg("-Z")
            .arg("build-std=core,alloc")
            .arg("--target")
            .arg("riscv64imac-unknown-none-elf")
            .arg("--bin")
            .arg("deploy")
            .arg("--features")
            .arg("deploy")
            .current_dir(path)
            .status()
            .expect("Failed to execute cargo command");

        if !status.success() {
            error!("Cargo command failed with status: {}", status);
            std::process::exit(1);
        } else {
            info!("Cargo command completed successfully");
        }

        let path = format!(
            "{}/target/riscv64imac-unknown-none-elf/release/deploy",
            path
        );
        let mut file = match fs::File::open(path) {
            Ok(file) => file,
            Err(e) => {
                anyhow::bail!("Failed to open file: {}", e);
            }
        };

        // Read the file contents into a vector.
        let mut bytecode = Vec::new();
        if let Err(e) = file.read_to_end(&mut bytecode) {
            anyhow::bail!("Failed to read file: {}", e);
        }

        Ok(bytecode)
    }
}
