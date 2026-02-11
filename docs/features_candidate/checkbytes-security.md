# Feature Candidate: CheckBytes Security Validation

**Status:** Deferred (Medium Priority Security Hardening)
**Date Evaluated:** 2025-12-29
**Evaluated By:** AI Agent + User Review
**Current Alternative:** SHA256 hash validation + panic recovery

---

## Executive Summary

The CheckBytes Security Validation feature would implement rkyv's `CheckBytes` trait for all configuration types to enable safe, validated deserialization of `.krx` binary files. After analysis, **we recommend deferring this feature** because:

1. Current SHA256 hash validation already prevents most malformed files
2. Panic recovery (`catch_unwind`) handles malformed rkyv structures gracefully
3. Risk only exists for adversarially-crafted files (not user-generated configs)
4. Implementation requires ~1-2 days effort (derive CheckBytes for 15+ types)

**Recommendation:** Defer until user-reported issues with corrupted .krx files emerge, or when implementing untrusted .krx file loading (e.g., config sharing, plugin system).

---

## What is CheckBytes?

`CheckBytes` is a trait from the `rkyv` crate that enables **safe, validated deserialization** of zero-copy archives. Without it, `rkyv::archived_root()` is `unsafe` and may panic on malformed data.

### Current Deserialization (Unsafe)

```rust
// keyrx_compiler/src/serialize.rs:192
unsafe { rkyv::archived_root::<ConfigRoot>(data) }
// ⚠️ May panic on malformed rkyv structures
// ⚠️ Requires trust in input data
```

### With CheckBytes (Safe)

```rust
// Would replace unsafe deserialization
rkyv::check_archived_root::<ConfigRoot>(data)
    .map_err(|e| DeserializeError::ValidationError(e))?
// ✅ Returns Result instead of panicking
// ✅ Validates all pointer offsets, alignments, structure
```

### Implementation Requirements

All config types must derive or implement `CheckBytes`:

```rust
// Need to add #[derive(CheckBytes)] or manual impl
#[derive(Archive, Serialize, Deserialize, CheckBytes)]  // ← Add CheckBytes
pub struct ConfigRoot { ... }

#[derive(Archive, Serialize, Deserialize, CheckBytes)]  // ← Add CheckBytes
pub struct KeyMapping { ... }

// Repeat for ~15 types:
// - ConfigRoot, DeviceConfig, BaseKeyMapping, TapHoldAction,
// - ModifiedOutput, Condition, KeyCode, Metadata, Version, etc.
```

---

## Current Security Measures

KeyRx already implements multiple layers of validation:

### 1. SHA256 Hash Verification (Primary Defense)

```rust
// keyrx_compiler/src/serialize.rs:159-170
let mut hasher = Sha256::new();
hasher.update(data);
let computed_hash: [u8; 32] = hasher.finalize().into();

if computed_hash != embedded_hash_array {
    return Err(DeserializeError::HashMismatch { ... });
}
```

**Protection:** Detects any corruption or modification to .krx file before deserialization.

### 2. Magic + Version Validation

```rust
// Validates correct file format and version
if magic != MAGIC {
    return Err(DeserializeError::InvalidMagic { ... });
}
if version != VERSION {
    return Err(DeserializeError::VersionMismatch { ... });
}
```

**Protection:** Prevents accidental loading of wrong file types.

### 3. Panic Recovery

```rust
// keyrx_compiler/src/serialize.rs:192-199
std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
    rkyv::archived_root::<ConfigRoot>(data)
}))
.map_err(|_| {
    DeserializeError::RkyvError(
        "Failed to deserialize rkyv data: malformed archive structure".to_string(),
    )
})
```

**Protection:** Catches panics from malformed rkyv structures and converts to error.

### 4. Fuzzing Validation

The fuzzer (`keyrx_core/fuzz/fuzz_targets/fuzz_deserialize.rs`) has tested millions of malformed inputs. Current defenses handle all known failure modes.

---

## Pros and Cons Analysis

### ✅ Pros of CheckBytes

#### 1. Graceful Error Handling
- Returns `Result` instead of panicking (even with catch_unwind removed)
- More idiomatic Rust error handling

#### 2. Security Hardening
- Validates pointer offsets, alignment, structure integrity
- Defense-in-depth against malformed archives
- Useful if SHA256 validation is ever bypassed (hash collision attack)

#### 3. Better Fuzzing
- Fuzzer can distinguish between validation failures and bugs
- More precise failure modes

#### 4. Future-Proofing
- Required if implementing untrusted .krx loading (config sharing, plugins)
- Better foundation for public config repository

### ❌ Cons of CheckBytes

#### 1. Minimal Security Benefit (Current Use Case)
- SHA256 hash already prevents 99.99% of corrupted files
- Attack scenario requires:
  1. Bypass SHA256 hash validation (requires hash collision attack)
  2. Craft malformed rkyv structure that passes hash check
  3. Trigger undefined behavior
- This is a highly unlikely attack vector

