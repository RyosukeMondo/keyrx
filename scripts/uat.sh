#!/bin/bash
# uat.sh - Unified User Acceptance Testing
#
# Consolidates: UAT.sh, uat_linux.sh, uat_windows.sh, uat_rebuild.sh, verify_uat.sh
#
# Usage:
#   ./scripts/uat.sh              # Full UAT: build UI, build daemon, start daemon
#   ./scripts/uat.sh --verify     # Verify running daemon (web UI, tray, etc.)
#   ./scripts/uat.sh --rebuild    # Force clean rebuild before starting
#   ./scripts/uat.sh --stop       # Stop running daemon
#   ./scripts/uat.sh --headless   # Run without opening browser
#   ./scripts/uat.sh --release    # Build in release mode
#
# This script ALWAYS builds the UI before starting the daemon to ensure
# the web interface is up-to-date.

# Get script directory for sourcing common.sh
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Source common utilities
# shellcheck source=lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"

# Script-specific variables
VERIFY_ONLY=false
REBUILD=false
STOP_ONLY=false
HEADLESS=false
RELEASE=false
DEBUG_MODE=false
SKIP_WASM=false
DAEMON_PID=""
DAEMON_NAME="keyrx_daemon"

# Configuration
RHAI_FILE="$PROJECT_ROOT/examples/user_layout.rhai"
KRX_FILE="$PROJECT_ROOT/user_layout.krx"
WEB_UI_URL="http://localhost:9867"

# Usage information
usage() {
    cat << EOF
Usage: $(basename "$0") [OPTIONS]

Unified User Acceptance Testing for KeyRX daemon.

This script ALWAYS builds the UI before starting the daemon to ensure
the web interface is up-to-date (unless --verify or --stop is used).

OPTIONS:
    --verify        Only verify running daemon (don't build/start)
    --rebuild       Force clean rebuild before starting
    --stop          Stop running daemon and exit
    --headless      Don't open browser automatically
    --release       Build in release mode
    --debug         Enable debug logging
    --skip-wasm     Skip WASM build and verification (for quick UAT)
    --error         Show only errors
    --json          Output results in JSON format
    --quiet         Suppress non-error output
    --log-file PATH Specify custom log file path
    -h, --help      Show this help message

EXAMPLES:
    $(basename "$0")                    # Full UAT with UI build
    $(basename "$0") --rebuild          # Clean rebuild + UAT
    $(basename "$0") --verify           # Verify running daemon
    $(basename "$0") --stop             # Stop daemon
    $(basename "$0") --release          # Release build UAT
    $(basename "$0") --headless --debug # Headless with debug logging

BUILD SEQUENCE:
    1. Check prerequisites (groups, udev, modules)
    2. Compile user_layout.rhai to .krx
    3. Build WASM module
    4. Verify WASM (hash, version matching)
    5. Build Web UI (Vite production build)
    6. Build daemon with embedded UI
    7. Stop existing daemon
    8. Start daemon with system tray
    9. Open browser (unless --headless)

EXIT CODES:
    0 - UAT completed successfully
    1 - UAT failed
    2 - Missing required tool

OUTPUT MARKERS:
    === accomplished === - UAT succeeded
    === failed ===       - UAT failed
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
            --verify)
                VERIFY_ONLY=true
                shift
                ;;
            --rebuild)
                REBUILD=true
                shift
                ;;
            --stop)
                STOP_ONLY=true
                shift
                ;;
            --headless)
                HEADLESS=true
                shift
                ;;
            --release)
                RELEASE=true
                shift
                ;;
            --debug)
                DEBUG_MODE=true
                shift
                ;;
            --skip-wasm)
                SKIP_WASM=true
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

# Check if daemon is running
check_daemon_running() {
    pgrep -f "$DAEMON_NAME" > /dev/null 2>&1
}

# Stop daemon
stop_daemon() {
    log_info "Stopping keyrx daemon..."

    if ! check_daemon_running; then
        log_info "Daemon is not running"
        return 0
    fi

    pkill -f "$DAEMON_NAME" || true
    sleep 1

    if check_daemon_running; then
        log_warn "Daemon still running, force killing..."
        pkill -9 -f "$DAEMON_NAME" || true
        sleep 1
    fi

    if check_daemon_running; then
        log_error "Failed to stop daemon"
        return 1
    fi

    log_info "Daemon stopped successfully"
    return 0
}

