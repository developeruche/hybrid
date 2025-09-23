use reth::revm::primitives::{Address, U256};

pub fn deserialize_sstore_input(data: &[u8]) -> (Address, U256, U256) {
    if data.len() < 24 {
        panic!("Data too short for SSTORE input headers");
    }

    let sa_len = u64::from_le_bytes(data[0..8].try_into().unwrap()) as usize;
    let si_len = u64::from_le_bytes(data[8..16].try_into().unwrap()) as usize;
    let sv_len = u64::from_le_bytes(data[16..24].try_into().unwrap()) as usize;

    let expected_len = sa_len + si_len + sv_len + 24;
    if data.len() != expected_len {
        panic!(
            "SSTORE input length mismatch: expected {}, got {}",
            expected_len,
            data.len()
        );
    }

    let address_bytes = &data[24..24 + sa_len];
    let index_bytes = &data[24 + sa_len..24 + sa_len + si_len];
    let value_bytes = &data[24 + sa_len + si_len..24 + sa_len + si_len + sv_len];

    let address: Address =
        bincode::serde::decode_from_slice(address_bytes, bincode::config::legacy())
            .unwrap()
            .0;
    let index: U256 = bincode::serde::decode_from_slice(index_bytes, bincode::config::legacy())
        .unwrap()
        .0;
    let value: U256 = bincode::serde::decode_from_slice(value_bytes, bincode::config::legacy())
        .unwrap()
        .0;

    (address, index, value)
}
