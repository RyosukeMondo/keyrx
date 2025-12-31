# Requirements Document

## Introduction

The Core Configuration System provides the foundational infrastructure for keyrx's configuration management. It enables users to write Rhai-based configuration scripts and compile them into deterministic, zero-copy binary files (.krx format). This system is the critical path dependency for all keyrx features, as it defines how configurations are authored, compiled, and consumed by the daemon.

**Key DSL Features**:
- **255 custom modifiers** (MD_00 - MD_FE) for creating custom modifier keys
- **255 custom locks** (LK_00 - LK_FE) for toggle states
- **Physical modifier outputs** (Shift+2, Ctrl+C, etc.) via helper functions
- **Cross-device state sharing** - modifiers/locks work across all keyboards
- **Tap/hold behavior** - keys behave differently when tapped vs held
- **Nested modifier cascades** - modifiers can activate other modifiers
- **Platform-agnostic** - same config works on Linux/Windows

## Alignment with Product Vision

This spec implements the **"AI Coding Agent First"** product principles (product.md):

**Single Source of Truth (SSOT)**:
- .krx binary is the single authoritative source for all remapping behavior
- Deterministic serialization ensures same Rhai input → same binary output → same hash
- AI agents can verify configuration changes via SHA256 hash comparison

**CLI-First Design**:
- `keyrx_compiler` provides pure CLI interface (no GUI required)
- AI agents can compile configurations programmatically
- Machine-parseable error messages with JSON output mode

**Deterministic Behavior**:
- rkyv serialization is deterministic (stable field ordering, no randomness)
- Hash verification detects any non-determinism immediately
- Same input configuration always produces identical binary output

**255 Modifiers + 255 Locks**:
- Far exceeds manual testing capability (product.md line 27)
- Requires AI-automated validation
- Enables advanced input paradigms impossible with standard 8 modifiers

## Requirements

### Requirement 1: Configuration Data Structures

**User Story:** As a **developer**, I want **well-defined configuration data structures**, so that **the compiler and daemon share a common understanding of configuration format**.

#### Acceptance Criteria

1. WHEN configuration structures are defined THEN the system SHALL create a `ConfigRoot` struct with version field (semantic versioning), list of device configurations, and metadata (compilation timestamp, compiler version, source file hash)

2. WHEN a device configuration is defined THEN the system SHALL create a `DeviceConfig` struct with device identifier pattern and list of key mappings

3. WHEN a key mapping is defined THEN the system SHALL create a `KeyMapping` enum with variants:
   - `Simple { from: KeyCode, to: KeyCode }` - Simple 1:1 remapping
   - `Modifier { from: KeyCode, modifier_id: u8 }` - Key acts as custom modifier (MD_00-MD_FE)
   - `Lock { from: KeyCode, lock_id: u8 }` - Key toggles custom lock (LK_00-LK_FE)
   - `TapHold { from: KeyCode, tap: KeyCode, hold_modifier: u8, threshold_ms: u16 }` - Dual tap/hold behavior
   - `ModifiedOutput { from: KeyCode, to: KeyCode, shift: bool, ctrl: bool, alt: bool, win: bool }` - Output with physical modifiers (Shift+2, Ctrl+C, etc.)
   - `Conditional { condition: Condition, mappings: Vec<KeyMapping> }` - Conditional mappings (when/when_not blocks)

4. WHEN conditional mappings are defined THEN the system SHALL create a `Condition` enum with variants:
   - `ModifierActive(u8)` - Single custom modifier active (MD_XX)
   - `LockActive(u8)` - Single custom lock active (LK_XX)
   - `AllActive(Vec<ConditionItem>)` - All conditions must be true (AND logic)
   - `NotActive(Box<Condition>)` - Negated condition (when_not)

5. WHEN key codes are defined THEN the system SHALL create a `KeyCode` enum with at least 100 common key codes organized by category (letters, numbers, function keys, modifiers, special keys, arrows)

6. WHEN structures are serialized THEN they SHALL derive `rkyv::Archive`, `rkyv::Serialize`, `rkyv::Deserialize` with `#[repr(C)]` for stable memory layout

7. WHEN structures are defined THEN they SHALL be located in `keyrx_core/src/config.rs` with no_std compatibility

### Requirement 2: Rhai DSL Parser and Evaluator

**User Story:** As a **user**, I want **a simple DSL for defining keyboard remappings**, so that **I can write configurations without learning complex syntax**.

#### Acceptance Criteria

