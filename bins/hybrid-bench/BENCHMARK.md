# Hybrid VM Benchmark Suite

Benchmark suite comparing REVM and Hybrid VM EVM mode performance using Criterion.

## Overview

This benchmark suite provides comprehensive performance analysis for two virtual machine implementations:
- **REVM**: The reference EVM implementation
- **Hybrid VM (EVM Mode)**: The hybrid virtual machine running in EVM-compatible mode

## Benchmarked Contracts

The suite benchmarks the following smart contracts, categorized by computational complexity:

### Fast Contracts (1000 iterations)
- `Push` - Basic stack push operations
- `ERC20Transfer` - Standard ERC20 transfer
- `Factorial` - Iterative factorial calculation
- `Fibonacci` - Iterative Fibonacci calculation

### Medium Complexity (500 iterations)
- `ERC20ApprovalTransfer` - ERC20 approval and transfer
- `ERC20Mint` - ERC20 token minting
- `MstoreBench` - Memory storage benchmarks
- `SstoreBench_no_opt` - Storage operations without optimization

### Slow Contracts (100 iterations)
- `BubbleSort` - Sorting algorithm benchmark
- `FactorialRecursive` - Recursive factorial (deep call stack)
- `FibonacciRecursive` - Recursive Fibonacci (deep call stack)
- `ManyHashes` - Cryptographic hash operations

## Running Benchmarks

### Run All Benchmarks
```bash
cargo bench --bench vm_comparison
```

### Run Specific Benchmark Groups

**REVM only:**
```bash
cargo bench --bench vm_comparison revm
```

**Hybrid VM only:**
```bash
cargo bench --bench vm_comparison hybrid_vm
```

**Direct comparison:**
```bash
cargo bench --bench vm_comparison comparison
```

### Run Specific Contract Benchmarks

**Single contract across both VMs:**
```bash
cargo bench --bench vm_comparison BubbleSort
```

**Specific VM and contract:**
```bash
cargo bench --bench vm_comparison "revm_BubbleSort"
cargo bench --bench vm_comparison "hybrid_BubbleSort"
```

## Benchmark Configuration

### Criterion Settings
- **Sample Size**: 10 measurements per benchmark
- **Measurement Time**: 30 seconds per benchmark
- **Warmup**: Automatic (Criterion default)

### Contract Iterations
- **NO_OF_ITERATIONS_ONE**: 120 (iterations passed to contract functions)
- **Benchmark Runs**: Varies by contract complexity (100-1000)

### Complexity-Based Run Counts
```rust
Fast contracts:    1000 runs per benchmark
Medium contracts:  500 runs per benchmark
Slow contracts:    100 runs per benchmark
```

## Output

Benchmark results are generated in multiple formats:

### Console Output
Real-time progress and summary statistics are displayed in the terminal.

### HTML Reports
Detailed interactive reports are generated in:
```
target/criterion/
├── revm/
│   ├── revm_BubbleSort/
│   └── ...
├── hybrid_vm/
│   ├── hybrid_BubbleSort/
│   └── ...
└── comparison/
    ├── revm_BubbleSort/
    ├── hybrid_BubbleSort/
    └── ...
```

Open `target/criterion/report/index.html` in a browser to view comprehensive results.

## Understanding Results

### Metrics Provided
- **Mean**: Average execution time
- **Std Dev**: Standard deviation of measurements
- **Median**: Median execution time
- **MAD**: Median Absolute Deviation

### Interpreting Results
- Lower values indicate better performance
- Compare `revm_*` vs `hybrid_*` benchmarks for the same contract
- Check for variance (std dev) to assess consistency

## Example Output

```
revm/revm/BubbleSort time:   [45.123 ms 45.456 ms 45.789 ms]
hybrid_vm/hybrid/BubbleSort time:   [43.234 ms 43.567 ms 43.901 ms]
                        change: [-5.23% -4.89% -4.55%] (p = 0.00 < 0.05)
                        Performance has improved.
```

## Development

### Adding New Contracts

1. Add the `.bin-runtime` file to `src/assets/`
2. Update the `CONTRACTS` array in `benches/vm_comparison.rs`:
```rust
ContractBenchConfig::new("NewContract", ContractComplexity::Medium),
```

### Modifying Benchmark Parameters

Edit `benches/vm_comparison.rs`:
```rust
criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(10)  // Increase for more precision
        .measurement_time(std::time::Duration::from_secs(30));  // Longer for stability
    targets = bench_revm, bench_hybrid_vm, bench_comparison
);
```

### Baseline Comparison
```bash
# Save current results as baseline
cargo bench --bench vm_comparison -- --save-baseline main

# Compare against baseline after changes
cargo bench --bench vm_comparison -- --baseline main
```

## Performance Tips

1. **Disable CPU frequency scaling** for consistent results:
   ```bash
   # Linux
   sudo cpupower frequency-set --governor performance
   ```

2. **Close background applications** to reduce noise

3. **Run on a quiet system** - avoid running during system updates or heavy I/O

4. **Use release mode** - Criterion automatically uses optimized builds

## Troubleshooting

### Long Benchmark Times
If benchmarks take too long, reduce:
- Sample size: `.sample_size(5)`
- Measurement time: `.measurement_time(Duration::from_secs(15))`
- Contract run counts in `ContractBenchConfig::runs()`

### Inconsistent Results
- Ensure system is idle
- Check for thermal throttling
- Disable turbo boost for more stable results
- Increase sample size for better statistics

## License

MIT License - See root LICENSE file

## Contributing

When contributing new benchmarks:
1. Follow the existing pattern in `vm_comparison.rs`
2. Choose appropriate complexity classification
3. Ensure contracts are deterministic
4. Document any special requirements
5. Test locally before submitting PR