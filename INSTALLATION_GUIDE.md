# KeyRx v0.1.4 Installation Guide

## ⚠️ IMPORTANT: Binary Update Required

**Current Issue**: The MSI installer may package an old binary. After installation, you MUST update the binary manually.

**Quick Fix** (Takes 30 seconds):
1. Right-click `UPDATE_BINARY.ps1` → **Run as Administrator**
2. Wait for completion
3. Verify build date: http://localhost:9867 (right-click tray icon → About)

---

## Quick Install

### Method 1: MSI Installer + Manual Binary Update (RECOMMENDED)

1. **Run the installer:**
   ```
   target\installer\KeyRx-0.1.3-x64.msi
   ```

2. **Complete installation**
   - UAC will prompt for administrator permission → **Approve**
   - Daemon starts automatically
   - Web UI opens at http://localhost:9867

3. **Update the binary** (REQUIRED):
   - Right-click `UPDATE_BINARY.ps1` → **Run as Administrator**
   - This replaces the old binary with the latest v0.1.4 build

4. **Verify installation**:
   - Open http://localhost:9867
   - Right-click tray icon → About
   - Build date should be: **2026/01/29 15:41:11** (not 14:23:22)

### Method 2: Manual Installation

If MSI fails or you want full control:

```powershell
# 1. Stop existing daemon (as Administrator)
Stop-Process -Name keyrx_daemon -Force

# 2. Copy binary
Copy-Item "target\release\keyrx_daemon.exe" "C:\Program Files\KeyRx\bin\keyrx_daemon.exe" -Force

# 3. Start daemon
Start-Process "C:\Program Files\KeyRx\bin\keyrx_daemon.exe" -ArgumentList "run"

# 4. Verify
Invoke-RestMethod http://localhost:9867/api/health
```

## What's New in v0.1.4

### ✅ Critical Fixes for Keyboard Remapping

| Fix | File | Impact |
|-----|------|--------|
| **Spawn blocking for file I/O** | `config.rs`, `devices.rs`, `profiles.rs`, `layouts.rs` | Prevents async runtime blocking that caused remapping failures |
| **Thread-safe API handlers** | All API handlers | Allows concurrent requests without data races |
| **Race condition fixes** | `device_registry.rs` | Prevents state corruption during device enumeration |
| **Memory safety improvements** | Various | Eliminates undefined behavior in event processing |

**Why These Fixes Matter:**
- v0.1.2/v0.1.3: Keyboard remapping didn't work - keys passed through unchanged
- v0.1.4: Remapping works correctly with proper async I/O handling

### ⚠️ Known Issue: Installer Binary Mismatch

The MSI installer may package an outdated binary due to Windows file locking. **Solution**: Use `UPDATE_BINARY.ps1` after installation (see Quick Install above).

## Verification

### Check Daemon is Running

```powershell
# Method 1: Check process
Get-Process keyrx_daemon

# Method 2: Check API
curl http://localhost:9867/api/health
# Should return: {"status":"ok","version":"0.1.1"}

# Method 3: Open web UI
Start-Process "http://localhost:9867"
```

### Check Auto-Start is Configured

```powershell
schtasks /Query /TN "KeyRx Daemon" /V /FO LIST
```

Expected output:
```
TaskName:                      \KeyRx Daemon
Status:                        Ready
Run With Highest Privileges:   Yes
Triggers:                      At log on of any user
```

### Check Admin Rights

```powershell
# In PowerShell as Admin
$process = Get-Process keyrx_daemon
$process.GetAccessToken().ElevationType
# Should return: Full (elevated)
```

## Using KeyRx

### 1. Open Web UI

```
http://localhost:9867
```

Or click: **Start Menu → KeyRx Web UI**

### 2. Activate a Profile

1. Go to **Profiles** page
2. Click **Activate** on "default" profile (or create your own)
3. Wait for activation (compiles Rhai → .krx binary)
4. Green indicator shows active profile

### 3. Test Keyboard Remapping

