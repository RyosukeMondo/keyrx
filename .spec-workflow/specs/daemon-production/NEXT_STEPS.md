# Phase 3.3 - Next Steps

## Quick Reference for Completing main.rs Refactoring

### Current Status
- ✅ Foundation complete (60% of Phase 3.2)
- ⏸️ Platform runners pending (40% remaining)
- 📊 main.rs: 1,995 lines → Target: <200 lines

### Priority Tasks

## 1. Extract Linux Platform Runner (2-3 hours)

**File:** `keyrx_daemon/src/daemon/platform_runners/linux.rs`

### Extract from main.rs:

#### A. Test Mode Handler (lines 434-586)
```rust
fn run_test_mode(config_path: &Path, debug: bool) -> Result<(), (i32, String)> {
    // IPC server setup
    // Service container creation
    // Web server startup
    // Macro recorder event loop
}
```

**Key Components:**
- ProfileManager initialization
- IPC server (Unix socket)
- Service container (use `ServiceContainerBuilder`)
- Web server (use `WebServerFactory`)
- Tokio runtime creation

#### B. Production Mode Handler (lines 589-812)
```rust
fn run_daemon(config_path: &Path, debug: bool, test_mode: bool) -> Result<(), (i32, String)> {
    // Platform creation (use platform_setup::initialize_platform)
    // Daemon creation (use DaemonFactory)
    // System tray setup (optional)
    // Web server + event broadcaster
    // Event loop with tray polling
}
```

**Key Components:**
- Platform creation via `platform_setup::initialize_platform()`
- Daemon creation via `DaemonFactory`
- Service container via `ServiceContainerBuilder`
- Web server via `WebServerFactory`
- LinuxSystemTray (optional)
- Event broadcasting setup
- Main event loop with tray polling

**Utilities to Extract:**
- `truncate_string()` - Move to utilities module

---

## 2. Extract Windows Platform Runner (3-4 hours)

**File:** `keyrx_daemon/src/daemon/platform_runners/windows.rs`

### Extract from main.rs:

#### A. Test Mode Handler (lines 869-1023)
```rust
fn run_test_mode(config_path: &Path, debug: bool) -> Result<(), (i32, String)> {
    // Similar to Linux, but:
    // - Named pipe socket (not Unix socket)
    // - Windows-specific paths
}
```

#### B. Helper Functions (lines 814-1051)
```rust
fn ensure_single_instance(config_dir: &Path) -> bool {
    // PID file management
    // Kill existing process
    // Write new PID
}

fn cleanup_pid_file(config_dir: &Path) {
    // Remove PID file on exit
}

fn find_available_port(start_port: u16) -> u16 {
    // Try ports 0-9 from start_port
    // Use TcpListener to check availability
}
```

#### C. Production Mode Handler (lines 1054-1369)
```rust
fn run_daemon(config_path: &Path, debug: bool, test_mode: bool) -> Result<(), (i32, String)> {
    // Single instance enforcement
    // Port finding
    // Platform creation
    // Daemon creation (use DaemonFactory)
    // Service container (use ServiceContainerBuilder)
    // Web server (use WebServerFactory)
    // Tray icon setup (optional)
    // Windows message loop with event processing
}
```

**Key Components:**
- `ensure_single_instance()` before daemon start
- `find_available_port()` for web server
- Settings service for port persistence
- Platform creation via `platform_setup::initialize_platform()`
- Daemon creation via `DaemonFactory`
- Service container via `ServiceContainerBuilder`
- Web server via `WebServerFactory`
- TrayIconController (optional)
- Windows message loop (PeekMessageW, DispatchMessageW)
- Event processing via `daemon.process_one_event()`
- `cleanup_pid_file()` on exit

#### D. Shared Utilities (lines 1372-1400, 359-430)
```rust
fn is_admin() -> bool {
    // Check for admin elevation (Windows)
}

fn open_browser(url: &str) -> Result<()> {
    // Platform-specific browser opening
}

fn show_about_dialog() {
    // MessageBoxW on Windows, log on Linux
}
```

**Move to:** `keyrx_daemon/src/utilities/` (new module)

---

## 3. Refactor main.rs (1 hour)

**Target:** <200 lines (aiming for ~120 lines)