1. WHEN the Rhai parser is initialized THEN the system SHALL create a Rhai engine with custom functions registered:
   - `map(from, to)` - Basic mapping
   - `tap_hold(key, tap, hold, threshold_ms)` - Tap/hold behavior
   - `when(condition) { ... }` - Conditional block
   - `when_not(condition) { ... }` - Negated conditional block
   - `device(pattern) { ... }` - Device-specific block
   - `with_shift(key)` - Output with Shift modifier
   - `with_ctrl(key)` - Output with Ctrl modifier
   - `with_alt(key)` - Output with Alt modifier
   - `with_mods(key, modifiers)` - Output with multiple physical modifiers

2. WHEN a Rhai script calls `map(from, to)` THEN the system SHALL:
   - Parse `from` as physical key (no prefix required)
   - Parse `to` with prefix validation:
     - `VK_X` → Virtual key output (Simple mapping)
     - `MD_XX` → Custom modifier (Modifier mapping, XX = 00-FE hex)
     - `LK_XX` → Custom lock toggle (Lock mapping, XX = 00-FE hex)
   - Create appropriate KeyMapping variant
   - Reject invalid prefixes or missing prefixes with clear error messages

3. WHEN a Rhai script calls `tap_hold(key, tap, hold, threshold_ms)` THEN the system SHALL:
   - Validate `tap` has `VK_` prefix (error if not)
   - Validate `hold` has `MD_` prefix (error if not)
   - Reject physical modifier names in `hold` (e.g., `MD_LShift` is invalid)
   - Use default threshold of 200ms if not specified
   - Create TapHold mapping variant

4. WHEN a Rhai script calls `when(condition) { ... }` THEN the system SHALL:
   - Accept single string (single modifier/lock) or array (multiple conditions with AND logic)
   - Parse condition strings with MD_/LK_ prefix validation
   - Create Conditional mapping with parsed condition and nested mappings
   - Support mixed conditions (e.g., `["MD_00", "LK_01"]` - modifier AND lock)

5. WHEN a Rhai script calls `when_not(condition) { ... }` THEN the system SHALL:
   - Accept single string only (no arrays)
   - Create Conditional mapping with NotActive condition wrapper
   - Parse condition string with MD_/LK_ prefix validation

6. WHEN a Rhai script calls helper functions (`with_shift`, `with_ctrl`, etc.) THEN the system SHALL:
   - Validate key has `VK_` prefix
   - Create ModifiedOutput mapping with appropriate modifier flags
   - Support `with_mods(key, shift: true, ctrl: true)` named parameter syntax
   - Support display name parameter (optional, documentation only)

7. WHEN the parser evaluates a script THEN it SHALL enforce resource limits: max 10,000 operations, max 100 recursion depth, 10-second timeout

8. WHEN a Rhai script has syntax errors THEN the system SHALL produce user-friendly error messages with line numbers, column positions, and suggestions

9. WHEN the parser completes THEN it SHALL return a fully populated `ConfigRoot` struct ready for serialization

### Requirement 3: Binary Serialization and Deserialization

**User Story:** As a **developer**, I want **deterministic binary serialization**, so that **configurations can be verified via hash comparison**.

#### Acceptance Criteria

1. WHEN a `ConfigRoot` is serialized THEN the system SHALL use rkyv to produce a binary format with 48-byte header (magic bytes, version, hash, size)

2. WHEN serialization completes THEN the system SHALL compute SHA256 hash of the serialized data and embed it in the header

3. WHEN the same `ConfigRoot` is serialized multiple times THEN the system SHALL produce bit-identical output (deterministic serialization)

4. WHEN a .krx file is deserialized THEN the system SHALL verify the magic bytes (0x4B52580A), version compatibility, and SHA256 hash

5. WHEN hash verification fails THEN the system SHALL reject the file with error message indicating corruption or tampering

6. WHEN deserialization succeeds THEN the system SHALL provide zero-copy access to the configuration (no heap allocation)

### Requirement 4: Import System

**User Story:** As a **user**, I want **to split configurations across multiple files**, so that **I can organize complex configurations modularly**.

#### Acceptance Criteria

1. WHEN a Rhai script uses `import "path/to/file.rhai"` THEN the system SHALL resolve the path relative to the current file's directory

2. WHEN imports are resolved THEN the system SHALL recursively load and evaluate imported files before continuing with the parent script

3. WHEN circular imports are detected THEN the system SHALL abort with error message showing the import cycle chain

4. WHEN an imported file is not found THEN the system SHALL report error with searched paths and suggestions

5. WHEN imports are successful THEN all imported device configurations SHALL be merged into the root configuration

### Requirement 5: CLI Compiler Interface

