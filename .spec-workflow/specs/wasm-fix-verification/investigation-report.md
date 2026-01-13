# WASM Build Investigation Report

**Date**: 2026-01-14
**Investigator**: Build Engineer (AI Agent)
**Spec**: wasm-fix-verification
**Task**: 1.1 - Investigate current WASM build state

## Executive Summary

The WASM build infrastructure is **mostly functional** but has several configuration issues that can cause failures and inconsistent behavior. The primary issue is that while WASM files exist and build successfully on this system, there are architectural problems that would prevent reliable operation.

## Key Findings

### ✅ Working Components

1. **WASM files exist and are current**
   - Location: `keyrx_ui/src/wasm/pkg/`
   - Files present:
     - `keyrx_core_bg.wasm` (1.88 MB, built 2026-01-14 02:23)
     - `keyrx_core.js` (20 KB)
     - `keyrx_core.d.ts` (7 KB)
     - `package.json`
     - `README.md`

2. **wasm-pack is installed and working**
   - Version: 0.13.1
   - Location: `/home/rmondo/.cargo/bin/wasm-pack`

3. **Build script executes successfully**
   - Script: `scripts/lib/build-wasm.sh`
   - Build time: ~1 second (cached)
   - Exit code: 0 (success)
   - All required output files verified

4. **keyrx_core Cargo.toml is properly configured**
   - Crate type includes `cdylib` (required for WASM)
   - WASM feature properly defined
   - All required dependencies present:
     - wasm-bindgen
     - serde-wasm-bindgen
     - console_error_panic_hook
     - once_cell
     - web-sys

### ⚠️ Issues Identified

#### Issue 1: CRITICAL - wasm32-unknown-unknown target NOT installed

**Severity**: HIGH
**Impact**: WASM builds will fail on clean systems

**Evidence**:
```bash
$ rustup target list --installed | grep wasm
# (no output - target not installed)
```

**Root Cause**: The wasm32-unknown-unknown Rust target is required for WASM compilation but is not installed by default.

**Current State**: Build succeeds because previously cached artifacts exist, but would fail if:
- Running on a new system
- After `cargo clean`
- After updating Rust toolchain

**Fix Required**: Install the target:
```bash
rustup target add wasm32-unknown-unknown
```

#### Issue 2: HIGH - Dual WASM initialization patterns

**Severity**: HIGH
**Impact**: Multiple WASM initializations, inconsistent state, performance overhead

**Evidence**:
- `keyrx_ui/src/hooks/useWasm.ts` - Old hook with inline initialization
- `keyrx_ui/src/contexts/WasmContext.tsx` - New context provider pattern
- `keyrx_ui/src/pages/SimulatorPage.tsx:67` - Uses `useWasm()` directly instead of context

**Root Cause**: Migration from hook-based to context-based WASM loading incomplete

**Current State**:
- WasmProvider initializes WASM once at app startup (correct)
- SimulatorPage also initializes via useWasm() (duplicate, incorrect)
- MonacoEditor uses WasmContext (correct)

**Architecture Issue**: Two initialization patterns coexist:
```typescript
// Pattern 1 (Deprecated): Direct hook usage
const { isWasmReady, validateConfig } = useWasm();

// Pattern 2 (Correct): Context usage
const { isWasmReady, validateConfig } = useWasmContext();
```

**Fix Required**:
1. Update SimulatorPage to use `useWasmContext()`
2. Deprecate or remove `useWasm()` hook
3. Ensure all components use WasmContext

#### Issue 3: MEDIUM - WASM file size exceeds target

**Severity**: MEDIUM
**Impact**: Longer load times, higher bandwidth usage

**Evidence**:
```
WASM file size: 1882 KB (1.83 MB)
[WARN] WASM file size exceeds 1MB target
```

**Root Cause**: Unoptimized build includes debug symbols and unused code

**Current State**: Build script warns but does not fail

**Fix Required** (Future optimization):
- Enable wasm-opt optimization
- Review feature flags to exclude unnecessary code
- Consider code splitting for large features

#### Issue 4: LOW - Missing Rust target in CI/CD check

**Severity**: LOW
**Impact**: CI/CD may fail inconsistently

**Evidence**: Setup scripts don't explicitly install wasm32 target

