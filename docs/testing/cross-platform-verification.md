# Cross-Platform Verification Guide

This document describes how to verify that KeyRx configurations (.krx files) produce identical behavior across Linux, Windows, and macOS platforms.

## Purpose

KeyRx is designed to be truly cross-platform, where the same compiled .krx configuration file works identically on all supported platforms. This guide provides procedures to verify this core requirement.

## Prerequisites

### Hardware Requirements
- One machine running Linux (Ubuntu 20.04+ recommended)
- One machine running Windows (10 or 11)
- One machine running macOS (12.0+ Monterey, Ventura, or Sonoma)

Alternatively:
- One physical machine with multi-boot setup
- Virtual machines for Linux and Windows (note: macOS cannot be legally virtualized on non-Apple hardware)

### Software Requirements
- KeyRx binaries built for each platform:
  ```bash
  # Linux
  cargo build --release -p keyrx_daemon --features linux
  cargo build --release -p keyrx_compiler

  # Windows
  cargo build --release -p keyrx_daemon --features windows
  cargo build --release -p keyrx_compiler

  # macOS
  cargo build --release -p keyrx_daemon --features macos
  cargo build --release -p keyrx_compiler
  ```

### Test Keyboards
- At least one USB keyboard that works on all three platforms
- Ideally 2+ keyboards for multi-device testing

## Test Configurations

Create test configurations that cover all KeyRx features:

### 1. Basic Remapping (test-basic.rhai)

```rhai
device_start("*");
    // Simple key remaps
    map("CapsLock", "VK_Escape");
    map("A", "VK_B");
    map("1", "VK_2");
device_end();
```

**Expected behavior (all platforms):**
- Pressing CapsLock produces Escape
- Pressing A produces B
- Pressing 1 produces 2

### 2. Custom Modifiers / Layers (test-layers.rhai)

```rhai
device_start("*");
    // CapsLock becomes layer modifier
    map("CapsLock", "MD_00");

    // Vim navigation layer
    when_start("MD_00");
        map("H", "VK_Left");
        map("J", "VK_Down");
        map("K", "VK_Up");
        map("L", "VK_Right");
        map("W", "VK_ControlLeft");  // Ctrl for word navigation
    when_end();
device_end();
```

**Expected behavior (all platforms):**
- Normal typing: H, J, K, L produce those letters
- With CapsLock held: H→Left, J→Down, K→Up, L→Right

### 3. Tap-Hold Behavior (test-tap-hold.rhai)

```rhai
device_start("*");
    // Space: tap=space, hold=layer
    tap_hold("Space", "VK_Space", "MD_01", 200);

    when_start("MD_01");
        map("H", "VK_Left");
        map("J", "VK_Down");
        map("K", "VK_Up");
        map("L", "VK_Right");
    when_end();
device_end();
```

**Expected behavior (all platforms):**
- Tapping Space quickly (<200ms) → space character
- Holding Space (≥200ms) then pressing H → Left arrow
- Timing should be consistent across platforms

### 4. Custom Locks / Toggles (test-locks.rhai)

```rhai
device_start("*");
    // ScrollLock toggles gaming mode
    map("ScrollLock", "LK_00");

    // When gaming mode active, WASD → arrow keys
    when_start("LK_00");
        map("W", "VK_Up");
        map("A", "VK_Left");
        map("S", "VK_Down");
        map("D", "VK_Right");
    when_end();
device_end();
```

**Expected behavior (all platforms):**
- Press ScrollLock once → mode activates
- WASD now produce arrow keys
- Press ScrollLock again → mode deactivates
- WASD produce normal letters

### 5. Multi-Device Configuration (test-multi-device.rhai)

```rhai
// Device-specific config for built-in keyboard
device_start("*Internal*");
    map("CapsLock", "VK_Escape");
device_end();

// Device-specific config for external keyboard
device_start("USB*");
    map("CapsLock", "MD_00");
    when_start("MD_00");
        map("H", "VK_Left");
    when_end();
device_end();

// Wildcard fallback
device_start("*");
    map("A", "VK_B");
device_end();
```

**Expected behavior (all platforms):**
- Built-in keyboard: CapsLock→Escape
- External USB keyboard: CapsLock activates layer
- All keyboards: A→B

