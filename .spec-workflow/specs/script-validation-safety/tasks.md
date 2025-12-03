# Tasks Document: Script Validation & Safety

## Phase 1: Core Validation Infrastructure

- [x] 1. Create ValidationConfig module
  - Files: `core/src/validation/config.rs` (new)
  - Implement ValidationConfig struct with all configurable thresholds
  - Add TOML loading from `~/.config/keyrx/validation.toml`
  - Implement Default trait with sensible values
  - Add `load()` and `load_from_path()` methods
  - _Leverage: existing config patterns, `dirs` crate for config path_
  - _Requirements: REQ-7_
  - _Prompt: Implement the task for spec script-validation-safety, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust config engineer | Task: Create core/src/validation/config.rs with ValidationConfig struct containing: max_errors, max_suggestions, similarity_threshold, blocked_keys_warning_threshold, max_cycle_depth, tap_timeout_warn_range, combo_timeout_warn_range, ui_validation_debounce_ms. Add serde derives, Default impl, load() from ~/.config/keyrx/validation.toml | Restrictions: ≤100 lines; use dirs crate; graceful fallback to defaults | Success: Config loads from file or uses defaults. Mark [-] before starting, use log-implementation after completion, mark [x] when done._

- [x] 2. Create validation module structure
  - Files: `core/src/validation/mod.rs` (new)
  - Create module with public exports for config, types, engine, semantic, conflicts, safety, coverage, suggestions
  - Add `pub mod validation;` to `core/src/lib.rs`
  - _Leverage: existing module patterns in `core/src/`_
  - _Requirements: All_
  - _Prompt: Implement the task for spec script-validation-safety, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust systems engineer | Task: Create core/src/validation/mod.rs with module declarations and public exports for: config, types, engine, semantic, conflicts, safety, coverage, suggestions. Add mod validation to core/src/lib.rs | Restrictions: ≤30 lines; follow existing module patterns; no implementation yet | Success: `cargo check` passes with empty submodules. Mark [-] before starting, use log-implementation after completion, mark [x] when done._

- [x] 3. Define validation types and result structures
  - Files: `core/src/validation/types.rs` (new)
  - Implement ValidationResult, ValidationError, ValidationWarning, CoverageReport, ValidationOptions, SourceLocation, WarningCategory
  - Add serde derives for JSON output
  - _Leverage: existing error types in `core/src/error.rs`_
  - _Requirements: REQ-1, REQ-2, REQ-3, REQ-5_
  - _Prompt: Implement the task for spec script-validation-safety, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust developer specializing in data modeling | Task: Create core/src/validation/types.rs with ValidationResult, ValidationError, ValidationWarning, CoverageReport, ValidationOptions structs as specified in design.md. Add serde Serialize/Deserialize derives | Restrictions: ≤150 lines; use existing patterns from error.rs; include doc comments | Success: All types compile, serde works for JSON. Mark [-] before, log-implementation after, mark [x] complete._

- [x] 4. Implement key name suggestion engine
  - Files: `core/src/validation/suggestions.rs` (new)
  - Implement Levenshtein distance-based similar key suggestions
  - Use `KeyCode::all_names()` as dictionary
  - Use `ValidationConfig.max_suggestions` and `ValidationConfig.similarity_threshold`
  - _Leverage: `core/src/drivers/keycodes/mod.rs` for KeyCode, `config.rs` for thresholds_
  - _Requirements: REQ-1.2, REQ-7_
  - _Prompt: Implement the task for spec script-validation-safety, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Algorithm developer | Task: Create core/src/validation/suggestions.rs with suggest_similar_keys(invalid: &str, config: &ValidationConfig) -> Vec<String> using Levenshtein distance. Add strsim crate to Cargo.toml. Return top config.max_suggestions similar key names within config.similarity_threshold distance | Restrictions: ≤100 lines; use config values not hardcoded numbers | Success: "Escpe" suggests "Escape", "LeftCrtl" suggests "LeftCtrl". Mark [-] before, log-implementation after, mark [x] complete._

## Phase 2: Semantic Validation

