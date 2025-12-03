# Requirements Document: Script Validation & Safety

## Introduction

This feature enhances KeyRx's script validation capabilities to catch errors before runtime and protect users from dangerous configurations. Currently, `keyrx check` only validates Rhai syntax, leaving semantic errors (invalid key names, undefined layers, conflicting remaps) to be discovered at runtime. This leads to a poor user experience where scripts that "pass" validation still crash or behave unexpectedly.

The Script Validation & Safety feature will provide comprehensive pre-flight validation, conflict detection, dangerous pattern warnings, and dry-run simulation to ensure users can confidently apply their configurations.

## Alignment with Product Vision

This feature directly supports multiple product principles from product.md:

1. **Safety First**: "Scripts are sandboxed. Community scripts cannot access filesystem, network, or crash the app." - Extends safety to include configuration safety, preventing user lockout.

2. **CLI First, GUI Later**: "Every feature must be exercisable via CLI before GUI implementation" - Enhanced `keyrx check` provides comprehensive CLI validation.

3. **Visual > Abstract**: The dry-run and coverage features make script behavior visible before application.

4. **Testable Configs**: "Users can write tests for their keyboard configurations. Refactor with confidence." - Validation enables confident refactoring.

5. **Trust is the foundation**: From the Safe Mode pillar - "Users must never fear losing control." Dangerous pattern warnings prevent lockout scenarios.

## Requirements

### REQ-1: Semantic Validation in `keyrx check`

**User Story:** As a user writing Rhai scripts, I want `keyrx check` to validate key names, layer references, and function arguments, so that I catch errors before running the engine.

#### Acceptance Criteria

1. WHEN `keyrx check script.rhai` is run THEN the system SHALL validate all key names in `remap()`, `block()`, `pass()`, `tap_hold()`, and `combo()` calls
2. WHEN a script contains an invalid key name THEN the system SHALL report the error with line number, column, the invalid key, and suggest similar valid keys
3. WHEN a script references an undefined layer in `layer_map()`, `layer_push()`, or `layer_toggle()` THEN the system SHALL report the undefined layer name
4. WHEN a script uses an undefined modifier in `modifier_on()`, `modifier_off()`, or `one_shot()` THEN the system SHALL report the undefined modifier name
5. WHEN validation finds multiple errors THEN the system SHALL report all errors (not fail-fast) up to `ValidationConfig.max_errors` limit
6. WHEN `--json` flag is provided THEN the system SHALL output validation results in JSON format for tooling integration

### REQ-2: Conflict Detection

**User Story:** As a user, I want to be warned when my script has conflicting remaps, so that I don't accidentally override mappings I intended to keep.

#### Acceptance Criteria

1. WHEN the same key is remapped multiple times (e.g., `remap("A", "B"); remap("A", "C");`) THEN the system SHALL warn about the conflict and show both mappings
2. WHEN a key is both remapped and blocked THEN the system SHALL warn about the conflict
3. WHEN a tap-hold key conflicts with a simple remap of the same key THEN the system SHALL warn about the conflict
4. WHEN combo keys overlap (e.g., `combo(["A","S"], "X"); combo(["A","S","D"], "Y");`) THEN the system SHALL warn about potential combo shadowing
5. WHEN the same key has different behaviors in the base layer vs a named layer THEN the system SHALL NOT warn (this is intentional layer design)
6. WHEN `--strict` flag is provided THEN the system SHALL treat conflicts as errors (exit code 1) instead of warnings

### REQ-3: Dangerous Pattern Warnings

**User Story:** As a user, I want to be warned about potentially dangerous configurations, so that I don't accidentally lock myself out of my keyboard.

#### Acceptance Criteria

1. WHEN a script remaps or blocks the Escape key THEN the system SHALL warn about potential lockout risk
2. WHEN a script remaps or blocks keys in the emergency exit combo (Ctrl+Alt+Shift+Escape) THEN the system SHALL warn that emergency exit may be affected
3. WHEN a script blocks all modifier keys (both Ctrl keys, both Shift keys, both Alt keys) THEN the system SHALL warn about inability to use shortcuts
4. WHEN a script creates a circular remap (A→B, B→A) THEN the system SHALL warn about the circular dependency
5. WHEN a script blocks more than `ValidationConfig.blocked_keys_warning_threshold` keys THEN the system SHALL show a summary of blocked keys for user awareness
6. WHEN `--no-warnings` flag is provided THEN the system SHALL suppress dangerous pattern warnings

### REQ-4: Dry-Run Simulation

**User Story:** As a user, I want to simulate what my script does for specific key inputs without actually applying it, so that I can verify behavior before committing.

#### Acceptance Criteria

