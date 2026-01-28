# KeyRx Windows Troubleshooting Guide

## 🔍 No Keyboard Input Detected

### Symptoms
- Daemon is running
- Web UI accessible at http://localhost:9867
- Metrics page shows no key events
- Keyboard not being remapped

### Diagnosis

**Step 1: Check if daemon has admin privileges**

The daemon MUST run as administrator to intercept keyboard input on Windows.

```powershell
# Check if daemon is running as admin
$proc = Get-Process keyrx_daemon
$proc.ProcessName
# If no error, check its privileges (needs more complex check)

# Better: Just restart with admin
Stop-Process -Name keyrx_daemon -Force
Start-Process keyrx_daemon.exe -ArgumentList "run --debug" -Verb RunAs
```

**Step 2: Enable debug logging**

```powershell
# Use the debug launch script
.\scripts\windows\Debug-Launch.ps1

# Or manually with debug flag
Start-Process keyrx_daemon.exe -ArgumentList "run --debug" -Verb RunAs
```

**Step 3: Check debug logs**

Debug logs show:
- Raw Input registration status
- Keyboard hook installation
- Device discovery
- Key event processing

```powershell
# View real-time logs
Get-Content "$env:TEMP\keyrx-debug.log" -Tail 50 -Wait
```

### Common Issues

#### Issue 1: Daemon Not Running as Admin

**Symptom:** No keyboard events captured

**Fix:**
```powershell
# Stop daemon
Stop-Process -Name keyrx_daemon -Force

# Launch with admin
.\scripts\windows\Debug-Launch.ps1
```

#### Issue 2: Raw Input Not Registered

**Symptom:** Debug log shows "Failed to register Raw Input"

**Cause:** Windows API call failed

**Fix:** Check Windows version, restart daemon

#### Issue 3: Config Not Loaded

**Symptom:** Debug log shows "No active profile"

**Fix:**
```powershell
# Check active profile
Get-Content "$env:APPDATA\keyrx\settings.json"

# Set active profile via Web UI or CLI
keyrx_daemon.exe profiles activate user_layout
```

#### Issue 4: Wrong Config Path

**Symptom:** Debug log shows "Config file not found"

**Fix:**
```powershell
# Use absolute path
.\scripts\windows\Debug-Launch.ps1 -ConfigPath "C:\Users\<username>\repos\keyrx\examples\user_layout.rhai"
```

## 📋 Debug Checklist

When keyboard input isn't working:

- [ ] **Admin privileges**: Daemon running as administrator?
  ```powershell
  # Launch with admin
  Start-Process keyrx_daemon.exe -ArgumentList "run --debug" -Verb RunAs
  ```

- [ ] **Debug logging enabled**: Using `--debug` flag?
  ```powershell
  .\scripts\windows\Debug-Launch.ps1
  ```

- [ ] **Raw Input registered**: Check debug log for "Raw Input registered"
  ```powershell
  Get-Content "$env:TEMP\keyrx-debug.log" | Select-String "Raw Input"
  ```

- [ ] **Config loaded**: Check debug log for "Loaded profile"
  ```powershell
  Get-Content "$env:TEMP\keyrx-debug.log" | Select-String "profile"
  ```

- [ ] **Devices discovered**: Check debug log for "Device discovered"
  ```powershell
  Get-Content "$env:TEMP\keyrx-debug.log" | Select-String "Device"
  ```

- [ ] **Events received**: Check debug log for "KeyEvent"
  ```powershell
  Get-Content "$env:TEMP\keyrx-debug.log" | Select-String "KeyEvent"
  ```

## 🔧 Quick Fixes

### Reset Everything

```powershell
# Stop daemon
Stop-Process -Name keyrx_daemon -Force

# Clear settings
Remove-Item -Force "$env:APPDATA\keyrx\settings.json" -ErrorAction SilentlyContinue

# Launch with debug
.\scripts\windows\Debug-Launch.ps1 -ConfigPath examples\user_layout.rhai

# Check Web UI
Start-Process http://localhost:9867
```

### Force Rebuild and Test