## Verification Procedure

### Step 1: Compile Configuration on One Platform

Choose one platform (e.g., Linux) as the reference:

```bash
# On Linux
./target/release/keyrx_compiler compile test-basic.rhai -o test-basic.krx
./target/release/keyrx_compiler compile test-layers.rhai -o test-layers.krx
./target/release/keyrx_compiler compile test-tap-hold.rhai -o test-tap-hold.krx
./target/release/keyrx_compiler compile test-locks.rhai -o test-locks.krx
./target/release/keyrx_compiler compile test-multi-device.rhai -o test-multi-device.krx
```

### Step 2: Verify .krx Hash

Verify that the compiled .krx file has a valid hash:

```bash
./target/release/keyrx_compiler hash test-basic.krx --verify
```

**Expected output:**
```
✓ Hash verification passed
SHA256: abc123...
```

### Step 3: Copy .krx Files to All Platforms

Transfer the compiled .krx files to Windows and macOS:

```bash
# Example: Using SCP
scp test-*.krx user@windows-machine:/path/to/keyrx/
scp test-*.krx user@macos-machine:/path/to/keyrx/
```

### Step 4: Verify .krx Files Are Identical

On each platform, verify the SHA256 hash matches:

```bash
# Linux
sha256sum test-basic.krx

# Windows (PowerShell)
Get-FileHash test-basic.krx -Algorithm SHA256

# macOS
shasum -a 256 test-basic.krx
```

**Success criteria:** All three hashes are identical.

### Step 5: Run Validation on Each Platform

Validate the configuration on each platform:

```bash
# Linux
sudo ./target/release/keyrx_daemon validate --config test-basic.krx

# Windows
.\target\release\keyrx_daemon.exe validate --config test-basic.krx

# macOS
./target/release/keyrx_daemon validate --config test-basic.krx
```

**Expected output (all platforms):**
```
Step 1/3: Loading configuration...
  [OK] Configuration loaded successfully

Step 2/3: Enumerating input devices...
  Found N keyboard device(s)

Step 3/3: Matching devices to configuration...
  [MATCH] Device Name
          Matched pattern: "*" (X mappings)

Validation successful! N device(s) will be remapped.
```

### Step 6: Test Actual Behavior

Run the daemon and verify behavior:

```bash
# Linux
sudo ./target/release/keyrx_daemon run --config test-basic.krx

# Windows
.\target\release\keyrx_daemon.exe run --config test-basic.krx

# macOS
./target/release/keyrx_daemon run --config test-basic.krx
```

For each test configuration, manually verify the expected behavior from the "Test Configurations" section above.

## Behavior Verification Checklist

For each test configuration, verify on all three platforms:

### Basic Remapping (test-basic.krx)
- [ ] Linux: CapsLock→Escape, A→B, 1→2
- [ ] Windows: CapsLock→Escape, A→B, 1→2
- [ ] macOS: CapsLock→Escape, A→B, 1→2

### Custom Modifiers (test-layers.krx)
- [ ] Linux: CapsLock+H/J/K/L produces arrows
- [ ] Windows: CapsLock+H/J/K/L produces arrows
- [ ] macOS: CapsLock+H/J/K/L produces arrows

### Tap-Hold (test-tap-hold.krx)
- [ ] Linux: Tap Space→space, Hold Space+H→Left
- [ ] Windows: Tap Space→space, Hold Space+H→Left
- [ ] macOS: Tap Space→space, Hold Space+H→Left
- [ ] Timing consistent across platforms (±10ms acceptable)

### Custom Locks (test-locks.krx)
- [ ] Linux: ScrollLock toggles WASD→arrows
- [ ] Windows: ScrollLock toggles WASD→arrows
- [ ] macOS: ScrollLock toggles WASD→arrows
- [ ] State persists across different keys

### Multi-Device (test-multi-device.krx)
- [ ] Linux: Device-specific configs work
- [ ] Windows: Device-specific configs work
- [ ] macOS: Device-specific configs work
- [ ] Wildcard pattern works consistently

## Known Platform Differences

While KeyRx strives for identical behavior, some platform-specific differences are acceptable:

