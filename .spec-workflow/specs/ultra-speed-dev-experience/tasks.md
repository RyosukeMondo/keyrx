# Ultra-Speed Dev Experience - Tasks

## Phase 1: Foundation (Week 1) - Quality Gates

### 1.1 Enhanced Analysis Options
- [x] 1.1.1 Create comprehensive `ui/analysis_options.yaml` with strict lint rules
- [x] 1.1.2 Add custom linter rules for file size (500 lines max)
- [x] 1.1.3 Add custom linter rules for function size (50 lines max)
- [x] 1.1.4 Create `ui/scripts/check_metrics.dart` for code metrics
- [x] 1.1.5 Test metrics checker on existing codebase
- [x] 1.1.6 Fix any violations found (or add to ignore list temporarily)

### 1.2 Pre-Commit Hooks for Dart
- [ ] 1.2.1 Update `.githooks/pre-commit` to include Dart checks
- [ ] 1.2.2 Add `dart format --set-exit-if-changed` check
- [ ] 1.2.3 Add `flutter analyze --fatal-infos` check
- [ ] 1.2.4 Add `flutter test --coverage` check (fast tests only)
- [ ] 1.2.5 Create `ui/scripts/check_coverage.sh` script
- [ ] 1.2.6 Configure coverage threshold (start at 60%, target 80%)
- [ ] 1.2.7 Test pre-commit hooks locally
- [ ] 1.2.8 Document how to skip hooks for WIP commits (`--no-verify`)

### 1.3 Coverage Tracking Infrastructure
- [ ] 1.3.1 Install `coverage` and `lcov` packages
- [ ] 1.3.2 Configure coverage collection in test suite
- [ ] 1.3.3 Create HTML coverage report generator
- [ ] 1.3.4 Add coverage report to `.gitignore`
- [ ] 1.3.5 Create `just ui-coverage` recipe
- [ ] 1.3.6 Document coverage workflow in `docs/TESTING.md`

### 1.4 Mock FFI Bridge
- [ ] 1.4.1 Create `ui/lib/ffi/mock_bridge.dart`
- [ ] 1.4.2 Implement `MockKeyrxBridge` class with all interface methods
- [ ] 1.4.3 Add realistic mock data for devices, profiles, runtime config
- [ ] 1.4.4 Add configurable latency simulation
- [ ] 1.4.5 Update `bridgeProvider` to support `MOCK_FFI` flag
- [ ] 1.4.6 Test app with mock bridge (`flutter run --dart-define=MOCK_FFI=true`)
- [ ] 1.4.7 Document mock bridge usage in `docs/DEVELOPMENT.md`

### 1.5 Justfile Recipes
- [ ] 1.5.1 Add `ui-setup` recipe (pub get + build_runner)
- [ ] 1.5.2 Add `ui-test` recipe (run tests)
- [ ] 1.5.3 Add `ui-test-watch` recipe (watch mode)
- [ ] 1.5.4 Add `ui-coverage` recipe (generate HTML report)
- [ ] 1.5.5 Add `ui-analyze` recipe (linting)
- [ ] 1.5.6 Add `ui-format` recipe (formatting check)
- [ ] 1.5.7 Add `ui-metrics` recipe (code metrics)
- [ ] 1.5.8 Add `ui-quality` recipe (all checks)
- [ ] 1.5.9 Add `dev-mock` recipe (run with mock FFI)
- [ ] 1.5.10 Update `quality` recipe to include Dart checks

## Phase 2: State Management & Testing (Week 2)

### 2.1 Riverpod Integration
- [ ] 2.1.1 Add `flutter_riverpod` and `riverpod_annotation` dependencies
- [ ] 2.1.2 Add `riverpod_generator` and `build_runner` to dev dependencies
- [ ] 2.1.3 Create `ui/lib/state/providers_setup.dart` for global providers
- [ ] 2.1.4 Migrate `bridgeProvider` to Riverpod
- [ ] 2.1.5 Create example provider for KeyFlow state
- [ ] 2.1.6 Update `main.dart` to use `ProviderScope`
- [ ] 2.1.7 Run `build_runner` for code generation
- [ ] 2.1.8 Document Riverpod patterns in `docs/STATE_MANAGEMENT.md`

### 2.2 Widget Test Harness
- [ ] 2.2.1 Create `ui/test/helpers/widget_test_harness.dart`
- [ ] 2.2.2 Implement `WidgetTestHarness` class with provider overrides
- [ ] 2.2.3 Add `pumpWidget` and `pumpPage` helpers
- [ ] 2.2.4 Create `WidgetTesterX` extension with common assertions
- [ ] 2.2.5 Add example widget test using harness
- [ ] 2.2.6 Refactor existing widget tests to use harness
- [ ] 2.2.7 Document test patterns in `docs/TESTING.md`

