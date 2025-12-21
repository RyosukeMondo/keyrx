# Tasks Document

## Phase 1: Validator Implementation (Foundation)

- [x] 1. Create prefix validation module
  - File: `keyrx_compiler/src/parser/validators.rs` (NEW)
  - Define `parse_physical_key()`, `parse_virtual_key()` functions
  - Implement string-to-KeyCode conversion with VK_ prefix check
  - Add fuzzy matching for "did you mean?" suggestions
  - Purpose: Foundation for all DSL functions - validates user key names
  - _Leverage: keyrx_core::config::KeyCode enum_
  - _Requirements: 2.1, 2.5_
  - _Prompt: Role: Rust Backend Developer with expertise in parsing and error handling | Task: Create comprehensive prefix validation module implementing parse_physical_key() and parse_virtual_key() functions that convert user strings like "VK_A" to KeyCode::A enum variants, following requirements 2.1 and 2.5, using KeyCode enum from keyrx_core | Restrictions: Must validate VK_ prefix exists, return actionable ParseError on invalid input, implement fuzzy matching for suggestions (e.g., "VK_Shft" suggests "VK_LShift"), do not panic on invalid input | Success: All 100+ KeyCode variants parseable by name, invalid names rejected with helpful suggestions, VK_ prefix enforced, no unwrap() or expect() in code_

- [ ] 2. Add modifier/lock ID validation
  - File: `keyrx_compiler/src/parser/validators.rs` (continue)
  - Implement `parse_modifier_id()` - extract and validate MD_XX (hex 00-FE)
  - Implement `parse_lock_id()` - extract and validate LK_XX (hex 00-FE)
  - Detect physical modifier names (LShift, RCtrl, etc.) and reject them
  - Purpose: Validate custom modifier/lock IDs with proper range checking
  - _Leverage: None (pure validation logic)_
  - _Requirements: 2.2, 2.3, 2.4_
  - _Prompt: Role: Rust Developer specializing in validation logic and hex parsing | Task: Implement parse_modifier_id() and parse_lock_id() functions that extract hex IDs from "MD_XX" and "LK_XX" strings, validate range 00-FE (0-254), and reject physical modifier names like "MD_LShift", following requirements 2.2, 2.3, and 2.4 | Restrictions: Must parse hex with u8::from_str_radix, reject values >0xFE, detect PHYSICAL_MODIFIERS constant list ["LShift", "RShift", "LCtrl", "RCtrl", "LAlt", "RAlt", "LWin", "RWin"] and return PhysicalModifierInMD error, do not use unwrap() | Success: "MD_00" through "MD_FE" accepted (0-254), "MD_FF" rejected with out-of-range error, "MD_LShift" rejected with physical name error, all error messages actionable_

- [ ] 3. Add condition string parsing
  - File: `keyrx_compiler/src/parser/validators.rs` (continue)
  - Implement `parse_condition_string()` - convert "MD_00" or "LK_01" to Condition enum
  - Implement `parse_condition_item()` - convert to ConditionItem enum
  - Purpose: Support when() and when_not() functions
  - _Leverage: keyrx_core::config::Condition, ConditionItem enums_
  - _Requirements: 1.8, 1.10_
  - _Prompt: Role: Rust Parser Developer with expertise in enum conversion | Task: Implement parse_condition_string() that converts user strings like "MD_00" to Condition::ModifierActive(0x00) and "LK_01" to Condition::LockActive(0x01), and parse_condition_item() for ConditionItem enum, following requirements 1.8 and 1.10, using Condition and ConditionItem enums from keyrx_core | Restrictions: Must detect prefix (MD_ or LK_), call appropriate validator (parse_modifier_id or parse_lock_id), return ParseError for invalid format, support only single modifiers/locks (no complex conditions here) | Success: "MD_00" → Condition::ModifierActive(0x00), "LK_01" → Condition::LockActive(0x01), invalid strings rejected, supports all 255 modifier/lock IDs_

