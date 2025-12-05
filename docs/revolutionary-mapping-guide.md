# Revolutionary Mapping Guide

## Overview

Revolutionary Mapping is KeyRx's advanced input management system that decouples physical devices from logical profiles, transforming KeyRx from a simple key remapper into a professional-grade Input Management System.

### What is Revolutionary Mapping?

Revolutionary Mapping solves the fundamental scalability problem of managing multiple identical devices. Instead of binding configurations to device types (e.g., "all Stream Decks share the same config"), Revolutionary Mapping:

- **Uniquely identifies each device** using serial numbers
- **Separates profiles from devices** so configurations are portable
- **Enables per-device control** for toggling remapping independently
- **Supports any layout** including macro pads, split keyboards, and button boxes

### Why "Revolutionary"?

This is a paradigm shift from traditional input remapping:

**Traditional approach:**
- One config per device type (VID:PID)
- All identical devices share the same behavior
- Changing behavior requires editing the config and reloading

**Revolutionary approach:**
- Profiles are independent of devices
- Each device gets its own serial-based identity
- Swap profiles instantly without editing files
- Manage multiple identical devices independently

## Key Concepts

### Device Identity

Every connected input device has a unique identity comprising:
- **Vendor ID (VID)**: USB vendor identifier (e.g., `0x0fd9` for Elgato)
- **Product ID (PID)**: USB product identifier (e.g., `0x0080` for Stream Deck MK.2)
- **Serial Number**: Hardware serial or synthetic identifier

Device identities are represented as `VID:PID:Serial` (e.g., `0fd9:0080:CL12345678`).

#### Serial Number Extraction

**Windows:**
- Attempts to read the USB iSerial descriptor via HidD_GetSerialNumberString
- Falls back to InstanceID from the device path if iSerial is unavailable
- Port-bound devices (no hardware serial) trigger a warning

**Linux:**
- Uses EVIOCGUNIQ ioctl to query the evdev device
- Falls back to udev properties (ID_SERIAL, ID_SERIAL_SHORT)
- Generates synthetic serial from USB port topology hash if needed

### Device Registry

The **Device Registry** manages runtime state for all connected devices:
- **Device State**: Active, Passthrough, or Failed
- **Remap Toggle**: Per-device enable/disable flag
- **Profile Assignment**: Which profile (if any) is assigned
- **Custom Labels**: User-friendly device names

The registry is **volatile** - it only tracks currently connected devices.

### Profile Registry

The **Profile Registry** manages persistent profile configurations stored in `~/.config/keyrx/profiles/`:
- **Profile**: A named configuration with layout type and key mappings
- **Layout Type**: Matrix, Standard, or Split
- **Mappings**: Physical position (row, col) → KeyAction
- **Portability**: Profiles can be assigned to any compatible device

Profiles are stored as JSON files with UUIDs as filenames (e.g., `abc123-def456.json`).

### Device Bindings

**Device Bindings** persist the relationship between devices and profiles across sessions:
- Stored in `~/.config/keyrx/device_bindings.json`
- Maps device identity → profile ID and custom label
- Automatically restored when devices reconnect

### Device Definitions

**Device Definitions** describe the physical layout of input devices:
- Stored as TOML files in `device_definitions/`
- Map scancodes to (row, col) positions
- Enable visual editor to render accurate layouts
- Indexed by VID:PID for fast lookup

See [device_definitions/README.md](../device_definitions/README.md) for detailed format specification.

## Workflows

### 1. Basic Setup: Single Device

**Scenario:** You have one Stream Deck and want to create a profile for it.

#### Step 1: Connect the device

```bash
# List connected devices
keyrx devices
```

Output:
```
Device: Elgato Stream Deck MK.2
  Identity: 0fd9:0080:CL12345678
  State: Passthrough (remap disabled)
  Profile: None
```

#### Step 2: Create a profile

Open the KeyRx UI and navigate to the Visual Editor:

1. Click "Create New Profile"
2. Name it (e.g., "OBS Controls")
3. Select layout type "Matrix (3×5)" for Stream Deck MK.2
4. Map buttons to actions:
   - Button [0,0] → F13 (scene 1)
   - Button [0,1] → F14 (scene 2)
   - Button [1,0] → F15 (start recording)
   - etc.

The profile is automatically saved to `~/.config/keyrx/profiles/{uuid}.json`.

#### Step 3: Assign the profile to the device

In the Devices tab:

1. Select your Stream Deck from the list
2. Click the profile dropdown
3. Select "OBS Controls"
4. Toggle "Remap Enabled" to ON