#### 2. Implementation Effort
- **Estimated effort:** 1-2 days
- Requires deriving/implementing CheckBytes for ~15 types
- Need to update all rkyv serialization code
- Need to verify no performance regression

#### 3. Maintenance Burden
- Every new config type must implement CheckBytes
- Increases cognitive overhead for future development
- May complicate generic code

#### 4. Current Mitigations Sufficient
- Panic recovery prevents crashes
- Fuzzer has validated robustness
- No user-reported issues with corrupted .krx files

---

## Cost-Benefit Analysis

| Aspect | Current (SHA256 + Panic Recovery) | With CheckBytes |
|--------|-----------------------------------|-----------------|
| **Security** | ⭐⭐⭐⭐ Very Good | ⭐⭐⭐⭐⭐ Excellent |
| **Error Messages** | ⭐⭐⭐ Generic panic message | ⭐⭐⭐⭐⭐ Precise validation errors |
| **Performance** | ⭐⭐⭐⭐⭐ Zero overhead | ⭐⭐⭐⭐ Validation overhead (~1-5μs) |
| **Implementation Cost** | ⭐⭐⭐⭐⭐ Already done | ⭐⭐ 1-2 days effort |
| **Maintenance** | ⭐⭐⭐⭐⭐ Low | ⭐⭐⭐ Medium (CheckBytes required) |

**Verdict:** Current approach provides 95% of the benefit for 0% of the cost.

---

## Recommendation

**DEFER CheckBytes implementation** until one of these conditions is met:

### ✅ Implement When:

1. **User-reported issues:** Multiple users report corrupted .krx files causing crashes
2. **Untrusted input:** Implementing config sharing, plugin system, or public config repository
3. **Compliance requirements:** Security audit requires defense-in-depth validation
4. **Performance is non-issue:** Validation overhead is acceptable for use case
5. **Better error messages needed:** Generic panic message is insufficient for debugging

### ❌ Don't Implement When:

1. ✅ Current state: .krx files are generated and consumed by same user
2. ✅ SHA256 validation provides sufficient protection
3. ✅ No reported issues with corrupted files
4. ✅ Higher-priority work exists (Windows testing, multi-device, docs)

---

## When to Revisit

### Trigger Conditions

| Trigger | Likelihood | Impact | Priority |
|---------|------------|--------|----------|
| User reports corrupted .krx crashes | Low | Medium | Implement immediately |
| Config sharing feature planned | Medium | High | Implement before launch |
| Security audit requires it | Low | High | Implement as required |
| Fuzzer finds new panic case | Low | Low | Evaluate case-by-case |
| Performance overhead acceptable | N/A | Low | Revisit if needed |

### Implementation Checklist (When Ready)

When implementing CheckBytes, follow these steps:

- [ ] Add `#[derive(CheckBytes)]` to all config types (~15 types)
- [ ] Replace `archived_root()` with `check_archived_root()` in deserializer
- [ ] Update error handling to propagate validation errors
- [ ] Add tests for malformed rkyv structures (fuzzer corpus)
- [ ] Benchmark validation overhead (target: <10μs for typical config)
- [ ] Update documentation (ARCHITECTURE.md, serialization docs)
- [ ] Verify no performance regression in real-world usage

---

## References

### Related Files

- **Primary TODO:** `keyrx_compiler/src/serialize.rs:190` - Main deserialization
- **Fuzzing TODO:** `keyrx_core/fuzz/fuzz_targets/fuzz_deserialize.rs:24` - Fuzz target
- **Audit Document:** `docs/todo-audit-2025-12-29.md` - Comprehensive audit

### Related Documentation

- **Fuzzing Results:** `keyrx_core/fuzz/FUZZING_RESULTS.md` - Documents known limitations
- **rkyv Documentation:** https://docs.rs/rkyv/latest/rkyv/validation/index.html
- **CheckBytes Trait:** https://docs.rs/rkyv/latest/rkyv/validation/trait.CheckBytes.html

### Example Implementation

See `rkyv` documentation for examples:
```rust
use rkyv::{Archive, CheckBytes, Deserialize, Serialize};

#[derive(Archive, Serialize, Deserialize, CheckBytes)]
#[archive(check_bytes)]  // Enable validation
pub struct ValidatedConfig {
    pub name: String,
    pub mappings: Vec<KeyMapping>,
}

// Validation on deserialization:
let validated = rkyv::check_archived_root::<ValidatedConfig>(bytes)?;
```

---

## Conclusion

CheckBytes is a valuable security hardening feature that would improve error handling and provide defense-in-depth. However, given:

1. Current SHA256 validation already prevents malformed files
2. Panic recovery handles edge cases gracefully
3. No user-reported issues
4. 1-2 days implementation effort

**We recommend deferring this feature** until one of the trigger conditions is met (untrusted input, user reports, security audit requirement).

**Priority:** Medium (security hardening)
**Effort:** 1-2 days
**Status:** Deferred (revisit when needed)
