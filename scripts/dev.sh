#!/usr/bin/env bash
# dev_ui.sh - Start development environment for React UI
#
# This script provides instant hot module replacement (HMR) for React development
# without needing to rebuild the daemon after every UI change.
#
# Usage:
#   ./scripts/dev_ui.sh [--no-daemon] [--no-browser]
#
# Options:
#   --no-daemon    Skip starting the daemon (assumes it's already running)
#   --no-browser   Don't open browser automatically
#
# How it works:
#   1. Ensures daemon is running (on port 9867)
#   2. Starts Vite dev server (on port 5173)
#   3. Vite proxies /api and /ws requests to daemon
#   4. React changes reload instantly with HMR
#
# Development workflow:
#   - React changes: Instant HMR (no rebuild needed)
#   - Rust changes: Rebuild daemon only (`cargo build -p keyrx_daemon`)
#   - WASM changes: Run `npm run build:wasm` in keyrx_ui/

set -euo pipefail

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
UI_DIR="$PROJECT_ROOT/keyrx_ui"

# Parse arguments
START_DAEMON=true
OPEN_BROWSER=true

while [[ $# -gt 0 ]]; do
    case $1 in
        --no-daemon)
            START_DAEMON=false
            shift
            ;;
        --no-browser)
            OPEN_BROWSER=false
            shift
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            echo "Usage: $0 [--no-daemon] [--no-browser]"
            exit 1
            ;;
    esac
done

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  KeyRx UI Development Server${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Check if daemon is running
check_daemon() {
    pgrep -f keyrx_daemon > /dev/null 2>&1
}

# Start daemon if needed
if [ "$START_DAEMON" = true ]; then
    if check_daemon; then
        echo -e "${GREEN}✓${NC} Daemon is already running"
    else
        echo -e "${YELLOW}→${NC} Starting keyrx daemon..."

        # Check if daemon binary exists
        if [ ! -f "$HOME/.local/bin/keyrx_daemon" ] && [ ! -f "$PROJECT_ROOT/target/debug/keyrx_daemon" ]; then
            echo -e "${RED}✗${NC} Daemon binary not found"
            echo -e "${YELLOW}→${NC} Building daemon..."
            cd "$PROJECT_ROOT"
            cargo build -p keyrx_daemon --features linux
        fi

        # Start daemon
        DAEMON_BIN="$HOME/.local/bin/keyrx_daemon"
        if [ ! -f "$DAEMON_BIN" ]; then
            DAEMON_BIN="$PROJECT_ROOT/target/debug/keyrx_daemon"
        fi

        CONFIG="${HOME}/.config/keyrx/config.krx"
        if [ ! -f "$CONFIG" ]; then
            CONFIG="$PROJECT_ROOT/user_layout.krx"
        fi

        nohup "$DAEMON_BIN" run --config "$CONFIG" > /tmp/keyrx_daemon_dev.log 2>&1 &
        sleep 2

        if check_daemon; then
            echo -e "${GREEN}✓${NC} Daemon started successfully"
        else
            echo -e "${RED}✗${NC} Failed to start daemon"
            echo -e "${YELLOW}→${NC} Check logs: /tmp/keyrx_daemon_dev.log"
            exit 1
        fi
    fi
else
    if check_daemon; then
        echo -e "${GREEN}✓${NC} Using existing daemon instance"
    else
        echo -e "${RED}✗${NC} Daemon not running (use without --no-daemon to start it)"
        exit 1
    fi
fi

# Verify daemon is accessible
echo -e "${YELLOW}→${NC} Checking daemon connection..."
if curl -s http://localhost:9867 -o /dev/null; then
    echo -e "${GREEN}✓${NC} Daemon API accessible at http://localhost:9867"
else
    echo -e "${RED}✗${NC} Daemon not responding on port 9867"
    exit 1
fi

echo ""
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  Starting Vite Dev Server${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo -e "${GREEN}Development URLs:${NC}"
echo -e "  ${BLUE}UI Dev Server:${NC}  http://localhost:5173"
echo -e "  ${BLUE}Daemon API:${NC}     http://localhost:9867"
echo ""
echo -e "${YELLOW}Features:${NC}"
echo -e "  ✓ Instant Hot Module Replacement (HMR)"
echo -e "  ✓ API/WebSocket proxy to daemon"
echo -e "  ✓ React changes reload instantly"
echo ""
echo -e "${YELLOW}Development Workflow:${NC}"
echo -e "  ${GREEN}React changes:${NC}  Just save - HMR handles it!"
echo -e "  ${GREEN}Rust changes:${NC}   cargo build -p keyrx_daemon && restart daemon"
echo -e "  ${GREEN}WASM changes:${NC}   cd keyrx_ui && npm run build:wasm"
echo ""
echo -e "${YELLOW}Press Ctrl+C to stop${NC}"
echo ""

# Change to UI directory
cd "$UI_DIR"

# Start Vite dev server
if [ "$OPEN_BROWSER" = true ]; then
    npm run dev
else
    npm run dev -- --no-open
fi
