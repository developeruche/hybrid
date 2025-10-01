#!/usr/bin/env bash

# Hybrid VM Benchmark Runner
# Professional benchmark execution script with environment optimization

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Banner
print_banner() {
    echo -e "${CYAN}"
    echo "╔════════════════════════════════════════════════════════════════╗"
    echo "║         Hybrid VM Benchmark Suite - Professional Runner       ║"
    echo "╚════════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

# Print colored message
info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

success() {
    echo -e "${GREEN}✓${NC} $1"
}

warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

error() {
    echo -e "${RED}✗${NC} $1"
}

# Check system readiness
check_system() {
    info "Checking system readiness..."

    # Check if cargo is available
    if ! command -v cargo &> /dev/null; then
        error "Cargo not found. Please install Rust."
        exit 1
    fi

    success "Cargo found: $(cargo --version)"

    # Check if we're in the right directory
    if [ ! -f "Cargo.toml" ]; then
        error "Cargo.toml not found. Please run from hybrid-bench directory."
        exit 1
    fi

    success "Directory check passed"

    # Warn about system load
    if command -v uptime &> /dev/null; then
        load=$(uptime | awk -F'load average:' '{print $2}' | awk '{print $1}' | tr -d ',')
        cores=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo "unknown")
        if [ "$cores" != "unknown" ] && [ "$(echo "$load > $cores" | bc -l 2>/dev/null || echo 0)" = "1" ]; then
            warning "System load is high ($load on $cores cores). Results may vary."
        else
            success "System load is acceptable"
        fi
    fi
}

# Optimize environment for benchmarking
optimize_environment() {
    info "Optimizing environment for benchmarking..."

    # Set Rust flags for optimal performance
    export RUSTFLAGS="-C target-cpu=native"
    success "Rust flags configured for native CPU"

    # Disable debug assertions
    export CARGO_PROFILE_RELEASE_DEBUG_ASSERTIONS=false
    success "Debug assertions disabled"

    # Set benchmark environment variables
    export CRITERION_HOME="${CRITERION_HOME:-target/criterion}"
    success "Criterion home: $CRITERION_HOME"
}

# Run benchmarks
run_benchmark() {
    local benchmark_type=$1

    case $benchmark_type in
        all)
            info "Running complete benchmark suite..."
            cargo bench --bench vm_comparison
            ;;
        revm)
            info "Running REVM benchmarks..."
            cargo bench --bench vm_comparison revm
            ;;
        hybrid)
            info "Running Hybrid VM benchmarks..."
            cargo bench --bench vm_comparison hybrid_vm
            ;;
        comparison)
            info "Running comparison benchmarks..."
            cargo bench --bench vm_comparison comparison
            ;;
        fast)
            info "Running fast benchmarks (reduced samples)..."
            CRITERION_SAMPLE_SIZE=5 cargo bench --bench vm_comparison
            ;;
        thorough)
            info "Running thorough benchmarks (increased samples)..."
            CRITERION_SAMPLE_SIZE=50 cargo bench --bench vm_comparison
            ;;
        *)
            info "Running benchmark for: $benchmark_type"
            cargo bench --bench vm_comparison "$benchmark_type"
            ;;
    esac
}

# Generate report
generate_report() {
    info "Benchmark complete!"
    echo ""
    info "Results saved to: target/criterion/"

    if [ -f "target/criterion/report/index.html" ]; then
        success "HTML report available at: target/criterion/report/index.html"

        # Ask to open report
        if [ -t 0 ]; then
            read -p "$(echo -e ${CYAN}Open report in browser? [y/N]:${NC} )" -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                if command -v open &> /dev/null; then
                    open target/criterion/report/index.html
                elif command -v xdg-open &> /dev/null; then
                    xdg-open target/criterion/report/index.html
                else
                    warning "Could not auto-open browser. Please open manually."
                fi
            fi
        fi
    fi
}

# Show usage
usage() {
    echo "Usage: $0 [OPTIONS] [BENCHMARK]"
    echo ""
    echo "Options:"
    echo "  -h, --help          Show this help message"
    echo "  -c, --check         Check system readiness only"
    echo "  -f, --fast          Run fast benchmarks (reduced samples)"
    echo "  -t, --thorough      Run thorough benchmarks (more samples)"
    echo "  --no-optimize       Skip environment optimization"
    echo ""
    echo "Benchmarks:"
    echo "  all                 Run all benchmarks (default)"
    echo "  revm                Run REVM benchmarks only"
    echo "  hybrid              Run Hybrid VM benchmarks only"
    echo "  comparison          Run comparison benchmarks"
    echo "  <CONTRACT>          Run specific contract (e.g., BubbleSort)"
    echo ""
    echo "Examples:"
    echo "  $0                            # Run all benchmarks"
    echo "  $0 --fast                     # Quick benchmark"
    echo "  $0 revm                       # REVM only"
    echo "  $0 BubbleSort                 # Specific contract"
    echo "  $0 --thorough comparison      # Thorough comparison"
    echo ""
}

# Main execution
main() {
    local benchmark_type="all"
    local skip_optimize=false
    local check_only=false

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                usage
                exit 0
                ;;
            -c|--check)
                check_only=true
                shift
                ;;
            -f|--fast)
                benchmark_type="fast"
                shift
                ;;
            -t|--thorough)
                benchmark_type="thorough"
                shift
                ;;
            --no-optimize)
                skip_optimize=true
                shift
                ;;
            *)
                benchmark_type=$1
                shift
                ;;
        esac
    done

    print_banner

    # Check system
    check_system

    if [ "$check_only" = true ]; then
        success "System check complete. Ready for benchmarking!"
        exit 0
    fi

    # Optimize if not skipped
    if [ "$skip_optimize" = false ]; then
        optimize_environment
    fi

    echo ""
    info "Starting benchmarks..."
    echo ""

    # Record start time
    start_time=$(date +%s)

    # Run benchmark
    if run_benchmark "$benchmark_type"; then
        # Calculate duration
        end_time=$(date +%s)
        duration=$((end_time - start_time))
        minutes=$((duration / 60))
        seconds=$((duration % 60))

        echo ""
        success "Benchmark completed in ${minutes}m ${seconds}s"
        echo ""

        # Generate report
        generate_report
    else
        error "Benchmark failed!"
        exit 1
    fi
}

# Run main function
main "$@"
