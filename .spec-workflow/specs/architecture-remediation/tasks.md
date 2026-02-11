# Architecture Remediation - Tasks

## Phase 1: Critical Security Fixes (10 days)

### Task 1.1: Replace unwrap() in daemon core
- [ ] 1.1.1 Audit keyrx_daemon/src/daemon/*.rs for unwrap usage
  - Find all unwrap(), expect(), panic!() calls
  - Categorize by file and function
  - Create replacement plan

- [ ] 1.1.2 Replace unwrap in event_loop.rs
  - Replace with proper Result propagation
  - Add error logging with context
  - Update tests to verify error handling

- [ ] 1.1.3 Replace unwrap in main.rs initialization
  - Create InitError type for startup failures
  - Graceful shutdown on init errors
  - User-friendly error messages

- [ ] 1.1.4 Replace unwrap in service initialization
  - ProfileManager, ConfigService, VirtualKeyboard
  - Propagate errors to caller
  - Add integration tests for error paths

### Task 1.2: Implement dependency injection traits
- [ ] 1.2.1 Create EnvProvider trait
  - Abstract environment variable access
  - Mock implementation for tests
  - Refactor direct env::var() calls

- [ ] 1.2.2 Create FileSystem trait
  - Abstract file operations (read, write, exists)
  - Mock implementation for tests
  - Refactor direct fs:: calls

- [ ] 1.2.3 Inject dependencies in services
  - Update ProfileManager to accept EnvProvider
  - Update ConfigService to accept FileSystem
  - Update all tests to use mocks

### Task 1.3: Add file size validation
- [ ] 1.3.1 Implement upload size limits
  - Max 10 MB for config files
  - Max 1 MB for Rhai scripts
  - Return 413 Payload Too Large

- [ ] 1.3.2 Add decompression bomb protection
  - Limit expanded size
  - Timeout for decompression
  - Tests for malicious archives

## Phase 2: SSOT & Type Safety (10 days)

### Task 2.1: Implement TypeShare
- [ ] 2.1.1 Add typeshare to backend
  - Add dependency to Cargo.toml
  - Annotate DeviceEntry with #[typeshare]
  - Annotate ProfileMetadata with #[typeshare]
  - Annotate all RPC types

- [ ] 2.1.2 Generate TypeScript types
  - Run typeshare CLI
  - Output to keyrx_ui/src/types/generated.ts
  - Add to build process

- [ ] 2.1.3 Replace frontend duplicates
  - Delete keyrx_ui/src/types/index.ts duplicates
  - Import from generated.ts
  - Update all imports
  - Verify TypeScript compilation

### Task 2.2: Consolidate configuration
- [ ] 2.2.1 Create central config
  - Create config/constants.ts with all constants
  - PORT, WS_PORT, API_BASE_URL, etc.
  - Environment-based overrides

- [ ] 2.2.2 Replace hardcoded values
  - Search for "9867", "localhost", magic numbers
  - Replace with config imports
  - Verify no hardcoded values remain

- [ ] 2.2.3 Backend config consolidation
  - Create config.rs with lazy_static CONFIG
  - PORT, BIND_ADDRESS, LOG_LEVEL from env
  - Replace hardcoded values

### Task 2.3: Standardize error handling
- [ ] 2.3.1 Create error hierarchy (frontend)
  - utils/errors.ts with custom error classes
  - ApiError, ValidationError, NetworkError
  - Error codes and i18n support

- [ ] 2.3.2 Create structured logger
  - utils/logger.ts with log levels
  - JSON format: timestamp, level, service, event, context
  - Replace all console.* calls

- [ ] 2.3.3 Remove silent catch blocks
  - Find all empty catch {} blocks
  - Add proper error handling
  - Show errors to user (toast/modal)

- [ ] 2.3.4 Backend error standardization
  - Custom Error enum with error codes
  - Structured logging (tracing)
  - Remove all eprintln! except CLI

### Task 2.4: Consolidate validation
- [ ] 2.4.1 Create validation module (backend)
  - validation/profile.rs - validate_profile_name
  - validation/device.rs - validate_device_id
  - validation/path.rs - validate_safe_path

- [ ] 2.4.2 Deduplicate validation calls
  - Replace inline validation with module functions
  - Remove redundant validation
  - Add validation tests

## Phase 3: SOLID Refactoring (10 days)

### Task 3.1: Create ServiceContainer
- [ ] 3.1.1 Design container interface
  - ServiceContainer struct with trait objects
  - register_service, get_service methods
  - Lifetime management

- [ ] 3.1.2 Implement container
  - Container with Arc<dyn Trait> services
  - ProfileService, ConfigService, PlatformService
  - Inject Clock, EnvProvider, FileSystem

- [ ] 3.1.3 Refactor main.rs to use container
  - Create container in main()
  - Register all services
  - Pass container to daemon
  - Reduce main.rs from 1,995 → ~200 lines

### Task 3.2: Split main.rs modules
- [ ] 3.2.1 Extract CLI dispatcher
  - cli/dispatcher.rs - route commands
  - cli/handlers/ - one file per command
  - Move 400+ lines

- [ ] 3.2.2 Extract daemon factory
  - daemon/factory.rs - build daemon with container
  - daemon/platform_setup.rs - platform initialization
  - Move 300+ lines

- [ ] 3.2.3 Extract web server factory
  - web/server_factory.rs - create Axum server
  - web/router.rs - route configuration
  - Move 200+ lines

- [ ] 3.2.4 Extract shutdown handler
  - shutdown.rs - graceful shutdown logic
  - Signal handling
  - Move 100+ lines

### Task 3.3: Refactor ProfileManager
- [ ] 3.3.1 Extract ProfileRepository
  - File I/O only (load, save, delete)
  - Inject FileSystem trait
  - 200-300 lines

- [ ] 3.3.2 Extract ProfileCompiler service
  - Compilation logic only
  - Inject compiler dependencies
  - 200-300 lines

- [ ] 3.3.3 Keep ProfileManager focused
  - Orchestration only
  - Delegate to repository and compiler
  - 200-300 lines remaining

### Task 3.4: Split large test files
- [ ] 3.4.1 Split e2e_harness.rs (1,919 lines)
  - harness/mod.rs - core harness
  - harness/device.rs - device helpers
  - harness/profile.rs - profile helpers
  - harness/assertions.rs - test assertions

- [ ] 3.4.2 Split virtual_e2e_tests.rs (1,265 lines)
  - tests/virtual/basic.rs - basic scenarios
  - tests/virtual/layers.rs - layer tests
  - tests/virtual/macros.rs - macro tests

## Phase 4: KISS Improvements (7 days)

### Task 4.1: Split keyDefinitions.ts (2,064 lines)
- [ ] 4.1.1 Create category modules
  - keys/letters.ts (A-Z)
  - keys/numbers.ts (0-9)
  - keys/function.ts (F1-F24)
  - keys/modifiers.ts (Shift, Ctrl, Alt, Meta)
  - keys/navigation.ts (Arrows, Home, End)
  - keys/special.ts (Tab, Enter, Escape)

- [ ] 4.1.2 Create index with re-exports
  - keys/index.ts - export all categories
  - Type-safe key categories
  - ~200 lines total

### Task 4.2: Fix SLAP violations
- [ ] 4.2.1 Extract event loop helpers
  - format_output_description(events)
  - log_mapping_result(input, output)
  - handle_timeout_scenario()

- [ ] 4.2.2 Refactor CLI config handler
  - Layered architecture
  - parse_and_validate → service.execute → serialize_output
  - Each layer in separate function

- [ ] 4.2.3 Split state.rs (1,225 lines)
  - state/core.rs - ExtendedState
  - state/layer.rs - layer management
  - state/condition.rs - condition evaluation

### Task 4.3: Remove over-engineering
- [ ] 4.3.1 Remove TapHoldConfigBuilder
  - Replace with Default impl
  - Direct struct construction
  - Save ~50 lines

- [ ] 4.3.2 Simplify const generics
  - Remove unused const generic parameters
  - Use simple constants where applicable

### Task 4.4: Reduce complexity
- [ ] 4.4.1 Reduce run_event_loop complexity (18 → <10)
  - Extract timeout handling
  - Extract error logging
  - Extract state updates

- [ ] 4.4.2 Reduce execute_inner complexity (22 → <10)
  - Extract validation layer
  - Extract business logic layer
  - Extract serialization layer

## Phase 5: Verification & Documentation (3 days)

### Task 5.1: Run all quality checks
- [ ] 5.1.1 Backend verification
  - cargo test --workspace
  - cargo clippy --workspace -- -D warnings
  - cargo fmt --check
  - scripts/verify_file_sizes.sh

- [ ] 5.1.2 Frontend verification
  - npm test (100% pass rate)
  - npm run test:coverage (≥80%)
  - npm run build (no warnings)

- [ ] 5.1.3 Security verification
  - No unwrap() in production paths
  - No hardcoded secrets
  - All inputs validated

### Task 5.2: Update documentation
- [ ] 5.2.1 Update CLAUDE.md
  - Document ServiceContainer pattern
  - Document TypeShare integration
  - Document new validation module

- [ ] 5.2.2 Create architecture diagrams
  - Service dependency graph
  - Module organization chart
  - Error handling flow

- [ ] 5.2.3 Update developer guide
  - How to add new services
  - How to use DI container
  - How to handle errors properly

### Task 5.3: Final audit
- [ ] 5.3.1 Re-run SOLID audit
  - Verify A+ grade (95%+)
  - No critical violations

- [ ] 5.3.2 Re-run KISS/SLAP audit
  - Verify no files >500 lines
  - Verify cyclomatic complexity <10

- [ ] 5.3.3 Re-run Security audit
  - Verify A grade (95%+)
  - Zero unwraps

- [ ] 5.3.4 Re-run SSOT audit
  - Verify zero duplication
  - Single source for all types

## Summary

**Total Tasks:** 80+ subtasks across 5 phases
**Estimated Effort:** 40 days (single developer) or 20 days (2 developers)
**Target Grade:** A+ (95%) across all categories
