# Tasks Document

## Phase 1: Audit and Planning

- [x] 1.1 Run file size audit for all Rust source files
  - Command: `find core/src -name "*.rs" -exec wc -l {} + | awk '$1 > 500' | sort -rn`
  - Identify all files exceeding 500-line limit
  - Create ranked list with line counts
  - Purpose: Get comprehensive view of scope
  - _Leverage: Unix utilities_
  - _Requirements: 1.1_
  - _Prompt: Implement the task for spec split-large-files. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Code metrics analyst

Task: Run file size audit following requirement 1.1. Execute `find core/src -name "*.rs" -exec wc -l {} + | awk '$1 > 500' | sort -rn > /tmp/oversized_files.txt`. Review output and create formatted list of files with line counts. Identify top 10 largest files. Document results in .spec-workflow/specs/split-large-files/audit_results.md with table format.

Restrictions: Just audit and document, do not make code changes. Ensure all 56+ files are captured. Verify counts match evaluation report.

Success: Complete audit results documented. All files >500 lines identified. Top 10 confirmed: bindings.rs (1893), state/mod.rs (1570), transitions/log.rs (1403), etc. Ready for splitting work.

After completing:
1. Change `[ ]` to `[-]` in tasks.md before starting
2. Run audit and document results
3. Use log-implementation to record audit with files identified
4. Change `[-]` to `[x]` when complete_

- [x] 1.2 Analyze top 10 files for logical split points
  - Files: Top 10 from audit results
  - Review each file to identify domain boundaries, function groups, type definitions
  - Document proposed module structure for each file
  - Purpose: Plan splits before implementation
  - _Leverage: Audit results from 1.1_
  - _Requirements: 1.1_
  - _Prompt: Implement the task for spec split-large-files. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Software architect with Rust expertise

Task: Analyze top 10 files following requirement 1.1. For each file: (1) Read entire file, (2) Identify logical groupings (functions, types, domains), (3) Propose module split with 3-6 submodules, (4) Estimate line counts per module, (5) Identify any challenges (circular deps, shared state). Document analysis in .spec-workflow/specs/split-large-files/split_plans.md with section per file.

Restrictions: Analysis only, no code changes. Consider domain boundaries, not just line counts. Each proposed module should be 200-450 lines. Flag any concerns about circular dependencies or API compatibility.

Success: Split plans documented for all top 10 files. Each plan includes: current structure, proposed modules, estimated lines, public API preservation strategy, potential issues. Plans are actionable and detailed.

After completing:
1. Change `[ ]` to `[-]` in tasks.md
2. Analyze files and create plans
3. Use log-implementation to record analysis
4. Change `[-]` to `[x]` when complete_

## Phase 2: Split Top 3 Files (Highest Impact)

- [x] 2.1 Split scripting/bindings.rs (1,893 lines → 8 modules)
  - File: core/src/scripting/bindings.rs
  - Create bindings/ directory with mod.rs, keyboard.rs, layers.rs, modifiers.rs, timing.rs, row_col.rs
  - Move register_* functions to appropriate submodules
  - Re-export all public functions from mod.rs
  - Update internal imports
  - Purpose: Split largest file into focused domain modules
  - _Leverage: Split plan from 1.2, existing module patterns_
  - _Requirements: 2.1, 2.2, 2.3_
  - _Prompt: Implement the task for spec split-large-files. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Rust refactoring specialist

Task: Split `core/src/scripting/bindings.rs` following requirements 2.1-2.3. Steps: (1) Create `bindings/` directory, (2) Create `mod.rs` with module declarations and re-exports, (3) Create `keyboard.rs` - move remap, block, pass, tap_hold functions, (4) Create `layers.rs` - move layer_define, layer_map, layer_push/pop/toggle, (5) Create `modifiers.rs` - move modifier functions and one_shot, (6) Create `timing.rs` - move timeout configuration, (7) Create `row_col.rs` - move all _rc variant functions, (8) Update imports to use `super::` or `crate::`, (9) Ensure `register_all_functions` in mod.rs works, (10) Delete original bindings.rs.