```powershell
# Rebuild release binary
cargo build --release --features windows

# Stop old daemon
Stop-Process -Name keyrx_daemon -Force

# Test with debug
.\target\release\keyrx_daemon.exe run --debug --config examples\user_layout.rhai
```

### Check Windows Version Compatibility

```powershell
# Check Windows version
[System.Environment]::OSVersion

# KeyRx requires:
# - Windows 10 or later
# - Raw Input API support
```

## 📝 Debug Log Analysis

### What to Look For

**Successful startup:**
```
[INFO] Starting KeyRx daemon
[INFO] Raw Input registered for device...
[INFO] Device discovered: ...
[INFO] Loaded profile: user_layout
[INFO] Event loop started
[DEBUG] KeyEvent { code: 30, pressed: true ... }  ← KEY EVENTS!
```

**Problem indicators:**
```
[ERROR] Failed to register Raw Input
[ERROR] No active profile found
[WARN] Config file not found
[ERROR] Access denied (need admin)
```

### Enable RUST_LOG for Maximum Verbosity

```powershell
# Set environment variable
$env:RUST_LOG = "keyrx_daemon=trace,keyrx_core=trace"

# Launch daemon
keyrx_daemon.exe run --debug
```

## 🎯 Still Not Working?

### Collect Diagnostic Info

```powershell
# Create diagnostic report
$report = @{
    WindowsVersion = [System.Environment]::OSVersion.Version
    DaemonRunning = (Get-Process keyrx_daemon -ErrorAction SilentlyContinue) -ne $null
    DaemonPath = (where.exe keyrx_daemon)
    ActiveProfile = (Get-Content "$env:APPDATA\keyrx\settings.json" -ErrorAction SilentlyContinue)
    RecentLogs = (Get-Content "$env:TEMP\keyrx-debug.log" -Tail 100 -ErrorAction SilentlyContinue)
}

$report | ConvertTo-Json | Out-File "$env:TEMP\keyrx-diagnostic.json"
Write-Host "Diagnostic report saved to: $env:TEMP\keyrx-diagnostic.json"
```

### Test with Minimal Config

Create `test-minimal.rhai`:
```rhai
device_start("*");
map("VK_A", "VK_B");  // A -> B
device_end();
```

Test:
```powershell
keyrx_compiler.exe compile test-minimal.rhai -o test-minimal.krx
.\scripts\windows\Debug-Launch.ps1 -ConfigPath test-minimal.krx
# Press 'A' key, should output 'B'
```

## 📚 Log File Locations

| Log Type | Location | When Created |
|----------|----------|--------------|
| Debug logs | `%TEMP%\keyrx-debug.log` | With `--debug` flag |
| Crash dumps | `%LOCALAPPDATA%\keyrx\crashes` | On crash |
| IPC logs | `%TEMP%\keyrx-ipc-*.log` | IPC operations |

## 🔑 Admin Privilege Check

```powershell
# Check if current PowerShell has admin
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if ($isAdmin) {
    Write-Host "Running as Administrator" -ForegroundColor Green
} else {
    Write-Host "NOT running as Administrator" -ForegroundColor Red
    Write-Host "Daemon will NOT capture keyboard input!" -ForegroundColor Yellow
}
```

## 💡 Pro Tips

1. **Always use Debug-Launch.ps1** during troubleshooting - it handles admin elevation and logging automatically

2. **Watch logs in real-time:**
   ```powershell
   Get-Content "$env:TEMP\keyrx-debug.log" -Tail 50 -Wait
   ```

3. **Test keyboard input:**
   - Open Web UI metrics page
   - Press keys
   - Should see events in real-time

4. **If still no events:**
   - Daemon likely not running as admin
   - Or Raw Input registration failed
   - Check debug logs for errors

---

**Quick Summary:**
1. ✅ Run with admin: `.\scripts\windows\Debug-Launch.ps1`
2. ✅ Enable debug logging (done automatically by script)
3. ✅ Check logs: `Get-Content "$env:TEMP\keyrx-debug.log" -Tail 50 -Wait`
4. ✅ Open Web UI: http://localhost:9867
5. ✅ Press keys and verify metrics update