- [ ] 4. Write validator unit tests
  - File: `keyrx_compiler/tests/validators_tests.rs` (NEW)
  - Test parse_physical_key() with all KeyCode variants
  - Test parse_modifier_id() success cases (MD_00 through MD_FE)
  - Test parse_modifier_id() error cases (MD_FF, MD_LShift, invalid hex)
  - Test parse_lock_id() success and error cases
  - Test parse_condition_string() for all condition types
  - Purpose: Ensure validators are reliable and error messages are correct
  - _Leverage: None (unit tests)_
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_
  - _Prompt: Role: QA Engineer with Rust testing expertise | Task: Create comprehensive unit tests for all validator functions covering requirements 2.1-2.5, testing both success and failure scenarios for parse_physical_key, parse_modifier_id, parse_lock_id, and parse_condition_string | Restrictions: Must test all edge cases (empty string, missing prefix, out of range, physical names), verify error messages are actionable, use assert_eq! for success cases and assert!(matches!(...)) for error variants, maintain test isolation | Success: 100% coverage of validator functions, all error scenarios tested, fuzzy matching suggestions verified, tests run independently_

## Phase 2: DSL Function Implementation (Core Functionality)

- [ ] 5. Implement map() function with prefix dispatch
  - File: `keyrx_compiler/src/parser/functions/map.rs` (ENHANCED)
  - Register `map(from, to)` function in Rhai engine
  - Detect output prefix (VK_/MD_/LK_) and create appropriate mapping
  - Use validators from Phase 1
  - Add mapping to current device in ParserState
  - Purpose: Core mapping function - handles 3 different output types
  - _Leverage: validators.rs, KeyMapping::simple/modifier/lock helpers_
  - _Requirements: 1.1, 1.2, 1.3_
  - _Prompt: Role: Rust Developer with Rhai scripting expertise | Task: Implement map() function that detects output key prefix and creates Simple (VK_), Modifier (MD_), or Lock (LK_) mapping using validators and KeyMapping helpers, following requirements 1.1, 1.2, and 1.3, leveraging validators.rs and KeyMapping helper functions from keyrx_core | Restrictions: Must call parse_physical_key for from parameter, dispatch on to.starts_with("VK_"/"MD_"/"LK_"), return MissingPrefix error if no valid prefix, add to state.current_device.mappings, return error if current_device is None | Success: map("VK_A", "VK_B") creates Simple mapping, map("VK_CapsLock", "MD_00") creates Modifier mapping, map("VK_ScrollLock", "LK_01") creates Lock mapping, missing prefix returns actionable error_

- [ ] 6. Implement tap_hold() function with validation
  - File: `keyrx_compiler/src/parser/functions/tap_hold.rs` (ENHANCED)
  - Register `tap_hold(key, tap, hold, threshold_ms)` in Rhai engine
  - Validate tap parameter has VK_ prefix (reject others)
  - Validate hold parameter has MD_ prefix (reject VK_ and physical names)
  - Create TapHold mapping using KeyMapping::tap_hold()
  - Purpose: Dual-function keys (tap for key, hold for modifier)
  - _Leverage: validators.rs, KeyMapping::tap_hold helper_
  - _Requirements: 1.4_
  - _Prompt: Role: Rust Developer with input validation expertise | Task: Implement tap_hold() function that validates tap has VK_ prefix and hold has MD_ prefix (rejecting physical names), then creates TapHold mapping, following requirement 1.4, using validators and KeyMapping::tap_hold helper | Restrictions: Must call parse_physical_key for key parameter, enforce tap.starts_with("VK_") else return InvalidTapPrefix error, enforce hold.starts_with("MD_") else return InvalidHoldPrefix error, parse_modifier_id will reject physical names, convert threshold_ms from i64 to u16, add to current device | Success: tap_hold("VK_Space", "VK_Space", "MD_00", 200) creates TapHold mapping, tap_hold("VK_Space", "MD_00", "VK_Space", 200) returns InvalidTapPrefix error, tap_hold("VK_Space", "VK_Space", "MD_LShift", 200) returns PhysicalModifierInMD error_