1. Open any text editor (Notepad, VS Code, browser, etc.)
2. Type keys configured in your profile
3. See remapped output in real-time

### 4. View Metrics

1. Go to **Dashboard** page
2. See real-time event log
3. Monitor latency statistics
4. Check active profile status

## Troubleshooting

### Daemon Not Starting

**Problem:** No process found, API not responding

**Solutions:**
```powershell
# 1. Check if Task Scheduler entry exists
schtasks /Query /TN "KeyRx Daemon"

# 2. Manually start daemon
keyrx_daemon run
# (UAC will prompt for admin)

# 3. Check logs
Get-Content "C:\Program Files\KeyRx\daemon.log" -Tail 50

# 4. Reinstall with admin rights
# Right-click installer → Run as Administrator
```

### Remapping Not Working

**Problem:** Daemon running but keys not remapped

**Checklist:**
1. ✅ Is daemon running with admin rights?
   ```powershell
   Get-Process keyrx_daemon | Select-Object Path, @{Name="Elevated";Expression={$_.GetAccessToken().ElevationType}}
   ```

2. ✅ Is a profile activated?
   ```powershell
   curl http://localhost:9867/api/profiles/active
   # Should NOT return: {"active_profile":null}
   ```

3. ✅ Activate a profile:
   - Open http://localhost:9867
   - Go to Profiles page
   - Click "Activate" on default profile

4. ✅ Check configuration:
   ```powershell
   curl http://localhost:9867/api/config
   # Should show base_mappings and layers
   ```

### Gathering Diagnostic Information

**Problem:** Need to collect logs for troubleshooting

**Solution 1: Automated File Collection**
```powershell
.\GATHER_LOGS.ps1
# Creates: LOGS_YYYYMMDD_HHMMSS.txt with comprehensive diagnostics
```

**Solution 2: REST API Endpoints**
```powershell
# Health check
Invoke-RestMethod http://localhost:9867/api/health | ConvertTo-Json

# Active profile
Invoke-RestMethod http://localhost:9867/api/profiles | ConvertTo-Json

# Configuration
Invoke-RestMethod http://localhost:9867/api/config | ConvertTo-Json

# Recent events (last 20)
Invoke-RestMethod http://localhost:9867/api/metrics/events?count=20 | ConvertTo-Json

# Latency statistics
Invoke-RestMethod http://localhost:9867/api/metrics/latency | ConvertTo-Json

# Device information
Invoke-RestMethod http://localhost:9867/api/devices | ConvertTo-Json
```

**What GATHER_LOGS.ps1 Collects:**
- Daemon process info (PID, memory, start time)
- Binary info and timestamps (verifies correct version)
- API health check
- Active profile info
- Daemon log (last 100 lines)
- Config files listing
- Network port status (9867)
- Windows event log errors

**Manual Log Locations:**
- Daemon log: `%APPDATA%\keyrx\daemon.log` or `%USERPROFILE%\.keyrx\daemon.log`
- Config files: `%APPDATA%\keyrx\profiles\`

### Layer Contamination (Multiple Outputs)

**Problem:** Pressing 'a' outputs 'wa' or multiple characters

**This is being diagnosed.** See `REMAP_DIAGNOSIS_REPORT.md` for details.

**Quick test:**
```powershell
# 1. Clear metrics
curl -X DELETE http://localhost:9867/api/metrics/events

# 2. Type a single 'a' key on keyboard

# 3. Check what was captured
curl http://localhost:9867/api/metrics/events?count=20

# 4. Look for:
# - Multiple press events (should be 1)
# - Original key 'w' appearing (shouldn't happen)
# - Multiple remapped outputs (should be 1)
```

### UAC Prompts on Every Launch

**Problem:** Task Scheduler not configured

**Solution:**
Reinstall and CHECK "Auto-start daemon on Windows login" during install.

Or manually create Task Scheduler entry:
```powershell
schtasks /Create /TN "KeyRx Daemon" /TR "\"C:\Program Files\KeyRx\keyrx_daemon.exe\" run" /SC ONLOGON /RL HIGHEST /F /DELAY 0000:05
```

## Manual Installation (Advanced)

If you want full control:

### 1. Extract Files

```powershell
# Copy daemon to Program Files
Copy-Item "target\release\keyrx_daemon.exe" "C:\Program Files\KeyRx\"
Copy-Item "target\release\keyrx_compiler.exe" "C:\Program Files\KeyRx\"

