use hybrid_vm::{
    evm::HybridEvm,
    revm::{
        db::{BenchmarkDB, BENCH_TARGET},
        handler::EthPrecompiles,
        inspector::NoOpInspector,
        primitives::{address, hex},
        state::Bytecode,
        Context, ExecuteEvm, MainBuilder, MainContext,
    },
};
use revm::primitives::TxKind;
use std::hint::black_box;

pub fn run_with_hybrid_vm_evm_mode(contract_code: &str, runs: u64, calldata: &str) {
    let rich_acc_address = address!("1000000000000000000000000000000000000000");
    let bytes = hex::decode(contract_code).unwrap();
    let raw_bytecode = Bytecode::new_raw(bytes.clone().into());

    let evm = Context::mainnet()
        .with_db(BenchmarkDB::new_bytecode(raw_bytecode))
        .modify_tx_chained(|tx| {
            tx.caller = rich_acc_address;
            tx.data = hex::decode(calldata).unwrap().into();
            tx.kind = TxKind::Call(BENCH_TARGET);
        })
        .build_mainnet_with_inspector(NoOpInspector {})
        .with_precompiles(EthPrecompiles::default());

    let mut h_evm = HybridEvm(evm);

    for _ in 0..runs {
        let result = black_box(h_evm.replay()).unwrap();
        assert!(result.result.is_success(), "{:?}", result.result);
    }
}
