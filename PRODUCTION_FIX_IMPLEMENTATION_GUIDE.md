# Production Fix Implementation Guide v0.1.4

## Overview

This document provides a step-by-step implementation guide for fixing all blocking I/O operations found during the production readiness audit.

**Status:** ✅ Task #2 Complete (config.rs fixed)
**Next:** Tasks #3-5 (devices.rs, profiles.rs, layouts.rs)

---

## ✅ COMPLETED: Config API (Task #2)

### Files Changed
- `keyrx_daemon/src/web/api/config.rs`

### Endpoints Fixed
1. ✅ GET /api/config - `get_config()`
2. ✅ POST /api/config/key-mappings - `set_key_mapping()`
3. ✅ DELETE /api/config/key-mappings/:id - `delete_key_mapping()`
4. ✅ PUT /api/config - `update_config()`
5. ✅ GET /api/layers - `list_layers()`

### Pattern Applied
```rust
async fn endpoint() -> Result<...> {
    tokio::task::spawn_blocking(move || {
        // ALL blocking operations here:
        // - get_config_dir()
        // - std::fs::write/read()
        // - RhaiGenerator::load/save()

        Ok::<ReturnType, ErrorType>(result)
    })
    .await
    .map_err(|e| ConfigError::ParseError {
        path: std::path::PathBuf::from("config"),
        reason: format!("Task join error: {}", e),
    })?
}
```

### Verification
```bash
cargo check -p keyrx_daemon --lib  # ✅ No errors
```

---

## 🔄 TODO: Devices API (Task #3)

### File: `keyrx_daemon/src/web/api/devices.rs`

### Endpoints to Fix

#### 1. GET /api/devices - `list_devices()` (lines 45-75)
**Current (BROKEN):**
```rust
async fn list_devices(State(state): State<Arc<AppState>>) -> Result<...> {
    let config_dir = get_config_dir()?;  // ❌ BLOCKING
    let registry_path = config_dir.join("devices.json");

    let registry = DeviceRegistry::load(&registry_path)?;  // ❌ BLOCKING
    // ...
}
```

**Fix:**
```rust
async fn list_devices(State(state): State<Arc<AppState>>) -> Result<...> {
    tokio::task::spawn_blocking(move || {
        let config_dir = get_config_dir()?;
        let registry_path = config_dir.join("devices.json");
        let registry = DeviceRegistry::load(&registry_path)?;

        // Return result
        Ok::<Json<Value>, ApiError>(Json(json!({ ... })))
    })
    .await
    .map_err(|e| ApiError::InternalError(format!("Task join error: {}", e)))??
}
```

#### 2. PUT /api/devices/:id/rename - `rename_device()` (lines 94-137)
**Pattern:** Same as list_devices()
**Blocking operations:**
- `get_config_dir()`
- `DeviceRegistry::load()`
- `registry.save()`

#### 3. PUT /api/devices/:id/layout - `set_device_layout()` (lines 146-173)
**Pattern:** Same as list_devices()

#### 4. GET /api/devices/:id/layout - `get_device_layout()` (lines 175-207)
**Pattern:** Same as list_devices()

#### 5. PUT /api/devices/:id/enable - `set_device_enabled()` (lines 209-254)
**Pattern:** Same as list_devices()

#### 6. DELETE /api/devices/:id - `forget_device()` (lines 281-295)
**Pattern:** Same as list_devices()

### Test Command
```bash
cargo check -p keyrx_daemon --lib
cargo test -p keyrx_daemon device_  # Test device endpoints
```

---

## 🔄 TODO: Profiles API (Task #4)

### File: `keyrx_daemon/src/web/api/profiles.rs`

### Endpoints to Fix

#### 1. GET /api/profiles - `list_profiles()` (lines 140-178)
**Current (BROKEN):**
```rust
async fn list_profiles(State(state): State<Arc<AppState>>) -> Result<...> {
    let profile_list = state.profile_service.list_profiles().await?;

    let profiles: Vec<ProfileResponse> = profile_list
        .iter()
        .map(|info| {
            let config_dir = get_config_dir().unwrap_or_else(...);  // ❌ BLOCKING in map!
            // ...
        })
        .collect();
}
```

**Fix:**
```rust
async fn list_profiles(State(state): State<Arc<AppState>>) -> Result<...> {
    let profile_list = state.profile_service.list_profiles().await?;

    // Move get_config_dir() outside the map
    let config_dir = tokio::task::spawn_blocking(|| {
        crate::cli::config_dir::get_config_dir()
    })
    .await
    .map_err(...)??;

    let profiles_dir = config_dir.join("profiles");

    let profiles: Vec<ProfileResponse> = profile_list
        .iter()
        .map(|info| {
            // No blocking operations in map now
            let rhai_path = profiles_dir.join(format!("{}.rhai", info.name));
            let krx_path = profiles_dir.join(format!("{}.krx", info.name));
            ProfileResponse { ... }
        })
        .collect();

    Ok(Json(ProfilesListResponse { profiles }))
}
```

