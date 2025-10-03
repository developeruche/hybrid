use hybrid_bench::{
    hybrid_vm_bench::run_with_hybrid_vm,
    revm_bench::run_with_revm,
    utils::{generate_calldata, load_contract_bytecode, load_hybrid_contract_bytecode},
    NO_OF_ITERATIONS_ONE, RUNS,
};

fn main() {
    let contracts = [
        "BubbleSort",
        "ERC20ApprovalTransfer",
        "ERC20Mint",
        "ERC20Transfer",
        "Factorial",
        "Fibonacci",
        "ManyHashes",
        "MstoreBench",
        "Push",
        "SstoreBench_no_opt",
    ];
    
    let hybrid_contracts = [
        "ERC20ApprovalTransfer",
        "ERC20Mint",
        "ERC20Transfer",
        "Factorial",
        "Fibonacci",
        "ManyHashes"
    ];

    for contract in contracts {
        let runtime_code = load_contract_bytecode(contract);
        let calldata = generate_calldata("Benchmark", NO_OF_ITERATIONS_ONE);

        println!("Running this contract: {}", contract);
        run_with_revm(&runtime_code, RUNS, &calldata);
        run_with_hybrid_vm(&runtime_code, RUNS, &calldata);
    }
    
    for contract in hybrid_contracts {
        let hybrid_runtime_code = load_hybrid_contract_bytecode(contract);
        let calldata = generate_calldata("Benchmark", NO_OF_ITERATIONS_ONE);
        run_with_hybrid_vm(&hybrid_runtime_code, RUNS, &calldata);
    }
}
