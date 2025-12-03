# Dependency Audit Report

**Date:** 2025-12-03
**Purpose:** Identify optimization opportunities in Cargo dependencies

## Executive Summary

The project currently uses **tokio = { version = "1", features = ["full"] }** which includes many unused features. This audit identifies specific features needed and unused dependencies, targeting a >20% reduction in build times.

### Key Findings
- **Tokio**: Using "full" feature set but only needs ~6 specific features
- **Serde**: Already optimized with selective features
- **Windows-rs**: Already uses minimal feature set
- **Platform gating**: Already uses target-specific dependencies correctly
- **Opportunities**: Primarily in tokio, some optimization possible for optional features

---

## Primary Dependencies Analysis

### 1. tokio (CRITICAL OPTIMIZATION TARGET)

**Current:** `tokio = { version = "1", features = ["full"] }`

**Actual Usage Analysis:**
```rust
// Runtime features
- #[tokio::main]                    → needs "rt", "macros"
- #[tokio::test]                    → needs "rt", "macros" (dev)
- tokio::runtime::Builder           → needs "rt"

// Async primitives
- tokio::spawn()                    → needs "rt" (multi-thread)
- tokio::task::LocalSet             → needs "rt" (current-thread)
- tokio::task::spawn_local()        → needs "rt"

// Sync primitives
- Used by crossbeam-channel         → may need "sync" (channels)

// Time features
- tokio::time::sleep()              → needs "time"
- tokio::time::timeout()            → needs "time"
- tokio::time::Duration             → needs "time"

// Signal handling
- tokio::signal::ctrl_c()           → needs "signal"
```

**Recommended Configuration:**
```toml
tokio = { version = "1", default-features = false, features = [
    "rt",           # Basic runtime
    "rt-multi-thread", # For tokio::spawn
    "macros",       # For #[tokio::main] and #[tokio::test]
    "time",         # For sleep/timeout
    "signal",       # For ctrl_c handling
    "sync",         # For channels (if needed, verify)
]}
```

**Unused Features from "full":**
- `io-util` - No tokio I/O operations found
- `io-std` - No stdin/stdout async usage
- `net` - No tokio networking (TcpStream, etc.)
- `fs` - No tokio::fs usage
- `process` - No tokio::process usage
- `parking_lot` - Not explicitly needed

**Estimated Impact:** 30-40% reduction in tokio compile time

---

### 2. serde

**Current:** `serde = { version = "1", features = ["derive"] }`

**Status:** ✅ Already optimized
- Only includes "derive" feature
- Required for config serialization (JSON, TOML)

**No changes recommended.**

---

### 3. tracing-subscriber

**Current:** `tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }`

**Status:** ✅ Minimal feature set
- `env-filter`: Used for RUST_LOG filtering
- `json`: Used for structured logging

**No changes recommended.**

---

### 4. clap

**Current:** `clap = { version = "4", features = ["derive"] }`

**Status:** ✅ Minimal feature set
- Only includes derive macros
- Used throughout CLI commands

**No changes recommended.**

---

### 5. chrono

**Current:** `chrono = { version = "0.4", features = ["serde", "clock"] }`

**Usage:** Timestamps in session recording, config

**Potential Optimization:**
```toml
# Verify if "clock" feature is actually needed
chrono = { version = "0.4", default-features = false, features = ["serde", "std"] }
```

**Action:** Check if `Utc::now()` or system time is used (requires "clock")

---

## Platform-Specific Dependencies

### Windows (Already Well-Gated)

```toml
[target.'cfg(windows)'.dependencies]
windows = { version = "0.52", features = [
    "Win32_UI_WindowsAndMessaging",      # Window messages
    "Win32_System_LibraryLoader",        # DLL loading
    "Win32_System_Threading",            # Thread handling
    "Win32_Foundation",                  # Basic types
    "Win32_UI_Input_KeyboardAndMouse",   # Input handling
]}
```

**Status:** ✅ Already minimal
- Only includes specific APIs used
- No "Win32" root feature (would pull everything)

**No changes recommended.**

---

### Linux (Already Well-Gated)

```toml
[target.'cfg(target_os = "linux")'.dependencies]
evdev = { version = "0.12", features = ["tokio"] }
nix = { version = "0.27", features = ["ioctl"] }
signal-hook = "0.3"
```

**Status:** ✅ Already minimal
- evdev: Only tokio integration
- nix: Only ioctl feature
- signal-hook: Default features (minimal)

**No changes recommended.**

---

## Optional Dependencies

### OpenTelemetry (Behind Feature Flag)

```toml
[features]
otel-tracing = ["opentelemetry", "opentelemetry_sdk", "opentelemetry-otlp"]

[dependencies]
opentelemetry = { version = "0.27", optional = true }
opentelemetry_sdk = { version = "0.27", features = ["rt-tokio"], optional = true }
opentelemetry-otlp = { version = "0.27", features = ["tonic"], optional = true }
```

**Status:** ✅ Already feature-gated
- Only compiled when "otel-tracing" feature enabled
- Good design pattern

**No changes recommended.**

---

## Other Dependencies Analysis

### Always-Needed Dependencies (Optimized)

```toml
rhai = "1.16"                    # Script engine - core functionality
thiserror = "1"                  # Error types - minimal
anyhow = "1"                     # Error handling - minimal
tracing = "0.1"                  # Logging - minimal
crossbeam-channel = "0.5"        # Channels - minimal
async-trait = "0.1"              # Trait impl - minimal (proc-macro)
smallvec = "1"                   # Stack vectors - minimal
```

