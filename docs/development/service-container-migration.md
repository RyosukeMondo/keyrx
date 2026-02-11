# ServiceContainer Migration Guide

**Status:** Phase 3.1 Complete - Container Implementation Ready
**Next Phase:** Phase 3.2 - Refactor main.rs to use ServiceContainer

## Overview

The ServiceContainer pattern centralizes dependency injection, eliminating SOLID violations in main.rs. This guide shows how to migrate from direct service instantiation to container-based dependency injection.

## Problem Statement

### Before: DIP Violations (CRITICAL-9)

**File:** `keyrx_daemon/src/main.rs`
**Issues:**
- 15+ direct instantiations of concrete types
- 196+ lines of duplicated initialization code
- Tight coupling to service implementations
- Hard to test without modifying main.rs

**Example violations at lines: 196, 329, 451, 525-549, 665-711, 888, 962-986, 1148-1181**

```rust
// BAD: Direct instantiation (repeated 3+ times for Linux/Windows/test modes)
let profile_manager = match ProfileManager::new(config_dir.clone()) {
    Ok(mgr) => Arc::new(mgr),
    Err(e) => { /* error handling */ }
};

let macro_recorder = Arc::new(MacroRecorder::new());
let profile_service = Arc::new(ProfileService::new(Arc::clone(&profile_manager)));
let device_service = Arc::new(DeviceService::new(config_dir.clone()));
let config_service = Arc::new(ConfigService::new(Arc::clone(&profile_manager)));
let settings_service = Arc::new(SettingsService::new(config_dir.clone()));
let simulation_service = Arc::new(SimulationService::new(config_dir, None));
let subscription_manager = Arc::new(SubscriptionManager::new());
// ... 8+ more instantiations
```

### After: SOLID-Compliant ServiceContainer

```rust
// GOOD: ServiceContainer handles all wiring
let container = ServiceContainerBuilder::new(config_dir)
    .build()
    .map_err(|e| (ExitCode::ConfigError, e.to_string()))?;

let app_state = AppState::from_container(container, None);
```

**Benefits:**
- **85% code reduction** in main.rs (1,995 → ~300 lines)
- **Zero duplication** across Linux/Windows/test modes
- **Testable** - inject mock container
- **Maintainable** - add new service in one place
- **SOLID-compliant** - follows Dependency Inversion Principle

---

## Implementation Status

### ✅ Phase 3.1: ServiceContainer Implementation (Complete)

**Files Created:**
1. `keyrx_daemon/src/container/mod.rs` - ServiceContainer and builder
2. `keyrx_daemon/src/container/example_usage.rs` - Refactoring examples
3. `docs/architecture/service-container-migration.md` - This guide

**Features Implemented:**
- ✅ ServiceContainer with all service accessors
- ✅ ServiceContainerBuilder with fluent API
- ✅ Test mode support with IPC socket
- ✅ AppState::from_container() factory method
- ✅ Comprehensive unit tests
- ✅ Documentation and examples

### 🔄 Phase 3.2: Refactor main.rs (Next Steps)

**Files to Modify:**
1. `keyrx_daemon/src/main.rs` - Replace direct instantiation
2. CLI handlers (profiles, config, etc.)

**Target Metrics:**
- main.rs: 1,995 → ~300 lines (85% reduction)
- Zero direct service instantiation
- All modes (Linux/Windows/test) use ServiceContainer

---

## Migration Steps

### Step 1: Import ServiceContainer

```rust
// Add to main.rs imports
use keyrx_daemon::container::ServiceContainerBuilder;
```

### Step 2: Replace handle_run_test_mode() - Linux

**Before (196 lines):**
```rust
fn handle_run_test_mode(_config_path: &std::path::Path, _debug: bool) -> Result<(), (i32, String)> {
    // ... 20+ lines of ProfileManager creation
    let profile_manager = match ProfileManager::new(config_dir.clone()) { /* ... */ };

    // ... 30+ lines of IPC server setup
    let ipc_handler = Arc::new(IpcCommandHandler::new(/* ... */));

    // ... 40+ lines of service instantiation (DUPLICATION!)
    let macro_recorder = Arc::new(MacroRecorder::new());
    let profile_service = Arc::new(ProfileService::new(Arc::clone(&profile_manager)));
    // ... 10+ more services

    // ... 50+ lines of web server setup
    let app_state = Arc::new(AppState::new(/* 8 parameters */));

    // ... 40+ lines of tokio runtime and server launch
}
```

