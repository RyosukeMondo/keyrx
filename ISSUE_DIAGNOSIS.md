# Issue Diagnosis and Resolution

## Issues Reported

### 1. Web UI Error: WasmProvider Context Error

**Error Message:**
```
useWasmContext must be used within WasmProvider
```

**Location:** `/config` page (ConfigPage.tsx)

**Root Cause:**
The error comes from `MonacoEditor.tsx:41` which calls `useWasmContext()`. This component is used in the ConfigPage for editing Rhai configuration files.

**Analysis:**
- `WasmProvider` IS correctly wrapping the entire app in `App.tsx:20-46`
- `MonacoEditor` needs WASM for syntax validation
- The error suggests either:
  1. Build mismatch (old UI code running)
  2. WASM failed to load/initialize
  3. Race condition during lazy loading

**Most Likely Cause:** The daemon is serving an outdated UI build from when it was compiled at 17:13, but the current codebase has changed.

### 2. Metrics Page Not Catching Key Events

**Symptom:** Pressing keys shows nothing on `/metrics` page - no remapping happening

**Root Causes:**
1. **No debug logging** - Daemon not started with `--debug` flag, so can't diagnose
2. **No config loaded** - Previous logs showed "0 key mappings loaded"
3. **Config not activated** - Example config not imported and activated through Web UI

## Solutions

### Automated Fix (Recommended)

Run the diagnostic and fix script:
```powershell
.\scripts\windows\Diagnose-And-Fix.ps1
```

This will:
1. ✅ Check daemon status
2. ✅ Verify build timestamps
3. ✅ Check profile status
4. ✅ Rebuild if needed (WASM → UI → Daemon)
5. ✅ Restart daemon with debug logging
6. ✅ Verify everything is working

### Manual Fix Steps

#### Fix Issue #1: WasmProvider Error

**Option A: Rebuild Everything (Recommended)**
```powershell
# 1. Stop daemon
Stop-Process -Name keyrx_daemon -Force

# 2. Rebuild UI (this fixes the WasmProvider issue)
cd keyrx_ui
npm run build:wasm       # Build WASM
npm run build            # Build UI
cd ..

# 3. Rebuild daemon with fresh UI
cargo build --release --features windows

# 4. Launch with debug
.\scripts\windows\Debug-Launch.ps1
```

**Option B: Just Restart (If builds are recent)**
```powershell
# Stop and restart daemon
Stop-Process -Name keyrx_daemon -Force
.\scripts\windows\Debug-Launch.ps1
```

#### Fix Issue #2: Metrics Not Catching Events

**Step 1: Ensure Debug Logging**
```powershell
# Daemon must run with --debug flag
.\scripts\windows\Debug-Launch.ps1
```

**Step 2: Import Example Config**
```powershell
# Copy example config to profiles directory
.\scripts\windows\Import-Example-Config.ps1
```

**Step 3: Activate Through Web UI**
1. Open http://localhost:9867
2. Click "Profiles" in sidebar
3. Click "Activate" on "user_layout" profile
4. Wait for green notification: "Profile 'user_layout' applied!"

**Step 4: Verify on Metrics Page**
1. Go to http://localhost:9867/metrics
2. Press keys - should see remapped output
3. If nothing appears, check debug log:
   ```powershell
   Get-Content "$env:TEMP\keyrx-debug.log" -Tail 50 -Wait
   ```

## Understanding the System

### Build Chain
```
keyrx_core (Rust)
    ↓ wasm-pack
WASM module (.wasm)
    ↓ Vite
UI Bundle (dist/)
    ↓ cargo build
keyrx_daemon.exe (with embedded UI)
```

**Important:** When you rebuild the daemon, it embeds the **current** `keyrx_ui/dist/` contents. If you change UI code, you must:
1. Rebuild UI (`npm run build`)
2. Rebuild daemon (`cargo build --release`)

### Profile Activation Flow
```
1. User clicks "Activate" in Web UI
2. Web UI → POST /api/profiles/{name}/activate
3. Daemon compiles .rhai → .krx
4. Daemon restarts automatically
5. Daemon loads .krx on startup
6. Remapping starts working
```

### Why Metrics Shows Nothing

**Possible Reasons:**
1. ❌ No config loaded (0 mappings) → Activate a profile
2. ❌ Daemon not running as admin → Right-click → Run as administrator
3. ❌ Config compilation failed → Check for errors in Web UI activation
4. ❌ Daemon not capturing input → Check debug logs for Raw Input registration

## Verification Checklist

After applying fixes:

- [ ] Daemon running with debug logging
  ```powershell
  Get-Process keyrx_daemon
  Test-Path "$env:TEMP\keyrx-debug.log"
  ```

- [ ] Web UI accessible
  ```powershell
  Invoke-WebRequest -Uri "http://localhost:9867" -UseBasicParsing
  ```

- [ ] Profile imported
  ```powershell
  Test-Path "$env:APPDATA\keyrx\profiles\user_layout.rhai"
  ```

- [ ] Profile activated (check Web UI - green checkmark on profile)

- [ ] Config loaded (check logs)
  ```powershell
  Get-Content "$env:TEMP\keyrx-debug.log" | Select-String "Loaded.*mappings"
  ```
  Should show: `Loaded 256 key mappings` (not 0)

- [ ] Keys remapping (test on /metrics page)

## Debug Commands

```powershell
# Check daemon status
Get-Process keyrx_daemon | Format-List

# View logs in real-time
Get-Content "$env:TEMP\keyrx-debug.log" -Tail 50 -Wait

# Check build timestamps
Get-Item keyrx_ui/dist/index.html, target/release/keyrx_daemon.exe | Format-Table FullName, LastWriteTime

# Check profiles
Get-ChildItem "$env:APPDATA\keyrx\profiles"

# Test API
Invoke-RestMethod -Uri "http://localhost:9867/api/profiles"

# Restart daemon with debug
Stop-Process -Name keyrx_daemon -Force
.\scripts\windows\Debug-Launch.ps1
```

## Common Gotchas

1. **UI changes don't appear** → Rebuild UI, then rebuild daemon
2. **"WASM unavailable" in UI** → WASM didn't build correctly, run `npm run build:wasm`
3. **"0 mappings loaded"** → No profile activated, activate one through Web UI
4. **Metrics shows nothing** → Daemon not as admin, or no profile activated
5. **Web UI shows old content** → Daemon serving old embedded UI, rebuild daemon

## Quick Recovery

If everything is broken:
```powershell
# Nuclear option: rebuild everything
.\scripts\windows\Diagnose-And-Fix.ps1

# Or manually:
Stop-Process -Name keyrx_daemon -Force
cd keyrx_ui && npm run build:wasm && npm run build && cd ..
cargo build --release --features windows
.\scripts\windows\Debug-Launch.ps1
.\scripts\windows\Import-Example-Config.ps1
Start-Process "http://localhost:9867"
```

Then activate profile in Web UI.

---

**TL;DR:**
1. Run `.\scripts\windows\Diagnose-And-Fix.ps1`
2. Import config: `.\scripts\windows\Import-Example-Config.ps1`
3. Activate profile in Web UI
4. Test on /metrics page