- [ ] 7. Implement helper functions (with_shift, with_ctrl, with_alt, with_mods)
  - File: `keyrx_compiler/src/parser/functions/modifiers.rs` (NEW)
  - Define `ModifiedKey` struct (temporary builder type)
  - Register `with_shift(key)`, `with_ctrl(key)`, `with_alt(key)`, `with_win(key)` functions
  - Register `with_mods(key, shift, ctrl, alt, win)` function
  - Register `map(from, ModifiedKey)` overload that creates ModifiedOutput mapping
  - Purpose: Enable physical modifier output (Shift+2, Ctrl+C, etc.)
  - _Leverage: validators.rs, KeyMapping::modified_output helper_
  - _Requirements: 1.5, 1.6, 1.7_
  - _Prompt: Role: Rust Developer with builder pattern expertise | Task: Implement modifier helper functions (with_shift, with_ctrl, etc.) that return ModifiedKey struct, and map() overload that consumes ModifiedKey to create ModifiedOutput mapping, following requirements 1.5, 1.6, and 1.7, using validators and KeyMapping::modified_output helper | Restrictions: ModifiedKey must be #[derive(Clone)] for Rhai compatibility, all helpers must call parse_virtual_key, map() overload must call parse_physical_key for from parameter, create ModifiedOutput with appropriate boolean flags, register ModifiedKey as Rhai type | Success: with_shift("VK_1") returns ModifiedKey, map("VK_2", with_shift("VK_1")) creates ModifiedOutput with shift=true, with_mods("VK_C", false, true, false, false) creates Ctrl+C mapping_

- [ ] 8. Implement when() function with array support
  - File: `keyrx_compiler/src/parser/functions/conditional.rs` (ENHANCED)
  - Register `when(condition, closure)` for single condition string
  - Register `when(conditions, closure)` for array of conditions (AllActive)
  - Parse condition strings using validators
  - Execute Rhai closure to collect nested mappings
  - Create Conditional mapping with collected mappings
  - Purpose: Conditional key mappings based on modifier/lock state
  - _Leverage: validators.rs (parse_condition_string), KeyMapping::conditional helper_
  - _Requirements: 1.8, 1.9_
  - _Prompt: Role: Rust Developer with Rhai closure handling expertise | Task: Implement when() function with two overloads - single condition string and array of conditions (AllActive), parse conditions using parse_condition_string, execute Rhai closure to collect mappings, create Conditional mapping, following requirements 1.8 and 1.9, using validators and KeyMapping::conditional helper | Restrictions: Single condition overload accepts &str, array overload accepts rhai::Array, parse each condition in array, create Condition::AllActive for arrays, Condition::ModifierActive or LockActive for single, closure execution must capture mappings into Vec<BaseKeyMapping>, add Conditional to current device | Success: when("MD_00", || { map("VK_H", "VK_Left"); }) creates Conditional with single ModifierActive condition, when(["MD_00", "LK_01"], || { ... }) creates Conditional with AllActive containing both items_

- [ ] 9. Implement when_not() function
  - File: `keyrx_compiler/src/parser/functions/conditional.rs` (continue)
  - Register `when_not(condition, closure)` for negated conditions
  - Parse condition and wrap in NotActive
  - Execute closure and collect mappings
  - Create Conditional mapping with NotActive condition
  - Purpose: Negated conditionals (when modifier is NOT active)
  - _Leverage: validators.rs, KeyMapping::conditional helper_
  - _Requirements: 1.10_
  - _Prompt: Role: Rust Developer with conditional logic expertise | Task: Implement when_not() function that parses condition string, wraps in Condition::NotActive, executes closure to collect mappings, creates Conditional mapping, following requirement 1.10, using validators and KeyMapping::conditional helper | Restrictions: Parse condition with parse_condition_string, convert to ConditionItem (ModifierActive or LockActive only), create Condition::NotActive(vec![item]), execute closure, add to current device | Success: when_not("MD_00", || { map("VK_K", "VK_Up"); }) creates Conditional with NotActive([ModifierActive(0x00)]) condition_

