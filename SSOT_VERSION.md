# SSOT Version Management

## Single Source of Truth Architecture

KeyRx enforces SSOT (Single Source of Truth) for version and build information:

```
Cargo.toml (workspace.package.version)
    ↓
    ├─→ Backend (keyrx_daemon, keyrx_core, keyrx_compiler)
    └─→ package.json (must match manually)
            ↓
            └─→ Frontend (generate-version.js → version.ts)
```

## Version Sources

| File | Purpose | Type | Auto-Updated? |
|------|---------|------|---------------|
| `Cargo.toml` | **Backend SSOT** | Source | ❌ Manual |
| `keyrx_ui/package.json` | **Frontend SSOT** | Source | ❌ Manual (should match Cargo.toml) |
| `keyrx_ui/src/version.ts` | Frontend runtime | Generated | ✅ Auto (via npm prebuild) |
| `keyrx_daemon/build.rs` | Backend build info | Generated | ✅ Auto (via cargo build) |
| `keyrx_installer.wxs` | MSI version | Source | ❌ Manual (should match) |

## Current Version: v0.1.5

### Version History
- **v0.1.5** - SSOT enforcement, build time fixes (2026/01/29)
- v0.1.4 - Never released (dev only)
- v0.1.3 - Installer improvements
- v0.1.2 - Bug fixes
- v0.1.1 - Auto-start with admin rights
- v0.1.0 - Initial release

## Updating Version (SSOT Protocol)

### Step 1: Update Source Files (Manual)

```bash
# 1. Update Cargo.toml
version = "0.1.X"

# 2. Update package.json (must match!)
"version": "0.1.X"

# 3. Update keyrx_installer.wxs
Version="0.1.X.0"

# 4. Update build_windows_installer.ps1
$Version = "0.1.X"
```

### Step 2: Regenerate Everything

Run the SSOT rebuild script:
```bat
REBUILD_SSOT.bat
```

This will:
1. Stop daemon
2. Clean all cached artifacts
3. Regenerate `version.ts` from package.json
4. Rebuild UI with fresh version
5. Rebuild daemon (forces build.rs to regenerate BUILD_DATE)
6. Install fresh binary
7. Verify all timestamps and versions

## Verification

After rebuild, all these should show **v0.1.5** with **current timestamp**:

### 1. Cargo Metadata
```bash
cargo metadata --format-version 1 | grep -A 5 keyrx_daemon
```
Should show: `"version": "0.1.5"`

### 2. Source Binary
```bash
Get-Item target\release\keyrx_daemon.exe | Select-Object LastWriteTime
```
Should show: Current timestamp

### 3. Installed Binary
```bash
Get-Item "C:\Program Files\KeyRx\bin\keyrx_daemon.exe" | Select-Object LastWriteTime
```
Should show: Same as source

### 4. System Tray
- Right-click tray icon → About
- Should show: **v0.1.5**, current build time in JST

### 5. Web UI
- Open http://localhost:9867
- Footer should show: **v0.1.5**, current date

### 6. API
```bash
curl http://localhost:9867/api/health
```
Should return:
```json
{
  "status": "ok",
  "version": "0.1.5"
}
```

### 7. API Version Info
```bash
curl http://localhost:9867/api/version
```
Should return:
```json
{
  "version": "0.1.5",
  "build_time": "2026-01-29T...",
  "git_hash": "...",
  "platform": "windows"
}
```

## Automated Tests

```powershell
# Quick installation verification (auto-detects expected timestamp)
.\scripts\test_installation.ps1

# All 7 tests should pass
```

## Common Issues

### Issue: System tray shows old version

**Cause:** Cached build.rs output, binary not fully rebuilt

**Fix:**
```bat
REBUILD_SSOT.bat
```

### Issue: Web UI shows old version

**Cause:** version.ts not regenerated, old UI embedded

**Fix:**
```bash
cd keyrx_ui
node ../scripts/generate-version.js
npm run build
cd ..
cargo build --release -p keyrx_daemon
```

### Issue: Version mismatch between Cargo.toml and package.json

**Cause:** Manual update missed one file

**Fix:**
1. Manually update both to match
2. Run `REBUILD_SSOT.bat`

The `generate-version.js` script will warn about mismatches:
```
⚠️  Version mismatch!
   Cargo.toml: 0.1.5
   package.json: 0.1.4
   Please sync versions manually
```

## SSOT Principles

1. **Never** hardcode version strings in code
2. **Always** use `env!("CARGO_PKG_VERSION")` in Rust
3. **Always** import from `version.ts` in TypeScript
4. **Never** edit `version.ts` manually (auto-generated)
5. **Always** regenerate after version change
6. **Never** commit with version mismatches

## Development Workflow

```bash
# Before committing version change:
1. Update Cargo.toml workspace.package.version
2. Update package.json version (must match)
3. Update keyrx_installer.wxs Version
4. Run REBUILD_SSOT.bat
5. Run tests: .\scripts\test_installation.ps1
6. Verify all 7 tests pass
7. Commit changes
```

## CI/CD Integration

The CI/CD pipeline should:
1. Check version consistency (Cargo.toml == package.json)
2. Run `generate-version.js` before UI build
3. Run `cargo build` (triggers build.rs)
4. Verify all versions match in artifacts
5. Fail if any mismatch detected

## Files Changed for v0.1.5

- ✅ `Cargo.toml` → `version = "0.1.5"`
- ✅ `keyrx_ui/package.json` → `"version": "0.1.5"`
- ✅ `keyrx_daemon/keyrx_installer.wxs` → `Version="0.1.5.0"`
- ✅ `scripts/build_windows_installer.ps1` → `$Version = "0.1.5"`
- ✅ `scripts/test_installation.ps1` → Auto-detect timestamp
- ✅ Created `REBUILD_SSOT.bat` → Enforce SSOT rebuild
- ✅ Created this documentation

## Next Version Update

When bumping to v0.1.6:
```bash
1. Update 4 source files (see Step 1 above)
2. Run REBUILD_SSOT.bat
3. Test with test_installation.ps1
4. Update this file with new version number
5. Commit
```
