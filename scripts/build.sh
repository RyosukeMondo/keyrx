#!/bin/bash
# Build script for keyrx workspace
# Orchestrates: WASM → UI → Daemon build sequence
# Supports: --release, --watch, --error, --json, --quiet, --log-file

# Get script directory for sourcing common.sh
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Source common utilities
# shellcheck source=lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"

# Script-specific variables
RELEASE_MODE=false
WATCH_MODE=false
START_TIME=0

# Usage information
usage() {
    cat << EOF
Usage: $(basename "$0") [OPTIONS]

Build the keyrx workspace in sequence: WASM → UI → Daemon

OPTIONS:
    --release       Build in release mode (optimized)
    --watch         Watch mode - rebuild on file changes (requires cargo-watch)
    --error         Show only errors
    --json          Output results in JSON format
    --quiet         Suppress non-error output
    --log-file PATH Specify custom log file path
    -h, --help      Show this help message

EXAMPLES:
    $(basename "$0")                    # Debug build (WASM → UI → Daemon)
    $(basename "$0") --release          # Release build (optimized)
    $(basename "$0") --watch            # Watch mode (debug)
    $(basename "$0") --watch --release  # Watch mode (release)
    $(basename "$0") --json             # JSON output

EXIT CODES:
    0 - Build succeeded
    1 - Build failed
    2 - Missing required tool

OUTPUT MARKERS:
    === accomplished === - Build succeeded
    === failed ===       - Build failed

BUILD SEQUENCE:
    1. WASM (keyrx_core → WebAssembly)
    2. UI (keyrx_ui_v2 with embedded WASM)
    3. Daemon (keyrx_daemon with embedded UI)
EOF
}

# Parse arguments
parse_args() {
    # Parse common flags first
    parse_common_flags "$@"

    # Parse script-specific flags from remaining args
    set -- "${REMAINING_ARGS[@]}"

    while [[ $# -gt 0 ]]; do
        case $1 in
            --release)
                RELEASE_MODE=true
                shift
                ;;
            --watch)
                WATCH_MODE=true
                shift
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done
}

# Helper function to format file size
format_size() {
    local size_bytes=$1
    if (( size_bytes < 1024 )); then
        echo "${size_bytes}B"
    elif (( size_bytes < 1048576 )); then
        echo "$(( size_bytes / 1024 ))KB"
    else
        echo "$(( size_bytes / 1048576 ))MB"
    fi
}

# Helper function to get file size
get_file_size() {
    local filepath=$1
    if [[ -f "$filepath" ]]; then
        stat -c%s "$filepath" 2>/dev/null || stat -f%z "$filepath" 2>/dev/null || echo "0"
    else
        echo "0"
    fi
}

# Build WASM module
build_wasm() {
    log_info "Building WASM module..."
    local wasm_start
    wasm_start=$(date +%s)

    if "$SCRIPT_DIR/build_wasm.sh"; then
        local wasm_end
        wasm_end=$(date +%s)
        local wasm_time=$((wasm_end - wasm_start))
        log_info "WASM build completed in ${wasm_time}s"
        return 0
    else
        log_error "WASM build failed"
        return 1
    fi
}

# Build UI
build_ui() {
    log_info "Building UI..."
    local ui_start
    ui_start=$(date +%s)

    if "$SCRIPT_DIR/build_ui.sh"; then
        local ui_end
        ui_end=$(date +%s)
        local ui_time=$((ui_end - ui_start))
        log_info "UI build completed in ${ui_time}s"
        return 0
    else
        log_error "UI build failed"
        return 1
    fi
}

# Build daemon
build_daemon() {
    local build_cmd="cargo build -p keyrx_daemon"

    if [[ "$RELEASE_MODE" == "true" ]]; then
        build_cmd="$build_cmd --release"
        log_info "Building daemon in release mode..."
    else
        log_info "Building daemon in debug mode..."
    fi

    local daemon_start
    daemon_start=$(date +%s)

    # Execute build
    if $build_cmd; then
        local daemon_end
        daemon_end=$(date +%s)
        local daemon_time=$((daemon_end - daemon_start))
        log_info "Daemon build completed in ${daemon_time}s"
        return 0
    else
        log_error "Daemon build failed"
        return 1
    fi
}

