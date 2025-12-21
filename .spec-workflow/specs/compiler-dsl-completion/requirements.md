# Requirements Document

## Introduction

The Compiler DSL Completion spec finalizes the `core-config-system` implementation by completing the Rhai DSL parser, polishing the CLI interface, and providing comprehensive user documentation. This enables users to write keyboard remapping configurations in a high-level scripting language (Rhai) and compile them to deterministic binary files (.krx format) using the `keyrx_compiler` CLI tool.

**Current State:** The core configuration data structures, binary serialization (rkyv), import resolution, and basic CLI scaffolding are complete (~60% of core-config-system). The Rhai DSL parser has partial implementation but lacks critical functions and validation.

**This Spec Completes:** The remaining 40% of core-config-system, delivering a production-ready compiler that users can use to create keyboard remapping configurations.

## Alignment with Product Vision

This spec directly supports the goals outlined in product.md:

**From product.md - Core Value Proposition:**
> "keyrx allows power users to create complex keyboard remapping configurations with firmware-class performance in userspace software."

**How This Spec Delivers:**
- **High-level DSL:** Users write readable Rhai scripts instead of editing binary files or JSON
- **Deterministic Compilation:** Same .rhai input always produces same .krx output (reproducible builds)
- **Type Safety:** Prefix validation (VK_, MD_, LK_) prevents common configuration errors at compile-time
- **User-Friendly Errors:** Clear error messages with suggestions guide users to correct syntax

**From product.md - Target Users:**
> "Developers, system administrators, and power users who need advanced keyboard customization"

**How This Spec Serves Them:**
- **Developer-Friendly:** CLI-first workflow integrates with scripts, build systems, version control
- **Documentation:** Comprehensive manual and examples enable self-service configuration
- **Debugging Tools:** `parse --json` subcommand helps users understand how configs are interpreted

## Requirements

### Requirement 1: Rhai DSL Function Implementation

**User Story:** As a keyboard power user, I want to write configurations using intuitive function calls like `map("VK_A", "VK_B")` and `tap_hold("VK_Space", "VK_Space", "MD_00", 200)`, so that I can create complex remapping rules without learning binary formats or low-level APIs.

#### Acceptance Criteria

1. **WHEN** user calls `map(from, to)` with valid VK_ keys **THEN** compiler **SHALL** create Simple mapping
   - Example: `map("VK_A", "VK_B")` → `KeyMapping::simple(KeyCode::A, KeyCode::B)`

2. **WHEN** user calls `map(from, to)` with MD_ output **THEN** compiler **SHALL** create Modifier mapping
   - Example: `map("VK_CapsLock", "MD_00")` → `KeyMapping::modifier(KeyCode::CapsLock, 0x00)`

3. **WHEN** user calls `map(from, to)` with LK_ output **THEN** compiler **SHALL** create Lock mapping
   - Example: `map("VK_ScrollLock", "LK_01")` → `KeyMapping::lock(KeyCode::ScrollLock, 0x01)`

4. **WHEN** user calls `tap_hold(key, tap, hold, threshold_ms)` **THEN** compiler **SHALL** create TapHold mapping
   - Example: `tap_hold("VK_Space", "VK_Space", "MD_00", 200)` → `KeyMapping::tap_hold(KeyCode::Space, KeyCode::Space, 0x00, 200)`
   - **AND** tap parameter **SHALL** require VK_ prefix
   - **AND** hold parameter **SHALL** require MD_ prefix (not physical names like "MD_LShift")

5. **WHEN** user calls `with_shift(key)` **THEN** compiler **SHALL** return ModifiedKey with shift=true
   - Example: `map("VK_1", with_shift("VK_1"))` → `KeyMapping::modified_output(KeyCode::Num1, KeyCode::Num1, true, false, false, false)`

6. **WHEN** user calls `with_ctrl(key)`, `with_alt(key)`, or `with_win(key)` **THEN** compiler **SHALL** return ModifiedKey with appropriate modifier flag

7. **WHEN** user calls `with_mods(key, shift, ctrl, alt, win)` **THEN** compiler **SHALL** return ModifiedKey with all specified modifiers
   - Example: `with_mods("VK_C", false, true, false, false)` → Ctrl+C

8. **WHEN** user calls `when(condition, closure)` with single condition string **THEN** compiler **SHALL** create Conditional mapping with single condition
   - Example: `when("MD_00", || { map("VK_H", "VK_Left"); })` → `KeyMapping::Conditional { condition: Condition::ModifierActive(0x00), mappings: [...] }`

9. **WHEN** user calls `when(conditions, closure)` with array of conditions **THEN** compiler **SHALL** create Conditional mapping with AllActive condition
   - Example: `when(["MD_00", "LK_01"], || { ... })` → `Condition::AllActive([ModifierActive(0x00), LockActive(0x01)])`

10. **WHEN** user calls `when_not(condition, closure)` **THEN** compiler **SHALL** create Conditional mapping with NotActive condition
    - Example: `when_not("MD_00", || { ... })` → `Condition::NotActive([ModifierActive(0x00)])`

