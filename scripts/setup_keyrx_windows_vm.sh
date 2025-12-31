#!/bin/bash
# Automated setup script for keyrx on Windows VM
# Builds, installs, and configures keyrx so you can test via RDP

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
VAGRANT_DIR="$PROJECT_ROOT/vagrant/windows"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*"
}

warn() {
    echo -e "${YELLOW}[WARNING]${NC} $*"
}

echo ""
echo "========================================="
echo "  keyrx Windows VM Setup"
echo "========================================="
echo ""

# Check Vagrant directory exists
if [[ ! -d "$VAGRANT_DIR" ]]; then
    echo "Error: Vagrant directory not found: $VAGRANT_DIR"
    exit 1
fi

cd "$VAGRANT_DIR"

# 1. Start VM
info "Starting Vagrant Windows VM..."
vagrant up

# 2. Sync files
info "Syncing project files to VM..."
vagrant rsync

# 3. Build keyrx
info "Building keyrx for Windows (this may take a few minutes)..."
vagrant winrm -c 'cd C:\vagrant_project; cargo build --release --features windows'

# 4. Create installation directory
info "Creating installation directory..."
vagrant winrm -c 'New-Item -ItemType Directory -Path "C:\Program Files\keyrx" -Force' >/dev/null

# 5. Install binaries
info "Installing binaries..."
vagrant winrm -c 'Copy-Item C:\vagrant_project\target\release\keyrx_daemon.exe "C:\Program Files\keyrx\" -Force'
vagrant winrm -c 'Copy-Item C:\vagrant_project\target\release\keyrx_compiler.exe "C:\Program Files\keyrx\" -Force -ErrorAction SilentlyContinue'

# 6. Create config directory
info "Creating config directory..."
vagrant winrm -c 'New-Item -ItemType Directory -Path "C:\Users\vagrant\.config\keyrx" -Force' >/dev/null

# 7. Create example config
info "Creating example configuration..."
vagrant winrm -c @"
@'
// KeyRx Example Configuration for Windows

// Example 1: Swap Caps Lock and Escape
map(KEY_CAPSLOCK, KEY_ESC)
map(KEY_ESC, KEY_CAPSLOCK)

// Example 2: Make Right Alt act as Ctrl
// Uncomment to enable:
// map(KEY_RIGHTALT, KEY_LEFTCTRL)

// Example 3: Tap/Hold - Caps as Esc when tapped, Ctrl when held
// Uncomment to enable:
// tap_hold(KEY_CAPSLOCK, KEY_ESC, KEY_LEFTCTRL)

info(\"KeyRx Windows configuration loaded!\");
'@ | Out-File -FilePath C:\Users\vagrant\.config\keyrx\config.rhai -Encoding UTF8
"

# 8. Compile config
info "Compiling configuration..."
vagrant winrm -c 'cd "C:\Program Files\keyrx"; .\keyrx_compiler.exe C:\Users\vagrant\.config\keyrx\config.rhai -o C:\Users\vagrant\.config\keyrx\config.krx'

# 9. Create desktop shortcut
info "Creating desktop shortcut..."
vagrant winrm -c @'
$WshShell = New-Object -ComObject WScript.Shell
$Shortcut = $WshShell.CreateShortcut("C:\Users\vagrant\Desktop\keyrx.lnk")
$Shortcut.TargetPath = "C:\Program Files\keyrx\keyrx_daemon.exe"
$Shortcut.Arguments = "--config C:\Users\vagrant\.config\keyrx\config.krx"
$Shortcut.WorkingDirectory = "C:\Program Files\keyrx"
$Shortcut.Description = "KeyRx Keyboard Remapping Daemon"
$Shortcut.Save()
'

# 10. Get IP address for RDP connection
info "Getting VM IP address..."
VM_IP=$(vagrant winrm -c 'ipconfig' | grep -A 10 "Ethernet adapter" | grep "IPv4" | head -1 | awk '{print $NF}' | tr -d '\r')

# Get host IP (for connecting from Windows PC)
HOST_IP=$(ip route get 1 | awk '{print $7; exit}')

echo ""
echo "========================================="
echo "  Setup Complete!"
echo "========================================="
echo ""
success "keyrx has been installed on the Windows VM"
echo ""
info "Installation details:"
echo "  • Binaries: C:\\Program Files\\keyrx\\"
echo "  • Config: C:\\Users\\vagrant\\.config\\keyrx\\config.rhai"
echo "  • Compiled: C:\\Users\\vagrant\\.config\\keyrx\\config.krx"
echo "  • Desktop shortcut: Created"
echo ""
info "Connect via RDP:"
echo ""
echo "  From your Windows PC:"
echo "    mstsc /v:${HOST_IP}:13389"
echo ""
echo "  From this Linux host:"
echo "    xfreerdp /v:localhost:13389 /u:vagrant /p:vagrant /size:1920x1080 +clipboard"
echo ""
echo "  Login credentials:"
echo "    Username: vagrant"
echo "    Password: vagrant"
echo ""
info "Run keyrx in RDP session:"
echo ""
echo "  Option 1: Double-click 'keyrx' shortcut on Desktop"
echo ""
echo "  Option 2: PowerShell:"
echo "    cd 'C:\\Program Files\\keyrx'"
echo "    .\\keyrx_daemon.exe"
echo ""
info "Test the remapping:"
echo "  1. Open Notepad in the RDP session"
echo "  2. Press Caps Lock → should produce Escape"
echo "  3. Press Escape → should toggle Caps Lock"
echo ""
warn "Note: RDP has limitations for low-level input testing."
info "See docs/TESTING_WINDOWS_RDP.md for details."
echo ""
