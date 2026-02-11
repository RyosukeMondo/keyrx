# KISS/SLAP Quality Checklist

**Quick reference for developers** - Check before committing code

---

## File Size Limits

### ✅ PASS Criteria
- [ ] File has ≤500 code lines (excluding comments/blanks)
- [ ] No single array/object literal >200 lines
- [ ] Data files split into logical categories

### ❌ FAIL Indicators
- File >500 lines with mixed concerns
- Massive data structures in single file
- "God file" containing multiple unrelated features

### Quick Fix
```bash
# Check file size (code only)
grep -cv "^[[:space:]]*$\|^[[:space:]]*//" your_file.rs
# Should be ≤500
```

---

## Function Size Limits

### ✅ PASS Criteria
- [ ] Function has ≤50 lines
- [ ] Single responsibility (does ONE thing)
- [ ] Can be understood in <2 minutes

### ❌ FAIL Indicators
- Function >50 lines doing multiple tasks
- Scrolling required to see entire function
- Multiple levels of nested control flow

### Quick Fix
```rust
// Extract helper functions
fn process_event(event: Event) {
    let validated = validate_event(event)?;
    let transformed = transform_event(validated)?;
    store_event(transformed)
}

fn validate_event(event: Event) -> Result<Event> { /* ... */ }
fn transform_event(event: Event) -> Result<Event> { /* ... */ }
fn store_event(event: Event) -> Result<()> { /* ... */ }
```

---

## SLAP (Single Level of Abstraction)

### ✅ PASS Criteria
- [ ] Function operates at ONE level of abstraction
- [ ] No mixing of high-level orchestration with low-level details
- [ ] Helper functions for different abstraction levels

### ❌ FAIL Indicators
```rust
// BAD: Mixing orchestration with formatting
fn process() {
    let data = fetch_data();

    // HIGH-LEVEL orchestration
    let result = transform(data);

    // LOW-LEVEL formatting (SLAP violation!)
    let output = if result.is_empty() {
        "(none)".to_string()
    } else {
        result.iter().map(|x| format!("{:?}", x)).collect::<Vec<_>>().join(", ")
    };

    save(output);
}
```

### Quick Fix
```rust
// GOOD: Extract formatting to helper
fn process() {
    let data = fetch_data();
    let result = transform(data);
    let output = format_output(&result);  // Same level of abstraction
    save(output);
}

fn format_output(result: &[Item]) -> String {
    // Low-level details isolated
    if result.is_empty() {
        "(none)".to_string()
    } else {
        result.iter().map(|x| format!("{:?}", x)).collect::<Vec<_>>().join(", ")
    }
}
```

---

## KISS (Keep It Simple, Stupid)

### ✅ PASS Criteria
- [ ] Simplest solution that works
- [ ] No abstraction until ≥3 similar cases
- [ ] No premature optimization
- [ ] Direct construction over builder pattern (for simple structs)

### ❌ FAIL Indicators
- Builder pattern for struct with <5 fields
- Generic type parameters with only 1 usage
- Factory pattern where `new()` suffices
- Multiple layers of indirection

### Examples

**Builder Pattern (OVER-ENGINEERED)**
```rust
// BAD: 50 lines for 3-field struct
pub struct ConfigBuilder { /* ... */ }
impl ConfigBuilder {
    pub fn new() -> Self { /* ... */ }
    pub fn threshold(mut self, val: u16) -> Self { /* ... */ }
    pub fn build(self) -> Config { /* ... */ }
}

// GOOD: Direct construction
pub struct Config {
    pub threshold: u16,
    pub mode: Mode,
}

impl Default for Config { /* ... */ }
```

**Unnecessary Generics (OVER-ENGINEERED)**
```rust
// BAD: Generic with only 1 instantiation
pub struct Processor<const N: usize = 32> { /* ... */ }

// GOOD: Fixed constant
pub struct Processor {
    const MAX_PENDING: usize = 32;
    // ...
}
```

---

## Complexity Metrics

### ✅ PASS Criteria
- [ ] Cyclomatic complexity ≤10 per function
- [ ] Nesting depth ≤3 levels
- [ ] No "arrow code" (excessive indentation)

### ❌ FAIL Indicators
```rust
// BAD: Nesting depth = 5
fn process() {
    if condition1 {
        for item in items {
            if condition2 {
                match item {
                    Some(x) => {
                        if condition3 {
                            // Too deep!
                        }
                    }
                }
            }
        }
    }
}
```

### Quick Fix
```rust
// GOOD: Early returns, guard clauses
fn process() {
    if !condition1 { return; }  // Guard clause

    for item in items {
        process_item(item);  // Extract nested logic
    }
}

fn process_item(item: Item) {
    if !condition2 { return; }

    let Some(x) = item else { return; };

    if condition3 {
        // Now only 2 levels deep
    }
}
```

---

## Pre-Commit Checklist

### Before `git commit`:

- [ ] **File size:** All files ≤500 code lines
- [ ] **Function size:** All functions ≤50 lines
- [ ] **SLAP:** Each function at single abstraction level
- [ ] **KISS:** No over-engineering
- [ ] **Complexity:** No function >10 cyclomatic complexity
- [ ] **Nesting:** No nesting >3 levels
- [ ] **Tests:** All tests pass
- [ ] **Linter:** `cargo clippy` passes
- [ ] **Formatter:** `cargo fmt --check` passes

### Commands
```bash
# Check violations
make verify

# Auto-fix formatting
cargo fmt

# Run tests
cargo test --workspace
cd keyrx_ui && npm test
```

---

## When to Refactor

### Immediate (Block PR)
- File >500 code lines
- Function >50 lines
- Critical SLAP violation (mixing I/O with business logic)

### Short-term (Next sprint)
- Complexity >10
- Nesting >3 levels
- Unnecessary abstraction

### Long-term (Technical debt)
- Builder patterns for simple structs
- Unused generic parameters
- Premature optimization

---

## Tools

### Automated Checks
```bash
# File size check (in pre-commit hook)
find . -name "*.rs" -exec sh -c '
  lines=$(grep -cv "^[[:space:]]*$\|^[[:space:]]*//" "$1")
  if [ "$lines" -gt 500 ]; then
    echo "ERROR: $1 has $lines lines (max 500)"
    exit 1
  fi
' _ {} \;

# Complexity check (via clippy)
cargo clippy -- -W clippy::cognitive_complexity
```

### Manual Review
- Read code aloud - does it make sense?
- Can a new team member understand it in <5 minutes?
- Could this be simpler?

---

## Resources

- **Full Audit Report:** `docs/kiss-slap-audit.md`
- **Refactoring Guide:** `docs/architecture/refactoring-guide.md`
- **Guidelines:** `.claude/CLAUDE.md`

---

**Remember:**

> "Simplicity is prerequisite for reliability." - Edsger Dijkstra

> "Any fool can write code that a computer can understand. Good programmers write code that humans can understand." - Martin Fowler