# Check Linux prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."

    # Check if user is in input group
    if ! groups | grep -q "input"; then
        log_error "User not in 'input' group"
        log_error "Fix: sudo usermod -aG input \$USER && logout"
        return 1
    fi
    log_info "User in 'input' group"

    # Check if uinput group exists and user is in it
    if getent group uinput > /dev/null; then
        if ! groups | grep -q "uinput"; then
            log_warn "User not in 'uinput' group (may cause permission errors)"
            log_warn "Fix: sudo usermod -aG uinput \$USER && logout"
        else
            log_info "User in 'uinput' group"
        fi
    fi

    # Check if udev rules are installed
    if [[ ! -f "/etc/udev/rules.d/99-keyrx.rules" ]]; then
        log_warn "udev rules not installed"
        log_warn "Fix: sudo cp keyrx_daemon/udev/99-keyrx.rules /etc/udev/rules.d/"
    fi

    # Check if uinput module is loaded
    if ! lsmod | grep -q "uinput"; then
        log_warn "uinput module not loaded, attempting to load..."
        if sudo modprobe uinput 2>/dev/null; then
            log_info "uinput module loaded"
        else
            log_error "Failed to load uinput module"
            return 1
        fi
    fi
    log_info "uinput module loaded"

    log_info "Prerequisites OK"
    return 0
}

# Compile layout file
compile_layout() {
    log_info "Compiling layout: $RHAI_FILE -> $KRX_FILE"

    if [[ ! -f "$RHAI_FILE" ]]; then
        log_error "Layout file not found: $RHAI_FILE"
        return 1
    fi

    cd "$PROJECT_ROOT"
    if ! cargo run --bin keyrx_compiler --quiet -- compile "$RHAI_FILE" -o "$KRX_FILE"; then
        log_error "Layout compilation failed"
        return 1
    fi

    if [[ ! -f "$KRX_FILE" ]]; then
        log_error "Failed to create .krx file"
        return 1
    fi

    log_info "Layout compiled successfully"
    return 0
}

# Build WASM module
build_wasm() {
    if [[ "$SKIP_WASM" == "true" ]]; then
        log_info "Skipping WASM build (--skip-wasm flag)"
        return 0
    fi

    log_info "Building WASM module..."

    if [[ -f "$SCRIPT_DIR/lib/build-wasm.sh" ]]; then
        if ! "$SCRIPT_DIR/lib/build-wasm.sh"; then
            log_error "WASM build failed"
            return 1
        fi
    elif [[ -f "$SCRIPT_DIR/build_wasm.sh" ]]; then
        if ! "$SCRIPT_DIR/build_wasm.sh"; then
            log_error "WASM build failed"
            return 1
        fi
    else
        log_error "WASM build script not found"
        return 1
    fi

    log_info "WASM build completed"
    return 0
}

# Verify WASM module
verify_wasm() {
    if [[ "$SKIP_WASM" == "true" ]]; then
        log_info "Skipping WASM verification (--skip-wasm flag)"
        return 0
    fi

    log_info "Verifying WASM module..."

    # Get hash before verification
    local manifest_file="$PROJECT_ROOT/keyrx_ui/src/wasm/pkg/wasm-manifest.json"
    local hash_before=""
    if [[ -f "$manifest_file" ]]; then
        hash_before=$(jq -r '.hash' "$manifest_file" 2>/dev/null || echo "")
    fi

    # Run verification script
    if [[ -f "$SCRIPT_DIR/verify-wasm.sh" ]]; then
        if ! "$SCRIPT_DIR/verify-wasm.sh" --quiet; then
            log_error "WASM verification failed"
            log_error "This likely means the WASM module is corrupted or has version mismatches"
            log_error "Try running: npm run rebuild:wasm"
            return 1
        fi
    else
        log_warn "WASM verification script not found - skipping verification"
        return 0
    fi

    # Check if hash changed (informational only)
    if [[ -f "$manifest_file" ]]; then
        local hash_after
        hash_after=$(jq -r '.hash' "$manifest_file" 2>/dev/null || echo "")

        if [[ -n "$hash_before" ]] && [[ -n "$hash_after" ]] && [[ "$hash_before" != "$hash_after" ]]; then
            log_info "WASM hash changed: $hash_before -> $hash_after"
        fi
    fi

    log_info "WASM verification passed"
    return 0
}

# Build Web UI
build_ui() {
    log_info "Building Web UI..."

    cd "$PROJECT_ROOT/keyrx_ui"

    # Install dependencies if needed
    if [[ ! -d "node_modules" ]]; then
        log_info "Installing npm dependencies..."
        npm install --silent
    fi

    # Build production bundle
    log_info "Building production bundle..."
    if ! npx vite build; then
        log_error "UI build failed"
        return 1
    fi

    if [[ ! -f "dist/index.html" ]]; then
        log_error "UI build failed - dist/index.html not found"
        return 1
    fi

    log_info "UI build completed"
    cd "$PROJECT_ROOT"
    return 0
}

