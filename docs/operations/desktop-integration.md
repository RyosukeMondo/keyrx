# Linux Desktop Integration Guide

Complete guide for KeyRx desktop integration including system tray, .desktop files, and startup options.

## Architecture Overview

```
┌──────────────────────────────────────────────────────┐
│ User Session (no root required)                      │
│                                                       │
│ ┌──────────────┐    ┌─────────────┐   ┌───────────┐ │
│ │ System Tray  │───▶│  Web UI     │   │  .desktop │ │
│ │ (keyrx-tray) │    │  (Browser)  │   │   files   │ │
│ └──────┬───────┘    └─────────────┘   └───────────┘ │
│        │                                              │
│        │ HTTP API (localhost:9867)                   │
└────────┼──────────────────────────────────────────────┘
         │
         ▼
┌──────────────────────────────────────────────────────┐
│ System Service (root, for keyboard access)           │
│                                                       │
│ ┌──────────────────────────────────────────────────┐ │
│ │  keyrx_daemon (systemd service)                  │ │
│ │  - Port 9867 (HTTP + WebSocket)                  │ │
│ │  - Keyboard interception (evdev - needs root)    │ │
│ │  - Profile/config management API                 │ │
│ └──────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────┘
```

## Components

### 1. System Daemon (keyrx_daemon)
- **Runs as**: root (systemd service)
- **Purpose**: Keyboard interception and remapping
- **Access**: Requires root for evdev
- **API**: HTTP server on port 9867
- **Files**: `/usr/bin/keyrx_daemon`, `/etc/systemd/system/keyrx.service`

### 2. System Tray (keyrx-tray.py)
- **Runs as**: user
- **Purpose**: Quick daemon control, status indicator
- **Technology**: Python + GTK + AppIndicator3
- **Auto-start**: `~/.config/autostart/keyrx-tray.desktop`

### 3. Desktop Files
- **keyrx.desktop**: Application launcher (shows in menu)
- **keyrx-tray.desktop**: Auto-start for tray (hidden from menu)
- **keyrx-config.desktop**: Direct link to web UI

## Installation Methods

### Method 1: Debian Package (Recommended)
```bash
sudo dpkg -i keyrx_0.1.0_amd64.deb

# Daemon auto-installed and started
# Desktop files in /usr/share/applications/
# System tray NOT included (optional)
```

### Method 2: Tarball
```bash
tar -xzf keyrx-0.1.0-linux-x86_64.tar.gz
cd keyrx-0.1.0-linux-x86_64
sudo ./install.sh

# Installs to /usr/local by default
# System tray NOT included (optional)
```

### Method 3: From Source
```bash
make package
sudo dpkg -i target/debian/keyrx_*.deb
```

## System Tray Setup (Optional)

### Install Dependencies (Ubuntu/Debian)
```bash
sudo apt-get install \
    python3-gi \
    gir1.2-appindicator3-0.1 \
    gir1.2-notify-0.7 \
    python3-requests
```

### Install Tray Application
```bash
# Copy script
sudo install -m 755 keyrx_tray/keyrx-tray.py /usr/local/bin/keyrx-tray

# Install .desktop files
mkdir -p ~/.local/share/applications
cp keyrx_tray/keyrx.desktop ~/.local/share/applications/

# Auto-start (optional)
mkdir -p ~/.config/autostart
cp keyrx_tray/keyrx-tray.desktop ~/.config/autostart/

# Or system-wide
sudo cp keyrx_tray/keyrx.desktop /usr/share/applications/
sudo cp keyrx_tray/keyrx-config.desktop /usr/share/applications/
```

### Start Tray
```bash
# Manual
keyrx-tray &

# Or log out and back in (auto-start)
```

## Startup Options Comparison

### Option 1: Systemd Service Only (Recommended for Servers)
**Pros:**
- Automatic start on boot
- Survives user logout
- Proper service management
- Resource limits and security

**Cons:**
- Requires root/sudo
- No GUI indicator

**Setup:**
```bash
sudo systemctl enable keyrx  # Enable auto-start
sudo systemctl start keyrx   # Start now
sudo systemctl status keyrx  # Check status
```

### Option 2: Systemd + System Tray (Recommended for Desktops)
**Pros:**
- Best of both worlds
- Visual status indicator
- Quick profile switching
- No terminal needed

**Cons:**
- Tray requires user session
- Two components to manage

**Setup:**
```bash
# Daemon (system service)
sudo systemctl enable keyrx
sudo systemctl start keyrx

# Tray (user auto-start)
mkdir -p ~/.config/autostart
cp keyrx_tray/keyrx-tray.desktop ~/.config/autostart/
keyrx-tray &
```

### Option 3: User Systemd Service (NOT Recommended - Evdev Needs Root)
**Note**: This won't work because evdev requires root access.

If you want non-root operation, you would need:
- udev rules for device access
- User in `input` group
- Security implications

## Desktop Files Explained

### keyrx.desktop (Application Launcher)
Shows "KeyRx" in application menu, launches system tray.

