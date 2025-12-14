# Requirements Document

## Introduction

The KeyRx codebase has **56 files exceeding the 500-line limit** mandated by user guidelines (max 500 lines/file excluding comments/blank lines). This spec addresses the #2 priority improvement: systematically splitting large files into smaller, focused modules to improve maintainability, compilation speed, and code organization.

**Problem Statement:**
- 56 files violate 500-line limit (worst: 1,893 lines)
- Large files slow incremental compilation
- Difficult to review and understand (cognitive overload)
- Higher merge conflict probability
- Violates Single Responsibility Principle
- Top 10 files contain 10,000+ lines that should be ~4,000 lines

**Value Proposition:**
- **20-30% faster incremental builds** (only changed modules recompile)
- **Easier code review** (smaller, focused diffs)
- **Better code organization** (clear domain boundaries)
- **Reduced merge conflicts** (changes isolated to smaller files)
- **Improved maintainability** (easier to find and understand code)
- **Compliance with code quality standards** (max 500 lines/file)

## Alignment with Product Vision

This feature supports the core development principles outlined in `~/.claude/CLAUDE.md`:

- **Code Metrics (KPI)**: Enforces max 500 lines/file guideline
- **Architecture**: SOLID principles - Single Responsibility Principle per file
- **Code Quality**: Modular design with clear boundaries
- **Maintainability**: Smaller files are easier to understand and modify

## Requirements

### Requirement 1: File Size Audit and Prioritization

**User Story:** As a developer, I want a comprehensive audit of all oversized files, so that I can prioritize which files to split first based on impact.

#### Acceptance Criteria

1. WHEN file size audit is run THEN it SHALL identify all Rust source files exceeding 500 lines
2. WHEN files are identified THEN they SHALL be ranked by size (largest first)
3. WHEN ranking is complete THEN top 10 files SHALL be prioritized for immediate action
4. IF a file is identified for splitting THEN its logical structure SHALL be analyzed to find split points
5. WHEN analysis is complete THEN a splitting plan SHALL be documented for each file

**Success Metrics:**
- Complete list of 56+ files exceeding 500-line limit
- Files ranked by size with line counts
- Top 10 files identified for immediate action (1,893 → 505 lines)
- Logical split points identified for each priority file
- Splitting plans documented with proposed module structure

### Requirement 2: Split Large Files into Logical Modules

**User Story:** As a developer, I want oversized files split into focused modules along natural boundaries, so that each module has a single, clear responsibility.

#### Acceptance Criteria

1. WHEN a file is split THEN each resulting module SHALL be under 500 lines (ideally under 400 lines)
2. WHEN a file is split THEN split SHALL follow logical domain boundaries (not arbitrary line counts)
3. WHEN modules are created THEN each SHALL have a single, well-defined purpose
4. IF related functionality exists THEN it SHALL be grouped together in the same module
5. WHEN split is complete THEN public API SHALL remain identical (re-exported from parent module)
6. IF types are shared THEN they SHALL be moved to appropriate shared module or parent

**Logical Splitting Strategies:**
- **By domain:** Group related functions (e.g., keyboard bindings, layer management, modifiers)
- **By type:** Separate data types, implementations, and utilities
- **By responsibility:** Each module handles one aspect of functionality
- **By abstraction level:** Separate high-level and low-level concerns

**Success Metrics:**
- All files under 500-line limit after splitting
- Each module has clear, single responsibility
- Public APIs unchanged (consumers unaffected)
- Logical grouping improves code discoverability
- Module names clearly describe their purpose

### Requirement 3: Maintain API Compatibility

**User Story:** As a code consumer, I want the public API to remain unchanged after file splitting, so that my code continues working without modifications.

#### Acceptance Criteria

1. WHEN a file is split THEN all public items SHALL be re-exported from the parent module
2. WHEN imports are used THEN existing `use` statements SHALL continue working unchanged
3. IF a public function is moved THEN it SHALL be re-exported with identical signature
4. WHEN a public type is moved THEN it SHALL be re-exported with identical definition
5. IF visibility is changed THEN it SHALL only be for internal reorganization, not public API