The device is now active with your profile.

### 2. Advanced Setup: Multiple Identical Devices

**Scenario:** You have two Stream Deck MK.2 units and want different profiles for each.

#### Step 1: Label your devices

When both devices are connected:

```bash
keyrx devices
```

Output:
```
Device: Elgato Stream Deck MK.2
  Identity: 0fd9:0080:CL12345678
  State: Passthrough
  Profile: None

Device: Elgato Stream Deck MK.2
  Identity: 0fd9:0080:CL87654321
  State: Passthrough
  Profile: None
```

In the UI Devices tab:

1. Click "Edit Label" on the first device
2. Name it "OBS Deck"
3. Click "Edit Label" on the second device
4. Name it "Photoshop Deck"

Labels are stored in `device_bindings.json` and persist across sessions.

#### Step 2: Create two profiles

Create two profiles with different mappings:

**Profile 1: "OBS Controls"** (Matrix 3×5)
- F13-F24 for scene switching and recording controls

**Profile 2: "Photoshop Shortcuts"** (Matrix 3×5)
- Ctrl+Z, Ctrl+Y, Ctrl+S, layer shortcuts, etc.

#### Step 3: Assign profiles

In the Devices tab:

1. "OBS Deck" → assign "OBS Controls" → enable remap
2. "Photoshop Deck" → assign "Photoshop Shortcuts" → enable remap

Now each device operates independently with its own profile.

### 3. Profile Swapping

**Scenario:** You want to repurpose a device for a different task.

#### Swap profiles on the fly

In the Devices tab:

1. Select the device
2. Change the profile dropdown to a different profile
3. The device behavior changes immediately (< 100ms)

**Use case:** Switch your "Photoshop Deck" to "Blender Controls" when you open Blender.

### 4. Temporary Passthrough

**Scenario:** You want to temporarily disable remapping for debugging.

#### Toggle remap per device

In the Devices tab:

1. Select the device
2. Toggle "Remap Enabled" to OFF
3. The device passes through all inputs unchanged

This is useful for:
- Testing whether an issue is caused by KeyRx
- Using a device with its factory defaults temporarily
- Debugging profile mappings

### 5. Migrating from Old System

**Scenario:** You're upgrading from the old device-type-based system.

#### Automatic migration

When you first run KeyRx with the new system:

1. KeyRx detects old profiles in `~/.config/keyrx/devices/{vid}_{pid}.json`
2. A migration prompt appears: "Migrate old profiles to the new system?"
3. Click "Migrate"
4. Old profiles are converted to the new format:
   - New UUID assigned
   - Layout type inferred from rows/cols
   - Mappings converted from keymap → (row, col) mappings
5. Backups created in `~/.config/keyrx/devices_backup/`
6. If a connected device matches a migrated profile's VID:PID, the profile is auto-assigned

You can also run migration manually:

```bash
keyrx migrate --from v1 --backup
```

## CLI Reference

### Device Management

```bash
# List all connected devices
keyrx devices list

# Show device details
keyrx devices show 0fd9:0080:CL12345678

# Set custom label
keyrx devices label 0fd9:0080:CL12345678 "My Stream Deck"

# Toggle remap
keyrx devices remap 0fd9:0080:CL12345678 on
keyrx devices remap 0fd9:0080:CL12345678 off

# Assign profile
keyrx devices assign 0fd9:0080:CL12345678 abc123-def456

# Unassign profile
keyrx devices unassign 0fd9:0080:CL12345678
```

### Profile Management

```bash
# List all profiles
keyrx profiles list

# Show profile details
keyrx profiles show abc123-def456

# Create new profile
keyrx profiles create --name "My Profile" --layout matrix --rows 3 --cols 5

# Delete profile
keyrx profiles delete abc123-def456

# Find compatible profiles for a device
keyrx profiles compatible 0fd9:0080:CL12345678
```

### Device Definitions

```bash
# List all loaded definitions
keyrx definitions list

# Show definition for VID:PID
keyrx definitions show 0fd9:0080

# Validate a definition file
keyrx definitions validate device_definitions/custom/my-device.toml

# Reload definitions from disk
keyrx definitions reload
```

### Migration

```bash
# Migrate old profiles to new system
keyrx migrate --from v1 --backup

# Dry run (preview without changes)
keyrx migrate --from v1 --dry-run

# Show migration report
keyrx migrate --report
```

## UI Reference

### Devices Tab

The **Devices Tab** is the default landing page when you open KeyRx.

