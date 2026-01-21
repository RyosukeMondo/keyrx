#!/usr/bin/env bash
# install.sh - Install KeyRx daemon and desktop integration
#
# This script installs the KeyRx keyboard remapping daemon with platform-specific
# integration:
#
# Linux:
# - Binary installation
# - Desktop application launcher
# - System tray icon
# - Autostart on login
# - Systemd user service
# - udev rules and groups
#
# macOS:
# - Binary installation
# - LaunchAgent for autostart
# - Config directory setup
#
# Usage:
#   ./install.sh [--user|--system] [--no-autostart] [--no-udev]
#
# Options:
#   --user         Install for current user only (default)
#   --system       Install system-wide (requires sudo, Linux only)
#   --no-autostart Skip autostart installation
#   --no-udev      Skip udev rules installation (Linux only)
#   --help         Show this help message

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Installation mode (user or system)
INSTALL_MODE="user"
INSTALL_AUTOSTART=true
INSTALL_UDEV=true

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --user)
            INSTALL_MODE="user"
            shift
            ;;
        --system)
            INSTALL_MODE="system"
            shift
            ;;
        --no-autostart)
            INSTALL_AUTOSTART=false
            shift
            ;;
        --no-udev)
            INSTALL_UDEV=false
            shift
            ;;
        --help)
            head -n 20 "$0" | grep "^#" | sed 's/^# \?//'
            exit 0
            ;;
        *)
            echo -e "${RED}Error: Unknown option: $1${NC}"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Utility functions
info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*"
}

warning() {
    echo -e "${YELLOW}[WARNING]${NC} $*"
}

error() {
    echo -e "${RED}[ERROR]${NC} $*"
}

# Check if binaries exist
DAEMON_BIN="$PROJECT_ROOT/target/release/keyrx_daemon"
COMPILER_BIN="$PROJECT_ROOT/target/release/keyrx_compiler"

if [[ ! -f "$DAEMON_BIN" ]]; then
    error "keyrx_daemon binary not found at: $DAEMON_BIN"
    info "Please build the project first: cargo build --release"
    exit 1
fi

if [[ ! -f "$COMPILER_BIN" ]]; then
    warning "keyrx_compiler binary not found at: $COMPILER_BIN"
    warning "Only daemon will be installed. Build with: cargo build --release -p keyrx_compiler"
fi

# Set installation paths based on mode
if [[ "$INSTALL_MODE" == "system" ]]; then
    BIN_DIR="/usr/local/bin"
    APPLICATIONS_DIR="/usr/share/applications"
    ICONS_DIR="/usr/share/icons/hicolor/256x256/apps"
    SYSTEMD_DIR="/etc/systemd/user"

    # Check for sudo
    if [[ $EUID -ne 0 ]]; then
        error "System-wide installation requires root privileges"
        info "Please run with sudo or use --user for user installation"
        exit 1
    fi
else
    BIN_DIR="$HOME/.local/bin"
    APPLICATIONS_DIR="$HOME/.local/share/applications"
    ICONS_DIR="$HOME/.local/share/icons/hicolor/256x256/apps"
    SYSTEMD_DIR="$HOME/.config/systemd/user"
fi

# Autostart directory (always user-specific)
AUTOSTART_DIR="$HOME/.config/autostart"

# Config directory (platform-specific)
if [[ "$OSTYPE" == "darwin"* ]]; then
    CONFIG_DIR="$HOME/Library/Application Support/keyrx"
else
    CONFIG_DIR="$HOME/.config/keyrx"
fi

