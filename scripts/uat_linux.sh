#!/usr/bin/env bash
# uat_linux.sh - User Acceptance Test setup for Linux
#
# Usage: ./scripts/uat_linux.sh [--headless] [--debug] [--release]
#
# This script:
# 1. Compiles user_layout.rhai to .krx binary
# 2. Builds daemon with web UI embedded
# 3. Launches daemon with system tray + web UI
# 4. Opens browser to web interface (unless --headless)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

# Parse flags
HEADLESS=false
DEBUG=false
RELEASE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --headless)
            HEADLESS=true
            shift
            ;;
        --debug)
            DEBUG=true
            shift
            ;;
        --release)
            RELEASE=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [--headless] [--debug] [--release]"
            echo ""
            echo "Options:"
            echo "  --headless   Don't open browser (daemon runs in foreground)"
            echo "  --debug      Enable debug logging"
            echo "  --release    Build and run in release mode"
            echo ""
            echo "This script performs full UAT testing:"
            echo "  0. Checks prerequisites (groups, udev, modules)"
            echo "  1. Compiles user_layout.rhai to .krx"
            echo "  2. Builds daemon with web UI"
            echo "  3. Launches daemon with system tray"
            echo "  4. Opens web UI in browser"
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

cd "$PROJECT_ROOT"

# ============================================================
# STEP 0: Check Prerequisites
# ============================================================
log_step "0/4 - Checking prerequisites"

# Check if user is in input and uinput groups
if ! groups | grep -q "input"; then
    log_error "User not in 'input' group"
    echo ""
    echo "To fix this, run:"
    echo "  sudo usermod -aG input \$USER"
    echo "  # Log out and log back in"
    exit 1
fi

if ! groups | grep -q "uinput"; then
    log_warn "User not in 'uinput' group (may cause permission errors)"
    echo ""
    echo "To fix this, run:"
    echo "  sudo groupadd -f uinput"
    echo "  sudo usermod -aG uinput \$USER"
    echo "  # Log out and log back in"
    echo ""
    echo "Press Enter to continue anyway, or Ctrl+C to abort..."
    read -r
fi

# Check if udev rules are installed
if [[ ! -f "/etc/udev/rules.d/99-keyrx.rules" ]]; then
    log_warn "udev rules not installed"
    echo ""
    echo "To install udev rules, run:"
    echo "  sudo cp keyrx_daemon/udev/99-keyrx.rules /etc/udev/rules.d/"
    echo "  sudo udevadm control --reload-rules"
    echo "  sudo udevadm trigger"
    echo ""
    echo "Press Enter to continue anyway, or Ctrl+C to abort..."
    read -r
fi

# Check if uinput module is loaded
if ! lsmod | grep -q "uinput"; then
    log_warn "uinput module not loaded, attempting to load..."
    if sudo modprobe uinput 2>/dev/null; then
        log_info "✓ uinput module loaded"
    else
        log_error "Failed to load uinput module (may need root)"
        echo "Try: sudo modprobe uinput"
        exit 1
    fi
fi

log_info "✓ Prerequisites checked"

# ============================================================
# STEP 1: Compile user_layout.rhai to .krx
# ============================================================
log_step "1/4 - Compiling user_layout.rhai to user_layout.krx"

RHAI_FILE="examples/user_layout.rhai"
KRX_FILE="user_layout.krx"

if [[ ! -f "$RHAI_FILE" ]]; then
    log_error "user_layout.rhai not found at: $RHAI_FILE"
    exit 1
fi

log_info "Compiling: $RHAI_FILE -> $KRX_FILE"
cargo run --bin keyrx_compiler -- compile "$RHAI_FILE" -o "$KRX_FILE"

if [[ ! -f "$KRX_FILE" ]]; then
    log_error "Failed to create .krx file"
    exit 1
fi

log_info "✓ Compilation successful: $KRX_FILE"

# ============================================================
# STEP 2: Build Web UI
# ============================================================
log_step "2/5 - Building Web UI"

cd keyrx_ui
log_info "Installing dependencies..."
npm install --quiet

log_info "Building production bundle..."
npm run build:wasm
npx vite build

if [ $? -ne 0 ]; then
    log_error "Web UI build failed"
    exit 1
fi

if [[ ! -d "dist" ]]; then
    log_error "Web UI build failed - dist directory not found"
    exit 1
fi

log_info "✓ Web UI built successfully"
cd "$PROJECT_ROOT"

# ============================================================
# STEP 3: Build Daemon
# ============================================================
log_step "3/5 - Building daemon with embedded UI"

BUILD_CMD="cargo build --bin keyrx_daemon"
DAEMON_BIN="target/debug/keyrx_daemon"

if [[ "$RELEASE" == "true" ]]; then
    BUILD_CMD="$BUILD_CMD --release"
    DAEMON_BIN="target/release/keyrx_daemon"
    log_info "Building daemon in release mode..."
else
    log_info "Building daemon in debug mode..."
fi

$BUILD_CMD

if [[ ! -f "$DAEMON_BIN" ]]; then
    log_error "Daemon build failed - binary not found"
    exit 1
fi

log_info "✓ Daemon built successfully: $DAEMON_BIN"

# ============================================================
# STEP 4: Launch Daemon
# ============================================================
log_step "4/5 - Launching daemon with system tray and web UI"

DAEMON_ARGS=()
DAEMON_ARGS+=("run")
DAEMON_ARGS+=("--config" "$KRX_FILE")

if [[ "$DEBUG" == "true" ]]; then
    DAEMON_ARGS+=("--debug")
    log_info "Debug logging enabled"
fi

log_info "Starting daemon with configuration: $KRX_FILE"
log_info "Command: $DAEMON_BIN ${DAEMON_ARGS[*]}"
echo ""
log_info "================================================"
log_info "UAT Setup Complete!"
log_info "================================================"
log_info ""
log_info "The daemon will start with:"
log_info "  - System tray icon (right-click for menu)"
log_info "  - Web UI at: http://localhost:9867"
log_info "  - Configuration: $KRX_FILE"
log_info ""
log_info "Press Ctrl+C to stop the daemon"
log_info "================================================"
echo ""

# Ensure GTK/tray environment variables are set
export GDK_BACKEND=x11
# Auto-detect DISPLAY from X11 socket if not set
if [ -z "$DISPLAY" ]; then
    # Find the X11 display number from socket
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

# Open browser if not headless
if [[ "$HEADLESS" == "false" ]]; then
    sleep 2
    if command -v xdg-open &> /dev/null; then
        log_info "Opening browser..."
        xdg-open "http://localhost:9867" &> /dev/null &
    elif command -v gnome-open &> /dev/null; then
        gnome-open "http://localhost:9867" &> /dev/null &
    fi
fi

# Run daemon (foreground)
exec "$DAEMON_BIN" "${DAEMON_ARGS[@]}"
