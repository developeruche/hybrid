//! Module holds all contants and static variables used in this node
use alloy_genesis::Genesis;
use reth::chainspec::{Chain, ChainSpec};

pub const HYBRID_MAX_CODE_SIZE: usize = 2000800;

/// Returns the chain specs for this node
pub fn obtain_specs() -> ChainSpec {
    let custom_genesis = r#"
    {
        "nonce": "0x42",
        "timestamp": "0x0",
        "extraData": "0x5343",
        "gasLimit": "0xf3880000000000",
        "difficulty": "0x400000000",
        "mixHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
        "coinbase": "0x0000000000000000000000000000000000000000",
        "alloc": {
            "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266": {
                "balance": "0x21E19E0C9BAB2400000"
            },
            "0x70997970C51812dc3A010C7d01b50e0d17dc79C8": {
                "balance": "0x21E19E0C9BAB2400000"
            },
            "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC": {
                "balance": "0x21E19E0C9BAB2400000"
            },
            "0x90F79bf6EB2c4f870365E785982E1f101E93b906": {
                "balance": "0x21E19E0C9BAB2400000"
            },
            "0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65": {
                "balance": "0x21E19E0C9BAB2400000"
            },
            "0x9965507D1a55bcC2695C58ba16FB37d819B0A4dc": {
                "balance": "0x21E19E0C9BAB2400000"
            },
            "0x976EA74026E726554dB657fA54763abd0C3a0aa9": {
                "balance": "0x21E19E0C9BAB2400000"
            },
            "0x14dC79964da2C08b23698B3D3cc7Ca32193d9955": {
                "balance": "0x21E19E0C9BAB2400000"
            },
            "0x23618e81E3f5cdF7f54C3d65f7FBc0aBf5B21E8f": {
                "balance": "0x21E19E0C9BAB2400000"
            },
            "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720": {
                "balance": "0x21E19E0C9BAB2400000"
            }
        },
        "number": "0x0",
        "gasUsed": "0x0",
        "parentHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
        "config": {
            "ethash": {},
            "chainId": 33500,
            "homesteadBlock": 0,
            "eip150Block": 0,
            "eip155Block": 0,
            "eip158Block": 0,
            "byzantiumBlock": 0,
            "constantinopleBlock": 0,
            "petersburgBlock": 0,
            "istanbulBlock": 0,
            "berlinBlock": 0,
            "londonBlock": 0,
            "terminalTotalDifficulty": 0,
            "terminalTotalDifficultyPassed": true,
            "shanghaiTime": 0
        }
    }
    "#;

    let genesis: Genesis = serde_json::from_str(custom_genesis).unwrap();

    let spec = ChainSpec::builder()
        .chain(Chain::mainnet())
        .genesis(genesis)
        .london_activated()
        .paris_activated()
        .shanghai_activated()
        .cancun_activated()
        .prague_activated()
        .build();

    spec
}
