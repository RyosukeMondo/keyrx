# Validation Architecture

This document describes the architecture of KeyRx's validation system, which detects potential issues in key remapping configurations before they are applied.

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Core Components](#core-components)
- [Detector Pattern](#detector-pattern)
- [Built-in Detectors](#built-in-detectors)
- [Adding New Detectors](#adding-new-detectors)
- [Performance Considerations](#performance-considerations)
- [Testing](#testing)

## Overview

The validation system analyzes a list of pending operations (remaps, blocks, combos, etc.) before they are applied to the system. It uses a modular detector-based architecture where each detector focuses on a specific validation concern.

### Design Goals

1. **Modularity**: Each detector is independent and focuses on a single concern
2. **Extensibility**: New detectors can be added without modifying existing code
3. **Performance**: O(n) or O(n log n) complexity for most detectors
4. **Testability**: Each detector can be tested in isolation
5. **Comprehensive**: Detect conflicts, cycles, shadowing, and semantic issues

## Architecture

```
┌─────────────────────────────────────────────────┐
│         ValidationEngine                         │
│  (Main entry point)                              │
└────────────────┬────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────┐
│      DetectorOrchestrator                        │
│  (Coordinates detector execution)                │
└────────┬────────────────────────────────────────┘
         │
         ├──────────┬──────────┬──────────┐
         ▼          ▼          ▼          ▼
    ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐
    │Conflict │ │ Cycle   │ │Shadowing│ │ Future  │
    │Detector │ │Detector │ │Detector │ │Detector │
    └─────────┘ └─────────┘ └─────────┘ └─────────┘
         │          │          │          │
         └──────────┴──────────┴──────────┘
                     │
                     ▼
            ┌──────────────────┐
            │ ValidationReport │
            │  (Aggregated     │
            │   results)       │
            └──────────────────┘
```

### Data Flow

1. **Input**: List of `PendingOp` operations from script execution
2. **Context**: `DetectorContext` with configuration and metadata
3. **Processing**: Each detector analyzes operations independently
4. **Aggregation**: `DetectorOrchestrator` combines results into a `ValidationReport`
5. **Output**: Report with issues, statistics, and severity levels

## Core Components

### Detector Trait

The `Detector` trait defines the common interface all detectors must implement:

```rust
pub trait Detector: Send + Sync {
    /// Returns the unique name of this detector
    fn name(&self) -> &'static str;

    /// Runs detection on operations
    fn detect(&self, ops: &[PendingOp], ctx: &DetectorContext) -> DetectorResult;

    /// Indicates if this detector can be skipped for performance
    fn is_skippable(&self) -> bool {
        false
    }
}
```

**Key characteristics:**

- **Object-safe**: Can be used as `Box<dyn Detector>`
- **Thread-safe**: Implements `Send + Sync` for concurrent execution
- **Stateless**: No mutable state; all context passed via parameters

Location: `core/src/validation/detectors/mod.rs:42`

### DetectorContext

Provides context and configuration to detectors:

```rust
pub struct DetectorContext {
    pub script_path: Option<PathBuf>,
    pub config: ValidationConfig,
    pub skip_optional: bool,
}
```

Location: `core/src/validation/detectors/mod.rs:76`

### DetectorResult

Contains the results from a detector's analysis:

```rust
pub struct DetectorResult {
    pub issues: Vec<ValidationIssue>,
    pub stats: DetectorStats,
}
```

Location: `core/src/validation/detectors/mod.rs:114`

### ValidationIssue

Represents a single validation issue:

```rust
pub struct ValidationIssue {
    pub severity: Severity,           // Error, Warning, or Info
    pub detector: String,              // Which detector found it
    pub message: String,               // Human-readable description
    pub locations: Vec<SourceLocation>, // Where it occurred
    pub suggestion: Option<String>,    // How to fix it
}
```

**Severity levels:**

- `Error`: Must be fixed; configuration cannot be used
- `Warning`: Potential issue that may cause problems
- `Info`: Informational message, no action required

Location: `core/src/validation/detectors/mod.rs:194`

### DetectorOrchestrator

Coordinates execution of multiple detectors:

```rust
pub struct DetectorOrchestrator {
    detectors: Vec<Box<dyn Detector>>,
}

impl DetectorOrchestrator {
    pub fn register(&mut self, detector: Box<dyn Detector>);
    pub fn run(&self, ops: &[PendingOp], ctx: &DetectorContext) -> ValidationReport;
}
```

**Features:**

- Runs detectors in registration order
- Supports skipping optional detectors
- Aggregates results with timing statistics
- Thread-safe execution

Location: `core/src/validation/orchestrator.rs:99`

### ValidationReport

Aggregated results from all detectors:

```rust
pub struct ValidationReport {
    pub issues: Vec<ValidationIssue>,
    pub detector_stats: Vec<NamedDetectorStats>,
    pub total_operations: usize,
    pub total_duration_us: u64,
    pub skipped_detectors: usize,
}
```

**Utility methods:**

- `has_issues()`: Whether any issues were found
- `issue_count()`: Total number of issues
- `error_count()`, `warning_count()`, `info_count()`: Count by severity

Location: `core/src/validation/orchestrator.rs:16`

## Detector Pattern

### Basic Detector Structure

A typical detector follows this pattern:

```rust
pub struct MyDetector {
    // Detector configuration (if needed)
}

impl MyDetector {
    pub fn new() -> Self {
        Self {}
    }

    // Private helper methods for detection logic
    fn analyze_operations(&self, ops: &[PendingOp]) -> Vec<ValidationIssue> {
        // Detection logic here
        Vec::new()
    }
}

impl Detector for MyDetector {
    fn name(&self) -> &'static str {
        "my-detector"  // Lowercase, hyphen-separated
    }

    fn detect(&self, ops: &[PendingOp], ctx: &DetectorContext) -> DetectorResult {
        let start = Instant::now();

        // Perform analysis
        let issues = self.analyze_operations(ops);

        // Build statistics
        let stats = DetectorStats::new(
            ops.len(),
            issues.len(),
            start.elapsed()
        );

        DetectorResult::with_stats(issues, stats)
    }

    fn is_skippable(&self) -> bool {
        false  // Set to true if expensive and optional
    }
}
```

### Creating Issues

Use the builder API for creating issues:

```rust
// Error-level issue
let issue = ValidationIssue::error("my-detector", "Duplicate operation found")
    .with_location(SourceLocation::new(42))
    .with_suggestion("Remove one of the duplicate operations");

// Warning-level issue
let issue = ValidationIssue::warning("my-detector", "Potential shadowing detected")
    .with_locations(vec![
        SourceLocation::new(10),
        SourceLocation::new(20),
    ]);

// Info-level issue
let issue = ValidationIssue::info("my-detector", "Performance tip");
```

### Common Patterns

#### Building Key Indices

For O(n) detection, build a map of keys to operations:

```rust
let mut key_map: HashMap<KeyCode, Vec<OpInfo>> = HashMap::new();

for (index, op) in ops.iter().enumerate() {
    if let PendingOp::Remap { from, .. } = op {
        key_map.entry(*from).or_default().push(OpInfo { index });
    }
}

// Then check for conflicts on keys with multiple operations
for (key, op_list) in key_map {
    if op_list.len() > 1 {
        // Found a conflict
    }
}
```

#### Traversing Dependencies

For cycle detection or dependency analysis:

```rust
let mut visited = HashSet::new();
let mut rec_stack = HashSet::new();

fn visit(&mut self, key: KeyCode, visited: &mut HashSet<KeyCode>,
         rec_stack: &mut HashSet<KeyCode>) -> bool {
    visited.insert(key);
    rec_stack.insert(key);

    // Visit dependencies
    if let Some(deps) = self.dependencies.get(&key) {
        for dep in deps {
            if !visited.contains(dep) {
                if self.visit(*dep, visited, rec_stack) {
                    return true; // Cycle found
                }
            } else if rec_stack.contains(dep) {
                return true; // Back edge = cycle
            }
        }
    }

    rec_stack.remove(&key);
    false
}
```

## Built-in Detectors

### ConflictDetector

**Purpose**: Detects conflicting operations on the same key

**What it catches:**

- Duplicate remaps (key remapped multiple times)
- Remap + block conflicts
- Tap-hold conflicts with simple remaps
- Multiple pass operations on the same key

**Algorithm**: O(n) using HashMap to group operations by key

**Location**: `core/src/validation/detectors/conflicts.rs`

**Example issue:**

```
Error: Duplicate remap detected for key 'A'
  Location: line 10
  Location: line 15
  Suggestion: Remove one of the remap definitions
```

### ShadowingDetector

**Purpose**: Detects combo keys that shadow other operations

**What it catches:**

- Combo prefix keys that shadow single-key remaps
- Multi-key combos that shadow shorter combos
- Order-dependent shadowing issues

**Algorithm**: O(n*m) where m is max combo length (typically small)

**Skippable**: Yes (marked as optional for performance)

**Location**: `core/src/validation/detectors/shadowing.rs`

**Example issue:**

```
Warning: Combo 'Ctrl+A' shadows remap of 'Ctrl'
  Location: line 8 (combo definition)
  Location: line 3 (shadowed remap)
  Suggestion: Reorder operations or remove the shadowed remap
```

### CycleDetector

**Purpose**: Detects circular dependencies in remaps

**What it catches:**

- Direct cycles (A->B, B->A)
- Indirect cycles (A->B, B->C, C->A)
- Self-loops (A->A)

**Algorithm**: O(V+E) using DFS with coloring

**Location**: `core/src/validation/detectors/cycles.rs`

**Example issue:**

```
Error: Circular dependency detected
  Cycle path: A -> B -> C -> A
  Location: line 5 (A->B)
  Location: line 7 (B->C)
  Location: line 9 (C->A)
```

## Adding New Detectors

Follow these steps to add a new detector to the system:

### Step 1: Create Detector Module

Create a new file in `core/src/validation/detectors/`:

```rust
// core/src/validation/detectors/my_detector.rs

use super::{Detector, DetectorContext, DetectorResult, DetectorStats, ValidationIssue};
use crate::scripting::PendingOp;
use std::time::Instant;

pub struct MyDetector {
    // Configuration fields if needed
}

impl MyDetector {
    pub fn new() -> Self {
        Self {}
    }

    fn analyze(&self, ops: &[PendingOp]) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        // Your detection logic here

        issues
    }
}

impl Detector for MyDetector {
    fn name(&self) -> &'static str {
        "my-detector"
    }

    fn detect(&self, ops: &[PendingOp], ctx: &DetectorContext) -> DetectorResult {
        let start = Instant::now();
        let issues = self.analyze(ops);

        let stats = DetectorStats::new(
            ops.len(),
            issues.len(),
            start.elapsed()
        );

        DetectorResult::with_stats(issues, stats)
    }

    fn is_skippable(&self) -> bool {
        false  // Or true if this is an expensive, optional check
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_detector() {
        let detector = MyDetector::new();
        // Add tests
    }
}
```

### Step 2: Register in Module Tree

Update `core/src/validation/detectors/mod.rs`:

```rust
pub mod conflicts;
pub mod cycles;
pub mod shadowing;
pub mod my_detector;  // Add this line
```

### Step 3: Register with Orchestrator

Update `core/src/validation/engine.rs` to include your detector:

```rust
use super::detectors::my_detector::MyDetector;

// In the function that creates the orchestrator:
let mut orchestrator = DetectorOrchestrator::new();
orchestrator.register(Box::new(ConflictDetector::new()));
orchestrator.register(Box::new(CycleDetector::new()));
orchestrator.register(Box::new(ShadowingDetector::new()));
orchestrator.register(Box::new(MyDetector::new()));  // Add this line
```

### Step 4: Write Tests

Create comprehensive tests in `core/src/validation/tests/`:

```rust
// core/src/validation/tests/my_detector_tests.rs

#[cfg(test)]
mod my_detector_tests {
    use crate::validation::detectors::my_detector::MyDetector;
    use crate::validation::detectors::{Detector, DetectorContext};
    use crate::validation::config::ValidationConfig;
    use crate::scripting::PendingOp;

    #[test]
    fn test_detects_issue() {
        let detector = MyDetector::new();
        let ops = vec![
            // Create test operations
        ];

        let ctx = DetectorContext::new(ValidationConfig::default());
        let result = detector.detect(&ops, &ctx);

        assert_eq!(result.issues.len(), 1);
        assert_eq!(result.issues[0].detector, "my-detector");
    }

    #[test]
    fn test_no_false_positives() {
        let detector = MyDetector::new();
        let ops = vec![
            // Valid operations
        ];

        let ctx = DetectorContext::new(ValidationConfig::default());
        let result = detector.detect(&ops, &ctx);

        assert_eq!(result.issues.len(), 0);
    }
}
```

### Step 5: Update Documentation

Add your detector to this document's Built-in Detectors section with:

- Purpose
- What it catches
- Algorithm complexity
- Location
- Example issue

## Performance Considerations

### Complexity Goals

- **Conflict detection**: O(n) using hash maps
- **Cycle detection**: O(V+E) using DFS
- **Shadowing detection**: O(n*m) where m is max combo length

### Optimization Techniques

1. **Early termination**: Return as soon as critical issue found
2. **Indexing**: Build indices once, query multiple times
3. **Skippable detectors**: Mark expensive checks as optional
4. **Parallel execution**: Detectors are stateless and thread-safe (future enhancement)

### Memory Management

- Use `Vec::with_capacity()` when size is known
- Clear and reuse collections when possible
- Avoid unnecessary clones; use references

### Benchmarking

Run benchmarks to ensure performance:

```bash
cargo bench --bench validation_bench
```

Target: < 1ms for 1000 operations on typical hardware

## Testing

### Unit Tests

Each detector should have comprehensive unit tests covering:

- **Happy path**: Valid configurations with no issues
- **Error cases**: Configurations that should trigger issues
- **Edge cases**: Empty operations, single operation, etc.
- **Boundary conditions**: Large configurations, deeply nested combos

### Integration Tests

Test the full validation pipeline:

```rust
#[test]
fn test_full_validation_pipeline() {
    let script = r#"
        remap("A", "B");
        remap("B", "C");
        remap("C", "A");  // Creates cycle
    "#;

    let result = validate_script(script);

    assert!(result.has_errors());
    assert!(result.issues.iter().any(|i| i.detector == "cycle"));
}
```

### Test Helpers

Use shared test utilities from `core/src/validation/common/test_helpers.rs`:

```rust
use crate::validation::common::test_helpers::*;

#[test]
fn test_with_helpers() {
    let ops = vec![
        op_remap(KeyCode::A, KeyCode::B),
        op_block(KeyCode::C),
    ];

    assert_has_issue(&result, "conflict");
    assert_issue_count(&result, 1);
}
```

### Coverage

Maintain high test coverage:

```bash
cargo tarpaulin --out Html --output-dir coverage
```

Target: > 80% line coverage for detector code

## Best Practices

### Detector Design

1. **Single Responsibility**: Each detector focuses on one type of issue
2. **Deterministic**: Same input always produces same output
3. **Idempotent**: Running multiple times doesn't change results
4. **Independent**: Detectors don't depend on other detectors

### Error Messages

Make messages clear and actionable:

- **What**: Clearly state what the issue is
- **Where**: Include source locations
- **Why**: Explain why it's a problem
- **How**: Suggest how to fix it

**Good:**

```
Error: Circular dependency detected
  A remaps to B, which remaps back to A
  Location: line 5 (A->B)
  Location: line 7 (B->A)
  Suggestion: Remove one of the remaps to break the cycle
```

**Bad:**

```
Error: Invalid configuration
  Location: line 5
```

### Code Organization

- Keep detector files under 500 lines
- Extract complex logic into helper functions
- Use descriptive variable names
- Add doc comments for public APIs

### Backward Compatibility

When modifying detectors:

- Maintain existing issue formats for tooling
- Add new features via optional flags
- Deprecate old APIs before removing

## Future Enhancements

Potential improvements to the validation system:

1. **Parallel execution**: Run detectors concurrently using rayon
2. **Incremental validation**: Only re-validate changed operations
3. **Custom detectors**: Plugin system for user-defined detectors
4. **LSP integration**: Real-time validation in editors
5. **Fix suggestions**: Auto-fix capabilities for common issues
6. **Configuration profiles**: Different validation levels (strict, relaxed)
7. **Machine learning**: Learn from user corrections to improve detection

## References

- Detector trait definition: `core/src/validation/detectors/mod.rs`
- Orchestrator implementation: `core/src/validation/orchestrator.rs`
- Validation engine: `core/src/validation/engine.rs`
- Test suite: `core/src/validation/tests/`

## Contributing

When contributing new detectors or improvements:

1. Open an issue to discuss the detection strategy
2. Follow the detector pattern outlined in this document
3. Write comprehensive tests
4. Update this documentation
5. Submit a pull request with examples

For questions or suggestions, please open an issue on the KeyRx repository.
