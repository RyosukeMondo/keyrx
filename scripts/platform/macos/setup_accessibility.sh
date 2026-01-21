#!/usr/bin/env bash
# Interactive guide for granting Accessibility permission on macOS

set -euo pipefail

# Colors
readonly COLOR_RESET='\033[0m'
readonly COLOR_GREEN='\033[0;32m'
readonly COLOR_YELLOW='\033[1;33m'
readonly COLOR_BLUE='\033[0;34m'
readonly COLOR_RED='\033[0;31m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo -e "${COLOR_BLUE}=== macOS Accessibility Permission Setup ===${COLOR_RESET}"
echo ""
echo "KeyRx needs Accessibility permission to:"
echo "  • Enumerate keyboard devices"
echo "  • Intercept keyboard events"
echo "  • Inject remapped keys"
echo ""

# Check current status
if "$SCRIPT_DIR/check_permission.sh" 2>/dev/null; then
    echo -e "${COLOR_GREEN}✅ Accessibility permission already granted!${COLOR_RESET}"
    exit 0
fi

echo -e "${COLOR_YELLOW}❌ Accessibility permission not granted${COLOR_RESET}"
echo ""
echo "To grant permission:"
echo ""
echo "  1. Open System Settings (the script will open it for you)"
echo "  2. Go to: Privacy & Security > Accessibility"
echo "  3. Click the lock icon 🔒 (you'll need to enter your password)"
echo "  4. Click the + button to add an application"
echo "  5. Navigate to one of these:"
echo "     • Terminal.app (if running from Terminal)"
echo "     • Your IDE (VS Code, IntelliJ, etc.)"
echo "     • keyrx_daemon binary (if built)"
echo "  6. Click 'Open' to add it"
echo "  7. Toggle the switch ON to enable Accessibility"
echo ""

read -r -p "Press Enter to open System Settings..." || true
echo ""

# Try to open System Settings to the Accessibility pane
# The URL scheme works on macOS 13+ (Ventura and later)
if ! open "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility" 2>/dev/null; then
    # Fallback: just open System Settings
    open "/System/Applications/System Settings.app" 2>/dev/null || true
fi

echo "System Settings should now be open."
echo ""
echo "After granting permission, you can verify by running:"
echo "  $SCRIPT_DIR/check_permission.sh"
echo ""
echo "Or re-run this script to check automatically."
echo ""