Restrictions: Maintain exact public API. All `use keyrx_core::scripting::bindings::register_*` must work unchanged. Do not modify function logic. Keep each module 200-450 lines. Preserve all documentation and `#[rhai_doc]` attributes.

Success: bindings/ directory created with 6 submodules. mod.rs is 200 lines. Each submodule 300-450 lines. Public API unchanged. Imports work. `cargo build` succeeds. `cargo test` passes. `cargo clippy` clean.

After completing:
1. Mark [-] before starting
2. Implement split carefully
3. Test after completion (cargo build, cargo test, cargo clippy)
4. Use log-implementation with comprehensive artifacts (files created/modified, functions moved to each module)
5. Mark [x] when tests pass_

- [x] 2.2 Split engine/state/mod.rs (1,570 lines → 4 modules)
  - File: core/src/engine/state/mod.rs
  - Create key_state.rs, layer_state.rs, modifier_state.rs submodules
  - Keep EngineState struct in mod.rs, move state implementations to submodules
  - Re-export public types
  - Purpose: Organize state management by responsibility
  - _Leverage: Existing state/ directory structure_
  - _Requirements: 2.1, 2.2, 2.3_
  - _Prompt: Implement the task for spec split-large-files. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Rust refactoring specialist

Task: Split `core/src/engine/state/mod.rs` following requirements 2.1-2.3. Create submodules: (1) `key_state.rs` - physical/virtual key tracking logic and KeyState type, (2) `layer_state.rs` - layer stack, activation, LayerState type, (3) `modifier_state.rs` - modifier tracking, ModifierState type. Keep `EngineState` main struct in mod.rs with public API methods. Move state field implementations to submodules. Re-export types from mod.rs. Update imports.

Restrictions: EngineState public API must remain unchanged. Internal field access through private fields. Each module 350-500 lines. Preserve all tests. No logic changes.

Success: 4 modules created (mod.rs + 3 submodules). mod.rs ~200 lines with EngineState struct. Submodules 400-500 lines each. Public API unchanged. All tests pass. Clippy clean.

After completing:
1. Mark [-] before starting
2. Implement split
3. Run full test suite
4. Use log-implementation with artifacts detailing types/functions moved
5. Mark [x] after tests pass_

- [x] 2.3 Split engine/transitions/log.rs (1,403 lines → 4 modules)
  - File: core/src/engine/transitions/log.rs
  - Create log/ directory with mod.rs, entry.rs, query.rs, analysis.rs
  - Move TransitionLog entry types to entry.rs
  - Move query/filter logic to query.rs
  - Move analysis/reporting to analysis.rs
  - Purpose: Separate logging concerns
  - _Leverage: Split plan from 1.2_
  - _Requirements: 2.1, 2.2, 2.3_
  - _Prompt: Implement the task for spec split-large-files. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Rust refactoring specialist

Task: Split `core/src/engine/transitions/log.rs` following requirements 2.1-2.3. Create `transitions/log/` directory. Create submodules: (1) `mod.rs` - TransitionLog struct and public API, (2) `entry.rs` - entry types, formatting, serialization, (3) `query.rs` - query methods, filtering, searching, (4) `analysis.rs` - analysis, statistics, reporting. Re-export types. Update imports.

Restrictions: TransitionLog public API unchanged. Each module 300-450 lines. All tests pass. No logic changes. Preserve all serialization logic.

Success: log/ directory with 4 modules. mod.rs ~200 lines. Submodules 350-450 lines. API compatible. Tests pass. Clippy clean.

After completing:
1. Mark [-] before starting
2. Implement split
3. Full testing (build, test, clippy)
4. Use log-implementation with detailed artifacts
5. Mark [x] when done_

## Phase 3: Split Files 4-7