# Print build summary
print_summary() {
    local total_time=$1
    local build_dir="target/debug"

    if [[ "$RELEASE_MODE" == "true" ]]; then
        build_dir="target/release"
    fi

    local daemon_binary="$build_dir/keyrx_daemon"
    local wasm_file="keyrx_ui_v2/src/wasm/pkg/keyrx_core_bg.wasm"
    local ui_index="keyrx_ui_v2/dist/index.html"

    echo ""
    log_info "=== Build Summary ==="
    log_info "Total build time: ${total_time}s"
    echo ""

    # Daemon binary
    if [[ -f "$daemon_binary" ]]; then
        local daemon_size
        daemon_size=$(get_file_size "$daemon_binary")
        log_info "  Daemon: $(format_size "$daemon_size") ($daemon_binary)"
    else
        log_warn "  Daemon: not found"
    fi

    # WASM module
    if [[ -f "$wasm_file" ]]; then
        local wasm_size
        wasm_size=$(get_file_size "$wasm_file")
        log_info "  WASM:   $(format_size "$wasm_size") ($wasm_file)"
    else
        log_warn "  WASM:   not found"
    fi

    # UI
    if [[ -f "$ui_index" ]]; then
        log_info "  UI:     Built (keyrx_ui_v2/dist/)"
    else
        log_warn "  UI:     not found"
    fi

    echo ""
}

# Main build function - orchestrates complete build sequence
do_build() {
    START_TIME=$(date +%s)

    log_info "Starting build sequence: WASM → UI → Daemon"
    echo ""

    # Step 1: Build WASM
    if ! build_wasm; then
        log_failed
        return 1
    fi
    echo ""

    # Step 2: Build UI
    if ! build_ui; then
        log_failed
        return 1
    fi
    echo ""

    # Step 3: Build Daemon
    if ! build_daemon; then
        log_failed
        return 1
    fi

    local end_time
    end_time=$(date +%s)
    local total_time=$((end_time - START_TIME))

    echo ""
    print_summary "$total_time"

    log_accomplished
    return 0
}

# Watch mode function
do_watch() {
    # Check if cargo-watch is installed
    if ! require_tool "cargo-watch" "Install cargo-watch: cargo install cargo-watch"; then
        log_failed
        return 2
    fi

    local watch_cmd="cargo watch -x 'build --workspace"

    if [[ "$RELEASE_MODE" == "true" ]]; then
        watch_cmd="$watch_cmd --release"
        log_info "Starting watch mode (release)..."
    else
        log_info "Starting watch mode (debug)..."
    fi

    watch_cmd="$watch_cmd'"

    # Execute watch (this blocks until interrupted)
    eval "$watch_cmd"

    # If we get here, watch was interrupted (Ctrl+C)
    return 0
}

# Main execution
main() {
    local exit_code=0
    local build_type="debug"
    local mode="standard"

    # Parse arguments
    parse_args "$@"

    # Setup log file if not provided via --log-file
    if [[ -z "$LOG_FILE" ]]; then
        setup_log_file "build"
    fi

    # Verify cargo is installed
    if ! require_tool "cargo" "Install Rust from https://rustup.rs"; then
        if [[ "$JSON_MODE" == "true" ]]; then
            output_json "status" "failed" "error" "cargo not found" "exit_code" "2"
        fi
        exit 2
    fi

    separator

    # Determine build type and mode
    if [[ "$RELEASE_MODE" == "true" ]]; then
        build_type="release"
    fi

    if [[ "$WATCH_MODE" == "true" ]]; then
        mode="watch"
    fi

    # Execute appropriate mode
    if [[ "$WATCH_MODE" == "true" ]]; then
        do_watch
        exit_code=$?
    else
        do_build
        exit_code=$?
    fi

    separator

    # JSON output
    if [[ "$JSON_MODE" == "true" ]]; then
        local build_dir="target/debug"
        if [[ "$RELEASE_MODE" == "true" ]]; then
            build_dir="target/release"
        fi

        local daemon_size=0
        local wasm_size=0
        if [[ -f "$build_dir/keyrx_daemon" ]]; then
            daemon_size=$(get_file_size "$build_dir/keyrx_daemon")
        fi
        if [[ -f "keyrx_ui_v2/src/wasm/pkg/keyrx_core_bg.wasm" ]]; then
            wasm_size=$(get_file_size "keyrx_ui_v2/src/wasm/pkg/keyrx_core_bg.wasm")
        fi

        local total_time=0
        if [[ $START_TIME -gt 0 ]]; then
            local end_time
            end_time=$(date +%s)
            total_time=$((end_time - START_TIME))
        fi

        if [[ $exit_code -eq 0 ]]; then
            output_json \
                "status" "success" \
                "build_type" "$build_type" \
                "mode" "$mode" \
                "total_time_seconds" "$total_time" \
                "daemon_size_bytes" "$daemon_size" \
                "wasm_size_bytes" "$wasm_size" \
                "exit_code" "0"
        else
            output_json \
                "status" "failed" \
                "build_type" "$build_type" \
                "mode" "$mode" \
                "exit_code" "$exit_code"
        fi
    fi

    exit $exit_code
}

# Run main function
main "$@"