#### 2. POST /api/profiles - `create_profile()` (lines 188-229)
**Blocking:** `get_config_dir()` at line 217
**Fix:** Call `get_config_dir()` in spawn_blocking BEFORE response construction

#### 3. POST /api/profiles/:name/duplicate - `duplicate_profile()` (lines 366-393)
**Blocking:** `get_config_dir()` at line 382
**Fix:** Same pattern

#### 4. PUT /api/profiles/:name/rename - `rename_profile()` (lines 395-432)
**Blocking:** `get_config_dir()` at line 419
**Fix:** Same pattern

#### 5. PUT /api/profiles/:name/validate - `validate_profile_config()` (lines 507-563)
**Blocking:**
- `get_config_dir()` at line 519
- `ProfileManager::new()` at line 520
- `std::fs::remove_file()` at line 539

**Fix:**
```rust
async fn validate_profile_config(State(state): State<Arc<AppState>>, Path(name): Path<String>) -> Result<...> {
    // Validate profile name first (no blocking)
    validate_profile_name(&name)?;

    tokio::task::spawn_blocking(move || {
        let config_dir = get_config_dir()?;
        let pm = ProfileManager::new(config_dir)?;

        let profile = pm.get(&name).ok_or(...)?;

        // Compile to temp file
        let compiler = Compiler::new();
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let temp_krx = std::env::temp_dir().join(format!("{}_{}.krx", name, timestamp));

        let validation_result = compiler.compile_profile(&profile.rhai_path, &temp_krx);

        // Clean up (blocking)
        let _ = std::fs::remove_file(&temp_krx);

        // Return result
        Ok::<Json<Value>, ApiError>(Json(json!({ ... })))
    })
    .await
    .map_err(...)??
}
```

### Test Command
```bash
cargo check -p keyrx_daemon --lib
cargo test -p keyrx_daemon profile_  # Test profile endpoints
```

---

## 🔄 TODO: Layouts API (Task #5)

### File: `keyrx_daemon/src/web/api/layouts.rs`

### Endpoints to Fix

#### 1. GET /api/layouts - `list_layouts()` (lines 18-28)
**Current (BROKEN):**
```rust
async fn list_layouts() -> Result<Json<Value>, ApiError> {
    let config_dir = get_config_dir()?;  // ❌ BLOCKING
    let lm = LayoutManager::new(config_dir.join("layouts"))?;  // ❌ BLOCKING
    // ...
}
```

**Fix:**
```rust
async fn list_layouts() -> Result<Json<Value>, ApiError> {
    tokio::task::spawn_blocking(move || {
        let config_dir = get_config_dir()?;
        let lm = LayoutManager::new(config_dir.join("layouts"))?;

        let layouts = lm.list()?;
        Ok::<Json<Value>, ApiError>(Json(json!({ "layouts": layouts })))
    })
    .await
    .map_err(|e| ApiError::InternalError(format!("Task join error: {}", e)))??
}
```

#### 2. GET /api/layouts/:name - `get_layout()` (lines 31-40)
**Pattern:** Same as list_layouts()

### Test Command
```bash
cargo check -p keyrx_daemon --lib
cargo test -p keyrx_daemon layout_  # Test layout endpoints
```

---

## Implementation Checklist

### Phase 1: Core Fixes (v0.1.4)
- [x] ✅ Fix config.rs (5 endpoints) - **COMPLETE**
- [ ] 🔄 Fix devices.rs (6 endpoints)
- [ ] 🔄 Fix profiles.rs (5 endpoints)
- [ ] 🔄 Fix layouts.rs (2 endpoints)
- [ ] 🔄 Build and verify: `cargo build --release`