- [x] 3.1 Split bin/keyrx.rs (1,382 lines → 4 modules)
  - File: core/src/bin/keyrx.rs
  - Create commands_core.rs, commands_config.rs, commands_test.rs modules
  - Keep main() and CLI arg parsing in keyrx.rs
  - Move command implementations to appropriate modules
  - Purpose: Organize CLI commands by category
  - _Leverage: Clap command structure_
  - _Requirements: 2.1, 2.2, 2.3_
  - _Prompt: Implement the task for spec split-large-files. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: CLI application developer

Task: Split `core/src/bin/keyrx.rs` following requirements 2.1-2.3. Keep main() and Cli struct in keyrx.rs. Create: (1) `commands_core.rs` - run, simulate, check, discover commands, (2) `commands_config.rs` - devices, hardware, layout, keymap, runtime commands, (3) `commands_test.rs` - test, replay, analyze, uat, regression, doctor, repl. Move command handler functions. Import and call from main dispatch.

Restrictions: Binary must work identically. All commands must execute. Each module 400-500 lines. Main file <200 lines. No command logic changes.

Success: keyrx.rs ~200 lines. 3 command modules created 400-500 lines each. Binary compiles. All CLI commands work (`keyrx --help`, `keyrx run --help`, etc.).

After completing:
1. Mark [-] before starting
2. Implement split
3. Test all major commands (run, simulate, test)
4. Use log-implementation with artifacts
5. Mark [x] when verified_

- [x] 3.2 Split scripting/docs/generators/html.rs (1,069 lines → 3 modules)
  - File: core/src/scripting/docs/generators/html.rs
  - Create html/ directory with mod.rs, templates.rs, rendering.rs
  - Move HTML template functions to templates.rs
  - Move rendering logic to rendering.rs
  - Purpose: Separate HTML generation concerns
  - _Leverage: Existing generators/ structure_
  - _Requirements: 2.1, 2.2, 2.3_
  - _Prompt: Implement the task for spec split-large-files. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Rust refactoring specialist

Task: Split html.rs following requirements 2.1-2.3. Create `html/` directory: (1) `mod.rs` - HtmlGenerator struct and public API, (2) `templates.rs` - HTML template string functions, (3) `rendering.rs` - rendering logic, traversal, output generation. Re-export public API.

Restrictions: Doc generation must work identically. Each module <500 lines. API unchanged. HTML output identical. Tests pass.

Success: html/ directory with 3 modules. mod.rs ~200 lines. Submodules 350-450 lines. Doc generation works. Tests pass.

After completing:
1. Mark [-], implement, test, log, mark [x]_

- [x] 3.3 Split validation/engine.rs (968 lines → 3 modules)
  - File: core/src/validation/engine.rs
  - Create engine/ directory with mod.rs, rules.rs, report.rs
  - Move validation rules to rules.rs
  - Move error reporting to report.rs
  - Purpose: Separate validation concerns
  - _Leverage: Existing validation/ structure_
  - _Requirements: 2.1, 2.2, 2.3_
  - _Prompt: Implement the task for spec split-large-files. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Rust refactoring specialist

Task: Split validation/engine.rs. Create `engine/`: (1) `mod.rs` - public validation API, (2) `rules.rs` - validation rule implementations and checks, (3) `report.rs` - error formatting, reporting, suggestions. Re-export API.

Restrictions: Validation behavior unchanged. Each module <500 lines. All validation tests pass. Error messages identical.

Success: engine/ with 3 modules. mod.rs ~200 lines. Others 300-400 lines. Validation works. Tests pass.

After completing:
1. Mark [-], implement, test, log, mark [x]_

- [x] 3.4 Split config/loader.rs (949 lines → 3 modules)
  - File: core/src/config/loader.rs
  - Create loader/ directory with mod.rs, parsing.rs, validation.rs
  - Move parsing logic to parsing.rs
  - Move validation to validation.rs
  - Purpose: Separate config loading concerns
  - _Leverage: Existing config/ structure_
  - _Requirements: 2.1, 2.2, 2.3_
  - _Prompt: Implement the task for spec split-large-files. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Rust refactoring specialist

