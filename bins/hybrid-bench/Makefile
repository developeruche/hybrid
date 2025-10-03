# Hybrid VM Benchmark Suite Makefile
# Professional benchmark orchestration for REVM vs Hybrid VM comparison

.PHONY: all bench bench-all bench-revm bench-hybrid bench-riscv bench-compare bench-evm-vs-riscv bench-three-way help clean report bench-fast bench-slow list

# Default target
all: help

# Run all benchmarks
bench: bench-all

bench-all:
	@echo "🚀 Running complete benchmark suite..."
	@cargo bench --bench vm_comparison

# Run REVM benchmarks only
bench-revm:
	@echo "🔧 Running REVM benchmarks..."
	@cargo bench --bench vm_comparison revm

# Run Hybrid VM (EVM mode) benchmarks only
bench-hybrid:
	@echo "⚡ Running Hybrid VM (EVM mode) benchmarks..."
	@cargo bench --bench vm_comparison hybrid_vm

# Run Hybrid VM (RISC-V mode) benchmarks only
bench-riscv:
	@echo "🦀 Running Hybrid VM (RISC-V mode) benchmarks..."
	@cargo bench --bench vm_comparison hybrid_vm_riscv

# Run comparison benchmarks (side-by-side)
bench-compare:
	@echo "📊 Running comparison benchmarks..."
	@cargo bench --bench vm_comparison comparison

# Run EVM vs RISC-V mode comparison
bench-evm-vs-riscv:
	@echo "⚖️  Running EVM vs RISC-V mode comparison..."
	@cargo bench --bench vm_comparison evm_vs_riscv

# Run three-way comparison (REVM vs Hybrid EVM vs Hybrid RISC-V)
bench-three-way:
	@echo "🎯 Running three-way comparison (REVM vs EVM vs RISC-V)..."
	@cargo bench --bench vm_comparison three_way_comparison

# Quick benchmark with reduced sample size
bench-fast:
	@echo "⚡ Running fast benchmark (reduced samples)..."
	@CRITERION_SAMPLE_SIZE=5 cargo bench --bench vm_comparison

# Thorough benchmark with increased samples
bench-slow:
	@echo "🐌 Running thorough benchmark (increased samples)..."
	@CRITERION_SAMPLE_SIZE=50 cargo bench --bench vm_comparison

# Individual contract benchmarks
bench-bubblesort:
	@echo "🫧 Benchmarking BubbleSort..."
	@cargo bench --bench vm_comparison BubbleSort

bench-erc20:
	@echo "💰 Benchmarking ERC20 contracts..."
	@cargo bench --bench vm_comparison ERC20

bench-factorial:
	@echo "🔢 Benchmarking Factorial contracts..."
	@cargo bench --bench vm_comparison Factorial

bench-fibonacci:
	@echo "🌀 Benchmarking Fibonacci contracts..."
	@cargo bench --bench vm_comparison Fibonacci

bench-hashes:
	@echo "🔐 Benchmarking ManyHashes..."
	@cargo bench --bench vm_comparison ManyHashes

bench-storage:
	@echo "💾 Benchmarking storage operations..."
	@cargo bench --bench vm_comparison Bench

# Save baseline for future comparisons
baseline-save:
	@echo "💾 Saving current results as baseline..."
	@cargo bench --bench vm_comparison -- --save-baseline main

# Compare against saved baseline
baseline-compare:
	@echo "📊 Comparing against baseline..."
	@cargo bench --bench vm_comparison -- --baseline main

# Open HTML report in browser
report:
	@echo "📊 Opening benchmark report..."
	@open target/criterion/report/index.html || xdg-open target/criterion/report/index.html || echo "Please open target/criterion/report/index.html manually"

