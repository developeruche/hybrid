use revm::{
    context::Evm,
    database::{BenchmarkDB, BENCH_TARGET},
    handler::{instructions::EthInstructions, EthPrecompiles},
    primitives::{address, hex, TxKind},
    state::Bytecode,
    Context, ExecuteEvm, MainContext,
};
use std::hint::black_box;

pub fn run_with_revm(contract_code: &str, runs: u64, calldata: &str) {
    let rich_acc_address = address!("1000000000000000000000000000000000000000");
    let bytes = hex::decode(contract_code).unwrap();
    let raw_bytecode = Bytecode::new_raw(bytes.clone().into());

    let context = Context::mainnet()
        .with_db(BenchmarkDB::new_bytecode(raw_bytecode))
        .modify_tx_chained(|tx| {
            tx.caller = rich_acc_address;
            tx.data = hex::decode(calldata).unwrap().into();
            tx.kind = TxKind::Call(BENCH_TARGET);
        });

    let mut evm = Evm::new(
        context,
        EthInstructions::new_mainnet(),
        EthPrecompiles::default(),
    );

    for _ in 0..runs {
        let result = black_box(evm.replay()).unwrap();
        assert!(result.result.is_success(), "{:?}", result.result);
    }
}
