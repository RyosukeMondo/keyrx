# KISS & SLAP Quick Reference Card

**For developers and code reviewers**

---

## KISS (Keep It Simple, Stupid)

### Golden Rules:

1. **File Size Limit: 500 lines** (code only, excluding comments/blanks)
2. **Function Size Limit: 50 lines** (stretch to 100 if necessary)
3. **Cyclomatic Complexity: < 10** (< 5 is ideal)
4. **No speculative generality** - Don't build features you might need someday
5. **Extract on third duplication** - Not before, not after

### Red Flags:

❌ Builder pattern for simple structs (< 5 fields)
❌ Generic with 4+ type parameters
❌ Nested conditionals > 3 levels deep
❌ Function with > 5 parameters
❌ God objects (classes/modules doing too much)

### How to Check:

```bash
# Check file sizes
tokei <file> --files

# Check function complexity
cargo clippy -- -W clippy::too_many_lines -W clippy::cognitive_complexity

# Count functions
grep -c "^fn " <file>
```

---

## SLAP (Single Level of Abstraction Principle)

### Golden Rules:

1. **Each function operates at ONE abstraction level**
2. **Extract helpers for low-level details**
3. **Use layers: High-level calls → Mid-level orchestration → Low-level details**
4. **No mixing: Don't put bit manipulation in business logic**

### Examples:

#### ❌ BAD (Mixed Levels):
```rust
pub fn process_event(event: KeyEvent) -> Vec<KeyEvent> {
    // HIGH-LEVEL
    let mapping = self.find_mapping(event.keycode());

    // LOW-LEVEL (bit manipulation)
    let hash = (event.keycode as u32) * 2654435761;
    let index = (hash % self.mphf_size) as usize;

    // HIGH-LEVEL
    Ok(vec![KeyEvent::new(...)])
}
```

#### ✅ GOOD (Single Level):
```rust
pub fn process_event(&mut self, event: KeyEvent) -> Vec<KeyEvent> {
    let mapping = self.lookup.find_mapping(event.keycode())?;  // All high-level
    let result = self.tap_hold.process(event, mapping)?;
    Ok(self.generate_output(result))
}

// Low-level details in separate module (lookup.rs)
fn find_mapping(&self, keycode: u16) -> Option<Mapping> {
    let hash = self.compute_hash(keycode);  // Low-level isolated here
    let index = self.get_index(hash);
    self.mappings.get(index)
}
```

### How to Check:

1. Read function top to bottom
2. Do all statements feel like the same "altitude"?
3. Can you replace low-level details with descriptive helper names?

---

## Quick Audit Checklist

### Before Committing Code:

- [ ] File < 500 lines? (run `tokei <file> --files`)
- [ ] Function < 50 lines? (check largest functions)
- [ ] No mixed abstraction levels? (read through main functions)
- [ ] No unused imports? (run `cargo clippy`)
- [ ] Tests pass? (run `cargo test`)
- [ ] Coverage maintained? (run `cargo tarpaulin` on changed modules)

### Before Merging PR:

- [ ] No new file size violations?
- [ ] No new function complexity warnings?
- [ ] Abstractions appropriate for problem complexity?
- [ ] Code follows existing patterns?
- [ ] Documentation updated?

---

## Common Refactoring Patterns

### Pattern 1: Extract Helper Function

**BEFORE:**
```rust
pub fn run_loop() {
    loop {
        let event = capture_event();

        // 20 lines of processing
        let result = if condition {
            // Complex logic
        } else {
            // More complex logic
        };

        inject_output(result);
    }
}
```

**AFTER:**
```rust
pub fn run_loop() {
    loop {
        let event = capture_event();
        let result = process_event(event);  // ← Extracted
        inject_output(result);
    }
}

fn process_event(event: KeyEvent) -> KeyEvent {
    // Complex logic isolated here
}
```

### Pattern 2: Extract Data to Module

**BEFORE:**
```typescript
// component.tsx (800 lines)
export const Component = () => {
  const data = [
    { id: 1, ... },
    { id: 2, ... },
    // 200 more lines
  ];

  // 600 lines of logic
};
```

**AFTER:**
```typescript
// data/componentData.ts (200 lines)
export const COMPONENT_DATA = [ ... ];

// components/Component.tsx (400 lines)
import { COMPONENT_DATA } from '@/data/componentData';
export const Component = () => {
  // 400 lines of logic
};
```

### Pattern 3: Extract Validation Logic

**BEFORE:**
```rust
pub fn create_profile(name: &str) -> Result<Profile> {
    if name.is_empty() {
        return Err(Error::EmptyName);
    }
    if name.len() > 50 {
        return Err(Error::NameTooLong);
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(Error::InvalidCharacters);
    }

    // 50 more lines of creation logic
}
```

**AFTER:**
```rust
pub fn create_profile(name: &str) -> Result<Profile> {
    validate_profile_name(name)?;  // ← Extracted

    // Creation logic (now at single abstraction level)
}

fn validate_profile_name(name: &str) -> Result<()> {
    if name.is_empty() { return Err(Error::EmptyName); }
    if name.len() > 50 { return Err(Error::NameTooLong); }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(Error::InvalidCharacters);
    }
    Ok(())
}
```

---

## Tooling Reference

### Rust:

```bash
# Check file sizes
tokei <file> --files

# Function complexity
cargo clippy -- -W clippy::too_many_lines -W clippy::cognitive_complexity

# Count functions in file
grep -c "^fn " <file>

# Find large functions
cargo clippy --package <crate> -- -W clippy::too_many_lines 2>&1 | grep "too_many_lines"

# Test coverage
cargo tarpaulin --workspace

# Auto-fix
cargo fix --allow-dirty
cargo fmt
```

### TypeScript:

```bash
# Check file sizes
tokei <file> --files

# Count lines (excluding comments)
npx cloc <file> --json

# Complexity analysis
npx eslint <file> --rule 'complexity: ["error", 10]'

# Test coverage
npm test -- --coverage
```

---

## Grade Interpretation

| Score | Quality Level | Action |
|-------|---------------|--------|
| **9.0-10.0** | Excellent | Maintain quality |
| 7.0-8.9 | Good | Minor refactoring |
| 5.0-6.9 | Fair | Moderate refactoring |
| 3.0-4.9 | Poor | Significant refactoring |
| 0.0-2.9 | Critical | Immediate action |

**Current keyrx Grade: 9.0/10** ✅

---

## When to Refactor?

### Extract Helper Function When:
- Function > 50 lines
- Mixed abstraction levels (high + low in same function)
- Duplicate logic (> 2 occurrences)
- Complex conditional (> 3 nested levels)

### Split File When:
- File > 500 lines of code
- Multiple responsibilities in one file
- Hard to find specific functionality
- Import list > 15 items

### Simplify Abstraction When:
- Builder for simple struct (< 5 fields)
- Generic with > 3 type parameters (and not in tests)
- Interface with > 5 methods (consider ISP)
- Deep inheritance (> 2 levels)

---

## Resources

- **Full Audit Report:** `docs/kiss-slap-audit-2026.md`
- **Grade Summary:** `docs/kiss-slap-grade-summary.md`
- **Audit Script:** `scripts/kiss_slap_audit.sh`
- **CLAUDE.md Guidelines:** `.claude/CLAUDE.md` (section on Code Quality)

---

**Last Updated:** 2026-02-01
**Next Audit:** Q2 2026 (May 2026)