### 2.3 Golden Test Infrastructure
- [ ] 2.3.1 Add `golden_toolkit` dependency
- [ ] 2.3.2 Create `ui/test/helpers/golden_test_harness.dart`
- [ ] 2.3.3 Configure golden test setup (fonts, devices)
- [ ] 2.3.4 Create `flutter_test_config.dart` for test executable
- [ ] 2.3.5 Add example golden test for one widget
- [ ] 2.3.6 Create `just ui-golden-update` recipe
- [ ] 2.3.7 Create `just ui-golden-verify` recipe
- [ ] 2.3.8 Add golden files to git LFS (if needed)
- [ ] 2.3.9 Document golden test workflow in `docs/TESTING.md`

### 2.4 Structured Logging (Dart)
- [ ] 2.4.1 Add `logging` package dependency
- [ ] 2.4.2 Create `ui/lib/utils/logger.dart`
- [ ] 2.4.3 Implement `StructuredLogger` class (JSON format)
- [ ] 2.4.4 Add `getLogger` factory function
- [ ] 2.4.5 Configure log levels for dev/prod
- [ ] 2.4.6 Add example usage in one service
- [ ] 2.4.7 Create log aggregation viewer (optional)
- [ ] 2.4.8 Document logging patterns in `docs/LOGGING.md`

## Phase 3: FFI Optimization (Week 3)

### 3.1 FFI Watch Mode
- [ ] 3.1.1 Install `watchexec` tool (document in setup)
- [ ] 3.1.2 Create `scripts/watch_ffi.sh` script
- [ ] 3.1.3 Configure file watchers for `core/src/ffi/*.rs`
- [ ] 3.1.4 Add debouncing (500ms) to avoid excessive rebuilds
- [ ] 3.1.5 Integrate with `just codegen` command
- [ ] 3.1.6 Test watch mode with live Rust changes
- [ ] 3.1.7 Add `dev-watch` recipe to justfile
- [ ] 3.1.8 Document watch mode workflow in `docs/DEVELOPMENT.md`

### 3.2 FFI Debugging Tools
- [ ] 3.2.1 Create FFI call tracing utility
- [ ] 3.2.2 Add Timeline instrumentation for FFI calls
- [ ] 3.2.3 Create FFI error formatter (user-friendly messages)
- [ ] 3.2.4 Add logging for all FFI boundary crossings
- [ ] 3.2.5 Create debugging guide in `docs/FFI_DEBUGGING.md`
- [ ] 3.2.6 Document common FFI errors and solutions

### 3.3 Rust Rebuild Optimization
- [ ] 3.3.1 Enable incremental compilation in Cargo.toml
- [ ] 3.3.2 Configure sccache for build caching
- [ ] 3.3.3 Optimize FFI module structure for faster rebuilds
- [ ] 3.3.4 Measure rebuild time before/after optimizations
- [ ] 3.3.5 Document caching setup in `docs/BUILD_OPTIMIZATION.md`

## Phase 4: Performance & Observability (Week 4)

### 4.1 Flutter DevTools Integration
- [ ] 4.1.1 Document Flutter DevTools setup
- [ ] 4.1.2 Create profiling workflow guide
- [ ] 4.1.3 Add performance timeline instrumentation
- [ ] 4.1.4 Create example performance test
- [ ] 4.1.5 Document profiling workflow in `docs/PROFILING.md`

### 4.2 Performance Benchmarking
- [ ] 4.2.1 Create baseline performance tests
- [ ] 4.2.2 Add widget render time benchmarks
- [ ] 4.2.3 Add FFI call latency benchmarks
- [ ] 4.2.4 Set performance regression thresholds
- [ ] 4.2.5 Add performance tests to CI (optional)
- [ ] 4.2.6 Document benchmarking in `docs/PROFILING.md`

### 4.3 Memory Profiling
- [ ] 4.3.1 Create memory leak detection tests
- [ ] 4.3.2 Document heap snapshot workflow
- [ ] 4.3.3 Add memory usage metrics
- [ ] 4.3.4 Document memory profiling in `docs/PROFILING.md`

## Phase 5: Documentation Updates (Week 4-5)

### 5.1 Create Design System Document
- [ ] 5.1.1 Create `.spec-workflow/steering/design.md`
- [ ] 5.1.2 Document color palette and theming
- [ ] 5.1.3 Document typography system
- [ ] 5.1.4 Document spacing and layout guidelines
- [ ] 5.1.5 Document component library structure
- [ ] 5.1.6 Document Flutter widget patterns
- [ ] 5.1.7 Document accessibility requirements
- [ ] 5.1.8 Document dark mode support
- [ ] 5.1.9 Add design decision records (ADRs)

### 5.2 Update Technical Documentation
- [ ] 5.2.1 Update `.spec-workflow/steering/tech.md` with Dart section
- [ ] 5.2.2 Add Flutter architecture patterns
- [ ] 5.2.3 Add state management guidelines
- [ ] 5.2.4 Add testing strategy for Flutter
- [ ] 5.2.5 Add code quality standards (matching Rust)
- [ ] 5.2.6 Add performance targets for Flutter
- [ ] 5.2.7 Add FFI development workflow
- [ ] 5.2.8 Add debugging guidelines

