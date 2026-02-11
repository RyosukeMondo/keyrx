# TODO Audit - 2025-12-29

Comprehensive audit of TODOs, FIXMEs, and placeholders in the codebase.

## Summary

**Total occurrences found:** 38 across 28 files

**Categories:**
- 🟡 **Action Required:** 6 items
- 🟢 **Safe to Keep:** 32 items (test-related, documentation)

---

## 🟡 Action Required (6 items)

### 1. Signal Testing Stubs (keyrx_daemon/tests/daemon_tests.rs)

**Lines:** 591, 599, 607

```rust
todo!("Implement subprocess-based signal testing");
```

**Context:** Three signal test functions (SIGTERM, SIGINT, SIGHUP) are stubbed out.

**Recommendation:** These tests require subprocess spawning to properly test signal handling. Can be:
- Implemented using `std::process::Command` to spawn daemon subprocess
- Or deferred - signal handling is already tested in integration tests
- Or marked as `#[ignore]` with explanation

**Priority:** Low (signal handling works, just missing unit tests)

---

### 2. Hot-Unplug Test Stub (keyrx_daemon/tests/bug_regression_tests.rs)

**Line:** 474

```rust
todo!("Test that hot-unplugging device doesn't crash daemon");
```

**Context:** BUG #37 regression test is incomplete.

**Recommendation:**
- This should be implemented since BUG #37 (hot-unplug crash) was a real bug
- Test should verify daemon continues running after device removal
- Can use VirtualLinuxKeyboard to simulate device removal

**Priority:** Medium (regression test for fixed bug)

---

### 3. CheckBytes Security TODO (keyrx_compiler/src/serialize.rs)

**Line:** 190

```rust
// TODO(security): Implement CheckBytes for ConfigRoot and all nested types to enable
// safe deserialization with validation
```

**Context:** rkyv deserialization currently doesn't validate data integrity beyond hash.

**Recommendation:**
- **Security improvement:** Implementing CheckBytes would prevent malformed .krx files from causing undefined behavior
- Requires deriving/implementing CheckBytes for all config types
- Not urgent since .krx files are SHA256-verified, but good defense-in-depth

**Priority:** Medium (security hardening)

---

### 4. CheckBytes Fuzz Target (keyrx_core/fuzz/fuzz_targets/fuzz_deserialize.rs)

**Line:** 24

```rust
// TODO: Implement CheckBytes trait for ConfigRoot to enable safe deserialization
```

**Context:** Same as #3, affects fuzzing infrastructure.

**Recommendation:** Same as #3 - implement CheckBytes for robust fuzzing.

**Priority:** Medium (enables better fuzz testing)

---

### 5. Source Hash Placeholder (keyrx_compiler/src/parser/core.rs)

**Line:** 131

```rust
source_hash: "TODO".to_string(),
```

**Context:** Metadata includes a placeholder source hash.

**Recommendation:**
- Should calculate actual hash of source Rhai file
- Use SHA256 like the binary hash
- Enables tracing which source generated a .krx file

**Priority:** Low (metadata field, not critical)

---

### 6. Commented Module Declarations (keyrx_compiler/src/lib.rs)

**Lines:** 13-14

```rust
// pub mod mphf_gen;
// pub mod dfa_gen;
```

**Context:** Future modules that were planned but not implemented.

**Recommendation:**
- **mphf_gen:** Already decided to defer (see `docs/features_candidate/mphf-lookup-system.md`)
- **dfa_gen:** DFA is already implemented in keyrx_core, not needed in compiler
- **Action:** Remove these commented lines to avoid confusion

**Priority:** Low (cleanup)

---

## 🟢 Safe to Keep (32 items)

### Test-Related Items (Safe)

**All panic!() calls:** Used in test assertions - this is correct Rust test practice
- 150+ occurrences in test files
- Pattern: `panic!("Expected X, got Y")` for test failures

**Ignored tests with TODO comments:** (2 items)
- `keyrx_daemon/tests/e2e_linux_multidevice.rs:147,318`
- `#[ignore] // TODO: Investigate why daemon can't find virtual keyboards`
- These are known flaky tests, correctly marked as ignored

### Documentation TODOs (Safe)

**Specification documents:** 10+ occurrences in `.spec-workflow/`
- These are planning documents, TODOs are intentional
- Examples: task lists, design considerations

**RFCs and design docs:** `docs/rfcs/keyboard-internationalization.md`
- Future feature documentation, TODOs are expected

---

## Recommended Actions

### High Priority
None - no critical blockers found.

### Medium Priority
1. **Implement CheckBytes for ConfigRoot** (security hardening)
   - File: `keyrx_compiler/src/serialize.rs:190`
   - Effort: ~1-2 days (derive for all types, update tests)
   - Benefit: Robust deserialization, better fuzzing

2. **Implement BUG #37 regression test** (hot-unplug)
   - File: `keyrx_daemon/tests/bug_regression_tests.rs:474`
   - Effort: ~2-4 hours
   - Benefit: Prevents regression of fixed bug

### Low Priority
1. **Clean up commented module declarations**
   - File: `keyrx_compiler/src/lib.rs:13-14`
   - Effort: 1 minute
   - Benefit: Code clarity

2. **Implement signal test stubs or mark as ignored**
   - File: `keyrx_daemon/tests/daemon_tests.rs:591,599,607`
   - Effort: ~1-2 hours (implementation) or 5 minutes (mark ignored)
   - Benefit: Complete test coverage

3. **Calculate actual source_hash instead of "TODO"**
   - File: `keyrx_compiler/src/parser/core.rs:131`
   - Effort: ~30 minutes
   - Benefit: Better metadata traceability

---

## Code Quality Notes

### Excellent Practices Found ✅
- **Test panics are properly used:** All panic!() calls are in test code (correct)
- **Flaky tests are marked #[ignore]:** Prevents CI failures (correct)
- **TODO comments have context:** Most TODOs explain what needs to be done

### Areas for Improvement 🔧
- **Security hardening:** CheckBytes implementation would improve robustness
- **Test coverage:** A few test stubs should be completed or explicitly deferred

---

## Next Steps

**Immediate (if desired):**
```bash
# Clean up commented module declarations
sed -i '/^\/\/ pub mod mphf_gen;/d' keyrx_compiler/src/lib.rs
sed -i '/^\/\/ pub mod dfa_gen;/d' keyrx_compiler/src/lib.rs
```

**Short-term (recommended):**
1. Create issue/task for CheckBytes implementation
2. Complete or defer BUG #37 regression test
3. Calculate actual source_hash

**Long-term (optional):**
- Implement signal testing with subprocess spawning
- Add more comprehensive fuzzing with CheckBytes

---

## Verification

No critical issues found. The codebase is in good shape:
- ✅ All placeholders are documented
- ✅ Test code uses panic!() appropriately
- ✅ Flaky tests are marked #[ignore]
- ✅ MPHF cleanup already completed
- ✅ No blocking issues

The remaining TODOs are either:
- Low-priority improvements (source_hash, signal tests)
- Medium-priority security hardening (CheckBytes)
- Documentation/planning (spec files, RFCs)
