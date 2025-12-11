# Tasks Document

## Phase 1: Analysis and Measurement

- [x] 1.1 Audit function lengths across codebase
  - Create script to analyze function line counts (using syn crate or grep)
  - Identify all functions exceeding 50-line limit
  - Rank by severity (100+ lines vs 50-70 lines)
  - Purpose: Understand scope of function length violations
  - _Leverage: Rust AST parsing or grep analysis_
  - _Requirements: 1.1_
  - _Prompt: Implement the task for spec misc-improvements. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Code metrics analyst with Rust expertise

Task: Analyze function lengths following requirement 1.1. Create analysis approach: (1) Use ripgrep to find function definitions `rg "fn\s+\w+" --type rust`, (2) For each function, count lines excluding comments/blanks, (3) Report functions >50 lines with location and size. Create .spec-workflow/specs/misc-improvements/function_length_audit.md with ranked list. Alternatively, create Rust script using `syn` crate for accurate parsing.

Restrictions: Analysis only, no code changes. Exclude test functions (#[test], #[cfg(test)]). Count logical lines, not file lines. Identify top 20 worst violations.

Success: Complete audit with all long functions identified. Ranked by size. Locations documented. Ready for refactoring prioritization.

After completing:
1. Mark [-], run analysis, document results, use log-implementation, mark [x]_

- [x] 1.2 Measure test coverage and identify gaps
  - Prerequisites: Spec #3 (fix-failing-tests) must be complete
  - Command: `cargo llvm-cov --lib --html`
  - Analyze coverage report and identify modules below targets
  - Purpose: Quantify coverage gaps for improvement
  - _Leverage: cargo llvm-cov tool_
  - _Requirements: 2.1, 2.2, 2.3_
  - _Prompt: Implement the task for spec misc-improvements. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Test coverage analyst

Task: Measure coverage following requirements 2.1-2.3. Run `cargo llvm-cov --lib --summary-only` for overall stats. Run `cargo llvm-cov --lib --html` for detailed report. Analyze: (1) Overall coverage vs 80% target, (2) Critical paths (services/, api.rs, engine/, ffi/) vs 90% target, (3) Specific uncovered lines/branches. Document in .spec-workflow/specs/misc-improvements/coverage_analysis.md with: current coverage %, gaps, priority recommendations.

Restrictions: Measurement only. Requires tests passing (spec #3 complete). Identify gaps but don't write tests yet.

Success: Coverage comprehensively measured. Overall and per-module percentages known. Gaps identified with line numbers. Priorities documented. HTML report generated.

After completing:
1. Mark [-], measure, analyze, document, log findings, mark [x]_

- [x] 1.3 Verify structured logging compliance
  - Files: core/src/observability/entry.rs, core/src/observability/logger.rs
  - Review LogEntry struct for required fields
  - Verify JSON output format
  - Check for PII/secrets in logging
  - Purpose: Ensure logging meets standards
  - _Leverage: Existing logging implementation_
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7_
  - _Prompt: Implement the task for spec misc-improvements. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Observability and logging specialist

Task: Verify logging compliance following requirements 3.1-3.7. Review: (1) LogEntry struct in observability/entry.rs - check fields (timestamp, level, target/service, message/event, fields/context), (2) JSON serialization in logger.rs, (3) Sample log output from tests, (4) Search for potential PII/secrets being logged (`rg "password|secret|token|key" core/src --type rust`). Document findings in .spec-workflow/specs/misc-improvements/logging_compliance.md with: current state, required fields present/missing, format compliance, recommendations.

Restrictions: Verification only. Document gaps but don't fix yet. Check if mapping needed (target→service, message→event).

Success: Logging compliance assessed. Required fields verified. JSON format validated. PII/secrets check done. Gaps documented with fix recommendations.

After completing:
1. Mark [-], verify, document, log findings, mark [x]_

- [ ] 1.4 Audit documentation coverage
  - Command: `cargo doc --no-deps 2>&1 | grep -i warning`
  - Identify undocumented public APIs
  - Check for missing parameter/return documentation
  - Purpose: Understand documentation gaps
  - _Leverage: cargo doc warnings_
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5_
  - _Prompt: Implement the task for spec misc-improvements. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Technical documentation auditor

Task: Audit docs following requirements 4.1-4.5. Run `cargo doc --no-deps` and capture warnings. Check for: (1) Missing docs on public functions, (2) Missing docs on public types/traits, (3) Unclear parameter docs, (4) Missing examples on complex APIs. Create .spec-workflow/specs/misc-improvements/documentation_audit.md listing: undocumented items, priority (critical/nice-to-have), recommended additions.

Restrictions: Audit only. Focus on public API (pub items). Don't write documentation yet.

Success: Doc coverage comprehensively audited. All missing docs identified. Priorities assigned. Ready for documentation work.

After completing:
1. Mark [-], audit, document, log, mark [x]_

- [ ] 1.5 Calculate code complexity (optional)
  - Identify functions with high cyclomatic complexity
  - Find functions >10 complexity needing simplification
  - Purpose: Identify overly complex code
  - _Leverage: Manual analysis or cargo-geiger_
  - _Requirements: 5.1, 5.2, 5.3, 5.4_
  - _Prompt: Implement the task for spec misc-improvements. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Code complexity analyst

Task: Analyze complexity following requirements 5.1-5.4. Approach: (1) Manual - review functions from 1.1 audit, count branches (if/match/loop), estimate complexity, OR (2) Tool - use cargo-complexity or similar if available. Identify functions >10 complexity. Document in .spec-workflow/specs/misc-improvements/complexity_analysis.md with: function name, estimated complexity, simplification suggestions.

Restrictions: Analysis only. This is optional - skip if time-constrained. Focus on functions already identified as long (from 1.1).

Success: Complexity analysis complete OR marked as optional/skipped. High complexity functions identified if analyzed. Simplification strategies noted.

After completing:
1. Mark [-], analyze (or skip if optional), document, log, mark [x]_

## Phase 2: Prioritization

- [ ] 2.1 Create prioritized improvement plan
  - Input: Results from Phase 1 (1.1-1.5)
  - Rank all improvements by impact and effort
  - Create actionable plan with top 20 items
  - Purpose: Focus on highest-value improvements
  - _Leverage: All Phase 1 analysis results_
  - _Requirements: All_
  - _Prompt: Implement the task for spec misc-improvements. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Engineering lead and prioritization specialist

Task: Create improvement plan. Review all Phase 1 results. Prioritize: (1) High - Critical path coverage <90%, functions >100 lines, missing critical API docs, logging violations, (2) Medium - Overall coverage <80%, functions 70-100 lines, utility docs, complexity >15, (3) Low - Functions 50-70 lines, nice-to-have docs, moderate complexity. Create .spec-workflow/specs/misc-improvements/improvement_plan.md with ranked top 20 items, estimated effort, expected impact.

Restrictions: Planning only, no implementation. Be realistic about effort. Consider dependencies (e.g., DI spec helps coverage).

Success: Prioritized plan created. Top 20 improvements identified. Effort estimated. Impact quantified. Plan is actionable and realistic.

After completing:
1. Mark [-], prioritize, create plan, log, mark [x]_

## Phase 3: Function Length Fixes (Top Violations)

- [ ] 3.1 Refactor top 5 longest functions
  - Files: From 1.1 audit results
  - Break down functions >100 lines into logical helpers
  - Maintain exact behavior (zero logic changes)
  - Purpose: Address worst function length violations
  - _Leverage: Refactoring patterns from design doc_
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_
  - _Prompt: Implement the task for spec misc-improvements. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Rust refactoring specialist

Task: Refactor top 5 longest functions following requirements 1.1-1.5. For each function: (1) Read and understand logic, (2) Identify logical steps/sections, (3) Extract steps into helper functions with descriptive names, (4) Keep public API unchanged, (5) Verify tests still pass, (6) Ensure each function (main + helpers) <50 lines. Refactor one at a time, test between each.

Restrictions: Zero behavior changes. Preserve all error handling. Maintain exact functionality. Extract helpers as private functions (unless reusable). Don't over-extract (keep logical coherence).

Success: Top 5 functions refactored. All under 50 lines. Helpers are clear and logical. Tests pass. No regressions. Code is more readable.

After completing:
1. Mark [-], refactor carefully, test after each, use log-implementation with detailed artifacts, mark [x]_

- [ ] 3.2 Refactor functions 6-15 if time permits
  - Files: Next 10 from audit
  - Continue refactoring pattern from 3.1
  - Purpose: Further reduce function length violations
  - _Leverage: 3.1 refactoring experience_
  - _Requirements: 1.1-1.5_
  - _Prompt: Implement the task for spec misc-improvements. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Rust refactoring specialist

Task: Continue refactoring following same pattern as 3.1. Refactor functions 6-15 from audit. Same process: understand, identify steps, extract helpers, test. Work through systematically.

Restrictions: Same as 3.1. Zero behavior changes. Test after each. Stop if time-constrained (lower priority than other phases).

Success: Functions 6-15 refactored OR marked as deferred if time-constrained. All refactored functions <50 lines. Tests pass.

After completing:
1. Mark [-], refactor, test, log, mark [x] or mark as deferred_

## Phase 4: Test Coverage Improvements

- [ ] 4.1 Add tests for critical path gaps (<90%)
  - Files: Test files for services/, api.rs, engine/, ffi/
  - Write targeted tests to cover uncovered lines in critical paths
  - Focus on error paths and edge cases
  - Purpose: Achieve 90% coverage on critical paths
  - _Leverage: Coverage report from 1.2, mocks from DI spec_
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_
  - _Prompt: Implement the task for spec misc-improvements. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: QA engineer with Rust testing expertise

Task: Add tests for critical paths following requirements 2.1-2.5. Review coverage report from 1.2. For each critical module <90%: (1) Identify uncovered lines (red/yellow in HTML report), (2) Understand what scenarios hit those lines, (3) Write targeted tests, (4) Use mocks from DI spec for unit tests, (5) Re-run coverage to verify improvement. Focus on services/, api.rs, engine/, ffi/. Write quality tests that validate behavior, not just hit lines.

Restrictions: Write meaningful tests. Use mocks for unit tests (fast, isolated). Test error paths and edge cases. Each test should validate actual behavior. Target 90% for critical paths.

Success: Critical paths reach ≥90% coverage. New tests are meaningful and valuable. Tests pass reliably. Coverage measured and verified.

After completing:
1. Mark [-], write tests, measure coverage, use log-implementation with test details, mark [x]_

- [ ] 4.2 Add tests for overall coverage gaps (<80%)
  - Files: Test files across codebase
  - Write tests to bring overall coverage to 80%
  - Purpose: Meet minimum coverage standard
  - _Leverage: Coverage report from 1.2_
  - _Requirements: 2.1-2.5_
  - _Prompt: Implement the task for spec misc-improvements. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: QA engineer with comprehensive testing expertise

Task: Add tests for overall gaps following requirements 2.1-2.5. Review modules <80% coverage. For each: (1) Identify uncovered code, (2) Write tests to cover, (3) Focus on important paths first (skip trivial getters/setters if needed). Prioritize modules with significant business logic. Write quality tests.

Restrictions: Meaningful tests, not coverage-boosting. Skip truly trivial code if already at 75%+. Target is 80% overall, not 100%. Be pragmatic.

Success: Overall coverage ≥80%. New tests add value. Tests pass. Coverage verified.

After completing:
1. Mark [-], write tests, measure, log, mark [x]_

## Phase 5: Logging and Documentation

- [ ] 5.1 Fix structured logging gaps if any
  - Files: core/src/observability/entry.rs, core/src/observability/logger.rs
  - Add missing required fields to LogEntry
  - Fix JSON serialization if needed
  - Purpose: Ensure full logging compliance
  - _Leverage: Analysis from 1.3_
  - _Requirements: 3.1-3.7_
  - _Prompt: Implement the task for spec misc-improvements. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Logging and observability engineer

Task: Fix logging following requirements 3.1-3.7 and analysis from 1.3. Apply fixes: (1) Add missing fields to LogEntry struct if needed, (2) Update JSON serialization to include all required fields, (3) Add field mapping if needed (target→service, message→event), (4) Ensure timestamp is ISO 8601 format, (5) Verify no PII/secrets logged. If already compliant, mark complete quickly.

Restrictions: Minimal changes. Ensure backward compatibility. Test logging output. Verify JSON format.

Success: Logging fully compliant. All required fields present. JSON format correct. No PII logged. Tests pass.

After completing:
1. Mark [-], fix if needed or verify compliance, test, log, mark [x]_

- [ ] 5.2 Add documentation to critical public APIs
  - Files: Undocumented items from 1.4 audit (high priority)
  - Add comprehensive doc comments
  - Include examples for complex APIs
  - Purpose: Document critical APIs
  - _Leverage: Audit from 1.4_
  - _Requirements: 4.1-4.5_
  - _Prompt: Implement the task for spec misc-improvements. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Technical documentation writer with Rust expertise

Task: Document critical APIs following requirements 4.1-4.5. Review high-priority items from 1.4 audit. For each: (1) Write clear /// doc comment explaining purpose, (2) Document parameters with descriptions, (3) Document return values and errors, (4) Add examples for complex APIs (especially services, API layer), (5) Keep concise but complete. Use documentation template from design doc.

Restrictions: Focus on high-priority (critical APIs). Don't over-document trivial items. Be clear and concise. Examples should compile.

Success: Critical APIs fully documented. Doc comments are clear and helpful. Examples provided where needed. `cargo doc` builds without warnings for these items.

After completing:
1. Mark [-], write docs, verify cargo doc, log documentation added, mark [x]_

- [ ] 5.3 Add documentation to remaining public APIs
  - Files: Remaining undocumented items from 1.4 (medium/low priority)
  - Complete documentation coverage
  - Purpose: 100% public API documentation
  - _Leverage: Audit from 1.4_
  - _Requirements: 4.1-4.5_
  - _Prompt: Implement the task for spec misc-improvements. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Technical documentation writer

Task: Complete documentation following requirements 4.1-4.5. Document remaining undocumented public APIs from 1.4 audit. Follow same pattern as 5.2. Can be more concise for utility functions. Focus on clarity.

Restrictions: Complete coverage but be pragmatic. Trivial getters can have brief docs. Focus on user value.

Success: 100% public API documented OR 95%+ with only truly trivial items remaining. `cargo doc` builds without warnings. Documentation is helpful.

After completing:
1. Mark [-], complete docs, verify, log, mark [x]_

## Phase 6: Verification and Enforcement

- [ ] 6.1 Run full test suite and verify all passing
  - Command: `cargo test --all`
  - Ensure all improvements didn't break anything
  - Purpose: Verify no regressions
  - _Leverage: Full test suite_
  - _Requirements: All_
  - _Prompt: Implement the task for spec misc-improvements. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: QA validation specialist

Task: Verify no regressions. Run `cargo test --all --verbose`. Verify all tests pass. Check test count matches expected. Document results.

Restrictions: Verification only. All tests must pass.

Success: All tests pass. No regressions from improvements. Test suite stable.

After completing:
1. Mark [-], run tests, verify, log results, mark [x]_

- [ ] 6.2 Measure final coverage and verify targets met
  - Command: `cargo llvm-cov --lib --summary-only`
  - Verify ≥80% overall, ≥90% critical paths
  - Purpose: Confirm coverage targets achieved
  - _Leverage: cargo llvm-cov_
  - _Requirements: 2.1, 2.2_
  - _Prompt: Implement the task for spec misc-improvements. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Coverage verification specialist

Task: Verify coverage targets. Run `cargo llvm-cov --lib --summary-only` and `cargo llvm-cov --lib --html`. Check: (1) Overall ≥80%, (2) Critical paths (services, api, engine, ffi) ≥90%. Document final numbers. If targets not met, document remaining gaps.

Restrictions: Measurement only. Targets should be met or very close.

Success: Coverage targets met OR remaining gaps are documented. Final coverage percentages recorded. HTML report generated.

After completing:
1. Mark [-], measure, verify, document, log final numbers, mark [x]_

- [ ] 6.3 Verify all quality standards met
  - Verify file sizes (from spec #2)
  - Verify function lengths (from 3.1-3.2)
  - Verify coverage (from 6.2)
  - Verify documentation (cargo doc)
  - Purpose: Comprehensive quality check
  - _Leverage: All previous work_
  - _Requirements: All_
  - _Prompt: Implement the task for spec misc-improvements. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Quality assurance lead

Task: Comprehensive quality verification. Check: (1) All files <500 lines (spec #2), (2) All functions <50 lines (this spec), (3) Coverage ≥80%/90%, (4) `cargo doc` builds clean, (5) `cargo clippy` passes, (6) Logging compliant. Document compliance status for each metric. Create summary report.

Restrictions: Verification only. Create compliance matrix.

Success: All quality standards verified. Compliance documented. Summary report shows full compliance OR documents remaining minor gaps.

After completing:
1. Mark [-], verify all metrics, create summary, log compliance report, mark [x]_

- [ ] 6.4 Add CI enforcement for quality standards
  - File: .github/workflows/ or similar CI config
  - Add coverage check (fail if <80%)
  - Add doc check (fail on warnings)
  - Purpose: Prevent quality regressions
  - _Leverage: CI infrastructure_
  - _Requirements: 2.6_
  - _Prompt: Implement the task for spec misc-improvements. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: CI/CD engineer

Task: Add quality gates to CI. Update CI configuration to: (1) Run `cargo llvm-cov --lib --summary-only` and parse output, fail if <80%, (2) Run `cargo doc --no-deps`, fail on warnings, (3) Optionally add function length check script. Integrate into existing `just ci-check` or CI workflow.

Restrictions: Don't break existing CI. Add checks incrementally. Make sure checks are reliable. Consider making strict enforcement a warning first, then error after confirmed working.

Success: CI enforces coverage ≥80%. CI enforces doc warnings. Quality gates prevent regressions. CI configuration updated and tested.

After completing:
1. Mark [-], update CI, test CI locally, log changes, mark [x]_

## Phase 7: Documentation and Wrap-up

- [ ] 7.1 Update CODEBASE_EVALUATION.md with results
  - File: CODEBASE_EVALUATION.md
  - Add section documenting misc improvements results
  - Record before/after metrics
  - Purpose: Document completion
  - _Leverage: All verification results_
  - _Requirements: All_
  - _Prompt: Implement the task for spec misc-improvements. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Technical writer

Task: Update evaluation with results. Add "# Implementation Results - Misc Improvements" section. Document: (1) Function lengths - functions refactored, compliance achieved, (2) Coverage - before/after percentages, gaps filled, (3) Logging - compliance verified, fixes applied, (4) Documentation - items documented, coverage achieved, (5) Overall quality metrics - all standards met. Use before/after comparison tables.

Restrictions: Be factual. Use actual measurements. Document any remaining minor gaps honestly.

Success: Evaluation comprehensively updated. All improvements documented. Metrics shown. Value demonstrated.

After completing:
1. Mark [-], update docs, log update, mark [x]_

- [ ] 7.2 Create quality standards reference guide
  - File: .spec-workflow/specs/misc-improvements/QUALITY_STANDARDS.md
  - Document all enforced standards
  - Provide examples and guidelines
  - Purpose: Help developers maintain quality
  - _Leverage: All spec work_
  - _Requirements: All_
  - _Prompt: Implement the task for spec misc-improvements. First run spec-workflow-guide to get the workflow guide, then implement the task:

Role: Technical documentation specialist

Task: Create quality standards guide. Structure: (1) Overview - project quality standards, (2) Code Metrics - max 500 lines/file, max 50 lines/function, (3) Test Coverage - 80% overall, 90% critical, (4) Logging - JSON format requirements, (5) Documentation - public API requirements, (6) CI Enforcement - what's checked and how, (7) Examples - good vs bad for each standard. Keep under 300 lines. Be practical and helpful.

Restrictions: Make it developer-friendly, not bureaucratic. Focus on "why" and "how", not just "what". Include examples.

Success: Quality standards guide created. Comprehensive but practical. Helps developers write quality code. Serves as reference for reviews.

After completing:
1. Mark [-], write guide, log creation, mark [x]_

## Summary

**Total Tasks:** 22 tasks across 7 phases

**Estimated Effort:** 3-5 days (depending on scope of gaps)

**Expected Impact:**
- ✅ All functions ≤50 lines (was unknown)
- ✅ Coverage ≥80% overall, ≥90% critical (will be verified)
- ✅ Structured logging fully compliant
- ✅ 100% public API documented (or 95%+)
- ✅ Quality standards enforced in CI
- ✅ Maintainable, quality codebase

**Key Deliverables:**
- Function length compliance achieved (all functions refactored)
- Test coverage meets targets (80%/90%)
- Structured logging verified and fixed if needed
- Comprehensive API documentation
- Quality standards enforced automatically in CI
- Reference guide for maintaining quality