Task: Split config/loader.rs. Create `loader/`: (1) `mod.rs` - ConfigManager and public API, (2) `parsing.rs` - YAML/TOML parsing, file I/O, (3) `validation.rs` - config validation, normalization, defaults. Re-export API.

Restrictions: Config loading behavior unchanged. Each module <500 lines. All config tests pass. File formats supported unchanged.

Success: loader/ with 3 modules. mod.rs ~200 lines. Others 300-400 lines. Config loading works. Tests pass.

After completing:
1. Mark [-], implement, test, log, mark [x]_

## Phase 4: Split Files 8-10

- [x] 4.1 Split registry/profile.rs (918 lines → 3 modules)
  - File: core/src/registry/profile.rs
  - Create profile/ directory with mod.rs, storage.rs, resolution.rs
  - Move persistence logic to storage.rs
  - Move resolution logic to resolution.rs
  - Purpose: Separate profile management concerns
  - _Leverage: Existing registry/ structure_
  - _Requirements: 2.1, 2.2, 2.3_
  - _Prompt: Implement the task for spec split-large-files. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Rust refactoring specialist

Task: Split registry/profile.rs. Create `profile/`: (1) `mod.rs` - ProfileRegistry and public CRUD API, (2) `storage.rs` - persistence, file I/O, serialization, (3) `resolution.rs` - profile resolution, priority handling, slot management. Re-export API.

Restrictions: Registry behavior unchanged. Each module <500 lines. All registry tests pass. Profile resolution logic preserved.

Success: profile/ with 3 modules. mod.rs ~200 lines. Others 300-400 lines. Registry works. Tests pass.

After completing:
1. Mark [-], implement, test, log, mark [x]_

- [x] 4.2 Split engine/advanced.rs (906 lines → 3 modules)
  - File: core/src/engine/advanced.rs
  - Create advanced/ directory with mod.rs, combos.rs, sequences.rs
  - Move combo detection to combos.rs
  - Move macro/sequence logic to sequences.rs
  - Purpose: Separate advanced feature concerns
  - _Leverage: Existing engine/ structure_
  - _Requirements: 2.1, 2.2, 2.3_
  - _Prompt: Implement the task for spec split-large-files. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Rust refactoring specialist

Task: Split engine/advanced.rs. Create `advanced/`: (1) `mod.rs` - public advanced features API, (2) `combos.rs` - combo detection, matching, handling, (3) `sequences.rs` - macro playback, sequence execution. Re-export API.

Restrictions: Advanced features behavior unchanged. Each module <500 lines. All advanced feature tests pass. Combo/macro logic preserved.

Success: advanced/ with 3 modules. mod.rs ~200 lines. Others 300-400 lines. Advanced features work. Tests pass.

After completing:
1. Mark [-], implement, test, log, mark [x]_

- [ ] 4.3 Split cli/commands/run.rs (899 lines → 3 modules)
  - File: core/src/cli/commands/run.rs
  - Create run/ directory with mod.rs, setup.rs, execution.rs
  - Move setup logic to setup.rs
  - Move execution loop to execution.rs
  - Purpose: Separate run command concerns
  - _Leverage: Existing commands/ structure_
  - _Requirements: 2.1, 2.2, 2.3_
  - _Prompt: Implement the task for spec split-large-files. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Rust refactoring specialist

Task: Split cli/commands/run.rs. Create `run/`: (1) `mod.rs` - RunCommand struct, args, dispatch, (2) `setup.rs` - engine initialization, config loading, device setup, (3) `execution.rs` - main event loop, signal handling. Re-export command.

Restrictions: Run command behavior unchanged. Each module <500 lines. Run command works identically. All run tests pass.

