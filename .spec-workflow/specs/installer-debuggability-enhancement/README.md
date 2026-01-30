# Installer & Debuggability Enhancement

## Overview

This specification addresses critical pain points in version management, installer reliability, and debuggability that caused repeated issues in past releases (v0.1.1-v0.1.5).

**Problem:** Past releases suffered from:
- Manual version synchronization failures (Cargo.toml ≠ package.json)
- Stale binaries being packaged in installers
- Admin rights complications preventing updates
- Difficult-to-diagnose version mismatches
- No fail-fast validation catching issues early

**Solution:** Implement bulletproof automation that makes version mismatches **impossible** and makes debugging **obvious**.

## Implementation Status

✅ **COMPLETE** - All 17 tasks implemented and tested.

### Phase 1: Version Synchronization (SSOT) ✅
- ✅ Task 1: Version synchronization script (`scripts/sync-version.sh`)
- ✅ Task 2: Build-time validation in `build.rs`
- ✅ Task 3: Runtime verification script (`scripts/version-check.ps1`)

### Phase 2: Enhanced Health Checks ✅
- ✅ Task 4: `/api/diagnostics` endpoint
- ✅ Task 5: Enhanced `/api/health` endpoint
- ✅ Task 6: Startup version validation

### Phase 3: Installer Enhancements ✅
- ✅ Task 7: Version pre-flight checks in installer
- ✅ Task 8: Enhanced daemon stop logic with retry
- ✅ Task 9: Post-install health checks

### Phase 4: Diagnostic Scripts ✅
- ✅ Task 10: `installer-health-check.ps1`
- ✅ Task 11: `diagnose-installation.ps1`
- ✅ Task 12: `force-clean-reinstall.ps1`

### Phase 5: Integration & Testing ✅
- ✅ Task 13: Version consistency integration tests
- ✅ Task 14: Installer validation test suite
- ✅ Task 15: Diagnostic scripts test suite
- ✅ Task 16: CI/CD version validation
- ✅ Task 17: Complete documentation (this file)

## Quick Start

### For Users: Installing KeyRx

1. **Download the MSI** from releases
2. **Verify before installing:**
   ```powershell
   .\scripts\installer-health-check.ps1 -PreInstall
   ```
3. **Install** by double-clicking the MSI or:
   ```powershell
   msiexec /i KeyRx-0.1.5-x64.msi /qn
   ```
4. **Verify installation:**
   ```powershell
   .\scripts\installer-health-check.ps1 -PostInstall
   ```

### For Developers: Updating Version

1. **Update the version** (single command):
   ```bash
   ./scripts/sync-version.sh 0.2.0
   ```
2. **Build and test:**
   ```bash
   cargo build --release
   make test
   ```
3. **Verify consistency:**
   ```bash
   ./scripts/sync-version.sh --check
   ```

### For Troubleshooting: Diagnosing Issues

1. **Run diagnostics:**
   ```powershell
   .\scripts\diagnose-installation.ps1
   ```
2. **Check specific issues:**
   ```powershell
   .\scripts\diagnose-installation.ps1 -Json | jq '.issues'
   ```
3. **If all else fails, clean reinstall:**
   ```powershell
   .\scripts\force-clean-reinstall.ps1
   ```

## Complete Workflow for Version Updates

This section documents the step-by-step process for updating versions across the project.

### 1. Pre-Update Checks

Before changing any version numbers:

```bash
# Check current version consistency
./scripts/sync-version.sh --check

# Ensure working directory is clean
git status

# Run full test suite
make verify
```

### 2. Update Version (Single Command)

```bash
# Update all version sources from command line
./scripts/sync-version.sh 0.2.0
```

This automatically updates:
- `Cargo.toml` (workspace.package.version)
- `keyrx_ui/package.json` (version field)
- `keyrx_daemon/keyrx_installer.wxs` (Version attribute)
- `scripts/build_windows_installer.ps1` ($Version variable)

### 3. Rebuild Everything

```bash
# Full clean rebuild
cargo clean
cd keyrx_ui && npm run build:wasm && npm run build && cd ..
cargo build --release -p keyrx_daemon

# Build installer (Windows)
.\scripts\build_windows_installer.ps1
```

### 4. Verify Version Consistency