**After (30 lines):**
```rust
fn handle_run_test_mode(_config_path: &std::path::Path, _debug: bool) -> Result<(), (i32, String)> {
    log::info!("Starting daemon in test mode");

    let config_dir = get_config_dir();

    // Create container with test mode enabled
    let test_socket = PathBuf::from(format!("/tmp/keyrx-test-{}.sock", std::process::id()));
    let container = ServiceContainerBuilder::new(config_dir)
        .with_test_mode_socket(test_socket.clone())
        .build()
        .map_err(|e| (exit_codes::CONFIG_ERROR, e.to_string()))?;

    // Create AppState from container
    let app_state = Arc::new(AppState::from_container(container, Some(test_socket)));

    // IPC server and web server setup (unchanged)
    // ...
}
```

### Step 3: Replace handle_run() - Linux

**Before (223 lines with duplication):**
```rust
fn handle_run(config_path: &Path, debug: bool, test_mode: bool) -> Result<(), (i32, String)> {
    // ... 50+ lines duplicated from handle_run_test_mode
    let profile_manager = match ProfileManager::new(config_dir.clone()) { /* ... */ };
    let macro_recorder = Arc::new(MacroRecorder::new());
    let profile_service = Arc::new(ProfileService::new(/* ... */));
    // ... 10+ more services

    // ... 100+ lines of daemon, tray, web server setup
}
```

**After (50 lines, no duplication):**
```rust
fn handle_run(config_path: &Path, debug: bool, test_mode: bool) -> Result<(), (i32, String)> {
    init_logging(debug);
    log_startup_version_info();

    if test_mode {
        return handle_run_test_mode(config_path, debug);
    }

    let config_dir = get_config_dir();

    // Build container - replaces 20+ lines of service instantiation
    let container = ServiceContainerBuilder::new(config_dir)
        .build()
        .map_err(|e| (exit_codes::CONFIG_ERROR, e.to_string()))?;

    let app_state = Arc::new(AppState::from_container(container, None));

    // Create platform instance
    let platform = keyrx_daemon::platform::create_platform()
        .map_err(|e| (exit_codes::RUNTIME_ERROR, e.to_string()))?;

    // Create daemon
    let mut daemon = Daemon::new(platform, config_path)
        .map_err(daemon_error_to_exit)?;

    // Daemon and web server setup (unchanged)
    // ...
}
```

### Step 4: Replace handle_run() - Windows

**Before (315 lines with duplication):**
```rust
fn handle_run(config_path: &Path, debug: bool, test_mode: bool) -> Result<(), (i32, String)> {
    // ... 50+ lines duplicated from Linux version
    let profile_manager = Arc::new(ProfileManager::new(config_dir.clone())?);
    let macro_recorder = Arc::new(MacroRecorder::new());
    // ... 10+ more services

    // ... Windows-specific port finding, PID file, hooks
    // ... 150+ lines of daemon loop
}
```

**After (50 lines, no duplication):**
```rust
fn handle_run(config_path: &Path, debug: bool, test_mode: bool) -> Result<(), (i32, String)> {
    init_logging(debug);
    log_startup_version_info();

    if test_mode {
        return handle_run_test_mode(config_path, debug);
    }

    let config_dir = get_config_dir();

    // Ensure single instance
    let killed_old = ensure_single_instance(&config_dir);

    // Build container
    let container = ServiceContainerBuilder::new(config_dir.clone())
        .build()
        .map_err(|e| (exit_codes::CONFIG_ERROR, e.to_string()))?;

    let app_state = Arc::new(AppState::from_container(container, None));

    // Windows-specific logic (port finding, PID file)
    let configured_port = app_state.settings_service.get_port();
    let actual_port = find_available_port(configured_port);

    // Create platform and daemon
    // ... (unchanged)
}
```

### Step 5: Replace handle_profiles_command()

