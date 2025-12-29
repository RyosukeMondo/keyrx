# Feature Candidate: MPHF Lookup System

**Status:** Deferred (Not Recommended for Current Implementation)
**Date Evaluated:** 2025-12-29
**Evaluated By:** AI Agent + User Review
**Current Alternative:** HashMap-based lookup (hashbrown)

---

## Executive Summary

The Minimal Perfect Hash Function (MPHF) Lookup System was proposed to replace the current HashMap-based key lookup with a compile-time generated, zero-collision hash function. After benchmarking and analysis, **we recommend deferring this feature** because:

1. Current HashMap performance (4.7ns) already exceeds requirements by 21,000x
2. MPHF would provide ~2ns improvement (negligible in real-world usage)
3. Implementation cost (~1000 lines) is high relative to benefit
4. The real bottleneck is OS I/O (1,000-10,000ns), not lookup

**Recommendation:** Focus development effort on higher-impact areas (Windows testing, multi-device scenarios, user documentation).

---

## What is MPHF?

A **Minimal Perfect Hash Function** is a hash function that:
- Maps `n` keys to exactly `n` unique values (no collisions)
- Is "minimal" because it uses exactly `n` slots (no gaps)
- Is generated at compile-time using algorithms like CHD (Compress, Hash and Displace)
- Provides guaranteed O(1) lookup with zero collisions

### Proposed Implementation

**Architecture:**
```
┌─────────────────────────────────────┐
│ keyrx_compiler (build-time)         │
│                                     │
│  1. Extract all KeyCodes from config│
│  2. Generate MPHF using boomphf CHD │
│  3. Serialize MPHF params to .krx   │
└─────────────────┬───────────────────┘
                  │
                  ▼
      ┌───────────────────────────────┐
      │ .krx binary                   │
      │  - Magic/version/hash         │
      │  - Config data (rkyv)         │
      │  - MPHF parameters ← NEW      │
      └───────────────┬───────────────┘
                      │
                      ▼
      ┌───────────────────────────────┐
      │ keyrx_core (runtime)          │
      │                               │
      │  1. Load MPHF params (mmap)   │
      │  2. Evaluate hash(KeyCode)    │
      │  3. Verify key match          │
      │  4. Return mapping            │
      └───────────────────────────────┘
```

**Key Dependencies:**
- `boomphf` (0.6+): CHD algorithm implementation
- `rkyv`: Zero-copy serialization of MPHF parameters

---

## Current HashMap Performance

### Benchmark Results (2025-12-29)

**Hardware:** Linux x86_64
**Compiler:** rustc 1.70+
**Test Config:** 100 key mappings (realistic scenario)

```
Key Lookup (HashMap):     4.7 ns  ← find_mapping()
State Update:             1.4 ns  ← set_modifier()
End-to-End Processing:   21.7 ns  ← complete event pipeline
```

### Performance vs Requirements

| Metric | Requirement | Actual | Margin |
|--------|-------------|--------|--------|
| **Key Lookup** | <100μs (100,000ns) | **4.7ns** | **21,276x faster** |
| **State Update** | <10μs (10,000ns) | **1.4ns** | **7,143x faster** |
| **End-to-End** | <1ms (1,000,000ns) | **21.7ns** | **46,083x faster** |

**Conclusion:** HashMap already exceeds all performance targets by orders of magnitude.

---

## Pros and Cons Analysis

### ✅ Pros of MPHF

#### 1. Theoretical Performance Gain
- **HashMap**: 4.7ns average (measured)
- **MPHF**: 2-3ns optimistic estimate
- **Improvement**: ~2ns (42% faster lookup)

#### 2. Perfect Determinism
- **HashMap**: Hash function is deterministic, table layout can vary
- **MPHF**: Identical across all compilations and platforms
- **Benefit**: Reproducible benchmarks, easier debugging

#### 3. Memory Efficiency
- **HashMap overhead**: ~24 bytes per entry (buckets, capacity, metadata)
- **MPHF overhead**: ~8 bytes per key (displacement table only)
- **Savings**: With 100 mappings, saves ~1.6KB

#### 4. Zero Collisions Guarantee
- **HashMap**: Uses Robin Hood hashing, collisions are rare but possible
- **MPHF**: Mathematically guaranteed zero collisions
- **Benefit**: Absolutely consistent latency (no worst-case scenarios)

#### 5. Cache Locality
- **HashMap**: Can have scattered memory access
- **MPHF**: More predictable access patterns
- **Benefit**: Slightly better CPU cache utilization

### ❌ Cons of MPHF

#### 1. High Implementation Complexity ⚠️
**Estimated effort:** ~1000 lines of code

