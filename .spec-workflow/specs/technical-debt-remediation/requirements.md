# Requirements Document

## Introduction

This specification addresses critical technical debt identified across recently implemented features (profile-management, visual-config-builder, config-validation-linting, wasm-simulation-integration, macro-recorder). The technical debt analysis revealed systematic violations of project code quality standards defined in CLAUDE.md, including file size limits, missing test coverage, hard-coded dependencies, code duplication, and insufficient error handling.

The purpose of this remediation is to bring the codebase into full compliance with project standards, improve maintainability, enhance testability, and establish patterns that prevent future technical debt accumulation. This work is critical for ensuring the codebase remains manageable as the project scales, particularly given the "AI Coding Agent First" design philosophy that requires machine-verifiable, deterministic behavior.

## Alignment with Product Vision

This technical debt remediation directly supports the product vision outlined in product.md through:

1. **AI-First Verification**: By eliminating hard-coded dependencies and enforcing dependency injection, we enable fully automated testing without manual intervention, aligning with the "AI Coding Agent First" principle.

2. **Code Quality as Infrastructure**: Maintaining strict file/function size limits and test coverage ensures the codebase remains analyzable by AI agents, supporting deterministic verification and automated code analysis.

3. **Single Source of Truth (SSOT)**: Eliminating code duplication reinforces SSOT principles, ensuring configuration and implementation remain synchronized.

4. **Structured Logging**: Improving error handling with structured logging enhances observability for AI agents to diagnose issues programmatically.

5. **Maintainability at Scale**: Reducing file sizes and extracting common utilities supports the long-term vision of extending keyrx to new platforms and features without codebase degradation.

## Requirements

### Requirement 1: File Size Compliance

**User Story:** As a developer (human or AI), I want all source files to comply with the 500-line maximum limit, so that code remains modular, maintainable, and analyzable.

#### Acceptance Criteria

1. WHEN measuring file length (excluding comments and blank lines) THEN keyrx_daemon/src/config/profile_manager.rs SHALL be ≤500 lines
2. WHEN measuring file length THEN keyrx_daemon/src/cli/config.rs SHALL be ≤500 lines
3. WHEN measuring file length THEN keyrx_daemon/src/cli/profiles.rs SHALL be ≤500 lines
4. WHEN measuring file length THEN keyrx_ui/src/components/MacroRecorderPage.tsx SHALL be ≤500 lines
5. WHEN file size reduction is achieved THEN extracted modules SHALL follow Single Responsibility Principle
6. WHEN extracting modules THEN all tests SHALL continue to pass without modification
7. WHEN compilation is complete THEN no new warnings or errors SHALL be introduced

### Requirement 2: Test Coverage Completeness

**User Story:** As a developer, I want comprehensive test coverage for all components, so that changes can be validated automatically without manual testing (supporting AI-first verification).

#### Acceptance Criteria

1. WHEN running test suite THEN ProfileCard.tsx SHALL have a corresponding test file with ≥80% coverage
2. WHEN running test suite THEN ProfileDialog.tsx SHALL have a corresponding test file with ≥80% coverage
3. WHEN running test suite THEN DashboardEventTimeline.tsx SHALL have a corresponding test file with ≥80% coverage
4. WHEN running test suite THEN DashboardPage.tsx SHALL have a corresponding test file with ≥80% coverage
5. WHEN running test suite THEN DeviceList.tsx SHALL have a corresponding test file with ≥80% coverage
6. WHEN running test suite THEN EventTimeline.tsx SHALL have a corresponding test file with ≥80% coverage
7. WHEN running test suite THEN MetricsChart.tsx SHALL have a corresponding test file with ≥80% coverage
8. WHEN running test suite THEN StateIndicatorPanel.tsx SHALL have a corresponding test file with ≥80% coverage
9. WHEN running test suite THEN TemplateLibrary.tsx SHALL have a corresponding test file with ≥80% coverage
10. WHEN all tests are implemented THEN total project coverage SHALL be ≥80% (verified by tarpaulin/coverage tools)
11. WHEN tests run in CI THEN all tests SHALL pass consistently without flakiness

### Requirement 3: Dependency Injection Compliance

**User Story:** As a developer, I want all external dependencies injected rather than hard-coded, so that components are testable, mockable, and follow SOLID principles.

#### Acceptance Criteria

1. WHEN ProfilesPage component renders THEN API base URL SHALL be injected via props or context (not hard-coded to localhost:3030)
2. WHEN ConfigurationPage saves data THEN storage layer SHALL be abstracted behind an interface (not direct localStorage access)
3. WHEN ProfilesPage downloads files THEN browser APIs SHALL be abstracted behind an injectable interface
4. WHEN DeviceList connects to WebSocket THEN connection URLs SHALL be configurable via environment or props
5. WHEN components use injected dependencies THEN unit tests SHALL demonstrate mocking capabilities
6. WHEN injected dependencies are used THEN components SHALL work correctly with different implementations (dev, test, production)

### Requirement 4: Code Duplication Elimination

**User Story:** As a developer, I want shared logic consolidated into reusable modules, so that changes propagate consistently and maintenance is simplified.

#### Acceptance Criteria