```ini
[Desktop Entry]
Name=KeyRx
GenericName=Keyboard Remapper
Comment=Advanced keyboard remapping
Exec=keyrx-tray
Icon=input-keyboard
Categories=Utility;Settings;System;
```

**Location**: `/usr/share/applications/` or `~/.local/share/applications/`

### keyrx-tray.desktop (Auto-start)
Starts system tray on login, hidden from menu.

```ini
[Desktop Entry]
Name=KeyRx System Tray
Exec=keyrx-tray
NoDisplay=true
X-GNOME-Autostart-enabled=true
```

**Location**: `~/.config/autostart/`

### keyrx-config.desktop (Web UI Shortcut)
Direct link to configuration web UI.

```ini
[Desktop Entry]
Name=KeyRx Configuration
Exec=xdg-open http://127.0.0.1:9867
Icon=preferences-desktop-keyboard
Categories=Settings;
```

**Location**: `/usr/share/applications/`

## GNOME Integration

### System Tray Icons
KeyRx uses AppIndicator3 for GNOME compatibility.

**Install GNOME Shell extension** (if tray icon doesn't appear):
```bash
# GNOME 40+
sudo apt-get install gnome-shell-extension-appindicator

# Enable
gnome-extensions enable appindicatorsupport@rgcjonas.gmail.com
```

### Keyboard Shortcuts
Add custom shortcut to toggle remapping:

1. Settings → Keyboard → Custom Shortcuts
2. Name: "Toggle KeyRx"
3. Command: `curl -X POST http://127.0.0.1:9867/api/toggle`
4. Shortcut: Super+K (or your choice)

### Notifications
KeyRx uses libnotify for desktop notifications:
- Profile switches
- Remapping enable/disable
- Error alerts

## KDE Integration

### System Tray
AppIndicator3 works natively in KDE Plasma 5.12+.

### Auto-start
Same as GNOME: `~/.config/autostart/keyrx-tray.desktop`

### Custom Shortcuts
System Settings → Shortcuts → Custom Shortcuts

## Troubleshooting

### Tray Icon Not Showing (GNOME)
```bash
# Install extension
sudo apt-get install gnome-shell-extension-appindicator
gnome-extensions enable appindicatorsupport@rgcjonas.gmail.com

# Restart GNOME Shell
Alt+F2 → type "r" → Enter

# Or relogin
```

### Tray Shows "Daemon Not Running"
```bash
# Check daemon status
sudo systemctl status keyrx

# Start daemon
sudo systemctl start keyrx

# Check port
curl http://127.0.0.1:9867/api/status
```

### Auto-start Not Working
```bash
# Check file exists
ls ~/.config/autostart/keyrx-tray.desktop

# Check executable
which keyrx-tray

# Test manually
keyrx-tray

# Check logs
journalctl --user -u keyrx-tray  # If using systemd --user
```

### Permission Errors
```bash
# Daemon needs root for evdev
sudo systemctl start keyrx

# Tray runs as user (no sudo)
keyrx-tray
```

## Uninstallation

### Remove Daemon
```bash
# Debian package
sudo systemctl stop keyrx
sudo systemctl disable keyrx
sudo dpkg -r keyrx

# Tarball
sudo systemctl stop keyrx
sudo systemctl disable keyrx
sudo rm /etc/systemd/system/keyrx.service
sudo systemctl daemon-reload
cd keyrx-* && sudo ./uninstall.sh
```

### Remove Tray
```bash
# Stop tray
pkill -f keyrx-tray

# Remove auto-start
rm ~/.config/autostart/keyrx-tray.desktop

# Remove desktop files
rm ~/.local/share/applications/keyrx*.desktop

# Remove binary
sudo rm /usr/local/bin/keyrx-tray
```

## Future Enhancements

- [ ] DBus interface for system integration
- [ ] GNOME Settings panel integration
- [ ] KDE System Settings module
- [ ] Wayland support (currently X11 only via evdev)
- [ ] App indicator with custom icons (enabled/disabled states)
- [ ] Notification history in tray menu
- [ ] Quick macro recording from tray
- [ ] Profile templates in tray menu

## Security Considerations

### Why Daemon Needs Root
- evdev requires root to grab devices (`/dev/input/eventX`)
- uinput requires root to create virtual keyboard
- Alternative: udev rules + `input` group (complex, security risk)

### Tray Security
- Tray runs as user (no elevated privileges)
- Communicates via localhost HTTP (not exposed to network)
- Web UI uses CORS restrictions

### systemd Security Hardening
The keyrx.service includes:
```ini
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/log/keyrx
```

## References

- [FreeDesktop Desktop Entry Spec](https://specifications.freedesktop.org/desktop-entry-spec/latest/)
- [systemd Service Files](https://www.freedesktop.org/software/systemd/man/systemd.service.html)
- [AppIndicator API](https://lazka.github.io/pgi-docs/AppIndicator3-0.1/index.html)
- [GNOME HIG](https://developer.gnome.org/hig/)