### 5.3 Update Structure Documentation
- [ ] 5.3.1 Update `.spec-workflow/steering/structure.md`
- [ ] 5.3.2 Document actual `ui/lib/` directory structure
- [ ] 5.3.3 Document naming conventions for Dart files
- [ ] 5.3.4 Document module boundaries
- [ ] 5.3.5 Document import organization rules
- [ ] 5.3.6 Add examples for each directory

### 5.4 Create Development Guides
- [ ] 5.4.1 Create `docs/DEVELOPMENT.md` (setup, workflows)
- [ ] 5.4.2 Create `docs/TESTING.md` (test patterns, coverage)
- [ ] 5.4.3 Create `docs/STATE_MANAGEMENT.md` (Riverpod patterns)
- [ ] 5.4.4 Create `docs/FFI_DEBUGGING.md` (FFI troubleshooting)
- [ ] 5.4.5 Create `docs/PROFILING.md` (performance optimization)
- [ ] 5.4.6 Create `docs/LOGGING.md` (structured logging)
- [ ] 5.4.7 Create `docs/BUILD_OPTIMIZATION.md` (faster builds)

## Phase 6: CI/CD Integration (Week 5)

### 6.1 Update CI Pipeline
- [ ] 6.1.1 Add Dart tests to CI workflow
- [ ] 6.1.2 Add coverage reporting to CI
- [ ] 6.1.3 Add coverage threshold enforcement (80%)
- [ ] 6.1.4 Add golden test verification to CI
- [ ] 6.1.5 Add code metrics check to CI
- [ ] 6.1.6 Add performance regression tests (optional)
- [ ] 6.1.7 Configure parallel test execution
- [ ] 6.1.8 Optimize CI runtime (<5min total)

### 6.2 Quality Gates in CI
- [ ] 6.2.1 Enforce zero lint warnings
- [ ] 6.2.2 Enforce zero type errors
- [ ] 6.2.3 Enforce coverage thresholds
- [ ] 6.2.4 Enforce code metrics (file/function size)
- [ ] 6.2.5 Add PR status checks for all gates

## Phase 7: Developer Experience Polish (Week 6)

### 7.1 Setup Automation
- [ ] 7.1.1 Create `scripts/setup_dev_env.sh`
- [ ] 7.1.2 Automate tool installation (watchexec, lcov, etc.)
- [ ] 7.1.3 Automate dependency installation
- [ ] 7.1.4 Automate initial build
- [ ] 7.1.5 Test setup on clean machine
- [ ] 7.1.6 Create `just setup` recipe
- [ ] 7.1.7 Document setup in README

### 7.2 Error Message Improvements
- [ ] 7.2.1 Review common error scenarios
- [ ] 7.2.2 Add actionable error messages
- [ ] 7.2.3 Add troubleshooting links to docs
- [ ] 7.2.4 Create error code registry (like Rust)
- [ ] 7.2.5 Document error handling in `docs/ERROR_HANDLING.md`

### 7.3 IDE Integration
- [ ] 7.3.1 Create VSCode workspace settings
- [ ] 7.3.2 Create recommended extensions list
- [ ] 7.3.3 Configure launch configurations
- [ ] 7.3.4 Add code snippets for common patterns
- [ ] 7.3.5 Document IDE setup in `docs/IDE_SETUP.md`

## Definition of Done (Per Task)

Each task is complete when:
- ✅ Implementation finished
- ✅ Tests written and passing (if applicable)
- ✅ Coverage ≥80% for new code
- ✅ All quality gates pass (lint, analyze, format)
- ✅ Documentation updated (if public API changed)
- ✅ Committed with structured message
- ✅ Manually tested (if UI/workflow change)

## Success Metrics

Track these metrics before/after implementation:

| Metric | Baseline | Target | Current |
|--------|----------|--------|---------|
| FFI change feedback | 60s | <10s | - |
| UI-only change | 3-5s | <1s | - |
| New feature impl time | 4-6h | <2h | - |
| FFI debug time | 2h | <30min | - |
| Test feedback | Manual | <30s | - |
| Dart coverage | Unknown | 80% | - |
| Pre-commit time | N/A | <30s | - |

## Dependencies

### Phase 1 → Phase 2
- Phase 2.1 (Riverpod) depends on Phase 1.4 (Mock Bridge)
- Phase 2.2 (Test Harness) depends on Phase 2.1 (Riverpod)

### Phase 2 → Phase 3
- Phase 3.1 (FFI Watch) can run in parallel
- Phase 3.2 (FFI Debugging) depends on Phase 2.4 (Logging)

### All → Phase 5
- Documentation should be written as features are implemented

### Phase 5 → Phase 6
- CI updates require documentation to be complete

## Notes

- Tasks can be implemented autonomously following this order
- Each task should take 1-4 hours max (if longer, break it down)
- Pre-commit hooks should remain <30s (skip slow tests)
- All tools should work cross-platform (Linux, macOS, Windows)
- Mock FFI bridge is critical for UI development velocity
- Riverpod migration can be incremental (new code only initially)