### 1. Device Names

Device names may differ across platforms:
- **Linux:** `/dev/input/event0`, `USB Keyboard`
- **Windows:** `HID Keyboard Device`, `\\?\HID#...`
- **macOS:** `Apple Internal Keyboard`, `Magic Keyboard`

**Impact:** Use wildcards (`*`) for cross-platform configs, or platform-specific device names.

### 2. System Modifier Keys

- **Linux/Windows:** Use `LeftSuper`/`RightSuper` for Windows key
- **macOS:** Use `LeftSuper`/`RightSuper` for Command (⌘) key

**Impact:** None - KeyRx abstracts this correctly.

### 3. Exclusive Keyboard Grab

- **Linux:** Full exclusive grab available (blocks other apps)
- **Windows:** Partial grab (low-level hooks)
- **macOS:** No exclusive grab (CGEventTap doesn't block other apps from seeing events)

**Impact:** On macOS, other remapping tools (Karabiner-Elements) may also see events. Use only one remapping tool at a time.

### 4. Permission Requirements

- **Linux:** Requires root or udev rules + input group
- **Windows:** No special permissions
- **macOS:** Requires Accessibility permission

**Impact:** Setup process differs, but behavior is identical once running.

## Automated Testing

For deterministic verification, use the WASM simulator:

```bash
cd keyrx_ui
npm run dev
```

1. Load the same .krx file in the browser
2. Use the virtual keyboard to simulate key presses
3. Verify output matches expected behavior
4. Simulator behavior should be identical to all platforms

## Troubleshooting

### Different Behavior Observed

If behavior differs across platforms:

1. **Verify .krx file is identical:**
   ```bash
   sha256sum test-basic.krx  # Compare hashes
   ```

2. **Check daemon version:**
   ```bash
   ./target/release/keyrx_daemon --version
   ```
   All platforms should use the same version.

3. **Enable debug logging:**
   ```bash
   ./target/release/keyrx_daemon run --config test.krx --debug
   ```
   Compare logs across platforms to identify differences.

4. **Check for interference:**
   - Linux: Disable other input tools (xmodmap, xkb)
   - Windows: Disable other remapping tools (AutoHotkey, PowerToys)
   - macOS: Disable Karabiner-Elements, BetterTouchTool

### Timing Differences

If tap-hold timing differs:

1. **Measure actual latency:**
   - Use criterion benchmarks on each platform
   - Compare p50 and p95 latencies

2. **Adjust threshold:**
   - Increase tap-hold threshold (e.g., 200ms → 250ms)
   - Use same threshold across platforms for consistency

### Device Matching Issues

If device names don't match:

1. **List devices on each platform:**
   ```bash
   ./target/release/keyrx_daemon list-devices
   ```

2. **Use wildcard patterns:**
   ```rhai
   device_start("*");  // Matches all devices
   ```

3. **Use partial matching:**
   ```rhai
   device_start("*USB*");  // Matches any device with "USB" in name
   ```

## Success Criteria

Cross-platform verification is successful when:

1. ✅ Same .krx file loads on all platforms (hash verified)
2. ✅ `validate` command succeeds on all platforms
3. ✅ All test configurations produce identical behavior
4. ✅ No platform-specific bugs or crashes
5. ✅ Performance is comparable across platforms (±10% acceptable)
6. ✅ Known differences are documented and acceptable

## Reporting Issues

If cross-platform behavior differs unexpectedly:

1. Document the difference:
   - Platform where it works correctly
   - Platform where it differs
   - Expected vs. actual behavior
   - Debug logs from both platforms

2. Create a minimal reproduction:
   - Simplest .krx that demonstrates the issue
   - Exact steps to reproduce

3. Report on GitHub Issues:
   - Include platform details (OS version, architecture)
   - Include KeyRx version (`keyrx_daemon --version`)
   - Attach .krx file and debug logs

## References

- [Linux Setup Guide](../user-guide/linux-setup.md)
- [Windows Setup Guide](../user-guide/windows-setup.md)
- [macOS Setup Guide](../user-guide/macos-setup.md)
- [DSL Manual](../user-guide/dsl-manual.md)
- [E2E Test Checklist](macos-e2e-checklist.md)
