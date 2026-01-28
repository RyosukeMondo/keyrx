# Windows Admin Requirements

## 🔑 Why Admin Rights Are Needed

KeyRx daemon requires **administrator privileges** to access keyboard input on Windows, similar to how it needs root/udev rules on Linux.

### What Needs Admin

| Component | Admin Required | Why |
|-----------|---------------|-----|
| **Installation** | ❌ No | Installs to user folder if not admin |
| **Running Daemon** | ✅ **Yes** | Needs low-level keyboard access |
| **Web UI** | ❌ No | Just a web browser |
| **Compiler** | ❌ No | Just compiles config files |

## 🚀 How to Run the Daemon

### Option 1: From Start Menu (Easy)

1. Click **Start Menu**
2. Find **KeyRx → KeyRx Daemon**
3. **Right-click** → **Run as administrator**
4. Accept UAC prompt

### Option 2: From Desktop Shortcut

1. **Right-click** on KeyRx shortcut
2. Select **Run as administrator**
3. Accept UAC prompt

### Option 3: From Command Line

```powershell
# Method 1: Using Start-Process
Start-Process "C:\Users\<username>\AppData\Local\Programs\KeyRx\keyrx_daemon.exe" -ArgumentList "run" -Verb RunAs

# Method 2: If in PATH
Start-Process keyrx_daemon.exe -ArgumentList "run" -Verb RunAs
```

### Option 4: Always Run as Admin (Recommended)

Make the shortcut always request admin:

1. **Right-click** on KeyRx shortcut
2. Select **Properties**
3. Click **Advanced**
4. Check **Run as administrator**
5. Click **OK**, **OK**

Now double-clicking will always prompt for admin.

## 📊 Installation Scenarios

### Scenario 1: Install Without Admin

```
✅ Installer runs successfully
✅ Installs to: %LOCALAPPDATA%\Programs\KeyRx
✅ Adds to user PATH
❗ Must run daemon as admin (right-click → Run as administrator)
```

### Scenario 2: Install With Admin (Recommended)

```
✅ Installer runs as admin
✅ Installs to: C:\Program Files\KeyRx
✅ Adds to system PATH
❗ Must still run daemon as admin (right-click → Run as administrator)
```

## 🎯 Best Practices

### For Regular Use

1. **Install with admin** (right-click installer → Run as administrator)
   - Installs to `C:\Program Files\KeyRx`
   - Available system-wide

2. **Create shortcut with "Always run as admin"**
   - Shortcut Properties → Advanced → Run as administrator
   - No need to right-click every time

3. **Pin to Taskbar**
   - Right-click shortcut → Pin to Taskbar
   - Single click will prompt for admin

### For Development/Testing

1. **Install without admin** for quick testing
2. **Use PowerShell with elevation:**
   ```powershell
   Start-Process keyrx_daemon.exe -ArgumentList "run" -Verb RunAs
   ```

3. **Or create a launch script:**
   ```powershell
   # launch-keyrx.ps1
   Start-Process "$PSScriptRoot\keyrx_daemon.exe" -ArgumentList "run" -Verb RunAs
   ```

## 🔧 Automatic Startup (Optional)

### Method 1: Task Scheduler (Recommended)

Create a scheduled task that runs on login with highest privileges:

```powershell
$action = New-ScheduledTaskAction -Execute "C:\Program Files\KeyRx\keyrx_daemon.exe" -Argument "run"
$trigger = New-ScheduledTaskTrigger -AtLogOn
$principal = New-ScheduledTaskPrincipal -UserId $env:USERNAME -LogonType Interactive -RunLevel Highest
Register-ScheduledTask -TaskName "KeyRx Daemon" -Action $action -Trigger $trigger -Principal $principal -Description "Start KeyRx keyboard remapper on login"
```

To remove:
```powershell
Unregister-ScheduledTask -TaskName "KeyRx Daemon" -Confirm:$false
```

### Method 2: Startup Folder (With Elevation Script)

Create `%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup\KeyRx.vbs`:

```vbscript
' KeyRx.vbs - Launch daemon with elevation
Set WshShell = CreateObject("WScript.Shell")
WshShell.Run "powershell -WindowStyle Hidden -Command ""Start-Process 'C:\Program Files\KeyRx\keyrx_daemon.exe' -ArgumentList 'run' -Verb RunAs""", 0
```

## ⚠️ Common Issues

### Issue 1: "CreateProcess failed; code 740"

**Error:** 要求された操作には管理者特権が必要です (Administrator privileges required)

**Solution:** Run as administrator
```powershell
# Right-click → Run as administrator
# Or:
Start-Process keyrx_daemon.exe -ArgumentList "run" -Verb RunAs
```

### Issue 2: UAC Prompt Every Time

**Solution:** Set shortcut to always run as admin (see Option 4 above)

### Issue 3: Daemon Not in PATH

**Solution:** Restart terminal or reload environment:
```powershell
$env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
```

### Issue 4: Web UI Can't Connect

**Cause:** Daemon not running (needs admin)

**Solution:**
1. Run daemon as admin: `Start-Process keyrx_daemon.exe -ArgumentList "run" -Verb RunAs`
2. Check it's running: `Get-Process keyrx_daemon`
3. Open browser: `Start-Process http://localhost:9867`

## 🔍 Technical Details

### Why Low-Level Access?

KeyRx intercepts keyboard input at a low level to:
- ✅ Capture keys before applications
- ✅ Implement tap-hold
- ✅ Support multi-device
- ✅ Provide consistent remapping

This requires:
- **Windows:** Administrator privileges
- **Linux:** Root or udev rules
- **macOS:** Accessibility permissions

### Windows API Requirements

The daemon uses:
- `SetWindowsHookEx` - Requires admin for global hooks
- Raw Input API - Requires admin for device access
- Registry access - For input device enumeration

### Security Considerations

KeyRx only:
- ✅ Runs locally (no network access required)
- ✅ Open source (audit the code)
- ✅ Does NOT log keystrokes
- ✅ Does NOT send data anywhere
- ✅ Only remaps keys as configured

**Always review keyboard remapping software before granting admin access.**

## 📋 Quick Reference

| Task | Command |
|------|---------|
| **Run daemon (GUI)** | Right-click shortcut → Run as administrator |
| **Run daemon (PowerShell)** | `Start-Process keyrx_daemon.exe -ArgumentList "run" -Verb RunAs` |
| **Check if running** | `Get-Process keyrx_daemon` |
| **Stop daemon** | `Stop-Process -Name keyrx_daemon` |
| **Open Web UI** | `Start-Process http://localhost:9867` |
| **Check version** | `keyrx_daemon.exe --version` (no admin needed) |

## 🎯 Recommendation

**For best experience:**

1. ✅ Install with administrator (right-click installer)
2. ✅ Create Start Menu shortcut with "Always run as admin"
3. ✅ Pin shortcut to Taskbar
4. ✅ (Optional) Set up Task Scheduler for auto-start

This gives you:
- One-click launch (just accepts UAC)
- Auto-start on login
- No manual elevation needed

## 📚 See Also

- `INSTALLER_FIX.md` - Why installation doesn't need admin
- `WINDOWS_BUILD_GUIDE.md` - Building from source
- `BUILD_COMPLETE.md` - Installation success guide

---

**Summary:** Installation = optional admin, Running daemon = **requires admin** 🔑
