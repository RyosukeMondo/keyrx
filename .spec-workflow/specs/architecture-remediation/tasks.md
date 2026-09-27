# Architecture Remediation - Tasks

**E1 verdict summary (2026-09-27):** 35 DONE-ELSEWHERE, 10 STILL-NEEDED, 7 OBSOLETE (52 tasks total). Verified against current code (grep/read, no full builds) by 4 parallel research agents cross-checked against .spec-workflow/RESUME_PLAN.md Phase E1. Genuinely-needed items are promoted to concrete tasks in RESUME_PLAN.md Phase F.

## Phase 1: Critical Security Fixes (10 days)

### Task 1.1: Replace unwrap() in daemon core
- [ ] 1.1.1 Audit keyrx_daemon/src/daemon/*.rs for unwrap usage — **[OBSOLETE]** No unwrap audit doc was ever the real deliverable; today's unwrap/expect footprint in daemon/*.rs is small (mostly poison-lock `.expect()` in shared_state.rs), addressed directly in 1.1.2-1.1.4.
  - Find all unwrap(), expect(), panic!() calls
  - Categorize by file and function
  - Create replacement plan

- [ ] 1.1.2 Replace unwrap in event_loop.rs — **[DONE-ELSEWHERE]** keyrx_daemon/src/daemon/event_loop.rs has zero .unwrap()/.expect()/panic!() (grep clean).
  - Replace with proper Result propagation
  - Add error logging with context
  - Update tests to verify error handling

- [ ] 1.1.3 Replace unwrap in main.rs initialization — **[DONE-ELSEWHERE]** keyrx_daemon/src/main.rs is 172 lines, zero unwrap/expect/panic, delegates to cli::dispatcher; keyrx_daemon/src/error/init.rs provides the Init error type.
  - Create InitError type for startup failures
  - Graceful shutdown on init errors
  - User-friendly error messages

- [ ] 1.1.4 Replace unwrap in service initialization — **[STILL-NEEDED]** keyrx_daemon/src/config/profile_manager.rs has 9 `.expect("...poisoned")`/`.unwrap()` calls in production code (e.g. lines 672,710,741,856,896,929,970,1002); errors aren't propagated to callers.
  - ProfileManager, ConfigService, VirtualKeyboard
  - Propagate errors to caller
  - Add integration tests for error paths

### Task 1.2: Implement dependency injection traits
- [ ] 1.2.1 Create EnvProvider trait — **[DONE-ELSEWHERE]** keyrx_daemon/src/traits/env.rs defines EnvProvider + RealEnvProvider + MockEnvProvider.
  - Abstract environment variable access
  - Mock implementation for tests
  - Refactor direct env::var() calls

- [ ] 1.2.2 Create FileSystem trait — **[DONE-ELSEWHERE]** keyrx_daemon/src/traits/filesystem.rs defines FileSystem + RealFileSystem + MockFileSystem.
  - Abstract file operations (read, write, exists)
  - Mock implementation for tests
  - Refactor direct fs:: calls

- [ ] 1.2.3 Inject dependencies in services — **[STILL-NEEDED]** Neither trait has a consumer outside traits/{env,filesystem,mod}.rs; profile_manager.rs/config_service.rs still call std::env::var directly (~20 sites) instead of an injected EnvProvider/FileSystem — the traits are dead code.
  - Update ProfileManager to accept EnvProvider
  - Update ConfigService to accept FileSystem
  - Update all tests to use mocks

### Task 1.3: Add file size validation
- [ ] 1.3.1 Implement upload size limits — **[DONE-ELSEWHERE]** keyrx_daemon/src/web/middleware/input_validation.rs enforces max_body_size/max_file_upload_size = 10 MB and returns 413 Payload Too Large.
  - Max 10 MB for config files
  - Max 1 MB for Rhai scripts
  - Return 413 Payload Too Large

- [ ] 1.3.2 Add decompression bomb protection — **[OBSOLETE]** No decompression/archive code path exists anywhere in the daemon (no flate2/zip/tar dependency); uploads are plain bodies already capped by 1.3.1, so there is no decompression-bomb surface.
  - Limit expanded size
  - Timeout for decompression
  - Tests for malicious archives

## Phase 2: SSOT & Type Safety (10 days)

### Task 2.1: Implement TypeShare
- [ ] 2.1.1 Add typeshare to backend — **[DONE-ELSEWHERE]** typeshare = "1.0" wired in keyrx_daemon/Cargo.toml; #[typeshare] on ProfileMetadata, DeviceEntry and RPC/event types across 10 files.
  - Add dependency to Cargo.toml
  - Annotate DeviceEntry with #[typeshare]
  - Annotate ProfileMetadata with #[typeshare]
  - Annotate all RPC types

- [ ] 2.1.2 Generate TypeScript types — **[DONE-ELSEWHERE]** keyrx_ui/src/types/generated.ts exists with real generated content (6.5K); `npm run typeshare` documented as the regen step in .claude/CLAUDE.md.
  - Run typeshare CLI
  - Output to keyrx_ui/src/types/generated.ts
  - Add to build process

- [ ] 2.1.3 Replace frontend duplicates — **[DONE-ELSEWHERE]** keyrx_ui/src/types/index.ts's local DeviceEntry/ProfileMetadata (lines 36,51) are deliberately-different, documented post-mapping frontend view-models (different fields than generated.ts:41/145's wire types), not a stale SSOT duplicate; other types in the same file already import from ./generated.
  - Delete keyrx_ui/src/types/index.ts duplicates
  - Import from generated.ts
  - Update all imports
  - Verify TypeScript compilation

### Task 2.2: Consolidate configuration
- [ ] 2.2.1 Create central config — **[DONE-ELSEWHERE]** keyrx_ui/src/config/constants.ts is documented as "the single source of truth" for PORT/URL/etc config values.
  - Create config/constants.ts with all constants
  - PORT, WS_PORT, API_BASE_URL, etc.
  - Environment-based overrides

- [ ] 2.2.2 Replace hardcoded values — **[DONE-ELSEWHERE]** remaining "9867"/"localhost" hits in keyrx_ui/src are confined to config/constants.ts and config/env.ts themselves; no stray magic numbers found elsewhere.
  - Search for "9867", "localhost", magic numbers
  - Replace with config imports
  - Verify no hardcoded values remain

- [ ] 2.2.3 Backend config consolidation — **[DONE-ELSEWHERE]** keyrx_daemon/src/daemon_config.rs is the single backend config source (KEYRX_* env > settings.json > defaults), not literally lazy_static but the consolidation goal is met.
  - Create config.rs with lazy_static CONFIG
  - PORT, BIND_ADDRESS, LOG_LEVEL from env
  - Replace hardcoded values

### Task 2.3: Standardize error handling
- [ ] 2.3.1 Create error hierarchy (frontend) — **[DONE-ELSEWHERE]** keyrx_ui/src/utils/errors.ts exists with custom error classes.
  - utils/errors.ts with custom error classes
  - ApiError, ValidationError, NetworkError
  - Error codes and i18n support

- [ ] 2.3.2 Create structured logger — **[STILL-NEEDED]** keyrx_ui/src/utils/logger.ts exists but 96 raw console.(log|error|warn|info) calls remain in non-test UI source (e.g. ConfigurationPanel.tsx, RhaiSyncEngine.tsx, metricsStore.ts, useSimulation.ts) — logger built but not adopted.
  - utils/logger.ts with log levels
  - JSON format: timestamp, level, service, event, context
  - Replace all console.* calls

- [ ] 2.3.3 Remove silent catch blocks — **[DONE-ELSEWHERE]** no literal empty `catch {}` blocks found anywhere in keyrx_ui/src.
  - Find all empty catch {} blocks
  - Add proper error handling
  - Show errors to user (toast/modal)

- [ ] 2.3.4 Backend error standardization — **[STILL-NEEDED]** DaemonError enum + tracing exist (error/mod.rs, 52 files use tracing), but eprintln! remains outside cli/ in 6 files (config/layout_manager.rs, web/ws_rpc.rs, platform/linux/{mod,input_capture,output_injection}.rs, main.rs).
  - Custom Error enum with error codes
  - Structured logging (tracing)
  - Remove all eprintln! except CLI

### Task 2.4: Consolidate validation
- [ ] 2.4.1 Create validation module (backend) — **[DONE-ELSEWHERE]** keyrx_daemon/src/validation/{mod,path,device,profile_name,sanitization,content}.rs matches the intent (module names differ slightly from the task's proposal).
  - validation/profile.rs - validate_profile_name
  - validation/device.rs - validate_device_id
  - validation/path.rs - validate_safe_path

- [ ] 2.4.2 Deduplicate validation calls — **[DONE-ELSEWHERE]** validation functions are consumed from web/handlers/device.rs, web/api/validation.rs, config/device_registry.rs, web/api/profiles/*, web/handlers/profile.rs — reused, not reinvented per call site.
  - Replace inline validation with module functions
  - Remove redundant validation
  - Add validation tests

## Phase 3: SOLID Refactoring (10 days)

### Task 3.1: Create ServiceContainer
- [ ] 3.1.1 Design container interface — **[DONE-ELSEWHERE]** keyrx_daemon/src/container/mod.rs (356 lines) is a real ServiceContainer, built via ServiceContainerBuilder; uses named accessor methods (profile_service(), device_service(), ...) rather than generic register_service/get_service, but achieves the same DI goal.
  - ServiceContainer struct with trait objects
  - register_service, get_service methods
  - Lifetime management

- [ ] 3.1.2 Implement container — **[DONE-ELSEWHERE]** container/mod.rs holds Arc<ProfileService/DeviceService/ConfigService/SettingsService/SimulationService>; built in cli/handlers/run.rs:53-54.
  - Container with Arc<dyn Trait> services
  - ProfileService, ConfigService, PlatformService
  - Inject Clock, EnvProvider, FileSystem

- [ ] 3.1.3 Refactor main.rs to use container — **[DONE-ELSEWHERE]** main.rs is 172 lines (target ~200 met); the container is built one layer down in cli/handlers/run.rs rather than literally inside fn main(), same net effect.
  - Create container in main()
  - Register all services
  - Pass container to daemon
  - Reduce main.rs from 1,995 → ~200 lines

### Task 3.2: Split main.rs modules
- [ ] 3.2.1 Extract CLI dispatcher — **[DONE-ELSEWHERE]** keyrx_daemon/src/cli/dispatcher.rs (120 lines) + cli/handlers/{run,profiles,record,validate,list_devices}.rs.
  - cli/dispatcher.rs - route commands
  - cli/handlers/ - one file per command
  - Move 400+ lines

- [ ] 3.2.2 Extract daemon factory — **[DONE-ELSEWHERE]** daemon/factory.rs was deleted; its role is now split across cli/handlers/run.rs (container build) + daemon/platform_runners/{linux,windows}.rs + daemon/platform_setup.rs.
  - daemon/factory.rs - build daemon with container
  - daemon/platform_setup.rs - platform initialization
  - Move 300+ lines

- [ ] 3.2.3 Extract web server factory — **[DONE-ELSEWHERE]** web/mod.rs has build_cors_layer() (~224), a router-builder fn (~269), create_app_with_config (~295); file is already under the 500-code-line gate, no forced router.rs split needed.
  - web/server_factory.rs - create Axum server
  - web/router.rs - route configuration
  - Move 200+ lines

- [ ] 3.2.4 Extract shutdown handler — **[DONE-ELSEWHERE]** keyrx_daemon/src/daemon/signals/linux.rs (290 lines): SIGTERM/SIGINT graceful stop + SIGHUP reload via install_signal_handlers/SignalHandler/ReloadState.
  - shutdown.rs - graceful shutdown logic
  - Signal handling
  - Move 100+ lines

### Task 3.3: Refactor ProfileManager
- [ ] 3.3.1 Extract ProfileRepository — **[STILL-NEEDED]** keyrx_daemon/src/config/profile_manager.rs (712 code lines, over the 500-line gate) still does file I/O itself (scan_profiles/create/list/load_active_profile/clear_active_profile_file); no ProfileRepository type exists.
  - File I/O only (load, save, delete)
  - Inject FileSystem trait
  - 200-300 lines

- [ ] 3.3.2 Extract ProfileCompiler service — **[DONE-ELSEWHERE]** keyrx_daemon/src/config/profile_compiler.rs (272 lines) handles compilation; profile_manager.rs::compile_and_reload delegates to self.compiler.compile_profile(...).
  - Compilation logic only
  - Inject compiler dependencies
  - 200-300 lines

- [ ] 3.3.3 Keep ProfileManager focused — **[STILL-NEEDED]** same root cause as 3.3.1: profile_manager.rs is 712 lines (target 200-300) because file I/O was never extracted to a repository.
  - Orchestration only
  - Delegate to repository and compiler
  - 200-300 lines remaining

### Task 3.4: Split large test files
- [ ] 3.4.1 Split e2e_harness.rs (1,919 lines) — **[STILL-NEEDED]** keyrx_daemon/tests/e2e_harness.rs is still 3386 lines and is the LIVE module (`mod e2e_harness;` in virtual_e2e_test.rs:33, tap_hold_e2e_test.rs, e2e_windows_basic_test.rs, e2e_windows_bugs_test.rs); a split tree at tests/harness/{mod,harness,error,config,assertions}.rs exists with equivalent content (commit 90834f9d) but is referenced nowhere — dead scaffolding, split never wired in.
  - harness/mod.rs - core harness
  - harness/device.rs - device helpers
  - harness/profile.rs - profile helpers
  - harness/assertions.rs - test assertions

- [ ] 3.4.2 Split virtual_e2e_tests.rs (1,265 lines) — **[STILL-NEEDED]** keyrx_daemon/tests/virtual_e2e_test.rs is still 1904 lines and live (its own #[test] fns); tests/virtual/{mod,basic,complex,layers,passthrough,advanced_output,advanced_sequences}.rs (same commit 90834f9d) is never referenced by any `mod virtual;` — same dead-scaffolding pattern as 3.4.1.
  - tests/virtual/basic.rs - basic scenarios
  - tests/virtual/layers.rs - layer tests
  - tests/virtual/macros.rs - macro tests

## Phase 4: KISS Improvements (7 days)

### Task 4.1: Split keyDefinitions.ts (2,064 lines)
- [ ] 4.1.1 Create category modules — **[DONE-ELSEWHERE]** keyrx_ui/src/data/keys/{letters,numbers,function,modifiers,navigation,special,media,types}.ts already split by category.
  - keys/letters.ts (A-Z)
  - keys/numbers.ts (0-9)
  - keys/function.ts (F1-F24)
  - keys/modifiers.ts (Shift, Ctrl, Alt, Meta)
  - keys/navigation.ts (Arrows, Home, End)
  - keys/special.ts (Tab, Enter, Escape)

- [ ] 4.1.2 Create index with re-exports — **[DONE-ELSEWHERE]** keyrx_ui/src/data/keys/index.ts re-exports all categories.
  - keys/index.ts - export all categories
  - Type-safe key categories
  - ~200 lines total

### Task 4.2: Fix SLAP violations
- [ ] 4.2.1 Extract event loop helpers — **[DONE-ELSEWHERE]** daemon/event_loop.rs::run_event_loop (~76 lines) delegates to process_input_event/log_reload_error/EventLoopStats/release_held_outputs.
  - format_output_description(events)
  - log_mapping_result(input, output)
  - handle_timeout_scenario()

- [ ] 4.2.2 Refactor CLI config handler — **[DONE-ELSEWHERE]** cli/config/mod.rs::execute_inner (45 lines) is a pure dispatch match; each arm calls a handlers::handle_* backed by ProfileService.
  - Layered architecture
  - parse_and_validate → service.execute → serialize_output
  - Each layer in separate function

- [ ] 4.2.3 Split state.rs (1,225 lines) — **[DONE-ELSEWHERE]** keyrx_core/src/runtime/state/{mod.rs,core.rs,condition.rs} + tests/ subdir already matches the prescribed split (layer management folded into core/condition rather than a separate layer.rs).
  - state/core.rs - ExtendedState
  - state/layer.rs - layer management
  - state/condition.rs - condition evaluation

### Task 4.3: Remove over-engineering
- [ ] 4.3.1 Remove TapHoldConfigBuilder — **[DONE-ELSEWHERE]** no TapHoldConfigBuilder exists anywhere (0 grep matches); tap_hold/types.rs:69 TapHoldConfig is a plain struct built directly.
  - Replace with Default impl
  - Direct struct construction
  - Save ~50 lines

- [ ] 4.3.2 Simplify const generics — **[OBSOLETE]** the only const generics found (PendingKeyRegistry<const N>, TapHoldProcessor<const N>) have defaults and no caller instantiates non-default N — a deliberate fixed-capacity design, not leftover unused generality.
  - Remove unused const generic parameters
  - Use simple constants where applicable

### Task 4.4: Reduce complexity
- [ ] 4.4.1 Reduce run_event_loop complexity (18 → <10) — **[DONE-ELSEWHERE]** run_event_loop is a ~76-line single loop delegating to named helpers (same evidence as 4.2.1).
  - Extract timeout handling
  - Extract error logging
  - Extract state updates

- [ ] 4.4.2 Reduce execute_inner complexity (22 → <10) — **[DONE-ELSEWHERE]** cli/config/mod.rs::execute_inner and cli/devices.rs::execute_inner are flat ~40-line matches, one delegate call per arm.
  - Extract validation layer
  - Extract business logic layer
  - Extract serialization layer

## Phase 5: Verification & Documentation (3 days)

### Task 5.1: Run all quality checks
- [ ] 5.1.1 Backend verification — **[OBSOLETE]** not a code deliverable; RESUME_PLAN.md Phase G already tracks live cargo test/clippy results as the current source of truth (Linux run 2026-09-27: 2020 pass/4 fail/91 ignored).
  - cargo test --workspace
  - cargo clippy --workspace -- -D warnings
  - cargo fmt --check
  - scripts/verify_file_sizes.sh

- [ ] 5.1.2 Frontend verification — **[OBSOLETE]** same reasoning; .claude/CLAUDE.md Quality Gates table (dated 2026-09-27) is the live tracker (~1,874 TS cases/92 files).
  - npm test (100% pass rate)
  - npm run test:coverage (≥80%)
  - npm run build (no warnings)

- [ ] 5.1.3 Security verification — **[DONE-ELSEWHERE]** every .unwrap() remaining in keyrx_daemon/src/daemon/*.rs is inside a #[cfg(test)] module (shared_state.rs, metrics.rs, event_broadcaster.rs); no hardcoded secret literals found in daemon or ui src.
  - No unwrap() in production paths
  - No hardcoded secrets
  - All inputs validated

### Task 5.2: Update documentation
- [ ] 5.2.1 Update CLAUDE.md — **[DONE-ELSEWHERE]** .claude/CLAUDE.md documents the DaemonQueryService/typeshare SSOT pattern; doesn't literally name "ServiceContainer" but container/mod.rs carries its own rustdoc.
  - Document ServiceContainer pattern
  - Document TypeShare integration
  - Document new validation module

- [ ] 5.2.2 Create architecture diagrams — **[STILL-NEEDED]** no architecture-diagram files exist anywhere in the repo (documentation-only, low priority).
  - Service dependency graph
  - Module organization chart
  - Error handling flow

- [ ] 5.2.3 Update developer guide — **[DONE-ELSEWHERE]** .claude/CLAUDE.md's "Common Tasks"/"Architecture Patterns"/"Dependency Injection" sections already cover this ground without a literal "how to use DI container" heading.
  - How to add new services
  - How to use DI container
  - How to handle errors properly

### Task 5.3: Final audit
- [ ] 5.3.1 Re-run SOLID audit — **[OBSOLETE]** grade-based SOLID audit was a one-off tool output (comprehensive-architecture-refactoring/audit-report.md); no such grading tool exists in this repo today to re-run.
  - Verify A+ grade (95%+)
  - No critical violations

- [ ] 5.3.2 Re-run KISS/SLAP audit — **[STILL-NEEDED]** "no files >500 lines" is not yet true: config/profile_manager.rs (712), daemon/platform_runners/windows.rs (537, delegated to a separate Windows session per RESUME_PLAN G3/G5), config/simulation_engine.rs (529) all exceed the gate.
  - Verify no files >500 lines
  - Verify cyclomatic complexity <10

- [ ] 5.3.3 Re-run Security audit — **[DONE-ELSEWHERE]** same evidence as 5.1.3; all remaining unwrap() calls in daemon/*.rs are test-only.
  - Verify A grade (95%+)
  - Zero unwraps

- [ ] 5.3.4 Re-run SSOT audit — **[OBSOLETE]** same reasoning as 5.3.1; no SSOT grading tool exists — real SSOT enforcement today (typeshare, generated.ts, DaemonQueryService) is already verified true directly.
  - Verify zero duplication
  - Single source for all types

## Summary

**Total Tasks:** 80+ subtasks across 5 phases
**Estimated Effort:** 40 days (single developer) or 20 days (2 developers)
**Target Grade:** A+ (95%) across all categories
