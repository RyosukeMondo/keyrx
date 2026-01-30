# SSOT Port Configuration - Complete Fix

## Problem Identified ✅

You're absolutely right! The port number was **NOT** SSOT:

1. **Daemon**: Port 9868 (settings.json)
2. **UI Dev**: Port 9867 (.env.development - hardcoded)
3. **UI Prod**: Uses window.location.origin (correct, but UI was built in dev mode)
4. **Vite Proxy**: Port 9867 (vite.config.ts - hardcoded)

**Result**: When you open UI from system tray, daemon serves UI from port 9868, but UI makes API calls to port 9867 ❌

## SSOT Solution Implemented ✅

### Single Source of Truth

**SSOT Location**: `keyrx_daemon/src/services/settings_service.rs`

```rust
pub const DEFAULT_PORT: u16 = 9867;
```

All other configs are **generated from this** during build.

### Automated Sync

**Script**: `scripts/sync-port-config.ts`

Extracts `DEFAULT_PORT` from Rust and updates:
1. ✅ `keyrx_ui/.env.development` → `VITE_API_URL=http://localhost:9867`
2. ✅ `keyrx_ui/.env.example` → `VITE_API_URL=http://localhost:9867`
3. ✅ `keyrx_ui/vite.config.ts` → `target: 'http://localhost:9867'`

**Trigger**: Automatically runs before every build via `npm run prebuild`

### Production Build Enforcement

**Updated**: `REBUILD_SSOT.bat`

Now enforces:
1. ✅ Sync port config from Rust SSOT
2. ✅ Build UI in **PRODUCTION mode** (not dev mode)
3. ✅ Production mode uses `window.location.origin` dynamically

## How It Works

### Build Flow (SSOT Enforced)

```
[User runs REBUILD_SSOT.bat]
    ↓
[1] Extract DEFAULT_PORT from settings_service.rs (9867)
    ↓
[2] Update .env.development → port 9867
[3] Update .env.example → port 9867
[4] Update vite.config.ts proxy → port 9867
    ↓
[5] Build UI in PRODUCTION mode
    ↓
    env.ts detects PROD mode
    Uses window.location.origin (not hardcoded!)
    ↓
[6] Embed UI in daemon binary
    ↓
[7] Install daemon

[User clicks System Tray → Open UI]
    ↓
Daemon serves UI from http://localhost:9867
    ↓
UI loads at http://localhost:9867
    ↓
UI JavaScript: window.location.origin = "http://localhost:9867"
    ↓
API calls go to http://localhost:9867 ✅
```

### Runtime Config

At runtime, daemon can still use `settings.json` to override the default port:

```json
{
  "port": 9868
}
```

But then you must rebuild the UI (or delete settings.json to use default 9867).

## Verification Steps

### Step 1: Check SSOT Source

```bash
grep "DEFAULT_PORT" keyrx_daemon/src/services/settings_service.rs
```

Expected: `pub const DEFAULT_PORT: u16 = 9867;`

### Step 2: Run Sync Script

```bash
cd keyrx_ui
npm run sync-port
```

Expected output:
```
✓ Extracted DEFAULT_PORT from Rust: 9867
✓ Updated vite.config.ts proxy: http://localhost:9867
✓ Updated .env.development: http://localhost:9867
✓ Updated .env.example: http://localhost:9867
✓ SSOT enforced successfully!
```

### Step 3: Verify Generated Configs

```bash
# Check .env.development
cat keyrx_ui/.env.development | grep VITE_API_URL
# Expected: VITE_API_URL=http://localhost:9867

# Check vite.config.ts
grep "target: 'http://localhost" keyrx_ui/vite.config.ts
# Expected: target: 'http://localhost:9867'

# Check .env.production (should be empty for dynamic origin)
cat keyrx_ui/.env.production | grep VITE_API_URL
# Expected: VITE_API_URL=
```

### Step 4: Rebuild Everything

```bash
.\REBUILD_SSOT.bat
```

New output shows:
```
[4/8] Syncing port configuration (SSOT)...
  ✓ SSOT enforced successfully!
[5/8] Rebuilding UI (PRODUCTION MODE)...
  UI built successfully
```

### Step 5: Test System Tray Launch

1. Kill daemon: `taskkill /F /IM keyrx_daemon.exe`
2. Delete custom settings (use default):
   ```powershell
   Remove-Item "C:\Users\$env:USERNAME\AppData\Roaming\keyrx\settings.json" -ErrorAction SilentlyContinue
   ```
3. Start daemon from tray icon or:
   ```powershell
   Start-Process "C:\Program Files\KeyRx\bin\keyrx_daemon.exe" -ArgumentList "run"
   ```
4. Right-click tray icon → **Open UI**
5. Should open: `http://localhost:9867` ✅
6. Press F12 → Network tab
7. API calls should go to: `http://localhost:9867/api/*` ✅

## How to Change Port (With SSOT)

### Option 1: Change Default (Permanent)

1. Edit **ONE FILE**: `keyrx_daemon/src/services/settings_service.rs:10`
   ```rust
   pub const DEFAULT_PORT: u16 = 8080;  // Change this
   ```