Success: run/ with 3 modules. mod.rs ~200 lines. Others 300-400 lines. `keyrx run` command works. Tests pass.

After completing:
1. Mark [-], implement, test, log, mark [x]_

## Phase 5: Verification and Documentation

- [ ] 5.1 Run full test suite after all splits
  - Command: `cargo test --all`
  - Verify all 2,440+ tests pass
  - Check for any failures or warnings
  - Purpose: Ensure all splits maintain correctness
  - _Leverage: Existing test suite_
  - _Requirements: 6.1_
  - _Prompt: Implement the task for spec split-large-files. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: QA validation specialist

Task: Run comprehensive test suite following requirement 6.1. Execute `cargo test --all --verbose` and capture output. Verify all tests pass. If any fail, document failures with file, test name, error. Run multiple times to check for flaky tests. Record pass rate and any issues.

Restrictions: Verification only, do not fix issues in this task. Document all failures comprehensively for follow-up tasks.

Success: Test results documented. All tests pass (or failures documented). No regressions from splits. Test output saved for reference.

After completing:
1. Mark [-], run tests, document results, log findings, mark [x]_

- [ ] 5.2 Verify incremental compilation improvement
  - Command: Touch various submodules and measure rebuild time
  - Compare against baseline (touch large file before split)
  - Document improvement percentage
  - Purpose: Quantify compilation speed benefit
  - _Leverage: cargo build --timings_
  - _Requirements: Performance requirements_
  - _Prompt: Implement the task for spec split-large-files. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Build performance specialist

Task: Measure incremental compilation improvement. For each of top 10 splits: (1) Make trivial change to one submodule (add comment), (2) Run `time cargo build --lib`, (3) Record build time, (4) Calculate average across all. Compare to historical baselines from before splits. Document in .spec-workflow/specs/split-large-files/performance_results.md.

Restrictions: Measurement only. Use clean builds. Run each test 3 times and average. Control for other processes.

Success: Build times measured for all major splits. Improvement quantified (target: 20-30% faster). Results documented with before/after comparison. Clear evidence of benefit.

After completing:
1. Mark [-], measure builds, document results, log data, mark [x]_

- [ ] 5.3 Run clippy and ensure no new warnings
  - Command: `cargo clippy --all-targets -- -D warnings`
  - Verify no warnings introduced by splits
  - Check for common issues (unused imports, dead code, etc.)
  - Purpose: Maintain code quality
  - _Leverage: cargo clippy_
  - _Requirements: 6.1_
  - _Prompt: Implement the task for spec split-large-files. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Code quality specialist

Task: Run clippy validation. Execute `cargo clippy --all-targets -- -D warnings`. Check for: (1) Unused imports from refactoring, (2) Dead code from moves, (3) Visibility issues, (4) Module organization warnings. Document any warnings.

Restrictions: Verification only. If warnings exist, document but don't fix. This identifies cleanup needed.

Success: Clippy results documented. Either all clean, or warnings listed for cleanup task. Code quality status known.

After completing:
1. Mark [-], run clippy, document, log, mark [x]_

- [ ] 5.4 Verify all files under 500-line limit
  - Command: `find core/src -name "*.rs" -exec wc -l {} + | awk '$1 > 500'`
  - Ensure no files exceed limit after splits
  - Document compliance
  - Purpose: Verify requirement met
  - _Leverage: File size audit_
  - _Requirements: 1.1, 2.1_
  - _Prompt: Implement the task for spec split-large-files. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Code metrics specialist

Task: Verify file size compliance. Run `find core/src -name "*.rs" -exec wc -l {} + | awk '$1 > 500' | sort -rn`. Should return empty (no files >500 lines) or very short list. If any files still exceed: (1) Document which files, (2) Check if they were in scope, (3) Flag for additional splits if needed.

Restrictions: Verification only. Document any remaining violations but don't fix.