- [ ] 5. Implement semantic validator core
  - Files: `core/src/validation/semantic.rs` (new)
  - Validate key names in all PendingOp variants
  - Validate layer references exist when used
  - Validate modifier references exist when used
  - _Leverage: `core/src/scripting/pending_ops.rs`, `suggestions.rs`, `config.rs`_
  - _Requirements: REQ-1.1, REQ-1.3, REQ-1.4, REQ-7_
  - _Prompt: Implement the task for spec script-validation-safety, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust developer specializing in static analysis | Task: Create core/src/validation/semantic.rs with SemanticValidator struct and validate_operations(ops: &[PendingOp], layers: &HashSet<String>, modifiers: &HashSet<String>, config: &ValidationConfig) -> Vec<ValidationError>. Check all key names are valid, all layer refs exist, all modifier refs exist. Pass config to suggest_similar_keys | Restrictions: ≤200 lines; reuse KeyCode::from_name(); use config for suggestions | Success: Invalid keys and undefined layers/modifiers produce errors with suggestions. Mark [-] before, log-implementation after, mark [x] complete._

- [ ] 6. Implement timing validation
  - Files: `core/src/validation/semantic.rs` (extend)
  - Validate timing parameters using `ValidationConfig.tap_timeout_warn_range` and `ValidationConfig.combo_timeout_warn_range`
  - Warn on values outside configured ranges
  - _Leverage: existing `validate_timeout()` in `scripting/builtins.rs`, `config.rs`_
  - _Requirements: REQ-1.1, REQ-7_
  - _Prompt: Implement the task for spec script-validation-safety, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Validation engineer | Task: Extend core/src/validation/semantic.rs to validate timing operations (TapTimeout, ComboTimeout, HoldDelay). Use config.tap_timeout_warn_range and config.combo_timeout_warn_range for bounds | Restrictions: ≤80 lines added; produce warnings not errors; use config values not hardcoded numbers | Success: Extreme timing values produce warnings based on config. Mark [-] before, log-implementation after, mark [x] complete._

## Phase 3: Conflict Detection

- [ ] 7. Implement remap conflict detection
  - Files: `core/src/validation/conflicts.rs` (new)
  - Detect when same key is remapped multiple times
  - Detect when key is both remapped and blocked
  - Detect tap-hold conflicts with simple remaps
  - _Leverage: `PendingOp` enum from `scripting/pending_ops.rs`_
  - _Requirements: REQ-2.1, REQ-2.2, REQ-2.3_
  - _Prompt: Implement the task for spec script-validation-safety, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Conflict analysis engineer | Task: Create core/src/validation/conflicts.rs with ConflictDetector and detect_remap_conflicts(ops: &[PendingOp]) -> Vec<ValidationWarning>. Track key→operation mappings, detect duplicates. Show both conflicting lines | Restrictions: ≤200 lines; track operation indices for line numbers; categorize as WarningCategory::Conflict | Success: Duplicate remaps and remap+block conflicts detected. Mark [-] before, log-implementation after, mark [x] complete._

- [ ] 8. Implement combo shadowing detection
  - Files: `core/src/validation/conflicts.rs` (extend)
  - Detect when combo keys overlap (e.g., [A,S] shadows [A,S,D])
  - _Leverage: existing combo parsing_
  - _Requirements: REQ-2.4_
  - _Prompt: Implement the task for spec script-validation-safety, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Combo analysis engineer | Task: Extend core/src/validation/conflicts.rs with detect_combo_shadowing(ops: &[PendingOp]) -> Vec<ValidationWarning>. Detect when one combo's keys are a subset of another (potential shadowing) | Restrictions: ≤80 lines added; only warn if subset relationship exists | Success: Overlapping combos produce warnings. Mark [-] before, log-implementation after, mark [x] complete._

- [ ] 9. Implement circular remap detection
  - Files: `core/src/validation/conflicts.rs` (extend)
  - Detect A→B, B→A circular dependencies
  - Detect longer chains using `ValidationConfig.max_cycle_depth`
  - _Leverage: `config.rs` for max depth_
  - _Requirements: REQ-3.4, REQ-7_
  - _Prompt: Implement the task for spec script-validation-safety, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Graph algorithm developer | Task: Extend core/src/validation/conflicts.rs with detect_circular_remaps(ops: &[PendingOp], config: &ValidationConfig) -> Vec<ValidationWarning>. Build directed graph of remaps, detect cycles using DFS up to config.max_cycle_depth | Restrictions: ≤100 lines added; use config.max_cycle_depth not hardcoded value | Success: A→B→A and longer cycles detected. Mark [-] before, log-implementation after, mark [x] complete._

## Phase 4: Safety Analysis