# Add to PATH
$env:Path += ";C:\Program Files\KeyRx"
[Environment]::SetEnvironmentVariable("Path", $env:Path, [EnvironmentVariableTarget]::Machine)
```

### 2. Create Auto-Start Task

```powershell
schtasks /Create `
  /TN "KeyRx Daemon" `
  /TR "\"C:\Program Files\KeyRx\keyrx_daemon.exe\" run" `
  /SC ONLOGON `
  /RL HIGHEST `
  /F `
  /DELAY 0000:05
```

### 3. Start Daemon

```powershell
Start-Process "C:\Program Files\KeyRx\keyrx_daemon.exe" -ArgumentList "run" -Verb RunAs
```

## Uninstallation

### Using Installer

1. **Start Menu → KeyRx → Uninstall KeyRx**
2. Or: **Settings → Apps → KeyRx → Uninstall**

The uninstaller automatically:
- Stops the daemon
- Removes Task Scheduler entry
- Removes all files

### Manual Uninstall

```powershell
# 1. Stop daemon
Stop-Process -Name keyrx_daemon -Force

# 2. Remove Task Scheduler entry
schtasks /Delete /TN "KeyRx Daemon" /F

# 3. Remove files
Remove-Item "C:\Program Files\KeyRx" -Recurse -Force

# 4. Remove from PATH (if added manually)
# Edit: Control Panel → System → Advanced → Environment Variables
```

## Files and Locations

| Item | Location |
|------|----------|
| Daemon executable | `C:\Program Files\KeyRx\keyrx_daemon.exe` |
| Compiler | `C:\Program Files\KeyRx\keyrx_compiler.exe` |
| Config directory | `C:\Users\<username>\.config\keyrx\` |
| Profiles | `C:\Users\<username>\.config\keyrx\profiles\` |
| Compiled configs | `C:\Users\<username>\.config\keyrx\profiles\*.krx` |
| Logs | `C:\Program Files\KeyRx\daemon.log` |
| Task Scheduler | `Task Scheduler Library\KeyRx Daemon` |
| Registry keys | `HKLM\Software\KeyRx` |

## Command Line Usage

```bash
# Start daemon (requires admin)
keyrx_daemon run

# Compile Rhai config to .krx
keyrx_compiler user_layout.rhai

# Show help
keyrx_daemon --help
keyrx_compiler --help
```

## Web API

The daemon exposes a REST API at http://localhost:9867:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/health` | GET | Health check |
| `/api/status` | GET | Daemon status |
| `/api/profiles` | GET | List profiles |
| `/api/profiles/{name}/activate` | POST | Activate profile |
| `/api/profiles/active` | GET | Get active profile |
| `/api/config` | GET | Get active config |
| `/api/devices` | GET | List devices |
| `/api/metrics/events` | GET | Event log |
| `/api/metrics/latency` | GET | Latency stats |
| `/api/simulator/events` | POST | Simulate key events |

Full API documentation: http://localhost:9867 (when daemon is running)

## Support

- **Documentation**: https://github.com/RyosukeMondo/keyrx
- **Issues**: https://github.com/RyosukeMondo/keyrx/issues
- **Installer**: `target\windows-installer\keyrx_0.1.1.0_x64_setup.exe` (9.0 MB)

## Version History

### v0.1.1 (Current)
- ✅ Auto-start with administrator rights
- ✅ Task Scheduler integration
- ✅ 67 bug fixes (memory, WebSocket, API, security, validation)
- ✅ Comprehensive test suite (962/962 backend tests passing)
- ✅ Web UI improvements

### v0.1.0
- Initial release
- Manual admin elevation required