```bash
# Runtime verification (all sources)
.\scripts\version-check.ps1

# Expected output:
# All versions should match: 0.2.0
```

### 5. Test Installation

```bash
# Pre-install validation
.\scripts\installer-health-check.ps1 -PreInstall

# Install
msiexec /i target\installer\KeyRx-0.2.0-x64.msi /qn

# Post-install validation
.\scripts\installer-health-check.ps1 -PostInstall
```

### 6. Commit and Tag

```bash
git add Cargo.toml keyrx_ui/package.json keyrx_daemon/keyrx_installer.wxs scripts/build_windows_installer.ps1
git commit -m "chore: bump version to 0.2.0"
git tag v0.2.0
git push origin main --tags
```

## Troubleshooting Guide

### Decision Tree

```
Issue Detected
│
├─ Version Mismatch?
│  ├─ YES → Run: ./scripts/sync-version.sh --fix
│  └─ NO → Continue
│
├─ Installation Failed?
│  ├─ Admin Rights? → Run PowerShell as Administrator
│  ├─ Daemon Running? → Run: .\scripts\diagnose-installation.ps1 -AutoFix
│  └─ Stale Binary? → Run: .\scripts\force-clean-reinstall.ps1
│
├─ Daemon Won't Start?
│  ├─ Port Conflict? → Check: netstat -ano | findstr "9867"
│  ├─ File Locked? → Check: .\scripts\diagnose-installation.ps1
│  └─ Config Error? → Check: %USERPROFILE%\.keyrx\daemon.log
│
└─ Wrong Version Displayed?
   ├─ Web UI? → Rebuild UI: cd keyrx_ui && npm run build
   ├─ Tray Icon? → Rebuild daemon: cargo build --release -p keyrx_daemon
   └─ API? → Check: curl http://localhost:9867/api/version
```

### Common Issues and Fixes

#### Issue 1: "Version Mismatch Between Sources"

**Symptoms:**
- Build fails with version mismatch error
- `sync-version.sh --check` reports inconsistencies

**Root Cause:** Manual edit of one version file without updating others

**Fix:**
```bash
# Automatic fix (uses Cargo.toml as source of truth)
./scripts/sync-version.sh --fix

# Or specify the correct version
./scripts/sync-version.sh 0.1.5
```

**Prevention:** Always use `sync-version.sh` to update versions

#### Issue 2: "Installation Failed - Access Denied"

**Symptoms:**
- MSI installer fails with "access denied" error
- Cannot overwrite installed binary

**Root Cause:** Daemon running with admin rights, file locked

**Fix:**
```powershell
# Option 1: Auto-diagnose and fix
.\scripts\diagnose-installation.ps1 -AutoFix

# Option 2: Manual fix
Stop-Process -Name keyrx_daemon -Force
msiexec /i KeyRx-0.1.5-x64.msi /qn
```

**Prevention:** Installer now automatically stops daemon before upgrade (Task 8)

#### Issue 3: "Stale Binary Installed"

**Symptoms:**
- Web UI shows old version number
- API returns old version
- System tray shows old build time

**Root Cause:** Installer packaged old binary, not fresh build

**Fix:**
```powershell
# Complete clean reinstall
.\scripts\force-clean-reinstall.ps1
```

**Prevention:** Installer now validates binary timestamp (Task 7)

#### Issue 4: "Daemon Won't Start After Install"

**Symptoms:**
- Installation succeeds but daemon doesn't run
- No system tray icon
- API not responding

**Root Cause:** Multiple possible causes (port conflict, config error, permissions)

**Diagnosis:**
```powershell
# Comprehensive diagnostics
.\scripts\diagnose-installation.ps1

# Check logs
Get-Content "$env:USERPROFILE\.keyrx\daemon.log" -Tail 50

# Check port
netstat -ano | findstr "9867"
```

**Fix:** Based on diagnostic output

#### Issue 5: "Build Time Shows Old Date"

**Symptoms:**
- System tray "About" shows old build time
- Web UI footer shows old date
- `build.rs` not regenerating constants

**Root Cause:** Incremental compilation cache not invalidated

**Fix:**
```bash
# Force full rebuild
cargo clean
cargo build --release -p keyrx_daemon

# Or use the SSOT rebuild script
.\REBUILD_SSOT.bat
```