- [ ] 10. Implement safety analyzer
  - Files: `core/src/validation/safety.rs` (new)
  - Warn on Escape key remapping/blocking
  - Warn on emergency exit combo interference
  - Warn on blocking all modifiers
  - Use `ValidationConfig.blocked_keys_warning_threshold` for blocked keys warning
  - _Leverage: `EMERGENCY_EXIT_KEYS` from `drivers/emergency_exit.rs`, `config.rs`_
  - _Requirements: REQ-3.1, REQ-3.2, REQ-3.3, REQ-3.5, REQ-7_
  - _Prompt: Implement the task for spec script-validation-safety, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Security-focused developer | Task: Create core/src/validation/safety.rs with SafetyAnalyzer and analyze_safety(ops: &[PendingOp], config: &ValidationConfig) -> Vec<ValidationWarning>. Check for Escape remap/block, emergency combo keys (Ctrl+Alt+Shift+Escape), blocking both Left/Right of same modifier type, blocked keys > config.blocked_keys_warning_threshold | Restrictions: ≤200 lines; categorize as WarningCategory::Safety; use config values | Success: Dangerous patterns produce clear warnings. Mark [-] before, log-implementation after, mark [x] complete._

## Phase 5: Coverage Analysis

- [ ] 11. Implement coverage analyzer
  - Files: `core/src/validation/coverage.rs` (new)
  - Categorize keys by behavior type (remapped, blocked, tap-hold, combo, unaffected)
  - Track per-layer coverage
  - _Requirements: REQ-5.1, REQ-5.2, REQ-5.3_
  - _Prompt: Implement the task for spec script-validation-safety, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Coverage analysis developer | Task: Create core/src/validation/coverage.rs with CoverageAnalyzer and analyze_coverage(ops: &[PendingOp]) -> CoverageReport. Categorize all affected keys by type, track layer-specific mappings | Restrictions: ≤150 lines; use HashSet for deduplication | Success: Coverage report shows all affected keys by category. Mark [-] before, log-implementation after, mark [x] complete._

- [ ] 12. Implement ASCII keyboard visualization
  - Files: `core/src/validation/coverage.rs` (extend)
  - Render ANSI keyboard layout with affected keys highlighted
  - Use symbols: [R]emap, [B]lock, [T]ap-hold, [C]ombo, [ ] unaffected
  - _Leverage: existing ANSI layout from `drivers/keycodes/layouts/`_
  - _Requirements: REQ-5.4_
  - _Prompt: Implement the task for spec script-validation-safety, first run spec-workflow-guide to get the workflow guide then implement the task: Role: TUI developer | Task: Extend core/src/validation/coverage.rs with render_ascii_keyboard(coverage: &CoverageReport) -> String. Render ANSI 104-key layout with key indicators | Restrictions: ≤150 lines added; use simple ASCII art; include legend | Success: ASCII keyboard shows affected keys visually. Mark [-] before, log-implementation after, mark [x] complete._

## Phase 6: Validation Engine

- [ ] 13. Implement validation engine orchestrator
  - Files: `core/src/validation/engine.rs` (new)
  - Orchestrate all validation passes
  - Load and apply ValidationConfig
  - Collect and aggregate results respecting `config.max_errors`
  - _Leverage: all validator modules, `config.rs`_
  - _Requirements: REQ-1.5, REQ-1.6, REQ-2.6, REQ-3.6, REQ-7_
  - _Prompt: Implement the task for spec script-validation-safety, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Integration architect | Task: Create core/src/validation/engine.rs with ValidationEngine that loads ValidationConfig and validate(script: &str, options: ValidationOptions) -> ValidationResult. Parse script with Rhai, collect PendingOps, run all validators passing config, aggregate results respecting config.max_errors | Restrictions: ≤200 lines; handle parse errors gracefully; use config throughout | Success: Full validation pipeline works end-to-end with config. Mark [-] before, log-implementation after, mark [x] complete._

- [ ] 14. Add operation collection for validation
  - Files: `core/src/validation/engine.rs` (extend)
  - Extract layer and modifier definitions during parsing
  - Track operation source locations
  - _Leverage: `LayerView`, `ModifierView` from `scripting/builtins.rs`_
  - _Requirements: REQ-1.3, REQ-1.4_
  - _Prompt: Implement the task for spec script-validation-safety, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Parser integration developer | Task: Extend core/src/validation/engine.rs to collect defined layers/modifiers during validation parse. Use existing LayerView/ModifierView patterns. Track source positions if Rhai provides them | Restrictions: ≤100 lines added; don't duplicate runtime logic | Success: Validator knows which layers/modifiers are defined. Mark [-] before, log-implementation after, mark [x] complete._

## Phase 7: CLI Integration