# Installation functions
install_macos() {
    echo ""
    echo "========================================="
    echo "  KeyRx Installation (macOS)"
    echo "========================================="
    echo ""

    local install_dir="$HOME/.local/bin"
    local config_dir="$HOME/Library/Application Support/keyrx"

    info "Installation mode: user"
    info "Binary directory: $install_dir"
    info "Config directory: $config_dir"
    echo ""

    # Create directories
    info "Creating installation directories..."
    mkdir -p "$install_dir"
    mkdir -p "$config_dir"

    # Install binaries
    info "Installing binaries..."
    install -m 755 "$DAEMON_BIN" "$install_dir/keyrx_daemon"
    success "Installed keyrx_daemon to $install_dir"

    if [[ -f "$COMPILER_BIN" ]]; then
        install -m 755 "$COMPILER_BIN" "$install_dir/keyrx_compiler"
        success "Installed keyrx_compiler to $install_dir"
    fi

    # Create LaunchAgent if autostart is requested
    if [[ "$INSTALL_AUTOSTART" == true ]]; then
        local plist_path="$HOME/Library/LaunchAgents/com.keyrx.daemon.plist"
        info "Creating LaunchAgent for autostart..."

        cat > "$plist_path" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.keyrx.daemon</string>
    <key>ProgramArguments</key>
    <array>
        <string>$install_dir/keyrx_daemon</string>
        <string>run</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>$HOME/Library/Logs/keyrx-daemon.log</string>
    <key>StandardErrorPath</key>
    <string>$HOME/Library/Logs/keyrx-daemon-error.log</string>
</dict>
</plist>
EOF
        success "Created LaunchAgent at $plist_path"
    fi

    # Create example config
    local example_config="$config_dir/example.rhai"
    if [[ ! -f "$example_config" ]]; then
        info "Creating example configuration..."
        cat > "$example_config" <<'EOF'
// KeyRx Example Configuration
// Save as: ~/Library/Application Support/keyrx/config.rhai

// Example: Swap Caps Lock and Escape
// Uncomment to enable:
// map(KEY_CAPSLOCK, KEY_ESC)
// map(KEY_ESC, KEY_CAPSLOCK)

// Example: Make Caps Lock a Ctrl when held
// tap_hold(KEY_CAPSLOCK, KEY_ESC, KEY_LEFTCTRL)

info("KeyRx configuration loaded successfully!");
info("Edit this file to customize your keyboard remapping");
EOF
        success "Created example configuration at $example_config"
    fi

    # Summary
    echo ""
    echo "========================================="
    echo "  Installation Complete!"
    echo "========================================="
    echo ""

    success "KeyRx has been installed successfully"
    echo ""
    info "Next steps:"
    echo ""
    echo "  1. Grant Accessibility permission:"
    echo "     System Settings > Privacy & Security > Accessibility"
    echo "     Add Terminal (or keyrx_daemon) to the list"
    echo ""
    echo "  2. Configure your key mappings:"
    echo "     \$ nano \"$config_dir/example.rhai\""
    echo ""
    echo "  3. Compile your configuration:"
    echo "     \$ keyrx_compiler \"$config_dir/example.rhai\" -o \"$config_dir/config.krx\""
    echo ""

    if [[ "$INSTALL_AUTOSTART" == true ]]; then
        echo "  4. Start the daemon (will auto-start on login):"
        echo "     \$ launchctl load \"$HOME/Library/LaunchAgents/com.keyrx.daemon.plist\""
        echo ""
        echo "  5. Stop the daemon:"
        echo "     \$ launchctl unload \"$HOME/Library/LaunchAgents/com.keyrx.daemon.plist\""
    else
        echo "  4. Start the daemon manually:"
        echo "     \$ keyrx_daemon run"
    fi

    echo ""
    info "You can also use the interactive setup script:"
    info "  $SCRIPT_DIR/platform/macos/setup_accessibility.sh"
    echo ""
}