**Prevention:** Use `force-clean-reinstall.ps1` for releases

## Diagnostic Script Usage

### installer-health-check.ps1

Comprehensive MSI and installation verification.

**Usage:**
```powershell
# Full health check (pre + post install)
.\scripts\installer-health-check.ps1

# Pre-install validation only
.\scripts\installer-health-check.ps1 -PreInstall

# Post-install validation only
.\scripts\installer-health-check.ps1 -PostInstall

# JSON output for CI
.\scripts\installer-health-check.ps1 -Json
```

**Checks Performed:**

Pre-Install:
1. Admin rights available
2. MSI file exists and valid
3. MSI version matches source binary
4. Binary timestamp is recent (< 24 hours)
5. All required files present in MSI
6. No conflicting installation exists

Post-Install:
1. Binary installed at correct path
2. Binary version matches MSI
3. Daemon starts successfully
4. API responds to health checks
5. Profiles endpoint functional
6. Configuration loaded correctly

**Example Output:**
```
========================================
 KeyRx Installer Health Check
========================================

[Pre-Install Validation]

[1/6] Checking admin rights...
  ✓ Admin Rights
    Running with administrator privileges

[2/6] Checking MSI file...
  ✓ MSI File
    Found MSI (12.45 MB)

[3/6] Checking MSI version...
  ✓ Version Match
    MSI version matches binary: 0.1.5

Summary: 6 passed, 0 failed, 0 warnings
```

### diagnose-installation.ps1

Comprehensive troubleshooting for installation and runtime issues.

**Usage:**
```powershell
# Full diagnosis
.\scripts\diagnose-installation.ps1

# JSON output
.\scripts\diagnose-installation.ps1 -Json

# Auto-fix common issues
.\scripts\diagnose-installation.ps1 -AutoFix
```

**Information Gathered:**

1. **System Information:**
   - OS version
   - Current user and admin status
   - PowerShell version

2. **Version Analysis:**
   - Cargo.toml version
   - package.json version
   - WiX installer version
   - Source binary version
   - Installed binary version
   - Running daemon version (via API)

3. **File Analysis:**
   - Binary existence and timestamps
   - File size validation
   - File locks (processes holding binary)

4. **Process Analysis:**
   - Daemon process status
   - Process uptime
   - Admin privileges of process

5. **Network Analysis:**
   - Port 9867 availability
   - API health check
   - WebSocket connectivity

6. **Event Log Analysis:**
   - Recent daemon errors
   - Installation events
   - Service events

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

[Issues Found]
  ⚠ Port 9867 in use by another process (PID: 12345)
     Fix: Stop conflicting process or change daemon port

[Suggestions]
  1. Check daemon.log for errors
  2. Verify firewall allows port 9867
  3. Consider running: .\scripts\force-clean-reinstall.ps1
```

### version-check.ps1

Quick version consistency verification.

**Usage:**
```powershell
# Check all version sources
.\scripts\version-check.ps1

# Compact output
.\scripts\version-check.ps1 -Compact
```

**Example Output:**
```
Version Consistency Check
========================

Source              Version    Status
------              -------    ------
Cargo.toml          0.1.5      ✓
package.json        0.1.5      ✓
keyrx_installer.wxs 0.1.5.0    ✓
Source Binary       0.1.5      ✓
Installed Binary    0.1.5      ✓
Running Daemon      0.1.5      ✓

Result: All versions match (0.1.5)
```

### force-clean-reinstall.ps1

Complete clean reinstall automation.

**Usage:**
```powershell
# Interactive mode (prompts for confirmation)
.\scripts\force-clean-reinstall.ps1

# Auto-confirm mode (for scripts)
.\scripts\force-clean-reinstall.ps1 -Force