### New Structure

```rust
//! keyrx_daemon - OS-level keyboard remapping daemon

#![cfg_attr(all(target_os = "windows", not(debug_assertions)), windows_subsystem = "windows")]

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process;

mod cli;
mod daemon;
mod services;
// ... other modules

/// KeyRx daemon for OS-level keyboard remapping.
#[derive(Parser)]
#[command(name = "keyrx_daemon")]
#[command(version, about = "OS-level keyboard remapping daemon")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Available subcommands for the daemon.
#[derive(Subcommand)]
enum Commands {
    Run { /* ... */ },
    Devices(cli::devices::DevicesArgs),
    Profiles(cli::profiles::ProfilesArgs),
    Config(cli::config::ConfigArgs),
    Layers(cli::layers::LayersArgs),
    Layouts(cli::layouts::LayoutsArgs),
    Simulate(cli::simulate::SimulateArgs),
    Test(cli::test::TestArgs),
    Status(cli::status::StatusArgs),
    State(cli::state::StateArgs),
    Metrics(cli::metrics::MetricsArgs),
    ListDevices,
    Validate { config: PathBuf },
    Record { output: PathBuf, device: Option<PathBuf> },
}

fn main() {
    let cli = Cli::parse();

    // Convert clap Commands to dispatcher Command enum
    let command = convert_to_dispatcher_command(cli.command);

    // Dispatch to appropriate handler
    match cli::dispatcher::dispatch(command) {
        Ok(()) => process::exit(cli::dispatcher::exit_codes::SUCCESS),
        Err((code, message)) => {
            if !message.is_empty() {
                eprintln!("Error: {}", message);
            }
            process::exit(code);
        }
    }
}

/// Convert clap Commands to dispatcher Command enum.
fn convert_to_dispatcher_command(cmd: Commands) -> cli::dispatcher::Command {
    match cmd {
        Commands::Run { config, debug, test_mode } => {
            cli::dispatcher::Command::Run { config, debug, test_mode }
        }
        Commands::Devices(args) => cli::dispatcher::Command::Devices(args),
        Commands::Profiles(args) => cli::dispatcher::Command::Profiles(args),
        Commands::Config(args) => cli::dispatcher::Command::Config(args),
        Commands::Layers(args) => cli::dispatcher::Command::Layers(args),
        Commands::Layouts(args) => cli::dispatcher::Command::Layouts(args),
        Commands::Simulate(args) => cli::dispatcher::Command::Simulate(args),
        Commands::Test(args) => cli::dispatcher::Command::Test(args),
        Commands::Status(args) => cli::dispatcher::Command::Status(args),
        Commands::State(args) => cli::dispatcher::Command::State(args),
        Commands::Metrics(args) => cli::dispatcher::Command::Metrics(args),
        Commands::ListDevices => cli::dispatcher::Command::ListDevices,
        Commands::Validate { config } => cli::dispatcher::Command::Validate { config },
        Commands::Record { output, device } => {
            cli::dispatcher::Command::Record { output, device }
        }
    }
}
```

**Total:** ~120 lines

### Remove from main.rs
- ❌ All handler functions (moved to cli/handlers/)
- ❌ Profile resolution (moved to handlers/run.rs)
- ❌ Platform-specific run handlers (moved to platform_runners/)
- ❌ Helper functions (moved to utilities/ or platform_setup)
- ❌ exit_codes module (moved to dispatcher)
- ❌ All duplicate service initialization

---

## 4. Create Utilities Module (30 minutes)

**File:** `keyrx_daemon/src/utilities/mod.rs`

```rust
pub mod browser;
pub mod dialogs;

pub use browser::open_browser;
pub use dialogs::show_about_dialog;
```

**File:** `keyrx_daemon/src/utilities/browser.rs`
```rust
/// Opens a URL in the default web browser.
pub fn open_browser(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "linux")]
    { std::process::Command::new("xdg-open").arg(url).spawn()?; }

    #[cfg(target_os = "windows")]
    { std::process::Command::new("cmd").args(&["/c", "start", url]).spawn()?; }

    #[cfg(target_os = "macos")]
    { std::process::Command::new("open").arg(url).spawn()?; }

    Ok(())
}
```