# Build daemon
build_daemon() {
    local build_cmd="cargo build --bin keyrx_daemon"
    local build_type="debug"

    if [[ "$RELEASE" == "true" ]]; then
        build_cmd="$build_cmd --release"
        build_type="release"
        log_info "Building daemon in release mode..."
    else
        log_info "Building daemon in debug mode..."
    fi

    cd "$PROJECT_ROOT"

    # Touch static_files.rs to force re-embedding UI
    if [[ -f "keyrx_daemon/src/web/static_files.rs" ]]; then
        touch "keyrx_daemon/src/web/static_files.rs"
    fi

    if ! $build_cmd; then
        log_error "Daemon build failed"
        return 1
    fi

    local daemon_binary="$PROJECT_ROOT/target/$build_type/keyrx_daemon"
    if [[ ! -f "$daemon_binary" ]]; then
        log_error "Daemon binary not found: $daemon_binary"
        return 1
    fi

    log_info "Daemon build completed"
    return 0
}

# Clean build artifacts
clean_build() {
    log_info "Cleaning build artifacts..."

    cd "$PROJECT_ROOT/keyrx_ui"
    rm -rf dist node_modules/.vite
    log_info "UI artifacts cleaned"

    cd "$PROJECT_ROOT"
    cargo clean -p keyrx_daemon 2>/dev/null || true
    rm -rf target/debug/build/keyrx_daemon-* 2>/dev/null || true
    log_info "Daemon artifacts cleaned"

    return 0
}

# Start daemon
start_daemon() {
    local build_type="debug"
    if [[ "$RELEASE" == "true" ]]; then
        build_type="release"
    fi

    local daemon_binary="$PROJECT_ROOT/target/$build_type/keyrx_daemon"

    log_info "Starting daemon: $daemon_binary"
    log_info "Config: $KRX_FILE"

    # Set up X11 environment
    export GDK_BACKEND=x11
    if [[ -z "${DISPLAY:-}" ]]; then
        if [[ -S /tmp/.X11-unix/X1 ]]; then
            export DISPLAY=:1
        elif [[ -S /tmp/.X11-unix/X0 ]]; then
            export DISPLAY=:0
        else
            log_warn "No X11 display found, system tray may not work"
            export DISPLAY=:0
        fi
        log_info "Auto-detected DISPLAY=$DISPLAY"
    fi

    # Build daemon arguments
    local daemon_args=("run" "--config" "$KRX_FILE")
    if [[ "$DEBUG_MODE" == "true" ]]; then
        daemon_args+=("--debug")
    fi

    # Start daemon
    "$daemon_binary" "${daemon_args[@]}" > /tmp/keyrx_daemon.log 2>&1 &
    DAEMON_PID=$!

    sleep 3

    # Verify daemon started
    if ! kill -0 "$DAEMON_PID" 2>/dev/null; then
        log_error "Daemon failed to start"
        log_error "Last 20 lines of log:"
        tail -20 /tmp/keyrx_daemon.log 2>/dev/null || true
        return 1
    fi

    log_info "Daemon started (PID: $DAEMON_PID)"
    log_info "Web UI: $WEB_UI_URL"
    log_info "Logs: /tmp/keyrx_daemon.log"

    # Open browser if not headless
    if [[ "$HEADLESS" == "false" ]]; then
        sleep 1
        if command -v xdg-open &> /dev/null; then
            log_info "Opening browser..."
            xdg-open "$WEB_UI_URL" &> /dev/null &
        fi
    fi

    return 0
}

