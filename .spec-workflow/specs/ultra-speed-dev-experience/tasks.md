# Ultra-Speed Dev Experience - Tasks

- [x] 1. Enhanced Analysis Options
  - File: ui/analysis_options.yaml, ui/scripts/check_metrics.dart
  - Create comprehensive analysis_options.yaml with strict lint rules, custom linter rules for file size (500 lines max) and function size (50 lines max), and code metrics checker script
  - Purpose: Establish strict code quality standards matching Rust standards
  - _Leverage: existing Rust quality standards from CLAUDE.md_
  - _Requirements: FR5.1, FR5.5, FR5.6, NFR3.2, NFR3.4, NFR3.5_
  - _Prompt: Implement the task for spec ultra-speed-dev-experience, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter/Dart Developer specializing in code quality and linting | Task: Create comprehensive analysis_options.yaml with strict lint rules, add custom linter rules for file size (500 lines max) and function size (50 lines max), create check_metrics.dart script, test on existing codebase and fix violations | Restrictions: Must match Rust quality standards from CLAUDE.md, do not lower quality thresholds, ensure cross-platform compatibility | Success: analysis_options.yaml enforces strict rules, metrics checker detects violations, existing codebase passes or has documented exceptions | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts (files, functions, integrations)_

- [x] 2. Pre-Commit Hooks for Dart
  - File: .githooks/pre-commit, ui/scripts/check_coverage.sh
  - Update pre-commit hooks to include Dart format, analyze, test, and coverage checks with 60% initial threshold (target 80%)
  - Purpose: Prevent broken code from entering repository, match Rust pre-commit quality gates
  - _Leverage: existing .githooks/pre-commit for Rust patterns_
  - _Requirements: FR2.1, FR2.2, FR5.4, NFR1.3, NFR3.1_
  - _Prompt: Implement the task for spec ultra-speed-dev-experience, first run spec-workflow-guide to get the workflow guide then implement the task: Role: DevOps Engineer with expertise in Git hooks and CI/CD | Task: Update .githooks/pre-commit to include dart format --set-exit-if-changed, flutter analyze --fatal-infos, flutter test --coverage (fast tests only), create check_coverage.sh script with configurable thresholds, test locally, document skip option | Restrictions: Pre-commit must complete in <30s, do not include slow integration tests, ensure cross-platform compatibility | Success: Pre-commit hooks run Dart checks, coverage threshold enforced, hooks complete in <30s, --no-verify option documented | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts (apiEndpoints, components, functions, classes, integrations)_

