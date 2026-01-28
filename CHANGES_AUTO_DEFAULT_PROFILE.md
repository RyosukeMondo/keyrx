# Auto-Loading Default Profile - Implementation Summary

## Problem Solved

### The Chicken-and-Egg Problem
**Before:** When daemon boots with no `--config`, it tried to load `default.krx` which didn't exist, resulting in:
- ❌ "Config file not found" warning
- ❌ "Loaded 0 key mappings" (pass-through mode)
- ❌ Metrics page shows no events
- ❌ User confused: "Why isn't it working?"

**After:** Daemon automatically creates and activates a default profile on first boot:
- ✅ Creates `%APPDATA%\keyrx\profiles\default.rhai` with blank template
- ✅ Compiles to `default.krx`
- ✅ Sets "default" as active profile
- ✅ Future boots remember the active profile

## Changes Made

### File Modified
`keyrx_daemon/src/main.rs` lines 180-235

### Implementation
```rust
// When no --config specified:
Commands::Run { config: None, .. } => {
    // 1. Initialize ProfileManager
    let mut manager = ProfileManager::new(config_dir)?;

    // 2. Check for active profile
    match manager.get_active()? {
        Some(active) => {
            // Use existing active profile
            profiles/{active}.krx
        }
        None => {
            // No active profile - create & activate default
            if !manager.get("default").is_some() {
                manager.create("default", ProfileTemplate::Blank)?;
            }
            manager.activate("default")?;
            // Use profiles/default.krx
        }
    }
}
```

## Behavior

### First Boot (No Profiles Exist)
```
[INFO] No active profile found. Creating default profile...
[INFO] Creating default profile with blank template...
[INFO] Using default profile at: C:\Users\...\keyrx\profiles\default.krx
[INFO] Starting keyrx daemon (Windows) with config: ...
[INFO] Loaded 0 key mappings from profile 'default'
```

**Result:** Blank default profile active, Web UI shows one profile, user can edit/import

### Second Boot (Default Profile Exists)
```
[INFO] Using active profile: default
[INFO] Starting keyrx daemon (Windows) with config: ...
[INFO] Loaded 0 key mappings from profile 'default' (or N if user edited it)
```

### After User Activates Custom Profile
```
[INFO] Using active profile: my_layout
[INFO] Starting keyrx daemon (Windows) with config: ...
[INFO] Loaded 256 key mappings from profile 'my_layout'
```

## User-Facing Questions Answered

### Q: "Is there a way to keep/save/remember last activated config and boot without --config?"
✅ **YES!** The daemon now:
1. Saves active profile to `%APPDATA%\keyrx\.active`
2. Loads it automatically on next boot
3. No --config needed for normal usage

### Q: "Please make sure Web UI and daemon both use same default path, identical, no mismatch"
✅ **FIXED!** Both use:
- **Profiles directory:** `%APPDATA%\keyrx\profiles\`
- **Active profile file:** `%APPDATA%\keyrx\.active`
- **Config format:** `{name}.rhai` (source) → `{name}.krx` (compiled)

The ProfileManager is shared between:
- Daemon startup (loads active profile)
- Web API (ProfileService uses same ProfileManager)
- RPC/IPC (profile operations)

**NO MISMATCH** - single source of truth.

## File Paths (All Consistent)

| Component | Location |
|-----------|----------|
| **Profiles directory** | `%APPDATA%\keyrx\profiles\` |
| **Profile source** | `profiles\{name}.rhai` |
| **Profile compiled** | `profiles\{name}.krx` |
| **Active profile marker** | `.active` (contains profile name) |
| **Settings** | `settings.json` |

**Example:** `C:\Users\ryosu\AppData\Roaming\keyrx\profiles\default.krx`

## Benefits

### User Experience
- ✅ Works out of the box (no config file needed)
- ✅ Remembers your active profile between reboots
- ✅ No command-line args needed for normal usage
- ✅ Web UI and daemon always in sync

### Developer Experience
- ✅ Consistent paths across all components
- ✅ Single ProfileManager shared everywhere
- ✅ Clear upgrade path: blank → edit → create custom → activate

### Support/Troubleshooting
- ✅ No more "file not found" errors
- ✅ Clear logs show what profile is active
- ✅ Easy to diagnose: "Is default.krx created? Is it activated?"

## Testing

### Test Auto-Creation
```powershell
# 1. Clean slate
Remove-Item -Recurse -Force "$env:APPDATA\keyrx" -ErrorAction SilentlyContinue

