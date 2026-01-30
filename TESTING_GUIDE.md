# Testing Guide - Prevent "daemon runs but doesn't remap" Bug

## The Bug You're Experiencing

**Symptoms:**
- Config page loads ✓
- Can activate profile ✓
- Remap NOT working ✗
- Metrics page shows no events ✗

**Root Cause:** Old binary (v0.1.2 from 14:23:22) installed instead of v0.1.4 (15:41:11)

## Tests to Catch This Bug

I've created comprehensive tests to prevent this from happening again:

### 1. Version Verification Test (Integration)

**File:** `keyrx_daemon/tests/version_verification_test.rs`

**What it tests:**
- Binary version matches Cargo.toml version
- Installed binary timestamp is recent (within 24 hours)
- Daemon has admin manifest (Windows)

**Run:**
```bash
cargo test --test version_verification_test -- --ignored
```

**Output if bug exists:**
```
Binary version mismatch! Expected: 0.1.4, Got: 0.1.2
Binary is too old! Last modified: 26 hours ago. Expected: within 24 hours.
```

---

### 2. Keyboard Interception E2E Test

**File:** `keyrx_daemon/tests/keyboard_interception_e2e_test.rs`

**What it tests:**
- Daemon starts and API responds
- Profile can be activated
- **Keyboard events are actually captured** (catches your bug!)
- Metrics show events

**Run:**
```bash
cargo test --test keyboard_interception_e2e_test -- --ignored --test-threads=1
```

**Output if bug exists:**
```
CRITICAL: No events detected! Keyboard interception is NOT working.
This is the bug the user is experiencing - daemon runs but doesn't intercept keys.
```

---

### 3. Daemon Health Test (Unit/Integration)

**File:** `keyrx_daemon/tests/daemon_health_test.rs`

**What it tests:**
- Daemon version command works
- Health endpoint responds within timeout
- Config directory exists
- Platform-specific initialization works

**Run:**
```bash
cargo test --test daemon_health_test
cargo test --test daemon_health_test -- --ignored  # For tests requiring daemon restart
```

---

### 4. Installation Verification Script (PowerShell)

**File:** `scripts/test_installation.ps1`

**What it tests:**
- Binary exists at correct path
- Binary timestamp matches expected
- Daemon is running
- API health check passes
- Profiles endpoint works
- **Keyboard interception captures events** (catches your bug!)
- Port 9867 is open

**Run:**
```powershell
.\scripts\test_installation.ps1

# With custom expected timestamp
.\scripts\test_installation.ps1 -ExpectedTimestamp "2026/01/29 15:41:11"
```

**Output:**
```
========================================
 Testing KeyRx Installation
========================================

[1/7] Checking binary exists...
  ✓ Binary found
[2/7] Checking binary timestamp...
  ✗ Timestamp mismatch!
    Expected: 2026/01/29 15:41:11
    Got:      2026/01/29 14:23:22
...
[6/7] Checking keyboard interception...
  ✗ No events captured - keyboard interception NOT working!
    This is the bug user reported: daemon runs but doesn't intercept keys

========================================
 Installation Test FAILED
========================================

Action Required:
1. Right-click UPDATE_BINARY.ps1 → Run as Administrator
2. Re-run this test script
```

---

## Recommended Testing Workflow

### Before Deployment

```powershell
# 1. Build
cargo build --release -p keyrx_daemon

# 2. Build installer
.\scripts\build_windows_installer.ps1

# 3. Test version
cargo test --test version_verification_test -- --ignored

# 4. Install
.\QUICK_REINSTALL.ps1

# 5. Verify installation
.\scripts\test_installation.ps1

# 6. E2E test
cargo test --test keyboard_interception_e2e_test -- --ignored --test-threads=1
```

### After Installation (Quick Check)

```powershell
# Quick installation verification (7 tests)
.\scripts\test_installation.ps1
```

If all tests pass → Keyboard remapping will work ✓
If Test 6 fails → Your bug is present ✗

---

## CI/CD Integration

Add to `.github/workflows/ci.yml`:

```yaml
- name: Version Verification
  run: cargo test --test version_verification_test -- --ignored

- name: Daemon Health Test
  run: cargo test --test daemon_health_test

- name: Installation Test
  run: |
    .\scripts\build_windows_installer.ps1
    .\scripts\test_installation.ps1
```

---

## What Each Test Catches

| Test | Catches |
|------|---------|
| **version_verification_test** | Wrong binary version installed, old binary deployed |
| **keyboard_interception_e2e_test** | **Keyboard events not captured (your bug!)** |
| **daemon_health_test** | Daemon doesn't start, API doesn't respond, config issues |
| **test_installation.ps1** | **Complete deployment verification (fastest)** |

---

## Fix Your Current Issue

**Right now, you need to:**

1. **Right-click `UPDATE_BINARY.ps1`** → **"Run as Administrator"**
2. Wait for completion (stops daemon → copies v0.1.4 binary → starts daemon)
3. Run verification:
   ```powershell
   .\scripts\test_installation.ps1
   ```
4. All 7 tests should pass
5. Open http://localhost:9867 and test remapping

---

## Why This Happened

The MSI installer couldn't stop the daemon (access denied) so it left the old binary in place. The CustomActions we added should fix this, but you need admin rights to:
1. Stop the daemon
2. Replace the binary in "C:\Program Files\KeyRx\bin\"

**Prevention:** Run `test_installation.ps1` after every deployment.

---

## Running All Tests

```bash
# All unit tests
cargo test --workspace

# All integration tests (including these new ones)
cargo test --workspace -- --ignored

# Quick installation verification
.\scripts\test_installation.ps1

# Full E2E suite
cargo test --test keyboard_interception_e2e_test -- --ignored --test-threads=1
```

---

## Expected Test Output (Success)

```
========================================
 Testing KeyRx Installation
========================================

[1/7] Checking binary exists...
  ✓ Binary found
[2/7] Checking binary timestamp...
  ✓ Timestamp matches: 2026/01/29 15:41:11
[3/7] Checking daemon process...
  ✓ Daemon is running (PID: 12345)
[4/7] Checking API health...
  ✓ API is responding (version: 0.1.2)
[5/7] Checking profiles endpoint...
  ✓ Profiles endpoint works (15 profiles found)
[6/7] Checking keyboard interception...
  ✓ Keyboard interception works (2 events captured)
[7/7] Checking port 9867...
  ✓ Port 9867 is open

========================================
 Installation Test PASSED
========================================

✓ All checks passed!
✓ Keyboard remapping should work correctly
```

---

## Summary

These tests **automatically catch the exact bug you're experiencing** - daemon runs but keyboard interception doesn't work. Run `test_installation.ps1` after every deployment to verify everything works before you discover issues manually.
