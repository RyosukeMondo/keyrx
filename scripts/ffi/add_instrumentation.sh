#!/usr/bin/env bash
# Add FFI instrumentation to all export functions

set -euo pipefail

# This script adds #[instrument] attributes to all FFI export functions
# that don't already have them.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
FFI_DIR="$PROJECT_ROOT/core/src/ffi"

echo "FFI Instrumentation Tool"
echo "========================"
echo ""
echo "Scanning for FFI export functions without instrumentation..."

# Find all FFI export files
export_files=(
    "$FFI_DIR/exports_compat.rs"
    "$FFI_DIR/exports_engine.rs"
    "$FFI_DIR/exports_metrics.rs"
    "$FFI_DIR/exports_runtime.rs"
    "$FFI_DIR/exports_telemetry.rs"
    "$FFI_DIR/exports_transition_log.rs"
    "$FFI_DIR/domains/device_registry.rs"
    "$FFI_DIR/domains/device_definitions.rs"
    "$FFI_DIR/domains/profile_registry.rs"
    "$FFI_DIR/domains/observability.rs"
)

total_functions=0
instrumented=0
not_instrumented=0

for file in "${export_files[@]}"; do
    if [[ ! -f "$file" ]]; then
        continue
    fi

    echo "Checking: $(basename "$file")"

    # Count functions with #[ffi_export] or extern "C" fn keyrx_
    fn_count=$(grep -cE '^(pub )?extern "C" fn keyrx_|#\[ffi_export' "$file" || true)
    total_functions=$((total_functions + fn_count))

    # Count functions with #[instrument]
    inst_count=$(grep -cE '#\[instrument' "$file" || true)
    instrumented=$((instrumented + inst_count))
done

not_instrumented=$((total_functions - instrumented))

echo ""
echo "Summary:"
echo "  Total FFI functions: $total_functions"
echo "  Already instrumented: $instrumented"
echo "  Need instrumentation: $not_instrumented"
echo ""

if [[ $not_instrumented -eq 0 ]]; then
    echo "✓ All FFI functions are instrumented!"
    exit 0
fi

echo "Note: Automatic instrumentation is complex and may require manual review."
echo "This script provides a report only. Manual instrumentation is recommended."
echo ""
echo "Functions missing instrumentation can be found by running:"
echo "  grep -B5 'extern \"C\" fn keyrx_' core/src/ffi/*.rs | grep -v '#\[instrument'"
