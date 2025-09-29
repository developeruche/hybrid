use crate::{
    revm_bench::run_with_revm,
    utils::{generate_calldata, load_contract_bytecode},
};

pub(crate) mod hybrid_vm_bench;
pub(crate) mod revm_bench;
pub(crate) mod utils;

pub const NO_OF_ITERATIONS_ONE: u64 = 60;
pub const NO_OF_ITERATIONS_TWO: u64 = 120;
pub const NO_OF_ITERATIONS_THREE: u64 = 500;

pub const RUNS: u64 = 10000;
pub const RUNS_SLOW: u64 = 200;

fn main() {
    let contracts = [
        "BubbleSort",
        "ERC20ApprovalTransfer",
        "ERC20Mint",
        "ERC20Transfer",
        "Factorial",
        "FactorialRecursive",
        "Fibonacci",
        "FibonacciRecursive",
        "ManyHashes",
        "MstoreBench",
        "Push",
        "SstoreBench_no_opt",
    ];

    for contract in contracts {
        let runtime_code = load_contract_bytecode(contract);
        let calldata = generate_calldata(contract, NO_OF_ITERATIONS_ONE);

        run_with_revm(&runtime_code, RUNS, &calldata);
    }
}
