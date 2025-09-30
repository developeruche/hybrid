use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use hybrid_bench::{
    hybrid_vm_bench::run_with_hybrid_vm_evm_mode,
    revm_bench::run_with_revm,
    utils::{generate_calldata, load_contract_bytecode},
    NO_OF_ITERATIONS_ONE, NO_OF_ITERATIONS_TWO,
};

/// Contract categories for determining appropriate benchmark iterations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContractComplexity {
    /// Fast contracts - simple operations that execute quickly
    Fast,
    /// Medium complexity - moderate computation/storage operations
    Medium,
    /// Slow contracts - recursive algorithms or intensive operations
    Slow,
}

/// Configuration for each contract benchmark
#[derive(Debug, Clone, Copy)]
struct ContractBenchConfig {
    name: &'static str,
    complexity: ContractComplexity,
}

impl ContractBenchConfig {
    /// Create a new contract benchmark configuration
    const fn new(name: &'static str, complexity: ContractComplexity) -> Self {
        Self { name, complexity }
    }

    /// Determine the number of runs based on contract complexity
    /// This controls how many times the VM executes the contract per benchmark iteration
    const fn runs(&self) -> u64 {
        match self.complexity {
            // Fast contracts use 10 runs for quick benchmarking
            ContractComplexity::Fast => 10,
            // Medium complexity contracts use 10 runs
            ContractComplexity::Medium => 10,
            // Slow contracts use 5 runs to keep benchmark time reasonable
            ContractComplexity::Slow => 5,
        }
    }
}

/// Comprehensive contract benchmark configurations
///
/// Each contract is categorized by its computational complexity to ensure
/// benchmarks complete in reasonable time while maintaining statistical significance
const CONTRACTS: &[ContractBenchConfig] = &[
    // Slow contracts - Deep recursion or intensive computation
    ContractBenchConfig::new("BubbleSort", ContractComplexity::Slow),
    // ContractBenchConfig::new("FactorialRecursive", ContractComplexity::Slow),
    // ContractBenchConfig::new("FibonacciRecursive", ContractComplexity::Slow),
    ContractBenchConfig::new("ManyHashes", ContractComplexity::Slow),
    // Medium complexity contracts - Standard smart contract operations
    ContractBenchConfig::new("ERC20ApprovalTransfer", ContractComplexity::Medium),
    ContractBenchConfig::new("ERC20Mint", ContractComplexity::Medium),
    ContractBenchConfig::new("MstoreBench", ContractComplexity::Medium),
    ContractBenchConfig::new("SstoreBench_no_opt", ContractComplexity::Medium),
    // Fast contracts - Simple operations
    ContractBenchConfig::new("ERC20Transfer", ContractComplexity::Fast),
    ContractBenchConfig::new("Factorial", ContractComplexity::Fast),
    ContractBenchConfig::new("Fibonacci", ContractComplexity::Fast),
    ContractBenchConfig::new("Push", ContractComplexity::Fast),
];

/// Benchmark group for REVM execution
///
/// This function benchmarks the reference REVM implementation across all contracts.
/// Results are organized under the "revm" group for easy identification.
fn bench_revm(c: &mut Criterion) {
    let mut group = c.benchmark_group("revm");

    for config in CONTRACTS {
        // Load contract bytecode from assets
        let runtime_code = load_contract_bytecode(config.name);

        // Generate calldata with NO_OF_ITERATIONS_TWO (120) iterations
        let calldata = if config.name == "FibonacciRecursive" {
            generate_calldata("Benchmark", NO_OF_ITERATIONS_ONE)
        } else {
            generate_calldata("Benchmark", NO_OF_ITERATIONS_TWO)
        };

        // Determine run count based on complexity
        let runs = config.runs();

        group.bench_with_input(
            BenchmarkId::new("revm", config.name),
            &(runtime_code.as_str(), calldata.as_str(), runs),
            |b, &(code, data, runs)| {
                b.iter(|| run_with_revm(black_box(code), black_box(runs), black_box(data)));
            },
        );
    }

    group.finish();
}

/// Benchmark group for Hybrid VM EVM mode execution
///
/// This function benchmarks the Hybrid VM running in EVM-compatible mode.
/// Results are organized under the "hybrid_vm" group for comparison.
fn bench_hybrid_vm(c: &mut Criterion) {
    let mut group = c.benchmark_group("hybrid_vm");

    for config in CONTRACTS {
        // Load contract bytecode from assets
        let runtime_code = load_contract_bytecode(config.name);

        // Generate calldata with NO_OF_ITERATIONS_TWO (120) iterations
        let calldata = if config.name == "FibonacciRecursive" {
            generate_calldata("Benchmark", NO_OF_ITERATIONS_ONE)
        } else {
            generate_calldata("Benchmark", NO_OF_ITERATIONS_TWO)
        };

        // Determine run count based on complexity
        let runs = config.runs();

        group.bench_with_input(
            BenchmarkId::new("hybrid", config.name),
            &(runtime_code.as_str(), calldata.as_str(), runs),
            |b, &(code, data, runs)| {
                b.iter(|| {
                    run_with_hybrid_vm_evm_mode(black_box(code), black_box(runs), black_box(data))
                });
            },
        );
    }

    group.finish();
}

/// Direct comparison benchmark between REVM and Hybrid VM
///
/// This function runs both VM implementations for each contract within the same
/// benchmark group, making it easy to compare performance side-by-side.
///
/// Benchmark names follow the pattern:
/// - `revm_<ContractName>` for REVM implementation
/// - `hybrid_<ContractName>` for Hybrid VM implementation
fn bench_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("comparison");

    for config in CONTRACTS {
        // Load contract bytecode from assets
        let runtime_code = load_contract_bytecode(config.name);

        // Generate calldata with NO_OF_ITERATIONS_TWO (120) iterations
        let calldata = generate_calldata("Benchmark", NO_OF_ITERATIONS_TWO);

        // Determine run count based on complexity
        let runs = config.runs();

        // Benchmark REVM implementation
        group.bench_with_input(
            BenchmarkId::new(format!("revm_{}", config.name), config.name),
            &(runtime_code.as_str(), calldata.as_str(), runs),
            |b, &(code, data, runs)| {
                b.iter(|| run_with_revm(black_box(code), black_box(runs), black_box(data)));
            },
        );

        // Benchmark Hybrid VM implementation
        group.bench_with_input(
            BenchmarkId::new(format!("hybrid_{}", config.name), config.name),
            &(runtime_code.as_str(), calldata.as_str(), runs),
            |b, &(code, data, runs)| {
                b.iter(|| {
                    run_with_hybrid_vm_evm_mode(black_box(code), black_box(runs), black_box(data))
                });
            },
        );
    }

    group.finish();
}

// Configure Criterion benchmark group
criterion_group!(
    name = benches;
    config = Criterion::default()
        // Reduce sample size to 10 for quick benchmarking
        .sample_size(10)
        // Spend only 3 seconds measuring each benchmark for faster iteration
        .measurement_time(std::time::Duration::from_secs(3))
        // Reduce warmup time to 1 second
        .warm_up_time(std::time::Duration::from_secs(1))
        // Confidence level for statistical analysis (95%)
        .confidence_level(0.95)
        // Noise threshold - 5% change is considered significant
        .noise_threshold(0.05);
    targets = bench_revm, bench_hybrid_vm, bench_comparison
);

criterion_main!(benches);
