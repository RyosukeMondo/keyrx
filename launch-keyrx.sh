#!/bin/bash
echo "========================================"
echo "KeyRx - Advanced Keyboard Remapping"
echo "Version: 0.1.5"
echo "========================================"
echo ""
echo "Starting KeyRx Daemon..."
echo ""

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Kill any existing keyrx processes to avoid conflicts
echo "Stopping existing keyrx processes..."
if command -v taskkill &>/dev/null; then
    # Windows
    taskkill //F //IM keyrx_daemon.exe 2>/dev/null && echo "Killed existing keyrx_daemon.exe" || true
else
    # Linux/macOS
    pkill -f keyrx_daemon 2>/dev/null && echo "Killed existing keyrx_daemon" || true
fi
sleep 1

# Check if daemon executable exists
if [ ! -f "$SCRIPT_DIR/keyrx_daemon" ]; then
    echo "ERROR: keyrx_daemon not found!"
    echo "Please ensure you extracted all files from the archive."
    exit 1
fi

# Make executable if not already
chmod +x "$SCRIPT_DIR/keyrx_daemon"

# Start the daemon in the background
"$SCRIPT_DIR/keyrx_daemon" run &
DAEMON_PID=$!

echo "KeyRx daemon is now running! (PID: $DAEMON_PID)"
echo ""
echo "========================================"
echo "Web UI available at: http://localhost:9867"
echo "========================================"
echo ""
echo "To stop the daemon, run: kill $DAEMON_PID"
echo ""