Success: File size verification complete. Either: (1) All files <500 lines (goal met), or (2) Remaining violations documented for follow-up. Top 10 files confirmed under limit.

After completing:
1. Mark [-], verify, document, log, mark [x]_

- [ ] 5.5 Update module documentation
  - Files: All mod.rs files created during splits
  - Add comprehensive module-level documentation
  - Explain organization and submodule purposes
  - Purpose: Document new structure
  - _Leverage: Split plans from Phase 1_
  - _Requirements: 5.1_
  - _Prompt: Implement the task for spec split-large-files. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Technical documentation specialist

Task: Add/update module documentation following requirement 5.1. For each split module's mod.rs, add doc comment explaining: (1) Overall module purpose, (2) List of submodules with brief descriptions, (3) Public API overview, (4) Usage example if appropriate. Use consistent format across all modules.

Restrictions: Documentation only, no code changes. Use clear, concise language. Include intra-doc links to submodules.

Success: All split modules have comprehensive mod.rs documentation. Submodules explained. Examples provided. Documentation generates cleanly (`cargo doc`).

After completing:
1. Mark [-], document all modules, log updates, mark [x]_

- [ ] 5.6 Create migration guide for contributors
  - File: .spec-workflow/specs/split-large-files/MIGRATION_GUIDE.md
  - Document where old files are now located
  - Provide import update examples
  - Purpose: Help contributors navigate new structure
  - _Leverage: All split changes_
  - _Requirements: 5.1_
  - _Prompt: Implement the task for spec split-large-files. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Technical writer

Task: Create migration guide. Structure: (1) Introduction - what changed and why, (2) Summary table - old file → new location, (3) Import updates - before/after examples, (4) Finding code - how to locate moved functions, (5) Public API - assurance nothing broke, (6) FAQ - common questions. Include concrete examples for top 10 splits.

Restrictions: Focus on helping contributors. Clear, friendly tone. Practical examples. Keep under 400 lines.

Success: Migration guide created. Table shows all major moves. Examples are clear. Contributors can easily find relocated code. FAQ addresses common concerns.

After completing:
1. Mark [-], write guide, log creation, mark [x]_

## Phase 6: Fix Any Issues

- [ ] 6.1 Fix any test failures from 5.1
  - Files: Various as needed based on failures
  - Address test failures from verification
  - Ensure all tests pass
  - Purpose: Achieve 100% passing tests
  - _Leverage: Failure details from 5.1_
  - _Requirements: 6.1_
  - _Prompt: Implement the task for spec split-large-files. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Bug fix specialist

Task: Fix test failures identified in 5.1. For each failure: (1) Identify root cause (import issue, moved function, etc.), (2) Apply minimal fix, (3) Verify fix locally, (4) Re-run affected tests. Prioritize actual bugs over flaky tests.

Restrictions: Minimal fixes only. Don't refactor beyond what's needed. Keep changes focused on making tests pass.

Success: All test failures resolved. `cargo test --all` passes completely. Fixes are targeted and correct. No new issues introduced.

After completing:
1. Mark [-], fix issues, test, log fixes with artifacts, mark [x]_

- [ ] 6.2 Fix any clippy warnings from 5.3
  - Files: Various as needed based on warnings
  - Address clippy warnings from verification
  - Clean up unused imports, dead code, etc.
  - Purpose: Maintain code quality standards
  - _Leverage: Warning details from 5.3_
  - _Requirements: 6.1_
  - _Prompt: Implement the task for spec split-large-files. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Code quality specialist

Task: Fix clippy warnings. Common issues from splits: (1) Unused imports - remove them, (2) Dead code - verify actually unused then remove, (3) Needless re-exports - simplify, (4) Visibility issues - adjust pub(crate) vs pub. Run `cargo clippy --fix --allow-dirty --allow-staged` to auto-fix where possible, then manual fixes for rest.

Restrictions: Follow clippy suggestions unless they conflict with design. Keep changes minimal. Don't suppress warnings unnecessarily.

