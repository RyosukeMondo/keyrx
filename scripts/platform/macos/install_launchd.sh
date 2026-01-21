#!/usr/bin/env bash
# Install keyrx_daemon as a LaunchAgent for automatic startup on macOS

set -euo pipefail

# Colors
readonly COLOR_RESET='\033[0m'
readonly COLOR_GREEN='\033[0;32m'
readonly COLOR_YELLOW='\033[1;33m'
readonly COLOR_BLUE='\033[0;34m'
readonly COLOR_RED='\033[0;31m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

PLIST_PATH="$HOME/Library/LaunchAgents/com.keyrx.daemon.plist"
DAEMON_PATH="$PROJECT_ROOT/target/release/keyrx_daemon"
LOG_DIR="$HOME/Library/Logs"

# Check if daemon binary exists
if [[ ! -f "$DAEMON_PATH" ]]; then
    echo -e "${COLOR_RED}Error: Daemon binary not found at: $DAEMON_PATH${COLOR_RESET}"
    echo "Please build the project first: cargo build --release"
    exit 1
fi

echo -e "${COLOR_BLUE}=== KeyRx LaunchAgent Installation ===${COLOR_RESET}"
echo ""
echo "This script will install keyrx_daemon as a LaunchAgent"
echo "to automatically start on login."
echo ""
echo "Daemon binary: $DAEMON_PATH"
echo "LaunchAgent plist: $PLIST_PATH"
echo "Logs directory: $LOG_DIR"
echo ""

# Create LaunchAgents directory if it doesn't exist
mkdir -p "$HOME/Library/LaunchAgents"
mkdir -p "$LOG_DIR"

# Create the plist file
cat > "$PLIST_PATH" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.keyrx.daemon</string>
    <key>ProgramArguments</key>
    <array>
        <string>$DAEMON_PATH</string>
        <string>run</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>$LOG_DIR/keyrx-daemon.log</string>
    <key>StandardErrorPath</key>
    <string>$LOG_DIR/keyrx-daemon-error.log</string>
</dict>
</plist>
EOF

echo -e "${COLOR_GREEN}✅ LaunchAgent plist created: $PLIST_PATH${COLOR_RESET}"
echo ""
echo "To start the daemon now and on every login:"
echo -e "${COLOR_BLUE}  launchctl load \"$PLIST_PATH\"${COLOR_RESET}"
echo ""
echo "To stop the daemon:"
echo -e "${COLOR_BLUE}  launchctl unload \"$PLIST_PATH\"${COLOR_RESET}"
echo ""
echo "To check daemon status:"
echo -e "${COLOR_BLUE}  launchctl list | grep keyrx${COLOR_RESET}"
echo ""
echo "Logs are written to:"
echo "  $LOG_DIR/keyrx-daemon.log"
echo "  $LOG_DIR/keyrx-daemon-error.log"
echo ""

# Ask if user wants to load it now
read -r -p "Load the LaunchAgent now? [y/N] " response || {
    echo ""
    exit 0
}

case "$response" in
    [yY][eE][sS]|[yY])
        echo ""
        echo "Loading LaunchAgent..."
        if launchctl load "$PLIST_PATH" 2>&1; then
            echo -e "${COLOR_GREEN}✅ LaunchAgent loaded successfully${COLOR_RESET}"
            echo ""
            echo "The daemon is now running and will auto-start on login."
        else
            echo -e "${COLOR_RED}❌ Failed to load LaunchAgent${COLOR_RESET}"
            echo "You can try loading it manually later with:"
            echo "  launchctl load \"$PLIST_PATH\""
            exit 1
        fi
        ;;
    *)
        echo ""
        echo "LaunchAgent not loaded. Load it later with:"
        echo "  launchctl load \"$PLIST_PATH\""
        ;;
esac

echo ""
