# Installer Troubleshooting Guide

Comprehensive guide to diagnosing and fixing KeyRx installation issues.

## Table of Contents

- [Quick Diagnostics](#quick-diagnostics)
- [Decision Tree](#decision-tree)
- [Common Issues](#common-issues)
- [Diagnostic Scripts](#diagnostic-scripts)
- [Admin Rights Issues](#admin-rights-issues)
- [Version Mismatch](#version-mismatch)
- [Port Conflicts](#port-conflicts)
- [File Lock Issues](#file-lock-issues)
- [Daemon Won't Start](#daemon-wont-start)
- [Clean Reinstall](#clean-reinstall)
- [Advanced Diagnostics](#advanced-diagnostics)

## Quick Diagnostics

### Step 1: Run Comprehensive Diagnostics

```powershell
# Full system diagnosis
.\scripts\diagnose-installation.ps1

# Auto-fix common issues
.\scripts\diagnose-installation.ps1 -AutoFix

# JSON output for analysis
.\scripts\diagnose-installation.ps1 -Json | jq
```

### Step 2: Check Version Consistency

```powershell
# Quick version check
.\scripts\version-check.ps1

# Expected: All versions match
# If not: Run sync-version.sh --fix
```

### Step 3: Verify Installation Health

```powershell
# Post-install validation
.\scripts\installer-health-check.ps1 -PostInstall

# Pre-install validation (before installing)
.\scripts\installer-health-check.ps1 -PreInstall
```

## Decision Tree

```
┌─────────────────────────────┐
│  Installation Issue?        │
└─────────────┬───────────────┘
              │
              ├─ Yes: Cannot Install MSI
              │  │
              │  ├─ "Access Denied" Error?
              │  │  ├─ YES → Issue: Admin Rights
              │  │  │         Fix: Run PowerShell as Administrator
              │  │  │
              │  │  ├─ NO → "File in Use" Error?
              │  │     ├─ YES → Issue: Daemon Running
              │  │     │         Fix: .\scripts\diagnose-installation.ps1 -AutoFix
              │  │     │
              │  │     └─ NO → "Version Mismatch" Error?
              │  │        ├─ YES → Issue: Stale Binary
              │  │        │         Fix: .\scripts\force-clean-reinstall.ps1
              │  │        │
              │  │        └─ NO → Run: .\scripts\installer-health-check.ps1 -PreInstall
              │  │
              │  └─ Installation Succeeds but...
              │     │
              │     ├─ Daemon Won't Start?
              │     │  ├─ Port Conflict? → netstat -ano | findstr "9867"
              │     │  ├─ Config Error? → Check: %USERPROFILE%\.keyrx\daemon.log
              │     │  └─ Permission Error? → Run as Administrator
              │     │
              │     └─ Wrong Version Displayed?
              │        ├─ Web UI? → Rebuild UI: cd keyrx_ui && npm run build
              │        ├─ Tray? → Rebuild daemon: cargo build --release
              │        └─ API? → Check: curl http://localhost:9867/api/version
              │
              └─ No: Daemon Issues After Install
                 │
                 ├─ Daemon Crashes on Startup?
                 │  ├─ Check logs: Get-Content "$env:USERPROFILE\.keyrx\daemon.log"
                 │  ├─ Check event viewer: eventvwr.msc → Application logs
                 │  └─ Run diagnostics: .\scripts\diagnose-installation.ps1
                 │
                 ├─ Keys Not Remapping?
                 │  ├─ Profile activated? → Check Web UI → Profiles
                 │  ├─ Hook installed? → Check: .\scripts\diagnose-installation.ps1
                 │  └─ Config valid? → Check: curl http://localhost:9867/api/config
                 │
                 └─ Web UI Not Loading?
                    ├─ Port conflict? → netstat -ano | findstr "9867"
                    ├─ Firewall blocking? → Add exception for port 9867
                    └─ Daemon running? → Get-Process -Name keyrx_daemon
```

## Common Issues

### Issue 1: "Access Denied" During Installation

**Full Error Message:**
```
Error 1925. You do not have sufficient privileges to complete this installation.
Contact your system administrator.
```

**Symptoms:**
- MSI installation fails immediately
- Error code 1925 or 1920
- "Access Denied" in error message

**Root Cause:**
- PowerShell not running with Administrator privileges
- Windows UAC blocking installation

**Diagnosis:**
```powershell
# Check if running as admin
$currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
$isAdmin = $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if ($isAdmin) {
    Write-Host "Running as Administrator: YES" -ForegroundColor Green
} else {
    Write-Host "Running as Administrator: NO" -ForegroundColor Red
}
```

**Fix:**

1. **Option 1: Run PowerShell as Administrator**
   ```
   1. Right-click on PowerShell
   2. Select "Run as Administrator"
   3. Navigate to project directory
   4. Retry installation
   ```

2. **Option 2: Use Run As Administrator from Explorer**
   ```
   1. Right-click on KeyRx-0.1.5-x64.msi
   2. Select "Run as administrator"
   3. Follow installation wizard
   ```

3. **Option 3: Command Line with Elevation**
   ```powershell
   Start-Process powershell -Verb runAs -ArgumentList "-Command", "msiexec /i target\installer\KeyRx-0.1.5-x64.msi /qn"
   ```

**Prevention:**
- Always run installation scripts from elevated PowerShell
- Add UAC prompt to installer (future enhancement)

---

### Issue 2: "File in Use" Error

**Full Error Message:**
```
Error 1920. Service 'KeyRx Daemon' failed to start.
Error 1603. Fatal error during installation.
Another version of this product is already installed.
```

**Symptoms:**
- Installation fails during upgrade
- "File is being used by another process"
- Cannot overwrite `keyrx_daemon.exe`

**Root Cause:**
- Daemon process still running
- File handle held by Windows Explorer
- Antivirus software locking file

**Diagnosis:**
```powershell
# Check if daemon is running
Get-Process -Name keyrx_daemon -ErrorAction SilentlyContinue

# Check file locks
.\scripts\diagnose-installation.ps1

# Look for:
# "Process holding binary: keyrx_daemon.exe (PID: 12345)"
```

**Fix:**

1. **Option 1: Auto-Fix (Recommended)**
   ```powershell
   .\scripts\diagnose-installation.ps1 -AutoFix
   ```

2. **Option 2: Manual Stop**
   ```powershell
   # Graceful stop
   Stop-Process -Name keyrx_daemon

   # Force stop if needed
   Stop-Process -Name keyrx_daemon -Force

   # Verify stopped
   Get-Process -Name keyrx_daemon -ErrorAction SilentlyContinue
   # Should show nothing
   ```

3. **Option 3: Complete Clean Reinstall**
   ```powershell
   .\scripts\force-clean-reinstall.ps1
   ```

**Prevention:**
- Installer now automatically stops daemon (v0.1.4+)
- Use retry logic with timeout (implemented in Task 8)

---

### Issue 3: Version Mismatch

**Full Error Message:**
```
ERROR: Version mismatch detected
Binary version: 0.1.4
MSI version: 0.1.5

This installer contains the wrong binary.
Please rebuild the installer with the correct version.
```

**Symptoms:**
- Installation fails during pre-flight check
- Error mentions version numbers
- Binary timestamp very old

**Root Cause:**
- Stale binary packaged in MSI
- Source files not synchronized
- Build cache not cleared

**Diagnosis:**
```powershell
# Check all version sources
.\scripts\version-check.ps1

# Expected output shows mismatches:
# Cargo.toml          0.1.5      ✓
# package.json        0.1.4      ✗  <- MISMATCH
# Source Binary       0.1.4      ✗  <- MISMATCH
```

**Fix:**

1. **Option 1: Sync Versions and Rebuild**
   ```bash
   # Sync all version sources
   ./scripts/sync-version.sh 0.1.5

   # Clean rebuild
   cargo clean
   cd keyrx_ui && npm run build:wasm && npm run build && cd ..
   cargo build --release -p keyrx_daemon

   # Rebuild installer
   .\scripts\build_windows_installer.ps1
   ```

2. **Option 2: Force Clean Reinstall**
   ```powershell
   .\scripts\force-clean-reinstall.ps1
   ```

**Prevention:**
- Use `sync-version.sh` for all version updates
- Enable build-time validation (implemented in Task 2)
- Run CI version check (implemented in Task 16)

---

### Issue 4: Port 9867 Already in Use

**Full Error Message:**
```
ERROR: Failed to bind to 0.0.0.0:9867
Error: Address already in use (os error 10048)
```

**Symptoms:**
- Daemon starts but API doesn't respond
- "Address already in use" in logs
- `netstat` shows port 9867 taken

**Root Cause:**
- Another application using port 9867
- Previous daemon instance still running
- Port not released cleanly

**Diagnosis:**
```powershell
# Check what's using port 9867
netstat -ano | findstr "9867"

# Example output:
# TCP    0.0.0.0:9867    0.0.0.0:0    LISTENING    12345
#                                                   ^^^^^
#                                                   Process ID

# Find the process
Get-Process -Id 12345

# Check if it's keyrx_daemon
Get-Process -Name keyrx_daemon
```

**Fix:**

1. **Option 1: Stop Conflicting Process**
   ```powershell
   # If it's old keyrx_daemon
   Stop-Process -Name keyrx_daemon -Force

   # If it's another app
   Stop-Process -Id 12345 -Force
   ```

2. **Option 2: Change Port (Temporary)**
   ```bash
   # Edit daemon config
   vim ~/.keyrx/daemon_config.toml

   [server]
   port = 9868  # Use different port

   # Restart daemon
   ```

3. **Option 3: Reboot**
   ```powershell
   # Cleanest solution if port won't release
   Restart-Computer
   ```

**Prevention:**
- Daemon now detects port conflicts on startup
- Graceful shutdown releases port properly
- Consider making port configurable via CLI flag

---

### Issue 5: Stale Binary Timestamp

**Full Error Message:**
```
WARNING: Binary is stale
Binary timestamp: 2026-01-28 10:00:00
Age: 30 hours

This may not be the latest build.
Consider rebuilding before installation.
```

**Symptoms:**
- Web UI shows old version
- System tray "About" shows old build time
- Binary file timestamp is > 24 hours old

**Root Cause:**
- Binary not rebuilt after code changes
- Incremental compilation cached old binary
- Wrong binary copied to installer

**Diagnosis:**
```powershell
# Check binary timestamps
Get-Item target\release\keyrx_daemon.exe | Select-Object LastWriteTime
Get-Item "C:\Program Files\KeyRx\bin\keyrx_daemon.exe" | Select-Object LastWriteTime

# Check build time via API
curl http://localhost:9867/api/version | jq '.build_time'
```

**Fix:**

1. **Option 1: Force Rebuild**
   ```bash
   # Clean all cached artifacts
   cargo clean
   rm -rf target/

   # Rebuild daemon
   cargo build --release -p keyrx_daemon

   # Verify timestamp
   Get-Item target\release\keyrx_daemon.exe | Select-Object LastWriteTime
   # Should show current time
   ```

2. **Option 2: Use SSOT Rebuild Script**
   ```bat
   # Windows only
   .\REBUILD_SSOT.bat
   ```

3. **Option 3: Complete Clean Reinstall**
   ```powershell
   .\scripts\force-clean-reinstall.ps1
   ```

**Prevention:**
- Installer now validates binary age (Task 7)
- Use `cargo clean` before release builds
- CI builds always start from clean state

---

### Issue 6: Config File Errors

**Full Error Message:**
```
ERROR: Failed to load configuration
Error: TOML parse error at line 15, column 3
```

**Symptoms:**
- Daemon crashes on startup
- "Failed to load configuration" in logs
- Web UI shows error banner

**Root Cause:**
- Invalid TOML syntax in daemon_config.toml
- Missing required fields
- Corrupted config file

**Diagnosis:**
```powershell
# Check config file
Get-Content "$env:USERPROFILE\.keyrx\daemon_config.toml"

# Validate TOML syntax
# (Use online validator or Rust tool)
```

**Fix:**

1. **Option 1: Reset to Default**
   ```powershell
   # Backup current config
   Copy-Item "$env:USERPROFILE\.keyrx\daemon_config.toml" "$env:USERPROFILE\.keyrx\daemon_config.toml.bak"

   # Delete config (daemon will create default)
   Remove-Item "$env:USERPROFILE\.keyrx\daemon_config.toml"

   # Restart daemon
   Stop-Process -Name keyrx_daemon -Force
   Start-Process "C:\Program Files\KeyRx\bin\keyrx_daemon.exe" -ArgumentList "run"
   ```

2. **Option 2: Fix Syntax Error**
   ```powershell
   # Edit config file
   notepad "$env:USERPROFILE\.keyrx\daemon_config.toml"

   # Common errors:
   # - Missing quotes around strings
   # - Invalid escape sequences
   # - Duplicate keys
   ```

3. **Option 3: Validate Config via API**
   ```bash
   # After fixing, validate
   curl http://localhost:9867/api/config/validate
   ```

**Prevention:**
- Implement config schema validation
- Provide config editor in Web UI
- Add `--validate-config` CLI flag

## Diagnostic Scripts

### diagnose-installation.ps1

**Purpose:** Comprehensive system diagnostics

**Usage:**
```powershell
# Full diagnosis
.\scripts\diagnose-installation.ps1

# Auto-fix common issues
.\scripts\diagnose-installation.ps1 -AutoFix

# JSON output
.\scripts\diagnose-installation.ps1 -Json
```

**What It Checks:**

1. **System Information**
   - OS version
   - User and admin status
   - PowerShell version

2. **Version Consistency**
   - All source files (Cargo.toml, package.json, WiX, etc.)
   - Source binary version
   - Installed binary version
   - Running daemon version (via API)

3. **File Analysis**
   - Binary exists and correct size
   - File timestamps
   - File locks (processes holding files)

4. **Process Analysis**
   - Daemon running status
   - Process uptime
   - Admin privileges

5. **Network Analysis**
   - Port 9867 availability
   - API health check
   - WebSocket connectivity

6. **Event Log Analysis**
   - Recent daemon errors
   - Installation events
   - System errors

**Example Output:**
```
========================================
 KeyRx Installation Diagnostics
========================================

[System Information]
  OS: Microsoft Windows NT 10.0.19045.0
  User: developer
  Admin: True

[Version Analysis]
  Cargo.toml: 0.1.5
  package.json: 0.1.5
  WiX installer: 0.1.5.0
  Source binary: 0.1.5
  Installed binary: 0.1.5
  Running daemon: 0.1.5
  ✓ All versions consistent

[File Analysis]
  Source binary: target\release\keyrx_daemon.exe
    Size: 12.5 MB
    Timestamp: 2026-01-30 10:00:00
    Age: 2 hours
  Installed binary: C:\Program Files\KeyRx\bin\keyrx_daemon.exe
    Size: 12.5 MB
    Timestamp: 2026-01-30 10:05:00
    Age: 2 hours

[Process Analysis]
  Daemon process: keyrx_daemon.exe (PID: 12345)
    Status: Running
    Uptime: 2 hours
    Admin: True
    CPU: 0.1%
    Memory: 45 MB

[Network Analysis]
  Port 9867: Available
  API health: OK
  WebSocket: Connected

[Issues Found]
  None

[Suggestions]
  ✓ Installation healthy
  ✓ All checks passed
```

### installer-health-check.ps1

**Purpose:** Pre/post-install validation

**Usage:**
```powershell
# Full check (pre + post)
.\scripts\installer-health-check.ps1

# Pre-install only
.\scripts\installer-health-check.ps1 -PreInstall

# Post-install only
.\scripts\installer-health-check.ps1 -PostInstall

# JSON output
.\scripts\installer-health-check.ps1 -Json
```

**Pre-Install Checks:**
1. Admin rights available
2. MSI file exists
3. MSI version matches binary
4. Binary timestamp fresh (< 24 hours)
5. All files present in MSI

**Post-Install Checks:**
1. Binary installed correctly
2. Daemon starts successfully
3. API responds to health checks
4. Profiles endpoint functional
5. Configuration loaded

### version-check.ps1

**Purpose:** Quick version consistency check

**Usage:**
```powershell
.\scripts\version-check.ps1
```

**Checks:**
- Cargo.toml version
- package.json version
- WiX installer version
- Source binary version
- Installed binary version
- Running daemon version

### force-clean-reinstall.ps1

**Purpose:** Complete clean reinstall

**Usage:**
```powershell
# Interactive (prompts for confirmation)
.\scripts\force-clean-reinstall.ps1

# Auto-confirm
.\scripts\force-clean-reinstall.ps1 -Force

# Skip UI rebuild
.\scripts\force-clean-reinstall.ps1 -SkipUiBuild
```

**Steps:**
1. Stop daemon (graceful, then force)
2. Uninstall MSI
3. Remove state files (`~/.keyrx/*`)
4. Clean build artifacts
5. Rebuild UI
6. Rebuild daemon
7. Build installer
8. Install MSI
9. Verify installation

## Admin Rights Issues

### Why Admin Rights Are Needed

KeyRx requires administrator privileges for:

1. **Low-Level Keyboard Hook**
   - Windows API: `SetWindowsHookExW(WH_KEYBOARD_LL, ...)`
   - Requires admin to intercept global keyboard events

2. **Raw Input Processing**
   - Access to `WM_INPUT` messages
   - Requires admin for system-wide input capture

3. **File Installation**
   - Installing to `C:\Program Files\KeyRx\`
   - System directory requires admin write access

4. **Service Registration** (Future)
   - Running as Windows service
   - Requires admin for service creation

### Checking Admin Status

```powershell
# Check if current PowerShell has admin rights
$currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
$isAdmin = $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if ($isAdmin) {
    Write-Host "✓ Running as Administrator" -ForegroundColor Green
} else {
    Write-Host "✗ NOT running as Administrator" -ForegroundColor Red
    Write-Host "  Right-click PowerShell and select 'Run as Administrator'" -ForegroundColor Yellow
}
```

### Checking Daemon Process Admin Status

```powershell
# Check if daemon process has admin rights
$process = Get-Process -Name keyrx_daemon -ErrorAction SilentlyContinue
if ($process) {
    $processToken = [Microsoft.Win32.SafeHandles.SafeAccessTokenHandle]::new($process.Handle)
    $elevationType = [System.Security.Principal.TokenElevationType]::Limited

    # Simplified check
    if ($process.StartInfo.Verb -eq "runas") {
        Write-Host "✓ Daemon running with admin rights" -ForegroundColor Green
    } else {
        Write-Host "⚠ Daemon may not have admin rights" -ForegroundColor Yellow
    }
}
```

### Starting Daemon with Admin Rights

```powershell
# Option 1: Start-Process with -Verb RunAs
Start-Process "C:\Program Files\KeyRx\bin\keyrx_daemon.exe" -Verb RunAs -ArgumentList "run"

# Option 2: Via installer (automatically starts with admin)
msiexec /i KeyRx-0.1.5-x64.msi /qn

# Option 3: Create shortcut with "Run as administrator"
# 1. Right-click keyrx_daemon.exe → Create shortcut
# 2. Right-click shortcut → Properties
# 3. Advanced → Run as administrator
```

## Version Mismatch Resolution

See [version-management.md](version-management.md) for detailed version management procedures.

**Quick Fix:**
```bash
# Sync all versions
./scripts/sync-version.sh 0.1.5

# Verify
./scripts/sync-version.sh --check

# Rebuild
cargo clean
cargo build --release
```

## Port Conflicts

**Check Port Usage:**
```powershell
# Find what's using port 9867
netstat -ano | findstr "9867"

# Output:
# TCP    0.0.0.0:9867    0.0.0.0:0    LISTENING    12345
#                                                   ^^^^^
#                                                   PID
```

**Kill Process:**
```powershell
# Find process by PID
Get-Process -Id 12345

# Stop it
Stop-Process -Id 12345 -Force
```

**Change Port (if needed):**
```toml
# Edit: ~/.keyrx/daemon_config.toml
[server]
port = 9868
```

## File Lock Issues

**Check File Locks:**
```powershell
# Using diagnose-installation.ps1
.\scripts\diagnose-installation.ps1

# Look for:
# "Process holding binary: ..."
```

**Manual Check:**
```powershell
# Get all processes
Get-Process | Where-Object {
    $_.Modules | Where-Object {
        $_.FileName -eq "C:\Program Files\KeyRx\bin\keyrx_daemon.exe"
    }
}
```

**Release Locks:**
```powershell
# Stop all processes holding the file
Stop-Process -Name keyrx_daemon -Force

# If that doesn't work, use Process Explorer (Sysinternals)
# Or reboot (nuclear option)
```

## Daemon Won't Start

### Check Logs

```powershell
# View daemon logs
Get-Content "$env:USERPROFILE\.keyrx\daemon.log" -Tail 50

# Watch logs in real-time
Get-Content "$env:USERPROFILE\.keyrx\daemon.log" -Wait -Tail 0
```

### Check Event Viewer

```powershell
# Open Event Viewer
eventvwr.msc

# Navigate to:
# Windows Logs → Application
# Filter for "keyrx" or errors around daemon start time
```

### Common Causes

1. **Port Conflict** → See [Port Conflicts](#port-conflicts)
2. **Missing Config** → Daemon creates default on first run
3. **Corrupted Config** → See Issue 6 above
4. **Permission Denied** → Run with admin rights
5. **Missing Dependencies** → Reinstall Visual C++ Redistributable

### Manual Start

```powershell
# Start directly (see errors in console)
& "C:\Program Files\KeyRx\bin\keyrx_daemon.exe" run

# Start with debug logging
$env:RUST_LOG = "debug"
& "C:\Program Files\KeyRx\bin\keyrx_daemon.exe" run
```

## Clean Reinstall

When all else fails, perform a complete clean reinstall:

```powershell
.\scripts\force-clean-reinstall.ps1
```

This will:
1. Stop daemon (force if needed)
2. Uninstall existing MSI
3. Delete all state files
4. Clean build artifacts
5. Rebuild everything from scratch
6. Install fresh MSI
7. Verify installation

**Warning:** This will delete all profiles and configuration!

## Advanced Diagnostics

### Check Windows Event Log

```powershell
# Get recent application errors
Get-EventLog -LogName Application -EntryType Error -Newest 10 | Where-Object {
    $_.Source -like "*keyrx*" -or $_.Message -like "*keyrx*"
}
```

### Check Binary Integrity

```powershell
# Get file hash
Get-FileHash "C:\Program Files\KeyRx\bin\keyrx_daemon.exe" -Algorithm SHA256

# Compare with source
Get-FileHash "target\release\keyrx_daemon.exe" -Algorithm SHA256

# Should match if installation is correct
```

### Check API Response

```powershell
# Health check
Invoke-RestMethod http://localhost:9867/api/health

# Full diagnostics (when implemented)
Invoke-RestMethod http://localhost:9867/api/diagnostics

# Version info
Invoke-RestMethod http://localhost:9867/api/version
```

### Network Trace

```powershell
# Test WebSocket connection
Test-NetConnection -ComputerName localhost -Port 9867

# Check firewall rules
Get-NetFirewallRule | Where-Object { $_.DisplayName -like "*keyrx*" }

# Add firewall rule if needed
New-NetFirewallRule -DisplayName "KeyRx Daemon" -Direction Inbound -LocalPort 9867 -Protocol TCP -Action Allow
```

## Related Documentation

- **[version-management.md](version-management.md)** - Version update procedures
- **[.spec-workflow/specs/installer-debuggability-enhancement/README.md](../.spec-workflow/specs/installer-debuggability-enhancement/README.md)** - Complete spec
- **[SSOT_VERSION.md](../SSOT_VERSION.md)** - SSOT principles
- **[INSTALLER_FIX.md](../INSTALLER_FIX.md)** - v0.1.4 installer improvements
- **[CRITICAL_DIAGNOSIS.md](../CRITICAL_DIAGNOSIS.md)** - Key blocking issue analysis

## Support

For additional help:

1. Run: `.\scripts\diagnose-installation.ps1 -Json`
2. Save output to file
3. Open GitHub issue with diagnostic output
4. Include:
   - Windows version
   - PowerShell version
   - Error messages
   - Diagnostic script output
   - Log file contents