Success: All clippy warnings resolved. `cargo clippy --all-targets -- -D warnings` passes. Code is clean. No false positives suppressed.

After completing:
1. Mark [-], fix warnings, verify, log fixes, mark [x]_

- [ ] 6.3 Address any remaining file size violations from 5.4
  - Files: Any files still >500 lines identified in 5.4
  - Split additional files if needed
  - Ensure full compliance
  - Purpose: Achieve 100% compliance with file size guideline
  - _Leverage: Violation list from 5.4_
  - _Requirements: 1.1, 2.1_
  - _Prompt: Implement the task for spec split-large-files. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Refactoring specialist

Task: Split any remaining oversized files from 5.4. For each file: (1) Analyze structure, (2) Identify split strategy, (3) Create submodules, (4) Test, (5) Verify under 500 lines. Use same patterns as top 10 splits.

Restrictions: Only if violations exist. If all files compliant, mark task complete immediately. Follow established splitting patterns.

Success: All files under 500-line limit. No remaining violations. Full compliance achieved.

After completing:
1. Mark [-], split if needed or skip if compliant, log, mark [x]_

## Phase 7: Final Validation

- [ ] 7.1 Run full CI checks
  - Command: `just ci-check` or equivalent
  - Verify all checks pass: fmt, clippy, tests, docs
  - Ensure no regressions
  - Purpose: Final gate before completion
  - _Leverage: Project CI configuration_
  - _Requirements: All_
  - _Prompt: Implement the task for spec split-large-files. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: CI/CD specialist

Task: Run full CI validation. Execute `just ci-check` or run all steps: (1) cargo fmt --check, (2) cargo clippy --all-targets -- -D warnings, (3) cargo test --all, (4) cargo doc --no-deps. Verify all pass. Document results.

Restrictions: Validation only. If checks fail, refer to fix tasks. Document any issues for follow-up.

Success: All CI checks pass. Project is green. Ready for use. Any failures clearly documented.

After completing:
1. Mark [-], run CI, document results, log, mark [x]_

- [ ] 7.2 Measure and document overall impact
  - File: CODEBASE_EVALUATION.md (update)
  - Document file count changes, average file size improvement
  - Record compilation speed improvement
  - Purpose: Show value delivered
  - _Leverage: Results from all verification tasks_
  - _Requirements: All_
  - _Prompt: Implement the task for spec split-large-files. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Technical writer and analyst

Task: Update CODEBASE_EVALUATION.md with results section for file splitting. Document: (1) Files split: 10 large files → 30+ focused modules, (2) Line count: 10,000+ lines → ~4,000 lines across modules, (3) Average file size: before/after, (4) Compilation improvement: percentage faster, (5) Compliance: 0 violations (was 56), (6) Lessons learned. Use tables and numbers.

Restrictions: Be factual with actual measurements. Use data from verification tasks. Professional tone. Acknowledge any challenges encountered.

Success: Evaluation updated with comprehensive results. Improvements quantified. Value clear. Stakeholders can see impact. Serves as reference for future work.

After completing:
1. Mark [-], update docs, log, mark [x]_

## Summary

**Total Tasks:** 30 tasks across 7 phases

**Estimated Effort:** 1 week (top 10 files), 2 weeks (all 56 if needed)

**Expected Impact:**
- ✅ All files under 500-line limit (was 56 violations)
- ✅ 20-30% faster incremental builds
- ✅ Better code organization (clear domain boundaries)
- ✅ Easier code review (smaller, focused diffs)
- ✅ Reduced merge conflicts
- ✅ Improved maintainability
- ✅ Zero breaking changes (100% API compatible)

**Key Deliverables:**
- Top 10 largest files split into 30+ focused modules
- All files compliant with 500-line limit
- Comprehensive module documentation
- Migration guide for contributors
- Measured and documented performance improvements
- All tests passing, clippy clean, fully documented
