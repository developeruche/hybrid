use hybrid_bench::{
    hybrid_vm_bench::run_with_hybrid_vm_evm_mode,
    revm_bench::run_with_revm,
    utils::{generate_calldata, load_contract_bytecode},
    NO_OF_ITERATIONS_ONE, RUNS,
};

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
        let calldata = generate_calldata("Benchmark", NO_OF_ITERATIONS_ONE);

        // run_with_revm(&runtime_code, RUNS, &calldata);
        run_with_hybrid_vm_evm_mode(&runtime_code, RUNS, &calldata);
        break;
    }
}