**Re-export Pattern:**
```rust
// Before split: core/src/example.rs
pub fn function_a() { }
pub fn function_b() { }

// After split:
// core/src/example/mod.rs
pub mod submodule_a;
pub mod submodule_b;
pub use submodule_a::function_a;
pub use submodule_b::function_b;

// Users still write: use keyrx_core::example::function_a;
```

**Success Metrics:**
- Zero breaking changes to public API
- All existing imports continue working
- Cargo build succeeds without changes to consuming code
- Tests pass without modification
- Documentation links remain valid

### Requirement 4: Update Internal Imports

**User Story:** As a developer maintaining the codebase, I want internal imports updated to reflect new module structure, so that the code is clean and uses the new organization correctly.

#### Acceptance Criteria

1. WHEN a file is split THEN internal imports SHALL be updated to use new module paths
2. WHEN a module imports from sibling THEN it SHALL use relative or absolute path as appropriate
3. IF circular dependencies exist THEN they SHALL be resolved through proper module organization
4. WHEN imports are updated THEN unused imports SHALL be removed
5. IF a module uses many items from another THEN glob imports MAY be used judiciously

**Import Organization:**
```rust
// Good: Clear, organized imports
use crate::config::models::{DeviceInstanceId, HardwareProfile, Keymap};
use super::types::StateSnapshot;
use super::layers::LayerState;

// Avoid: Glob imports unless truly many items needed
use super::keyboard::*; // Only if >10 items used
```

**Success Metrics:**
- All imports reference correct module paths
- No unused imports remain (clippy clean)
- No circular dependency errors
- Import groups are logical (std, external crates, internal)
- Code compiles without warnings

### Requirement 5: Document Module Organization

**User Story:** As a new contributor, I want module structure documented, so that I can understand the organization and find code quickly.

#### Acceptance Criteria

1. WHEN a file is split THEN the parent module SHALL have doc comments explaining structure
2. WHEN a submodule is created THEN it SHALL have doc comments explaining its purpose
3. IF module structure is complex THEN a diagram or explanation SHALL be provided
4. WHEN documentation is written THEN it SHALL explain what each module contains
5. IF modules have dependencies THEN relationships SHALL be documented

**Module Documentation Pattern:**
```rust
//! Scripting engine bindings.
//!
//! This module provides Rhai script bindings for KeyRx functionality.
//! Split into focused submodules:
//!
//! - [`keyboard`] - Keyboard event and key code bindings
//! - [`layers`] - Layer management and state bindings
//! - [`modifiers`] - Modifier key state bindings
//! - [`utilities`] - Utility functions for scripting
```

**Success Metrics:**
- All parent modules have doc comments explaining structure
- All submodules have doc comments explaining purpose
- Module relationships are clear
- New contributors can navigate code easily
- Documentation is up-to-date after splits

### Requirement 6: Test All Splits

**User Story:** As a quality assurance engineer, I want comprehensive testing after file splits, so that I can verify functionality remains correct.

#### Acceptance Criteria

1. WHEN a file is split THEN all existing tests SHALL pass without modification
2. WHEN tests are run THEN compilation SHALL succeed with zero errors
3. IF tests fail THEN root cause SHALL be identified and fixed before proceeding
4. WHEN clippy is run THEN it SHALL pass with zero warnings
5. WHEN formatter is run THEN code SHALL be properly formatted
6. IF benchmarks exist THEN performance SHALL be verified as unchanged

**Testing Checklist Per Split:**
- [ ] `cargo build` succeeds
- [ ] `cargo test` passes (all tests)
- [ ] `cargo clippy` passes (no warnings)
- [ ] `cargo fmt --check` passes
- [ ] No new compiler warnings
- [ ] Documentation builds (`cargo doc`)

**Success Metrics:**
- All tests pass after each split
- Zero compilation errors or warnings
- No performance regressions
- Clippy clean
- Formatting correct
- Documentation builds successfully

### Requirement 7: Incremental Implementation