1. WHEN `keyrx simulate --script config.rhai --input "CapsLock"` is run THEN the system SHALL show what output the script would produce for that input
2. WHEN simulating a tap-hold key THEN the system SHALL show both tap and hold behaviors with their timing thresholds
3. WHEN simulating a combo THEN the system SHALL show what happens when combo keys are pressed together vs individually
4. WHEN simulating with `--sequence "A,B,C"` THEN the system SHALL process keys in order and show the cumulative output
5. WHEN simulating with `--interactive` flag THEN the system SHALL enter a REPL where user can type key names and see results
6. WHEN `--json` flag is provided THEN simulation results SHALL be output in JSON format

### REQ-5: Coverage Report

**User Story:** As a user, I want to see which keys are affected by my script, so that I understand the scope of my configuration.

#### Acceptance Criteria

1. WHEN `keyrx check --coverage script.rhai` is run THEN the system SHALL list all keys that have remaps, blocks, tap-holds, or combos
2. WHEN generating coverage THEN the system SHALL categorize keys by behavior type (remapped, blocked, tap-hold, combo trigger, unaffected)
3. WHEN a key appears in multiple contexts (e.g., base layer remap and layer-specific remap) THEN the system SHALL show all contexts
4. WHEN `--visual` flag is provided THEN the system SHALL display an ASCII keyboard layout with affected keys highlighted
5. WHEN `--export` flag is provided THEN the system SHALL generate a markdown summary suitable for documentation

### REQ-6: Validation Integration Points

**User Story:** As a developer integrating KeyRx, I want validation to be available programmatically and in the Flutter UI, so that the GUI can show real-time feedback.

#### Acceptance Criteria

1. WHEN validation is requested via FFI THEN the system SHALL return structured validation results (errors, warnings, coverage)
2. WHEN the Flutter editor page loads or modifies a script THEN the system SHALL trigger validation and display results inline
3. WHEN a validation error has a line number THEN the Flutter UI SHALL highlight the problematic line
4. WHEN hovering over an error in the Flutter UI THEN the system SHALL show the full error message with suggestions
5. WHEN saving a script with errors THEN the system SHALL warn but allow save (errors are warnings by default)

### REQ-7: Config-Driven Validation

**User Story:** As a power user, I want to customize validation thresholds via a config file, so that I can tune the validation behavior to my preferences.

#### Acceptance Criteria

1. WHEN `~/.config/keyrx/validation.toml` exists THEN the system SHALL load validation thresholds from it
2. WHEN config file is missing or malformed THEN the system SHALL use sensible defaults
3. WHEN `--config <path>` flag is provided THEN the system SHALL load config from that path instead
4. WHEN any configurable threshold is used THEN the system SHALL reference `ValidationConfig` instead of hardcoded values
5. WHEN `keyrx check --show-config` is run THEN the system SHALL display current validation config values

**Configurable Values:**
- `max_errors` - Maximum errors before stopping (default: 20)
- `max_suggestions` - Similar key suggestions (default: 5)
- `similarity_threshold` - Levenshtein distance for "similar" (default: 3)
- `blocked_keys_warning_threshold` - Keys blocked before warning (default: 10)
- `max_cycle_depth` - Circular remap detection depth (default: 10)
- `tap_timeout_warn_range` - Tap timeout warning bounds [min, max] ms (default: [50, 500])
- `combo_timeout_warn_range` - Combo timeout warning bounds [min, max] ms (default: [10, 100])
- `ui_validation_debounce_ms` - Flutter UI validation debounce (default: 500)

## Non-Functional Requirements

### Code Architecture and Modularity

- **Single Responsibility**: Validation logic SHALL be in a dedicated `core/src/validation/` module, separate from runtime execution
- **Modular Design**: Each validation type (semantic, conflict, dangerous, coverage) SHALL be in its own submodule
- **Dependency Management**: Validation SHALL reuse existing `KeyCode::from_name()` and layer/modifier parsing without duplicating logic
- **Clear Interfaces**: Validation results SHALL use a structured `ValidationResult` type with errors, warnings, and info

### Performance

- Validation SHALL complete in under 100ms for scripts up to 1000 lines
- Coverage analysis SHALL complete in under 50ms
- Simulation of single key SHALL complete in under 10ms

### Security

- Validation SHALL NOT execute arbitrary Rhai code beyond parsing and static analysis
- Dry-run simulation SHALL use a sandboxed mock engine that cannot affect real input

### Reliability

- Validation SHALL never crash on malformed input (graceful error handling)
- All validation error messages SHALL include actionable remediation hints

### Usability

- Error messages SHALL follow the existing pattern: "[Error type]: [Description]. [Suggestion]."
- Similar key suggestions SHALL use Levenshtein distance to find close matches
- Warnings SHALL be distinguishable from errors in both CLI and JSON output