**Layout:**
- List of connected devices (one card per device)
- Each card shows:
  - User label (or fallback to device name)
  - VID:PID:Serial
  - Profile selector dropdown
  - Remap toggle switch (ON/OFF)
  - "Edit Label" button
  - "Manage Profiles" button

**Empty State:**
- "No devices connected. Connect a device to get started."

**Actions:**
- **Edit Label:** Opens dialog to set custom device name
- **Profile Dropdown:** Select which profile to assign (filtered by layout compatibility)
- **Remap Toggle:** Enable/disable remapping for this device
- **Manage Profiles:** Navigate to profile management for this device

### Visual Editor

The **Visual Editor** provides a graphical interface for creating and editing profiles.

**Features:**
- **Profile Selector:** Dropdown at top to select which profile to edit
- **Dynamic Layout Rendering:** Shows device layout (Matrix, Standard, or Split)
- **Soft Keyboard Palette:** Right panel with all available output keys
- **Mapping Workflow:**
  1. Click a physical position (e.g., button [0,0])
  2. Position highlights and prompts: "Now select an output key"
  3. Click a key from the palette (e.g., F13)
  4. Mapping created, button shows "F13" label
- **Auto-Save:** Profile changes are saved automatically

**Layout Types:**
- **Matrix:** Uniform grid (e.g., 3×5 for Stream Deck)
- **Standard:** Traditional keyboard with irregular rows
- **Split:** Split keyboard with two halves

### Profiles Management

**List View:**
- Shows all profiles with name, layout type, timestamps
- Click to edit in Visual Editor
- Delete button (warns if in use by a device)

**Create New:**
- Name input
- Layout type selector
- Dimensions (rows/cols)
- Opens Visual Editor for mapping

## Performance

Revolutionary Mapping is designed for sub-millisecond latency:

| Pipeline Stage | Latency Target | Description |
|----------------|----------------|-------------|
| Device Resolution | < 50μs (p99) | OS handle → DeviceIdentity |
| Registry Lookup | < 10μs | Load device state (cached) |
| Profile Lookup | < 100μs (p99) | Load profile (cached) |
| Coordinate Translation | < 20μs (p99) | Scancode → (row, col) |
| Action Lookup | < 10μs | (row, col) → KeyAction |
| **Total Pipeline** | **< 1ms (p99)** | Input event → output injection |

**Passthrough Mode:** When remapping is disabled, overhead is < 10μs.

## File Locations

| Purpose | Path | Format |
|---------|------|--------|
| Profiles | `~/.config/keyrx/profiles/{uuid}.json` | JSON |
| Device Bindings | `~/.config/keyrx/device_bindings.json` | JSON |
| Device Definitions | `device_definitions/**/*.toml` | TOML |
| Old Profiles (V1) | `~/.config/keyrx/devices/{vid}_{pid}.json` | JSON |
| Migration Backups | `~/.config/keyrx/devices_backup/` | JSON |

## Troubleshooting

### Device Not Detected

**Symptom:** `keyrx devices list` shows no devices

**Solutions:**
- Ensure device is connected
- On Linux: Check permissions (`sudo usermod -aG input $USER`)
- On Windows: Check for driver conflicts
- Run `keyrx doctor` for diagnostics

### Profile Assignment Fails

**Symptom:** "Layout incompatible" error when assigning profile

**Cause:** Profile layout doesn't match device layout

**Solution:**
- Check profile layout type: `keyrx profiles show {profile_id}`
- Check device layout: `keyrx devices show {device_identity}`
- Create a new profile with the correct layout type

### Serial Number Warning

**Symptom:** "Device configuration is port-dependent" warning

**Cause:** Device lacks hardware serial, using synthetic ID based on USB port

**Impact:**
- Moving device to different USB port creates new identity
- Profile binding won't persist across port changes

**Solution:**
- Keep device in the same USB port
- Or manually reassign profile after moving
- Consider devices with hardware serial numbers for multi-device setups

### Migration Issues

**Symptom:** Old profiles not migrated correctly

**Solutions:**
- Check migration report: `keyrx migrate --report`
- Verify old profiles exist in `~/.config/keyrx/devices/`
- Check backups in `~/.config/keyrx/devices_backup/`
- Run migration again (idempotent, safe to retry)
- Manually create profiles from old configs if needed

### Profile Not Loading

**Symptom:** Device shows "Profile: None" after restart

**Cause:** Profile deleted or `device_bindings.json` corrupted

**Solutions:**
- Check bindings file exists: `~/.config/keyrx/device_bindings.json`
- Verify profile still exists: `keyrx profiles list`
- Reassign profile: `keyrx devices assign {identity} {profile_id}`
- Check logs for corruption errors

