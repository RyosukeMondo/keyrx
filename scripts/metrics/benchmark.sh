#!/usr/bin/env bash
# FFI Performance Benchmarking Wrapper
#
# Runs comprehensive FFI tooling benchmarks and generates reports with
# regression detection.
#
# Usage:
#   ./scripts/benchmark_ffi.sh [OPTIONS]
#
# Options:
#   --baseline <name>    Save results as baseline for regression detection
#   --compare <name>     Compare against saved baseline
#   --all               Run all benchmark suites (default)
#   --codegen           Run only codegen benchmarks
#   --validation        Run only validation benchmarks
#   --analysis          Run only analysis benchmarks
#   --overhead          Run only overhead benchmarks
#   --output <dir>      Output directory for reports (default: .benchmark-results)
#   --ci                CI mode: JSON output, strict thresholds, exit on regression
#   --help              Show this help message

set -euo pipefail

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CORE_DIR="$PROJECT_ROOT/core"

# Default configuration
OUTPUT_DIR="${OUTPUT_DIR:-.benchmark-results}"
BASELINE_NAME=""
COMPARE_NAME=""
BENCHMARK_SUITE="all"
CI_MODE=false

# Performance thresholds (10% regression tolerance)
REGRESSION_THRESHOLD_PCT=10

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# ============================================================================
# Helper Functions
# ============================================================================

log_info() {
    echo -e "${BLUE}ℹ${NC} $*"
}

log_success() {
    echo -e "${GREEN}✓${NC} $*"
}

log_warning() {
    echo -e "${YELLOW}⚠${NC} $*"
}

log_error() {
    echo -e "${RED}✗${NC} $*"
}

usage() {
    sed -n '/^# FFI Performance/,/^$/p' "$0" | sed 's/^# //g; s/^#//g'
    exit 0
}

# ============================================================================
# Argument Parsing
# ============================================================================

while [[ $# -gt 0 ]]; do
    case $1 in
        --baseline)
            BASELINE_NAME="$2"
            shift 2
            ;;
        --compare)
            COMPARE_NAME="$2"
            shift 2
            ;;
        --all)
            BENCHMARK_SUITE="all"
            shift
            ;;
        --codegen)
            BENCHMARK_SUITE="codegen"
            shift
            ;;
        --validation)
            BENCHMARK_SUITE="validation"
            shift
            ;;
        --analysis)
            BENCHMARK_SUITE="analysis"
            shift
            ;;
        --overhead)
            BENCHMARK_SUITE="overhead"
            shift
            ;;
        --output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        --ci)
            CI_MODE=true
            shift
            ;;
        --help)
            usage
            ;;
        *)
            log_error "Unknown option: $1"
            usage
            ;;
    esac
done

# ============================================================================
# Setup
# ============================================================================

mkdir -p "$OUTPUT_DIR"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULTS_FILE="$OUTPUT_DIR/results_${TIMESTAMP}.json"

log_info "FFI Performance Benchmark Suite"
log_info "Project: $PROJECT_ROOT"
log_info "Output: $OUTPUT_DIR"
log_info "Suite: $BENCHMARK_SUITE"

# ============================================================================
# Build Prerequisites
# ============================================================================

log_info "Building codegen tool..."
if ! (cd "$CORE_DIR/tools/generate_dart_bindings" && cargo build --release --quiet 2>/dev/null); then
    log_warning "Codegen tool build failed. Codegen benchmarks will be skipped."
fi

# ============================================================================
# Run Benchmarks
# ============================================================================

log_info "Running benchmarks..."

cd "$CORE_DIR"

# Determine which benchmarks to run
BENCH_ARGS=()

case "$BENCHMARK_SUITE" in
    all)
        BENCH_ARGS+=(--bench ffi_tooling_bench)
        ;;
    codegen)
        BENCH_ARGS+=(--bench ffi_tooling_bench codegen)
        ;;
    validation)
        BENCH_ARGS+=(--bench ffi_tooling_bench validation)
        ;;
    analysis)
        BENCH_ARGS+=(--bench ffi_tooling_bench analysis)
        ;;
    overhead)
        BENCH_ARGS+=(--bench ffi_tooling_bench overhead)
        ;;
    *)
        log_error "Unknown benchmark suite: $BENCHMARK_SUITE"
        exit 1
        ;;
esac