# Skip UI rebuild
.\scripts\force-clean-reinstall.ps1 -SkipUiBuild
```

**Steps Performed:**

1. **Stop Daemon:** Graceful shutdown, then force kill if needed
2. **Uninstall MSI:** Clean removal of existing installation
3. **Clean State:** Remove `~/.keyrx/*` state files
4. **Clean Build:** Remove `target/release/*` artifacts
5. **Rebuild UI:** `npm run build` with WASM compilation
6. **Rebuild Daemon:** `cargo build --release -p keyrx_daemon`
7. **Build Installer:** WiX compilation to fresh MSI
8. **Install MSI:** Silent installation with logs
9. **Verify:** Post-install health checks

**Example Output:**
```
========================================
 KeyRx Force Clean Reinstall
========================================

WARNING: This will completely remove and reinstall KeyRx.
All configuration and state will be lost.

Continue? [Y/N]: Y

[1/9] Stopping daemon...
  ✓ Daemon stopped (PID: 12345)

[2/9] Uninstalling existing MSI...
  ✓ MSI uninstalled successfully

[3/9] Cleaning state files...
  ✓ Removed C:\Users\developer\.keyrx\*

[4/9] Cleaning build artifacts...
  ✓ Removed target\release\*

[5/9] Rebuilding UI...
  ✓ UI built successfully

[6/9] Rebuilding daemon...
  ✓ Daemon built successfully

[7/9] Building installer...
  ✓ Installer built: KeyRx-0.1.5-x64.msi

[8/9] Installing MSI...
  ✓ Installation successful

[9/9] Verifying installation...
  ✓ All post-install checks passed

========================================
 Reinstall Complete
========================================
Version: 0.1.5
Build Time: 2026-01-29 15:30:45
Status: All checks passed
```

## Installer Validation Process

### Pre-Flight Checks (Before `InstallFiles`)

The WiX installer performs these validations before copying files:

1. **Binary Version Check:**
   - Extract version from `keyrx_daemon.exe --version`
   - Compare with MSI `ProductVersion` property
   - Fail installation if mismatch with clear error:
     ```
     ERROR: Version mismatch detected
     Binary version: 0.1.4
     MSI version: 0.1.5

     This installer contains the wrong binary.
     Please rebuild the installer with the correct version.
     ```

2. **Binary Timestamp Check:**
   - Get binary file timestamp
   - Calculate age in hours
   - Warn if binary older than 24 hours:
     ```
     WARNING: Binary is stale
     Binary timestamp: 2026-01-28 10:00:00
     Age: 30 hours

     This may not be the latest build.
     Consider rebuilding before installation.
     ```

3. **File Integrity Check:**
   - Verify all required files present
   - Validate file sizes
   - Check digital signature (if signed)

### Daemon Stop Logic (Before `RemoveExistingProducts`)

The installer reliably stops the daemon during upgrades:

1. **Attempt 1:** Graceful shutdown via API
   ```
   POST http://localhost:9867/api/shutdown
   Wait 2 seconds
   ```

2. **Attempt 2:** Graceful process termination
   ```
   Stop-Process -Name keyrx_daemon
   Wait 2 seconds
   ```

3. **Attempt 3:** Force termination
   ```
   Stop-Process -Name keyrx_daemon -Force
   Wait 1 second
   ```

4. **Timeout:** After 10 seconds total, fail installation:
   ```
   ERROR: Cannot stop existing daemon

   Manual steps required:
   1. Open Task Manager
   2. End process: keyrx_daemon.exe
   3. Retry installation
   ```

### Post-Install Verification (After `InstallFinalize`)

The installer verifies successful installation:

1. **Binary Exists:**
   - Check `C:\Program Files\KeyRx\bin\keyrx_daemon.exe`
   - Verify file size > 0
   - Check version matches MSI

2. **Daemon Starts:**
   - Start daemon process
   - Wait up to 10 seconds for startup
   - Check process is running

3. **API Responds:**
   - Check `GET http://localhost:9867/api/health`
   - Verify `{ "status": "ok" }`
   - Timeout after 5 seconds

4. **Installation Report:**
   ```
   KeyRx Installation Successful

   Version: 0.1.5
   Location: C:\Program Files\KeyRx
   Status: Running
   API: http://localhost:9867

   Open web interface: http://localhost:9867
   ```

## References

### Related Documentation

- **[SSOT_VERSION.md](../../../SSOT_VERSION.md)** - Original SSOT principles and v0.1.5 changes
- **[INSTALLER_FIX.md](../../../INSTALLER_FIX.md)** - v0.1.4 installer auto-stop implementation
- **[CRITICAL_DIAGNOSIS.md](../../../CRITICAL_DIAGNOSIS.md)** - Key blocking thread_local bug analysis
- **[docs/version-management.md](../../../docs/version-management.md)** - Detailed version update procedures
- **[docs/troubleshooting-installer.md](../../../docs/troubleshooting-installer.md)** - Comprehensive troubleshooting guide

### Scripts

- **[scripts/sync-version.sh](../../../scripts/sync-version.sh)** - Version synchronization automation
- **[scripts/version-check.ps1](../../../scripts/version-check.ps1)** - Runtime version verification
- **[scripts/installer-health-check.ps1](../../../scripts/installer-health-check.ps1)** - MSI validation
- **[scripts/diagnose-installation.ps1](../../../scripts/diagnose-installation.ps1)** - Comprehensive diagnostics
- **[scripts/force-clean-reinstall.ps1](../../../scripts/force-clean-reinstall.ps1)** - Clean reinstall automation

### Code

- **[keyrx_daemon/build.rs](../../../keyrx_daemon/build.rs)** - Build-time version validation
- **[keyrx_daemon/src/version.rs](../../../keyrx_daemon/src/version.rs)** - Version constants
- **[keyrx_daemon/src/web/api/diagnostics.rs](../../../keyrx_daemon/src/web/api/diagnostics.rs)** - Diagnostics API
- **[keyrx_daemon/keyrx_installer.wxs](../../../keyrx_daemon/keyrx_installer.wxs)** - WiX installer definition

### Tests

- **[keyrx_daemon/tests/version_consistency_test.rs](../../../keyrx_daemon/tests/version_consistency_test.rs)** - Version management tests
- **[tests/installer_validation_test.rs](../../../tests/installer_validation_test.rs)** - Installer validation tests
- **[tests/diagnostic_scripts_test.ps1](../../../tests/diagnostic_scripts_test.ps1)** - Diagnostic script tests

## Success Criteria

All success criteria have been met:

- ✅ Version mismatch at build time → compilation fails with clear error (Task 2)
- ✅ Version mismatch at runtime → logged warnings with fix instructions (Task 6)
- ✅ Installer with stale binary → installation fails before copying files (Task 7)
- ✅ Any installation issue → diagnostic script identifies root cause (Task 11)
- ✅ Version update → single command syncs all files automatically (Task 1)
- ✅ CI/CD → prevents merging code with version inconsistencies (Task 16)

## Lessons Learned

### What Worked Well

1. **SSOT Enforcement:** Single source of truth (Cargo.toml) with automated sync prevented version mismatches
2. **Fail-Fast Validation:** Build-time checks caught issues before deployment
3. **Comprehensive Diagnostics:** Decision tree and auto-fix suggestions made troubleshooting obvious
4. **Installer Pre-Flight:** Version and timestamp validation prevented stale binary installation
5. **Retry Logic:** Daemon stop with retry/timeout handled edge cases gracefully

### What Could Be Improved

1. **Cross-Platform Scripts:** PowerShell scripts are Windows-only; need Bash equivalents for Linux
2. **CI Integration:** Version validation in CI works but could be more prominent (fail earlier)
3. **User Communication:** Error messages clear but could link to troubleshooting docs
4. **Automated Testing:** E2E installer tests exist but require admin rights (CI limitation)
5. **Documentation:** Comprehensive but could benefit from video tutorials

### Future Enhancements

1. **Auto-Update Mechanism:** Check for new versions and prompt user to upgrade
2. **Rollback Support:** Allow reverting to previous version if upgrade fails
3. **Version Compatibility Matrix:** Document which daemon versions work with which UI versions
4. **Telemetry:** Anonymous error reporting to identify common issues
5. **Diagnostic Dashboard:** Web UI page showing all diagnostic information

## Conclusion

This specification successfully addressed all critical pain points from past releases. The combination of automated version management, fail-fast validation, and comprehensive diagnostics has made version mismatches **impossible** and debugging **obvious**.

**Key Achievements:**
- Zero manual version syncing required
- Build failures happen at compile time, not at runtime
- Installation issues are diagnosed automatically with fix suggestions
- All checks run in CI to prevent problematic merges
- Complete audit trail from version update to deployed installation

**Next Release Readiness:**
The infrastructure is now in place to confidently release v0.2.0 and beyond without version management or installer issues.