install_linux() {
    echo ""
    echo "========================================="
    echo "  KeyRx Installation (Linux)"
    echo "========================================="
    echo ""
    info "Installation mode: $INSTALL_MODE"
    info "Binary directory: $BIN_DIR"
    info "Applications directory: $APPLICATIONS_DIR"
    info "Systemd directory: $SYSTEMD_DIR"
    echo ""

# Create necessary directories
info "Creating installation directories..."
mkdir -p "$BIN_DIR"
mkdir -p "$APPLICATIONS_DIR"
mkdir -p "$ICONS_DIR"
mkdir -p "$SYSTEMD_DIR"
mkdir -p "$CONFIG_DIR"

if [[ "$INSTALL_AUTOSTART" == true ]]; then
    mkdir -p "$AUTOSTART_DIR"
fi

# Install binaries
info "Installing binaries..."
install -m 755 "$DAEMON_BIN" "$BIN_DIR/keyrx_daemon"
success "Installed keyrx_daemon to $BIN_DIR"

if [[ -f "$COMPILER_BIN" ]]; then
    install -m 755 "$COMPILER_BIN" "$BIN_DIR/keyrx_compiler"
    success "Installed keyrx_compiler to $BIN_DIR"
fi

# Install icon
ICON_SRC="$PROJECT_ROOT/keyrx_daemon/assets/icon.png"
if [[ -f "$ICON_SRC" ]]; then
    info "Installing application icon..."
    install -m 644 "$ICON_SRC" "$ICONS_DIR/keyrx.png"
    success "Installed icon to $ICONS_DIR"

    # Update icon cache
    if command -v gtk-update-icon-cache >/dev/null 2>&1; then
        gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true
    fi
else
    warning "Icon not found at: $ICON_SRC"
fi

# Install desktop file
DESKTOP_SRC="$PROJECT_ROOT/keyrx_daemon/desktop/keyrx.desktop"
if [[ -f "$DESKTOP_SRC" ]]; then
    info "Installing application launcher..."
    install -m 644 "$DESKTOP_SRC" "$APPLICATIONS_DIR/keyrx.desktop"
    success "Installed desktop entry to $APPLICATIONS_DIR"

    # Update desktop database
    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database "$APPLICATIONS_DIR" 2>/dev/null || true
    fi
else
    warning "Desktop file not found at: $DESKTOP_SRC"
fi

# Install autostart entry
if [[ "$INSTALL_AUTOSTART" == true ]]; then
    AUTOSTART_SRC="$PROJECT_ROOT/keyrx_daemon/desktop/keyrx-autostart.desktop"
    if [[ -f "$AUTOSTART_SRC" ]]; then
        info "Installing autostart entry..."
        install -m 644 "$AUTOSTART_SRC" "$AUTOSTART_DIR/keyrx.desktop"
        success "Installed autostart entry to $AUTOSTART_DIR"
    else
        warning "Autostart file not found at: $AUTOSTART_SRC"
    fi
fi

# Install systemd user service
SYSTEMD_SRC="$PROJECT_ROOT/keyrx_daemon/systemd/keyrx-user.service"
if [[ -f "$SYSTEMD_SRC" ]]; then
    info "Installing systemd user service..."
    install -m 644 "$SYSTEMD_SRC" "$SYSTEMD_DIR/keyrx.service"
    success "Installed systemd service to $SYSTEMD_DIR"

    # Reload systemd user daemon
    info "Reloading systemd user daemon..."
    systemctl --user daemon-reload
else
    warning "Systemd service file not found at: $SYSTEMD_SRC"
fi

# Install udev rules (requires sudo)
if [[ "$INSTALL_UDEV" == true ]]; then
    UDEV_SRC="$PROJECT_ROOT/keyrx_daemon/udev/99-keyrx.rules"
    UDEV_DEST="/etc/udev/rules.d/99-keyrx.rules"

    if [[ -f "$UDEV_SRC" ]]; then
        info "Installing udev rules (requires sudo)..."

        if [[ "$INSTALL_MODE" == "system" ]]; then
            # Already running as root
            install -m 644 "$UDEV_SRC" "$UDEV_DEST"
            udevadm control --reload-rules
            udevadm trigger
            success "Installed udev rules to $UDEV_DEST"
        else
            # Need sudo for udev rules
            if sudo -n true 2>/dev/null; then
                sudo install -m 644 "$UDEV_SRC" "$UDEV_DEST"
                sudo udevadm control --reload-rules
                sudo udevadm trigger
                success "Installed udev rules to $UDEV_DEST"
            else
                warning "Cannot install udev rules without sudo privileges"
                info "To install udev rules manually, run:"
                info "  sudo install -m 644 '$UDEV_SRC' '$UDEV_DEST'"
                info "  sudo udevadm control --reload-rules"
                info "  sudo udevadm trigger"
            fi
        fi

        # Add user to required groups
        info "Adding user to input and uinput groups (requires sudo)..."

        if [[ "$INSTALL_MODE" == "system" ]] || sudo -n true 2>/dev/null; then
            CURRENT_USER="${SUDO_USER:-$USER}"

            # Create groups if they don't exist
            if ! getent group input >/dev/null; then
                if [[ "$INSTALL_MODE" == "system" ]]; then
                    groupadd input
                else
                    sudo groupadd input
                fi
            fi

            if ! getent group uinput >/dev/null; then
                if [[ "$INSTALL_MODE" == "system" ]]; then
                    groupadd uinput
                else
                    sudo groupadd uinput
                fi
            fi

            # Add user to groups
            if [[ "$INSTALL_MODE" == "system" ]]; then
                usermod -a -G input,uinput "$CURRENT_USER"
            else
                sudo usermod -a -G input,uinput "$CURRENT_USER"
            fi

            success "Added $CURRENT_USER to input and uinput groups"
            warning "You may need to log out and log back in for group changes to take effect"
        else
            warning "Cannot add user to groups without sudo privileges"
            info "To add groups manually, run:"
            info "  sudo usermod -a -G input,uinput \$USER"
            info "Then log out and log back in"
        fi

        # Load uinput kernel module
        info "Loading uinput kernel module (requires sudo)..."

        if [[ "$INSTALL_MODE" == "system" ]] || sudo -n true 2>/dev/null; then
            if [[ "$INSTALL_MODE" == "system" ]]; then
                modprobe uinput || warning "Failed to load uinput module"
            else
                sudo modprobe uinput || warning "Failed to load uinput module"
            fi

            # Make uinput load on boot
            MODULES_LOAD="/etc/modules-load.d/keyrx.conf"
            if [[ "$INSTALL_MODE" == "system" ]]; then
                echo "uinput" > "$MODULES_LOAD"
            else
                echo "uinput" | sudo tee "$MODULES_LOAD" >/dev/null
            fi

            success "Loaded uinput kernel module"
        else
            warning "Cannot load kernel module without sudo privileges"
            info "To load the module manually, run:"
            info "  sudo modprobe uinput"
            info "  echo 'uinput' | sudo tee /etc/modules-load.d/keyrx.conf"
        fi
    else
        warning "Udev rules file not found at: $UDEV_SRC"
    fi
fi

# Create example config if it doesn't exist
EXAMPLE_CONFIG="$CONFIG_DIR/example.rhai"
if [[ ! -f "$EXAMPLE_CONFIG" ]]; then
    info "Creating example configuration..."
    cat > "$EXAMPLE_CONFIG" <<'EOF'
// KeyRx Example Configuration
// Save as: ~/.config/keyrx/config.rhai

// Example: Swap Caps Lock and Escape
// Uncomment to enable:
// map(KEY_CAPSLOCK, KEY_ESC)
// map(KEY_ESC, KEY_CAPSLOCK)

// Example: Make Caps Lock a Ctrl when held
// tap_hold(KEY_CAPSLOCK, KEY_ESC, KEY_LEFTCTRL)

info("KeyRx configuration loaded successfully!");
info("Edit this file to customize your keyboard remapping");
EOF
    success "Created example configuration at $EXAMPLE_CONFIG"
    info "Edit $EXAMPLE_CONFIG to configure your key mappings"
fi

# Summary
echo ""
echo "========================================="
echo "  Installation Complete!"
echo "========================================="
echo ""

success "KeyRx has been installed successfully"
echo ""
info "Next steps:"
echo ""
echo "  1. Configure your key mappings:"
echo "     \$ nano $CONFIG_DIR/example.rhai"
echo ""
echo "  2. Compile your configuration:"
echo "     \$ keyrx_compiler $CONFIG_DIR/example.rhai -o $CONFIG_DIR/config.krx"
echo ""
echo "  3. Start the daemon:"
echo "     \$ systemctl --user start keyrx"
echo ""
echo "  4. Enable autostart:"
echo "     \$ systemctl --user enable keyrx"
echo ""
echo "  5. Check daemon status:"
echo "     \$ systemctl --user status keyrx"
echo ""

if [[ "$INSTALL_UDEV" == true ]]; then
    echo "  Note: If udev rules were installed, you may need to:"
    echo "    - Log out and log back in for group membership to take effect"
    echo "    - Reconnect your keyboard if it was plugged in during installation"
    echo ""
fi

info "The daemon will appear in your application launcher and system tray"
info "You can also configure it to start automatically on login"

echo ""
}

# Main installation flow
case "$OSTYPE" in
    darwin*)
        # macOS doesn't support system-wide installation
        if [[ "$INSTALL_MODE" == "system" ]]; then
            warning "System-wide installation not supported on macOS, using user installation"
            INSTALL_MODE="user"
        fi

        # Udev rules don't apply to macOS
        if [[ "$INSTALL_UDEV" == true ]]; then
            INSTALL_UDEV=false
        fi

        install_macos
        ;;
    linux*)
        install_linux
        ;;
    *)
        error "Unsupported operating system: $OSTYPE"
        error "KeyRx supports Linux and macOS only"
        exit 1
        ;;
esac