# List all available benchmarks
list:
	@echo "📋 Available benchmarks:"
	@echo ""
	@echo "  EVM Mode Contracts:"
	@echo "    Fast (10 runs):"
	@echo "      - Push"
	@echo "      - ERC20Transfer"
	@echo "      - Factorial"
	@echo "      - Fibonacci"
	@echo ""
	@echo "    Medium (10 runs):"
	@echo "      - ERC20ApprovalTransfer"
	@echo "      - ERC20Mint"
	@echo "      - MstoreBench"
	@echo "      - SstoreBench_no_opt"
	@echo ""
	@echo "    Slow (5 runs):"
	@echo "      - BubbleSort"
	@echo "      - ManyHashes"
	@echo ""
	@echo "  RISC-V Mode Contracts:"
	@echo "    Fast (10 runs):"
	@echo "      - ERC20Transfer"
	@echo "      - Factorial"
	@echo "      - Fibonacci"
	@echo ""
	@echo "    Medium (10 runs):"
	@echo "      - ERC20ApprovalTransfer"
	@echo "      - ERC20Mint"
	@echo ""
	@echo "    Slow (5 runs):"
	@echo "      - ManyHashes"
	@echo ""

# Clean benchmark artifacts
clean:
	@echo "🧹 Cleaning benchmark artifacts..."
	@rm -rf target/criterion
	@echo "✅ Cleaned benchmark data"

# Check benchmark code
check:
	@echo "🔍 Checking benchmark code..."
	@cargo check --package hybrid-bench --benches

# Build benchmarks without running
build:
	@echo "🔨 Building benchmarks..."
	@cargo bench --bench vm_comparison --no-run

# Help target
help:
	@echo "╔════════════════════════════════════════════════════════════════╗"
	@echo "║         Hybrid VM Benchmark Suite - Make Commands             ║"
	@echo "╚════════════════════════════════════════════════════════════════╝"
	@echo ""
	@echo "Main Targets:"
	@echo "  make bench              - Run all benchmarks"
	@echo "  make bench-revm         - Run REVM benchmarks only"
	@echo "  make bench-hybrid       - Run Hybrid VM (EVM mode) benchmarks only"
	@echo "  make bench-riscv        - Run Hybrid VM (RISC-V mode) benchmarks only"
	@echo "  make bench-compare      - Run comparison benchmarks"
	@echo "  make bench-evm-vs-riscv - Run EVM vs RISC-V mode comparison"
	@echo "  make bench-three-way    - Run three-way comparison (REVM vs EVM vs RISC-V)"
	@echo ""
	@echo "Speed Variants:"
	@echo "  make bench-fast         - Quick benchmark (reduced samples)"
	@echo "  make bench-slow         - Thorough benchmark (more samples)"
	@echo ""
	@echo "Individual Contracts:"
	@echo "  make bench-bubblesort   - Benchmark BubbleSort"
	@echo "  make bench-erc20        - Benchmark ERC20 contracts"
	@echo "  make bench-factorial    - Benchmark Factorial contracts"
	@echo "  make bench-fibonacci    - Benchmark Fibonacci contracts"
	@echo "  make bench-hashes       - Benchmark ManyHashes"
	@echo "  make bench-storage      - Benchmark storage operations"
	@echo ""
	@echo "Baseline Comparison:"
	@echo "  make baseline-save      - Save current results as baseline"
	@echo "  make baseline-compare   - Compare against saved baseline"
	@echo ""
	@echo "Utilities:"
	@echo "  make report             - Open HTML benchmark report"
	@echo "  make list               - List all available benchmarks"
	@echo "  make check              - Check benchmark code compilation"
	@echo "  make build              - Build benchmarks without running"
	@echo "  make clean              - Remove benchmark artifacts"
	@echo "  make help               - Show this help message"
	@echo ""
	@echo "Examples:"
	@echo "  make bench                              # Run all benchmarks"
	@echo "  make bench-fast && make report          # Quick bench + open report"
	@echo "  make bench-evm-vs-riscv                 # Compare EVM vs RISC-V modes"
	@echo "  make bench-three-way                    # Full performance comparison"
	@echo "  make baseline-save && make bench        # Save baseline & compare"
	@echo ""