**Fix Required**: Add to setup/CI scripts:
```bash
rustup target add wasm32-unknown-unknown
```

## WASM Loading Mechanism Analysis

### Current Implementation (keyrx_ui/src/hooks/useWasm.ts)

**Initialization Process**:
1. `useEffect` runs on component mount
2. Dynamic import: `import('@/wasm/pkg/keyrx_core.js')`
3. Calls `module.wasm_init()` to initialize panic hook
4. Sets state: `isWasmReady`, `isLoading`, `error`
5. Performance logging with timestamps

**Error Handling**:
- Catches import failures: "WASM module not found. Run build:wasm to compile..."
- Catches initialization failures with timing
- Graceful degradation: returns empty arrays/null if WASM not ready

**Validation Flow**:
```
validateConfig(code)
  → wasmModule.load_config(code)
  → Success: return []
  → Error: parse error message, extract line/column, return ValidationError[]
```

**Simulation Flow**:
```
runSimulation(code, input)
  → load_config(code) → configHandle
  → simulate(configHandle, JSON.stringify(input))
  → return SimulationResult | null
```

### Context Implementation (keyrx_ui/src/contexts/WasmContext.tsx)

**Improvements Over Hook**:
- Single initialization at app root (via `<WasmProvider>`)
- Prevents re-initialization on page navigation
- Shared state across all components
- Better performance (WASM loaded once)

**Usage Pattern**:
```tsx
// App.tsx (root)
<WasmProvider>
  <BrowserRouter>
    <Routes>...</Routes>
  </BrowserRouter>
</WasmProvider>

// Component
const { isWasmReady, validateConfig } = useWasmContext();
```

## Error Messages Found in Codebase

### UI Error Messages

1. **MonacoEditor.tsx:49**: `'⚠ WASM unavailable'`
   - Status badge when WASM fails to load
   - User sees this in editor status bar

2. **SimulatorPage.tsx:570**: `'⚠ Using mock simulation (WASM not ready)'`
   - Shown when profile config loaded but WASM not ready
   - Indicates fallback to mock simulation

3. **SimulatorPage.tsx:574**: `'⚠ WASM not available (run build:wasm)'`
   - Shown when WASM not ready and not loading
   - Provides actionable error message

4. **useWasm.ts:81**: `'WASM module not found. Run build:wasm to compile...'`
   - Thrown when import fails (file not found)
   - Caught and displayed in error state

### When Errors Occur

**Scenario 1**: WASM files don't exist
- User sees: "⚠ WASM not available (run build:wasm)"
- Console: "WASM module not found..."
- Impact: No validation, no simulation

**Scenario 2**: WASM files exist but import fails
- User sees: "⚠ WASM unavailable"
- Console: Import error details
- Impact: Graceful degradation to mock

**Scenario 3**: WASM loading in progress
- User sees: "⏳ Loading WASM validator..."
- Console: "[WASM] Starting initialization..."
- Impact: Temporary state, resolves quickly

## Build Script Analysis (scripts/lib/build-wasm.sh)

### Positive Aspects

1. **Comprehensive verification**
   - Checks wasm-pack installation
   - Verifies all required output files
   - Reports file sizes
   - Logs build time

2. **Error handling**
   - Exits with code 1 on failure
   - Clear error messages
   - JSON output mode for CI/CD

3. **Output validation**
   ```bash
   REQUIRED_FILES=(
     "$OUTPUT_DIR/keyrx_core_bg.wasm"
     "$OUTPUT_DIR/keyrx_core.js"
     "$OUTPUT_DIR/keyrx_core.d.ts"
   )
   ```

### Missing Verification

1. **No hash generation**: Build script doesn't generate/store WASM hash
2. **No version matching**: Doesn't verify keyrx_core version matches WASM
3. **No target check**: Doesn't verify wasm32-unknown-unknown target installed
4. **Size check is warning only**: Doesn't fail if size exceeds threshold

## Conclusions

### Root Cause of "WASM not available" Error

The error message appears when:
1. **wasm32 target not installed** (build would fail on clean system)
2. **Component uses wrong hook** (SimulatorPage uses `useWasm()` instead of context)
3. **Import path issues** (if WASM files not built)

### Current State Assessment

