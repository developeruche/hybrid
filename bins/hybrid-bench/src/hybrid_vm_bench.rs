use hybrid_vm::{
    evm::HybridEvm,
    revm::{
        context::TxEnv,
        db::BenchmarkDB,
        handler::EthPrecompiles,
        inspector::NoOpInspector,
        primitives::{address, hex},
        state::Bytecode,
        Context, ExecuteEvm, MainBuilder, MainContext,
    },
};
use std::hint::black_box;

pub fn run_with_hybrid_vm(contract_code: &str, runs: u64, calldata: &str) {
    let rich_acc_address = address!("1000000000000000000000000000000000000000");
    let bytes = hex::decode(contract_code).unwrap();
    let raw_bytecode = Bytecode::new_raw(bytes.clone().into());

    let evm = Context::mainnet()
        .modify_tx_chained(|tx| {
            tx.caller = rich_acc_address;
            tx.data = hex::decode(calldata).unwrap().into();
        })
        .with_db(BenchmarkDB::new_bytecode(raw_bytecode))
        .build_mainnet_with_inspector(NoOpInspector {})
        .with_precompiles(EthPrecompiles::default());

    let mut h_evm = HybridEvm(evm);

    let tx = TxEnv {
        caller: rich_acc_address,
        data: hex::decode(calldata).unwrap().into(),
        ..Default::default()
    };

    for _ in 0..runs {
        let result = black_box(h_evm.transact(tx.clone())).unwrap();
        assert!(result.result.is_success(), "{:?}", result.result);
    }
}
