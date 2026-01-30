# Test Plan: Installer Enhancements (Tasks 7-9)

**Spec:** `.spec-workflow/specs/installer-debuggability-enhancement/tasks.md`
**Implementation:** `docs/installer-enhancements-tasks-7-9.md`
**Date:** 2026-01-30

## Prerequisites

1. **Build environment:**
   - Rust toolchain (cargo 1.70+)
   - WiX Toolset 3.11+ installed
   - PowerShell 5.1+
   - Administrator privileges

2. **Clean state:**
   ```powershell
   # Uninstall existing KeyRx
   Get-WmiObject -Class Win32_Product | Where-Object {$_.Name -like "*KeyRx*"} | ForEach-Object {$_.Uninstall()}

   # Stop any running daemon
   Get-Process keyrx_daemon -ErrorAction SilentlyContinue | Stop-Process -Force

   # Clean build artifacts
   cargo clean
   ```

## Test Suite

### Task 7: ValidateBinaryVersion

#### Test 7.1: Version Mismatch Detection
**Objective:** Ensure installer rejects binaries with wrong version

**Steps:**
1. Build with version 0.1.5: `cargo build --release -p keyrx_daemon`
2. Manually edit `target\release\keyrx_daemon.exe` metadata or build old version
3. Build MSI: `.\scripts\build_windows_installer.ps1`
4. Run installer

**Expected Result:**
- ❌ Installation fails before copying files
- Error message: "Version mismatch" or similar
- MSI log shows PowerShell validation failure

**Pass Criteria:**
- Installation does not proceed
- Clear error message shown to user
- No files copied to Program Files

---

#### Test 7.2: Stale Binary Detection
**Objective:** Ensure installer rejects binaries older than 24 hours

**Steps:**
1. Build binary: `cargo build --release -p keyrx_daemon`
2. Touch file to simulate old timestamp:
   ```powershell
   (Get-Item target\release\keyrx_daemon.exe).LastWriteTime = (Get-Date).AddHours(-25)
   ```
3. Build MSI: `.\scripts\build_windows_installer.ps1`
4. Run installer

**Expected Result:**
- ❌ Installation fails with "Binary too old" error
- MSI log shows timestamp validation failure
- User advised to rebuild

**Pass Criteria:**
- Installation blocked
- Timestamp check logged
- Clear actionable error message

---

#### Test 7.3: Fresh Binary Success
**Objective:** Verify successful installation with fresh, correct binary

**Steps:**
1. Clean build: `cargo clean && cargo build --release -p keyrx_daemon`
2. Immediately build MSI: `.\scripts\build_windows_installer.ps1`
3. Run installer

**Expected Result:**
- ✅ Validation passes
- Installation proceeds normally
- MSI log shows "Version validation passed"

**Pass Criteria:**
- No validation errors
- Installation completes
- Binary copied to Program Files

---

### Task 8: StopDaemonBeforeUpgrade

#### Test 8.1: Daemon Running During Upgrade
**Objective:** Verify daemon stops during upgrade

**Steps:**
1. Install KeyRx v0.1.4 (or earlier version)
2. Start daemon: `keyrx_daemon run`
3. Verify running: `Get-Process keyrx_daemon`
4. Run v0.1.5 installer (upgrade)

**Expected Result:**
- ✅ Installer stops daemon automatically
- Upgrade proceeds without errors
- MSI log shows "SUCCESS: Daemon stopped"

**Pass Criteria:**
- Daemon stops within 10 seconds
- No file locking errors
- Upgrade completes successfully

---

#### Test 8.2: Daemon Not Running
**Objective:** Verify graceful handling when daemon already stopped

**Steps:**
1. Install KeyRx
2. Ensure daemon NOT running: `Stop-Process -Name keyrx_daemon -Force -ErrorAction SilentlyContinue`
3. Run installer upgrade

**Expected Result:**
- ✅ Installer handles gracefully
- MSI log shows "Daemon is not running"
- Upgrade proceeds normally

**Pass Criteria:**
- No errors logged
- Installation continues
- No timeouts or hangs

---