| Component | Status | Notes |
|-----------|--------|-------|
| WASM files | ✅ Present | Built and current |
| wasm-pack | ✅ Installed | Version 0.13.1 |
| Build script | ✅ Working | Succeeds with warnings |
| Cargo.toml | ✅ Configured | cdylib + WASM features |
| Rust target | ❌ Missing | wasm32-unknown-unknown not installed |
| UI architecture | ⚠️ Mixed | Dual initialization patterns |
| Verification | ❌ Missing | No hash/version checks |

### Priority Fixes Required

1. **CRITICAL**: Install wasm32-unknown-unknown target
2. **HIGH**: Fix SimulatorPage to use WasmContext
3. **HIGH**: Remove/deprecate useWasm() hook
4. **MEDIUM**: Add WASM build verification (hash, version)
5. **MEDIUM**: Add health check CLI tool
6. **LOW**: Optimize WASM size

## Recommendations for Next Steps

### Immediate (Task 1.2 - Fix WASM build configuration)

1. Add wasm32 target installation to setup scripts
2. Update build-wasm.sh to verify target before building
3. Add hash generation to build script

### Short-term (Tasks 2-3)

1. Update SimulatorPage to use WasmContext
2. Add WASM status badge to UI
3. Create verify-wasm.sh script
4. Add npm scripts for WASM operations

### Long-term (Tasks 4-8)

1. Integrate verification into UAT
2. Create wasm-health.sh diagnostics tool
3. Update documentation
4. Add automated tests

## Test Evidence

### Build Test Output

```
[2026-01-14 02:30:25] [INFO] Building keyrx_core to WebAssembly...
[2026-01-14 02:30:25] [INFO] wasm-pack found: wasm-pack 0.13.1
[2026-01-14 02:30:25] [INFO] Building WASM from /home/rmondo/repos/keyrx2/keyrx_core...
[2026-01-14 02:30:26] [INFO] WASM build completed successfully
[2026-01-14 02:30:26] [INFO] Verifying output files...
[2026-01-14 02:30:26] [INFO] WASM file size: 1882 KB (1.83 MB)
[2026-01-14 02:30:26] === accomplished ===
```

### File Verification

```bash
$ ls -lh keyrx_ui/src/wasm/pkg/
total 1.9M
-rw-rw-r-- 1 rmondo rmondo    1 Jan 14 02:23 .gitignore
-rw------- 1 rmondo rmondo 1.4K Jan 14 02:23 README.md
-rw-rw-r-- 1 rmondo rmondo 7.1K Jan 14 02:23 keyrx_core.d.ts
-rw-rw-r-- 1 rmondo rmondo  20K Jan 14 02:23 keyrx_core.js
-rw-rw-r-- 1 rmondo rmondo 1.9M Jan 14 02:23 keyrx_core_bg.wasm
-rw-rw-r-- 1 rmondo rmondo  884 Jan 14 02:23 keyrx_core_bg.wasm.d.ts
-rw-rw-r-- 1 rmondo rmondo  433 Jan 14 02:23 package.json
```

## Appendices

### A. WASM Import Path

Current path in code:
```typescript
import('@/wasm/pkg/keyrx_core.js')
```

Maps to:
```
keyrx_ui/src/wasm/pkg/keyrx_core.js
```

Vite alias `@/` resolves to `keyrx_ui/src/`.

### B. Cargo.toml WASM Configuration

```toml
[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = { workspace = true, optional = true }
serde-wasm-bindgen = { workspace = true, optional = true }
console_error_panic_hook = { workspace = true, optional = true }
once_cell = { workspace = true, optional = true }
web-sys = { workspace = true, optional = true }
rhai = { workspace = true, optional = true }
sha2 = { workspace = true, optional = true }
serde_json = { workspace = true, optional = true }
getrandom = { version = "0.3", features = ["wasm_js"] }

[features]
wasm = ["wasm-bindgen", "serde-wasm-bindgen", "console_error_panic_hook",
        "once_cell", "web-sys", "rhai", "sha2", "serde_json"]
```

### C. Build Command

```bash
wasm-pack build \
  --target web \
  --out-dir "$OUTPUT_DIR" \
  --release \
  -- --features wasm
```

---

**Report Status**: COMPLETE
**Next Task**: 1.2 - Fix WASM build configuration based on findings
