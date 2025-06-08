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
                "balance": "0x4a47e3c12448f4ad000000"
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
