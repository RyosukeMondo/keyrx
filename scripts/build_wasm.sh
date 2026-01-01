#!/bin/bash
# Build keyrx_core to WebAssembly using wasm-pack
#
# Purpose:
#   Compiles keyrx_core Rust crate to WASM for browser use
#   Outputs to keyrx_ui_v2/src/wasm/pkg/ for frontend integration
#
# Dependencies:
#   - wasm-pack (install: cargo install wasm-pack)
#   - Rust toolchain with wasm32-unknown-unknown target
#
# Usage:
#   ./scripts/build_wasm.sh [--quiet] [--json] [--log-file PATH]
#
# Exit codes:
#   0 - Build successful
#   1 - Missing dependencies or build failure

set -euo pipefail

# Source common utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"

# Store the project root directory (current directory) BEFORE any other operations
PROJECT_ROOT="$(pwd)"

# Parse command line arguments
parse_common_flags "$@"
setup_log_file "build_wasm"

# Convert LOG_FILE to absolute path if it's relative
if [[ -n "$LOG_FILE" ]] && [[ ! "$LOG_FILE" = /* ]]; then
    LOG_FILE="$PROJECT_ROOT/$LOG_FILE"
fi

log_info "Building keyrx_core to WebAssembly..."
separator

# Check if wasm-pack is installed
if ! command_exists wasm-pack; then
    log_error "wasm-pack is not installed"
    log_error "Install it with: cargo install wasm-pack"
    log_error "Or visit: https://rustwasm.github.io/wasm-pack/installer/"
    log_failed
    exit 1
fi

log_info "wasm-pack found: $(wasm-pack --version)"

# Navigate to keyrx_core directory
KEYRX_CORE_DIR="$PROJECT_ROOT/keyrx_core"
OUTPUT_DIR="$PROJECT_ROOT/keyrx_ui_v2/src/wasm/pkg"

if [[ ! -d "$KEYRX_CORE_DIR" ]]; then
    log_error "keyrx_core directory not found"
    log_failed
    exit 1
fi

log_info "Building WASM from $KEYRX_CORE_DIR..."

# Record build start time
BUILD_START=$(date +%s)

# Build WASM with wasm-pack
cd "$KEYRX_CORE_DIR"

if wasm-pack build \
    --target web \
    --out-dir "$OUTPUT_DIR" \
    --release \
    -- --features wasm; then
    log_info "WASM build completed successfully"
else
    log_error "wasm-pack build failed"
    cd "$PROJECT_ROOT"
    log_failed
    exit 1
fi

cd "$PROJECT_ROOT"

# Record build end time
BUILD_END=$(date +%s)
BUILD_TIME=$((BUILD_END - BUILD_START))

# Verify output files exist
REQUIRED_FILES=(
    "$OUTPUT_DIR/keyrx_core_bg.wasm"
    "$OUTPUT_DIR/keyrx_core.js"
    "$OUTPUT_DIR/keyrx_core.d.ts"
)

log_info "Verifying output files..."
MISSING_FILES=()

for file in "${REQUIRED_FILES[@]}"; do
    if [[ -f "$file" ]]; then
        log_debug "Found: $file"
    else
        log_error "Missing: $file"
        MISSING_FILES+=("$file")
    fi
done

if [[ ${#MISSING_FILES[@]} -gt 0 ]]; then
    log_error "Build verification failed - missing files:"
    for file in "${MISSING_FILES[@]}"; do
        log_error "  - $file"
    done
    log_failed
    exit 1
fi

# Get WASM file size
WASM_FILE="$OUTPUT_DIR/keyrx_core_bg.wasm"
WASM_SIZE=$(stat -c%s "$WASM_FILE" 2>/dev/null || stat -f%z "$WASM_FILE" 2>/dev/null || echo "unknown")

if [[ "$WASM_SIZE" != "unknown" ]]; then
    WASM_SIZE_KB=$((WASM_SIZE / 1024))
    WASM_SIZE_MB=$((WASM_SIZE_KB / 1024))

    log_info "WASM file size: ${WASM_SIZE_KB} KB (${WASM_SIZE_MB}.$(( (WASM_SIZE_KB % 1024) * 100 / 1024 )) MB)"

    # Check if WASM size is under 1MB (as per requirements)
    if [[ $WASM_SIZE_MB -ge 1 ]]; then
        log_warn "WASM file size exceeds 1MB target"
    fi
else
    log_warn "Could not determine WASM file size"
fi

separator
log_info "Build time: ${BUILD_TIME} seconds"
log_accomplished

# Output JSON if requested
if [[ "$JSON_MODE" == "true" ]]; then
    output_json \
        "status" "success" \
        "build_time_seconds" "$BUILD_TIME" \
        "wasm_size_bytes" "$WASM_SIZE" \
        "wasm_size_kb" "$WASM_SIZE_KB" \
        "output_dir" "$OUTPUT_DIR"
fi

exit 0