2. Run SSOT rebuild:
   ```bash
   .\REBUILD_SSOT.bat
   ```

3. Everything automatically syncs to port 8080 ✅

### Option 2: Override at Runtime (Temporary)

1. Edit `settings.json`:
   ```json
   {"port": 8080}
   ```

2. Restart daemon

**Note**: This only affects the daemon. If UI was built for 9867, you'll have a mismatch again. That's why changing the DEFAULT_PORT and rebuilding is better.

## Commands Added

### UI Package Scripts

```json
{
  "sync-port": "tsx ../scripts/sync-port-config.ts",
  "prebuild": "npm run sync-port && node ../scripts/generate-version.js"
}
```

### Usage

```bash
# Manual sync (if needed)
cd keyrx_ui
npm run sync-port

# Automatic sync (happens on build)
npm run build         # Runs sync-port automatically
npm run build:production  # Runs sync-port automatically
```

## REBUILD_SSOT.bat Updated

New steps:
```
[1/8] Stopping daemon...
[2/8] Cleaning build artifacts...
[3/8] Regenerating UI version.ts...
[4/8] Syncing port configuration (SSOT)...  ← NEW
[5/8] Rebuilding UI (PRODUCTION MODE)...    ← CHANGED (was dev mode)
[6/8] Rebuilding daemon (clean build)...
[7/8] Installing fresh binary...
[8/8] Verifying timestamps...
```

## Testing the Fix

### Quick Test (30 seconds)

```powershell
# 1. Remove custom settings
Remove-Item "C:\Users\$env:USERNAME\AppData\Roaming\keyrx\settings.json" -ErrorAction SilentlyContinue

# 2. Run SSOT rebuild
.\REBUILD_SSOT.bat

# 3. Open UI from system tray
# Should work perfectly now!
```

### Comprehensive Test (5 minutes)

```powershell
# 1. Run the sync script manually
cd keyrx_ui
npm run sync-port

# 2. Verify all configs match
Write-Host "Checking SSOT compliance..."

$rustPort = (Select-String -Path "..\keyrx_daemon\src\services\settings_service.rs" -Pattern "DEFAULT_PORT.*=\s*(\d+)" | ForEach-Object { $_.Matches.Groups[1].Value })
$envPort = (Select-String -Path ".env.development" -Pattern "localhost:(\d+)" | Select-Object -First 1 | ForEach-Object { $_.Matches.Groups[1].Value })
$vitePort = (Select-String -Path "vite.config.ts" -Pattern "localhost:(\d+)" | Select-Object -First 1 | ForEach-Object { $_.Matches.Groups[1].Value })

Write-Host "Rust DEFAULT_PORT: $rustPort"
Write-Host ".env.development: $envPort"
Write-Host "vite.config.ts: $vitePort"

if ($rustPort -eq $envPort -and $envPort -eq $vitePort) {
    Write-Host "✓ SSOT verified! All ports match: $rustPort" -ForegroundColor Green
} else {
    Write-Host "✗ SSOT violated! Ports don't match" -ForegroundColor Red
}

# 3. Rebuild and test
cd ..
.\REBUILD_SSOT.bat

# 4. Test from system tray
```

## Benefits of This Solution

1. ✅ **True SSOT**: One place to change port (Rust source)
2. ✅ **Automatic Sync**: Runs on every build (can't forget)
3. ✅ **Type Safety**: TypeScript extracts from Rust at build time
4. ✅ **Production Ready**: UI uses dynamic `window.location.origin`
5. ✅ **Dev Friendly**: Dev server proxy also synced
6. ✅ **Version Control**: Generated files show clear sync history

## Migration from Old System

If you have existing installs:

1. **Delete custom settings** (optional, to use default):
   ```powershell
   Remove-Item "C:\Users\$env:USERNAME\AppData\Roaming\keyrx\settings.json"
   ```

2. **Rebuild with SSOT**:
   ```bash
   .\REBUILD_SSOT.bat
   ```

3. **Verify**:
   - Open UI from system tray
   - Check it opens at `http://localhost:9867`
   - Test profile activation
   - Check metrics show events

## Future Improvements

### Phase 2: Runtime Port Discovery

Add API endpoint for UI to discover port dynamically:

```typescript
// ui/src/config/env.ts
async function getApiUrl(): Promise<string> {
  // In production, try to fetch port from daemon
  if (import.meta.env.PROD) {
    try {
      const response = await fetch('/api/config/port');
      const { port } = await response.json();
      return `${window.location.protocol}//${window.location.hostname}:${port}`;
    } catch {
      // Fallback to same origin
      return window.location.origin;
    }
  }

  // Development: use synced config
  return import.meta.env.VITE_API_URL || 'http://localhost:9867';
}
```

### Phase 3: Config File as SSOT

Centralize all configuration:

```toml
# config/default.toml (SSOT for everything)
[server]
port = 9867
host = "127.0.0.1"

[ui]
title = "KeyRx"
```

Generate both Rust and TypeScript from this during build.

---

**Status**: ✅ SSOT Fully Implemented
**Testing**: Ready (run REBUILD_SSOT.bat)
**Maintenance**: Zero (automatic sync on every build)