## Best Practices

### Device Labeling

- **Use descriptive labels** that indicate purpose (e.g., "OBS Deck", "Gaming Keypad")
- **Label immediately** when first connecting a device
- **Be consistent** with naming conventions across your setup

### Profile Organization

- **Name profiles by purpose**, not by device (e.g., "Video Editing Controls" not "Stream Deck Config")
- **Keep profiles focused** - one profile per task/application
- **Use layout types correctly** - don't create Matrix profile for keyboard layouts

### Multi-Device Setups

- **Use hardware serials** when possible (check with `keyrx devices show`)
- **Keep USB ports consistent** for devices with synthetic serials
- **Test profile swapping** to ensure smooth transitions
- **Document your setup** - which device uses which profile

### Performance Optimization

- **Avoid unnecessary profile changes** - each change triggers cache invalidation
- **Use passthrough mode** for debugging rather than unassigning profiles
- **Monitor latency** with `keyrx bench` to verify performance

### Migration Strategy

- **Backup first** - migration creates backups, but manual backup is safer
- **Test in stages** - migrate one device at a time if concerned
- **Verify bindings** - check that auto-assigned profiles work correctly
- **Keep old system** - don't delete backups until fully validated

## Advanced Topics

### Custom Device Definitions

To add support for a new device:

1. Determine VID:PID (`lsusb` on Linux, Device Manager on Windows)
2. Map physical layout (rows × columns)
3. Identify scancodes (`evtest` on Linux, Raw Input on Windows)
4. Create TOML definition in `device_definitions/vendor/device.toml`
5. Validate: `keyrx definitions validate path/to/device.toml`
6. Reload: `keyrx definitions reload`

See [device_definitions/README.md](../device_definitions/README.md) for format specification.

### Profile JSON Schema

Profiles are stored as JSON with this structure:

```json
{
  "id": "abc123-def456-...",
  "name": "My Profile",
  "layout_type": {
    "Matrix": {
      "rows": 3,
      "cols": 5
    }
  },
  "mappings": {
    "[0,0]": { "Key": "F13" },
    "[0,1]": { "Key": "F14" },
    "[1,0]": { "Chord": ["LeftCtrl", "Z"] }
  },
  "created_at": "2025-01-15T10:30:00Z",
  "modified_at": "2025-01-16T14:20:00Z"
}
```

**KeyAction Types:**
- `"Key": "KeyCode"` - Single key output
- `"Chord": ["Key1", "Key2"]` - Multiple keys pressed together
- `"Script": "script_name"` - Rhai script execution
- `"Block"` - Block the input (no output)
- `"Pass"` - Pass through unchanged

### Device Bindings Schema

`device_bindings.json` structure:

```json
{
  "bindings": [
    {
      "device_identity": {
        "vendor_id": 4057,
        "product_id": 128,
        "serial_number": "CL12345678"
      },
      "profile_id": "abc123-def456-...",
      "user_label": "OBS Deck",
      "bound_at": "2025-01-15T10:30:00Z"
    }
  ]
}
```

## FAQ

**Q: Can I use the same profile on multiple devices?**
A: Yes! Profiles are portable. Assign the same profile to multiple devices if they have compatible layouts.

**Q: What happens if I delete a profile that's assigned to a device?**
A: The device transitions to Passthrough state and logs a warning. You'll need to assign a new profile.

**Q: Do profiles support Rhai scripts?**
A: Yes! Use the `"Script": "script_name"` KeyAction to execute Rhai scripts on key press.

**Q: Can I edit profiles while KeyRx is running?**
A: Yes! Profile changes take effect immediately (< 100ms). The engine caches profiles and invalidates on updates.

**Q: How do I reset everything to defaults?**
A: Delete `~/.config/keyrx/profiles/` and `~/.config/keyrx/device_bindings.json`. All devices will start in Passthrough mode.

**Q: Does this work with the old device-based system?**
A: No, Revolutionary Mapping replaces the old system. Use migration to convert old profiles.

**Q: What's the difference between "Block" and "Passthrough"?**
A: "Block" is a mapping (key → nothing). "Passthrough" is a device state (all keys pass through unchanged).

**Q: Can I create profiles without a device connected?**
A: Yes! Profiles are device-independent. Create them anytime and assign later when devices connect.

## See Also

- [Device Definitions Format](../device_definitions/README.md)
- [Architecture Overview](./ARCHITECTURE.md)
- [FFI Layer Documentation](./ffi-architecture.md)
- [Main README](../README.md)