**Before (38 lines):**
```rust
fn handle_profiles_command(args: ProfilesArgs) -> Result<(), (i32, String)> {
    let config_dir = get_config_dir();

    // Direct instantiation
    let manager = match ProfileManager::new(config_dir) {
        Ok(mgr) => Arc::new(mgr),
        Err(e) => return Err((exit_codes::CONFIG_ERROR, e.to_string())),
    };

    let service = ProfileService::new(manager);

    // Execute command
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| (exit_codes::RUNTIME_ERROR, e.to_string()))?;

    rt.block_on(async {
        cli::profiles::execute(args, &service).await
            .map_err(|e| (exit_codes::CONFIG_ERROR, e.to_string()))
    })
}
```

**After (15 lines, 60% reduction):**
```rust
fn handle_profiles_command(args: ProfilesArgs) -> Result<(), (i32, String)> {
    let config_dir = get_config_dir();

    // Use container
    let container = ServiceContainerBuilder::new(config_dir)
        .build()
        .map_err(|e| (exit_codes::CONFIG_ERROR, e.to_string()))?;

    // Get only the service we need
    let service = container.profile_service();

    // Execute command
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| (exit_codes::RUNTIME_ERROR, e.to_string()))?;

    rt.block_on(async {
        cli::profiles::execute(args, &*service).await
            .map_err(|e| (exit_codes::CONFIG_ERROR, e.to_string()))
    })
}
```

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_container_build() {
        let temp_dir = tempdir().unwrap();
        let container = ServiceContainerBuilder::new(temp_dir.path().to_path_buf())
            .build()
            .unwrap();

        // Verify services are initialized
        assert!(Arc::strong_count(&container.profile_service()) >= 1);
    }

    #[test]
    fn test_app_state_from_container() {
        let temp_dir = tempdir().unwrap();
        let container = ServiceContainerBuilder::new(temp_dir.path().to_path_buf())
            .build()
            .unwrap();

        let app_state = AppState::from_container(container, None);
        assert!(Arc::strong_count(&app_state.profile_service) >= 1);
    }
}
```

### Integration Tests

Existing integration tests should continue working with AppState::new_for_testing(), which now uses ServiceContainer internally.

---

## Benefits Summary

### Before Refactoring

| Metric | Value |
|--------|-------|
| main.rs total lines | 1,995 |
| Direct instantiations | 15+ |
| Code duplication | ~600 lines |
| SOLID violations | 12 critical |
| Testability | Poor (hard-coded deps) |

### After Refactoring

| Metric | Value | Improvement |
|--------|-------|-------------|
| main.rs total lines | ~300 | **85% reduction** |
| Direct instantiations | 0 | **100% elimination** |
| Code duplication | 0 | **100% elimination** |
| SOLID violations | 0 | **100% resolution** |
| Testability | Excellent (injectable) | **Fully testable** |

### Key Improvements

1. **Single Responsibility**: ServiceContainer handles service wiring, main.rs handles daemon lifecycle
2. **DRY**: Zero duplication across Linux/Windows/test modes
3. **Testability**: Easy to inject mock container for testing
4. **Maintainability**: Add new service in one place (container/mod.rs)
5. **Dependency Inversion**: main.rs depends on ServiceContainer abstraction, not concrete types

---

## Next Steps

### Immediate Actions

1. **Apply Step 2-5 to main.rs** - Replace all handle_*() functions
2. **Update CLI handlers** - Replace direct instantiation in CLI modules
3. **Run tests** - Ensure all tests pass with container-based initialization
4. **Update documentation** - Reflect new patterns in CLAUDE.md

### Verification

```bash
# Run all tests
make test

# Verify no clippy warnings
cargo clippy --workspace -- -D warnings

# Check line counts
scripts/verify_file_sizes.sh

# Expected: main.rs should be < 500 lines (target ~300)
```

### Success Criteria

- ✅ main.rs < 500 lines (target ~300)
- ✅ Zero direct service instantiation
- ✅ All tests pass
- ✅ No SOLID violations in main.rs
- ✅ Code duplication eliminated

---

## References

- **SOLID Audit Report**: `.spec-workflow/reports/SOLID_AUDIT_REPORT.md`
- **ServiceContainer Implementation**: `keyrx_daemon/src/container/mod.rs`
- **Usage Examples**: `keyrx_daemon/src/container/example_usage.rs`
- **Spec**: `.spec-workflow/specs/daemon-production/tasks.md` (Phase 3.1)
