# Build Optimization - Baseline Metrics

**Date:** 2025-12-03
**System:** Linux 6.14.0-36-generic
**Rust Version:** (as per cargo build output)

## Measurement Methodology

All measurements were performed on a clean build after running `cargo clean`. The `/usr/bin/time -v` command was used to capture detailed build statistics.

## Development Build (debug profile)

### Build Time
- **Wall Clock Time:** 43.54 seconds (0:43.54)
- **CPU Time:** 339.04 seconds (316.06 user + 22.98 system)
- **CPU Utilization:** 778% (highly parallelized)

### Binary Size
- **Executable:** 141 MB
- **Location:** `target/debug/keyrx`

### Build Resources
- **Maximum Memory:** 1,497,732 KB (~1.46 GB)
- **Page Faults (Minor):** 6,356,719
- **Context Switches (Voluntary):** 41,192
- **Context Switches (Involuntary):** 39,408

## Release Build (release profile)

### Build Time
- **Wall Clock Time:** 1:07.69 (67.69 seconds)
- **CPU Time:** 199.50 seconds (184.95 user + 14.55 system)
- **CPU Utilization:** 294%

### Binary Size
- **Executable:** 6.2 MB
- **Location:** `target/release/keyrx`

### Build Resources
- **Maximum Memory:** 858,164 KB (~838 MB)
- **Page Faults (Minor):** 3,914,182
- **Context Switches (Voluntary):** 20,318
- **Context Switches (Involuntary):** 19,072

## Summary Comparison

| Metric | Dev Build | Release Build | Delta |
|--------|-----------|---------------|-------|
| **Wall Time** | 43.54s | 67.69s | +55.5% |
| **Binary Size** | 141 MB | 6.2 MB | -95.6% |
| **Memory Usage** | 1.46 GB | 838 MB | -42.5% |
| **CPU Utilization** | 778% | 294% | -62.2% |

## Key Observations

1. **Dev Build Performance**: The development build is relatively fast at ~44 seconds with excellent parallelization (778% CPU usage suggests ~8 cores utilized).

2. **Release Build Trade-offs**: The release build takes ~56% longer but produces a binary that is 95.6% smaller (6.2 MB vs 141 MB).

3. **Memory Footprint**: Both builds are memory-intensive, with the dev build peaking at nearly 1.5 GB.

4. **Optimization Opportunities**:
   - High CPU utilization in dev builds suggests good parallelization but also indicates potential for dependency reduction
   - Large debug binary size (141 MB) suggests significant debug information overhead
   - Release build time of ~68 seconds provides room for profile optimization

## Target Goals

Based on these baselines, optimization targets should aim for:

- **Dev Build:** < 30 seconds (30% improvement)
- **Release Build:** < 50 seconds (26% improvement)
- **Release Binary:** < 5 MB (19% improvement)
- **Memory Usage:** < 1 GB for dev, < 700 MB for release (25-30% improvement)

## Next Steps

1. Audit dependencies for unused features (especially tokio, serde, windows-rs)
2. Implement feature flags for platform-specific code
3. Optimize build profiles (LTO, codegen-units, opt-level)
4. Enable incremental compilation and caching strategies
5. Re-measure after each optimization phase