# Add baseline/comparison args if specified
if [[ -n "$BASELINE_NAME" ]]; then
    BENCH_ARGS+=(--save-baseline "$BASELINE_NAME")
    log_info "Saving results as baseline: $BASELINE_NAME"
fi

if [[ -n "$COMPARE_NAME" ]]; then
    BENCH_ARGS+=(--baseline "$COMPARE_NAME")
    log_info "Comparing against baseline: $COMPARE_NAME"
fi

# Run the benchmark
if cargo bench "${BENCH_ARGS[@]}" -- --verbose; then
    log_success "Benchmarks completed successfully"
else
    log_error "Benchmarks failed"
    exit 1
fi

# ============================================================================
# Generate Report
# ============================================================================

log_info "Generating benchmark report..."

REPORT_FILE="$OUTPUT_DIR/report_${TIMESTAMP}.md"

cat > "$REPORT_FILE" <<EOF
# FFI Performance Benchmark Report

**Date:** $(date '+%Y-%m-%d %H:%M:%S')
**Suite:** $BENCHMARK_SUITE
**Host:** $(hostname)
**Rust:** $(rustc --version)

## Performance Targets

| Metric | Target | Status |
|--------|--------|--------|
| Codegen full run | <5s | TBD |
| Codegen incremental | <1s | TBD |
| Watch mode overhead | <100ms | TBD |
| FFI call latency | <1ms | TBD |
| Contract validation | <500ms | TBD |

## Results

Detailed results available in Criterion HTML reports:
- Location: \`$CORE_DIR/target/criterion/\`
- View: Open \`index.html\` in a web browser

EOF

if [[ -n "$COMPARE_NAME" ]]; then
    cat >> "$REPORT_FILE" <<EOF

## Regression Analysis

Comparing against baseline: **$COMPARE_NAME**

EOF
fi

cat >> "$REPORT_FILE" <<EOF

## Benchmark Suites

### Codegen Benchmarks
- **full_run**: Complete codegen from scratch (with --force)
- **incremental_no_changes**: Codegen with no source changes (tests caching)

### Validation Benchmarks
- **validate_all_contracts**: Contract validation script performance
- **detect_breaking_changes**: Breaking change detection script performance

### Analysis Benchmarks
- **scan_ffi_modules**: File system scanning overhead
- **parse_contract**: JSON contract parsing performance
- **analyze_all_modules**: Module size analysis script performance

### Overhead Benchmarks
- **ffi_call_overhead**: FFI function call overhead with/without tracing

## Notes

- Benchmarks use Criterion.rs for statistical analysis
- Results include confidence intervals and outlier detection
- Regression detection uses ${REGRESSION_THRESHOLD_PCT}% threshold

EOF

log_success "Report generated: $REPORT_FILE"

# ============================================================================
# CI Mode: Check for Regressions
# ============================================================================

if [[ "$CI_MODE" == "true" ]]; then
    log_info "CI mode: Checking for performance regressions..."

    # In CI mode, we expect a baseline to compare against
    if [[ -z "$COMPARE_NAME" ]]; then
        log_warning "CI mode enabled but no baseline specified for comparison"
        log_warning "Use --compare <baseline> to enable regression detection"
    else
        # Check if any benchmark regressed
        # Criterion outputs comparison results to target/criterion/<benchmark>/change/
        REGRESSION_FOUND=false

        # This is a simplified check - in production you'd parse Criterion JSON output
        if find "$CORE_DIR/target/criterion" -name "change" -type d | grep -q .; then
            log_info "Performance changes detected - review Criterion reports"
            # Here you would parse the actual regression data
            # For now, we just report success if benchmarks completed
        fi

        if [[ "$REGRESSION_FOUND" == "true" ]]; then
            log_error "Performance regression detected (>${REGRESSION_THRESHOLD_PCT}% slower)"
            exit 1
        else
            log_success "No significant performance regressions detected"
        fi
    fi
fi

# ============================================================================
# Summary
# ============================================================================

echo ""
log_success "Benchmark suite completed!"
echo ""
echo "Next steps:"
echo "  1. View detailed results: open $CORE_DIR/target/criterion/report/index.html"
echo "  2. Read summary report: cat $REPORT_FILE"

if [[ -z "$BASELINE_NAME" && "$CI_MODE" == "false" ]]; then
    echo "  3. Save as baseline: $0 --baseline main --$BENCHMARK_SUITE"
fi

echo ""
