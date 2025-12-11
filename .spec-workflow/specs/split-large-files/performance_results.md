# Incremental Compilation Performance Results

**Test Date:** 2025-12-12
**Test Environment:** Linux 6.14.0-36-generic
**Rust Compiler:** cargo build (default toolchain)

## Executive Summary

Rust's incremental compilation operates at the **crate level**, not the individual file level. When any source file within `keyrx_core` is modified, the entire crate is recompiled. The file splitting effort provides significant benefits for **code organization, maintainability, and review quality**, but does not reduce incremental compilation time since Rust recompiles the full crate.

## Build Time Measurements

### Full Build Times (Clean Build)

| Build Type | Time | Notes |
|------------|------|-------|
| Release (`--release`) | 50.28s | Optimized build |
| Debug (default) | 1m 30s | Unoptimized + debuginfo |

### Incremental Build Times (After Touching Submodules)

#### Release Profile

| Submodule Modified | Build Time |
|-------------------|------------|
| scripting/bindings/keyboard.rs | 18.41s |
| engine/state/key_state.rs | 18.26s |
| engine/transitions/log/entry.rs | 18.39s |
| engine/advanced/combos.rs | 18.18s |
| config/loader/parsing.rs | 18.06s |
| validation/engine/rules.rs | 18.32s |
| **Average** | **18.27s** |

#### Debug Profile

| Submodule Modified | Build Time |
|-------------------|------------|
| scripting/bindings/keyboard.rs | 4.54s |
| engine/state/key_state.rs | 4.39s |
| engine/advanced/combos.rs | 4.39s |
| registry/profile/storage.rs | 4.36s |
| validation/engine/report.rs | 4.37s |
| cli/commands/run/setup.rs | 4.38s |
| **Average** | **4.41s** |

## Analysis

### Why No Per-File Compilation Improvement

1. **Crate-Level Compilation**: Rust compiles at the crate level. The `keyrx_core` crate is a single compilation unit.

2. **Incremental Caching**: Cargo does use incremental compilation internally, but it's based on changed code structures (functions, types, etc.), not file boundaries.

3. **Dependency Graph**: When a file is touched, Cargo invalidates the incremental cache for all code that might depend on it within the crate.

### Actual Benefits of File Splitting

Despite no reduction in build times, the file splitting effort provides substantial value:

| Benefit | Impact |
|---------|--------|
| **Code Readability** | Files are now focused and easier to understand |
| **Merge Conflicts** | Smaller files reduce conflict probability |
| **Code Review** | Smaller diffs make reviews more effective |
| **Navigation** | Logical module structure aids code discovery |
| **Maintainability** | Separation of concerns is clearer |
| **Onboarding** | New contributors can understand modules faster |

### Files Split

| Original File | Lines | New Modules |
|--------------|-------|-------------|
| scripting/bindings.rs | 1,893 | 8 modules |
| engine/state/mod.rs | 1,570 | 4 modules |
| engine/transitions/log.rs | 1,403 | 4 modules |
| bin/keyrx.rs | 1,382 | 4 modules |
| scripting/docs/generators/html.rs | 1,069 | 3 modules |
| validation/engine.rs | 968 | 3 modules |
| config/loader.rs | 949 | 3 modules |
| registry/profile.rs | 918 | 3 modules |
| engine/advanced.rs | 906 | 3 modules |
| cli/commands/run.rs | 899 | 3 modules |
| **Total** | ~12,000 | 38 modules |

## Recommendations for Build Speed

If build speed is a concern, consider these alternatives:

1. **Split into Multiple Crates**: Extract independent features into separate crates (e.g., `keyrx-validation`, `keyrx-scripting`)

2. **Use cargo-nextest**: Parallel test execution can speed up test runs

3. **Disable LTO for Dev**: Ensure link-time optimization is only for release

4. **Use mold Linker**: Faster linking on Linux: `RUSTFLAGS="-C link-arg=-fuse-ld=mold"`

5. **Parallel Frontend**: `RUSTFLAGS="-Z threads=8"` (nightly only)

## Conclusion

The file splitting effort achieves its primary goals of **code organization** and **maintainability**. While it does not reduce incremental build times (due to Rust's crate-level compilation model), the benefits for code quality, review efficiency, and contributor experience are substantial.

**Build time status**: Consistent at ~4.4s (debug) / ~18.3s (release) per incremental build
**Code quality status**: Significantly improved (38 focused modules from 10 large files)
