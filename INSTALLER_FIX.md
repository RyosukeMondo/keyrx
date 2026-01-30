# Installer Fix - v0.1.4

## What Was Fixed

### Problem
The MSI installer was packaging old binaries because:
1. The daemon was running with administrator privileges
2. Windows prevented overwriting the binary during installation
3. The installer didn't stop the daemon before attempting to upgrade

### Solution
The WiX installer now includes **CustomActions** that automatically stop the daemon:

```xml
<!-- Stop daemon before uninstall/upgrade -->
<CustomAction Id="StopDaemonBeforeUpgrade"
              Execute="immediate"
              Return="ignore"
              Directory="TARGETDIR"
              ExeCommand="cmd.exe /c taskkill /F /IM keyrx_daemon.exe" />

<InstallExecuteSequence>
  <!-- Stop daemon before RemoveExistingProducts (upgrade) -->
  <Custom Action="StopDaemonBeforeUpgrade" Before="RemoveExistingProducts">Installed</Custom>
</InstallExecuteSequence>
```

**How it works:**
1. When upgrading: Stops daemon → Removes old version → Installs new version → Starts daemon
2. When uninstalling: Stops daemon → Removes files

**Benefits:**
- ✅ No manual scripts needed
- ✅ Handles upgrades automatically via MajorUpgrade element
- ✅ Always installs the latest binary from target\release\
- ✅ Clean uninstall without locked files

## How to Use the New Installer

### Quick Reinstall (RECOMMENDED)
```powershell
.\QUICK_REINSTALL.ps1
```

This will:
- Install KeyRx-0.1.4-x64.msi
- Installer automatically stops existing daemon
- Installs latest binary (2026/01/29 15:41:11)
- Starts daemon
- Tests API

### OR: Manual Installation
```powershell
msiexec /i "target\installer\KeyRx-0.1.4-x64.msi" /qn
```

## Verification
```powershell
# Check binary timestamp (should be 15:41:11, not 14:23:22)
Get-Item "C:\Program Files\KeyRx\bin\keyrx_daemon.exe" | Select-Object LastWriteTime

# Check daemon running
Get-Process -Name keyrx_daemon

# Check API
Invoke-RestMethod http://localhost:9867/api/health
```

## Summary of Changes

| File | Change |
|------|--------|
| `keyrx_daemon\keyrx_installer.wxs` | Added CustomActions to stop daemon, updated version to 0.1.4.0 |
| `scripts\build_windows_installer.ps1` | Updated default version to 0.1.4 |
| `INSTALL_LATEST.ps1` | Updated to use KeyRx-0.1.4-x64.msi |
| `QUICK_REINSTALL.ps1` | Simplified (no manual daemon stop) |

## What's Different from v0.1.3

| Aspect | v0.1.3 | v0.1.4 |
|--------|--------|--------|
| Daemon stopping | Manual | Automatic |
| Binary updates | Manual script needed | Handled by installer |
| Upgrades | Failed (locked files) | Works correctly |
| User steps | 2-3 manual steps | 1 step (run installer) |

The installer now works as expected - **no manual intervention required**.