```
Component                     Lines    Complexity
─────────────────────────────────────────────────
keyrx_compiler/mphf_gen.rs    200-300  Algorithm integration
keyrx_core/lookup.rs          100-150  Runtime evaluation
Serialization (rkyv)           50-100  Binary format
Unit tests                    200-250  CHD correctness
Property tests                 50-100  Invariant checks
Integration tests              50-100  End-to-end
─────────────────────────────────────────────────
TOTAL                         650-1000 lines
```

**Risk:** Bugs in MPHF generation could cause silent lookup failures.

#### 2. Slower Compile-Time
- **HashMap**: Instant construction at runtime
- **MPHF**: CHD algorithm takes 0.1-1s for 1000 keys
- **Impact**: Slower `.krx` compilation (acceptable for offline tool, but noticeable)

#### 3. Loss of Runtime Flexibility
- **HashMap**: Can add/remove mappings at runtime (if needed in future)
- **MPHF**: Fixed at compile-time, cannot change
- **Impact:** Blocks future features like:
  - Live config editing via web UI
  - Dynamic macro recording
  - Per-application config switching

#### 4. Dependency Risk
- **boomphf**: Last commit 2 years ago, minimal maintenance
- **hashbrown**: Actively maintained, will be std::HashMap backend
- **Risk**: Future Rust version incompatibilities

#### 5. Negligible Real-World Impact
**System-level latency breakdown:**

```
Component                Latency     % of Total
────────────────────────────────────────────────
OS Input (evdev/WM_INPUT)  1-10μs      95-99%
System calls (ioctl/poll)  1-5μs       1-4%
KeyLookup (HashMap)        0.0047μs    0.05%
State update               0.0014μs    0.01%
────────────────────────────────────────────────
TOTAL (typical)            ~10μs       100%
```

**MPHF improvement:** 4.7ns → 2.7ns = **2ns saved**
**Real system impact:** 10,000ns → 9,998ns = **0.02% improvement**

**Conclusion:** The bottleneck is OS I/O, not lookup. Optimizing lookup has no measurable user impact.

---

## Recommendation: DEFER

### Why Not Implement MPHF Now?

1. **No Performance Bottleneck**
   - Current HashMap at 4.7ns is already exceptional
   - 21,000x faster than requirements
   - 2ns improvement is unmeasurable in practice

2. **Poor Return on Investment**
   - ~1000 lines of complex code
   - 0.02% system-wide improvement
   - High maintenance burden

3. **Future Flexibility**
   - HashMap allows runtime updates
   - May enable features like live config editing

4. **Real Bottlenecks Elsewhere**
   - OS I/O: 95-99% of latency
   - System calls: 1-4% of latency
   - Lookup: 0.05% of latency (not a bottleneck)

### Better Use of Development Time

1. **Windows Real Hardware Testing**
   - Test Raw Input API on multiple devices
   - Validate hot-plug behavior
   - Performance testing on Windows

2. **Multi-Device Scenarios**
   - Cross-device modifier testing
   - USB numpad as macro pad
   - Split keyboard workflows

3. **User Documentation**
   - Windows setup guide
   - Common configuration examples
   - Troubleshooting FAQ

4. **New Features**
   - Combo keys (Press A+B → C)
   - Macro recording
   - Per-application profiles

5. **OS I/O Optimization**
   - Optimize poll() timeout strategy
   - Reduce syscall overhead
   - Investigate io_uring (Linux) or IOCP (Windows)

---

## When to Revisit MPHF

Consider implementing MPHF if **any** of these conditions become true:

### 1. Profiling Shows Lookup is Bottleneck
- **Trigger:** Lookup contributes >10% of total latency
- **Likelihood:** Extremely low (currently 0.05%)
- **Action:** Re-profile with real workloads

### 2. Embedded System Target
- **Trigger:** Targeting devices with <1MB RAM
- **Rationale:** Memory efficiency becomes critical
- **Action:** Evaluate MPHF size vs HashMap overhead

### 3. Safety-Critical Application
- **Trigger:** Formal verification required
- **Rationale:** Zero collision guarantee may be needed
- **Action:** Consider MPHF + formal proof

### 4. Configuration Size Explosion
- **Trigger:** Users create configs with >10,000 mappings
- **Likelihood:** Low (typical configs have 50-200 mappings)
- **Action:** Benchmark HashMap with large configs first

### 5. Determinism is Critical
- **Trigger:** Need bit-identical behavior across platforms for debugging
- **Rationale:** MPHF provides perfect reproducibility
- **Action:** Evaluate if HashMap determinism is insufficient

---

## Alternative Optimizations (If Needed)

If lookup performance ever becomes a concern (unlikely), consider these **before MPHF:**

### 1. Perfect Hash (Not Minimal)
- Use `phf` crate instead of `boomphf`
- Simpler than MPHF, still zero collisions
- Allows gaps (not minimal), but easier to generate