- [ ] 15. Enhance keyrx check command
  - Files: `core/src/cli/commands/check.rs` (rewrite)
  - Replace syntax-only check with full semantic validation
  - Add --strict, --no-warnings, --coverage, --visual, --config, --show-config flags
  - Support JSON output
  - _Leverage: `ValidationEngine`, `OutputWriter`, `ValidationConfig`_
  - _Requirements: REQ-1, REQ-2, REQ-3, REQ-5, REQ-7_
  - _Prompt: Implement the task for spec script-validation-safety, first run spec-workflow-guide to get the workflow guide then implement the task: Role: CLI developer | Task: Rewrite core/src/cli/commands/check.rs to use ValidationEngine. Add clap flags: --strict (warnings as errors), --no-warnings, --coverage (include coverage report), --visual (ASCII keyboard), --config <path> (custom config), --show-config (display current config). Output errors/warnings with colors, support --json | Restrictions: ≤250 lines; maintain backward compatibility for basic check; exit code 0=valid, 1=errors, 2=warnings in strict mode | Success: `keyrx check --coverage --visual script.rhai` shows full validation. Mark [-] before, log-implementation after, mark [x] complete._

- [ ] 16. Enhance keyrx simulate for dry-run
  - Files: `core/src/cli/commands/simulate.rs` (extend)
  - Add --script flag to load script before simulation
  - Show both tap and hold behaviors for tap-hold keys
  - _Leverage: existing simulate infrastructure_
  - _Requirements: REQ-4.1, REQ-4.2, REQ-4.3_
  - _Prompt: Implement the task for spec script-validation-safety, first run spec-workflow-guide to get the workflow guide then implement the task: Role: CLI developer | Task: Extend core/src/cli/commands/simulate.rs with --script flag. When provided, load script first, then simulate with that config. For tap-hold keys, show "Tap: X, Hold: Y (threshold: Zms)" | Restrictions: ≤100 lines added; reuse existing simulation engine | Success: `keyrx simulate --script config.rhai --input CapsLock` shows tap-hold behavior. Mark [-] before, log-implementation after, mark [x] complete._

- [ ] 17. Add interactive simulation mode
  - Files: `core/src/cli/commands/simulate.rs` (extend)
  - Add --interactive flag for REPL-style simulation
  - User types key names, system shows output
  - _Leverage: existing REPL patterns from `cli/commands/repl.rs`_
  - _Requirements: REQ-4.5_
  - _Prompt: Implement the task for spec script-validation-safety, first run spec-workflow-guide to get the workflow guide then implement the task: Role: CLI developer | Task: Extend core/src/cli/commands/simulate.rs with --interactive flag. Start a REPL loop where user types key names and system shows what output would be produced. Support 'quit' to exit | Restrictions: ≤100 lines added; reuse rustyline from REPL; simple prompt "simulate> " | Success: Interactive mode allows testing keys one by one. Mark [-] before, log-implementation after, mark [x] complete._

## Phase 8: FFI & Flutter Integration

- [ ] 18. Add validation FFI exports
  - Files: `core/src/ffi/exports_validation.rs` (new)
  - Export validate_script() returning JSON result
  - Export get_key_suggestions() for autocomplete
  - _Leverage: existing FFI patterns in `ffi/exports_*.rs`_
  - _Requirements: REQ-6.1_
  - _Prompt: Implement the task for spec script-validation-safety, first run spec-workflow-guide to get the workflow guide then implement the task: Role: FFI engineer | Task: Create core/src/ffi/exports_validation.rs with extern "C" functions: keyrx_validate_script(script: *const c_char) -> *mut c_char (JSON result), keyrx_suggest_keys(partial: *const c_char) -> *mut c_char (JSON array). Add to ffi/mod.rs | Restrictions: ≤150 lines; follow existing FFI patterns; free strings properly | Success: FFI exports work from Dart. Mark [-] before, log-implementation after, mark [x] complete._

- [ ] 19. Update Flutter bridge for validation
  - Files: `ui/lib/ffi/bridge.dart` (extend)
  - Add validateScript() method returning ValidationResult
  - Add suggestKeys() for autocomplete
  - _Leverage: existing bridge patterns_
  - _Requirements: REQ-6.1, REQ-6.4_
  - _Prompt: Implement the task for spec script-validation-safety, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter FFI developer | Task: Extend ui/lib/ffi/bridge.dart with validateScript(String script) -> Future<ValidationResult> and suggestKeys(String partial) -> Future<List<String>>. Parse JSON from FFI, create Dart model classes for ValidationResult, ValidationError, ValidationWarning | Restrictions: ≤150 lines added; create models in lib/models/validation.dart | Success: Flutter can call validation and get structured results. Mark [-] before, log-implementation after, mark [x] complete._

