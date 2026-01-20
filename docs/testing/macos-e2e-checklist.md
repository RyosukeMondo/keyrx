# macOS End-to-End Testing Checklist

This checklist ensures comprehensive pre-release testing of KeyRx on macOS. All tests require Accessibility permission and should be run on real hardware.

## Test Environment

**Before starting:**
- [ ] macOS version: __________ (12.0+)
- [ ] Architecture: [ ] Intel (x86_64) [ ] Apple Silicon (ARM64)
- [ ] Accessibility permission granted to keyrx_daemon
- [ ] Test configuration file available (use `examples/03-vim-navigation.rhai` or similar)
- [ ] Multiple USB keyboards connected (for multi-device tests)

**Build the daemon:**
```bash
cargo build --release -p keyrx_daemon --features macos
```

## 1. Permission Flow

### 1.1 Permission Check
- [ ] Run daemon without permission → clear error message displayed
- [ ] Error message includes step-by-step instructions
- [ ] Error message references `docs/user-guide/macos-setup.md`

**Command:**
```bash
./target/release/keyrx_daemon run --config test.krx
```

**Expected:** Permission denied error with instructions

### 1.2 Permission Grant
- [ ] Follow instructions to grant Accessibility permission
- [ ] Application appears in System Settings > Privacy & Security > Accessibility
- [ ] Toggle can be enabled
- [ ] Daemon starts successfully after permission granted

## 2. Device Enumeration

### 2.1 List Devices
- [ ] `list-devices` command works without errors
- [ ] All connected keyboards are detected
- [ ] Device names are readable and accurate
- [ ] VID:PID displayed correctly for USB devices
- [ ] Serial numbers displayed (or "-" for devices without)
- [ ] Built-in keyboard detected

**Command:**
```bash
./target/release/keyrx_daemon list-devices
```

**Expected output format:**
```
Available keyboard devices:
NAME                           VID:PID        SERIAL
Apple Internal Keyboard        05ac:027e      -
Magic Keyboard                 05ac:0267      A1B2C3D4E5F6
```

### 2.2 Device Matching
- [ ] Configuration with specific device name matches correctly
- [ ] Wildcard pattern (`device_start("*")`) matches all devices
- [ ] Non-matching device names are ignored

**Measurement:** Use `validate` command to check device matching

## 3. Performance Testing

### 3.1 Input Capture Latency
- [ ] Latency <1ms (95th percentile)
- [ ] Latency <500μs (median)
- [ ] No dropped events during rapid typing

**Measurement method:**
```bash
# Option 1: Use criterion benchmark
cargo bench --bench macos_latency -- input_capture

# Option 2: Manual measurement with debug logging
./target/release/keyrx_daemon run --config test.krx --debug | grep -i latency
```

**Tool:** `keyrx_daemon/benches/macos_latency.rs` (if implemented)

### 3.2 Output Injection Latency
- [ ] Latency <1ms (95th percentile)
- [ ] Latency <500μs (median)
- [ ] Injected events appear immediately

**Measurement method:**
```bash
cargo bench --bench macos_latency -- output_injection
```

### 3.3 Full Pipeline Latency
- [ ] End-to-end latency <1ms (95th percentile)
- [ ] User perceives instant response during typing
- [ ] No noticeable lag during rapid key sequences

**Measurement method:**
```bash
cargo bench --bench macos_latency -- full_pipeline
```

**Manual test:** Type rapidly while monitoring for lag

## 4. Functional Testing

### 4.1 Basic Key Remapping
- [ ] Simple remap works (`CapsLock → Escape`)
- [ ] Multiple simple remaps work simultaneously
- [ ] Physical modifier keys work (Shift, Ctrl, Alt, Cmd)

**Test config:**
```rhai
device_start("*");
    map("CapsLock", "VK_Escape");
    map("A", "VK_B");
device_end();
```

**Manual test:** Press CapsLock → Escape appears, Press A → B appears

### 4.2 Custom Modifiers (Layers)
- [ ] Custom modifier activates layer
- [ ] Keys behave differently when modifier held
- [ ] Layer deactivates when modifier released
- [ ] Multiple layers can be active simultaneously

**Test config:**
```rhai
device_start("*");
    map("CapsLock", "MD_00");
    when_start("MD_00");
        map("H", "VK_Left");
        map("J", "VK_Down");
        map("K", "VK_Up");
        map("L", "VK_Right");
    when_end();
device_end();
```

**Manual test:** Hold CapsLock, press HJKL → arrow keys appear

### 4.3 Tap-Hold Behavior
- [ ] Tap produces tap action
- [ ] Hold produces hold action
- [ ] Threshold timing correct (e.g., 200ms)
- [ ] Rapid tap-tap-tap works correctly

**Test config:**
```rhai
device_start("*");
    tap_hold("Space", "VK_Space", "MD_01", 200);
    when_start("MD_01");
        map("H", "VK_Left");
    when_end();
device_end();
```