- [ ] 10. Implement device() function
  - File: `keyrx_compiler/src/parser/functions/device.rs` (NEW)
  - Register `device(pattern, closure)` function
  - Create DeviceConfig with pattern string
  - Set ParserState.current_device before executing closure
  - Execute closure to collect mappings
  - Add completed DeviceConfig to ParserState.devices
  - Clear current_device after closure
  - Purpose: Group mappings by device identifier pattern
  - _Leverage: None (creates DeviceConfig directly)_
  - _Requirements: 1.11_
  - _Prompt: Role: Rust Developer with state management expertise | Task: Implement device() function that creates DeviceConfig with pattern string, sets ParserState.current_device, executes Rhai closure to collect mappings, adds completed DeviceConfig to devices list, following requirement 1.11 | Restrictions: Must lock ParserState mutex, create DeviceConfig with DeviceIdentifier { pattern }, set current_device to Some(device), execute closure (mappings will be added to current_device by other functions), push device to state.devices, set current_device to None, handle errors if device() called inside another device() | Success: device("*", || { map("VK_A", "VK_B"); }) creates DeviceConfig with pattern "*" and one Simple mapping, nested device() calls return error_

- [ ] 11. Write DSL function unit tests
  - File: `keyrx_compiler/tests/parser_function_tests.rs` (NEW)
  - Test map() with VK_/MD_/LK_ outputs
  - Test tap_hold() with valid and invalid parameters
  - Test helper functions (with_shift, with_ctrl, etc.)
  - Test when() with single and array conditions
  - Test when_not() with valid conditions
  - Test device() creates DeviceConfig correctly
  - Test error cases (function called outside device, invalid prefixes, etc.)
  - Purpose: Ensure all DSL functions work correctly in isolation
  - _Leverage: ParserState setup utilities_
  - _Requirements: 1.1-1.11_
  - _Prompt: Role: QA Engineer with Rhai testing expertise | Task: Create comprehensive unit tests for all DSL functions (map, tap_hold, helpers, when, when_not, device) covering requirements 1.1-1.11, testing both success paths and error scenarios | Restrictions: Must test each function in isolation using mock ParserState, verify KeyMapping types created, test error messages for invalid input, test that functions require device() context, use Rhai engine to evaluate test scripts | Success: All DSL functions tested with valid input, all error scenarios covered, edge cases tested (empty device, missing prefix, out of range IDs), tests verify exact KeyMapping structure created_

## Phase 3: CLI Enhancement (User Interface)

- [ ] 12. Create CLI module structure
  - Files: `keyrx_compiler/src/cli/mod.rs`, `keyrx_compiler/src/cli/compile.rs`, `verify.rs`, `hash.rs`, `parse.rs` (ALL NEW)
  - Define CLI subcommand handlers as public functions
  - Create error types for each subcommand (CompileError, VerifyError, etc.)
  - Set up module exports
  - Purpose: Organize CLI code into separate modules for maintainability
  - _Leverage: None (new module structure)_
  - _Requirements: 3.1-3.5_
  - _Prompt: Role: Rust Software Architect with CLI design expertise | Task: Design CLI module structure with separate files for each subcommand (compile, verify, hash, parse), define error types, set up clean module boundaries, following requirements 3.1-3.5 | Restrictions: Each subcommand in separate file, define handler as pub fn handle_X(...) -> Result<(), XError>, create CompileError, VerifyError, HashError, ParseError (not same as parser ParseError), implement From<std::io::Error> for all error types, follow existing keyrx_compiler module patterns | Success: Clean module structure, each subcommand handler callable from main.rs, error types defined, no circular dependencies_

- [ ] 13. Implement compile subcommand handler
  - File: `keyrx_compiler/src/cli/compile.rs` (continue)
  - Implement `handle_compile(input: &Path, output: &Path) -> Result<(), CompileError>`
  - Call Parser::parse_file() to parse Rhai
  - Call serialize() to generate .krx
  - Write bytes to output file
  - Print success message with file size and SHA256 hash
  - Purpose: Main compilation workflow
  - _Leverage: Parser, serialize() from serialize.rs_
  - _Requirements: 3.1_
  - _Prompt: Role: Rust Backend Developer with file I/O expertise | Task: Implement compile subcommand handler that parses Rhai script, serializes to .krx, writes output file, prints success message with hash, following requirement 3.1, leveraging Parser and serialize() | Restrictions: Must print progress messages to stderr using eprintln!, extract hash from bytes[8..40], calculate file size, print hexadecimal hash using hex::encode(), handle all errors (parse, serialize, I/O) and convert to CompileError, print "✓ Compilation successful" on success | Success: Compiles valid .rhai to .krx, prints informative progress, shows final hash and size, handles errors gracefully with actionable messages_