**File:** `keyrx_daemon/src/utilities/dialogs.rs`
```rust
/// Show About dialog with version information.
pub fn show_about_dialog() {
    #[cfg(target_os = "windows")]
    { /* MessageBoxW implementation */ }

    #[cfg(not(target_os = "windows"))]
    {
        log::info!("About KeyRx v{}", crate::version::VERSION);
        log::info!("Build: {}", crate::version::BUILD_DATE);
        log::info!("Commit: {}", crate::version::GIT_HASH);
    }
}
```

---

## 5. Update Tests (1 hour)

### Integration Tests to Update
- `tests/cli_integration.rs` - Use new dispatcher
- `tests/rest_api_comprehensive_e2e_test.rs` - Use ServiceContainer
- `tests/virtual_e2e_tests.rs` - Use ServiceContainer

### New Tests to Add
- `tests/platform_runners/linux_test.rs` (Linux only)
- `tests/platform_runners/windows_test.rs` (Windows only)
- `tests/service_container_test.rs` (both platforms)

---

## Checklist

### Phase 3.3: Extract Platform Runners
- [ ] Extract Linux test mode handler
- [ ] Extract Linux production mode handler
- [ ] Extract Windows test mode handler
- [ ] Extract Windows helper functions
- [ ] Extract Windows production mode handler
- [ ] Create utilities module
- [ ] Move open_browser to utilities
- [ ] Move show_about_dialog to utilities

### Phase 3.4: Refactor main.rs
- [ ] Remove all handler functions
- [ ] Remove profile resolution logic
- [ ] Remove platform-specific code
- [ ] Implement convert_to_dispatcher_command
- [ ] Use cli::dispatcher::dispatch()
- [ ] Verify < 200 lines

### Phase 3.5: Testing
- [ ] Update CLI integration tests
- [ ] Update REST API tests
- [ ] Update virtual E2E tests
- [ ] Add platform runner tests
- [ ] Run full test suite
- [ ] Verify all tests pass

### Phase 3.6: Documentation
- [ ] Update CLAUDE.md with new structure
- [ ] Document module hierarchy
- [ ] Add examples of using factories
- [ ] Document service container usage

---

## Success Criteria

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| main.rs lines | <200 | 1,995 | ⏸️ Pending |
| All modules <500 lines | 100% | 100% | ✅ Achieved |
| Single responsibility | All modules | All modules | ✅ Achieved |
| Test coverage | >80% | >80% | ✅ Maintained |
| All tests pass | 100% | N/A | ⏳ Pending |
| No functionality lost | 100% | 100% | ✅ Preserved |

---

## Estimated Timeline

| Phase | Task | Duration | Status |
|-------|------|----------|--------|
| 3.2 | Foundation (completed) | 4 hours | ✅ Done |
| 3.3 | Linux platform runner | 2-3 hours | ⏸️ Pending |
| 3.3 | Windows platform runner | 3-4 hours | ⏸️ Pending |
| 3.4 | Refactor main.rs | 1 hour | ⏸️ Pending |
| 3.5 | Update tests | 1 hour | ⏸️ Pending |
| 3.6 | Documentation | 30 mins | ⏸️ Pending |
| **Total** | **Phase 3 Complete** | **11-13 hours** | **35% Done** |

---

## References

- **SOLID Audit Report:** `.spec-workflow/reports/SOLID_AUDIT_REPORT.md`
- **Phase 3.2 Summary:** `phase-3.2-completion-summary.md`
- **Implementation Summary:** `PHASE_3.2_IMPLEMENTATION_SUMMARY.md`
- **Original main.rs:** `keyrx_daemon/src/main.rs` (1,995 lines)

---

## Quick Commands

```bash
# Check current main.rs line count (excluding comments/blanks)
grep -v '^\s*$' keyrx_daemon/src/main.rs | grep -v '^\s*//' | wc -l

# Run tests
cargo test --package keyrx_daemon

# Check compilation
cargo check --package keyrx_daemon

# Format code
cargo fmt --package keyrx_daemon

# Clippy lints
cargo clippy --package keyrx_daemon -- -D warnings
```

---

**Next Action:** Start with Linux platform runner extraction (easier than Windows, no message loop complexity).
