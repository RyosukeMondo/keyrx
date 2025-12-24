# Multi-Device Configuration Guide

Complete guide for configuring KeyRx with multiple keyboards and input devices.

## Table of Contents

- [Overview](#overview)
- [Use Cases](#use-cases)
- [Identifying Devices](#identifying-devices)
  - [Listing Connected Devices](#listing-connected-devices)
  - [Device ID Types](#device-id-types)
  - [Serial Numbers vs Path-Based IDs](#serial-numbers-vs-path-based-ids)
- [Device-Specific Configuration](#device-specific-configuration)
  - [Basic Device Blocks](#basic-device-blocks)
  - [Device Pattern Matching](#device-pattern-matching)
  - [Conditional Device Mapping](#conditional-device-mapping)
- [Example Configurations](#example-configurations)
  - [Numpad as Stream Deck](#numpad-as-stream-deck)
  - [Split Keyboard Setup](#split-keyboard-setup)
  - [Gaming Keyboard with Separate Macros](#gaming-keyboard-with-separate-macros)
- [System Tray Usage](#system-tray-usage)
  - [Linux System Tray](#linux-system-tray)
  - [Reload Configuration](#reload-configuration)
  - [Exit Daemon](#exit-daemon)
- [Web Interface](#web-interface)
  - [Device List API](#device-list-api)
  - [Real-Time Device Activity](#real-time-device-activity)
- [Troubleshooting](#troubleshooting)
  - [Permission Issues](#permission-issues)
  - [Device Not Detected](#device-not-detected)
  - [Headless Mode](#headless-mode)
  - [Device Hot-Plug](#device-hot-plug)

---

## Overview

KeyRx supports **keyboard-aware remapping**, allowing you to apply different key mappings to different input devices. This feature enables powerful use cases like:

- Turning a cheap USB numpad into a Stream Deck alternative
- Having different mappings for laptop keyboard vs external keyboard
- Using a dedicated gaming keyboard with custom macros
- Split keyboard configurations (left and right halves)

**Key Concepts:**
- Each physical keyboard is identified by a unique **device ID**
- Device IDs come from USB serial numbers (preferred) or device paths (fallback)
- Mappings can target specific devices using `when_device_start(pattern)`
- All devices share the same modifier/lock state (QMK-inspired design)

---

## Use Cases

### 1. USB Numpad as Stream Deck

Transform a $10 USB numpad into a dedicated macro pad:
- Remap numpad keys to F13-F24 (unused by most applications)
- Assign these keys in OBS, Discord, or other software
- Main keyboard's numpad remains functional

### 2. Split Keyboard Support

Connect two keyboards (left and right halves):
- Left keyboard's Shift activates layers on right keyboard
- Cross-device modifier state sharing
- Inspired by QMK split keyboard architecture

### 3. Laptop + External Keyboard

Different mappings for built-in vs external:
- CapsLock → Escape on laptop
- CapsLock → Hyper key on external (for window management)

### 4. Gaming Keyboard Isolation

Dedicated gaming keyboard with macros:
- Game macros only on gaming keyboard
- Work keyboard unaffected
- No accidental macro triggers during work

---

## Identifying Devices

### Listing Connected Devices

Use the `list-devices` command to see all connected keyboards:

```bash
keyrx_daemon list-devices
```

**Sample Output:**
```
Available keyboard devices:
PATH                     NAME                            SERIAL
/dev/input/event3        AT Translated Set 2 keyboard    -
/dev/input/event18       USB Keyboard                    USB-12345
/dev/input/event21       Generic NumPad                  -

Tip: Use these names in your config with when_device_start("USB Keyboard")
     or use when_device_start("*") to match all keyboards.
```

### Device ID Types

KeyRx uses two types of device identifiers:

| Type | Source | Stability | Example |
|------|--------|-----------|---------|
| **Serial Number** | USB device descriptor | Permanent | `USB-12345`, `Logitech-G910-ABC` |
| **Path-Based ID** | `/dev/input/by-id/` symlink | Stable across reboots | `usb-Logitech_USB_Keyboard-event-kbd` |
| **Event Path** | `/dev/input/eventX` | Changes on reboot | `/dev/input/event18` |

**Recommendation:** Use serial numbers when available, path-based IDs otherwise. Avoid raw event paths.

### Serial Numbers vs Path-Based IDs

**Serial Numbers (Best Choice):**
- Unique per device, never changes
- Works if you move the device to a different USB port
- Not all devices have serial numbers

```bash
# Check if device has serial number
cat /sys/class/input/event18/device/id/serial
# Output: USB-12345 (if available)
```

**Path-Based IDs (Good Fallback):**
- Based on USB vendor/product/port path
- Stable across reboots
- May change if you move device to different USB port

```bash
# View path-based symlinks
ls -la /dev/input/by-id/
# Example: usb-Logitech_USB_Keyboard-event-kbd -> ../event18
```

**Device Names (Convenient but Imprecise):**
- Human-readable from evdev
- May be duplicated if you have two identical keyboards
- Good for pattern matching

---

## Device-Specific Configuration

### Basic Device Blocks

Use `device_start(name)` and `device_end()` to target specific devices:

```rhai
// Apply mappings only to USB Keyboard
device_start("USB Keyboard");
    map("CapsLock", "VK_Escape");
    map("ScrollLock", "LK_00");  // Toggle gaming mode
device_end();

// Different mappings for laptop keyboard
device_start("AT Translated Set 2");
    map("CapsLock", "MD_00");  // Navigation layer
device_end();
```

### Device Pattern Matching

Use `when_device_start(pattern)` for flexible matching with glob patterns:

| Pattern | Matches | Example Devices |
|---------|---------|-----------------|
| `"USB Keyboard"` | Exact match | Only "USB Keyboard" |
| `"USB*"` | Prefix match | "USB Keyboard", "USB NumPad" |
| `"*Keyboard"` | Suffix match | "USB Keyboard", "AT Translated Set 2 keyboard" |
| `"*numpad*"` | Contains | "Generic numpad", "USB-Numpad-123" |
| `"*"` | All devices | Every connected keyboard |

```rhai
// Match any device containing "numpad" in its name
when_device_start("*numpad*");
    map("Numpad1", "VK_F13");
    map("Numpad2", "VK_F14");
    map("Numpad3", "VK_F15");
when_device_end();

// Match all Logitech devices
when_device_start("Logitech*");
    map("G1", "VK_F20");  // Remap G-keys
when_device_end();

// Fallback for all other devices
when_device_start("*");
    map("CapsLock", "VK_Escape");
when_device_end();
```

### Conditional Device Mapping

Combine device matching with conditional blocks for advanced configurations:

```rhai
// USB numpad remapping
when_device_start("*numpad*");
    // Base layer: numpad keys to F13-F21
    map("Numpad1", "VK_F13");
    map("Numpad2", "VK_F14");
    map("Numpad3", "VK_F15");
    map("Numpad4", "VK_F16");
    map("Numpad5", "VK_F17");
    map("Numpad6", "VK_F18");
    map("Numpad7", "VK_F19");
    map("Numpad8", "VK_F20");
    map("Numpad9", "VK_F21");

    // NumLock toggles a second layer
    map("NumLock", "LK_00");

    when("LK_00") {
        // Second layer: more F-keys
        map("Numpad1", "VK_F22");
        map("Numpad2", "VK_F23");
        map("Numpad3", "VK_F24");
    }
when_device_end();

// Main keyboard unaffected
when_device_start("*keyboard*");
    // Normal CapsLock behavior
    map("CapsLock", "MD_00");  // Navigation layer

    when("MD_00") {
        map("H", "VK_Left");
        map("J", "VK_Down");
        map("K", "VK_Up");
        map("L", "VK_Right");
    }
when_device_end();
```

---

## Example Configurations

### Numpad as Stream Deck

Turn a USB numpad into a macro pad for OBS, Discord, or other applications:

```rhai
// =====================================================
// USB NUMPAD AS STREAM DECK
// =====================================================
// This config turns numpad keys into F13-F24 keys,
// which can be assigned as hotkeys in any application.

when_device_start("*numpad*");
    // Row 1: Scene switching (F13-F15)
    map("Numpad7", "VK_F13");  // OBS Scene 1
    map("Numpad8", "VK_F14");  // OBS Scene 2
    map("Numpad9", "VK_F15");  // OBS Scene 3

    // Row 2: Audio controls (F16-F18)
    map("Numpad4", "VK_F16");  // Mute Mic
    map("Numpad5", "VK_F17");  // Mute Desktop
    map("Numpad6", "VK_F18");  // Mute All

    // Row 3: Stream controls (F19-F21)
    map("Numpad1", "VK_F19");  // Start/Stop Stream
    map("Numpad2", "VK_F20");  // Start/Stop Recording
    map("Numpad3", "VK_F21");  // Pause Recording

    // Bottom row: Quick actions (F22-F24)
    map("Numpad0", "VK_F22");  // BRB Screen
    map("NumpadDecimal", "VK_F23");  // Transition
    map("NumpadEnter", "VK_F24");  // Raid Last Viewer
when_device_end();

// Main keyboard unchanged
// (No device block needed - unmapped keys pass through)
```

**OBS Setup:**
1. Go to Settings → Hotkeys
2. Assign F13 to "Switch to Scene 1"
3. Assign F19 to "Start/Stop Streaming"
4. Press numpad keys to trigger actions

### Split Keyboard Setup

Configure two keyboards as left and right halves:

```rhai
// =====================================================
// SPLIT KEYBOARD SETUP
// =====================================================
// Left keyboard provides modifiers
// Right keyboard responds to those modifiers

// Left keyboard (by serial number)
device_start("SERIAL_LEFT");
    // LShift acts as custom modifier for right keyboard
    map("LShift", "MD_00");

    // CapsLock toggles a layer on right keyboard
    map("CapsLock", "LK_00");

    // A+S+D chord = special modifier
    map("A", "MD_01");
    when("MD_01") {
        map("S", "MD_02");
    }
device_end();

// Right keyboard (by serial number)
device_start("SERIAL_RIGHT");
    // Respond to left keyboard's modifiers
    when("MD_00") {
        // Left Shift held: navigation on right hand
        map("J", "VK_Left");
        map("K", "VK_Down");
        map("I", "VK_Up");
        map("L", "VK_Right");
    }

    when("LK_00") {
        // CapsLock lock: number row becomes symbols
        map("U", with_shift("VK_1"));  // !
        map("I", with_shift("VK_2"));  // @
        map("O", with_shift("VK_3"));  // #
        map("P", with_shift("VK_4"));  // $
    }

    when("MD_02") {
        // A+S chord on left: special characters on right
        map("J", with_shift("VK_LeftBracket"));  // {
        map("K", with_shift("VK_RightBracket")); // }
    }
device_end();
```

### Gaming Keyboard with Separate Macros

Isolate gaming macros to a dedicated keyboard:

```rhai
// =====================================================
// GAMING KEYBOARD ISOLATION
// =====================================================
// Gaming keyboard has macros
// Work keyboard is standard

// Gaming keyboard (Razer, Corsair, etc.)
when_device_start("*Razer*");
    // F12 toggles gaming mode
    map("F12", "LK_00");

    when("LK_00") {
        // Gaming mode active
        // WASD → Arrow keys (for games that don't support rebinding)
        map("W", "VK_Up");
        map("A", "VK_Left");
        map("S", "VK_Down");
        map("D", "VK_Right");

        // Number row → function keys (abilities)
        map("1", "VK_F1");
        map("2", "VK_F2");
        map("3", "VK_F3");
        map("4", "VK_F4");
        map("5", "VK_F5");
    }

    when_not("LK_00") {
        // Normal mode: standard Vim navigation
        map("CapsLock", "MD_00");
        when("MD_00") {
            map("H", "VK_Left");
            map("J", "VK_Down");
            map("K", "VK_Up");
            map("L", "VK_Right");
        }
    }
when_device_end();

// Work keyboard (any other)
when_device_start("*");
    map("CapsLock", "VK_Escape");  // Simple remap for work
when_device_end();
```

---

## System Tray Usage

### Linux System Tray

On Linux desktop environments (KDE, GNOME, XFCE), KeyRx shows a system tray icon when running:

- **Icon Location:** Usually in the top-right panel area
- **Icon Appearance:** KeyRx logo (small keyboard icon)
- **Click Behavior:** Opens menu with options

**Supported Desktop Environments:**
- KDE Plasma 5+
- GNOME 3+ (with AppIndicator extension)
- XFCE 4+
- Other freedesktop-compliant panels

### Reload Configuration

To reload the configuration without restarting the daemon:

**From System Tray:**
1. Click the KeyRx tray icon
2. Select "Reload Config"

**From Command Line:**
```bash
# Using systemd (system service)
sudo systemctl reload keyrx

# Using systemd (user service)
systemctl --user reload keyrx

# Using signal (manual daemon)
kill -HUP $(pgrep keyrx_daemon)
```

**Hot Reload Behavior:**
- Current modifier states are preserved
- Device grabs remain active (no interruption)
- New mappings apply immediately
- No need to re-authenticate

### Exit Daemon

To stop the daemon gracefully:

**From System Tray:**
1. Click the KeyRx tray icon
2. Select "Exit"

**From Command Line:**
```bash
# Using systemd
sudo systemctl stop keyrx

# Using signal (manual daemon)
kill $(pgrep keyrx_daemon)

# Or press Ctrl+C in the terminal where daemon is running
```

**Graceful Shutdown:**
- Releases all device grabs
- Restores original keyboard behavior
- Saves any persistent state

---

## Web Interface

### Device List API

The daemon exposes a REST API for querying connected devices:

```bash
# Get device list
curl http://localhost:3000/api/devices
```

**Response:**
```json
{
  "devices": [
    {
      "id": "usb-Logitech_USB_Keyboard-event-kbd",
      "name": "Logitech USB Keyboard",
      "path": "/dev/input/event18",
      "serial": "123456",
      "active": true
    },
    {
      "id": "platform-i8042-serio-0-event-kbd",
      "name": "AT Translated Set 2 keyboard",
      "path": "/dev/input/event3",
      "serial": null,
      "active": true
    }
  ]
}
```

### Real-Time Device Activity

The web UI shows real-time activity for each device:

1. Open `http://localhost:3000` in your browser
2. Navigate to the "Devices" tab
3. See connected devices with activity indicators
4. Green highlight when device sends events

---

## Troubleshooting

### Permission Issues

**Symptom:** `Permission denied when accessing /dev/input/eventX`

**Solutions:**

1. Add user to input group:
   ```bash
   sudo usermod -aG input $USER
   ```

2. Install udev rules:
   ```bash
   sudo cp keyrx_daemon/udev/99-keyrx.rules /etc/udev/rules.d/
   sudo udevadm control --reload-rules && sudo udevadm trigger
   ```

3. Log out and log back in (or reboot)

4. Verify permissions:
   ```bash
   ls -la /dev/input/event*
   # Should show: crw-rw---- 1 root input ...
   ```

See [Linux Setup Guide](./linux-setup.md#permissions-setup) for detailed instructions.

### Device Not Detected

**Symptom:** `keyrx_daemon list-devices` doesn't show your keyboard

**Solutions:**

1. Verify device is connected:
   ```bash
   lsusb  # Check USB devices
   dmesg | tail -20  # Check kernel messages
   ```

2. Check if evdev module is loaded:
   ```bash
   lsmod | grep evdev
   sudo modprobe evdev
   ```

3. Verify input device exists:
   ```bash
   ls /dev/input/event*
   cat /proc/bus/input/devices | grep -A5 "Keyboard"
   ```

4. Check device type (KeyRx only captures keyboards):
   ```bash
   # Look for KEY capability
   cat /sys/class/input/event18/device/capabilities/key
   ```

### Headless Mode

**Symptom:** System tray not working on headless server

**Expected Behavior:** On servers without a display:
- System tray silently fails (warning in logs)
- Daemon continues running normally
- Use CLI or REST API for control

**Logs to Expect:**
```
[WARN] System tray unavailable: No display server found. Use CLI to control daemon.
```

**CLI Alternatives:**
```bash
# Reload configuration
kill -HUP $(pgrep keyrx_daemon)

# Check status via REST
curl http://localhost:3000/api/status

# View logs
journalctl -u keyrx -f
```

### Device Hot-Plug

**Current Limitation:** Hot-plugging (connecting/disconnecting devices while daemon runs) requires a restart.

**Workaround:**
```bash
# Reconnect device, then reload
systemctl --user restart keyrx

# Or manually restart
kill $(pgrep keyrx_daemon)
keyrx_daemon run --config ~/.config/keyrx/config.krx
```

**Future Enhancement:** Automatic device detection without restart is planned for a future release.

### Pattern Not Matching

**Symptom:** `when_device_start("pattern")` doesn't match expected device

**Debug Steps:**

1. List exact device names:
   ```bash
   keyrx_daemon list-devices
   ```

2. Try exact match first:
   ```rhai
   when_device_start("USB Keyboard");  // Exact name from list-devices
   ```

3. Check case sensitivity (patterns are case-sensitive):
   ```rhai
   when_device_start("*Keyboard*");  // Matches "USB Keyboard"
   when_device_start("*keyboard*");  // Matches "AT keyboard"
   ```

4. Use wildcards for flexibility:
   ```rhai
   when_device_start("*");  // Matches all devices (fallback)
   ```

### Keys Not Remapping

**Symptom:** Keys work on one device but not another

**Debug Steps:**

1. Verify device is matched:
   ```bash
   keyrx_daemon validate --config your-config.krx
   ```

   Output should show which devices match which patterns.

2. Check overlapping patterns (first match wins):
   ```rhai
   // Order matters! More specific patterns should come first
   when_device_start("USB Numpad");  // Specific
       map("Numpad1", "VK_F13");
   when_device_end();

   when_device_start("*");  // Fallback
       map("CapsLock", "VK_Escape");
   when_device_end();
   ```

3. Run with debug logging:
   ```bash
   keyrx_daemon run --config your-config.krx --debug
   ```

   Look for lines like:
   ```
   [DEBUG] Event from device "USB Numpad": KeyEvent { keycode: Numpad1, pressed: true }
   [DEBUG] Matched pattern: "USB Numpad", applying mapping...
   ```

---

## Quick Reference

### Device Matching Patterns

| Pattern | Meaning | Example Match |
|---------|---------|---------------|
| `"Name"` | Exact match | "Name" only |
| `"Prefix*"` | Starts with | "Prefix123", "Prefix ABC" |
| `"*Suffix"` | Ends with | "MySuffix", "Some Suffix" |
| `"*Contains*"` | Contains | "HasContainsHere" |
| `"*"` | All devices | Everything |

### Useful Commands

```bash
# List devices
keyrx_daemon list-devices

# Validate config
keyrx_daemon validate --config config.krx

# Run with debug
keyrx_daemon run --config config.krx --debug

# Check logs (systemd)
journalctl -u keyrx -f

# Reload config
kill -HUP $(pgrep keyrx_daemon)

# Get devices via API
curl http://localhost:3000/api/devices
```

### Configuration Template

```rhai
// =====================================================
// MULTI-DEVICE CONFIGURATION TEMPLATE
// =====================================================

// Device 1: Primary keyboard
when_device_start("*keyboard*");
    // Your main keyboard mappings here
    map("CapsLock", "MD_00");  // Navigation layer

    when("MD_00") {
        map("H", "VK_Left");
        map("J", "VK_Down");
        map("K", "VK_Up");
        map("L", "VK_Right");
    }
when_device_end();

// Device 2: Numpad/macro pad
when_device_start("*numpad*");
    // Macro pad mappings
    map("Numpad1", "VK_F13");
    map("Numpad2", "VK_F14");
when_device_end();

// Fallback: Any unmatched device
when_device_start("*");
    // Minimal mappings for unknown devices
    map("CapsLock", "VK_Escape");
when_device_end();
```

---

**See Also:**
- [DSL Manual](./dsl-manual.md) - Complete language reference
- [Linux Setup](./linux-setup.md) - Installation and permissions
- [Windows Setup](./windows-setup.md) - Windows-specific setup
