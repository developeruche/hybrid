# Hybrid VM Benchmark Suite

> Professional performance benchmarking for REVM vs Hybrid VM EVM mode comparison

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Criterion](https://img.shields.io/badge/benchmark-criterion-blue.svg)](https://github.com/bheisler/criterion.rs)

## Overview

This benchmark suite provides comprehensive, statistically-rigorous performance analysis comparing two Ethereum Virtual Machine implementations:

- **REVM**: The reference Rust EVM implementation
- **Hybrid VM (EVM Mode)**: The hybrid virtual machine running in EVM-compatible mode

## Quick Start

### Run All Benchmarks
```bash
cd bins/hybrid-bench
cargo bench --bench vm_comparison
```

### View Results
```bash
open target/criterion/report/index.html
```

### Using Make (Recommended)
```bash
make bench          # Run all benchmarks
make report         # Open HTML report
make help           # See all available commands
```

### Using Shell Script
```bash
./run_benchmarks.sh              # Run all benchmarks with optimization
./run_benchmarks.sh --fast       # Quick benchmark
./run_benchmarks.sh BubbleSort   # Benchmark specific contract
```

## Features

✅ **Comprehensive Coverage**: 12 smart contracts spanning various complexity levels  
✅ **Statistical Rigor**: Criterion.rs with confidence intervals and outlier detection  
✅ **Smart Iteration Counts**: Complexity-based run counts for optimal benchmark duration  
✅ **Professional Reports**: HTML reports with interactive charts and historical comparison  
✅ **CI/CD Ready**: Baseline comparison and regression detection  
✅ **Well-Documented**: Extensive documentation and examples  

## Benchmarked Contracts

### Fast Contracts (1000 runs)
Lightweight operations with minimal computational overhead:
- **Push** - Basic stack push operations
- **ERC20Transfer** - Standard ERC20 token transfer
- **Factorial** - Iterative factorial calculation
- **Fibonacci** - Iterative Fibonacci sequence

### Medium Complexity (500 runs)
Standard smart contract operations with moderate complexity:
- **ERC20ApprovalTransfer** - ERC20 approval and transfer flow
- **ERC20Mint** - ERC20 token minting operation
- **MstoreBench** - Memory storage benchmarks
- **SstoreBench_no_opt** - Storage operations without optimization

### Slow Contracts (100 runs)
Computationally intensive operations:
- **BubbleSort** - Sorting algorithm benchmark
- **FactorialRecursive** - Recursive factorial (deep call stack)
- **FibonacciRecursive** - Recursive Fibonacci (deep call stack)
- **ManyHashes** - Intensive cryptographic hash operations

## Architecture

### Project Structure
```
hybrid-bench/
├── benches/
│   └── vm_comparison.rs      # Criterion benchmark suite
├── src/
│   ├── lib.rs                # Public API
│   ├── main.rs               # CLI runner
│   ├── hybrid_vm_bench.rs    # Hybrid VM benchmarking
│   ├── revm_bench.rs         # REVM benchmarking
│   ├── utils.rs              # Utility functions
│   └── assets/               # Contract bytecode files
├── Cargo.toml                # Dependencies and config
├── Makefile                  # Build automation
├── run_benchmarks.sh         # Professional runner script
├── README.md                 # This file
├── BENCHMARK.md              # Detailed documentation
└── QUICKSTART.md             # Quick reference guide
```

### Benchmark Configuration

```rust
// Contract iteration count (passed to contract functions)
NO_OF_ITERATIONS_TWO: u64 = 120

// Benchmark runs per contract (based on complexity)
Fast:    1000 runs
Medium:  500 runs
Slow:    100 runs

// Criterion configuration
Sample Size:       10
Measurement Time:  30 seconds
Confidence Level:  95%
Noise Threshold:   5%
```

## Usage Examples

### Basic Benchmarking

```bash
# Run all benchmarks
cargo bench --bench vm_comparison

# Run specific VM benchmarks
cargo bench --bench vm_comparison revm
cargo bench --bench vm_comparison hybrid_vm

# Run comparison group
cargo bench --bench vm_comparison comparison
```

### Contract-Specific Benchmarks

```bash
# Single contract
cargo bench --bench vm_comparison BubbleSort

# Pattern matching
cargo bench --bench vm_comparison Fibonacci     # Both Fibonacci variants
cargo bench --bench vm_comparison ERC20         # All ERC20 contracts
```

### Named Benchmarks

```bash
# Specific VM + contract combination
cargo bench --bench vm_comparison "revm_BubbleSort"
cargo bench --bench vm_comparison "hybrid_BubbleSort"
```

### Baseline Comparison

```bash
# Save current performance as baseline
cargo bench --bench vm_comparison -- --save-baseline main

# Make code changes...

# Compare against baseline
cargo bench --bench vm_comparison -- --baseline main
```

### Make Commands

```bash
make bench              # Run all benchmarks
make bench-revm         # REVM only
make bench-hybrid       # Hybrid VM only
make bench-compare      # Side-by-side comparison
make bench-fast         # Quick benchmark (reduced samples)
make bench-slow         # Thorough benchmark (increased samples)
make bench-bubblesort   # Specific contract
make bench-erc20        # All ERC20 contracts
make baseline-save      # Save baseline
make baseline-compare   # Compare with baseline
make report             # Open HTML report
make clean              # Remove artifacts
make list               # List all contracts
make help               # Show all commands
```

## Understanding Results

### Console Output
```
revm_BubbleSort     time:   [45.123 ms 45.456 ms 45.789 ms]
                           change: [+2.1% +2.5% +2.9%] (p = 0.00 < 0.05)
                           Performance has regressed.

hybrid_BubbleSort   time:   [43.234 ms 43.567 ms 43.901 ms]
                           change: [-5.2% -4.9% -4.5%] (p = 0.00 < 0.05)
                           Performance has improved.
```

**Reading the output:**
- **First number**: Lower bound (95% confidence)
- **Second number**: Estimate (most reliable value)
- **Third number**: Upper bound (95% confidence)
- **Change**: Performance delta vs. previous run (if available)
- **P-value**: Statistical significance (< 0.05 is significant)

### HTML Reports

Open `target/criterion/report/index.html` for:
- Interactive charts
- Statistical analysis
- Historical comparisons
- Outlier detection
- Detailed timing breakdowns

## Performance Tips

### For Accurate Results

1. **Minimize System Noise**
   ```bash
   # Close unnecessary applications
   # Disable background services
   # Avoid system updates during benchmarking
   ```

2. **Stable Power Supply**
   - Use AC power on laptops (avoid battery throttling)
   - Disable CPU frequency scaling if possible

3. **Thermal Considerations**
   - Ensure adequate cooling
   - Let system cool between benchmark runs
   - Watch for thermal throttling

4. **System Configuration**
   ```bash
   # Linux: Disable CPU frequency scaling (requires root)
   sudo cpupower frequency-set --governor performance
   
   # macOS: Prevent sleep
   caffeinate -i cargo bench --bench vm_comparison
   ```

### For Faster Iteration

```bash
# Reduce sample size
make bench-fast

# Benchmark specific contracts only
cargo bench --bench vm_comparison Push Factorial

# Build without running
cargo bench --bench vm_comparison --no-run
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Benchmark

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Run benchmarks
        run: |
          cd bins/hybrid-bench
          cargo bench --bench vm_comparison -- --output-format bencher | tee output.txt
          
      - name: Store benchmark result
        uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'cargo'
          output-file-path: bins/hybrid-bench/output.txt
```

### Automated Baseline Comparison

```bash
# In CI pipeline
git checkout main
cargo bench --bench vm_comparison -- --save-baseline main

git checkout feature-branch
cargo bench --bench vm_comparison -- --baseline main
```

## Troubleshooting

### Issue: Benchmarks Take Too Long

**Solution:**
```bash
# Use fast mode
make bench-fast

# Or benchmark specific contracts
cargo bench --bench vm_comparison Push Factorial ERC20Transfer
```

### Issue: Inconsistent Results

**Causes:**
- High system load
- Thermal throttling
- Background processes
- CPU frequency scaling

**Solutions:**
- Close unnecessary applications
- Ensure adequate cooling
- Disable CPU frequency scaling
- Increase sample size: `make bench-slow`

### Issue: Out of Memory

**Solution:**
```bash
# Benchmark contracts individually
make bench-bubblesort
make bench-erc20
make bench-factorial
```

### Issue: Build Errors

**Solution:**
```bash
# Clean and rebuild
cargo clean
cargo check --package hybrid-bench --benches
cargo bench --bench vm_comparison --no-run
```

## Development

### Adding New Contracts

1. **Add bytecode file:**
   ```bash
   cp NewContract.bin-runtime src/assets/
   ```

2. **Update benchmark configuration:**
   ```rust
   // In benches/vm_comparison.rs
   const CONTRACTS: &[ContractBenchConfig] = &[
       // ... existing contracts ...
       ContractBenchConfig::new("NewContract", ContractComplexity::Medium),
   ];
   ```

3. **Run benchmark:**
   ```bash
   cargo bench --bench vm_comparison NewContract
   ```

### Modifying Benchmark Parameters

Edit `benches/vm_comparison.rs`:

```rust
criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(10)              // Number of samples
        .measurement_time(Duration::from_secs(30))  // Time per benchmark
        .confidence_level(0.95)       // Statistical confidence
        .noise_threshold(0.05);       // 5% significance threshold
    targets = bench_revm, bench_hybrid_vm, bench_comparison
);
```

### Modifying Iteration Counts

Edit `src/lib.rs`:

```rust
pub const NO_OF_ITERATIONS_TWO: u64 = 120;  // Contract iterations
```

Edit `benches/vm_comparison.rs`:

```rust
impl ContractBenchConfig {
    const fn runs(&self) -> u64 {
        match self.complexity {
            ContractComplexity::Fast => 1000,    // Fast contracts
            ContractComplexity::Medium => 500,   // Medium contracts
            ContractComplexity::Slow => 100,     // Slow contracts
        }
    }
}
```

## Documentation

- **README.md** (this file) - Overview and quick reference
- **BENCHMARK.md** - Detailed documentation and methodology
- **QUICKSTART.md** - 60-second getting started guide
- **Makefile** - Build automation reference
- **run_benchmarks.sh** - Professional runner with optimizations

## Dependencies

```toml
[dependencies]
revm = { workspace = true }
sha3 = "0.10.8"
hybrid-vm = { workspace = true }
hybrid-ethereum = { workspace = true }

[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
```

## Requirements

- **Rust**: 1.70 or later
- **Cargo**: Latest stable
- **Disk Space**: ~500MB for reports
- **Time**: 15-30 minutes for full benchmark suite

## Contributing

We welcome contributions! When adding benchmarks:

1. Follow the existing code style
2. Choose appropriate complexity classification
3. Ensure contracts are deterministic
4. Document any special requirements
5. Test locally before submitting PR
6. Update documentation as needed

## License

MIT License - See [LICENSE](../../LICENSE) file in repository root.

## Credits

- Built with [Criterion.rs](https://github.com/bheisler/criterion.rs)
- Part of the [Hybrid VM](https://github.com/developeruche/hybrid) project
- Maintained by the Hybrid VM team

## Support

- 📚 [Full Documentation](./BENCHMARK.md)
- 🚀 [Quick Start Guide](./QUICKSTART.md)
- 💬 GitHub Issues for bug reports
- 🎯 GitHub Discussions for questions

---

**Pro Tip**: Run `make bench-fast && make report` for quick iteration during development! ⚡