**Manual test:**
- Tap Space quickly → space appears
- Hold Space 200ms+ → H produces Left arrow

### 4.4 Custom Locks (Toggles)
- [ ] Lock toggles on first press
- [ ] Lock remains active until toggled again
- [ ] Lock state persists across different keys
- [ ] Lock state indicator works (if implemented)

**Test config:**
```rhai
device_start("*");
    map("ScrollLock", "LK_00");
    when_start("LK_00");
        map("A", "VK_1");
    when_end();
device_end();
```

**Manual test:**
- Press ScrollLock → lock activates
- Press A → 1 appears
- Press ScrollLock → lock deactivates
- Press A → A appears

### 4.5 Multi-Device Configuration
- [ ] Device-specific configs work
- [ ] Wildcard config applies to all devices
- [ ] More specific configs override wildcard
- [ ] Device hotplug works (unplug/replug keyboard)

**Test config:**
```rhai
device_start("Apple Internal Keyboard");
    map("CapsLock", "VK_Escape");
device_end();

device_start("Magic Keyboard");
    map("CapsLock", "MD_00");
device_end();

device_start("*");
    map("A", "VK_B");
device_end();
```

**Manual test:**
- CapsLock behaves differently on each keyboard
- A→B remap works on all keyboards

### 4.6 Cross-Device Modifiers
- [ ] Shift on keyboard A affects keyboard B
- [ ] Ctrl on keyboard A affects keyboard B
- [ ] Custom modifiers on keyboard A affect keyboard B
- [ ] Global state shared across all devices

**Manual test:**
- Hold Shift on keyboard A
- Press letter on keyboard B → uppercase appears
- Hold custom modifier on keyboard A
- Press mapped key on keyboard B → layer mapping works

## 5. Configuration Management

### 5.1 Validation (Dry-Run)
- [ ] `validate` command works without errors
- [ ] Shows all matched devices
- [ ] Shows mapping count per device
- [ ] Detects configuration errors

**Command:**
```bash
./target/release/keyrx_daemon validate --config test.krx
```

**Expected:** Clear output showing device matches and mapping counts

### 5.2 Hot Reload
- [ ] Config reload via SIGHUP works
- [ ] Config reload via menu bar works (Reload Config)
- [ ] Reload completes within 500ms
- [ ] Modifier states preserved during reload
- [ ] Lock states preserved during reload
- [ ] No event drops during reload

**Commands:**
```bash
# Method 1: SIGHUP
killall -HUP keyrx_daemon

# Method 2: Menu bar
# Click menu bar icon → Reload Config
```

**Measurement:** Time the reload operation with `time` or debug logs

## 6. System Tray / Menu Bar Integration

### 6.1 Menu Bar Icon
- [ ] Icon appears in menu bar
- [ ] Icon is visible and recognizable
- [ ] Clicking icon shows menu

### 6.2 Menu Items
- [ ] "Open Web UI" menu item exists
- [ ] "Reload Config" menu item exists
- [ ] "Exit" menu item exists
- [ ] All menu items clickable

### 6.3 Menu Actions
- [ ] "Open Web UI" opens browser to correct URL
- [ ] "Reload Config" reloads configuration
- [ ] "Exit" gracefully shuts down daemon
- [ ] No zombie processes after exit

**Manual test:** Click each menu item and verify behavior

## 7. Stability Testing

### 7.1 Stress Test - Key Presses
- [ ] Daemon survives 10,000+ key presses without crash
- [ ] No memory leaks during stress test
- [ ] Response time remains consistent
- [ ] No event drops or duplicates

**Test method:**
```bash
# Run daemon in one terminal
./target/release/keyrx_daemon run --config test.krx

# In another terminal, generate key events with automation tool
# Or manually type continuously for 5-10 minutes
```

**Success criteria:** Daemon still running, no errors in logs

### 7.2 Long-Running Session
- [ ] Daemon runs for 1+ hour without crash
- [ ] Memory usage stable (<50MB)
- [ ] No memory growth over time
- [ ] CPU usage <1% idle, <5% under load

**Measurement tools:**
```bash
# Monitor resource usage
top -pid $(pgrep keyrx_daemon)

# Or use Activity Monitor GUI
# Applications → Utilities → Activity Monitor
# Search for "keyrx_daemon"
```

**Success criteria:**
- Memory: <50MB throughout session
- CPU: <1% when idle, <5% during typing

### 7.3 Memory Leak Detection
- [ ] No memory leaks detected by Xcode Instruments
- [ ] Memory graph stable over time
- [ ] No unreleased IOKit objects
- [ ] No unreleased Core Foundation objects

**Measurement tools:**
```bash
# Option 1: Xcode Instruments (Leaks instrument)
# Open Xcode → Open Developer Tool → Instruments
# Select "Leaks" template
# Attach to keyrx_daemon process
# Type for 5-10 minutes, check for leaks

# Option 2: leaks command-line tool
sudo leaks keyrx_daemon --atExit
```

**Success criteria:** Zero leaks detected

