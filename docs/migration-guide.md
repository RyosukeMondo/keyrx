# Migration Guide: V1 to Revolutionary Mapping

## Overview

This guide covers migrating from KeyRx's original device-type-based profile system (V1) to the new Revolutionary Mapping system (V2).

**Migration is optional but recommended.** The old system is deprecated and will be removed in a future version.

## What Changes?

### Old System (V1)

- **One profile per device type** (VID:PID)
- **All identical devices share config**
- **Profiles stored at** `~/.config/keyrx/devices/{vid}_{pid}.json`
- **Layout defined in profile** (rows, cols_per_row, keymap)
- **No device identity tracking**

### New System (V2)

- **Unlimited profiles** independent of devices
- **Each device tracked individually** by serial number
- **Profiles stored at** `~/.config/keyrx/profiles/{uuid}.json`
- **Device bindings** stored at `~/.config/keyrx/device_bindings.json`
- **Device definitions** describe layouts in `device_definitions/**/*.toml`
- **Mappings use (row, col)** instead of scancodes

## Migration Strategy

### Automatic Migration (Recommended)

The easiest way to migrate is using the automatic migration system:

#### 1. Backup Your Config (Recommended)

```bash
# Manual backup
cp -r ~/.config/keyrx ~/.config/keyrx.backup

# Or let migration create backup
keyrx migrate --from v1 --backup
```

#### 2. Preview Migration

```bash
# Dry run to see what will happen
keyrx migrate --from v1 --dry-run
```

Output:
```
Migration Preview (V1 → V2)
===========================

Found 3 old device profiles:

1. devices/0fd9_0080.json → Elgato Stream Deck MK.2
   Layout: Matrix 3×5
   Mappings: 15 keys
   → Will create profile: "Stream Deck MK.2 Profile"

2. devices/046d_c31c.json → Logitech Keyboard
   Layout: Standard (6 rows)
   Mappings: 87 keys
   → Will create profile: "Logitech Keyboard Profile"

3. devices/1234_5678.json → Custom Macro Pad
   Layout: Matrix 5×5
   Mappings: 25 keys
   → Will create profile: "Custom Macro Pad Profile"

No devices currently connected. Profiles will be created but not auto-assigned.

Backups will be created in: ~/.config/keyrx/devices_backup/
```

#### 3. Run Migration

```bash
keyrx migrate --from v1 --backup
```

Output:
```
Starting migration: V1 → V2
============================

✓ Created backup: ~/.config/keyrx/devices_backup/

Migrating profiles...

✓ devices/0fd9_0080.json
  → Profile created: abc123-def456 ("Stream Deck MK.2 Profile")
  → Layout: Matrix { rows: 3, cols: 5 }
  → Mappings: 15 converted

✓ devices/046d_c31c.json
  → Profile created: def456-abc789 ("Logitech Keyboard Profile")
  → Layout: Standard (6 rows)
  → Mappings: 87 converted

✓ devices/1234_5678.json
  → Profile created: 789abc-123def ("Custom Macro Pad Profile")
  → Layout: Matrix { rows: 5, cols: 5 }
  → Mappings: 25 converted

Migration Summary
=================
Profiles migrated: 3
Profiles failed: 0
Devices auto-assigned: 0 (no matching devices connected)

Backups: ~/.config/keyrx/devices_backup/
New profiles: ~/.config/keyrx/profiles/

Migration complete!
```

#### 4. Verify Profiles

```bash
# List migrated profiles
keyrx profiles list
```

Output:
```
Profiles
========

abc123-def456  Stream Deck MK.2 Profile     Matrix (3×5)    2025-01-15 10:30
def456-abc789  Logitech Keyboard Profile    Standard        2025-01-15 10:30
789abc-123def  Custom Macro Pad Profile     Matrix (5×5)    2025-01-15 10:30
```

#### 5. Connect Devices and Assign Profiles

If devices are connected during migration, profiles are **auto-assigned** based on VID:PID matching.

If devices connect later:

```bash
# List devices
keyrx devices list

# Manually assign profile
keyrx devices assign 0fd9:0080:CL12345678 abc123-def456

# Enable remapping
keyrx devices remap 0fd9:0080:CL12345678 on
```

Or use the UI:
1. Open Devices tab
2. Select device
3. Choose profile from dropdown
4. Toggle "Remap Enabled" to ON

### Manual Migration

If automatic migration fails or you prefer manual control:

#### 1. Read Old Profile

```bash
cat ~/.config/keyrx/devices/0fd9_0080.json
```

```json
{
  "vendor_id": 4057,
  "product_id": 128,
  "layout": {
    "rows": 3,
    "cols_per_row": [5, 5, 5]
  },
  "keymap": {
    "1": "F13",
    "2": "F14",
    "3": "F15"
  }
}
```

#### 2. Create New Profile in UI

1. Open Visual Editor
2. Click "Create New Profile"
3. Name: "Stream Deck MK.2 Profile"
4. Layout: Matrix (3×5)
5. Map keys:
   - Scancode 1 → [0,0] → F13
   - Scancode 2 → [0,1] → F14
   - Scancode 3 → [0,2] → F15

#### 3. Save and Assign

Profile is auto-saved. Assign to device via Devices tab.

## Migration Details

### Layout Type Conversion

| Old Format | New Format |
|------------|------------|
| `rows: 3, cols_per_row: [5, 5, 5]` | `Matrix { rows: 3, cols: 5 }` |
| `rows: 6, cols_per_row: [15, 14, 14, 13, 12, 8]` | `Standard` (irregular rows) |
| Custom split layout | `Split` |

**Rules:**
- If all `cols_per_row` values are equal → `Matrix`
- If irregular → `Standard`
- Split keyboards → `Split` (requires manual definition)

