//! cargo-hybrid cli utils
use alloy::{
    network::{ReceiptResponse, TransactionBuilder},
    primitives::{Address, Bytes, U32},
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
};

/// Initialize the logger with a nice formatted output
pub fn init_logger() {
    use tracing_subscriber::{fmt, EnvFilter};

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    fmt::fmt().with_env_filter(filter).with_target(false).init();
}

/// The function is use to deploy a RISC-V smart contract to a hybrid node
pub async fn deploy_riscv_bytecode(
    rpc_url: &str,
    private_key: &str,
    bytecode: Vec<u8>,
    encoded_args: Option<Vec<u8>>,
) -> Result<Address, anyhow::Error> {
    let signer: PrivateKeySigner = private_key.parse()?;
    let provider = ProviderBuilder::new()
        .wallet(signer)
        .connect_http(rpc_url.parse()?);

    let init_code = prep_riscv_initcode(bytecode, encoded_args);
    let tx = TransactionRequest::default().with_deploy_code(init_code);

    let receipt = provider
        .send_transaction(tx)
        .await
        .unwrap()
        .get_receipt()
        .await?;

    Ok(receipt
        .contract_address()
        .expect("Failed to get contract address"))
}

fn prep_riscv_initcode(bytecode: Vec<u8>, encoded_args: Option<Vec<u8>>) -> Vec<u8> {
    let init_code = if Some(&0xff) == bytecode.first() {
        // Craft R55 initcode: [0xFF][codesize][bytecode][constructor_args]
        let codesize = U32::from(bytecode.len());

        let mut init_code = Vec::new();
        init_code.push(0xff);
        init_code.extend_from_slice(&Bytes::from(codesize.to_be_bytes_vec()));
        init_code.extend_from_slice(&bytecode);

        if let Some(args) = encoded_args {
            init_code.extend_from_slice(&args);
        }

        init_code
    } else {
        // do not modify bytecode for EVM contracts
        bytecode
    };

    init_code
}
