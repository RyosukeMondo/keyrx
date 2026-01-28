# KeyRx v0.1.1 Installation Guide

## Quick Install (Recommended)

1. **Run the installer:**
   ```
   target\windows-installer\keyrx_0.1.1.0_x64_setup.exe
   ```

2. **During installation, CHECK these options:**
   - ☑ **Auto-start daemon on Windows login** (recommended)
   - ☑ **Launch KeyRx Daemon now** (checked by default)

3. **Click Install**
   - UAC will prompt for administrator permission → **Approve**

4. **Daemon starts automatically**
   - Web UI opens at http://localhost:9867
   - Daemon runs with admin rights (required for keyboard interception)

5. **On next Windows login:**
   - Daemon auto-starts with admin rights (no UAC prompt)
   - Keyboard remapping works immediately

## What's Fixed in This Version

### ✅ Automatic Administrator Rights

**Before (v0.1.0):**
- User had to manually "Run as Administrator"
- Forgot? Remapping didn't work
- Had to do this every Windows login

**After (v0.1.1):**
- Installer creates Windows Task Scheduler entry
- Auto-starts with admin rights on every login
- No manual steps required

### ✅ One-Click Setup

The installer now handles everything:
1. ✅ Installs daemon with admin manifest
2. ✅ Creates Task Scheduler entry (auto-start)
3. ✅ Starts daemon immediately (with UAC prompt)
4. ✅ Opens web UI
5. ✅ Works on every login forever

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
