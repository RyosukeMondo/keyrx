#!/bin/bash
# Build keyrx_core to WebAssembly using wasm-pack
#
# Purpose:
#   Compiles keyrx_core Rust crate to WASM for browser use
#   Outputs to keyrx_ui/src/wasm/pkg/ for frontend integration
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
# shellcheck source=common.sh
source "$SCRIPT_DIR/common.sh"

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
OUTPUT_DIR="$PROJECT_ROOT/keyrx_ui/src/wasm/pkg"

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

if [[ "$WASM_SIZE" == "unknown" ]]; then
    log_error "Could not determine WASM file size"
    log_failed
    exit 1
fi

WASM_SIZE_KB=$((WASM_SIZE / 1024))
WASM_SIZE_MB=$((WASM_SIZE_KB / 1024))

log_info "WASM file size: ${WASM_SIZE_KB} KB (${WASM_SIZE_MB}.$(( (WASM_SIZE_KB % 1024) * 100 / 1024 )) MB)"

# Verify WASM size is reasonable (> 100KB)
MIN_SIZE_KB=100
if [[ $WASM_SIZE_KB -lt $MIN_SIZE_KB ]]; then
    log_error "WASM file size too small: ${WASM_SIZE_KB} KB < ${MIN_SIZE_KB} KB"
    log_error "Build may have failed or produced invalid output"
    log_failed
    exit 1
fi

# Check if WASM size is under 1MB (as per requirements)
if [[ $WASM_SIZE_MB -ge 1 ]]; then
    log_warn "WASM file size exceeds 1MB target"
fi

# Generate SHA256 hash of WASM file
log_info "Generating WASM hash..."
if command -v sha256sum >/dev/null 2>&1; then
    WASM_HASH=$(sha256sum "$WASM_FILE" | awk '{print $1}')
elif command -v shasum >/dev/null 2>&1; then
    WASM_HASH=$(shasum -a 256 "$WASM_FILE" | awk '{print $1}')
else
    log_error "Neither sha256sum nor shasum found - cannot generate hash"
    log_failed
    exit 1
fi

log_info "WASM hash: $WASM_HASH"

# Extract version from workspace Cargo.toml
WORKSPACE_TOML="$PROJECT_ROOT/Cargo.toml"
if [[ -f "$WORKSPACE_TOML" ]]; then
    KEYRX_VERSION=$(grep -E '^version\s*=' "$WORKSPACE_TOML" | head -1 | sed -E 's/.*version\s*=\s*"([^"]+)".*/\1/')
    log_info "keyrx_core version: $KEYRX_VERSION"
else
    log_warn "Could not find workspace Cargo.toml - version unknown"
    KEYRX_VERSION="unknown"
fi

# Write manifest file for version tracking
MANIFEST_FILE="$OUTPUT_DIR/wasm-manifest.json"
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

log_info "Writing WASM manifest to $MANIFEST_FILE..."
cat > "$MANIFEST_FILE" <<EOF
{
  "version": "$KEYRX_VERSION",
  "hash": "$WASM_HASH",
  "size": $WASM_SIZE,
  "size_kb": $WASM_SIZE_KB,
  "timestamp": "$TIMESTAMP",
  "file": "keyrx_core_bg.wasm"
}
EOF

if [[ ! -f "$MANIFEST_FILE" ]]; then
    log_error "Failed to write manifest file"
    log_failed
    exit 1
fi

log_info "WASM manifest created successfully"

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
        "wasm_hash" "$WASM_HASH" \
        "version" "$KEYRX_VERSION" \
        "manifest_file" "$MANIFEST_FILE" \
        "output_dir" "$OUTPUT_DIR"
fi

exit 0