11. **WHEN** user calls `device(pattern, closure)` **THEN** compiler **SHALL** create DeviceConfig with specified pattern
    - Example: `device("USB\\VID_04D9*", || { ... })` → `DeviceConfig { identifier: DeviceIdentifier { pattern: "USB\\VID_04D9*" }, mappings: [...] }`

### Requirement 2: Prefix Validation and Error Messages

**User Story:** As a keyboard power user, I want clear error messages when I make syntax mistakes, so that I can quickly identify and fix configuration errors without guessing.

#### Acceptance Criteria

1. **WHEN** user provides output key without VK_/MD_/LK_ prefix in `map()` **THEN** compiler **SHALL** emit `ParseError::MissingPrefix` with helpful suggestion
   - Error message **SHALL** explain all valid prefixes
   - Error message **SHALL** show correct syntax example

2. **WHEN** user provides physical modifier name in MD_ (e.g., "MD_LShift") **THEN** compiler **SHALL** emit `ParseError::PhysicalModifierInMD` error
   - Error message **SHALL** explain that MD_ requires hex IDs (MD_00 through MD_FE)
   - Error message **SHALL** clarify that physical modifiers are for output only (with_shift, etc.)

3. **WHEN** user provides modifier ID >254 (>0xFE) **THEN** compiler **SHALL** emit `ParseError::ModifierIdOutOfRange` error
   - Error message **SHALL** state valid range: MD_00 through MD_FE (0-254)

4. **WHEN** user provides lock ID >254 (>0xFE) **THEN** compiler **SHALL** emit `ParseError::LockIdOutOfRange` error
   - Error message **SHALL** state valid range: LK_00 through LK_FE (0-254)

5. **WHEN** user provides invalid VK_ key name **THEN** compiler **SHALL** emit `ParseError::InvalidKeyName` with suggestion
   - Error message **SHALL** show similar key names if available (fuzzy matching)

6. **WHEN** user provides VK_ prefix in tap_hold hold parameter **THEN** compiler **SHALL** emit `ParseError::InvalidHoldPrefix` error
   - Error message **SHALL** explain hold parameter must be MD_ (custom modifier)

7. **IF** error occurs in imported file **THEN** error message **SHALL** show import chain
   - Example: `main.rhai → common/vim.rhai (line 42) → error`

8. **ALL** error messages **SHALL** include:
   - File path and line number
   - Column position (if available)
   - Code snippet showing the error location
   - Clear explanation of the problem
   - Suggestion for how to fix it

### Requirement 3: CLI Interface Completeness

**User Story:** As a developer integrating keyrx into my workflow, I want a complete CLI with subcommands for compilation, verification, hashing, and debugging, so that I can automate config generation and validation in scripts.

#### Acceptance Criteria

1. **WHEN** user runs `keyrx_compiler compile input.rhai -o output.krx` **THEN** compiler **SHALL**:
   - Parse Rhai script with all imports
   - Generate ConfigRoot with all mappings
   - Serialize to .krx binary format
   - Write output file
   - Print success message with file size and SHA256 hash

2. **WHEN** user runs `keyrx_compiler verify config.krx` **THEN** compiler **SHALL**:
   - Verify magic bytes (KRX\n)
   - Verify format version
   - Compute SHA256 hash of data section
   - Compare with embedded hash
   - Validate rkyv structure
   - Print verification result (pass/fail with details)

3. **WHEN** user runs `keyrx_compiler hash config.krx` **THEN** compiler **SHALL**:
   - Extract embedded SHA256 hash from header
   - Print hash in hexadecimal format
   - Optionally compute hash of data section with `--verify` flag

4. **WHEN** user runs `keyrx_compiler parse input.rhai` **THEN** compiler **SHALL**:
   - Parse Rhai script
   - Build ConfigRoot
   - Print human-readable summary of configuration

5. **WHEN** user runs `keyrx_compiler parse input.rhai --json` **THEN** compiler **SHALL**:
   - Parse Rhai script
   - Build ConfigRoot
   - Serialize ConfigRoot to JSON
   - Print JSON to stdout

6. **WHEN** compilation succeeds **THEN** CLI **SHALL** exit with code 0

7. **WHEN** compilation fails **THEN** CLI **SHALL**:
   - Print error message to stderr
   - Exit with code 1

8. **WHEN** user provides invalid subcommand **THEN** CLI **SHALL**:
   - Print usage help
   - List all available subcommands
   - Exit with code 1

9. **WHEN** user runs `keyrx_compiler --help` **THEN** CLI **SHALL** print comprehensive help with:
   - Description of each subcommand
   - Examples of common usage
   - Link to full documentation

10. **IF** compilation takes >2 seconds **THEN** CLI **SHALL** show progress indicator

### Requirement 4: Integration Testing

**User Story:** As a compiler maintainer, I want comprehensive integration tests covering end-to-end workflows, so that I can confidently refactor code without breaking user configurations.

#### Acceptance Criteria

1. **WHEN** integration tests run **THEN** tests **SHALL** cover:
   - Compile simple config → verify output
   - Compile config with imports → verify all imports resolved
   - Compile config with all mapping types → verify all in output
   - Compile config with errors → verify error messages
   - Compile same config twice → verify deterministic output (byte-identical)
   - Verify valid .krx file → pass
   - Verify corrupted .krx file → fail with specific error
   - Parse config → verify JSON output structure

