# Requirements Document: code-quality-refactor

## Introduction

This spec addresses code quality issues identified during architectural review, including:
- File/function size violations (>500 lines, >50 lines)
- SSOT violations (keycode definitions repeated 6+ times)
- DRY violations (duplicated patterns across drivers)
- Missing abstractions affecting testability
- SLAP violations (mixed abstraction levels)

The goal is to improve maintainability, testability, and reduce bug risk while **preserving zero-latency performance** critical for keyboard remapping.

## Alignment with Product Vision

From `product.md`:
> "Performance > Features: Latency is the enemy. The tool must feel invisible."

From `tech.md`:
> "Input latency: < 1ms processing overhead (hard requirement)"

This refactoring maintains compile-time code generation to ensure **zero runtime overhead** while achieving single-source-of-truth for keycode definitions.

## Requirements

### REQ-1: Single Source of Truth for Keycodes

**User Story:** As a developer, I want keycode definitions in a single location, so that adding new keys requires changes in only one place.

#### Acceptance Criteria

1. WHEN a new keycode is added THEN the system SHALL require modification of exactly ONE source file
2. WHEN keycodes are defined THEN the system SHALL generate all conversions (evdev, VK, Display, FromStr) at compile time
3. WHEN the build runs THEN the system SHALL produce zero runtime overhead from keycode conversions (compiled match statements)
4. IF a keycode mapping is inconsistent THEN the build SHALL fail with a clear error message

### REQ-2: File Size Compliance

**User Story:** As a maintainer, I want files under 500 lines, so that the codebase remains navigable and reviewable.

#### Acceptance Criteria

1. WHEN refactoring is complete THEN each file SHALL contain fewer than 500 lines (excluding comments/blank lines)
2. WHEN splitting files THEN the system SHALL maintain backward-compatible public API exports
3. WHEN organizing modules THEN related functionality SHALL be grouped in subdirectories

### REQ-3: Function Size Compliance

**User Story:** As a developer, I want functions under 50 lines, so that each function has a single, testable responsibility.

#### Acceptance Criteria

1. WHEN refactoring is complete THEN each function SHALL contain fewer than 50 lines
2. WHEN splitting functions THEN the system SHALL preserve identical behavior (no semantic changes)
3. WHEN extracting helpers THEN they SHALL be reusable across similar patterns

### REQ-4: KeyInjector Trait Abstraction

**User Story:** As a test author, I want to inject mock key emitters, so that I can unit test driver logic without real hardware.

#### Acceptance Criteria

1. WHEN implementing key injection THEN the system SHALL use a `KeyInjector` trait
2. WHEN testing LinuxInput THEN a MockKeyInjector SHALL be injectable without real uinput
3. WHEN testing WindowsInput THEN a MockKeyInjector SHALL be injectable without real SendInput
4. IF the injector fails THEN appropriate errors SHALL propagate to the caller

### REQ-5: Shared Utility Extraction

**User Story:** As a developer, I want common patterns extracted to utilities, so that bug fixes apply everywhere.

#### Acceptance Criteria

1. WHEN handling panics in threads THEN a shared `extract_panic_message()` utility SHALL be used
2. WHEN parsing keys in Rhai functions THEN a shared `parse_key_or_error()` helper SHALL be used
3. WHEN the shared utility is fixed THEN all callers SHALL benefit without modification

### REQ-6: Driver Module Organization

**User Story:** As a contributor, I want driver code organized into submodules, so that I can find and modify specific functionality easily.

#### Acceptance Criteria

1. WHEN organizing Linux driver THEN it SHALL split into: `mod.rs`, `reader.rs`, `writer.rs`, `keymap.rs`
2. WHEN organizing Windows driver THEN it SHALL split into: `mod.rs`, `hook.rs`, `injector.rs`, `keymap.rs`
3. WHEN importing driver functionality THEN existing `use` statements SHALL continue to work

### REQ-7: Zero Performance Regression

**User Story:** As a user, I want refactoring to have zero impact on latency, so that my keyboard remapping remains imperceptible.

#### Acceptance Criteria

1. WHEN benchmarking after refactor THEN latency SHALL NOT increase by more than 100 microseconds
2. WHEN generating keycode conversions THEN they SHALL compile to jump tables (not runtime lookups)
3. WHEN running `cargo bench` THEN all existing benchmarks SHALL pass with equivalent or better results

## Non-Functional Requirements

### Code Architecture and Modularity
- **Single Responsibility Principle**: Each file has one well-defined purpose
- **Modular Design**: Keycodes, drivers, and utilities are isolated and reusable
- **Dependency Management**: No circular dependencies between new modules
- **Clear Interfaces**: `KeyInjector` trait defines clean contract for injection

### Performance
- Zero runtime overhead from refactoring
- Compile-time code generation for keycode mappings
- Match statements preserved for branch prediction optimization
- No HashMap/BTreeMap lookups in hot path

### Testability
- All driver components injectable via traits
- Mock implementations provided for each new trait
- Unit tests achievable without hardware access

### Maintainability
- Adding a new keycode requires exactly 1 file change
- File sizes comply with project KPIs
- Function sizes comply with project KPIs

### Backward Compatibility
- Public API unchanged
- Existing tests pass without modification
- External crate users unaffected
