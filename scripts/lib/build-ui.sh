#!/bin/bash
# Build keyrx UI frontend with WASM dependencies
#
# Purpose:
#   Builds the complete frontend application including WASM module
#   Clears previous build artifacts and creates fresh production build
#   Outputs to keyrx_ui/dist/ for embedding in daemon
#
# Dependencies:
#   - Node.js 18+ and npm
#   - wasm-pack (for WASM build step)
#   - scripts/build_wasm.sh
#
# Usage:
#   ./scripts/build_ui.sh [--quiet] [--json] [--log-file PATH]
#
# Exit codes:
#   0 - Build successful
#   1 - Missing dependencies or build failure

set -euo pipefail

# Source common utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=common.sh
source "$SCRIPT_DIR/common.sh"

# Store the project root directory (go up two levels: lib/ -> scripts/ -> project root)
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Parse command line arguments
parse_common_flags "$@"
setup_log_file "build_ui"

# Convert LOG_FILE to absolute path if it's relative
if [[ -n "$LOG_FILE" ]] && [[ ! "$LOG_FILE" = /* ]]; then
    LOG_FILE="$PROJECT_ROOT/$LOG_FILE"
fi

log_info "Building keyrx UI frontend..."
separator

# Check if Node.js is installed
if ! command_exists node; then
    log_error "Node.js is not installed"
    log_error "Install it from: https://nodejs.org/"
    log_failed
    exit 1
fi

# Check if npm is installed
if ! command_exists npm; then
    log_error "npm is not installed"
    log_error "Install it with Node.js from: https://nodejs.org/"
    log_failed
    exit 1
fi

log_info "Node.js found: $(node --version)"
log_info "npm found: $(npm --version)"

# Define paths
UI_DIR="$PROJECT_ROOT/keyrx_ui"
DIST_DIR="$UI_DIR/dist"

if [[ ! -d "$UI_DIR" ]]; then
    log_error "UI directory not found: $UI_DIR"
    log_failed
    exit 1
fi

# Step 1: Build WASM first
log_info "Step 1/4: Building WASM module..."
separator

if "$SCRIPT_DIR/build-wasm.sh" "$@"; then
    log_info "WASM build completed successfully"
else
    log_error "WASM build failed"
    log_error "UI build cannot continue without WASM"
    log_failed
    exit 1
fi

separator

# Step 2: Install dependencies if needed
log_info "Step 2/4: Checking dependencies..."
cd "$UI_DIR"

if [[ ! -d "node_modules" ]]; then
    log_info "node_modules not found, installing dependencies..."
    if npm install; then
        log_info "Dependencies installed successfully"
    else
        log_error "npm install failed"
        log_failed
        exit 1
    fi
else
    log_debug "node_modules already exists, skipping install"
fi

separator

# Step 3: Clear previous build
log_info "Step 3/4: Clearing previous build..."

if [[ -d "$DIST_DIR" ]]; then
    log_debug "Removing $DIST_DIR..."
    rm -rf "$DIST_DIR"
    log_info "Previous build cleared"
else
    log_debug "No previous build to clear"
fi

separator

# Step 4: Build UI
log_info "Step 4/4: Building UI application..."

# Record build start time
BUILD_START=$(date +%s)

# Run the build
if npm run build; then
    log_info "UI build completed successfully"
else
    log_error "npm run build failed"
    log_failed
    exit 1
fi

# Record build end time
BUILD_END=$(date +%s)
BUILD_TIME=$((BUILD_END - BUILD_START))

separator

# Verify output exists
log_info "Verifying build output..."

if [[ ! -f "$DIST_DIR/index.html" ]]; then
    log_error "Build verification failed - index.html not found"
    log_error "Expected at: $DIST_DIR/index.html"
    log_failed
    exit 1
fi

log_debug "Found: $DIST_DIR/index.html"

# Calculate bundle size
log_info "Calculating bundle size..."

# Total size (uncompressed)
if command_exists du; then
    TOTAL_SIZE=$(du -sh "$DIST_DIR" | cut -f1)
    TOTAL_SIZE_KB=$(du -sk "$DIST_DIR" | cut -f1)
    log_info "Total bundle size: $TOTAL_SIZE"
else
    TOTAL_SIZE="unknown"
    TOTAL_SIZE_KB=0
    log_warn "du command not found, cannot calculate bundle size"
fi

# Gzipped size (estimate using gzip if available)
if command_exists gzip && command_exists find; then
    # Create temporary directory for gzip calculation
    TEMP_DIR=$(mktemp -d)
    trap 'rm -rf "$TEMP_DIR"' EXIT

    # Copy and gzip all assets
    cp -r "$DIST_DIR"/* "$TEMP_DIR/" 2>/dev/null || true
    find "$TEMP_DIR" -type f -not -name "*.gz" -exec gzip -9 {} \; 2>/dev/null || true

    # Calculate total gzipped size
    GZIP_SIZE_KB=$(du -sk "$TEMP_DIR" | cut -f1)
    GZIP_SIZE_MB=$((GZIP_SIZE_KB / 1024))

    log_info "Estimated gzipped size: ${GZIP_SIZE_KB} KB (~${GZIP_SIZE_MB} MB)"

    # Check bundle size limits (as per requirements)
    # Main bundle should be < 500KB gzipped
    if [[ $GZIP_SIZE_MB -ge 1 ]]; then
        log_warn "Total gzipped size may exceed recommended limits"
    fi
else
    GZIP_SIZE_KB=0
    log_debug "gzip or find not available, skipping gzipped size calculation"
fi

# List key asset files
log_info "Build artifacts:"
if command_exists find; then
    find "$DIST_DIR" -type f -name "*.js" -o -name "*.css" -o -name "*.wasm" | while read -r file; do
        if command_exists stat; then
            SIZE=$(stat -c%s "$file" 2>/dev/null || stat -f%z "$file" 2>/dev/null || echo "unknown")
            if [[ "$SIZE" != "unknown" ]]; then
                SIZE_KB=$((SIZE / 1024))
                FILENAME=$(basename "$file")
                log_debug "  - $FILENAME: ${SIZE_KB} KB"
            fi
        fi
    done
fi

separator
log_info "Build time: ${BUILD_TIME} seconds"
log_accomplished

# Output JSON if requested
if [[ "$JSON_MODE" == "true" ]]; then
    output_json \
        "status" "success" \
        "build_time_seconds" "$BUILD_TIME" \
        "total_size_kb" "$TOTAL_SIZE_KB" \
        "gzipped_size_kb" "$GZIP_SIZE_KB" \
        "dist_dir" "$DIST_DIR"
fi

exit 0
