# Windows Web UI Access

## Quick Access

The keyrx daemon starts a web server automatically on **port 9867**.

### Option 1: Auto-Launch (Recommended)

Double-click **`keyrx_launch`** on Desktop - automatically opens the Web UI!

### Option 2: System Tray (Interactive Sessions Only)

When running in an interactive desktop session (RDP/console):
- Right-click the system tray icon
- Select "Open Web UI"

**Note**: System tray is only available in interactive desktop sessions. In headless/WinRM sessions, the daemon runs without a tray icon but Web UI remains accessible.

### Option 3: Manual Browser Access

While the daemon is running, open your browser to:
```
http://127.0.0.1:9867
```

### Option 4: Quick Open Script

Double-click `open_webui.bat` in `C:\Program Files\keyrx\`

## Web UI Features

The Web UI provides:
- ✅ **Visual Configuration Editor** - Edit key mappings graphically
- ✅ **Device Management** - See connected keyboards
- ✅ **Layer Editor** - Create and manage layers
- ✅ **Real-time Testing** - Test remapping live
- ✅ **Configuration Import/Export** - Save and load configs
- ✅ **Live Event Monitoring** - See key events in real-time

## Troubleshooting

### Web UI Won't Load

**Check daemon is running:**
```powershell
# In PowerShell
Get-Process keyrx_daemon
```

**Check web server started:**
Look for this in daemon output:
```
[INFO] Starting web server on http://127.0.0.1:9867
```

**Manual browser access:**
```
http://127.0.0.1:9867
```

### Port Already in Use

If port 9867 is taken, the daemon will fail to start web server.

**Check what's using the port:**
```powershell
netstat -ano | findstr :9867
```

**Kill the process:**
```powershell
taskkill /PID <pid> /F
```

### Firewall Blocking

If you can't access from another device on your network:

```powershell
# Add firewall rule (as Administrator)
New-NetFirewallRule -DisplayName "keyrx Web UI" -Direction Inbound -Protocol TCP -LocalPort 9867 -Action Allow
```

## System Tray Icon Warnings (Normal in Headless Sessions)

If you see warnings like:
```
[WARN] Failed to create system tray icon (this is normal in headless/WinRM sessions)
[INFO] Daemon will continue without system tray. Web UI is available at http://127.0.0.1:9867
```

**This is NORMAL when running via:**
- ✅ **WinRM/SSH** - No interactive desktop environment
- ✅ **Headless mode** - Background service without GUI
- ✅ **Remote automation** - Automated deployment scripts

**What it means:**
- System tray icons require an interactive desktop session
- Daemon continues running normally without the tray
- Web UI remains fully accessible at http://127.0.0.1:9867
- All functionality works except the tray menu shortcuts

**To get the system tray icon:**
- Connect via RDP or use console (virt-manager)
- Run the daemon from an interactive PowerShell session
- The tray icon will appear with "Open Web UI", "Reload Config", and "Exit" options

## RawInput Device Errors (Normal on Windows)

If you see errors like:
```
[ERROR] Failed to add new device: GetRawInputDeviceInfoW failed
```

**This is NORMAL on Windows**, especially in:
- ✅ **RDP sessions** - Virtual RDP input devices
- ✅ **Virtual machines** - QEMU/libvirt input devices
- ✅ **Virtual keyboards** - Software input emulators

**What it means:**
- Windows RawInput API reports devices that aren't real keyboards
- keyrx tries to query them but Windows API fails (expected)
- keyrx continues working fine - only real keyboards are captured

**Impact:**
- ✅ **NO impact on functionality** - real keyboards work fine
- ✅ keyrx filters out these phantom devices automatically
- ✅ Only physical/usable keyboards are remapped

**When to worry:**
- ❌ If **ALL** devices fail to be added
- ❌ If your **physical keyboard** isn't being remapped
- ❌ If you see **crashes** or **hangs**

**In RDP sessions:**
These errors are **100% expected** because RDP creates virtual input devices that Windows refuses to provide info about.

## Architecture

```
keyrx_daemon (Port 9867)
  ├─ RawInput Manager → Captures keyboard events
  ├─ Remapping Engine → Applies user_layout.krx rules
  ├─ Web Server (Axum) → Serves Web UI
  │   ├─ REST API → /api/*
  │   ├─ WebSocket → /ws (real-time events)
  │   └─ Static Files → Embedded keyrx_ui
  └─ Output Injection → Sends remapped keys
```

## UI Consolidation (SSOT Achieved)

**Previous state:**
- ❌ `keyrx_ui` - Obsolete (removed)
- ❌ `keyrx_ui_v2` - Active but confusing naming

**Current state (after consolidation):**
- ✅ `keyrx_ui` - **SINGLE** UI (renamed from v2)
- ✅ SSOT - One UI, one source of truth
- ✅ SRP - Single responsibility: configuration management

**What was removed:**
- Old `keyrx_ui` directory (399 MB) - obsolete React UI
- All references to `keyrx_ui_v2` in code

**What remains:**
- `keyrx_ui` - Modern React 19 + TypeScript UI
- Embedded in daemon at compile time
- Served at `http://127.0.0.1:9867`

## Files

**In `C:\Program Files\keyrx\`:**
- `launch.bat` - Start daemon + auto-open Web UI ⭐
- `open_webui.bat` - Open Web UI in browser
- `keyrx_daemon.exe` - Main daemon (includes embedded UI)
- `keyrx_compiler.exe` - Config compiler
- `user_layout.rhai` - Example configuration

**On Desktop:**
- `keyrx_launch.lnk` - Quick launch shortcut

## Summary

✅ **Web UI automatically opens** when you run `launch.bat`
✅ **Manual access:** `http://127.0.0.1:9867`
✅ **RawInput errors are normal** in RDP/VM environments
✅ **SSOT achieved:** Only one UI (`keyrx_ui`)
✅ **SRP achieved:** UI focused on configuration only
