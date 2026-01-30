# SSOT Port Configuration Analysis

## Current State (SSOT VIOLATED ❌)

### Port Definitions (4 places!)

1. **Daemon Settings** (Runtime)
   - File: `C:\Users\ryosu\AppData\Roaming\keyrx\settings.json`
   - Value: `{"port": 9868}`
   - Used by: Daemon web server

2. **Daemon Default** (Code)
   - File: `keyrx_daemon/src/services/settings_service.rs:10`
   - Value: `pub const DEFAULT_PORT: u16 = 9867;`
   - Used by: Daemon fallback if no settings.json

3. **UI Development** (Build-time)
   - File: `keyrx_ui/.env.development:5`
   - Value: `VITE_API_URL=http://localhost:9867`
   - Used by: Dev server + hardcoded in dev builds

4. **UI Dev Proxy** (Dev server)
   - File: `keyrx_ui/vite.config.ts:35`
   - Value: `target: 'http://localhost:9867'`
   - Used by: Vite dev server proxy

5. **UI Production** (Runtime)
   - File: `keyrx_ui/.env.production:5`
   - Value: `VITE_API_URL=` (empty)
   - Result: Uses `window.location.origin` ✅ (This is correct!)

## The Problem

When you click system tray → Open UI:
1. Daemon serves embedded UI from `http://localhost:9868`
2. UI loads in browser at `http://localhost:9868`
3. UI JavaScript checks `import.meta.env.PROD`
4. **If built in DEV mode**: Uses hardcoded `http://localhost:9867` ❌
5. **If built in PROD mode**: Uses `window.location.origin` = `http://localhost:9868` ✅

## Check Current Build Mode

```bash
cd keyrx_ui/dist/assets
grep -l "import.meta.env.PROD" *.js
```

If the built files contain development code, the UI was built in dev mode.

## SSOT Solution Architecture

### Option 1: Runtime Config (RECOMMENDED) ✅

**SSOT**: Daemon `settings.json` only

```
settings.json (port: 9868)
    ↓
Daemon reads at startup
    ↓
Daemon serves UI at http://localhost:9868
    ↓
UI uses window.location.origin → http://localhost:9868
```

**Requirements**:
1. UI must be built in **PRODUCTION mode** (not dev mode)
2. `.env.production` must have empty `VITE_API_URL`
3. `env.ts` already uses `window.location.origin` ✅

### Option 2: Build-time Config Injection

**SSOT**: Daemon `settings_service.rs::DEFAULT_PORT`

Generate `.env.development` from Rust code during build:

```typescript
// scripts/sync-port.ts
import fs from 'fs';
import path from 'path';

// Read Rust source for DEFAULT_PORT
const rustSrc = fs.readFileSync(
  '../keyrx_daemon/src/services/settings_service.rs',
  'utf8'
);
const match = rustSrc.match(/DEFAULT_PORT:\s*u16\s*=\s*(\d+)/);
const port = match ? match[1] : '9867';

// Update .env files
const envDev = `VITE_API_URL=http://localhost:${port}\nVITE_WS_URL=ws://localhost:${port}/ws-rpc\n`;
fs.writeFileSync('.env.development', envDev);

// Update vite.config.ts proxy
// ...
```

### Option 3: Settings File as SSOT

Generate daemon default from a single config file:

```toml
# config/default.toml (SSOT)
[server]
port = 9867

[ui]
# Empty - uses same origin
```

Generate both Rust and TypeScript from this file.

## Recommended Fix (5 minutes)

### Step 1: Ensure Production Build

```bash
cd keyrx_ui

# Build in production mode
npm run build:production

# Verify it uses window.location.origin
grep -r "window.location.origin" dist/assets/*.js
```

### Step 2: Standardize Port to 9867

```bash
# Delete custom settings (use default)
rm "C:\Users\ryosu\AppData\Roaming\keyrx\settings.json"

# Or set to 9867
echo '{"port":9867}' > "C:\Users\ryosu\AppData\Roaming\keyrx\settings.json"
```

### Step 3: Rebuild Daemon with Embedded UI

```bash
# Rebuild SSOT
.\REBUILD_SSOT.bat
```

### Step 4: Verify SSOT

1. Check daemon port:
```bash
taskkill /F /IM keyrx_daemon.exe
"C:\Program Files\KeyRx\bin\keyrx_daemon.exe" run
# Look for: "Starting web server on http://127.0.0.1:9867"
```

2. Open system tray → Open UI
3. Check browser URL: Should be `http://localhost:9867`
4. Check browser console: Should show API calls to same origin
5. Test profile activation: Should work

## SSOT Implementation Plan (Proper Fix)

### Phase 1: Enforce Production Build (IMMEDIATE)

**File**: `REBUILD_SSOT.bat` (already exists)

Add check:
```batch
echo [4/7] Building UI in PRODUCTION mode...
cd keyrx_ui
call npm run build:production
if %errorLevel% neq 0 (
    echo   ERROR: UI build failed!
    exit /b 1
)
```

### Phase 2: Port Sync Script (SHORT-TERM)

**File**: `scripts/sync-port-config.ts` (NEW)

```typescript
/**
 * Sync port configuration from Rust source to UI configs
 * Ensures SSOT for port number
 */