**User Story:** As a project maintainer, I want file splitting done incrementally, so that the codebase remains stable and changes are reviewable.

#### Acceptance Criteria

1. WHEN splitting begins THEN top 10 largest files SHALL be split first
2. WHEN a file is split THEN changes SHALL be committed separately per file
3. IF issues arise THEN previous splits SHALL not block progress on other files
4. WHEN a split is complete THEN it SHALL be verified before moving to next file
5. IF conflicts occur THEN split SHALL be rebased/merged before proceeding

**Split Order (by priority/size):**
1. `scripting/bindings.rs` (1,893 lines) → 4 modules
2. `engine/state/mod.rs` (1,570 lines) → 3 modules
3. `engine/transitions/log.rs` (1,403 lines) → 3 modules
4. `bin/keyrx.rs` (1,382 lines) → 3 modules
5. `scripting/docs/generators/html.rs` (1,069 lines) → 2 modules
6. `validation/engine.rs` (968 lines) → 2 modules
7. `config/loader.rs` (949 lines) → 2 modules
8. `registry/profile.rs` (918 lines) → 2 modules
9. `engine/advanced.rs` (906 lines) → 2 modules
10. `cli/commands/run.rs` (899 lines) → 2 modules

**Success Metrics:**
- Top 10 files split and verified
- Each split committed separately
- No blocking issues between splits
- All intermediate states are stable
- Changes are reviewable in isolation

## Non-Functional Requirements

### Code Architecture and Modularity

- **Single Responsibility Principle**: Each module file has one well-defined purpose
- **Module Cohesion**: Related functionality grouped together
- **Module Coupling**: Minimal dependencies between modules
- **Clear Boundaries**: Domain boundaries aligned with module boundaries
- **Logical Organization**: File structure mirrors conceptual structure

**Target Module Organization:**
```
example/
├── mod.rs           (100-200 lines) - Public API + re-exports
├── domain_a.rs      (200-400 lines) - Domain A functionality
├── domain_b.rs      (200-400 lines) - Domain B functionality
└── utilities.rs     (100-300 lines) - Shared utilities
```

### Performance

- **Compilation Speed**: Incremental builds 20-30% faster
- **Incremental Recompilation**: Only changed modules recompile
- **Binary Size**: No increase in final binary size
- **Runtime Performance**: Zero impact on runtime performance
- **Module Loading**: No additional overhead

**Benchmark Targets:**
```
Before: Touch large file → 5-10s recompile (entire file)
After:  Touch submodule → 3-5s recompile (only that module)
Improvement: ~30-50% faster incremental builds
```

### Maintainability

- **Discoverability**: Easy to find relevant code
- **Readability**: Smaller files easier to read and understand
- **Review**: Smaller diffs easier to review
- **Conflicts**: Reduced probability of merge conflicts
- **Navigation**: IDE navigation faster with smaller files
- **Cognitive Load**: Reduced complexity per file

### Compatibility

- **API Stability**: Zero breaking changes
- **Import Compatibility**: Existing imports continue working
- **Test Compatibility**: Tests pass without modification
- **Documentation Compatibility**: Doc links remain valid
- **Binary Compatibility**: FFI and ABI unchanged

### Code Quality

- **File Size**: All files ≤ 500 lines (target ≤ 400 lines)
- **Function Size**: All functions ≤ 50 lines (maintained)
- **Formatting**: Consistent with `cargo fmt`
- **Linting**: Passes `cargo clippy` with no warnings
- **Documentation**: All public items documented

### Reliability

- **Zero Regressions**: All tests pass after splits
- **Deterministic**: Splits are repeatable and verifiable
- **Reversible**: Changes can be backed out if issues arise
- **Safe**: No undefined behavior introduced
- **Stable**: Intermediate states all compile and test

### Usability (Developer Experience)

- **Clear Structure**: Module organization is intuitive
- **Easy Navigation**: Developers can find code quickly
- **Self-Documenting**: Module names describe contents
- **Minimal Friction**: Splits don't disrupt workflow
- **Good Examples**: Documentation shows how to use new structure
