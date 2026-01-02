# KeyRX UAT Guide

Complete guide for User Acceptance Testing with system tray and version information.

## Quick Start - Complete Rebuild

For a complete clean rebuild and restart:

```bash
# One command to rule them all
./scripts/uat_rebuild.sh
```

This script:
1. ✅ Stops all running daemons
2. ✅ Cleans build artifacts (UI + daemon)
3. ✅ Rebuilds WASM module
4. ✅ Rebuilds web UI with fresh assets
5. ✅ Rebuilds daemon with embedded UI
6. ✅ Starts daemon with system tray
7. ✅ Shows version information

## Version Information

### Check Version via API

```bash
curl http://localhost:9867/api/version | jq
```

**Response:**
```json
{
  "version": "0.1.0",
  "build_time": "2026-01-02T03:30:00+00:00",
  "git_hash": "b10857e",
  "platform": "linux"
}
```

### Version Fields

- `version`: Package version from Cargo.toml
- `build_time`: RFC3339 timestamp when daemon was built
- `git_hash`: Short commit hash (7 chars)
- `platform`: Target OS (linux/windows)

## System Tray Setup

### Why .desktop File Works But Command Line Doesn't

The `.desktop` file launched by GNOME automatically sets proper environment variables:
- `DISPLAY` - X11 display number (usually :1)
- `GDK_BACKEND` - Graphics backend (x11)
- `XDG_SESSION_TYPE` - Session type

When launching from SSH or terminal, these aren't set automatically.

### Install .desktop File

```bash
# Install to user applications
cp keyrx.desktop ~/.local/share/applications/

# Make daemon available globally
sudo ln -s /home/rmondo/repos/keyrx2/target/debug/keyrx_daemon /usr/local/bin/keyrx_daemon

# Update icon cache
gtk-update-icon-cache ~/.local/share/icons/ 2>/dev/null || true
update-desktop-database ~/.local/share/applications/
```

### Launch from GNOME

1. Press `Super` key
2. Type "KeyRx"
3. Click the KeyRx Daemon icon
4. System tray icon should appear in top-right

### Update .desktop After Rebuild

The .desktop file uses absolute paths. After rebuilding, either:

**Option A: Rebuild in place (recommended)**
```bash
./scripts/uat_rebuild.sh
# .desktop file still points to correct location
```

**Option B: Update paths**
Edit `~/.local/share/applications/keyrx.desktop` if you move the repo.

## Manual Launch with System Tray

If launching from terminal/SSH:

```bash
# Export required environment variables
export GDK_BACKEND=x11

# Auto-detect display (usually :1)
export DISPLAY=:1

# Start daemon
target/debug/keyrx_daemon run --config user_layout.krx
```

The UAT script handles this automatically.

## Troubleshooting

### System Tray Not Visible

**Check GNOME AppIndicator Extension:**
```bash
gnome-extensions list --enabled | grep appindicator
```

Expected output:
```
ubuntu-appindicators@ubuntu.com
```

**If not enabled:**
```bash
gnome-extensions enable ubuntu-appindicators@ubuntu.com
# Restart GNOME Shell: Alt+F2 → r → Enter
```

**Check daemon environment:**
```bash
DAEMON_PID=$(pgrep keyrx_daemon)
cat /proc/$DAEMON_PID/environ | tr '\0' '\n' | grep -E "DISPLAY|GDK_BACKEND"
```

Expected:
```
DISPLAY=:1
GDK_BACKEND=x11
```

### Web UI Shows Old Version

**Force rebuild:**
```bash
./scripts/uat_rebuild.sh
```

**Verify new UI is embedded:**
```bash
curl -s http://localhost:9867/ | grep '<script'
```

The asset hash should change after rebuild (e.g., `index-7bd6H6cj.js`).

**Clear browser cache:**
```bash
# Or press Ctrl+Shift+R in browser
```

### Daemon Won't Start

**Check logs:**
```bash
tail -50 /tmp/keyrx_daemon.log
```

**Check for device busy:**
```bash
# Kill any stuck daemons
pkill -9 keyrx_daemon

# Verify no processes holding devices
sudo lsof /dev/input/event* | grep keyrx
```

## Vagrant Windows VM - JST Timezone

### Configure Timezone

The Vagrantfile now automatically sets JST timezone on provision:

```bash
cd vagrant/windows
vagrant provision --provision-with set-timezone-jst
```

### Verify Timezone

```bash
vagrant winrm -c "Get-TimeZone"
```

Expected output:
```
Id                         : Tokyo Standard Time
DisplayName                : (UTC+09:00) Osaka, Sapporo, Tokyo
StandardName               : 東京 (標準時)
```

### Manual Timezone Change

If needed:
```bash
vagrant winrm -c "Set-TimeZone -Id 'Tokyo Standard Time'"
```

## Development Workflow

### Complete Rebuild Cycle

```bash
# 1. Stop daemon
pkill keyrx_daemon

# 2. Clean and rebuild everything
./scripts/uat_rebuild.sh

# 3. Check version
curl http://localhost:9867/api/version | jq

# 4. Test web UI
xdg-open http://localhost:9867

# 5. Verify system tray
# Look for keyboard icon in GNOME top bar
```

### Quick Daemon Restart (No Rebuild)

```bash
# Stop
pkill keyrx_daemon

# Start with tray
export GDK_BACKEND=x11 DISPLAY=:1
target/debug/keyrx_daemon run --config user_layout.krx
```

### UI-Only Rebuild

```bash
cd keyrx_ui
rm -rf dist node_modules/.vite
npm run build:wasm && npx vite build

# Rebuild daemon to embed new UI
cd ..
cargo build --bin keyrx_daemon
```

## Version History

To see version across builds:

```bash
# Get current version
curl -s http://localhost:9867/api/version

# Compare with git
git log --oneline -1

# Check build timestamp
curl -s http://localhost:9867/api/version | jq -r '.build_time'
```

## Integration with .desktop

The `.desktop` file now uses:
- **Icon**: `/home/rmondo/repos/keyrx2/keyrx_daemon/assets/icon.png`
- **Exec**: Absolute path to daemon binary
- **Path**: Project root for proper config loading

This ensures system tray always works when launched via GNOME.
