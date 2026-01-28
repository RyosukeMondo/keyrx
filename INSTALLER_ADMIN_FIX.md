# Installer Administrator Rights Fix

## Problem

The KeyRx daemon needs administrator privileges to intercept keyboard events on Windows, but the installer wasn't properly configuring this.

## Root Cause

1. **Manifest exists and is correct** ✅
   - `keyrx_daemon.exe.manifest` has `level="requireAdministrator"`
   - Embedded in release builds via `build.rs`

2. **Shortcuts didn't auto-start with admin** ❌
   - Start Menu shortcuts just launched the exe
   - No UAC prompt
   - No auto-start on login

3. **User had to manually "Run as Administrator"** ❌
   - Not user-friendly
   - Easy to forget

## Solution Implemented

### 1. Windows Task Scheduler Auto-Start

Added Task Scheduler entry that:
- Runs on Windows login (`/SC ONLOGON`)
- Always elevated (`/RL HIGHEST`)
- 5-second delay to avoid boot conflicts (`/DELAY 0000:05`)
- User can enable/disable via installer checkbox

**Installer Code:**
```iss
[Tasks]
Name: "autostart"; Description: "Auto-start daemon on Windows login (with administrator privileges)"; Check: IsAdmin

[Run]
Filename: "schtasks.exe";
Parameters: "/Create /TN ""KeyRx Daemon"" /TR ""\""{app}\{#AppExeName}\"" run"" /SC ONLOGON /RL HIGHEST /F /DELAY 0000:05";
Flags: runhidden;
Tasks: autostart;
StatusMsg: "Creating auto-start task with administrator privileges..."
```

### 2. Launch Daemon After Install

Changed post-install launch to:
- **Checked by default** (was unchecked)
- Uses `shellexec` flag to trigger UAC prompt
- Starts daemon immediately if user agrees

### 3. Clean Uninstall

Added Task Scheduler cleanup:
```iss
[UninstallRun]
Filename: "schtasks.exe"; Parameters: "/Delete /TN ""KeyRx Daemon"" /F"; Flags: runhidden
```

## How It Works

### During Installation

1. **User runs installer** → UAC prompts for admin (if needed)
2. **Files copied** → Daemon with embedded admin manifest
3. **User selects "Auto-start daemon on Windows login"** ✅ (recommended)
4. **Installer creates Task Scheduler entry** → Runs as SYSTEM with HIGHEST privileges
5. **User selects "Launch KeyRx Daemon now"** ✅ (checked by default)
6. **UAC prompts** → User approves admin rights
7. **Daemon starts** → Keyboard interception active

### On Windows Login

1. **Windows starts** → Task Scheduler loads
2. **5-second delay** → Let system boot complete
3. **Task Scheduler launches daemon** → With HIGHEST privileges (no UAC prompt needed)
4. **Daemon runs** → Keyboard remapping active

### Manual Launch

If user unchecks auto-start, they can manually launch via:

1. **Start Menu → KeyRx Daemon** → Triggers UAC prompt
2. **Desktop shortcut** (if created) → Triggers UAC prompt
3. **Command line**: `keyrx_daemon run` → Triggers UAC prompt

All methods work because the manifest is embedded in the exe.

## Verification

### Check Task Scheduler Entry
```powershell
schtasks /Query /TN "KeyRx Daemon" /V /FO LIST
```

Expected output:
```
TaskName:           \KeyRx Daemon
Run As User:        SYSTEM
Task To Run:        "C:\Program Files\KeyRx\keyrx_daemon.exe" run
Status:             Ready
Logon Mode:         Interactive/Background
Run With Highest Privileges: Yes
Triggers:           At log on of any user
```

### Check Daemon Status
```powershell
# Check if daemon is running
Get-Process keyrx_daemon

# Check daemon API
curl http://localhost:9867/api/health

# Should return: {"status":"ok","version":"0.1.1"}
```

### Check Admin Rights
```powershell
# In PowerShell running as Admin
$process = Get-Process keyrx_daemon
$process.GetAccessToken().ElevationType

# Should return: Full (elevated)
```

## User Experience

### ✅ Before Fix
1. User installs KeyRx
2. User clicks "KeyRx Daemon" shortcut
3. **Daemon starts without admin → No keyboard interception** ❌
4. User confused why it's not working
5. User must manually "Run as Administrator"

### ✅ After Fix
1. User installs KeyRx
2. During install, checks "Auto-start daemon on Windows login" ✅
3. During install, checks "Launch KeyRx Daemon now" ✅
4. UAC prompts for admin → User approves
5. **Daemon starts with admin → Keyboard interception works** ✅
6. On next login, daemon auto-starts with admin (no UAC prompt)
7. User opens web UI → http://localhost:9867
8. Everything works seamlessly

## Files Changed

1. **`scripts/package/keyrx-installer.iss`**
   - Added `autostart` task checkbox
   - Added Task Scheduler creation in `[Run]`
   - Added Task Scheduler cleanup in `[UninstallRun]`
   - Changed post-install launch to checked by default

## Building New Installer

```powershell
# 1. Rebuild daemon with manifest
cd keyrx_daemon
cargo clean -p keyrx_daemon
cargo build --release

# 2. Build installer
cd ../scripts/package
iscc keyrx-installer.iss

# Output: target/windows-installer/keyrx_0.1.1_x64_setup.exe
```

## Testing

### Test Auto-Start

1. Install KeyRx
2. Check "Auto-start daemon on Windows login"
3. Check "Launch KeyRx Daemon now"
4. Verify daemon is running: `curl http://localhost:9867/api/health`
5. Reboot Windows
6. After login (5 seconds), verify daemon auto-started
7. Check Task Scheduler has entry

### Test Manual Launch

1. Install KeyRx
2. **Uncheck** "Auto-start daemon on Windows login"
3. **Uncheck** "Launch KeyRx Daemon now"
4. Click Start Menu → KeyRx Daemon
5. UAC prompts → Approve
6. Verify daemon is running
7. Close daemon
8. Click Start Menu → KeyRx Daemon again
9. UAC prompts again (expected, no auto-start)

### Test Uninstall

1. Run uninstaller
2. Verify Task Scheduler entry is removed:
   ```powershell
   schtasks /Query /TN "KeyRx Daemon"
   # Should error: "The system cannot find the file specified"
   ```
3. Verify daemon is stopped:
   ```powershell
   Get-Process keyrx_daemon
   # Should error: "Cannot find a process with the name"
   ```

## Security Note

The daemon runs with HIGHEST privileges (equivalent to administrator) because it needs to:
1. Install low-level keyboard hook (`SetWindowsHookEx` with `WH_KEYBOARD_LL`)
2. Inject keyboard events (`SendInput`)
3. Access all keyboard input system-wide

This is standard for keyboard remapping tools (same as AutoHotkey, SharpKeys, etc.).

The manifest explicitly requests this via:
```xml
<requestedExecutionLevel level="requireAdministrator" uiAccess="false"/>
```

## Alternative: Run as Service

For enterprise deployment, consider running as Windows Service:
- No UAC prompts
- Runs even when no user logged in
- More complex setup

**Not implemented yet** - Task Scheduler is simpler and sufficient for most users.