- [ ] 14. Implement verify subcommand handler
  - File: `keyrx_compiler/src/cli/verify.rs` (continue)
  - Implement `handle_verify(file: &Path) -> Result<(), VerifyError>`
  - Read .krx file bytes
  - Call deserialize() which performs all validation
  - Print step-by-step verification results (magic, version, hash, rkyv)
  - Print final pass/fail result
  - Purpose: Validate .krx file integrity
  - _Leverage: deserialize() from serialize.rs_
  - _Requirements: 3.2_
  - _Prompt: Role: Rust Developer with file validation expertise | Task: Implement verify subcommand that reads .krx file, calls deserialize() for validation, prints step-by-step results, following requirement 3.2, using deserialize() from serialize.rs | Restrictions: deserialize() already validates magic/version/hash/rkyv, print "✓ Magic bytes valid", "✓ Version: 1", "✓ SHA256 hash matches", etc., print "✓ Verification passed" or "✗ Verification failed: {error}" at end, use eprintln! for output, return VerifyError on failure | Success: Valid .krx passes all checks, corrupted files fail with specific error, output is human-readable_

- [ ] 15. Implement hash subcommand handler
  - File: `keyrx_compiler/src/cli/hash.rs` (continue)
  - Implement `handle_hash(file: &Path, verify: bool) -> Result<(), HashError>`
  - Read .krx file and extract embedded hash (bytes 8-40)
  - Print hash in hexadecimal format
  - If --verify flag, compute hash of data section and compare
  - Print match/mismatch result
  - Purpose: Extract and verify embedded SHA256 hash
  - _Leverage: sha2 crate for hash computation_
  - _Requirements: 3.3_
  - _Prompt: Role: Rust Developer with cryptography expertise | Task: Implement hash subcommand that extracts embedded hash from .krx header, optionally verifies it matches data section hash, following requirement 3.3, using sha2::Sha256 for computation | Restrictions: Extract hash from bytes[8..40], print to stdout using println! (not eprintln!), verify flag computes hash of bytes[48..], compare with embedded hash, print "✓ Hash matches" or "✗ Hash mismatch" to stderr, return HashError::HashMismatch on mismatch | Success: Prints hash in hex format, --verify correctly validates hash, errors handled gracefully_

- [ ] 16. Implement parse subcommand handler
  - File: `keyrx_compiler/src/cli/parse.rs` (continue)
  - Implement `handle_parse(input: &Path, json: bool) -> Result<(), ParseError>`
  - Parse Rhai script using Parser
  - If --json flag, serialize ConfigRoot to JSON and print
  - Otherwise, print human-readable summary
  - Purpose: Debug tool - show parsed configuration
  - _Leverage: Parser, serde_json for JSON output_
  - _Requirements: 3.4, 3.5_
  - _Prompt: Role: Rust Developer with JSON serialization expertise | Task: Implement parse subcommand that parses Rhai, outputs JSON or human-readable summary, following requirements 3.4 and 3.5, using Parser and serde_json | Restrictions: Call Parser::parse_file(), if json flag use serde_json::to_string_pretty() and print to stdout, else print summary: version, device count, mapping count per device, use println! for output, return ParseError on failures | Success: --json outputs valid JSON, human mode prints readable summary, both formats show complete config structure_

- [ ] 17. Update main.rs with clap CLI definition
  - File: `keyrx_compiler/src/main.rs` (MODIFY EXISTING)
  - Define CLI using clap derive macros
  - Add subcommands: compile, verify, hash, parse
  - Route to appropriate cli handler functions
  - Set up colored output (respect NO_COLOR env var)
  - Purpose: User-facing CLI entry point
  - _Leverage: clap 4.x derive API, cli module handlers_
  - _Requirements: 3.1-3.10_
  - _Prompt: Role: Rust CLI Developer with clap expertise | Task: Update main.rs to define complete CLI using clap derive macros, add all 4 subcommands with appropriate arguments, route to cli handlers, following requirements 3.1-3.10, using clap 4.x and cli module | Restrictions: Use #[derive(Parser)] for main struct, #[derive(Subcommand)] for subcommands, define Compile { input: PathBuf, #[arg(short, long)] output: PathBuf }, Verify { file: PathBuf }, Hash { file: PathBuf, #[arg(long)] verify: bool }, Parse { input: PathBuf, #[arg(long)] json: bool }, check NO_COLOR env var and disable colored if set, call appropriate cli::handle_X() function, exit with code 0 on success or 1 on error | Success: All subcommands work, --help shows usage, arguments validated by clap, errors handled gracefully, colored output toggles correctly_