**Status:** ✅ All use default/minimal features

---

### Utility Dependencies (Consider Gating)

```toml
notify = "6.0"                   # File watching (only for --watch mode)
rustyline = "14.0"               # REPL readline (only for interactive mode)
dirs = "6.0"                     # Config directories (always needed)
strsim = "0.11"                  # String similarity (key name suggestions)
rand = "0.9"                     # Random number generation
```

**Optimization Opportunity:**
```toml
[features]
default = []
watch = ["dep:notify"]           # Only for test --watch
repl = ["dep:rustyline"]         # Only for interactive REPL
fuzzy = ["dep:strsim"]           # Only for fuzzy matching

[dependencies]
notify = { version = "6.0", optional = true }
rustyline = { version = "14.0", optional = true }
strsim = { version = "0.11", optional = true }
```

**Impact:** Moderate - these are small crates but rarely used

---

## Dev Dependencies Analysis

```toml
[dev-dependencies]
proptest = "1"                   # Property testing
criterion = "0.5"                # Benchmarking
tokio-test = "0.4"               # Tokio test utilities
tempfile = "3"                   # Temp file creation
serial_test = "3"                # Serial test execution
```

**Status:** ✅ Appropriate for testing
- Only compiled for tests/benches
- Already minimal feature usage

**No changes recommended.**

---

## Build Profile Analysis

### Current Dev Profile
```toml
[profile.dev]
opt-level = 0
debug = true

[profile.dev.package."*"]
opt-level = 2
```

**Status:** ✅ Good optimization
- Fast dev builds (opt-level = 0)
- Optimized dependencies (opt-level = 2)

**Potential Enhancement:**
```toml
[profile.dev]
opt-level = 0
debug = 2
incremental = true              # Add explicit incremental
split-debuginfo = "unpacked"    # Faster debuginfo on supported platforms
```

---

### Current Release Profile
```toml
[profile.release]
opt-level = 3
lto = "thin"
strip = true
panic = "abort"
codegen-units = 1
```

**Status:** ✅ Good for performance

**Alternative for Size Optimization:**
```toml
[profile.release]
opt-level = "z"                 # Optimize for size instead of 3
lto = "thin"                    # Keep thin LTO
strip = true                    # Keep stripping
panic = "abort"                 # Keep panic = abort
codegen-units = 1               # Keep single codegen unit
```

**Trade-off:** Slightly slower runtime (~5-10%) for smaller binary (~15-25%)

---

## Recommendations Summary

### Priority 1: Immediate Impact (Task 3)
1. **Replace tokio "full" with specific features**
   - Remove: `features = ["full"]`
   - Add: `["rt", "rt-multi-thread", "macros", "time", "signal", "sync"]`
   - Verify "sync" is needed (may be transitive from evdev)
   - **Expected impact:** 30-40% faster tokio compilation

### Priority 2: Size Optimization (Task 10)
2. **Change release profile to opt-level = "z"**
   - Test performance impact
   - Measure binary size reduction
   - **Expected impact:** 15-25% smaller binaries

### Priority 3: Optional Features (Tasks 6-8)
3. **Gate rarely-used dependencies**
   - Make `notify`, `rustyline`, `strsim` optional
   - Create features: `watch`, `repl`, `fuzzy`
   - Update CLI to enable features as needed
   - **Expected impact:** Faster builds when features unused

### Priority 4: Verification (Task 2, 14)
4. **Measure before/after metrics**
   - Baseline: Clean build time (dev + release)
   - Baseline: Binary size
   - Compare after optimizations
   - Document in implementation log

---

## Dependency Tree Depth

**Key observations from `cargo tree`:**
- Deep trees: criterion (many plot dependencies), proptest
- These are dev-only, don't affect production builds
- Main dependency tree is relatively flat
- No obvious duplication of crates with different versions

---

## Feature Flag Strategy

### Proposed Feature Structure
```toml
[features]
default = []

# Platform features
full = ["windows-driver", "linux-driver", "cli-full"]

# Optional features
cli-full = ["repl", "watch", "fuzzy"]
repl = ["dep:rustyline"]
watch = ["dep:notify"]
fuzzy = ["dep:strsim"]
otel-tracing = ["dep:opentelemetry", "dep:opentelemetry_sdk", "dep:opentelemetry-otlp"]

# Platform-specific (automatically enabled by target)
windows-driver = []  # Marker feature for docs
linux-driver = []    # Marker feature for docs
```

**Note:** Platform dependencies already use `[target.'cfg(...)'.dependencies]` which is superior to feature flags for platform code.

---

## Action Items

1. ✅ Complete dependency audit
2. [ ] Measure baseline build metrics (Task 2)
3. [ ] Optimize tokio features (Task 3)
4. [ ] Consider serde default-features = false (Task 4)
5. [ ] Windows features already minimal (Task 5 - verify only)
6. [ ] Create optional feature flags (Task 6)
7. [ ] Test feature combinations (Task 15)
8. [ ] Measure optimized build metrics (Task 14)
9. [ ] Document feature flags (Task 16)

---

## Notes

- The codebase already follows many best practices
- Platform-specific code is already well-gated
- Main optimization opportunity is tokio "full" feature
- Secondary optimizations are smaller but cumulative
- Feature gating of utilities is optional but recommended for flexibility