### Mapping Conversion

Old keymap format:
```json
{
  "keymap": {
    "30": "A",
    "31": "S",
    "32": "D"
  }
}
```

New mappings format:
```json
{
  "mappings": {
    "[2,1]": { "Key": "A" },
    "[2,2]": { "Key": "S" },
    "[2,3]": { "Key": "D" }
  }
}
```

**Conversion process:**
1. Load device definition for VID:PID
2. Use `matrix_map` to convert scancode → (row, col)
3. Convert simple key string to `KeyAction::Key`

**If device definition doesn't exist:**
- Migration uses fallback ANSI layout
- You may need to create a device definition manually

### KeyAction Conversion

| Old Format | New Format |
|------------|------------|
| `"A"` | `{ "Key": "A" }` |
| Block (not in old system) | `"Block"` |
| Pass (default) | `"Pass"` |
| Chord (not in old system) | `{ "Chord": ["LeftCtrl", "A"] }` |
| Script (not in old system) | `{ "Script": "script_name" }` |

**Note:** Old system only supported simple key remapping. New system adds Block, Chord, and Script actions.

## Troubleshooting

### Issue: "Device definition not found"

**Cause:** No TOML definition exists for the device's VID:PID

**Solution:**
1. Create device definition in `device_definitions/vendor/device.toml`
2. Or use generic fallback (ANSI keyboard layout)
3. Manually adjust mappings in Visual Editor

### Issue: "Invalid layout dimensions"

**Cause:** Old profile has irregular layout that doesn't match any device definition

**Solution:**
1. Check original `cols_per_row` values
2. Create custom device definition with correct layout
3. Re-run migration

### Issue: "Mappings out of bounds"

**Cause:** Scancode maps to position outside layout bounds

**Solution:**
1. Verify device definition `matrix_map` is correct
2. Check old profile for invalid scancodes
3. Manually fix mappings in Visual Editor

### Issue: "Migration creates duplicate profiles"

**Cause:** Running migration multiple times

**Solution:**
Migration is **idempotent by profile name**. If a profile with the same name exists, it won't be recreated. Delete duplicates manually:

```bash
keyrx profiles list
keyrx profiles delete {duplicate-id}
```

### Issue: "Auto-assignment didn't work"

**Cause:** Device wasn't connected during migration, or serial number changed

**Solution:**
Manually assign profile:

```bash
keyrx devices assign {device-identity} {profile-id}
```

### Issue: "Old profiles still exist after migration"

**Expected behavior:** Old profiles are **not deleted** automatically. Backups are created and originals remain.

**To remove old profiles:**
```bash
# After verifying migration succeeded
rm -rf ~/.config/keyrx/devices/
```

**Keep backups:**
```bash
# Backups are in devices_backup/
ls ~/.config/keyrx/devices_backup/
```

## Rollback

If migration fails or causes issues:

### Restore from Backup

```bash
# Stop KeyRx
killall keyrx

# Remove new system files
rm -rf ~/.config/keyrx/profiles/
rm ~/.config/keyrx/device_bindings.json

# Restore from backup
cp -r ~/.config/keyrx/devices_backup/* ~/.config/keyrx/devices/

# Restart KeyRx
```

### Keep Both Systems

You can keep both old and new profiles temporarily:

```bash
# Old profiles remain in devices/
ls ~/.config/keyrx/devices/

# New profiles are in profiles/
ls ~/.config/keyrx/profiles/
```

**Note:** KeyRx V2 only reads from `profiles/`. The old `devices/` directory is ignored.

## Migration Checklist

- [ ] Backup current config (`cp -r ~/.config/keyrx ~/.config/keyrx.backup`)
- [ ] Run dry-run migration (`keyrx migrate --from v1 --dry-run`)
- [ ] Review migration preview output
- [ ] Check device definitions exist for all devices
- [ ] Run actual migration (`keyrx migrate --from v1 --backup`)
- [ ] Verify profiles created (`keyrx profiles list`)
- [ ] Connect devices and check auto-assignment
- [ ] Manually assign profiles if needed
- [ ] Test remapping with new profiles
- [ ] Verify all mappings work correctly
- [ ] Delete old profiles (optional, after verification)
- [ ] Delete backups (optional, after long-term verification)

## FAQ

**Q: Do I have to migrate?**
A: No, but it's recommended. The old system is deprecated and will be removed in a future version.

**Q: Can I migrate one device at a time?**
A: Yes. Delete or rename profiles in `~/.config/keyrx/devices/` that you don't want to migrate yet.

**Q: Will migration break my current setup?**
A: No. Old profiles remain untouched. Migration creates new profiles in `profiles/`.

**Q: What if I have custom scancodes?**
A: Create a device definition first, then run migration. Or manually create the profile.

**Q: Can I run migration multiple times?**
A: Yes, it's idempotent. Existing profiles with the same name won't be recreated.

**Q: What happens to Rhai scripts in old profiles?**
A: Old system didn't support Rhai in profiles (scripts were global). New system supports per-key script actions.

**Q: Do I lose any functionality?**
A: No. New system is a superset of old functionality. You gain per-device control and portable profiles.

**Q: Can I export/import profiles?**
A: Yes. Profiles are JSON files. Copy `~/.config/keyrx/profiles/{uuid}.json` to another machine.

**Q: How do I migrate on Windows?**
A: Same commands work. Config path is `%APPDATA%\keyrx\` instead of `~/.config/keyrx/`.

## See Also

- [Revolutionary Mapping Guide](./revolutionary-mapping-guide.md)
- [Device Definitions Format](../device_definitions/README.md)
- [Main README](../README.md)
