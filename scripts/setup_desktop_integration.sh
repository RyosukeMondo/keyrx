#!/bin/bash
set -e

# setup_desktop_integration.sh
# Sets up KeyRx for the current user with GNOME/System Tray integration.

# 1. Configuration
BINARY_SOURCE="target/release/keyrx_daemon"
INSTALL_BIN_DIR="$HOME/.local/bin"
INSTALL_APP_DIR="$HOME/.local/share/applications"
INSTALL_AUTOSTART_DIR="$HOME/.config/autostart"
INSTALL_ICON_DIR="$HOME/.local/share/icons/hicolor/128x128/apps"
CONFIG_DIR="$HOME/.config/keyrx"

echo "Setting up KeyRx Desktop Integration for user: $USER"

# 2. Check Prerequisites (Groups)
echo "Checking permissions..."
if groups | grep -q "input" && groups | grep -q "uinput"; then
    echo "✅ User is member of 'input' and 'uinput' groups."
else
    echo "❌ User is NOT in required groups."
    echo "Please run the following commands and then LOG OUT and LOG BACK IN:"
    echo "  sudo groupadd -f uinput"
    echo "  sudo usermod -aG input $USER"
    echo "  sudo usermod -aG uinput $USER"
    echo "  sudo cp keyrx_daemon/udev/99-keyrx.rules /etc/udev/rules.d/"
    echo "  sudo udevadm control --reload-rules && sudo udevadm trigger"
    exit 1
fi

# 3. Install Binary
echo "Installing binary..."
mkdir -p "$INSTALL_BIN_DIR"
cp "$BINARY_SOURCE" "$INSTALL_BIN_DIR/"
echo "✅ keyrx_daemon installed to $INSTALL_BIN_DIR"

# 4. Install Icon
echo "Installing icon..."
mkdir -p "$INSTALL_ICON_DIR"
cp "keyrx_daemon/assets/icon.png" "$INSTALL_ICON_DIR/keyrx.png"
echo "✅ Icon installed to $INSTALL_ICON_DIR/keyrx.png"

# 5. Create/Install .desktop file
echo "Configuring desktop entry..."
mkdir -p "$INSTALL_APP_DIR"
mkdir -p "$INSTALL_AUTOSTART_DIR"

# Generate correct Exec path
EXEC_PATH="$INSTALL_BIN_DIR/keyrx_daemon"

# Create a temporary desktop file content
cat > /tmp/keyrx.desktop <<EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=KeyRx
GenericName=Keyboard Remapper
Comment=Advanced keyboard remapping daemon with layer support
Icon=keyrx
Exec=$EXEC_PATH run --config $CONFIG_DIR/config.krx
Terminal=false
Categories=Utility;
Keywords=keyboard;remap;keymap;mapping;input;
StartupNotify=true
X-GNOME-Autostart-enabled=true
X-GNOME-UsesNotifications=true
EOF

# Install to Applications menu
cp /tmp/keyrx.desktop "$INSTALL_APP_DIR/keyrx.desktop"
echo "✅ Desktop menu entry created: $INSTALL_APP_DIR/keyrx.desktop"

# Install to Autostart
cp /tmp/keyrx.desktop "$INSTALL_AUTOSTART_DIR/keyrx.desktop"
echo "✅ Autostart entry created: $INSTALL_AUTOSTART_DIR/keyrx.desktop"

# 6. Ensure Config Directory
echo "Checking configuration..."
mkdir -p "$CONFIG_DIR"
if [ ! -f "$CONFIG_DIR/config.krx" ]; then
    echo "⚠️  No compiled config found at $CONFIG_DIR/config.krx"
    echo "   You need to compile a config file using keyrx_compiler."
    echo "   Example: keyrx_compiler my_layout.rhai -o $CONFIG_DIR/config.krx"
else
    echo "✅ Configuration found at $CONFIG_DIR/config.krx"
fi

echo "========================================================"
echo "Setup Complete!"
echo "1. KeyRx will start automatically on next login."
echo "2. You can start it now from your Applications menu."
echo "3. System Tray icon should appear when running."
echo "========================================================"
