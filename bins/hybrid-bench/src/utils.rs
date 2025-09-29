use revm::primitives::hex;
use sha3::{Digest, Keccak256};
use std::{fs::File, io::Read};

pub fn generate_calldata(function: &str, n: u64) -> String {
    let function_signature = format!("{function}(uint256)");
    let hash = Keccak256::digest(function_signature.as_bytes());
    let function_selector = &hash[..4];

    // Encode argument n (uint256, padded to 32 bytes)
    let mut encoded_n = [0u8; 32];
    encoded_n[24..].copy_from_slice(&n.to_be_bytes());

    // Combine the function selector and the encoded argument
    let calldata: Vec<u8> = function_selector
        .iter()
        .chain(encoded_n.iter())
        .copied()
        .collect();

    hex::encode(calldata)
}

pub fn load_contract_bytecode(bench_name: &str) -> String {
    let path = format!(
        "{}/src/assets/{bench_name}.bin-runtime",
        env!("CARGO_MANIFEST_DIR"),
    );

    println!("Loading bytecode from file {path}");

    let mut file = File::open(path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    contents
}
