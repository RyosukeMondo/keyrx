# Profile Activation Bug Fix

## Issue

Profile activation via the Web UI was not persisting. When users created a profile and activated it via `POST /api/profiles/:name/activate`, the activation appeared to succeed but the profile would not be marked as active in subsequent API calls.

## Root Cause

The REST API endpoints in `keyrx_daemon/src/web/api/profiles.rs` were creating **new `ProfileManager` instances** for each HTTP request instead of using the shared instance from `AppState`'s `ProfileService`.

This meant:
- Profile activation would succeed on one ProfileManager instance
- Subsequent requests (e.g., `GET /api/profiles/active`) would create a fresh ProfileManager that knew nothing about the activation
- The activation was correctly persisted to disk (`~/.config/keyrx/.active`), but each request was reading from and writing to different in-memory states

## Fix Applied

**Changed 9 REST API endpoints to use ProfileService from AppState:**

1. `list_profiles` - Now uses `state.profile_service.list_profiles().await`
2. `create_profile` - Now uses `state.profile_service.create_profile().await`
3. `activate_profile` (production mode) - Now uses `state.profile_service.activate_profile().await`
4. `delete_profile` - Now uses `state.profile_service.delete_profile().await`
5. `duplicate_profile` - Now uses `state.profile_service.duplicate_profile().await`
6. `rename_profile` - Now uses `state.profile_service.rename_profile().await`
7. `get_profile_config` - Now uses `state.profile_service.get_profile_config().await`
8. `set_profile_config` - Now uses `state.profile_service.set_profile_config().await`
9. `validate_profile` - Updated to accept State parameter (uses ProfileManager read-only for validation)

**Key Changes:**
```rust
// BEFORE (BUGGY):
async fn list_profiles() -> Result<Json<ProfilesListResponse>, DaemonError> {
    let config_dir = get_config_dir()?;
    let mut pm = ProfileManager::new(config_dir)?; // Creates new instance!
    // ...
}

// AFTER (FIXED):
async fn list_profiles(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ProfilesListResponse>, DaemonError> {
    let profile_list = state.profile_service.list_profiles().await?; // Uses shared instance
    // ...
}
```

## E2E Test Coverage

Added comprehensive E2E tests in `keyrx_daemon/tests/profile_activation_e2e_test.rs`:

**13 tests covering:**
1. Complete web UI workflow (create → activate → verify)
2. Activation persistence across ProfileManager reloads
3. Switching between profiles
4. Deleting active profile
5. Activating non-existent profile
6. Getting active profile when none active
7. Profile list showing correct active indicators
8. Activation with invalid syntax
9. Activation timing metadata
10. Concurrent activation requests (serialization)

**All tests pass:** ✅ 13 passed; 0 failed

## Testing Instructions

```bash
# Run all profile activation E2E tests
cd keyrx_daemon
cargo test --test profile_activation_e2e_test -- --test-threads=1

# Run specific test
cargo test --test profile_activation_e2e_test test_create_and_activate_profile_via_api -- --test-threads=1

# Run with output
cargo test --test profile_activation_e2e_test -- --test-threads=1 --nocapture
```

## Manual Testing via Web UI

1. **Create profile:**
   ```bash
   POST /api/profiles
   {
     "name": "test-profile",
     "template": "blank"
   }
   ```

2. **Activate profile:**
   ```bash
   POST /api/profiles/test-profile/activate
   {}
   ```

3. **Verify activation:**
   ```bash
   GET /api/profiles/active
   # Should return: {"active_profile": "test-profile"}
   ```

4. **Check profile list:**
   ```bash
   GET /api/profiles
   # test-profile should have "isActive": true
   ```

## Files Modified

- `keyrx_daemon/src/web/api/profiles.rs` - Fixed 9 endpoints to use ProfileService
- `keyrx_daemon/tests/common/test_app.rs` - Added `TestAppClient` for concurrent testing
- `keyrx_daemon/tests/profile_activation_e2e_test.rs` - **NEW** - 13 comprehensive E2E tests

## Impact

**Before:**
- ❌ Profile activation appeared to work but didn't persist
- ❌ Active profile state was lost between requests
- ❌ Web UI couldn't reliably show which profile was active

**After:**
- ✅ Profile activation persists across all API requests
- ✅ Active profile state is consistent and shared across the application
- ✅ Web UI correctly displays active profile indicator
- ✅ Activation survives daemon restarts (persisted to `~/.config/keyrx/.active`)

## Future Improvements

1. Consider making ProfileManager methods return ProfileInfo instead of ProfileMetadata to avoid path reconstruction in API layer
2. Add rate limiting to activation endpoint to prevent abuse
3. Add WebSocket notifications for profile activation events
4. Consider adding activation history/audit log
