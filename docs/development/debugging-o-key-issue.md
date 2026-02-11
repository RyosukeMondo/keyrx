# Debugging: O Key Output Issue ("o -> otudq-lwa")

## Problem Report

User reports that after installing, key remapping shows:
- ✅ W → A works correctly
- ❌ O → "otudq-lwa" (9 characters output instead of expected single character)

## Analysis

### O Key Appears in Multiple Layers

From `examples/user_layout.rhai`, VK_O is mapped in **7 different places**:

1. **Line 57 (MD_00 layer):** `map("VK_O", "VK_Num8");` → 8
2. **Line 106 (MD_01 layer):** `map("VK_O", "VK_E");` → E
3. **Line 152 (MD_02 layer):** `map("VK_O", "VK_Backslash");` → ]
4. **Line 196 (MD_03 layer):** `map("VK_O", with_shift("VK_LeftBracket"));` → {
5. **Line 333 (Base layer):** `map("VK_O", "VK_T");` → T
6. **Line 364 (MD_10 layer):** `map("VK_O", with_shift("VK_T"));` → Shift+T

### Expected Behavior

1. **Base layer (no modifiers held):** O → T
2. **MD_00 layer (B held):** O → 8
3. **MD_01 layer (V held):** O → E
4. Etc.

**Key blocking should prevent:** Original O from reaching OS (only remapped key should output)

### Possible Causes

#### 1. Config Loading Failure (MOST LIKELY)

The .krx file might not be loading correctly:
- Old .krx format (recompiled in latest build)
- Deserialization error
- File not found

**Check daemon.log for:**
```
✗ Failed to load profile config for key blocking: <error>
```

If config loading fails, **NO keys are blocked** → original key + remapped keys = multiple outputs.

#### 2. Key Blocking Not Working

O's scan code (0x18) might not be:
- Extracted from config correctly
- Converted to scan code correctly
- Blocked by the hook

**Check daemon.log for:**
```
✓ Configured key blocking: <count> keys blocked
```

Expected count: ~50-100 keys depending on profile.

#### 3. Multiple Event Processing

The daemon might be processing the same key event multiple times:
- Event duplication in Raw Input
- Feedback loop
- Multiple device IDs

#### 4. Dvorak Layout Interference

User has Dvorak OS layout, so physical O → outputs "T" at OS level.

**If O is not blocked:**
1. User presses physical O
2. OS processes O → outputs "T" (Dvorak)
3. Daemon also sees O → outputs "T" (base layer mapping)
4. Result: "TT" or similar

But "otudq-lwa" (9 chars) suggests more complex issue.

## Diagnostic Steps

### 1. Check Daemon Log

**Location:** `%APPDATA%\keyrx\daemon.log` or `target/release/daemon.log`

**Look for:**
```
Configuring key blocking for profile: default
✓ Loaded profile config: X devices, Y total mappings
✓ Configured key blocking: Z keys blocked
```

**Expected values:**
- Devices: 1 (wildcard "*")
- Mappings: ~200-300 (all base + conditional mappings)
- Keys blocked: ~50-100 (unique source keys)

**If you see:**
```
✗ Failed to load profile config for key blocking: <error>
```
→ Config loading is failing!

### 2. Check Key Extraction Count

The log should show:
```
✓ Configured key blocking: 67 keys blocked
```

If the count is **0 or very low**, the key extraction is not working.

### 3. Test with Single Mapping

Create a minimal test profile:
```rhai
device_start("*");
map("VK_O", "VK_P");  // O → P
device_end();
```

Compile and activate:
```bash
keyrx_compiler compile test.rhai -o test.krx
# Activate via Web UI
```

**Expected:** O → P (single character)

**If still wrong:** Key blocking mechanism has fundamental issue.

### 4. Check Profile Activation

In Web UI (http://localhost:9867):
1. Check "default" profile is activated (green indicator)
2. Check profile was compiled recently (not old .krx)

### 5. Verify Scan Code Conversion

The daemon log (with RUST_LOG=debug) should show:
```
Blocking KeyCode::O (scan code: 0x0018)
```

If missing, scan code conversion failed.

## Fixed in Latest Build

### 1. Recompiled user_layout.krx
- Old .krx was from January 15
- Recompiled with current compiler (fresh build)
- SHA256: `64f7e6a57091f3655d70af717557bbd2162efd0404264ef931007f08e3c5765b`

### 2. Enhanced Logging
Added detailed logging to profile_service.rs:
```rust
log::info!("✓ Loaded profile config: {} devices, {} total mappings", ...)
log::info!("✓ Key blocking configured successfully");
```

Or on error:
```rust
log::error!("✗ Failed to load profile config for key blocking: {}", e);
log::error!("✗ Failed to configure key blocking: {}", e);
```

### 3. Verification

After installing the latest build, check daemon.log for:

**Success path:**
```
Configuring key blocking for profile: default
✓ Loaded profile config: 1 devices, 218 total mappings
✓ Configured key blocking: 67 keys blocked
```

**Failure path:**
```
Configuring key blocking for profile: default
✗ Failed to load profile config for key blocking: <reason>
```

## Installer Location

**New build:** `target\windows-installer\keyrx_0.1.1.0_x64_setup.exe` (8.91 MB)

**Includes:**
- Recompiled user_layout.krx (fresh)
- Enhanced logging for diagnostics
- Same key blocking logic (no changes needed - it was already generic)

## Next Steps

1. **Install new build** (Right-click → Run as administrator)
2. **Activate default profile** (Web UI)
3. **Check daemon.log** for the ✓/✗ messages above
4. **Test O key** in Notepad
5. **Report back** with:
   - What O outputs now
   - Relevant log messages
   - Key blocked count

## Expected Root Cause

Most likely: **Config loading was failing** due to old .krx file or deserialization issue.

**Evidence:** W → A works, but O produces garbage. This suggests:
- W is being blocked (config partially working?)
- O is not being blocked (extraction issue or config mismatch)

**With fresh .krx and better logging**, we should see exactly what's happening.