- [ ] 18. Write CLI integration tests
  - File: `keyrx_compiler/tests/cli_tests.rs` (ENHANCED)
  - Test compile subcommand creates .krx file
  - Test verify subcommand on valid and invalid .krx
  - Test hash subcommand prints hash correctly
  - Test hash --verify validates hash
  - Test parse subcommand prints summary
  - Test parse --json outputs valid JSON
  - Test --help flag
  - Test invalid subcommand
  - Purpose: Ensure CLI works end-to-end
  - _Leverage: assert_cmd crate for CLI testing_
  - _Requirements: 3.1-3.9, 4.1_
  - _Prompt: Role: QA Engineer with CLI testing expertise | Task: Create comprehensive CLI integration tests using assert_cmd crate, testing all subcommands with valid and invalid inputs, following requirements 3.1-3.9 and 4.1 | Restrictions: Use Command::cargo_bin("keyrx_compiler"), create temp files with tempfile crate, test exit codes (0 for success, 1 for errors), verify stdout/stderr output with .assert(), test help output, test invalid subcommands, clean up temp files | Success: All subcommands tested end-to-end, error cases covered, help text verified, tests run reliably in CI_

## Phase 4: Error Formatting (User Experience)

- [ ] 19. Implement error formatting with code snippets
  - File: `keyrx_compiler/src/error/formatting.rs` (NEW)
  - Implement `format_error(error: &ParseError, file: &Path, source: &str) -> String`
  - Generate colored terminal output with code snippets
  - Show file:line:column location
  - Show 3 lines of context around error
  - Show caret (^) pointing to error column
  - Include helpful suggestions and examples
  - Purpose: User-friendly error messages that guide users to fixes
  - _Leverage: colored crate for terminal colors_
  - _Requirements: 2.8_
  - _Prompt: Role: Rust Developer with terminal UI expertise | Task: Implement error formatter that generates user-friendly colored error messages with code snippets, location info, and fix suggestions, following requirement 2.8, using colored crate | Restrictions: Must show "Error: " in red, file path in blue, code snippet with line numbers, caret (^) pointing to error column in red, "help:" section in green with suggestions, respect NO_COLOR env var, format each ParseError variant with specific help text, include examples in suggestions | Success: Error messages are readable and actionable, code snippets show context, caret points to exact error location, help text is specific to error type, colors work correctly_

- [ ] 20. Add import chain display for errors in imports
  - File: `keyrx_compiler/src/error/formatting.rs` (continue)
  - Extend format_error() to show import chain when error occurs in imported file
  - Display chain: main.rhai → common/vim.rhai (line 5) → error
  - Purpose: Help users debug errors in imported files
  - _Leverage: ParseError should track import stack_
  - _Requirements: 2.7_
  - _Prompt: Role: Rust Developer with error context expertise | Task: Extend error formatter to display import chain when error occurs in imported file, showing path from main file through imports to error location, following requirement 2.7 | Restrictions: ParseError must store import_chain: Vec<(PathBuf, usize)> tracking file and line of each import, format as "main.rhai → a.rhai (line 5) → b.rhai (line 2) → error", show import chain before error message, use "→" arrows in blue | Success: Errors in imported files show full import path, users can trace how imports led to error, import chain display is clear and readable_