### 2. Direct Array Indexing
- If KeyCode enum is small and contiguous
- Use `mapping_table[keycode as usize]`
- O(1) with zero overhead (just an array access)
- **Limitation:** Only works if KeyCode range is small (~256 values)

### 3. Two-Level Lookup
- First level: Fast bitset for "has mapping" check
- Second level: HashMap only for keys with mappings
- Reduces HashMap size, improves cache locality

### 4. SIMD Lookup (Advanced)
- Use SIMD instructions to check multiple keys at once
- Requires architecture-specific code (x86 AVX, ARM NEON)
- Only beneficial for batch processing

---

## Implementation Spec (If Reconsidered)

If conditions change and MPHF becomes worthwhile, the implementation would follow these requirements:

### Requirements Summary

**REQ-1: Compile-Time MPHF Generation**
- Compiler SHALL identify all unique KeyCodes in config
- Compiler SHALL generate MPHF using CHD algorithm (boomphf)
- Generated MPHF SHALL be minimal (range = number of keys)
- Generation SHALL be deterministic (same input → same MPHF)

**REQ-2: Runtime O(1) Lookup**
- Runtime SHALL use MPHF to calculate mapping index
- Lookup SHALL verify key match (handle non-members)
- Lookup performance target: <50ns (vs <100μs requirement)

**REQ-3: Binary Serialization**
- .krx format SHALL include MPHF parameters (seeds, displacements)
- Parameters SHALL be serialized with rkyv for zero-copy access
- KeyLookup SHALL be compatible with serialized MPHF

**REQ-4: Code Quality**
- Implementation SHALL avoid unsafe code (or justify with safety proofs)
- Implementation SHALL handle 0, 1, and 10,000+ keys gracefully
- Compiler SHALL report MPHF quality metrics (efficiency, size)

### Estimated Implementation Tasks

1. **Compiler MPHF Generation** (3-5 days)
   - Integrate boomphf CHD algorithm
   - Extract KeyCodes from DeviceConfig
   - Generate MPHF parameters
   - Serialize to .krx binary

2. **Runtime MPHF Evaluation** (2-3 days)
   - Deserialize MPHF from .krx
   - Implement hash evaluation
   - Key verification (non-member handling)

3. **Testing** (3-4 days)
   - Unit tests: CHD correctness
   - Property tests: Collision-free invariant
   - Integration tests: End-to-end lookup
   - Benchmarks: Compare vs HashMap

4. **Documentation** (1-2 days)
   - Update architecture docs
   - Explain MPHF in user guide
   - Document .krx binary format changes

**Total Estimate:** 9-14 days of focused development

---

## References

### MPHF Algorithms
- [Compress, Hash and Displace (CHD)](http://cmph.sourceforge.net/papers/esa09.pdf) - Original CHD paper
- [boomphf crate](https://crates.io/crates/boomphf) - Rust CHD implementation
- [phf crate](https://crates.io/crates/phf) - Perfect hashing (alternative)

### HashMap Performance
- [hashbrown](https://crates.io/crates/hashbrown) - Current implementation
- [Robin Hood Hashing](https://programming.guide/robin-hood-hashing.html) - Collision resolution

### Related Discussions
- [KeyRx Architecture - tech.md](../../.spec-workflow/steering/tech.md) - Original MPHF proposal
- [Basic Key Remapping Spec](../../.spec-workflow/specs/basic-key-remapping/) - Current HashMap implementation

---

## Revision History

| Date | Version | Changes |
|------|---------|---------|
| 2025-12-29 | 1.0 | Initial evaluation, recommendation to defer |

---

## Appendix: Benchmark Details

### Test Environment
```
OS: Linux 6.14.0-37-generic
CPU: x86_64 (specific model not logged)
Rust: 1.70+
Build: --release with default optimizations
```

### Test Configuration
```rust
// 100 mappings: 90 simple + 5 modifiers + 5 locks
let config = create_realistic_config();
let lookup = KeyLookup::from_device_config(&config);
let state = DeviceState::new();

// Benchmark: lookup.find_mapping(KeyCode::H, &state)
```

### Raw Benchmark Output
```
key_lookup              time:   [4.6872 ns 4.7067 ns 4.7308 ns]
state_update_set_modifier
                        time:   [1.3914 ns 1.3938 ns 1.3960 ns]
state_update_toggle_lock
                        time:   [2.1330 ns 2.1354 ns 2.1378 ns]
process_event_simple    time:   [21.626 ns 21.717 ns 21.826 ns]
process_event_modifier  time:   [13.829 ns 13.847 ns 13.867 ns]
```

### Statistical Confidence
- **Sample size:** 100 iterations per benchmark
- **Confidence interval:** 95%
- **Variance:** <2% (highly stable results)

---

**Conclusion:** HashMap-based lookup is already exceptional. MPHF would be premature optimization with negligible real-world benefit. Focus development effort on higher-impact features and testing.
