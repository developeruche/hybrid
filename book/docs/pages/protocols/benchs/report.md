---
description: Hybrid-VM vs REVM benchmark
---

## Hybrid VM Benchmark Results

> Performance comparison between REVM and Hybrid VM (running on EVM Mode)

**Benchmark Date**: 2025-09-30 
**Configuration**: NO_OF_ITERATIONS = 120  
**Criterion Settings**: 10 samples, 3s measurement time, 95% confidence  
**System**: macOS M3 max (native CPU optimization)


This document presents the performance analysis of REVM vs Hybrid VM running in EVM-compatible mode across 10 smart contracts. The benchmarks reveal **significant performance differences** between the two implementations, with Hybrid VM showing substantially slower execution times across all tested contracts.

### Key Findings

⚠️ **Critical Performance Gap Identified**: Hybrid VM demonstrates **significantly slower performance** compared to REVM:
- **BubbleSort**: 595x slower (38.5 seconds vs 64.6ms)
- **ManyHashes**: 1,984x slower (551ms vs 277µs)
- **ERC20 Operations**: 455-781x slower
- **Simple Operations**: 1,490-2,649x slower

### Performance Impact
This represents a **critical performance issue** that requires immediate investigation and optimization before production deployment.

### Detailed Benchmark Results

**1. Intensive Computation Contract (100 runs)**

**🫧 BubbleSort**
```
REVM:       64.625 ms   [63.839 - 65.166 ms]
Hybrid VM:  38.460 s    [38.416 - 38.510 s]

Performance: Hybrid VM 595x slower (59,500% overhead)
Status: ❌ CRITICAL - Requires immediate optimization
```

**2. Cryptographic Operations (100 runs)**

**🔐 ManyHashes**
```
REVM:       277.74 µs   [276.31 - 279.90 µs]
Hybrid VM:  551.22 ms   [547.56 - 554.95 ms]

Performance: Hybrid VM 1,984x slower (198,400% overhead)
Status: ❌ CRITICAL - Requires immediate optimization
```


**3. Medium Complexity Contracts (500 runs)**

**💰 ERC20ApprovalTransfer**
```
REVM:       6.8709 ms   [6.8136 - 6.9054 ms]
Hybrid VM:  5.3662 s    [5.3487 - 5.3827 s]

Performance: Hybrid VM 781x slower (78,100% overhead)
Status: ❌ CRITICAL - Requires immediate optimization
```

**🪙 ERC20Mint**
```
REVM:       1.1797 ms   [1.1630 - 1.1908 ms]
Hybrid VM:  1.5045 s    [1.4966 - 1.5117 s]

Performance: Hybrid VM 1,275x slower (127,500% overhead)
Status: ❌ CRITICAL - Requires immediate optimization
```

**💾 MstoreBench (Memory operations)**
```
REVM:       255.68 µs   [253.01 - 257.76 µs]
Hybrid VM:  1.0311 s    [1.0267 - 1.0361 s]

Performance: Hybrid VM 4,032x slower (403,200% overhead)
Status: ❌ CRITICAL - Requires immediate optimization
```

**📦 SstoreBench_no_opt (Storage operations)**
```
REVM:       2.0400 ms   [2.0244 - 2.0521 ms]
Hybrid VM:  5.1756 s    [5.1625 - 5.1895 s]

Performance: Hybrid VM 2,537x slower (253,700% overhead)
Status: ❌ CRITICAL - Requires immediate optimization
```

**4. Fast Contracts (1000 runs, simple operations)**

**💸 ERC20Transfer**
```
REVM:       1.7650 ms   [1.7513 - 1.7767 ms]
Hybrid VM:  1.9586 s    [1.9492 - 1.9676 s]

Performance: Hybrid VM 1,110x slower (111,000% overhead)
Status: ❌ CRITICAL - Requires immediate optimization
```

**🔢 Factorial (Iterative)**
```
REVM:       329.80 µs   [327.58 - 331.64 µs]
Hybrid VM:  873.95 ms   [865.43 - 882.66 ms]

Performance: Hybrid VM 2,649x slower (264,900% overhead)
Status: ❌ CRITICAL - Requires immediate optimization
```

**🌀 Factorial (Iterative)**
```
REVM:       329.80 µs   [327.58 - 331.64 µs]
Hybrid VM:  873.95 ms   [865.43 - 882.66 ms]

Performance: Hybrid VM 2,649x slower (264,900% overhead)
Status: ❌ CRITICAL - Requires immediate optimization
```

**🌀 Fibonacci (Iterative)**
```
REVM:       587.24 µs   [582.34 - 593.41 µs]
Hybrid VM:  989.39 ms   [982.41 - 996.17 ms]

Performance: Hybrid VM 1,685x slower (168,500% overhead)
Status: ❌ CRITICAL - Requires immediate optimization
```

**📚 Push (Stack operations)**
```
REVM:       627.20 µs   [622.82 - 634.45 µs]
Hybrid VM:  1.2974 s    [1.2915 - 1.3042 s]

Performance: Hybrid VM 2,069x slower (206,900% overhead)
Status: ❌ CRITICAL - Requires immediate optimization
```


## Performance Analysis by Category

### Aggregated Results Table

| Contract | Type | REVM | Hybrid VM | Slowdown | Status |
|----------|------|------|-----------|----------|--------|
| BubbleSort | Slow | 64.6 ms | 38.46 s | 595x | ❌ CRITICAL |
| ManyHashes | Slow | 277.7 µs | 551.2 ms | 1,984x | ❌ CRITICAL |
| ERC20ApprovalTransfer | Medium | 6.87 ms | 5.37 s | 781x | ❌ CRITICAL |
| ERC20Mint | Medium | 1.18 ms | 1.50 s | 1,275x | ❌ CRITICAL |
| MstoreBench | Medium | 255.7 µs | 1.03 s | 4,032x | ❌ CRITICAL |
| SstoreBench_no_opt | Medium | 2.04 ms | 5.18 s | 2,537x | ❌ CRITICAL |
| ERC20Transfer | Fast | 1.77 ms | 1.96 s | 1,110x | ❌ CRITICAL |
| Factorial | Fast | 329.8 µs | 874.0 ms | 2,649x | ❌ CRITICAL |
| Fibonacci | Fast | 587.2 µs | 989.4 ms | 1,685x | ❌ CRITICAL |
| Push | Fast | 627.2 µs | 1.30 s | 2,069x | ❌ CRITICAL |