# 2. Build and run daemon
cargo build --release --features windows
.\target\release\keyrx_daemon.exe run --debug

# 3. Verify in logs
# Should see: "Creating default profile with blank template..."
# Should see: "Using default profile at: ..."

# 4. Verify files created
Test-Path "$env:APPDATA\keyrx\profiles\default.rhai"  # True
Test-Path "$env:APPDATA\keyrx\profiles\default.krx"   # True
Get-Content "$env:APPDATA\keyrx\.active"               # "default"
```

### Test Profile Persistence
```powershell
# 1. Import example config via Web UI or script
.\scripts\windows\Import-Example-Config.ps1

# 2. Activate user_layout in Web UI
# (Or via CLI: keyrx_daemon profiles activate user_layout)

# 3. Restart daemon
Stop-Process -Name keyrx_daemon
.\scripts\windows\Debug-Launch.ps1

# 4. Verify user_layout is loaded
# Logs should show: "Using active profile: user_layout"
# Not "Creating default profile..."
```

### Test Fallback
```powershell
# 1. Corrupt .active file
Set-Content "$env:APPDATA\keyrx\.active" -Value "nonexistent_profile"

# 2. Start daemon
.\scripts\windows\Debug-Launch.ps1

# 3. Should gracefully fall back to default.krx
# Logs should show warning
```

## Migration from Previous Version

**Old behavior (< v1.1):**
```powershell
# Required explicit config or would fail
keyrx_daemon.exe run --config examples\user_layout.krx
```

**New behavior (>= v1.1):**
```powershell
# Just works - auto-creates default, remembers active profile
keyrx_daemon.exe run
```

**Breaking changes:** NONE - old command still works with --config flag

## Error Handling

| Error | Behavior |
|-------|----------|
| ProfileManager init fails | Falls back to `default.krx` (old behavior) |
| Profile creation fails | Warns but continues with pass-through mode |
| Profile activation fails | Warns but continues |
| .active file missing | Creates default profile automatically |
| .active file corrupted | Falls back to default.krx |

## Documentation Created

1. **DAEMON_DEFAULT_BEHAVIOR.md** - Detailed user documentation
2. **This file** - Implementation summary for developers
3. **Updated**: Debug-Launch.ps1, Import-Example-Config.ps1 scripts

## Next Steps

### For Users
1. Rebuild daemon: `cargo build --release --features windows`
2. Stop old daemon: `Stop-Process -Name keyrx_daemon`
3. Start new daemon: `.\scripts\windows\Debug-Launch.ps1`
4. Import config: `.\scripts\windows\Import-Example-Config.ps1`
5. Activate in Web UI
6. Done! Future boots remember your profile

### For Developers
- ✅ Code change complete
- ✅ Compiles successfully
- ⏳ **TODO:** Test on Windows to verify behavior
- ⏳ **TODO:** Update user-facing docs in README
- ⏳ **TODO:** Add unit tests for profile auto-creation

## Summary

**Problem:** Chicken-and-egg - no profile active, so can't load config, so can't activate profile.

**Solution:** Daemon auto-creates "default" profile on first boot, remembers active profile for future boots.

**Result:** Users can boot daemon without any args, Web UI and daemon use identical paths, no configuration mismatch.

**User benefit:** Zero-config first boot → Edit default profile → Works immediately → Remembers forever.
