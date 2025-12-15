# FFI Performance Guide

Comprehensive guide to FFI performance benchmarking, optimization, and regression detection for the KeyRx AI-native FFI workflow.

## Table of Contents

- [Overview](#overview)
- [Performance Targets](#performance-targets)
- [Running Benchmarks](#running-benchmarks)
- [Understanding Results](#understanding-results)
- [Optimization Guidelines](#optimization-guidelines)
- [CI/CD Integration](#cicd-integration)
- [Troubleshooting](#troubleshooting)

## Overview

The KeyRx FFI workflow is designed to be fast and responsive, enabling rapid iteration during development. Performance benchmarks ensure that tooling overhead stays minimal and doesn't regress over time.

### What We Benchmark

1. **Codegen Performance**
   - Full codegen run (from scratch)
   - Incremental codegen (with caching)
   - File scanning and parsing overhead

2. **Validation & Analysis**
   - Contract validation speed
   - Breaking change detection
   - Module size analysis

3. **Runtime Overhead**
   - FFI call latency
   - Tracing instrumentation overhead
   - Watch mode responsiveness

## Performance Targets

Aggressive targets ensure the AI-native workflow feels instant:

| Metric | Target | Measurement | Priority |
|--------|--------|-------------|----------|
| **Codegen full run** | <5s | Cold start, all contracts | High |
| **Codegen incremental** | <1s | Hot cache, no changes | Critical |
| **Watch mode overhead** | <100ms | File change detection + debounce | Critical |
| **FFI call latency** | <1ms | Single function call | Medium |
| **Contract validation** | <500ms | All contracts validated | High |
| **Breaking change detection** | <2s | Git diff analysis | Medium |
| **Module size analysis** | <30s | Full codebase scan | Low |

### Why These Targets?

- **<5s codegen**: Developers won't wait more than 5s for full regeneration
- **<1s incremental**: Enables near-instant feedback in watch mode
- **<100ms watch overhead**: Feels instant, won't interrupt flow
- **<1ms FFI latency**: UI stays responsive even with frequent FFI calls
- **<500ms validation**: Fast enough for pre-commit hooks

## Running Benchmarks

### Prerequisites

1. Install Criterion.rs (included in dev-dependencies)
2. Build the codegen tool:
   ```bash
   cd core/tools/generate_dart_bindings
   cargo build --release
   ```

### Quick Start

Run all benchmarks with default settings:

```bash
just bench-ffi
```

Or use the script directly:

```bash
./scripts/benchmark_ffi.sh
```

### Benchmark Suites

Run specific benchmark suites:

```bash
# Codegen only
./scripts/benchmark_ffi.sh --codegen

# Validation only
./scripts/benchmark_ffi.sh --validation

# Analysis only
./scripts/benchmark_ffi.sh --analysis

# Overhead only
./scripts/benchmark_ffi.sh --overhead

# All suites (default)
./scripts/benchmark_ffi.sh --all
```

### Saving Baselines

Save current performance as a baseline for regression detection:

```bash
# Save as "main" baseline
./scripts/benchmark_ffi.sh --baseline main

# Save as "before-optimization" baseline
./scripts/benchmark_ffi.sh --baseline before-optimization
```

### Comparing Against Baselines

Detect performance regressions by comparing against a saved baseline:

```bash
# Compare current performance vs "main" baseline
./scripts/benchmark_ffi.sh --compare main

# CI mode: fail on regression >10%
./scripts/benchmark_ffi.sh --compare main --ci
```

### Viewing Results

Criterion generates beautiful HTML reports:

1. Open `core/target/criterion/report/index.html` in a browser
2. Navigate to specific benchmarks
3. Review statistical analysis, charts, and comparisons

## Understanding Results

### Criterion Output Explained

```
codegen/full_run       time:   [3.234 s 3.456 s 3.678 s]
                       change: [-5.234% +2.156% +9.876%] (p = 0.23 > 0.05)
                       No change in performance detected.
```

- **time**: [lower bound, estimate, upper bound] with 95% confidence
- **change**: Performance change vs previous run (if available)
- **p-value**: Statistical significance (p < 0.05 = significant change)

### Performance Status Indicators

| Status | Meaning | Action |
|--------|---------|--------|
| ✓ Green | Within target | No action needed |
| ⚠ Yellow | Close to threshold (>80% of target) | Monitor, consider optimization |
| ✗ Red | Exceeds target | Optimization required |

### Sample Benchmark Results

```
Benchmark Results (2024-12-15)

✓ codegen/full_run              3.2s  (target: <5s)
✓ codegen/incremental_no_changes 0.4s  (target: <1s)
⚠ contract_validation/validate   0.45s (target: <500ms)
✓ ffi_call_overhead/no_tracing   0.3µs (target: <1ms)
✓ ffi_call_overhead/with_tracing 0.8µs (target: <1ms)
```

## Optimization Guidelines

### General Principles

1. **Measure First**: Always benchmark before optimizing
2. **Profile Hot Paths**: Use `cargo flamegraph` to identify bottlenecks
3. **Incremental Improvements**: Optimize one thing at a time
4. **Verify Impact**: Re-benchmark after each change

### Codegen Optimization

#### Current Optimizations

1. **Incremental Generation**: SHA256 hashing skips unchanged contracts
2. **Parallel Processing**: Multiple contracts generated concurrently (TODO)
3. **Fast Linker**: mold (Linux) or zld (macOS) reduces link time

#### Potential Optimizations

```rust
// Before: Serial contract generation
for contract in contracts {
    generate_bindings(contract)?;
}

// After: Parallel generation with rayon
use rayon::prelude::*;
contracts.par_iter()
    .try_for_each(|contract| generate_bindings(contract))?;
```

**Expected Impact**: 2-3x faster on multi-core systems

### Validation Optimization

#### Current Approach

```bash
# Slow: Separate grep for each function
for func in $(grep -r "#\[ffi_export\]" core/src/ffi); do
    validate_contract "$func"
done
```

#### Optimized Approach

```bash
# Fast: Single pass with ripgrep
rg --json '#\[ffi_export\]' core/src/ffi | \
    jq -r '.data.lines.text' | \
    xargs -P $(nproc) -n 1 validate_contract
```

**Expected Impact**: 5-10x faster on large codebases

### Watch Mode Optimization

Key optimizations:

1. **Debouncing (500ms)**: Prevent rapid-fire regeneration
2. **Smart Change Detection**: Only trigger on `.rs` file changes in FFI modules
3. **Incremental Codegen**: Skip unchanged contracts
4. **Background Processing**: Don't block terminal UI

### FFI Call Optimization

Tracing overhead is minimal (<1µs per call) but can be disabled for production:

```rust
// Development: Full tracing
#[instrument(skip(engine))]
pub fn ffi_start_engine(engine: &Engine) -> Result<()> {
    // ...
}

// Production: Compile-time stripping
#[cfg_attr(debug_assertions, instrument(skip(engine)))]
pub fn ffi_start_engine(engine: &Engine) -> Result<()> {
    // ...
}
```

## CI/CD Integration

### GitHub Actions Workflow

Example workflow for continuous performance monitoring:

```yaml
name: FFI Performance Benchmarks

on:
  push:
    branches: [main]
  pull_request:

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
        with:
          fetch-depth: 0  # Need history for git diff

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Setup FFI tools
        run: just setup-ffi-tools

      - name: Download baseline
        uses: actions/cache@v3
        with:
          path: target/criterion
          key: criterion-baseline-${{ github.base_ref }}

      - name: Run benchmarks
        run: ./scripts/benchmark_ffi.sh --compare main --ci

      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: benchmark-results
          path: .benchmark-results/
```

### Regression Detection

The `--ci` flag enables strict regression checking:

- **Threshold**: 10% slower than baseline
- **Exit Code**: Non-zero on regression
- **Output**: JSON format for parsing

```bash
# In CI pipeline
./scripts/benchmark_ffi.sh --compare main --ci || {
    echo "Performance regression detected!"
    exit 1
}
```

## Troubleshooting

### Benchmark Failures

#### "Codegen tool not found"

**Problem**: Benchmark can't locate the codegen binary

**Solution**:
```bash
cd core/tools/generate_dart_bindings
cargo build --release
```

#### "Script not found" errors

**Problem**: Validation/analysis scripts missing

**Solution**:
```bash
# Ensure all scripts are executable
chmod +x scripts/*.sh
```

### Performance Issues

#### Slow Codegen (>5s)

**Diagnosis**:
```bash
# Profile codegen
cd core/tools/generate_dart_bindings
cargo build --release
time ./target/release/generate_dart_bindings --verbose
```

**Common Causes**:
- Large number of contracts (>100)
- Slow disk I/O
- No incremental caching

**Solutions**:
- Enable caching (remove `--force` flag)
- Use fast linker (mold/zld)
- Profile with `cargo flamegraph`

#### High Watch Mode Overhead (>100ms)

**Diagnosis**:
```bash
# Monitor watch mode responsiveness
./scripts/watch_ffi.sh &
time touch core/src/ffi/domains/engine/test.rs
```

**Common Causes**:
- Debounce too short (rapid-fire events)
- Large directory tree
- Slow codegen

**Solutions**:
- Increase debounce to 500ms
- Exclude non-FFI directories from watch
- Optimize incremental codegen

### Statistical Anomalies

#### High Variance in Results

**Problem**: Benchmark results vary widely between runs

**Possible Causes**:
- Background processes consuming CPU
- Thermal throttling
- I/O contention

**Solutions**:
```bash
# Increase sample size for better statistics
cd core
cargo bench -- --sample-size 20 --measurement-time 60

# Run on isolated machine
systemctl stop <background-services>
```

#### Unexplained Performance Changes

**Investigation Steps**:

1. Check git history:
   ```bash
   git log --oneline --no-merges -- core/tools/generate_dart_bindings
   ```

2. Compare Criterion reports:
   ```bash
   # Visual comparison
   open core/target/criterion/codegen/full_run/report/index.html
   ```

3. Profile with flamegraph:
   ```bash
   cargo install flamegraph
   cargo flamegraph --bench ffi_tooling_bench -- codegen/full_run
   ```

## Best Practices

### Development Workflow

1. **Before Optimization**
   - Run baseline: `./scripts/benchmark_ffi.sh --baseline before`
   - Note current performance

2. **During Optimization**
   - Make incremental changes
   - Benchmark after each change
   - Keep what works, discard what doesn't

3. **After Optimization**
   - Compare: `./scripts/benchmark_ffi.sh --compare before`
   - Document improvements
   - Save new baseline: `./scripts/benchmark_ffi.sh --baseline main`

### Performance Reviews

Include benchmark results in PR descriptions:

```markdown
## Performance Impact

Benchmarked with `./scripts/benchmark_ffi.sh --compare main`:

- codegen/full_run: 3.2s → 2.1s (-34% ✓)
- codegen/incremental: 0.4s → 0.3s (-25% ✓)
- No regressions detected

See: .benchmark-results/report_*.md
```

## Advanced Topics

### Custom Benchmarks

Add new benchmarks to `core/benches/ffi_tooling_bench.rs`:

```rust
fn benchmark_my_feature(c: &mut Criterion) {
    let mut group = c.benchmark_group("my_feature");

    group.bench_function("my_operation", |b| {
        b.iter(|| {
            // Your code here
            black_box(my_operation())
        });
    });

    group.finish();
}

criterion_group!(my_benches, benchmark_my_feature);
criterion_main!(my_benches);
```

### Profiling with perf

For deep performance analysis:

```bash
# Linux only
cargo install cargo-flamegraph

# Profile codegen
cd core/tools/generate_dart_bindings
cargo flamegraph --release -- --force

# View flamegraph.svg in browser
```

### Memory Profiling

Track memory allocations:

```bash
# Install valgrind/massif
sudo apt install valgrind

# Profile memory usage
valgrind --tool=massif \
    ./core/tools/generate_dart_bindings/target/release/generate_dart_bindings

# Analyze results
ms_print massif.out.*
```

## References

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [FFI Development Guide](./FFI_DEVELOPMENT.md)
- [FFI Tracing Guide](./FFI_TRACING.md)

---

**Last Updated**: 2024-12-15
**Maintained By**: KeyRx Core Team