# Verify running daemon
verify_daemon() {
    log_info "Verifying daemon..."

    local checks_passed=0
    local checks_failed=0

    # Check 1: Daemon running
    log_info "Check 1: Daemon status"
    if check_daemon_running; then
        local pid
        pid=$(pgrep -f "$DAEMON_NAME")
        log_info "  PASS: Daemon running (PID: $pid)"
        ((checks_passed++))
    else
        log_error "  FAIL: Daemon not running"
        ((checks_failed++))
    fi

    # Check 2: Web UI accessible
    log_info "Check 2: Web UI accessibility"
    if curl -s -f "$WEB_UI_URL/" > /dev/null 2>&1; then
        log_info "  PASS: Web UI accessible at $WEB_UI_URL"
        ((checks_passed++))
    else
        log_error "  FAIL: Web UI not accessible"
        ((checks_failed++))
    fi

    # Check 3: React root element
    log_info "Check 3: React app loaded"
    local html
    if html=$(curl -s -f "$WEB_UI_URL/" 2>&1); then
        if echo "$html" | grep -q '<div id="root">'; then
            log_info "  PASS: React root element found"
            ((checks_passed++))
        else
            log_error "  FAIL: React root element missing"
            ((checks_failed++))
        fi
    else
        log_error "  FAIL: Could not fetch HTML"
        ((checks_failed++))
    fi

    # Check 4: Script bundles
    log_info "Check 4: JavaScript bundles"
    if echo "$html" | grep -q '<script.*src='; then
        local script_count
        script_count=$(echo "$html" | grep -c '<script.*src=' || true)
        log_info "  PASS: $script_count script bundles found"
        ((checks_passed++))
    else
        log_error "  FAIL: No script bundles found"
        ((checks_failed++))
    fi

    # Summary
    separator
    log_info "Verification Summary"
    log_info "  Passed: $checks_passed"
    log_info "  Failed: $checks_failed"

    if [[ $checks_failed -eq 0 ]]; then
        log_accomplished
        return 0
    else
        log_failed
        return 1
    fi
}

# Full UAT sequence
run_full_uat() {
    local step=1
    local total_steps=8

    log_info "Starting full UAT sequence"
    separator

    # Step 1: Check prerequisites
    log_info "Step $step/$total_steps: Checking prerequisites"
    if ! check_prerequisites; then
        log_failed
        return 1
    fi
    ((step++))
    separator

    # Step 2: Compile layout
    log_info "Step $step/$total_steps: Compiling layout"
    if ! compile_layout; then
        log_failed
        return 1
    fi
    ((step++))
    separator

    # Step 3: Build WASM
    log_info "Step $step/$total_steps: Building WASM"
    if ! build_wasm; then
        log_failed
        return 1
    fi
    ((step++))
    separator

    # Step 4: Verify WASM
    log_info "Step $step/$total_steps: Verifying WASM"
    if ! verify_wasm; then
        log_error "WASM verification failed - aborting UAT"
        log_failed
        return 1
    fi
    ((step++))
    separator

    # Step 5: Build UI
    log_info "Step $step/$total_steps: Building Web UI"
    if ! build_ui; then
        log_failed
        return 1
    fi
    ((step++))
    separator

    # Step 6: Build daemon
    log_info "Step $step/$total_steps: Building daemon"
    if ! build_daemon; then
        log_failed
        return 1
    fi
    ((step++))
    separator

    # Step 7: Stop existing daemon
    log_info "Step $step/$total_steps: Stopping existing daemon"
    stop_daemon || true
    ((step++))
    separator

    # Step 8: Start daemon
    log_info "Step $step/$total_steps: Starting daemon"
    if ! start_daemon; then
        log_failed
        return 1
    fi
    separator

    log_accomplished
    echo ""
    log_info "UAT Complete!"
    log_info "  Web UI: $WEB_UI_URL"
    log_info "  Logs: /tmp/keyrx_daemon.log"
    log_info "  Stop: ./scripts/uat.sh --stop"

    return 0
}

# Main execution
main() {
    local exit_code=0

    # Parse arguments
    parse_args "$@"

    # Setup log file if not provided via --log-file
    if [[ -z "$LOG_FILE" ]]; then
        setup_log_file "uat"
    fi

    # Verify cargo is installed
    if ! require_tool "cargo" "Install Rust from https://rustup.rs"; then
        if [[ "$JSON_MODE" == "true" ]]; then
            output_json "status" "failed" "error" "cargo not found" "exit_code" "2"
        fi
        exit 2
    fi

    separator
    log_info "KeyRX User Acceptance Testing"
    separator

    # Handle different modes
    if [[ "$STOP_ONLY" == "true" ]]; then
        if stop_daemon; then
            log_accomplished
        else
            log_failed
            exit_code=1
        fi
    elif [[ "$VERIFY_ONLY" == "true" ]]; then
        if verify_daemon; then
            exit_code=0
        else
            exit_code=1
        fi
    else
        # Full UAT sequence
        if [[ "$REBUILD" == "true" ]]; then
            log_info "Clean rebuild requested"
            clean_build
        fi

        if run_full_uat; then
            exit_code=0
        else
            exit_code=1
        fi
    fi

    # JSON output
    if [[ "$JSON_MODE" == "true" ]]; then
        local status="success"
        if [[ $exit_code -ne 0 ]]; then
            status="failed"
        fi
        output_json \
            "status" "$status" \
            "daemon_pid" "${DAEMON_PID:-}" \
            "web_ui_url" "$WEB_UI_URL" \
            "exit_code" "$exit_code"
    fi

    exit $exit_code
}

# Run main function
main "$@"
