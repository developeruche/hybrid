# Hybrid VM Benchmark Suite

> Professional performance benchmarking for REVM vs Hybrid VM (EVM & RISC-V modes) comparison

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Criterion](https://img.shields.io/badge/benchmark-criterion-blue.svg)](https://github.com/bheisler/criterion.rs)

## Overview

This benchmark suite provides comprehensive, statistically-rigorous performance analysis comparing three execution modes:

- **REVM**: The reference Rust EVM implementation
- **Hybrid VM (EVM Mode)**: The hybrid virtual machine running in EVM-compatible mode
- **Hybrid VM (RISC-V Mode)**: The hybrid virtual machine running native RISC-V bytecode

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

✅ **Comprehensive Coverage**: 12 EVM contracts + 6 RISC-V contracts spanning various complexity levels  
✅ **Multi-Mode Comparison**: EVM vs RISC-V mode performance analysis  
✅ **Statistical Rigor**: Criterion.rs with confidence intervals and outlier detection  
✅ **Smart Iteration Counts**: Complexity-based run counts for optimal benchmark duration  
✅ **Professional Reports**: HTML reports with interactive charts and historical comparison  
✅ **CI/CD Ready**: Baseline comparison and regression detection  
✅ **Well-Documented**: Extensive documentation and examples

## Benchmarked Contracts

### EVM Mode Contracts

#### Fast Contracts (10 runs)
Lightweight operations with minimal computational overhead:
- **Push** - Basic stack push operations
- **ERC20Transfer** - Standard ERC20 token transfer
- **Factorial** - Iterative factorial calculation
- **Fibonacci** - Iterative Fibonacci sequence

#### Medium Complexity (10 runs)
Standard smart contract operations with moderate complexity:
- **ERC20ApprovalTransfer** - ERC20 approval and transfer flow
- **ERC20Mint** - ERC20 token minting operation
- **MstoreBench** - Memory storage benchmarks
- **SstoreBench_no_opt** - Storage operations without optimization

#### Slow Contracts (5 runs)
Computationally intensive operations:
- **BubbleSort** - Sorting algorithm benchmark
- **ManyHashes** - Intensive cryptographic hash operations

### RISC-V Mode Contracts

These contracts are compiled to native RISC-V bytecode and run on the Hybrid VM in RISC-V mode:

#### Fast Contracts (10 runs)
- **ERC20Transfer** - Standard ERC20 token transfer
- **Factorial** - Iterative factorial calculation
- **Fibonacci** - Iterative Fibonacci sequence

#### Medium Complexity (10 runs)
- **ERC20ApprovalTransfer** - ERC20 approval and transfer flow
- **ERC20Mint** - ERC20 token minting operation

#### Slow Contracts (5 runs)
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
NO_OF_ITERATIONS_ONE: u64 = 10   // RISC-V mode
NO_OF_ITERATIONS_TWO: u64 = 120  // EVM mode

// Benchmark runs per contract (based on complexity)
Fast:    10 runs
Medium:  10 runs
Slow:    5 runs

// Criterion configuration
Sample Size:       10
Measurement Time:  3 seconds
Confidence Level:  95%
Noise Threshold:   5%
```

## Usage Examples

### Basic Benchmarking

```bash
# Run all benchmarks (including RISC-V)
cargo bench --bench vm_comparison

# Run specific VM benchmarks
cargo bench --bench vm_comparison revm           # REVM only
cargo bench --bench vm_comparison hybrid_vm      # Hybrid VM (EVM mode)
cargo bench --bench vm_comparison hybrid_vm_riscv # Hybrid VM (RISC-V mode)

# Run comparison groups
cargo bench --bench vm_comparison comparison        # EVM comparison
cargo bench --bench vm_comparison evm_vs_riscv      # EVM vs RISC-V
cargo bench --bench vm_comparison three_way_comparison # All three modes
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
make bench                  # Run all benchmarks
make bench-revm             # REVM only
make bench-hybrid           # Hybrid VM (EVM mode) only
make bench-riscv            # Hybrid VM (RISC-V mode) only
make bench-compare          # Side-by-side EVM comparison
make bench-evm-vs-riscv     # EVM vs RISC-V mode comparison
make bench-three-way        # Three-way comparison (REVM vs EVM vs RISC-V)
make bench-fast             # Quick benchmark (reduced samples)
make bench-slow             # Thorough benchmark (increased samples)
make bench-bubblesort       # Specific contract
make bench-erc20            # All ERC20 contracts
make baseline-save          # Save baseline
make baseline-compare       # Compare with baseline
make report                 # Open HTML report
make clean                  # Remove artifacts
make list                   # List all contracts
make help                   # Show all commands
```

## Understanding Results

### Console Output

#### EVM Mode Comparison
```
revm_BubbleSort     time:   [45.123 ms 45.456 ms 45.789 ms]
                           change: [+2.1% +2.5% +2.9%] (p = 0.00 < 0.05)
                           Performance has regressed.

hybrid_BubbleSort   time:   [43.234 ms 43.567 ms 43.901 ms]
                           change: [-5.2% -4.9% -4.5%] (p = 0.00 < 0.05)
                           Performance has improved.
```

#### EVM vs RISC-V Mode Comparison
```
evm_mode/Factorial    time:   [1.234 ms 1.256 ms 1.278 ms]

riscv_mode/Factorial  time:   [0.987 ms 1.012 ms 1.037 ms]
                             change: [-21.3% -19.4% -17.5%] (p = 0.00 < 0.05)
                             RISC-V mode is faster!
```

#### Three-Way Comparison
```
revm/ERC20Transfer        time:   [2.123 ms 2.145 ms 2.167 ms]
hybrid_evm/ERC20Transfer  time:   [2.234 ms 2.256 ms 2.278 ms]
hybrid_riscv/ERC20Transfer time:   [1.789 ms 1.812 ms 1.835 ms]
                                  RISC-V mode is 19% faster!
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

**Pro Tip**: Run `make bench-evm-vs-riscv && make report` to see EVM vs RISC-V performance comparison! ⚡