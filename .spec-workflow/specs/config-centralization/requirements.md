# Requirements Document: Configuration Centralization

## Introduction

This specification addresses the elimination of magic numbers and strings scattered throughout the KeyRx codebase by establishing a systematic, centralized configuration architecture. The goal is to consolidate 90+ magic numbers and 50+ magic strings into well-organized, documented configuration modules that are easy to maintain, test, and modify.

## Alignment with Product Vision

This feature supports several key product principles from product.md:

- **Performance > Features**: Centralized timing constants (tap_timeout, combo_window) are critical for the sub-1ms latency guarantee
- **CLI First**: All config values will be CLI-overridable for rapid trial and AI agent development
- **Progressive Complexity**: Users can use defaults (simple) or override specific values (advanced)
- **Testable Configs**: Configuration constants enable deterministic testing without hardcoded values

## Requirements

### Requirement 1: Rust Configuration Module

**User Story:** As a Rust developer, I want all magic numbers and strings centralized in dedicated config modules, so that I can easily find, modify, and understand configuration values.

#### Acceptance Criteria

1. WHEN a developer looks for a timing constant THEN the system SHALL provide it in `core/src/config/timing.rs`
2. WHEN a developer needs key code constants THEN the system SHALL provide them in `core/src/config/keys.rs`
3. WHEN a developer needs path constants THEN the system SHALL provide them in `core/src/config/paths.rs`
4. WHEN a developer needs capacity/threshold limits THEN the system SHALL provide them in `core/src/config/limits.rs`
5. WHEN a developer needs CLI exit codes THEN the system SHALL provide them in `core/src/config/exit_codes.rs`
6. IF a constant is used in multiple places THEN the system SHALL use a single definition from the config module

### Requirement 2: Dart Configuration Module

**User Story:** As a Flutter developer, I want all UI constants and FFI-related strings centralized in config files, so that I can maintain consistency across the UI layer.

#### Acceptance Criteria

1. WHEN a developer needs UI timing constants (animation, debounce) THEN the system SHALL provide them in `ui/lib/config/timing_config.dart`
2. WHEN a developer needs UI dimension constants (padding, elevation) THEN the system SHALL provide them in `ui/lib/config/ui_constants.dart`
3. WHEN a developer needs SharedPreferences keys THEN the system SHALL provide them in `ui/lib/config/storage_keys.dart`
4. WHEN a developer needs FFI function names or JSON keys THEN the system SHALL provide them in `ui/lib/config/ffi_constants.dart`
5. WHEN a developer imports config THEN the system SHALL provide a single barrel export via `ui/lib/config/config.dart`

### Requirement 3: Runtime Configuration File

**User Story:** As a power user, I want to customize timing thresholds and behavior parameters via a config file, so that I can tune KeyRx to my typing style without modifying code.

#### Acceptance Criteria

1. WHEN KeyRx starts THEN it SHALL look for `.keyrx/config.toml` in standard XDG locations
2. IF config file exists THEN the system SHALL load and validate values at startup
3. IF config file is missing THEN the system SHALL use compiled-in defaults
4. WHEN a config value is invalid (out of range, wrong type) THEN the system SHALL log a warning and use the default
5. WHEN CLI arguments are provided THEN they SHALL override config file values
6. IF `--config <path>` is specified THEN the system SHALL use that file instead of default location

### Requirement 4: Configuration Documentation

**User Story:** As a new developer or power user, I want clear documentation of all configuration options, so that I can understand what each value controls.

#### Acceptance Criteria

1. WHEN a constant is defined THEN it SHALL have a doc comment explaining its purpose and valid range
2. WHEN a config file is created THEN it SHALL include inline comments explaining each section
3. IF a developer reads the config module THEN they SHALL find a reference to where each constant is used

### Requirement 5: Backward Compatibility

**User Story:** As an existing user, I want my current setup to continue working after this change, so that I don't need to reconfigure anything.

#### Acceptance Criteria

1. WHEN no config file exists THEN the system SHALL behave identically to pre-change behavior
2. IF existing `.keyrx/quality-gates.toml` exists THEN it SHALL continue to work unchanged
3. WHEN default values are extracted THEN they SHALL match current hardcoded values exactly

## Non-Functional Requirements

### Code Architecture and Modularity
- **Single Responsibility Principle**: Each config module handles one domain (timing, keys, paths, UI)
- **Modular Design**: Config modules are isolated and independently importable
- **Dependency Management**: Config modules have no external dependencies beyond std/dart:core
- **Clear Interfaces**: All constants are public with doc comments

### Performance
- Configuration loading SHALL complete in < 10ms
- Config values SHALL be resolved at startup, not on every access
- No runtime overhead for accessing constants (compile-time resolved where possible)

### Security
- Config file parsing SHALL not execute arbitrary code
- Invalid config values SHALL be bounded to safe ranges
- No sensitive defaults (API keys, passwords) in config modules

### Reliability
- Missing config file SHALL not cause crashes
- Invalid config values SHALL fall back to safe defaults
- Config module compilation errors SHALL be caught at build time

### Usability
- Config constants SHALL have self-documenting names
- Config file format (TOML) SHALL be human-readable
- IDE autocomplete SHALL work for all config values
