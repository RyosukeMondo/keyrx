# Immediate Fix Guide - Profile Activation Bug

## Problem
Your `default.rhai` (24KB, 276 commands) causes activation timeout. `profile-a` (253 bytes) works fine.

## Quick Fix Options

### Option 1: Run Diagnostic (Recommended First)
**Time**: 2 minutes

```powershell
# Right-click PowerShell → Run as Administrator
cd C:\Users\ryosu\repos\keyrx
.\DEBUG_ACTIVATION.ps1
```

This will:
- Stop daemon cleanly
- Start with logging
- Test both profiles
- Show exact error messages
- Save detailed log

**Output**: `DEBUG_yyyyMMdd_HHmmss.txt` with full diagnostics

### Option 2: Simplify default.rhai
**Time**: 5 minutes

Split your profile into smaller chunks:

```powershell
# Backup original
Copy-Item "C:\Users\ryosu\AppData\Roaming\keyrx\profiles\default.rhai" "default-BACKUP.rhai"

# Create simplified version (first 50 mappings only)
Get-Content "C:\Users\ryosu\AppData\Roaming\keyrx\profiles\default.rhai" | Select-Object -First 80 > "C:\Users\ryosu\AppData\Roaming\keyrx\profiles\default-simple.rhai"
```

Then activate `default-simple` instead of `default`.

### Option 3: Apply Code Fix
**Time**: 15 minutes

Fix the blocking I/O issue in the API handler:

1. Open `keyrx_daemon\src\api\profiles.rs`
2. Find the `activate_profile` function (around line 167)
3. Replace with async version (see below)
4. Rebuild and install

## Code Fix (Option 3 Details)

### Current Code (BLOCKING)
```rust
// keyrx_daemon/src/api/profiles.rs:167
pub async fn activate_profile(
    Path(profile_name): Path<String>,
    State(state): State<SharedState>,
) -> Result<Json<ProfileStatus>, AppError> {
    // This BLOCKS the async runtime! ❌
    let profile_path = state.config_dir.join("profiles").join(&profile_name).with_extension("rhai");

    if !profile_path.exists() {
        return Err(AppError::ProfileNotFound(profile_name));
    }

    // Compilation blocks for complex profiles (can take 30+ seconds)
    let compiler = Compiler::new();
    let krx_data = compiler.compile_file(&profile_path)
        .map_err(|e| AppError::CompilationError(e.to_string()))?;

    // Load config...
}
```

### Fixed Code (NON-BLOCKING) ✅
```rust
pub async fn activate_profile(
    Path(profile_name): Path<String>,
    State(state): State<SharedState>,
) -> Result<Json<ProfileStatus>, AppError> {
    let profile_path = state.config_dir.join("profiles").join(&profile_name).with_extension("rhai");

    if !profile_path.exists() {
        return Err(AppError::ProfileNotFound(profile_name));
    }

    // Move compilation to background thread (tokio spawn_blocking)
    let profile_path_clone = profile_path.clone();
    let krx_data = tokio::task::spawn_blocking(move || {
        let compiler = Compiler::new();
        compiler.compile_file(&profile_path_clone)
    })
    .await
    .map_err(|e| AppError::InternalError(format!("Join error: {}", e)))?
    .map_err(|e| AppError::CompilationError(e.to_string()))?;

    // Rest of the function...
}
```

### Rebuild After Fix
```powershell
# Clean build
cd C:\Users\ryosu\repos\keyrx
.\REBUILD_SSOT.bat
```

## Verify Fix Works

After applying any fix:

```powershell
# 1. Run tests
.\scripts\test_installation.ps1

# 2. Try activating default profile
# Open http://localhost:9867/profiles
# Click "Activate" on default profile
# Should complete within 10-15 seconds

# 3. Check metrics
# Open http://localhost:9867/metrics
# Type some keys
# Should see events

# 4. Analyze all profiles
.\scripts\analyze_profiles.ps1
```

## Test That Bug Is Fixed

Run the new comprehensive test:

```powershell
cd keyrx_daemon
cargo test profile_activation_test --release -- --nocapture
```

This tests:
- Simple profiles (10 remaps) - should be instant
- Complex profiles (500 remaps) - should complete <30s
- Large files (>10KB) - should not timeout
- Performance benchmarks for different sizes

## Expected Behavior After Fix

| Profile | Size | Commands | Activation Time |
|---------|------|----------|-----------------|
| profile-a | 253 bytes | 0 | <1 second |
| default | 24 KB | 276 | 5-15 seconds |
| huge (1000 remaps) | ~50 KB | 1000 | 20-30 seconds |

## If Still Fails

Check daemon stderr:

```powershell
# Look for compilation errors
Get-Content daemon_stderr_*.log
```

Common errors:
- "Parse error" → Syntax issue in .rhai file
- "Timeout" → Need to increase timeout further
- "Out of memory" → Profile too complex, must split

## Prevention (Future)

Add this check to UI:

```typescript
// Before activating, warn user
if (profileSize > 10_000) {
    showWarning("Large profile (${profileSize} bytes) may take 10-30 seconds to compile");
}
```

Add progress indicator:

```typescript
// Show compilation progress
activationStatus = "Compiling (this may take 30 seconds for complex profiles)...";
```

---

**Priority**: P0 (blocks real-world usage)
**ETA**: 15 minutes to implement Option 3
**Testing**: 5 minutes with test suite
