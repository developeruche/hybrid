use revm::{
    context::{Evm, TxEnv},
    database::BenchmarkDB,
    handler::{instructions::EthInstructions, EthPrecompiles},
    primitives::{address, hex},
    state::Bytecode,
    Context, ExecuteEvm, MainContext,
};
use std::hint::black_box;

pub fn run_with_revm(contract_code: &str, runs: u64, calldata: &str) {
    let rich_acc_address = address!("1000000000000000000000000000000000000000");
    let bytes = hex::decode(contract_code).unwrap();
    let raw_bytecode = Bytecode::new_raw(bytes.clone().into());

    let context = Context::mainnet()
        .modify_tx_chained(|tx| {
            tx.caller = rich_acc_address;
            tx.data = hex::decode(calldata).unwrap().into();
        })
        .with_db(BenchmarkDB::new_bytecode(raw_bytecode));

    let mut evm = Evm::new(
        context,
        EthInstructions::new_mainnet(),
        EthPrecompiles::default(),
    );
    let tx = TxEnv {
        caller: rich_acc_address,
        data: hex::decode(calldata).unwrap().into(),
        ..Default::default()
    };

    for _ in 0..runs {
        let result = black_box(evm.transact(tx.clone())).unwrap();
        assert!(result.result.is_success(), "{:?}", result.result);
    }
    //todo: remove this extra
    let result = black_box(evm.transact(tx)).unwrap();
    assert!(result.result.is_success(), "{:?}", result.result);

    println!("output: \t\t{}", result.result.into_output().unwrap());
}