2. **WHEN** property-based tests run **THEN** tests **SHALL** verify:
   - serialize(config) produces valid .krx every time (no crashes)
   - deserialize(serialize(config)) == config (round-trip)
   - serialize(config1) == serialize(config2) if config1 == config2 (determinism)

3. **ALL** integration tests **SHALL** use real .rhai files (not just programmatic API)

4. **ALL** tests **SHALL** clean up temporary files after execution

### Requirement 5: User Documentation

**User Story:** As a new keyrx user, I want complete documentation with examples, so that I can start writing configurations within 30 minutes without external help.

#### Acceptance Criteria

1. **DSL Manual** (`docs/DSL_MANUAL.md`) **SHALL** include:
   - Overview of Rhai syntax basics
   - Complete reference for all functions: `map()`, `tap_hold()`, `when()`, `when_not()`, `with_shift()`, etc.
   - Explanation of prefixes (VK_, MD_, LK_) with visual diagram
   - Full list of supported KeyCode names (VK_A through VK_F12, etc.)
   - Valid modifier ID ranges (MD_00 through MD_FE)
   - Valid lock ID ranges (LK_00 through LK_FE)
   - Common error messages and solutions
   - Best practices and patterns

2. **Example Configurations** **SHALL** include:
   - `examples/01-simple-remap.rhai` - Basic A→B remapping
   - `examples/02-capslock-escape.rhai` - Classic CapsLock→Escape
   - `examples/03-vim-navigation.rhai` - Vim arrow keys with modifier
   - `examples/04-dual-function-keys.rhai` - Tap/hold (Space tap=space, hold=ctrl)
   - `examples/05-multiple-devices.rhai` - Different configs for different keyboards
   - `examples/06-advanced-layers.rhai` - Complex multi-layer setup with modifiers

3. **Each example** **SHALL** include:
   - Header comment explaining what it does
   - Inline comments explaining each mapping
   - Expected behavior description
   - Compilation instructions

4. **README.md** **SHALL** include:
   - Quickstart section (install → write config → compile → load)
   - Link to DSL_MANUAL.md
   - Link to examples/
   - Troubleshooting section
   - Contribution guidelines

5. **Documentation** **SHALL** be tested for accuracy:
   - All code examples **SHALL** compile without errors
   - All .rhai examples **SHALL** be tested in CI

## Non-Functional Requirements

### Code Architecture and Modularity

- **Single Responsibility Principle**: Parser functions separated by DSL function (`map.rs`, `tap_hold.rs`, `when.rs`, etc.)
- **Modular Design**: Validators separated from parsers (prefix validation in `validators.rs`)
- **Dependency Management**: Parser state isolated in `ParserState` struct, passed via Arc<Mutex<>> to Rhai functions
- **Clear Interfaces**: Each Rhai function has dedicated Rust implementation with clear signature

**File Organization:**
```
keyrx_compiler/src/parser/
├── mod.rs              # Public API, Rhai engine setup
├── core.rs             # ParserState, Parser struct
├── validators.rs       # Prefix validation, key name parsing
├── functions/
│   ├── mod.rs          # Function registration
│   ├── map.rs          # map() implementation
│   ├── tap_hold.rs     # tap_hold() implementation
│   ├── modifiers.rs    # with_shift(), with_ctrl(), with_alt(), with_mods()
│   ├── conditional.rs  # when(), when_not()
│   └── device.rs       # device()
└── helpers.rs          # Utility functions for parsing
```

### Performance

- **Compilation Speed**: Compile 1000-line config in <2 seconds on modern hardware
- **Memory Usage**: Compilation memory usage <100MB for typical configs
- **Determinism**: Same input produces byte-identical output 100% of the time
- **Binary Size**: .krx output size <50KB for configs with 1000 mappings

### Security

- **Input Validation**: All user input validated before processing (key names, IDs, patterns)
- **Resource Limits**: Rhai engine configured with limits:
  - Max operations: 10,000 (prevents infinite loops)
  - Max recursion depth: 100 (prevents stack overflow)
  - Timeout: 10 seconds (prevents hanging)
- **Path Traversal Prevention**: Import paths sanitized to prevent directory traversal attacks
- **Error Information Disclosure**: Error messages do not reveal internal system details

### Reliability

- **Graceful Error Handling**: All errors caught and reported with actionable messages
- **No Panics**: Compiler never panics on user input (all errors returned as Result<>)
- **Deterministic Builds**: Given same input, compiler always produces same output
- **Import Cycle Detection**: Circular imports detected and reported before compilation

### Usability

- **Clear Error Messages**: Every error includes location, explanation, and fix suggestion
- **Progressive Disclosure**: Basic usage simple, advanced features documented separately
- **Familiar Syntax**: Rhai syntax similar to JavaScript/Rust (familiar to developers)
- **Inline Help**: `--help` flag provides usage examples for every subcommand
- **Fast Feedback**: Compilation errors shown immediately with clear actionable information