#### Test 8.3: Daemon Refuses to Stop
**Objective:** Verify timeout and force kill handling

**Setup:**
```powershell
# Simulate stubborn process (for testing, use sleep loop or debugger)
Start-Process -FilePath keyrx_daemon.exe -ArgumentList "run" -PassThru | ForEach-Object {
    # Attach debugger or hold file lock
}
```

**Steps:**
1. Start daemon with debugger attached or file lock
2. Run installer upgrade
3. Monitor process list and MSI log

**Expected Result:**
- ⚠️ Installer retries 3 times
- Final attempt uses force kill
- Timeout after 10 seconds if still failing
- Upgrade continues (Return="ignore")

**Pass Criteria:**
- Maximum 10-second wait
- Force kill attempted
- Installation doesn't hang
- MSI log shows retry attempts

---

#### Test 8.4: Uninstall Scenario
**Objective:** Verify daemon stops during uninstall

**Steps:**
1. Install KeyRx
2. Start daemon
3. Run uninstaller (via Add/Remove Programs or MSI)

**Expected Result:**
- ✅ Daemon stops before file removal
- Uninstallation completes
- No file locking errors

**Pass Criteria:**
- Clean uninstall
- All files removed
- No orphaned processes

---

### Task 9: VerifyInstallation

#### Test 9.1: Successful Installation
**Objective:** Verify post-install validation shows success

**Steps:**
1. Ensure port 9867 is available
2. Run fresh installation
3. Observe MessageBox

**Expected Result:**
- ✅ MessageBox displays:
  ```
  Installation verified successfully!

  ✓ Binary installed successfully (X.XX MB)
  ✓ Version verified: 0.1.5
  ✓ Daemon started successfully
  ✓ API health check passed

  KeyRx is ready to use.
  ```

**Pass Criteria:**
- All checkmarks green (✓)
- Binary exists at install path
- Daemon running
- API responding on port 9867

---

#### Test 9.2: Daemon Start Failure
**Objective:** Verify graceful handling when daemon fails to start

**Setup:**
```powershell
# Block port 9867
$listener = [System.Net.Sockets.TcpListener]::new([System.Net.IPAddress]::Any, 9867)
$listener.Start()
```

**Steps:**
1. Run installation with port blocked
2. Observe MessageBox

**Expected Result:**
- ⚠️ MessageBox shows warnings:
  ```
  Installation completed with warnings:

  ✓ Binary installed successfully (X.XX MB)
  ✓ Version verified: 0.1.5
  ⚠ Daemon start timeout (check logs)
  ⚠ API check skipped (daemon not running)

  You may need to start the daemon manually.
  ```