**User Story:** As a **user**, I want **a command-line compiler**, so that **I can compile configurations from scripts or CI/CD pipelines**.

#### Acceptance Criteria

1. WHEN `keyrx_compiler compile input.rhai -o output.krx` is executed THEN the system SHALL parse the Rhai script, serialize it, and write the .krx file

2. WHEN `keyrx_compiler verify config.krx` is executed THEN the system SHALL validate the binary format and hash, reporting success or specific errors

3. WHEN `keyrx_compiler hash config.krx` is executed THEN the system SHALL output the SHA256 hash in hexadecimal format

4. WHEN `keyrx_compiler parse input.rhai --json` is executed THEN the system SHALL parse the script and output the configuration as JSON (for debugging)

5. WHEN any command fails THEN the system SHALL exit with non-zero exit code and output structured error messages

6. WHEN `--help` flag is used THEN the system SHALL display usage information for all commands and flags

### Requirement 6: Error Handling and Validation

**User Story:** As a **user**, I want **clear error messages**, so that **I can quickly fix configuration mistakes**.

#### Acceptance Criteria

1. WHEN errors occur THEN the system SHALL use structured error types with error codes, context, and actionable suggestions

2. WHEN Rhai syntax errors occur THEN error messages SHALL include file path, line number, column number, and snippet of problematic code

3. WHEN prefix validation fails THEN error messages SHALL explain:
   - Missing prefix: "Output must have VK_, MD_, or LK_ prefix: B"
   - Invalid prefix: "Unknown key prefix: MD_LShift (use MD_00 through MD_FE)"
   - Wrong prefix in context: "tap_hold hold parameter must have MD_ prefix, got: VK_Space"

4. WHEN modifier/lock IDs are out of range THEN error messages SHALL indicate valid range: "Invalid modifier ID: MD_100 (must be MD_00 through MD_FE)"

5. WHEN `--json` flag is used THEN errors SHALL be formatted as JSON with fields: error_code, message, file, line, column, suggestion

6. WHEN multiple errors are present THEN the system SHALL report all errors (not just the first one)

### Requirement 7: Testing and Verification

**User Story:** As a **developer**, I want **comprehensive tests**, so that **the configuration system is reliable and correct**.

#### Acceptance Criteria

1. WHEN tests are implemented THEN they SHALL achieve at least 90% code coverage for the config system

2. WHEN unit tests are run THEN they SHALL test each component in isolation: parser, serializer, import resolver, CLI, prefix validation

3. WHEN integration tests are run THEN they SHALL test end-to-end workflows: Rhai script → .krx binary → verification

4. WHEN property-based tests are run THEN they SHALL verify deterministic serialization: serialize(x) == serialize(x) for all valid configurations

5. WHEN fuzz tests are run THEN they SHALL test parser robustness with randomly generated Rhai scripts for at least 60 seconds

## Non-Functional Requirements

### Code Architecture and Modularity
- **Single Responsibility Principle**: Each module handles one concern (parsing, serialization, import resolution)
- **Modular Design**: Components are isolated and reusable (core structs in keyrx_core, compiler logic in keyrx_compiler)
- **Dependency Management**: keyrx_core is no_std with minimal dependencies (rkyv, fixedbitset, arrayvec)
- **Clear Interfaces**: Public APIs are well-documented with examples

### Performance
- **Compilation Time**: Compile 1000-line configuration in <100ms
- **Serialization Time**: Serialize typical configuration in <10ms
- **Deserialization Time**: Deserialize .krx file in <5ms (zero-copy)
- **Binary Size**: .krx files <10KB for typical configurations (100 mappings)

### Security
- **Resource Limits**: Parser enforces operation limits, recursion limits, and timeouts to prevent DoS
- **Hash Verification**: SHA256 hashing prevents tampering with .krx files
- **Input Validation**: All user inputs are validated before processing (prefix validation, range checks)
- **No Code Execution**: Daemon never executes Rhai scripts (only loads pre-compiled .krx)

### Reliability
- **Deterministic Output**: Same input always produces identical output (bit-for-bit)
- **Validation**: All .krx files are validated before use (magic bytes, version, hash)
- **Error Recovery**: Parser recovers from syntax errors and reports all errors
- **No Panics**: All error conditions are handled gracefully with Result types

### Usability
- **Clear Syntax**: Rhai DSL is simple and intuitive (map/when/device API)
- **Helpful Errors**: Error messages include context, suggestions, and examples
- **JSON Output**: All commands support --json for machine parsing
- **Comprehensive Help**: --help flag provides usage examples for all commands
- **Display Names**: Optional display parameter documents what modified outputs produce