- [ ] 21. Write error formatting tests
  - File: `keyrx_compiler/tests/error_formatting_tests.rs` (NEW)
  - Test each ParseError variant produces expected formatted output
  - Test code snippet generation with different line numbers
  - Test caret positioning for different column numbers
  - Test import chain display
  - Test colored vs non-colored output (NO_COLOR)
  - Purpose: Ensure error messages are correct and helpful
  - _Leverage: None (unit tests)_
  - _Requirements: 2.8_
  - _Prompt: Role: QA Engineer with string comparison testing expertise | Task: Create comprehensive tests for error formatting, verifying output format for each error variant, code snippet generation, and color handling, following requirement 2.8 | Restrictions: Test with mock ParseError instances, capture formatted output, verify structure (error message, location, code snippet, caret, help text), test NO_COLOR env var disables colors, use snapshot testing if available (insta crate), verify suggestions are present | Success: All error variants tested, code snippets correct, caret positioning verified, colored/non-colored modes tested, help text present for all errors_

## Phase 5: Documentation (User Enablement)

- [ ] 22. Write DSL Manual
  - File: `docs/DSL_MANUAL.md` (NEW)
  - Overview section: What is keyrx DSL, why Rhai
  - Rhai syntax basics (variables, functions, closures, arrays)
  - Complete function reference with examples
  - Key naming reference (all VK_ codes, MD_00-MD_FE, LK_00-LK_FE)
  - Common patterns and best practices
  - Troubleshooting section with common errors
  - Purpose: Complete reference for users writing configurations
  - _Leverage: Rhai documentation for syntax basics_
  - _Requirements: 5.1_
  - _Prompt: Role: Technical Writer with Rust and scripting language expertise | Task: Write comprehensive DSL manual covering Rhai basics, all keyrx functions (map, tap_hold, when, etc.), key naming conventions, patterns, and troubleshooting, following requirement 5.1, using clear examples and explanations | Restrictions: Must include examples for every function, explain VK_/MD_/LK_ prefix system with visual diagram, list all supported KeyCode names (A-Z, Num0-Num9, F1-F12, etc.), document modifier/lock ID ranges (00-FE), add troubleshooting section with error messages and solutions, keep language clear and beginner-friendly | Success: New users can read manual and write first config in 30 minutes, all functions documented with examples, key naming clear, troubleshooting helps solve common errors_

- [ ] 23. Create example configurations
  - Files: `examples/01-simple-remap.rhai` through `06-advanced-layers.rhai` (ALL NEW)
  - 01: Basic A→B remapping
  - 02: CapsLock→Escape (classic)
  - 03: Vim navigation (MD_00 + HJKL → arrows)
  - 04: Dual-function keys (Space tap=space, hold=ctrl)
  - 05: Multiple devices (different configs per keyboard)
  - 06: Advanced layers (complex multi-modifier setup)
  - Purpose: Show practical usage patterns
  - _Leverage: None (examples)_
  - _Requirements: 5.2, 5.3, 5.4_
  - _Prompt: Role: Technical Writer with keyboard customization expertise | Task: Create 6 example .rhai configurations demonstrating different features and patterns, following requirements 5.2, 5.3, and 5.4, with clear comments explaining each mapping | Restrictions: Each example must have header comment explaining purpose, inline comments for each mapping explaining what it does, expected behavior section, compilation instructions, examples must actually compile without errors, test in CI | Success: Examples cover basic to advanced use cases, comments explain reasoning, examples compile successfully, users can copy-paste and modify for their needs_

- [ ] 24. Update README with quickstart
  - File: `README.md` (MODIFY EXISTING)
  - Add quickstart section (install → write config → compile → load)
  - Link to DSL_MANUAL.md and examples/
  - Add troubleshooting section
  - Add contribution guidelines
  - Purpose: Entry point for new users
  - _Leverage: Existing README structure_
  - _Requirements: 5.5_
  - _Prompt: Role: Technical Writer with developer documentation expertise | Task: Update README.md with comprehensive quickstart guide, links to documentation and examples, troubleshooting tips, following requirement 5.5, maintaining existing README structure | Restrictions: Keep existing project description, add Quickstart section after Introduction, show complete workflow (write .rhai, run compile, verify output), link to DSL_MANUAL.md and examples/, add Troubleshooting section with common issues (permission errors, syntax errors, etc.), add Contributing section, keep README focused (details go in manual) | Success: README guides new users through first compilation, links are correct, troubleshooting helps common issues, contribution process clear_