### Phase 2: Testing (v0.1.4)
- [ ] Create E2E test suite (Task #6)
- [ ] Test concurrent requests to same endpoint
- [ ] Test concurrent mixed endpoints
- [ ] Test config freeze regression
- [ ] Load test with 100+ concurrent requests

### Phase 3: Advanced Fixes (v0.1.5)
- [ ] Fix DeviceRegistry race conditions (Task #7)
- [ ] Replace unsafe pointer cast in ProfileService
- [ ] Add comprehensive input validation (Task #8)

---

## Build & Test Commands

### Incremental Verification
```bash
# After each file fix
cargo check -p keyrx_daemon --lib

# After all fixes
cargo build --release -p keyrx_daemon

# Run tests
cargo test -p keyrx_daemon --lib

# E2E tests (requires running daemon)
cargo test -p keyrx_daemon test_concurrent_api_requests --ignored
```

### Full Verification
```bash
# Build new installer
.\scripts\build_windows_installer.ps1

# Install and test
.\COMPLETE_REINSTALL.ps1

# Verify:
# 1. Check build date (should be today)
# 2. Test profile activation + config page (should not freeze)
# 3. Test concurrent API requests (should all complete)
```

---

## Performance Expected After Fixes

### Before (v0.1.3)
| Scenario | Time | Issue |
|----------|------|-------|
| 10 concurrent /api/profiles | ~500ms | Sequential (blocked) |
| Profile activation + config page | TIMEOUT | Runtime starvation |
| 100 concurrent requests | Multiple timeouts | Runtime exhaustion |

### After (v0.1.4)
| Scenario | Time | Issue |
|----------|------|-------|
| 10 concurrent /api/profiles | ~100ms | Parallel ✅ |
| Profile activation + config page | ~50ms | Non-blocking ✅ |
| 100 concurrent requests | All complete | Runtime responsive ✅ |

**Improvement:** 5x faster, no timeouts, runtime stays responsive

---

## Error Handling Pattern

All spawn_blocking calls should use this error handling:

```rust
tokio::task::spawn_blocking(move || {
    // Blocking operations
    Ok::<ReturnType, ErrorType>(result)
})
.await
.map_err(|e| ErrorType::InternalError(format!("Task join error: {}", e)))??
//       ^^^ First ? for JoinError (task panic)
//            ^^^ Second ? for operation error
```

---

## Testing Strategy

### 1. Unit Tests (Per-Endpoint)
Test each fixed endpoint individually:
```rust
#[tokio::test]
async fn test_get_config_non_blocking() {
    // Should complete within 100ms
    let start = std::time::Instant::now();
    let result = get_config().await;
    assert!(start.elapsed() < Duration::from_millis(100));
    assert!(result.is_ok());
}
```

### 2. Concurrent Tests
Test multiple concurrent requests:
```rust
#[tokio::test]
async fn test_concurrent_get_profiles() {
    let handles: Vec<_> = (0..50)
        .map(|_| tokio::spawn(async { get_profiles().await }))
        .collect();

    let results = futures::future::join_all(handles).await;
    assert_eq!(results.len(), 50);
    assert!(results.iter().all(|r| r.is_ok()));
}
```

### 3. Mixed Workload Test
Test different endpoints concurrently:
```rust
#[tokio::test]
async fn test_mixed_concurrent_requests() {
    let handles = vec![
        tokio::spawn(get_config()),
        tokio::spawn(list_profiles()),
        tokio::spawn(list_devices()),
        tokio::spawn(list_layouts()),
    ];

    let results = futures::future::join_all(handles).await;
    assert!(results.iter().all(|r| r.is_ok()));
}
```

### 4. Stress Test
Test under high load:
```rust
#[tokio::test]
#[ignore] // Run with: cargo test --ignored
async fn test_api_stress_1000_requests() {
    let start = std::time::Instant::now();

    let handles: Vec<_> = (0..1000)
        .map(|_| tokio::spawn(async { list_profiles().await }))
        .collect();

    let results = futures::future::join_all(handles).await;

    let duration = start.elapsed();
    let success_count = results.iter().filter(|r| r.is_ok()).count();

    println!("1000 requests completed in {:?}", duration);
    println!("Success rate: {}%", (success_count * 100) / 1000);

    assert!(success_count >= 950); // 95% success rate
    assert!(duration < Duration::from_secs(30)); // Complete within 30s
}
```

---

## Rollback Plan

If issues are found after deployment:

1. **Immediate:** Revert to v0.1.3
   ```bash
   git revert HEAD
   .\scripts\build_windows_installer.ps1
   .\COMPLETE_REINSTALL.ps1
   ```

2. **Investigate:** Check daemon logs for spawn_blocking errors
   ```powershell
   Get-Content "C:\Users\$env:USERNAME\.keyrx\daemon.log" | Select-String "spawn_blocking"
   ```

3. **Fix:** Address specific endpoint issues

4. **Retest:** Run full test suite before redeploying

---

## Next Steps

1. ✅ **Complete** - Fix config.rs (Task #2)
2. **In Progress** - Fix devices.rs (Task #3)
3. **Pending** - Fix profiles.rs (Task #4)
4. **Pending** - Fix layouts.rs (Task #5)
5. **Pending** - Create E2E tests (Task #6)
6. **Pending** - Build v0.1.4 installer
7. **Pending** - Test and deploy

---

## Conclusion

The production readiness audit found **25+ blocking operations** across all API endpoints. Task #2 (config.rs) is now complete with all 5 endpoints fixed. Tasks #3-5 require the same pattern applied to devices.rs, profiles.rs, and layouts.rs.

**Estimated time to complete all fixes:** 4-6 hours
**Estimated time for testing:** 2-3 hours
**Total time to v0.1.4 production-ready:** 1 day