- [ ] 20. Add validation display to editor page
  - Files: `ui/lib/pages/editor_page.dart` (extend)
  - Display validation errors/warnings inline
  - Highlight problematic lines
  - Show suggestions on hover
  - Use `ValidationConfig.ui_validation_debounce_ms` for debounce
  - _Leverage: existing editor infrastructure, config from FFI_
  - _Requirements: REQ-6.2, REQ-6.3, REQ-6.4, REQ-6.5, REQ-7_
  - _Prompt: Implement the task for spec script-validation-safety, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter UI developer | Task: Extend ui/lib/pages/editor_page.dart to call validateScript on text change (debounced using config.ui_validation_debounce_ms from ValidationConfig). Display errors/warnings below editor. Highlight error lines in red. Show tooltip with suggestions on error tap | Restrictions: ≤200 lines added; use config for debounce; allow save even with errors (show warning dialog) | Success: Editor shows real-time validation feedback. Mark [-] before, log-implementation after, mark [x] complete._

## Phase 9: Testing & Documentation

- [ ] 21. Add unit tests for ValidationConfig
  - Files: `core/src/validation/config.rs` (extend with #[cfg(test)])
  - Test config loading from file
  - Test default values
  - Test malformed config handling
  - _Leverage: existing test patterns_
  - _Requirements: REQ-7_
  - _Prompt: Implement the task for spec script-validation-safety, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Test engineer | Task: Add #[cfg(test)] module to config.rs. Test: load() returns defaults when no file, load_from_path() parses valid TOML, malformed TOML returns None, Default::default() has expected values | Restrictions: ≤100 lines; use tempfile for test config files | Success: `cargo test validation::config` passes. Mark [-] before, log-implementation after, mark [x] complete._

- [ ] 22. Add unit tests for validators
  - Files: `core/src/validation/semantic.rs`, `conflicts.rs`, `safety.rs` (extend with #[cfg(test)])
  - Test each validation rule with config parameters
  - Test edge cases and false positive prevention
  - _Leverage: existing test patterns_
  - _Requirements: All_
  - _Prompt: Implement the task for spec script-validation-safety, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Test engineer | Task: Add #[cfg(test)] modules to each validator file. Test: invalid key detection, layer/modifier undefined, duplicate remaps, combo shadowing, circular remaps, all safety patterns. Pass custom ValidationConfig to test config-driven behavior. Ensure valid scripts produce no errors | Restrictions: ≤200 lines per file; test both positive and negative cases; test with different config values | Success: `cargo test validation` passes with good coverage. Mark [-] before, log-implementation after, mark [x] complete._

- [ ] 23. Add integration tests
  - Files: `core/tests/validation_integration.rs` (new)
  - Test full validation pipeline with real scripts
  - Test CLI output format with --config flag
  - Test FFI roundtrip
  - _Requirements: All_
  - _Prompt: Implement the task for spec script-validation-safety, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Integration test engineer | Task: Create core/tests/validation_integration.rs. Test ValidationEngine with scripts from scripts/ directory. Verify no false positives on valid example scripts. Test JSON output parsing. Test CLI exit codes. Test --config flag with custom config | Restrictions: ≤200 lines; use existing test fixtures | Success: Integration tests pass. Mark [-] before, log-implementation after, mark [x] complete._

- [ ] 24. Update documentation
  - Files: `README.md` (extend)
  - Document validation features in README
  - Add examples of check command usage including --config
  - Document validation.toml config file format
  - _Requirements: All_
  - _Prompt: Implement the task for spec script-validation-safety, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical writer | Task: Add "## Script Validation" section to README.md. Document keyrx check flags (--strict, --coverage, --visual, --json, --config, --show-config). Show example output. Explain error/warning categories. Document ~/.config/keyrx/validation.toml format | Restrictions: ≤80 lines added; include code examples | Success: Users understand validation features and config from README. Mark [-] before, log-implementation after, mark [x] complete._

- [ ] 25. Verify all requirements met
  - Files: N/A (manual verification)
  - Run full test suite
  - Verify each requirement (REQ-1 through REQ-7) has corresponding tests
  - Test on sample user scripts
  - _Requirements: All_
  - _Prompt: Implement the task for spec script-validation-safety, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA engineer | Task: Verify all REQ-1 through REQ-7 requirements are implemented and tested. Run `cargo test`, `flutter test`, manual CLI testing. Create checklist mapping requirements to tests. Verify config-driven behavior works as expected | Restrictions: Document any gaps found; create follow-up issues if needed | Success: All requirements verified as implemented. Mark [-] before, log-implementation after, mark [x] complete._