1. WHEN JSON output logic exists in CLI modules THEN a common utility module (cli/common.rs) SHALL centralize serialization logic
2. WHEN timestamp formatting is needed THEN a shared utility (utils/timeFormatting.ts) SHALL provide consistent formatting functions
3. WHEN key code mapping is needed THEN a shared utility (utils/keyCodeMapping.ts) SHALL provide consistent mapping logic
4. WHEN common utilities are created THEN they SHALL have ≥90% test coverage
5. WHEN duplicated code is removed THEN all existing functionality SHALL continue to work identically
6. IF duplication is eliminated THEN at least 200+ lines of code SHALL be removed from the codebase

### Requirement 5: Error Handling and Logging Improvements

**User Story:** As a developer or AI agent, I want comprehensive error handling with structured logging, so that failures are observable, debuggable, and machine-parseable.

#### Acceptance Criteria

1. WHEN catch blocks exist THEN they SHALL log errors at appropriate severity levels (no silent failures)
2. WHEN errors occur in DeviceList.tsx THEN reconnection errors SHALL be logged at debug level
3. WHEN errors occur in configBuilderStore.ts THEN user-facing actions SHALL propagate errors to UI for display
4. WHEN errors are logged THEN they SHALL follow structured JSON format (per product.md) with timestamp, level, service, event_type, context
5. IF a component encounters an error THEN users SHALL receive actionable feedback (not just console warnings)
6. WHEN debug logging is added THEN it SHALL not impact production performance

### Requirement 6: Documentation Completeness

**User Story:** As a developer, I want comprehensive module-level documentation, so that code intent is clear and onboarding is accelerated.

#### Acceptance Criteria

1. WHEN ProfileCard.tsx is viewed THEN it SHALL have JSDoc comments describing component purpose, props, and usage
2. WHEN ProfileDialog.tsx is viewed THEN it SHALL have JSDoc comments describing component purpose, props, and usage
3. WHEN TemplateLibrary.tsx is viewed THEN it SHALL have comprehensive header documentation
4. WHEN EventTimeline.tsx is viewed THEN it SHALL have detailed feature documentation beyond the brief description
5. WHEN DeviceList.tsx is viewed THEN it SHALL have detailed documentation of features and integration points
6. WHEN public components/functions exist THEN they SHALL have documentation following TSDoc or rustdoc standards
7. WHEN complex logic exists THEN inline comments SHALL explain "why" (not "what")

### Requirement 7: Outstanding TODOs Resolution

**User Story:** As a product owner, I want all outstanding TODOs resolved or converted to tracked issues, so that incomplete work is visible and prioritized.

#### Acceptance Criteria

1. WHEN ConfigurationPage.tsx:44 TODO exists THEN actual API integration SHALL be implemented OR a GitHub issue SHALL be created
2. WHEN keyrx_daemon/src/web/ws.rs:48 TODO exists THEN WebSocket event streaming SHALL be fully implemented OR a GitHub issue SHALL be created
3. WHEN ProfilesPage.tsx:209 rename functionality exists THEN backend API SHALL implement rename endpoint
4. WHEN all TODOs are addressed THEN code comments SHALL not contain "TODO" markers for production-critical features
5. IF TODOs are converted to issues THEN issues SHALL be labeled "technical-debt" and linked to this spec

## Non-Functional Requirements

### Code Architecture and Modularity

- **Single Responsibility Principle**: All extracted modules (cli/common.rs, utils/timeFormatting.ts, etc.) SHALL have a single, well-defined purpose
- **Modular Design**: Utility modules SHALL be isolated, reusable, and independently testable
- **Dependency Management**: Minimize interdependencies between newly created modules
- **Clear Interfaces**: Define clean contracts for extracted utilities (JSDoc/rustdoc with parameter and return type specifications)
- **No Breaking Changes**: All refactoring SHALL maintain backward compatibility for public APIs unless explicitly approved

### Performance

- **No Performance Degradation**: Refactored code SHALL not introduce measurable performance regressions (verified via criterion benchmarks for Rust, performance tests for TypeScript)
- **Compilation Time**: File size reductions SHALL not increase overall compilation time
- **Test Execution**: New tests SHALL execute in <10 seconds combined (maintain fast feedback loop)

### Security

- **No Security Regressions**: Dependency injection SHALL not introduce new attack surfaces (validate all injected dependencies)
- **Logging Safety**: Structured logging SHALL not log sensitive data (PII, secrets, credentials)
- **Input Validation**: All new utility functions SHALL validate inputs and handle edge cases securely

### Reliability

- **Test Stability**: All new tests SHALL be deterministic and non-flaky (no race conditions, no timing dependencies)
- **Error Recovery**: Improved error handling SHALL enable graceful degradation (no crashes from improved logging)
- **Backward Compatibility**: Refactored modules SHALL maintain existing behavior (verified via regression testing)

### Usability

- **Developer Experience**: Extracted utilities SHALL have clear, well-documented APIs that improve developer productivity
- **Error Messages**: Enhanced error handling SHALL provide actionable error messages with context
- **Code Readability**: Reduced file sizes SHALL improve code navigation and comprehension