- [ ] 25. Add CI check for documentation accuracy
  - File: `.github/workflows/test.yml` (MODIFY EXISTING)
  - Add job that compiles all .rhai examples
  - Fail CI if any example fails to compile
  - Extract code blocks from DSL_MANUAL.md and test compilation
  - Purpose: Ensure documentation stays accurate as code evolves
  - _Leverage: Existing CI workflow_
  - _Requirements: 5.5_
  - _Prompt: Role: DevOps Engineer with CI/CD expertise | Task: Add documentation testing to CI workflow that compiles all example .rhai files and verifies they build without errors, following requirement 5.5, extending existing GitHub Actions workflow | Restrictions: Add new job "test-docs" to .github/workflows/test.yml, run after main tests, use `for f in examples/*.rhai; do cargo run -- compile "$f" -o /tmp/test.krx || exit 1; done`, optionally use mdbook or similar to extract code blocks from DSL_MANUAL.md and test them, fail CI if any example fails | Success: CI fails if examples don't compile, documentation stays accurate, examples tested on every PR_

## Phase 6: Integration Testing (Quality Assurance)

- [ ] 26. Write end-to-end workflow tests
  - File: `keyrx_compiler/tests/integration_tests.rs` (ENHANCED)
  - Test complete workflow: write .rhai → compile → verify → parse
  - Test all mapping types in one config
  - Test import resolution (main imports sub-files)
  - Test deterministic compilation (same input → same output)
  - Test error scenarios (syntax error, circular import, etc.)
  - Purpose: Verify entire system works together
  - _Leverage: Existing integration test structure_
  - _Requirements: 4.1_
  - _Prompt: Role: QA Engineer with integration testing expertise | Task: Create comprehensive end-to-end integration tests covering complete workflows from Rhai script to .krx file, following requirement 4.1, extending existing integration_tests.rs | Restrictions: Use tempfile for temporary .rhai and .krx files, test all mapping types (Simple, Modifier, Lock, TapHold, ModifiedOutput, Conditional), test imports (create temp files with import statements), test determinism (compile twice, compare bytes), test error handling (invalid syntax, circular imports), clean up temp files | Success: All workflow scenarios covered, imports tested, determinism verified, errors handled correctly, tests reliable in CI_

- [ ] 27. Add property-based tests for parser
  - File: `keyrx_compiler/tests/property_tests.rs` (ENHANCED)
  - Generate random valid .rhai scripts
  - Verify they all compile without panicking
  - Verify round-trip: .rhai → ConfigRoot → .rhai (if possible)
  - Purpose: Find edge cases and ensure robustness
  - _Leverage: Existing proptest setup, property_tests.rs_
  - _Requirements: 4.2_
  - _Prompt: Role: QA Engineer with property-based testing expertise | Task: Add parser property tests to existing property_tests.rs that generate random valid .rhai scripts and verify they compile without errors, following requirement 4.2, using proptest framework | Restrictions: Generate valid .rhai by combining random map()/tap_hold()/when() calls with valid VK_/MD_/LK_ names, wrap in device() block, feed to Parser::parse_file(), verify Result::Ok, verify no panics (already covered by proptest), run 1000+ iterations | Success: Random valid configs compile without errors, edge cases discovered and handled, proptest runs reliably_

- [ ] 28. Final integration and cleanup
  - Files: All modified files
  - Run full test suite (unit + integration + property)
  - Verify code coverage >90% (using cargo-tarpaulin or similar)
  - Fix any remaining clippy warnings
  - Format all code with rustfmt
  - Update CHANGELOG with new features
  - Purpose: Polish and finalize implementation
  - _Leverage: Existing CI tools (cargo test, clippy, rustfmt)_
  - _Requirements: All_
  - _Prompt: Role: Senior Rust Developer with code quality expertise | Task: Perform final integration, run all tests, verify coverage, fix warnings, format code, update CHANGELOG, ensuring all requirements are met and code is production-ready | Restrictions: Must run `cargo test --workspace`, verify coverage with `cargo tarpaulin --out Stdout`, fix all `cargo clippy` warnings, run `cargo fmt`, update CHANGELOG.md with new features (DSL completion, CLI subcommands, etc.), ensure no TODO comments remain, verify examples compile | Success: All tests pass, coverage >90%, no clippy warnings, code formatted consistently, CHANGELOG updated, ready for release_
