# Testing keyrx Over RDP

## Overview

keyrx uses low-level Windows APIs (RawInput + keyboard hooks) which have **different behavior over RDP** vs. physical input.

## Connection Methods Comparison

| Method | Input Path | RawInput Capture | Hooks Capture | Best For |
|--------|-----------|------------------|---------------|----------|
| **virt-manager console** | Physical → VM | ✅ Full | ✅ Full | **Real testing** |
| **RDP (mstsc/xfreerdp)** | RDP redirect | ⚠️ Partial | ⚠️ Partial | Debugging, UI work |
| **WinRM/SSH** | Command-line only | N/A | N/A | Automation |

## Quick Connect Guide

### Option 1: RDP (For GUI Access)

**From Linux host:**
```bash
# Install FreeRDP if needed
sudo apt install freerdp2-x11

# Connect to Vagrant VM
xfreerdp /v:localhost:13389 /u:vagrant /p:vagrant /size:1920x1080 +clipboard

# Or use Remmina (GUI client)
remmina -c rdp://vagrant:vagrant@localhost:13389
```

**Connection details:**
- Host: `localhost:13389` (forwarded from VM's 3389)
- Username: `vagrant`
- Password: `vagrant`

### Option 2: virt-manager Console (For Real Testing)

**From Linux host:**
```bash
# Install virt-manager if needed
sudo apt install virt-manager

# Launch virt-manager
virt-manager

# Double-click the "keyrx2-win-test" VM to open console
```

**Advantages:**
- Direct VM access (no RDP layer)
- Physical keyboard input path
- True low-level input capture

### Option 3: WinRM (For CLI/Automation)

```bash
cd vagrant/windows
vagrant winrm -c "cd C:\vagrant_project; cargo test --features windows"
```

## Testing keyrx Behavior

### Test 1: RawInput Capture Check

Create a test to verify if RawInput receives events:

```bash
# Build and run in VM via RDP
cd C:\vagrant_project
cargo build --features windows
.\target\debug\keyrx_daemon.exe --test-input-capture
```

**Expected behavior:**
- **Via virt-manager console**: Should capture all keypresses
- **Via RDP**: May miss some events or show different device handles

### Test 2: Hook Interception Check

Test if low-level hooks intercept input:

```bash
# Run hook test
.\target\debug\keyrx_daemon.exe --test-hooks
```

**Expected behavior:**
- **Via virt-manager console**: Hooks intercept before applications
- **Via RDP**: Hooks may not fire, or fire after input is processed

### Test 3: End-to-End Remapping Test

**Test scenario**: Remap CapsLock → Escape

```bash
# Start daemon with test config
.\target\debug\keyrx_daemon.exe --config test_config.toml

# Open Notepad and test
notepad.exe
# Press CapsLock - should produce Escape behavior
```

**Expected behavior:**
- **Via virt-manager console**: ✅ Remapping works perfectly
- **Via RDP**: ⚠️ May not work, or work inconsistently

## Known RDP Limitations

### 1. Special Key Combinations Captured by RDP Client

These keys are **intercepted by the RDP client** before reaching the VM:

- **Ctrl+Alt+Del** - Captured by local OS
- **Ctrl+Alt+End** - RDP equivalent of Ctrl+Alt+Del
- **Alt+Tab** - Often captured by local OS
- **Windows+L** - May lock local machine instead of VM
- **Ctrl+Esc** - Start menu handling varies

**Workaround**: Use virt-manager console for testing these keys.

### 2. RawInput Device Handles

RDP input shows as a **single virtual device**, not individual keyboards:

```
Via console: Multiple device handles (one per physical keyboard)
Via RDP:     Single RDP input device handle
```

**Impact**: If your app distinguishes between keyboards, RDP won't work correctly.

### 3. Input Timing Differences

RDP introduces network latency:
- **Console**: ~1ms input latency
- **RDP (localhost)**: ~5-20ms latency
- **RDP (network)**: ~50-200ms latency

**Impact**: Tap-vs-hold detection may behave differently.

## Recommended Testing Workflow

### For Feature Development

Use **RDP** for most work:
```bash
# Connect via RDP
xfreerdp /v:localhost:13389 /u:vagrant /p:vagrant /size:1920x1080

# Build and run in RDP session
cd C:\vagrant_project
cargo build --features windows
.\target\debug\keyrx_daemon.exe
```

**Advantages:**
- Copy-paste works
- Multiple monitors
- Comfortable GUI experience

### For Input Capture Testing

Use **virt-manager console**:
```bash
# Open console
virt-manager
# Double-click VM → Use console window

# Run tests in console
cargo test -p keyrx_daemon --features windows
```

**Advantages:**
- Real physical input path
- Accurate RawInput behavior
- True low-level hook behavior

### For Automated Testing

Use **WinRM**:
```bash
cd vagrant/windows
vagrant winrm -c 'cd C:\vagrant_project; cargo test --features windows'
```

**Advantages:**
- No GUI needed
- Scriptable
- CI/CD compatible

## RDP-Specific Issues & Solutions

### Issue: Keypresses Not Captured

**Symptom**: RawInput shows no events when typing over RDP

**Diagnosis:**
```powershell
# Check if RawInput is registered
Get-Process keyrx_daemon | Get-Member
```

**Solutions:**
1. **Use console mode**: Switch to virt-manager console for testing
2. **Use enhanced session mode**: Enable "Enhanced session mode" in RDP settings
3. **Disable input redirection**: Test with RDP input redirection disabled

### Issue: Hooks Not Firing

**Symptom**: Low-level hooks don't intercept RDP keyboard input

**Diagnosis:**
```rust
// Add logging in hook callback
log::debug!("Hook callback triggered: code={}", code);
```

**Solutions:**
1. **Test in console**: Hooks work reliably in virt-manager console
2. **Use polling instead**: Consider GetAsyncKeyState() as fallback
3. **Accept limitation**: Document that RDP testing is limited

### Issue: Device Handle Mismatch

**Symptom**: RawInput shows different device handle over RDP

**Solution:**
```rust
// Make device handle optional for testing
if device_handle == RDP_VIRTUAL_DEVICE {
    log::warn!("RDP input detected - limited functionality");
}
```

## Debugging Tips

### Enable RawInput Debug Logging

```rust
// In keyrx_daemon/src/platform/windows/rawinput.rs
log::debug!(
    "RawInput event: device={:?}, vkey={}, scancode={}, flags={}",
    device_handle,
    raw.VKey,
    raw.MakeCode,
    raw.Flags
);
```

### Compare Console vs RDP

Run the same test in both modes and compare:

```bash
# In console (virt-manager)
cargo test test_rawinput_capture > console.log 2>&1

# In RDP
cargo test test_rawinput_capture > rdp.log 2>&1

# Compare
diff console.log rdp.log
```

### Monitor Windows Event Logs

```powershell
# View keyboard device events
Get-WinEvent -LogName Microsoft-Windows-DeviceSetupManager/Admin -MaxEvents 20

# View input events
Get-WinEvent -LogName Microsoft-Windows-TerminalServices-LocalSessionManager/Operational
```

## Performance Comparison

| Scenario | Build Time | Test Execution | Input Latency |
|----------|-----------|----------------|---------------|
| **Console** | Normal | Normal | ~1ms |
| **RDP (localhost)** | Normal | Normal | ~10ms |
| **RDP (LAN)** | Normal | Normal | ~50ms |
| **WinRM** | Slow (no cache) | Normal | N/A |

## When to Use Each Method

### Use virt-manager Console When:
- ✅ Testing key remapping functionality
- ✅ Testing multi-keyboard support
- ✅ Testing special key combinations
- ✅ Verifying RawInput/hook behavior
- ✅ Performance testing with accurate latency

### Use RDP When:
- ✅ Building and compiling code
- ✅ Debugging non-input-related code
- ✅ UI/web interface development
- ✅ File management and editing
- ✅ Installing packages or tools

### Use WinRM When:
- ✅ Running automated tests
- ✅ CI/CD pipeline execution
- ✅ One-off command execution
- ✅ Scripted builds

## RDP Alternative: Enhanced Session Mode

**What is it?** Uses RemoteFX for better input handling.

**Enable on VM:**
```powershell
# In VM as Administrator
Enable-WindowsOptionalFeature -Online -FeatureName RemoteDesktop-SessionHost-EnhancedSession
```

**Enable on Linux client:**
```bash
# FreeRDP with enhanced session
xfreerdp /v:localhost:13389 /u:vagrant /p:vagrant \
    +drives +clipboard +usb \
    /drive:share,/home/user/share
```

**Benefits:**
- Better USB redirection
- Improved clipboard integration
- May improve input handling

**Limitations:**
- Still uses RDP input path
- Low-level hooks still limited

## Automated Testing Strategy

For CI/CD, use a hybrid approach:

```bash
#!/bin/bash
# ci_test_windows.sh

set -e

cd vagrant/windows

# Start VM
vagrant up

# Run non-input tests via WinRM (fast)
vagrant winrm -c 'cd C:\vagrant_project; cargo test --features windows --skip input_capture'

# For input tests, document limitation
echo "⚠️  Input capture tests require physical console access"
echo "    Run manually via virt-manager for full validation"

# Or use GUI automation tools
# vagrant winrm -c 'cd C:\vagrant_project; .\scripts\automated_input_test.ps1'
```

## Conclusion

**For keyrx development:**

1. **Use RDP** for general development, building, and debugging
2. **Use virt-manager console** for testing actual key remapping behavior
3. **Use WinRM** for automated tests that don't require input capture

**Remember:** RDP is great for productivity, but console is required for accurate input testing.

## Additional Resources

- **FreeRDP documentation**: https://github.com/FreeRDP/FreeRDP/wiki
- **virt-manager guide**: https://virt-manager.org/
- **Windows RawInput API**: https://learn.microsoft.com/en-us/windows/win32/inputdev/raw-input
- **RDP keyboard redirection**: https://learn.microsoft.com/en-us/windows-server/remote/remote-desktop-services/clients/keyboard-and-mouse
