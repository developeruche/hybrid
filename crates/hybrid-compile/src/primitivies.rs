//! Primitives for the compile module
use std::{fmt, fs, io::Read, path::PathBuf, process::Command};
use toml::Value;
use tracing::{debug, error, info};

#[derive(Debug)]
pub enum ContractError {
    IoError(std::io::Error),
    NotToml,
    MissingDependencies,
    MissingBinaries,
    MissingFeatures,
    WrongPath,
    CyclicDependency,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContractName {
    pub package: String,
    pub ident: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Contract {
    pub path: PathBuf,
    pub name: ContractName,
}

#[derive(Debug, Clone)]
pub struct ContractWithDeps {
    pub path: PathBuf,
    pub name: ContractName,
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
            if !(deps.contains_key("contract-derive") && deps.contains_key("eth-riscv-runtime")) {
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
    pub fn path_str(&self) -> Result<&str, anyhow::Error> {
        self.path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Failed to convert path to string {:?}", self.path))
    }

    pub fn compile_r55(&self) -> Result<Vec<u8>, anyhow::Error> {
        // First compile runtime
        self.compile_runtime()?;

        // Then compile deployment code
        let bytecode = self.compile_deploy()?;
        let mut prefixed_bytecode = vec![0xff]; // Add the 0xff prefix
        prefixed_bytecode.extend_from_slice(&bytecode);

        Ok(prefixed_bytecode)
    }

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

    // Requires previous runtime compilation
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
