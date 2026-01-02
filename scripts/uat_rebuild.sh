#!/usr/bin/env bash
#
# UAT Rebuild Script - Complete rebuild and restart cycle
#
# This script performs a complete clean rebuild of KeyRX:
# 1. Stops all running daemons
# 2. Cleans build artifacts
# 3. Rebuilds UI (WASM + Vite)
# 4. Rebuilds daemon with embedded UI
# 5. Starts daemon with system tray
#

set -euo pipefail

# Import common functions
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# shellcheck source=./lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"

# Configuration
DAEMON_BIN="$PROJECT_ROOT/target/debug/keyrx_daemon"
CONFIG_FILE="$PROJECT_ROOT/user_layout.krx"
UI_DIR="$PROJECT_ROOT/keyrx_ui"

# ============================================================================
# Helper Functions
# ============================================================================

stop_daemon() {
    log_info "Stopping any running daemons..."
    pkill -9 keyrx_daemon 2>/dev/null || true
    sleep 2

    if pgrep -x keyrx_daemon > /dev/null; then
        log_error "Failed to stop daemon"
        exit 1
    fi

    log_success "All daemons stopped"
}

clean_build() {
    log_info "Cleaning build artifacts..."

    # Clean UI
    cd "$UI_DIR"
    rm -rf dist node_modules/.vite
    log_success "UI artifacts cleaned"

    # Clean daemon
    cd "$PROJECT_ROOT"
    cargo clean -p keyrx_daemon
    rm -rf target/debug/build/keyrx_daemon-*
    log_success "Daemon artifacts cleaned"
}

build_ui() {
    log_info "Building Web UI..."
    cd "$UI_DIR"

    # Install dependencies if needed
    if [ ! -d "node_modules" ]; then
        log_info "Installing npm dependencies..."
        npm install
    fi

    # Build WASM
    log_info "Building WASM module..."
    npm run build:wasm

    # Build UI
    log_info "Building production bundle..."
    npx vite build

    # Verify build
    if [ ! -f "dist/index.html" ]; then
        log_error "UI build failed - dist/index.html not found"
        exit 1
    fi

    log_success "UI built successfully"
}

build_daemon() {
    log_info "Building daemon with embedded UI..."
    cd "$PROJECT_ROOT"

    # Touch static_files.rs to force re-embedding UI
    touch keyrx_daemon/src/web/static_files.rs

    # Build daemon
    cargo build --bin keyrx_daemon

    # Verify build
    if [ ! -f "$DAEMON_BIN" ]; then
        log_error "Daemon build failed"
        exit 1
    fi

    log_success "Daemon built successfully"
}

start_daemon() {
    log_info "Starting daemon..."
    cd "$PROJECT_ROOT"

    # Auto-detect X11 display
    export GDK_BACKEND=x11
    if [ -z "${DISPLAY:-}" ]; then
        if [ -S /tmp/.X11-unix/X1 ]; then
            export DISPLAY=:1
        elif [ -S /tmp/.X11-unix/X0 ]; then
            export DISPLAY=:0
        else
            log_warn "No X11 display found, system tray may not work"
            export DISPLAY=:0
        fi
        log_info "Auto-detected DISPLAY=$DISPLAY"
    fi

    # Start daemon in background
    "$DAEMON_BIN" run --config "$CONFIG_FILE" > /tmp/keyrx_daemon.log 2>&1 &
    DAEMON_PID=$!

    sleep 3

    # Verify daemon started
    if ! ps -p $DAEMON_PID > /dev/null; then
        log_error "Daemon failed to start. Check /tmp/keyrx_daemon.log"
        tail -20 /tmp/keyrx_daemon.log
        exit 1
    fi

    log_success "Daemon started (PID: $DAEMON_PID)"
    log_info "Web UI: http://localhost:9867"
    log_info "Logs: /tmp/keyrx_daemon.log"
}

show_version() {
    sleep 2
    log_info "Checking version..."
    curl -s http://localhost:9867/api/version | python3 -m json.tool || true
}

# ============================================================================
# Main
# ============================================================================

main() {
    log_header "KeyRX UAT - Complete Rebuild"

    stop_daemon
    clean_build
    build_ui
    build_daemon
    start_daemon
    show_version

    echo ""
    log_success "=== UAT Complete ==="
    log_info "Daemon is running in the background"
    log_info "Web UI: http://localhost:9867"
    log_info "Check system tray for keyboard icon"
    log_info ""
    log_info "To stop: pkill keyrx_daemon"
    log_info "To view logs: tail -f /tmp/keyrx_daemon.log"
}

main "$@"