- [x] 3. Coverage Tracking Infrastructure
  - File: ui/pubspec.yaml, ui/test/, justfile
  - Install coverage and lcov packages, configure coverage collection, create HTML report generator, add ui-coverage recipe, document workflow
  - Purpose: Enable 80% coverage tracking matching Rust standards
  - _Leverage: existing Rust coverage workflow patterns_
  - _Requirements: FR2.2, NFR3.1_
  - _Prompt: Implement the task for spec ultra-speed-dev-experience, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer with expertise in test coverage and reporting | Task: Install coverage and lcov packages in pubspec.yaml, configure coverage collection in test suite, create HTML coverage report generator, add coverage report to .gitignore, create just ui-coverage recipe, document coverage workflow in docs/TESTING.md | Restrictions: Must match Rust coverage standards (80% minimum, 90% critical paths), reports should be human-readable, ensure cross-platform compatibility | Success: Coverage collection works, HTML reports generated, just ui-coverage recipe functional, workflow documented | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [x] 4. Mock FFI Bridge
  - File: ui/lib/ffi/mock_bridge.dart, ui/lib/ffi/bridge_provider.dart
  - Create MockKeyrxBridge with all interface methods, realistic mock data, configurable latency simulation, MOCK_FFI flag support
  - Purpose: Enable UI development without Rust dependencies, accelerate UI iteration
  - _Leverage: existing ui/lib/ffi/bridge.dart interface_
  - _Requirements: FR1.4, NFR1.1_
  - _Prompt: Implement the task for spec ultra-speed-dev-experience, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer with expertise in FFI and mocking | Task: Create ui/lib/ffi/mock_bridge.dart, implement MockKeyrxBridge class with all interface methods from bridge.dart, add realistic mock data for devices/profiles/runtime config, add configurable latency simulation, update bridgeProvider to support MOCK_FFI flag, test app with flutter run --dart-define=MOCK_FFI=true, document usage | Restrictions: Must implement complete interface, mock data should be realistic, do not break existing FFI integration | Success: Mock bridge implements all methods, app runs with MOCK_FFI=true, mock data is realistic, latency simulation configurable, usage documented in docs/DEVELOPMENT.md | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [x] 5. Justfile Recipes for Dart
  - File: justfile
  - Add comprehensive Dart workflow recipes: ui-setup, ui-test, ui-test-watch, ui-coverage, ui-analyze, ui-format, ui-metrics, ui-quality, dev-mock, update quality recipe
  - Purpose: Standardize Dart development workflows, match Rust tooling convenience
  - _Leverage: existing justfile recipes for Rust_
  - _Requirements: FR5.3, NFR2.1_
  - _Prompt: Implement the task for spec ultra-speed-dev-experience, first run spec-workflow-guide to get the workflow guide then implement the task: Role: DevOps Engineer with expertise in build automation and developer tooling | Task: Add justfile recipes: ui-setup (pub get + build_runner), ui-test (run tests), ui-test-watch (watch mode), ui-coverage (HTML report), ui-analyze (linting), ui-format (formatting check), ui-metrics (code metrics), ui-quality (all checks), dev-mock (run with mock FFI), update quality recipe to include Dart checks | Restrictions: Recipes must be cross-platform, follow existing justfile patterns, ensure fast execution | Success: All recipes work correctly, quality recipe includes Dart checks, recipes follow existing patterns | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [x] 6. Riverpod Integration
  - File: ui/pubspec.yaml, ui/lib/state/providers_setup.dart, ui/lib/main.dart
  - Add flutter_riverpod, riverpod_annotation, riverpod_generator dependencies, create global providers setup, migrate bridgeProvider, create example KeyFlow provider, update main.dart with ProviderScope
  - Purpose: Standardize state management pattern, enable code generation for boilerplate
  - _Leverage: existing provider patterns in ui/lib/_
  - _Requirements: FR3.1, FR3.2, FR3.3_
  - _Prompt: Implement the task for spec ultra-speed-dev-experience, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer with expertise in Riverpod and state management | Task: Add flutter_riverpod, riverpod_annotation to dependencies and riverpod_generator, build_runner to dev_dependencies in pubspec.yaml, create ui/lib/state/providers_setup.dart for global providers, migrate bridgeProvider to Riverpod, create example provider for KeyFlow state, update main.dart to use ProviderScope, run build_runner for code generation, document Riverpod patterns in docs/STATE_MANAGEMENT.md | Restrictions: Must maintain existing functionality, follow Riverpod best practices, ensure type safety | Success: Riverpod configured, bridgeProvider migrated, example provider created, ProviderScope in main.dart, build_runner generates code, patterns documented | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [x] 7. Widget Test Harness
  - File: ui/test/helpers/widget_test_harness.dart, ui/test/helpers/widget_tester_extensions.dart
  - Create WidgetTestHarness class with provider overrides, pumpWidget/pumpPage helpers, WidgetTesterX extension with common assertions, refactor existing tests
  - Purpose: Accelerate widget testing with reusable test utilities
  - _Leverage: existing widget tests in ui/test/_
  - _Requirements: FR2.3, NFR3.1_
  - _Prompt: Implement the task for spec ultra-speed-dev-experience, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer with expertise in Flutter widget testing and test utilities | Task: Create ui/test/helpers/widget_test_harness.dart, implement WidgetTestHarness class with provider overrides and pumpWidget/pumpPage helpers, create WidgetTesterX extension with common assertions, add example widget test using harness, refactor existing widget tests to use harness, document test patterns in docs/TESTING.md | Restrictions: Must work with Riverpod, ensure test isolation, do not break existing tests | Success: WidgetTestHarness created with provider overrides, helper methods work correctly, extension adds useful assertions, example test passes, existing tests refactored, patterns documented | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [x] 8. Golden Test Infrastructure
  - File: ui/pubspec.yaml, ui/test/helpers/golden_test_harness.dart, ui/flutter_test_config.dart, justfile
  - Add golden_toolkit dependency, create golden test harness, configure setup (fonts, devices), add example golden test, create ui-golden-update and ui-golden-verify recipes
  - Purpose: Enable visual regression testing for UI components
  - _Leverage: widget test harness from task 7_
  - _Requirements: FR2.4, NFR3.1_
  - _Prompt: Implement the task for spec ultra-speed-dev-experience, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer with expertise in Flutter golden tests and visual regression testing | Task: Add golden_toolkit to pubspec.yaml, create ui/test/helpers/golden_test_harness.dart, configure golden test setup (fonts, devices) in flutter_test_config.dart, add example golden test for one widget, create just ui-golden-update and ui-golden-verify recipes, optionally add golden files to git LFS if needed, document golden test workflow in docs/TESTING.md | Restrictions: Golden tests must be deterministic, ensure cross-platform compatibility, keep golden file sizes reasonable | Success: golden_toolkit configured, example golden test passes, justfile recipes work, golden files properly versioned, workflow documented | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [x] 9. Structured Logging for Dart
  - File: ui/lib/utils/logger.dart, ui/pubspec.yaml
  - Add logging package, create StructuredLogger class with JSON format, getLogger factory, configure log levels for dev/prod, add example usage
  - Purpose: Match Rust structured logging standards, improve debugging
  - _Leverage: Rust logging patterns from core/src/_
  - _Requirements: FR6.2, NFR2.2_
  - _Prompt: Implement the task for spec ultra-speed-dev-experience, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Backend Developer with expertise in structured logging and observability | Task: Add logging package to pubspec.yaml, create ui/lib/utils/logger.dart, implement StructuredLogger class with JSON format (timestamp, level, service, event, context), add getLogger factory function, configure log levels for dev/prod environments, add example usage in one service, optionally create log aggregation viewer, document logging patterns in docs/LOGGING.md | Restrictions: Must match Rust JSON log format, do not log secrets/PII, ensure performance efficiency | Success: StructuredLogger outputs JSON logs, log levels configurable, example usage works, patterns documented | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [x] 10. FFI Watch Mode
  - File: scripts/watch_ffi.sh, justfile
  - Install watchexec, create watch_ffi.sh script for core/src/ffi/*.rs, configure debouncing (500ms), integrate with just codegen, add dev-watch recipe
  - Purpose: Auto-regenerate FFI bindings on Rust changes, reduce feedback loop
  - _Leverage: existing just codegen command_
  - _Requirements: FR1.2, NFR1.2_
  - _Prompt: Implement the task for spec ultra-speed-dev-experience, first run spec-workflow-guide to get the workflow guide then implement the task: Role: DevOps Engineer with expertise in file watchers and automation | Task: Document watchexec installation in setup docs, create scripts/watch_ffi.sh script, configure file watchers for core/src/ffi/*.rs, add debouncing (500ms) to avoid excessive rebuilds, integrate with just codegen command, test watch mode with live Rust changes, add dev-watch recipe to justfile, document watch mode workflow in docs/DEVELOPMENT.md | Restrictions: Watch mode must be efficient (debounced), do not watch unnecessary files, ensure cross-platform compatibility | Success: watchexec installed, watch_ffi.sh works, debouncing prevents excessive rebuilds, dev-watch recipe functional, workflow documented | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [ ] 11. FFI Debugging Tools
  - File: ui/lib/ffi/debug_tools.dart, docs/FFI_DEBUGGING.md
  - Create FFI call tracing utility, add Timeline instrumentation, create FFI error formatter, add logging for FFI boundary crossings, document common errors
  - Purpose: Accelerate FFI debugging, reduce debugging time from 2h to <30min
  - _Leverage: StructuredLogger from task 9_
  - _Requirements: FR1.3, FR6.4, FR6.5, NFR2.2_
  - _Prompt: Implement the task for spec ultra-speed-dev-experience, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Systems Developer with expertise in FFI debugging and cross-language integration | Task: Create ui/lib/ffi/debug_tools.dart with FFI call tracing utility, add Timeline instrumentation for FFI calls, create FFI error formatter for user-friendly messages, add logging for all FFI boundary crossings using StructuredLogger, create docs/FFI_DEBUGGING.md guide, document common FFI errors and solutions | Restrictions: Tracing must have minimal performance impact, do not expose sensitive data in logs, ensure error messages are actionable | Success: FFI tracing works, Timeline shows FFI calls, error formatter provides clear messages, debugging guide complete with common errors | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [ ] 12. Rust Rebuild Optimization
  - File: core/Cargo.toml, docs/BUILD_OPTIMIZATION.md
  - Enable incremental compilation, configure sccache for build caching, optimize FFI module structure, measure rebuild times
  - Purpose: Reduce Rust rebuild time from 30-60s to <5s
  - _Leverage: existing Cargo.toml configuration_
  - _Requirements: NFR1.2_
  - _Prompt: Implement the task for spec ultra-speed-dev-experience, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with expertise in build optimization and caching | Task: Enable incremental compilation in core/Cargo.toml, configure sccache for build caching, optimize FFI module structure for faster rebuilds, measure rebuild time before/after optimizations, document caching setup in docs/BUILD_OPTIMIZATION.md | Restrictions: Must not break existing builds, ensure cross-platform compatibility, cache configuration should be documented | Success: Incremental compilation enabled, sccache configured, rebuild time <5s with warm cache, optimizations measured and documented | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [ ] 13. Flutter DevTools Integration
  - File: docs/PROFILING.md
  - Document Flutter DevTools setup, create profiling workflow guide, add performance timeline instrumentation, create example performance test
  - Purpose: Enable comprehensive performance profiling and debugging
  - _Leverage: existing Flutter DevTools_
  - _Requirements: FR6.1, FR6.3_
  - _Prompt: Implement the task for spec ultra-speed-dev-experience, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Performance Engineer with expertise in DevTools and profiling | Task: Document Flutter DevTools setup process, create profiling workflow guide covering CPU/memory/network profiling, add performance timeline instrumentation to critical paths, create example performance test, document profiling workflow in docs/PROFILING.md | Restrictions: Documentation must be beginner-friendly, instrumentation should have minimal performance impact, examples should be realistic | Success: DevTools setup documented, profiling workflow guide complete, timeline instrumentation added, example performance test works | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [ ] 14. Performance Benchmarking
  - File: ui/test/performance/, docs/PROFILING.md
  - Create baseline performance tests, add widget render time benchmarks, add FFI call latency benchmarks, set regression thresholds
  - Purpose: Prevent performance regressions, ensure NFR1.5 targets are met
  - _Leverage: FFI debugging tools from task 11_
  - _Requirements: NFR1.5_
  - _Prompt: Implement the task for spec ultra-speed-dev-experience, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Performance Engineer with expertise in benchmarking and Flutter performance testing | Task: Create baseline performance tests in ui/test/performance/, add widget render time benchmarks for key components, add FFI call latency benchmarks, set performance regression thresholds (app startup <500ms to interactive), optionally add performance tests to CI, document benchmarking in docs/PROFILING.md | Restrictions: Benchmarks must be reproducible, thresholds should be realistic, avoid flaky tests | Success: Baseline performance tests created, widget and FFI benchmarks work, regression thresholds set, benchmarking documented | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [ ] 15. Memory Profiling
  - File: ui/test/memory/, docs/PROFILING.md
  - Create memory leak detection tests, document heap snapshot workflow, add memory usage metrics
  - Purpose: Detect and prevent memory leaks, ensure app stability
  - _Leverage: Flutter DevTools from task 13_
  - _Requirements: FR6.3_
  - _Prompt: Implement the task for spec ultra-speed-dev-experience, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Performance Engineer with expertise in memory profiling and leak detection | Task: Create memory leak detection tests in ui/test/memory/, document heap snapshot workflow using Flutter DevTools, add memory usage metrics for critical paths, document memory profiling in docs/PROFILING.md | Restrictions: Tests must be reliable, heap snapshots should be analyzed systematically, avoid false positives | Success: Memory leak detection tests created, heap snapshot workflow documented, memory metrics added, profiling documented | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [ ] 16. Design System Document
  - File: .spec-workflow/steering/design.md
  - Create comprehensive design system documentation covering color palette, typography, spacing, component library, Flutter widget patterns, accessibility, dark mode
  - Purpose: Accelerate UI development decisions, ensure consistency
  - _Leverage: existing UI components in ui/lib/widgets/_
  - _Requirements: FR4.1, FR4.2, FR4.3, FR4.4, FR4.5_
  - _Prompt: Implement the task for spec ultra-speed-dev-experience, first run spec-workflow-guide to get the workflow guide then implement the task: Role: UI/UX Designer with expertise in design systems and Flutter theming | Task: Create .spec-workflow/steering/design.md, document color palette and theming system, document typography system (font families, sizes, weights), document spacing and layout guidelines, document component library structure, document Flutter widget patterns and conventions, document accessibility requirements (WCAG 2.1 AA), document dark mode support guidelines, add design decision records (ADRs) for key choices | Restrictions: Must align with existing UI implementation, ensure accessibility compliance, follow Material Design 3 guidelines | Success: design.md is comprehensive, covers all UI patterns, accessibility documented, dark mode guidelines clear, design decisions recorded | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [ ] 17. Technical Documentation Updates
  - File: .spec-workflow/steering/tech.md
  - Update tech.md with comprehensive Dart/Flutter section: architecture patterns, state management guidelines, testing strategy, code quality standards, performance targets, FFI workflow, debugging guidelines
  - Purpose: Document technical standards matching Rust quality
  - _Leverage: existing tech.md Rust sections_
  - _Requirements: FR3.1, FR5.1, NFR2.3, NFR3.1-NFR3.5_
  - _Prompt: Implement the task for spec ultra-speed-dev-experience, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical Lead with expertise in Flutter architecture and documentation | Task: Update .spec-workflow/steering/tech.md with Dart section, add Flutter architecture patterns (layered architecture, dependency injection), add state management guidelines (Riverpod patterns), add testing strategy for Flutter (unit/widget/golden tests), add code quality standards matching Rust (80% coverage, max 500 lines/file, max 50 lines/function), add performance targets for Flutter (hot reload <1s, startup <500ms), add FFI development workflow, add debugging guidelines for Dart and FFI boundary | Restrictions: Must match Rust quality standards, ensure consistency with existing tech.md structure, keep it concise but comprehensive | Success: tech.md has comprehensive Flutter section, architecture documented, quality standards clear, performance targets specified | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [ ] 18. Structure Documentation
  - File: .spec-workflow/steering/structure.md
  - Update structure.md with actual ui/lib/ directory structure, naming conventions, module boundaries, import organization rules, examples
  - Purpose: Document actual codebase structure for onboarding and consistency
  - _Leverage: existing structure.md Rust sections, actual ui/lib/ structure_
  - _Requirements: NFR2.3, NFR4.2_
  - _Prompt: Implement the task for spec ultra-speed-dev-experience, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Software Architect with expertise in code organization and documentation | Task: Update .spec-workflow/steering/structure.md, document actual ui/lib/ directory structure (ffi/, models/, views/, widgets/, state/, utils/, etc.), document naming conventions for Dart files (snake_case for files, PascalCase for classes), document module boundaries and responsibilities, document import organization rules (Dart imports, package imports, relative imports), add examples for each directory showing typical file organization | Restrictions: Must reflect actual implementation, ensure consistency with Dart conventions, keep examples realistic | Success: structure.md documents actual ui/lib/ structure, naming conventions clear, module boundaries defined, import rules specified, examples provided | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [ ] 19. Development Guides
  - File: docs/DEVELOPMENT.md, docs/TESTING.md, docs/STATE_MANAGEMENT.md, docs/FFI_DEBUGGING.md, docs/PROFILING.md, docs/LOGGING.md, docs/BUILD_OPTIMIZATION.md
  - Create comprehensive development guides covering setup/workflows, test patterns/coverage, Riverpod patterns, FFI troubleshooting, performance optimization, structured logging, and faster builds
  - Purpose: Enable fast developer onboarding and reference
  - _Leverage: all previous implementation from tasks 1-15_
  - _Requirements: NFR2.1, NFR2.2, NFR2.3_
  - _Prompt: Implement the task for spec ultra-speed-dev-experience, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical Writer with expertise in developer documentation | Task: Create docs/DEVELOPMENT.md (setup instructions, common workflows, justfile recipes), create docs/TESTING.md (test patterns, widget/golden tests, coverage), create docs/STATE_MANAGEMENT.md (Riverpod patterns, provider setup, best practices), create docs/FFI_DEBUGGING.md (FFI troubleshooting, common errors, debugging tools), create docs/PROFILING.md (performance optimization, DevTools usage, benchmarking), create docs/LOGGING.md (structured logging patterns, log levels), create docs/BUILD_OPTIMIZATION.md (faster builds, caching, incremental compilation) | Restrictions: Must be beginner-friendly, include code examples, ensure accuracy, keep it up-to-date with implementation | Success: All 7 guides created, comprehensive coverage, code examples work, guides are clear and actionable | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [ ] 20. CI Pipeline Updates
  - File: .github/workflows/ci.yml, .github/workflows/flutter-ci.yml
  - Add Dart tests to CI, add coverage reporting, enforce coverage thresholds (80%), add golden test verification, add code metrics check, configure parallel test execution, optimize runtime
  - Purpose: Enforce quality gates in CI matching Rust standards
  - _Leverage: existing CI workflow for Rust_
  - _Requirements: FR2.2, NFR1.4, NFR3.1-NFR3.3_
  - _Prompt: Implement the task for spec ultra-speed-dev-experience, first run spec-workflow-guide to get the workflow guide then implement the task: Role: DevOps Engineer with expertise in GitHub Actions and CI/CD pipelines | Task: Add Dart tests to CI workflow (flutter test --coverage), add coverage reporting to CI (upload to codecov or similar), add coverage threshold enforcement (80% minimum), add golden test verification to CI (ui-golden-verify), add code metrics check to CI (max 500 lines/file, max 50 lines/function), optionally add performance regression tests, configure parallel test execution for speed, optimize CI runtime to <5min total | Restrictions: CI must be reliable, ensure cross-platform testing (Linux/macOS/Windows if needed), keep costs reasonable, fail fast on quality violations | Success: Dart tests run in CI, coverage reported and enforced, golden tests verified, code metrics checked, parallel execution works, CI runtime <5min | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [ ] 21. Quality Gates in CI
  - File: .github/workflows/ci.yml
  - Enforce zero lint warnings, zero type errors, coverage thresholds, code metrics (file/function size), add PR status checks for all gates
  - Purpose: Prevent low-quality code from merging
  - _Leverage: CI pipeline from task 20_
  - _Requirements: NFR3.2, NFR3.3, NFR3.4, NFR3.5_
  - _Prompt: Implement the task for spec ultra-speed-dev-experience, first run spec-workflow-guide to get the workflow guide then implement the task: Role: DevOps Engineer with expertise in CI quality gates and GitHub branch protection | Task: Configure CI to enforce zero lint warnings (flutter analyze --fatal-warnings), enforce zero type errors (dart analyze), enforce coverage thresholds (80% minimum), enforce code metrics (max 500 lines/file, max 50 lines/function), add PR status checks for all gates in GitHub branch protection rules | Restrictions: Quality gates must be strict, ensure fast failure on violations, provide clear error messages | Success: Zero lint warnings enforced, zero type errors enforced, coverage threshold blocks PRs, code metrics enforced, PR status checks configured | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [ ] 22. Setup Automation
  - File: scripts/setup_dev_env.sh, justfile
  - Create setup_dev_env.sh for automated tool installation (watchexec, lcov, etc.), dependency installation, initial build, add just setup recipe
  - Purpose: Enable new developer to start contributing in <10 minutes
  - _Leverage: existing project dependencies and build process_
  - _Requirements: NFR2.1_
  - _Prompt: Implement the task for spec ultra-speed-dev-experience, first run spec-workflow-guide to get the workflow guide then implement the task: Role: DevOps Engineer with expertise in developer environment setup and automation | Task: Create scripts/setup_dev_env.sh that automates tool installation (watchexec, lcov, sccache, etc.), automates dependency installation (cargo fetch, flutter pub get), automates initial build (cargo build, flutter build), test setup script on clean machine or container, create just setup recipe that calls setup script, document setup process in README.md | Restrictions: Script must be cross-platform or have platform-specific variants, must handle errors gracefully, should be idempotent (safe to run multiple times) | Success: setup_dev_env.sh automates full setup, works on clean machine, just setup recipe works, setup documented in README | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [ ] 23. Error Message Improvements
  - File: ui/lib/utils/error_handler.dart, docs/ERROR_HANDLING.md
  - Review common error scenarios, add actionable error messages, add troubleshooting links to docs, create error code registry
  - Purpose: Reduce debugging time with clear, actionable error messages
  - _Leverage: Rust error handling patterns from core/src/error.rs_
  - _Requirements: NFR2.2_
  - _Prompt: Implement the task for spec ultra-speed-dev-experience, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Developer Experience Engineer with expertise in error handling and user-friendly messaging | Task: Review common error scenarios (FFI errors, state errors, validation errors), add actionable error messages with suggestions for fixes, add troubleshooting links to docs in error messages, create error code registry (like Rust error codes E0001, etc.), document error handling patterns in docs/ERROR_HANDLING.md | Restrictions: Error messages must be user-friendly, avoid technical jargon when possible, ensure error codes are unique and meaningful | Success: Common errors have clear messages, error messages suggest fixes, troubleshooting links included, error code registry created, patterns documented | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [ ] 24. IDE Integration
  - File: .vscode/settings.json, .vscode/extensions.json, .vscode/launch.json, docs/IDE_SETUP.md
  - Create VSCode workspace settings, recommended extensions list, launch configurations for debugging, code snippets for common patterns
  - Purpose: Optimize IDE experience for Flutter/Dart development
  - _Leverage: existing VSCode configuration_
  - _Requirements: NFR2.4_
  - _Prompt: Implement the task for spec ultra-speed-dev-experience, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Developer Experience Engineer with expertise in IDE configuration and developer productivity | Task: Create .vscode/settings.json with workspace settings (format on save, organize imports, Dart SDK path, etc.), create .vscode/extensions.json with recommended extensions list (Dart, Flutter, Riverpod snippets, etc.), create .vscode/launch.json with launch configurations for debugging (app, tests, with mock FFI), add code snippets for common patterns (Riverpod providers, widget tests, etc.) to .vscode/snippets.json, document IDE setup in docs/IDE_SETUP.md | Restrictions: Settings should not override user preferences too aggressively, ensure cross-platform compatibility, recommended extensions should be essential only | Success: VSCode settings optimize workflow, recommended extensions enhance productivity, launch configurations work, code snippets available, IDE setup documented | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

## Definition of Done (Per Task)

Each task is complete when:
- ✅ Implementation finished
- ✅ Tests written and passing (if applicable)
- ✅ Coverage ≥80% for new code
- ✅ All quality gates pass (lint, analyze, format)
- ✅ Documentation updated (if public API changed)
- ✅ Implementation logged with log-implementation tool
- ✅ Task marked [x] in tasks.md

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
