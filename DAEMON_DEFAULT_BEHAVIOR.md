# Daemon Default Behavior

## Auto-Loading Default Profile

The daemon now automatically creates and loads a default profile when started without explicit config.

## Behavior on Startup

### With `--config` Flag
```bash
keyrx_daemon.exe run --config path/to/config.krx
```
Uses the specified config file directly.

### Without `--config` Flag
```bash
keyrx_daemon.exe run
```

**Automatic Behavior:**

1. **Check for Active Profile**
   - Looks in `%APPDATA%\keyrx\settings.json` for active profile
   - If active profile exists → uses `%APPDATA%\keyrx\profiles\{active}.krx`

2. **No Active Profile Found**
   - Creates `%APPDATA%\keyrx\profiles\default.rhai` with blank template
   - Compiles to `default.krx`
   - Activates "default" as the active profile
   - Logs: `[INFO] Using default profile at: ...`

3. **Fallback on Error**
   - If ProfileManager fails to initialize
   - Falls back to `%APPDATA%\keyrx\default.krx`
   - Shows warning about pass-through mode if file doesn't exist

## User Flow

### First Boot
```
1. User installs daemon
2. User starts daemon (no config specified)
3. Daemon auto-creates default profile (blank template)
4. Daemon loads default.krx (0 mappings - pass-through mode)
5. Web UI shows "default" profile exists
6. User can:
   - Edit default profile in Web UI
   - Import example config via script
   - Create new profiles
   - Activate different profile
```

### Subsequent Boots
```
1. User starts daemon
2. Daemon loads previously activated profile
3. Remapping works immediately with saved config
```

## Benefits

### No Chicken-and-Egg Problem
- ✅ Daemon always has a valid profile to load
- ✅ No "file not found" errors on first boot
- ✅ Web UI always has at least one profile to show
- ✅ User can activate and test immediately

### User-Friendly
- ✅ Works out of the box
- ✅ No command-line config needed for normal usage
- ✅ Sensible defaults
- ✅ Clear upgrade path (edit default → create custom → activate custom)

## Example Logs

### First Boot (No Profiles)
```
[INFO] No active profile found. Creating default profile...
[INFO] Creating default profile with blank template...
[INFO] Using default profile at: C:\Users\...\AppData\Roaming\keyrx\profiles\default.krx
[INFO] Starting keyrx daemon (Windows) with config: ...
[INFO] Loaded 0 key mappings from profile 'default'
```

### Second Boot (Default Active)
```
[INFO] Using active profile: default
[INFO] Starting keyrx daemon (Windows) with config: ...
[INFO] Loaded 0 key mappings from profile 'default'
```

### After User Edits Default Profile
```
[INFO] Using active profile: default
[INFO] Starting keyrx daemon (Windows) with config: ...
[INFO] Loaded 256 key mappings from profile 'default'
```

### After User Activates Custom Profile
```
[INFO] Using active profile: my_layout
[INFO] Starting keyrx daemon (Windows) with config: ...
[INFO] Loaded 512 key mappings from profile 'my_layout'
```

## Web UI Integration

The Web UI automatically:
1. Detects the active profile (green checkmark)
2. Shows all available profiles
3. Allows activation of any profile
4. Compiles and reloads daemon on activation

**No command line needed** - everything through Web UI.

## Manual Override

Users can still specify config explicitly:
```bash
# Override active profile with specific config
keyrx_daemon.exe run --config examples\user_layout.krx

# Use absolute path
keyrx_daemon.exe run --config "C:\path\to\my_config.krx"
```

## Migration from Old Behavior

**Before (v1.0):**
```
# Required explicit config or got "file not found" error
keyrx_daemon.exe run --config default.krx  # Manual specification needed
```

**After (v1.1+):**
```
# Just works - auto-creates and loads default profile
keyrx_daemon.exe run  # No arguments needed
```

## Implementation Details

**Location:** `keyrx_daemon/src/main.rs` lines 180-230

**Key Components:**
1. ProfileManager initialization
2. Active profile detection
3. Default profile creation (if needed)
4. Profile activation
5. Path resolution to .krx file

**Error Handling:**
- ProfileManager initialization failure → fallback to default.krx
- Profile creation failure → warns but continues
- Profile activation failure → warns but continues
- File not found → logs warning about pass-through mode

## Testing

### Test Default Profile Creation
```powershell
# 1. Clean slate
Remove-Item -Recurse -Force "$env:APPDATA\keyrx" -ErrorAction SilentlyContinue

# 2. Start daemon
Start-Process keyrx_daemon.exe -ArgumentList "run" -Verb RunAs

# 3. Verify default profile created
Test-Path "$env:APPDATA\keyrx\profiles\default.rhai"
Test-Path "$env:APPDATA\keyrx\profiles\default.krx"

# 4. Check Web UI shows default profile
Start-Process http://localhost:9867/profiles
```

### Test Active Profile Loading
```powershell
# 1. Create and activate custom profile via Web UI
# 2. Restart daemon
Stop-Process -Name keyrx_daemon
Start-Process keyrx_daemon.exe -ArgumentList "run" -Verb RunAs

# 3. Verify custom profile loaded (check logs or Web UI)
```

## Troubleshooting

### "Loaded 0 key mappings"
**Expected** on first boot with blank default profile.
- **Fix:** Edit default profile in Web UI or import example config

### "Config file not found"
Should NOT happen with new behavior.
- If you see this, ProfileManager failed to initialize
- Check permissions on `%APPDATA%\keyrx`

### "Failed to create default profile"
Check file system permissions:
```powershell
Test-Path "$env:APPDATA\keyrx\profiles" -IsValid
New-Item -ItemType Directory -Force "$env:APPDATA\keyrx\profiles"
```

---

**Summary:** The daemon now auto-creates a default profile on first boot, solving the chicken-and-egg problem where users couldn't activate profiles because no config was loaded, but no config could be loaded because no profile was activated.