import fs from 'fs';
import path from 'path';

const DAEMON_SOURCE = '../keyrx_daemon/src/services/settings_service.rs';
const VITE_CONFIG = '../keyrx_ui/vite.config.ts';
const ENV_DEV = '../keyrx_ui/.env.development';

// Extract DEFAULT_PORT from Rust
function extractDefaultPort(): number {
  const content = fs.readFileSync(DAEMON_SOURCE, 'utf8');
  const match = content.match(/pub const DEFAULT_PORT:\s*u16\s*=\s*(\d+);/);
  if (!match) throw new Error('DEFAULT_PORT not found in Rust source');
  return parseInt(match[1], 10);
}

// Update vite.config.ts proxy
function updateViteConfig(port: number): void {
  let content = fs.readFileSync(VITE_CONFIG, 'utf8');
  content = content.replace(
    /target: 'http:\/\/localhost:\d+'/g,
    `target: 'http://localhost:${port}'`
  );
  content = content.replace(
    /target: 'ws:\/\/localhost:\d+'/g,
    `target: 'ws://localhost:${port}'`
  );
  fs.writeFileSync(VITE_CONFIG, content);
}

// Update .env.development
function updateEnvDev(port: number): void {
  const content = `# Auto-generated from DEFAULT_PORT in settings_service.rs
# DO NOT EDIT MANUALLY - Run npm run sync-port
VITE_API_URL=http://localhost:${port}
VITE_WS_URL=ws://localhost:${port}/ws-rpc
VITE_DEBUG=true
VITE_ENV=development
`;
  fs.writeFileSync(ENV_DEV, content);
}

// Main
const port = extractDefaultPort();
console.log(`✓ Extracted DEFAULT_PORT: ${port}`);

updateViteConfig(port);
console.log(`✓ Updated vite.config.ts`);

updateEnvDev(port);
console.log(`✓ Updated .env.development`);

console.log(`\n✓ SSOT enforced: All configs use port ${port}`);
```

**Add to package.json**:
```json
{
  "scripts": {
    "sync-port": "tsx ../scripts/sync-port-config.ts",
    "prebuild": "npm run sync-port && node ../scripts/generate-version.js"
  }
}
```

### Phase 3: Pre-commit Hook (LONG-TERM)

**File**: `.husky/pre-commit`

```bash
#!/bin/sh

# Check port consistency
cd keyrx_ui
npm run sync-port

# Check if files changed
if ! git diff --quiet keyrx_ui/.env.development keyrx_ui/vite.config.ts; then
    echo "❌ Port configuration out of sync!"
    echo "   Run 'cd keyrx_ui && npm run sync-port' and commit changes"
    exit 1
fi
```

## Verification Checklist

After implementing SSOT:

```bash
# 1. Check DEFAULT_PORT in Rust
grep "DEFAULT_PORT" keyrx_daemon/src/services/settings_service.rs

# 2. Check .env.development
cat keyrx_ui/.env.development | grep VITE_API_URL

# 3. Check vite.config.ts
grep "target: 'http://localhost" keyrx_ui/vite.config.ts

# 4. Check .env.production (should be empty)
cat keyrx_ui/.env.production | grep VITE_API_URL

# 5. Verify production build uses window.location.origin
cd keyrx_ui/dist/assets
grep -l "window.location.origin" *.js
```

All should show **same port number** (9867 or 9868).

## Current Quick Fix

**Immediate workaround** (30 seconds):

```powershell
# Standardize to 9867
echo '{"port":9867}' | Out-File "C:\Users\$env:USERNAME\AppData\Roaming\keyrx\settings.json" -Encoding UTF8

# Restart daemon
taskkill /F /IM keyrx_daemon.exe
Start-Process "C:\Program Files\KeyRx\bin\keyrx_daemon.exe" -ArgumentList "run"
```

**Proper fix** (5 minutes):

```bash
cd keyrx_ui
npm run build:production  # Production mode = window.location.origin
cd ..
.\REBUILD_SSOT.bat        # Rebuild daemon with new UI
```

---

**Conclusion**: The SSOT violation is real, but **production builds already handle it correctly** via `window.location.origin`. The issue is the UI was built in dev mode with hardcoded port 9867.
