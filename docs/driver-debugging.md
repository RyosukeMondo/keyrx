# Driver Debugging Guide

This guide helps developers and users troubleshoot driver issues in KeyRx. It covers both Windows and Linux platforms, common issues, debugging techniques, and recovery procedures.

## Table of Contents

- [Quick Troubleshooting](#quick-troubleshooting)
- [Environment Variables](#environment-variables)
- [Logging Configuration](#logging-configuration)
- [Common Issues](#common-issues)
  - [Linux-Specific Issues](#linux-specific-issues)
  - [Windows-Specific Issues](#windows-specific-issues)
- [Error Recovery](#error-recovery)
- [Emergency Exit](#emergency-exit)
- [Debugging Techniques](#debugging-techniques)
- [Reporting Issues](#reporting-issues)

## Quick Troubleshooting

### 1. Enable Debug Logging

```bash
# Enable detailed debug logs
RUST_LOG=debug keyrx run

# Enable trace-level logs for drivers only
RUST_LOG=keyrx_core::drivers=trace keyrx run

# JSON-formatted structured logs
RUST_LOG=debug keyrx run 2>&1 | jq
```

### 2. Test Emergency Exit

Press **Ctrl+Alt+Shift+Esc** to immediately disable all key remapping and regain keyboard control. This works even during errors and panics.

### 3. Check Permissions

**Linux:**
```bash
# Check device permissions
ls -la /dev/input/event*
ls -la /dev/uinput

# Check group membership
groups $USER

# Check if uinput module is loaded
lsmod | grep uinput
```

**Windows:**
Run KeyRx with Administrator privileges if you encounter permission errors.

## Environment Variables

### RUST_LOG

Controls logging verbosity using the `tracing` and `env_logger` framework.

**Syntax:**
```bash
RUST_LOG=<level>                    # Global level
RUST_LOG=<module>=<level>           # Module-specific level
RUST_LOG=<mod1>=<level1>,<mod2>=<level2>  # Multiple modules
```

**Log Levels** (most to least verbose):
- `trace` - Very detailed, includes all function calls and events
- `debug` - Detailed information for debugging
- `info` - General informational messages (default)
- `warn` - Warning messages for potentially problematic situations
- `error` - Error messages only

**Examples:**
```bash
# Enable debug logs for entire application
RUST_LOG=debug keyrx run

# Trace driver operations only
RUST_LOG=keyrx_core::drivers=trace keyrx run

# Debug drivers, info for everything else
RUST_LOG=info,keyrx_core::drivers=debug keyrx run

# Trace Linux driver, debug Windows driver
RUST_LOG=keyrx_core::drivers::linux=trace,keyrx_core::drivers::windows=debug keyrx run

# Enable all keyrx logs at trace level
RUST_LOG=keyrx=trace keyrx run
```

### Platform-Specific Environment Variables

**Linux:**
- `XDG_CONFIG_HOME` - Configuration directory location (default: `~/.config`)
- `HOME` - User home directory

**Windows:**
- No platform-specific environment variables for debugging

## Logging Configuration

### Log Output Format

KeyRx uses **structured logging** with the following fields:

```json
{
  "timestamp": "2025-12-04T03:17:00.123Z",
  "level": "INFO",
  "service": "keyrx",
  "event": "driver_started",
  "component": "linux_input",
  "device": "/dev/input/event4",
  "message": "Device grabbed successfully"
}
```

### Filtering Logs

**By component:**
```bash
# Linux driver events only
RUST_LOG=keyrx_core::drivers::linux=debug keyrx run 2>&1 | grep linux_input

# Windows driver events only
RUST_LOG=keyrx_core::drivers::windows=debug keyrx run 2>&1 | grep windows_input

# Engine events only
RUST_LOG=keyrx_core::engine=debug keyrx run 2>&1 | grep engine
```

**By event type:**
```bash
# Device connection/disconnection
RUST_LOG=debug keyrx run 2>&1 | grep -E 'device_(connected|disconnected)'

# Error events
RUST_LOG=debug keyrx run 2>&1 | grep 'error'

# Emergency exit events
RUST_LOG=debug keyrx run 2>&1 | grep -E '(emergency|bypass)'
```

### Build with Debug Symbols

For detailed stack traces and debugging:

```bash
# Development build with debug symbols (default)
cargo build

# Release build with debug symbols
cargo build --profile release-debug

# Run with backtrace enabled
RUST_BACKTRACE=1 RUST_LOG=debug keyrx run
RUST_BACKTRACE=full RUST_LOG=trace keyrx run  # Full backtrace
```

## Common Issues

### Linux-Specific Issues

#### 1. Permission Denied: /dev/input/eventX

**Error:**
```
Permission denied: /dev/input/event4
Hint: Add your user to the 'input' group: sudo usermod -aG input $USER
```

**Solution:**
```bash
# Add user to input group
sudo usermod -aG input $USER

# Create udev rule for input devices
echo 'KERNEL=="event*", MODE="0660", GROUP="input"' | sudo tee /etc/udev/rules.d/99-input.rules

# Reload udev rules
sudo udevadm control --reload-rules && sudo udevadm trigger

# Log out and log back in for group changes to take effect
# Or use: newgrp input
```

**Verify:**
```bash
# Check group membership
groups $USER | grep input

# Check device permissions
ls -la /dev/input/event*
# Should show: crw-rw---- 1 root input ...
```

#### 2. uinput Device Not Found

**Error:**
```
Device not found: /dev/uinput

Remediation:
  1. Load the uinput kernel module: sudo modprobe uinput
  2. If that fails, check if uinput is built into your kernel
  3. Ensure your kernel supports CONFIG_INPUT_UINPUT
```

**Solution:**
```bash
# Load uinput module
sudo modprobe uinput

# Make it load on boot
echo "uinput" | sudo tee /etc/modules-load.d/uinput.conf

# Verify module is loaded
lsmod | grep uinput

# Check device exists
ls -la /dev/uinput
```

#### 3. uinput Permission Denied

**Error:**
```
Permission denied accessing /dev/uinput

Remediation:
  1. Add your user to the 'input' group: sudo usermod -aG input $USER
  2. Create a udev rule for uinput access:
     echo 'KERNEL=="uinput", MODE="0660", GROUP="input"' | sudo tee /etc/udev/rules.d/99-uinput.rules
  3. Reload udev rules: sudo udevadm control --reload-rules && sudo udevadm trigger
  4. Log out and log back in for group changes to take effect
```

**Solution:**
```bash
# Add user to input group (if not already done)
sudo usermod -aG input $USER

# Create udev rule for uinput
echo 'KERNEL=="uinput", MODE="0660", GROUP="input"' | sudo tee /etc/udev/rules.d/99-uinput.rules

# Reload udev rules
sudo udevadm control --reload-rules
sudo udevadm trigger

# Verify permissions
ls -la /dev/uinput
# Should show: crw-rw---- 1 root input ...
```

#### 4. Device Disconnected During Operation

**Error:**
```
Device disconnected: /dev/input/event4
```

**What happens:**
- KeyRx automatically ungrab s the device
- Retry logic attempts reconnection with exponential backoff
- Up to 5 retry attempts with delays: 100ms, 200ms, 400ms, 800ms, 1600ms
- If device reconnects, operation resumes automatically

**Manual recovery:**
```bash
# Check device status
ls -la /dev/input/event*

# Restart KeyRx if auto-recovery fails
# Emergency exit (Ctrl+Alt+Shift+Esc) stops the driver safely
```

**Debug:**
```bash
# Watch device events
RUST_LOG=keyrx_core::drivers::linux=trace keyrx run 2>&1 | grep -E '(disconnect|retry|reconnect)'
```

#### 5. Device Grab Failed

**Error:**
```
Failed to grab device: Device or resource busy
```

**Causes:**
- Another application is using the device exclusively
- Previous KeyRx instance didn't release the device

**Solution:**
```bash
# Check what's using the device
sudo lsof /dev/input/event4

# Kill other KeyRx instances
pkill keyrx

# If still stuck, reboot or:
sudo rmmod evdev
sudo modprobe evdev
```

### Windows-Specific Issues

#### 1. Hook Installation Failed

**Error:**
```
Hook installation failed with error code: 0x5
```

**Error code meanings:**
- `0x5` (ERROR_ACCESS_DENIED) - Insufficient permissions
- `0x1` (ERROR_INVALID_FUNCTION) - Invalid hook type
- `0x8` (ERROR_NOT_ENOUGH_MEMORY) - System resources exhausted

**Solution:**
```powershell
# Run as Administrator
Right-click keyrx.exe -> "Run as administrator"

# Check for conflicting software
# - Other keyboard remappers (AutoHotkey, SharpKeys, etc.)
# - Antivirus/security software blocking hooks
# - Game anti-cheat software
```

#### 2. Hook Unhook Failed

**Error:**
```
Failed to unhook keyboard hook: Hook handle is null
```

**What this means:**
- Usually safe to ignore
- Hook may have already been removed
- Can occur during emergency exit or shutdown

**If persistent:**
```powershell
# Restart KeyRx
# Emergency exit (Ctrl+Alt+Shift+Esc) first if needed
```

#### 3. Callback Panic

**Error:**
```
Callback panicked: thread 'main' panicked at 'assertion failed'
```

**What happens:**
- Panic is caught by `HookCallback` wrapper
- Returns `PassThrough` to maintain system stability
- Panic details logged for debugging
- Hook remains installed and functional

**Recovery:**
- Automatic - callback continues processing subsequent events
- Use emergency exit if behavior is incorrect

**Debug:**
```powershell
# Enable panic backtraces
$env:RUST_BACKTRACE="1"
$env:RUST_LOG="debug"
keyrx run
```

## Error Recovery

### Automatic Retry with Exponential Backoff

KeyRx automatically retries transient errors using exponential backoff:

**Retryable errors:**
- Device disconnection
- Temporary I/O errors (interrupted, would-block, timeout)
- Device grab failures
- Event injection failures

**Retry configuration:**
- **Max retries:** 5 attempts
- **Initial delay:** 100ms
- **Max delay:** 10 seconds
- **Backoff multiplier:** 2.0 (delay doubles each attempt)

**Example retry sequence:**
```
Attempt 1: Operation failed (Temporary error)
Waiting 100ms...
Attempt 2: Operation failed (Temporary error)
Waiting 200ms...
Attempt 3: Operation failed (Temporary error)
Waiting 400ms...
Attempt 4: Operation succeeded ✓
```

**Non-retryable errors** (fail immediately):
- Permission denied
- Device not found
- Invalid configuration
- Hook installation failure

**Debug retry behavior:**
```bash
RUST_LOG=keyrx_core::drivers::common::recovery=debug keyrx run
```

**Look for logs:**
```
[DEBUG] Retrying operation after delay (attempt=1, delay_ms=100)
[DEBUG] Operation succeeded after retry (attempt=2)
[WARN] Operation failed after maximum retries (attempts=5)
```

## Emergency Exit

### What It Does

**Ctrl+Alt+Shift+Esc** is a critical safety mechanism that:
1. Immediately disables all key remapping
2. Restores normal keyboard behavior
3. Works even during errors, panics, or device failures
4. **Cannot be blocked** by any configuration

### How It Works

**Priority guarantee:**
- Checked **first** before any event processing
- Never skipped or delayed
- Works in all error states

**Implementation:**
- **Linux:** Ungrab s device, stops event processing
- **Windows:** All keys pass through hook unchanged

**State persistence:**
- Bypass mode persists until toggled again
- Press **Ctrl+Alt+Shift+Esc** again to re-enable remapping

### Testing Emergency Exit

```bash
# Run with debug logs to see emergency exit activation
RUST_LOG=debug keyrx run

# Trigger emergency exit: Ctrl+Alt+Shift+Esc
# You should see:
# [WARN] Emergency exit triggered - ungrabbing device (Linux)
# or
# [INFO] Bypass mode activated (Windows)
```

### Verifying It Works

**Test scenarios:**
1. Normal operation - should toggle bypass mode
2. During high load - should still respond instantly
3. During device error - should ungrab/unhook successfully
4. After panic - should still be checkable

**Comprehensive tests:**
```bash
# Run emergency exit test suite
cargo test emergency_exit

# Run error scenario tests
cargo test --test emergency_exit_error_scenarios_test

# Verbose output
cargo test emergency_exit -- --nocapture
```

See [emergency-exit-safety.md](./emergency-exit-safety.md) for detailed verification.

## Debugging Techniques

### 1. Trace Driver Events

**Linux:**
```bash
# Trace all driver operations
RUST_LOG=keyrx_core::drivers::linux=trace keyrx run 2>&1 | tee driver.log

# Watch for specific events
RUST_LOG=trace keyrx run 2>&1 | grep -E '(grab|ungrab|inject|read)'
```

**Windows:**
```bash
# Trace hook operations
RUST_LOG=keyrx_core::drivers::windows=trace keyrx run 2>&1 | tee driver.log

# Watch for hook events
RUST_LOG=trace keyrx run 2>&1 | grep -E '(hook|unhook|callback)'
```

### 2. Monitor Device State

**Linux:**
```bash
# Watch device addition/removal
udevadm monitor --kernel --subsystem-match=input

# Check device properties
udevadm info /dev/input/event4

# Test device reading (requires permission)
sudo evtest /dev/input/event4
```

### 3. Inspect Error Patterns

**Check error types:**
```bash
RUST_LOG=warn keyrx run 2>&1 | grep -E 'error|Error|ERROR' | sort | uniq -c
```

**Monitor retry attempts:**
```bash
RUST_LOG=debug keyrx run 2>&1 | grep -i retry
```

**Track recovery success:**
```bash
RUST_LOG=debug keyrx run 2>&1 | grep -E '(succeeded after retry|recovery)'
```

### 4. Test Panic Handling

**Linux:**
```bash
# Panics are caught and logged
RUST_LOG=debug keyrx run

# Check for panic recovery:
# - Device should be ungrabbed
# - Emergency exit should still work
# - Application should handle gracefully
```

**Windows:**
```bash
# Callback panics are caught
RUST_LOG=debug keyrx run

# Check for panic in callback:
# - Logged as "Callback panicked"
# - Hook remains installed
# - Returns PassThrough for that event
```

### 5. Profile Performance

**Build with profiling:**
```bash
cargo build --profile release-debug
```

**Linux profiling:**
```bash
# Install perf tools
sudo apt-get install linux-tools-generic

# Record performance data
sudo perf record -g ./target/release-debug/keyrx run
sudo perf report
```

**Flamegraph:**
```bash
# Install cargo-flamegraph
cargo install flamegraph

# Generate flamegraph
sudo cargo flamegraph --bin keyrx -- run
```

## Reporting Issues

When reporting driver issues, include:

### 1. System Information

**Linux:**
```bash
# Gather system info
uname -a
lsb_release -a
cat /proc/version
lsmod | grep -E '(evdev|uinput)'
```

**Windows:**
```powershell
# Gather system info
systeminfo
Get-ComputerInfo | Select-Object WindowsVersion, OSArchitecture
```

### 2. Driver Logs

```bash
# Capture debug logs
RUST_LOG=debug keyrx run 2>&1 | tee keyrx-debug.log

# Include the log file when reporting
```

### 3. Device Information

**Linux:**
```bash
# List input devices
ls -la /dev/input/event*
cat /proc/bus/input/devices

# Device-specific info
udevadm info /dev/input/event4
```

**Windows:**
```powershell
# List keyboard devices
Get-PnpDevice -Class Keyboard
```

### 4. Error Context

- **What were you doing when the error occurred?**
- **Can you reproduce the issue consistently?**
- **What configuration/remapping were you using?**
- **Did emergency exit work correctly?**

### 5. Test Results

```bash
# Run driver tests
cargo test --lib drivers

# Run integration tests
cargo test --test '*driver*'

# Include test output
```

## Advanced Debugging

### Unsafe Block Tracing

All unsafe blocks in drivers have `SAFETY` comments explaining invariants:

```bash
# Find all unsafe blocks
rg -A 5 'unsafe' core/src/drivers/

# Check SAFETY documentation
rg 'SAFETY:' core/src/drivers/
```

### Safety Wrapper Debugging

**Linux SafeDevice:**
```bash
# Trace device operations
RUST_LOG=keyrx_core::drivers::linux::safety::device=trace keyrx run
```

**Windows SafeHook:**
```bash
# Trace hook lifecycle
RUST_LOG=keyrx_core::drivers::windows::safety::hook=trace keyrx run
```

### Thread-Local State Debugging

**Windows:**
```bash
# Debug thread-local routing
RUST_LOG=keyrx_core::drivers::windows::safety::thread_local=trace keyrx run
```

## Summary

This guide covers:
- ✅ Environment variable configuration (RUST_LOG, etc.)
- ✅ Common issues and solutions for both platforms
- ✅ Error recovery and retry mechanisms
- ✅ Emergency exit functionality and testing
- ✅ Debugging techniques and log analysis
- ✅ Issue reporting best practices

For additional help:
- **Architecture:** [ARCHITECTURE.md](./ARCHITECTURE.md)
- **Emergency Exit Details:** [emergency-exit-safety.md](./emergency-exit-safety.md)
- **Feature Flags:** [features.md](./features.md)
- **Source Code:** `core/src/drivers/`