**Pass Criteria:**
- Installation completes (doesn't rollback)
- Warning symbols shown (⚠)
- User advised to start manually
- Binary installed correctly

---

#### Test 9.3: API Health Check Failure
**Objective:** Verify API validation with daemon running but API not responding

**Setup:**
1. Modify daemon to not start API server (test build)
2. Or block HTTP traffic on port 9867

**Steps:**
1. Run installation
2. Daemon starts but API doesn't respond

**Expected Result:**
- ⚠️ MessageBox shows:
  ```
  ✓ Binary installed successfully
  ✓ Version verified: 0.1.5
  ✓ Daemon started successfully
  ⚠ API not responding (may need manual start)
  ```

**Pass Criteria:**
- Installation doesn't fail
- Warning shown
- User can investigate API issue

---

#### Test 9.4: Version Verification at Install Path
**Objective:** Verify installed binary version check

**Setup:**
1. Manually replace installed binary with wrong version (during test)

**Steps:**
1. Run installation normally
2. Script validates installed binary

**Expected Result:**
- Either ✓ or ❌ depending on version match
- Clear feedback in MessageBox

**Pass Criteria:**
- Version check executes
- Result matches actual binary version

---

## Integration Tests

### Integration 1: Complete Fresh Install
**Flow:** Task 7 → Install → Task 9

**Steps:**
1. Clean system (no KeyRx)
2. Run installer

**Expected Sequence:**
```
1. ValidateBinaryVersion → PASS
2. InstallFiles
3. VerifyInstallation → PASS
4. MessageBox: "Installation verified successfully!"
```

**Pass Criteria:**
- All validations pass
- Daemon running
- API accessible
- User sees success message

---

### Integration 2: Upgrade with Running Daemon
**Flow:** Task 8 → Task 7 → Install → Task 9

**Steps:**
1. Install v0.1.4
2. Start daemon
3. Upgrade to v0.1.5

**Expected Sequence:**
```
1. StopDaemonBeforeUpgrade → SUCCESS
2. ValidateBinaryVersion → PASS
3. RemoveExistingProducts
4. InstallFiles
5. VerifyInstallation → PASS (daemon restarted)
```

**Pass Criteria:**
- Old daemon stopped
- New version installed
- New daemon started
- API responding

---

### Integration 3: Uninstall with Running Daemon
**Flow:** Task 8 → Uninstall

**Steps:**
1. Install KeyRx
2. Start daemon
3. Uninstall

**Expected Sequence:**
```
1. StopDaemonBeforeRemove → SUCCESS
2. RemoveFiles
3. Clean uninstall
```

**Pass Criteria:**
- Daemon stopped
- All files removed
- No orphaned processes

---

## MSI Log Analysis

For each test, examine the MSI log:

```powershell
# Install with logging
msiexec /i KeyRx-0.1.5-x64.msi /l*v install.log

# Search for CustomAction execution
Select-String -Path install.log -Pattern "CustomAction|ValidateBinaryVersion|StopDaemon|VerifyInstallation"
```

**Look for:**
- Task 7: "Version validation passed" or failure reason
- Task 8: "SUCCESS: Daemon stopped" or retry attempts
- Task 9: Verification results and MessageBox display

---

## Regression Tests

### Existing Functionality
Ensure no regressions in existing installer features:

1. ✅ Admin rights check still works
2. ✅ PATH environment variable updated
3. ✅ Start Menu shortcut created
4. ✅ %APPDATA%\keyrx folder created
5. ✅ LICENSE, README, example config installed
6. ✅ Launch on exit checkbox works

---

## Performance Tests

### Installation Speed
- **Baseline:** Record time for v0.1.4 installation
- **New:** Time v0.1.5 with new CustomActions
- **Acceptable:** < 2 seconds overhead

### Daemon Stop Timeout
- **Normal case:** Should stop in < 3 seconds
- **Retry case:** Maximum 10 seconds
- **Acceptable:** Never hangs indefinitely

---

## Error Handling Tests

### PowerShell Execution Policy
**Test:** System with restricted execution policy

**Steps:**
1. Set execution policy: `Set-ExecutionPolicy Restricted`
2. Run installer

**Expected:** CustomActions still work (uses `-ExecutionPolicy Bypass`)

---

### Missing PowerShell
**Test:** System without PowerShell (unlikely but possible)

**Steps:**
1. Rename `powershell.exe` temporarily
2. Run installer

**Expected:** Graceful failure with clear error message

---

## Acceptance Criteria

All tests must pass before marking tasks complete:

- [ ] Task 7 tests (7.1, 7.2, 7.3) all pass
- [ ] Task 8 tests (8.1, 8.2, 8.3, 8.4) all pass
- [ ] Task 9 tests (9.1, 9.2, 9.3, 9.4) all pass
- [ ] Integration tests (1, 2, 3) all pass
- [ ] No regressions in existing functionality
- [ ] MSI logs show correct CustomAction execution
- [ ] Performance within acceptable limits
- [ ] Error handling works as designed

---

## Bug Tracking

If any test fails, log with:
- Test ID (e.g., "Test 7.1")
- Expected vs Actual result
- MSI log excerpt
- Screenshots of error messages
- Reproducibility (always/sometimes/rare)

---

## Sign-off

- [ ] Developer: All unit tests pass, code reviewed
- [ ] QA: All integration tests pass, edge cases covered
- [ ] DevOps: MSI builds successfully, logs are clean
- [ ] Product Owner: User experience meets requirements

**Tested by:** _____________
**Date:** _____________
**Version:** 0.1.5
**Result:** PASS / FAIL / PARTIAL
