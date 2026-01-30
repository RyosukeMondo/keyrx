# Version Management Guide

Complete guide to managing versions across the KeyRx project with Single Source of Truth (SSOT) enforcement.

## Table of Contents

- [Overview](#overview)
- [Version Architecture](#version-architecture)
- [Quick Reference](#quick-reference)
- [Step-by-Step Version Update](#step-by-step-version-update)
- [SSOT Principles](#ssot-principles)
- [Verification Methods](#verification-methods)
- [Common Errors](#common-errors)
- [CI/CD Integration](#cicd-integration)
- [Best Practices](#best-practices)

## Overview

KeyRx uses a unified version management system with **Cargo.toml** as the single source of truth (SSOT). Automated scripts ensure consistency across all components, and build-time validation prevents mismatched deployments.

**Key Features:**
- ✅ Single command updates all version files
- ✅ Build-time validation fails compilation on mismatch
- ✅ Runtime verification detects inconsistencies
- ✅ CI/CD integration prevents merging bad versions
- ✅ Comprehensive diagnostics for troubleshooting

## Version Architecture

### Version Sources (SSOT)

```
Cargo.toml (workspace.package.version) ← SINGLE SOURCE OF TRUTH
    ↓ (auto-synced by sync-version.sh)
    ├─→ keyrx_ui/package.json (version)
    ├─→ keyrx_daemon/keyrx_installer.wxs (Version)
    └─→ scripts/build_windows_installer.ps1 ($Version)
        ↓ (auto-generated at build time)
        ├─→ keyrx_ui/src/version.ts (VERSION, BUILD_TIME, GIT_COMMIT)
        └─→ keyrx_daemon/src/version.rs (BUILD_DATE, GIT_HASH)
```

### Version Flow

1. **Source of Truth:** `Cargo.toml` workspace.package.version
2. **Sync Script:** `sync-version.sh` propagates to all files
3. **Build Validation:** `build.rs` checks consistency at compile time
4. **Runtime Generation:** Version constants embedded in binaries
5. **Runtime Verification:** Scripts validate running system

## Quick Reference

### Update Version

```bash
# Single command to update all sources
./scripts/sync-version.sh 0.2.0
```

### Check Version Consistency

```bash
# Bash (Linux/Mac)
./scripts/sync-version.sh --check

# PowerShell (Windows)
.\scripts\version-check.ps1
```

### Verify Installation

```powershell
# Full verification (Windows)
.\scripts\installer-health-check.ps1 -PostInstall
```

### Current Version

Check these locations:
- **Cargo.toml:** `[workspace.package] version = "0.1.5"`
- **API:** `curl http://localhost:9867/api/version`
- **Web UI:** Footer shows version + build time
- **System Tray:** Right-click → About

## Step-by-Step Version Update

### Prerequisites

Before starting:

```bash
# 1. Ensure clean working directory
git status
# Should show: "nothing to commit, working tree clean"

# 2. Check current version consistency
./scripts/sync-version.sh --check
# Should show: "All versions match"

# 3. Pull latest changes
git pull origin main

# 4. Run full test suite
make verify
# All tests should pass
```

### Step 1: Update Version Number

**Single command (recommended):**

```bash
# Update all version sources from command line
./scripts/sync-version.sh 0.2.0
```

This automatically updates:
- `Cargo.toml` → `[workspace.package] version = "0.2.0"`
- `keyrx_ui/package.json` → `"version": "0.2.0"`
- `keyrx_daemon/keyrx_installer.wxs` → `Version="0.2.0.0"`
- `scripts/build_windows_installer.ps1` → `$Version = "0.2.0"`

**Manual update (not recommended):**

If you must edit manually, update these 4 files:

```toml
# Cargo.toml
[workspace.package]
version = "0.2.0"
```

```json
// keyrx_ui/package.json
{
  "version": "0.2.0"
}
```

```xml
<!-- keyrx_daemon/keyrx_installer.wxs -->
<Product Version="0.2.0.0" ...>
```

```powershell
# scripts/build_windows_installer.ps1
$Version = "0.2.0"
```

Then verify consistency:

```bash
./scripts/sync-version.sh --check
```

### Step 2: Clean Build Environment

```bash
# Remove all cached artifacts
cargo clean

# Clean UI build artifacts
cd keyrx_ui
rm -rf dist/ node_modules/.vite/
cd ..
```

### Step 3: Rebuild Everything

```bash
# Build WASM module
cd keyrx_ui
npm run build:wasm
# Expected: pkg/ directory with keyrx_core_bg.wasm

# Build UI
npm run build
# Expected: dist/ directory with index.html
cd ..

# Build daemon (embeds UI)
cargo build --release -p keyrx_daemon
# Expected: target/release/keyrx_daemon(.exe)
```

### Step 4: Build Installer (Windows)

```powershell
# Build MSI installer
.\scripts\build_windows_installer.ps1

# Expected output:
# target\installer\KeyRx-0.2.0-x64.msi
```

### Step 5: Verify Version Consistency

**All version sources:**

```powershell
.\scripts\version-check.ps1
```

Expected output:
```
Version Consistency Check
========================

Source              Version    Status
------              -------    ------
Cargo.toml          0.2.0      ✓
package.json        0.2.0      ✓
keyrx_installer.wxs 0.2.0.0    ✓
Source Binary       0.2.0      ✓
Installed Binary    0.2.0      ✓
Running Daemon      0.2.0      ✓

Result: All versions match (0.2.0)
```

**API check:**

```bash
# Check version via API
curl http://localhost:9867/api/version

# Expected:
{
  "version": "0.2.0",
  "build_time": "2026-01-30T10:00:00Z",
  "git_hash": "abc123",
  "platform": "windows"
}
```

**Binary check:**

```powershell
# Check binary directly
.\target\release\keyrx_daemon.exe --version

# Expected:
KeyRx v0.2.0
```

### Step 6: Test Installation

```powershell
# Pre-install validation
.\scripts\installer-health-check.ps1 -PreInstall

# Should show:
#   ✓ Admin Rights
#   ✓ MSI File
#   ✓ Version Match
#   ✓ Binary Fresh (< 24 hours)
#   ✓ All Files Present
```

```powershell
# Install the MSI
msiexec /i target\installer\KeyRx-0.2.0-x64.msi /qn

# Wait for installation (5-10 seconds)
Start-Sleep -Seconds 10
```

```powershell
# Post-install validation
.\scripts\installer-health-check.ps1 -PostInstall

# Should show:
#   ✓ Binary Installed
#   ✓ Version Match
#   ✓ Daemon Running
#   ✓ API Responding
#   ✓ Profiles Working
```

### Step 7: Commit and Tag

```bash
# Stage version files
git add Cargo.toml
git add keyrx_ui/package.json
git add keyrx_daemon/keyrx_installer.wxs
git add scripts/build_windows_installer.ps1

# Commit
git commit -m "chore: bump version to 0.2.0"

# Create annotated tag
git tag -a v0.2.0 -m "Release version 0.2.0"

# Push with tags
git push origin main
git push origin v0.2.0
```

### Step 8: Create GitHub Release (Optional)

1. Go to GitHub repository → Releases → Draft new release
2. Choose tag: `v0.2.0`
3. Release title: `KeyRx v0.2.0`
4. Description: Changelog and notable changes
5. Attach: `KeyRx-0.2.0-x64.msi`
6. Publish release

## SSOT Principles

### Principle 1: Single Source of Truth

**`Cargo.toml` is the ONLY manual version source.**

All other files are synchronized from it automatically.

### Principle 2: Never Hardcode Versions

**Bad:**
```rust
const VERSION: &str = "0.1.5"; // ❌ Hardcoded
```

**Good:**
```rust
const VERSION: &str = env!("CARGO_PKG_VERSION"); // ✅ From Cargo.toml
```

### Principle 3: Auto-Generate Everything Else

**Generated files (DO NOT EDIT manually):**
- `keyrx_ui/src/version.ts` - Generated by `generate-version.js`
- `keyrx_daemon/src/version.rs` - Generated by `build.rs`

**Source files (Edit via sync script):**
- `Cargo.toml` - Edit manually or via `sync-version.sh`
- `keyrx_ui/package.json` - Synced by `sync-version.sh`
- `keyrx_daemon/keyrx_installer.wxs` - Synced by `sync-version.sh`
- `scripts/build_windows_installer.ps1` - Synced by `sync-version.sh`

### Principle 4: Fail Fast on Mismatch

**Build-time validation in `build.rs`:**

```rust
// Compile-time check
let cargo_version = env!("CARGO_PKG_VERSION");
let package_json_version = read_package_json_version();

if cargo_version != package_json_version {
    panic!("Version mismatch! Cargo.toml={}, package.json={}",
           cargo_version, package_json_version);
}
```

This prevents building with mismatched versions.

### Principle 5: Always Verify After Changes

```bash
# After ANY version-related change:
./scripts/sync-version.sh --check
```

## Verification Methods

### Method 1: sync-version.sh --check

Validates all source files match Cargo.toml.

```bash
./scripts/sync-version.sh --check

# Pass: All versions match (0.1.5)
# Exit code: 0

# Fail: Version mismatch detected
#       Cargo.toml: 0.1.5
#       package.json: 0.1.4
# Exit code: 1
```

### Method 2: version-check.ps1

Comprehensive runtime verification (Windows).

```powershell
.\scripts\version-check.ps1

# Checks:
# 1. Cargo.toml version
# 2. package.json version
# 3. keyrx_installer.wxs version
# 4. Source binary (target/release/keyrx_daemon.exe)
# 5. Installed binary (C:\Program Files\KeyRx\bin\keyrx_daemon.exe)
# 6. Running daemon (via API)
```

### Method 3: Build-Time Validation

Automatic check during compilation.

```bash
cargo build --release -p keyrx_daemon

# Pass: Build succeeds
# Fail: Compilation error with version mismatch details
```

### Method 4: installer-health-check.ps1

Pre-install validation.

```powershell
.\scripts\installer-health-check.ps1 -PreInstall

# Checks:
# - MSI version matches binary version
# - Binary timestamp is fresh (< 24 hours)
# - All required files present in MSI
```

### Method 5: CI/CD Check

Automatic validation on every commit.

```yaml
# .github/workflows/ci.yml
- name: Verify Version Consistency
  run: ./scripts/sync-version.sh --check
```

## Common Errors

### Error 1: Version Mismatch at Build Time

**Symptoms:**
```
error: Version mismatch detected!
       Cargo.toml version: 0.1.5
       package.json version: 0.1.4
       Run: ./scripts/sync-version.sh --fix
```

**Cause:** Source files out of sync

**Fix:**
```bash
./scripts/sync-version.sh --fix
# Or
./scripts/sync-version.sh 0.1.5
```

### Error 2: Stale Binary Timestamp

**Symptoms:**
```
WARNING: Binary is stale
Binary timestamp: 2026-01-28 10:00:00
Age: 30 hours
```

**Cause:** Binary not rebuilt after version change

**Fix:**
```bash
cargo clean
cargo build --release -p keyrx_daemon
```

### Error 3: WiX Version Format Wrong

**Symptoms:**
```
error: Invalid version format in WiX file
Expected: X.Y.Z.W (4 components)
Got: 0.1.5 (3 components)
```

**Cause:** WiX requires 4-component version (X.Y.Z.W)

**Fix:**
```bash
# sync-version.sh automatically adds .0
./scripts/sync-version.sh 0.1.5
# Sets WiX to: 0.1.5.0
```

### Error 4: Web UI Shows Old Version

**Symptoms:**
- Web UI footer shows `v0.1.4`
- But Cargo.toml says `0.1.5`

**Cause:** `version.ts` not regenerated

**Fix:**
```bash
cd keyrx_ui
npm run prebuild  # Regenerates version.ts
npm run build
cd ..
cargo build --release -p keyrx_daemon
```

### Error 5: Git Hash Shows "unknown"

**Symptoms:**
```typescript
// keyrx_ui/src/version.ts
export const GIT_COMMIT = 'unknown';
```

**Cause:** Not in a git repository or git not installed

**Fix:**
```bash
# Check git works
git status

# Regenerate
cd keyrx_ui
node ../scripts/generate-version.js
```

## CI/CD Integration

### GitHub Actions Workflow

```yaml
# .github/workflows/ci.yml

name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  verify-version:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Check Version Consistency
        run: |
          ./scripts/sync-version.sh --check
          if [ $? -ne 0 ]; then
            echo "❌ Version mismatch detected!"
            echo "Run: ./scripts/sync-version.sh --fix"
            exit 1
          fi

  build:
    needs: verify-version
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Build
        run: cargo build --release --workspace

      # Build will fail automatically if versions mismatch
```

### Pre-Commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Check version consistency before commit
./scripts/sync-version.sh --check
if [ $? -ne 0 ]; then
    echo "❌ Commit blocked: Version mismatch"
    echo "Fix: ./scripts/sync-version.sh --fix"
    exit 1
fi

exit 0
```

Install:
```bash
make setup  # Installs pre-commit hooks
```

## Best Practices

### DO

✅ **Use sync-version.sh for all version updates**
```bash
./scripts/sync-version.sh 0.2.0
```

✅ **Verify consistency after changes**
```bash
./scripts/sync-version.sh --check
```

✅ **Clean build for releases**
```bash
cargo clean && cargo build --release
```

✅ **Test installation before committing**
```powershell
.\scripts\installer-health-check.ps1
```

✅ **Follow semantic versioning**
- MAJOR.MINOR.PATCH (0.2.0)
- Breaking changes → MAJOR
- New features → MINOR
- Bug fixes → PATCH

### DON'T

❌ **Edit version files manually**
```bash
# Wrong:
vim Cargo.toml  # Change version directly

# Right:
./scripts/sync-version.sh 0.2.0
```

❌ **Hardcode version strings**
```rust
// Wrong:
const VERSION: &str = "0.1.5";

// Right:
const VERSION: &str = env!("CARGO_PKG_VERSION");
```

❌ **Skip verification**
```bash
# Wrong:
git commit -m "bump version"  # Without checking

# Right:
./scripts/sync-version.sh --check && git commit -m "bump version"
```

❌ **Edit generated files**
```bash
# Wrong:
vim keyrx_ui/src/version.ts  # Manual edit

# Right:
cd keyrx_ui && npm run prebuild  # Regenerate
```

❌ **Use inconsistent version formats**
```bash
# Wrong:
v0.1.5     # In some places
0.1.5      # In others
0.1.5.0    # In others

# Right (automated):
./scripts/sync-version.sh 0.1.5
# Sets appropriate format for each file
```

### Version Bump Checklist

Before releasing a new version:

- [ ] All tests pass: `make verify`
- [ ] Clean working directory: `git status`
- [ ] Version updated: `./scripts/sync-version.sh X.Y.Z`
- [ ] Version verified: `./scripts/sync-version.sh --check`
- [ ] Clean rebuild: `cargo clean && cargo build --release`
- [ ] Installer built: `./scripts/build_windows_installer.ps1`
- [ ] Pre-install check: `.\scripts\installer-health-check.ps1 -PreInstall`
- [ ] Test install: Install MSI
- [ ] Post-install check: `.\scripts\installer-health-check.ps1 -PostInstall`
- [ ] Committed: `git commit -m "chore: bump version to X.Y.Z"`
- [ ] Tagged: `git tag vX.Y.Z`
- [ ] Pushed: `git push origin main --tags`
- [ ] Release created: GitHub Releases

## Related Documentation

- **[SSOT_VERSION.md](../SSOT_VERSION.md)** - Original SSOT documentation
- **[troubleshooting-installer.md](troubleshooting-installer.md)** - Comprehensive troubleshooting
- **[.spec-workflow/specs/installer-debuggability-enhancement/README.md](../.spec-workflow/specs/installer-debuggability-enhancement/README.md)** - Complete spec
- **[VERSION_MANAGEMENT.md](VERSION_MANAGEMENT.md)** - Previous version management docs (superseded by this file)

## Support

For issues or questions:

1. Check [troubleshooting-installer.md](troubleshooting-installer.md)
2. Run diagnostics: `.\scripts\diagnose-installation.ps1`
3. Open GitHub issue with diagnostic output