## 8. Error Handling

### 8.1 Invalid Configuration
- [ ] Invalid .krx file rejected with clear error
- [ ] Corrupted .krx file rejected with clear error
- [ ] Missing .krx file produces helpful error
- [ ] Syntax errors reported clearly

**Test:**
```bash
# Invalid file
./target/release/keyrx_daemon run --config /dev/null

# Missing file
./target/release/keyrx_daemon run --config nonexistent.krx
```

### 8.2 Permission Revocation
- [ ] Daemon detects permission revocation
- [ ] Clear error message when permission lost
- [ ] Graceful shutdown or retry logic

**Test:** Revoke permission while daemon running

### 8.3 Device Disconnect
- [ ] Daemon handles keyboard disconnect gracefully
- [ ] Daemon resumes when keyboard reconnected
- [ ] No crash on device hotplug

**Test:** Unplug USB keyboard while daemon running

## 9. Compatibility Testing

### 9.1 macOS Versions
Test on multiple macOS versions:
- [ ] macOS 14 (Sonoma) - Latest
- [ ] macOS 13 (Ventura)
- [ ] macOS 12 (Monterey) - Minimum supported

### 9.2 Architectures
- [ ] Intel (x86_64) build works on Intel Macs
- [ ] ARM64 (aarch64) build works on Apple Silicon
- [ ] Universal binary works on both architectures

### 9.3 Keyboard Types
- [ ] Built-in Mac keyboard works
- [ ] USB keyboards work
- [ ] Bluetooth keyboards work
- [ ] Third-party mechanical keyboards work

## 10. Cross-Platform Verification

### 10.1 Configuration Portability
- [ ] Same .krx file works on Linux
- [ ] Same .krx file works on Windows
- [ ] Same .krx file works on macOS
- [ ] Identical behavior across all platforms

**Test method:**
1. Compile a test config on one platform
2. Copy .krx file to macOS
3. Run daemon with the .krx
4. Verify behavior matches original platform

### 10.2 Binary Compatibility
- [ ] .krx compiled on Linux works on macOS
- [ ] .krx compiled on Windows works on macOS
- [ ] .krx compiled on macOS works on other platforms

## Test Results Summary

**Date:** __________
**Tester:** __________
**macOS Version:** __________
**Architecture:** __________

| Category | Pass | Fail | Notes |
|----------|------|------|-------|
| Permission Flow | ☐ | ☐ | |
| Device Enumeration | ☐ | ☐ | |
| Performance | ☐ | ☐ | |
| Functional Tests | ☐ | ☐ | |
| Config Management | ☐ | ☐ | |
| Menu Bar | ☐ | ☐ | |
| Stability | ☐ | ☐ | |
| Error Handling | ☐ | ☐ | |
| Compatibility | ☐ | ☐ | |
| Cross-Platform | ☐ | ☐ | |

**Overall Result:** [ ] PASS [ ] FAIL

**Critical Issues Found:**
-

**Non-Critical Issues Found:**
-

**Performance Metrics:**
- Input capture latency (p95): ____ ms
- Output injection latency (p95): ____ ms
- Full pipeline latency (p95): ____ ms
- Memory usage (1hr session): ____ MB
- CPU usage (idle): ____ %
- CPU usage (typing): ____ %

**Recommendations:**
-

---

## Tools and Commands Reference

### Build Commands
```bash
# Build daemon
cargo build --release -p keyrx_daemon --features macos

# Build compiler
cargo build --release -p keyrx_compiler

# Build benchmarks
cargo bench --bench macos_latency --no-run
```

### Daemon Commands
```bash
# List devices
./target/release/keyrx_daemon list-devices

# Validate config
./target/release/keyrx_daemon validate --config test.krx

# Run daemon
./target/release/keyrx_daemon run --config test.krx

# Run with debug logging
./target/release/keyrx_daemon run --config test.krx --debug
```

### Monitoring Commands
```bash
# Monitor process resources
top -pid $(pgrep keyrx_daemon)

# Check for memory leaks
sudo leaks keyrx_daemon

# View logs
tail -f /path/to/keyrx.log
```

### Benchmark Commands
```bash
# Run all benchmarks
cargo bench --bench macos_latency

# Run specific benchmark
cargo bench --bench macos_latency -- input_capture
cargo bench --bench macos_latency -- output_injection
cargo bench --bench macos_latency -- full_pipeline
```

## Notes

- **Accessibility Permission:** Required for all tests. Cannot be automated in CI.
- **Real Hardware:** Tests must run on actual macOS hardware, not VMs.
- **Multiple Keyboards:** Some tests require 2+ USB keyboards connected.
- **Xcode Tools:** Required for Instruments-based leak detection.
- **Criterion:** Performance benchmarks use the criterion crate.

## References

- [macOS Setup Guide](../user-guide/macos-setup.md)
- [Design Document](../../.spec-workflow/specs/macos-support/design.md)
- [Performance Requirements](../../.spec-workflow/specs/macos-support/requirements.md)
