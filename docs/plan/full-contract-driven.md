# Full Contract-Driven Development Plan

**Goal**: Eliminate runtime FFI errors (missing functions, type mismatches) by making the JSON Contract the Single Source of Truth (SSOT).

## 1. Current State & Problems
- **Partial Coverage**: Only `discovery` and `engine` domains have contracts. `config`, `recording`, etc., are "wild west".
- **Loose Coupling**: Contracts exist but are not strictly enforced. We have a test (`contract_adherence`) that checks for *existence*, but not *correctness* (signatures/types).
- **Manual Sync**: Developers must manually keep Dart bindings, Rust exports, and Contracts in sync. Mismatches lead to runtime crashes or logic bugs (e.g., `void` vs `HardwareProfile` return).

## 2. Strategy

### Phase 1: Complete the Schema (Immediate)
Define contracts for all remaining domains to establish the "Law".
- [x] Create `config.ffi-contract.json` (Hardware/Virtual profiles, Keymaps)
- [x] Create `recording.ffi-contract.json` (Session recording)
- [x] Create `runtime.ffi-contract.json` (Lifecycle)

### Phase 2: Enhanced Validation (Medium Term)
Upgrade `tests/contract_adherence.rs` from a simple "grep" to true static analysis.
- **Parse Rust AST**: Use `syn` to parse the actual Rust function signatures.
- **Type Checking**: Verify that `keyrx_config_save_profile` actually returns `FfiResult<HardwareProfile>` if the contract says `returns: HardwareProfile`.
- **Fail Build**: Determine breaking changes at compile time, not runtime.

### Phase 3: Code Generation (Long Term / Ideal)
Stop writing FFI boilerplate manually.
- **Rust Macros**: Create a `#[keyrx_domain("config")]` macro that reads the JSON contract and *generates* the `extern "C"` export functions automatically.
    - Developer only writes the implementation logic (e.g., `fn save_profile(...) -> Result<Profile>`).
    - Macro handles JSON serialization/deserialization, panic catching, and error pointer wrapping.
- **Dart Gen**: Extend the Dart binding generator to read exactly these contract files.

## 3. Workflow Example (Target State)

1. **Define**: Developer adds `save_profile` to `config.ffi-contract.json`.
2. **Build**:
    - Cargo runs. Macro sees new function in contract.
    - Complains: "Missing implementation for `save_profile`".
3. **Implement**:
    - Developer writes `impl ConfigFfi { fn save_profile(...) { ... } }`.
    - Macro generates the `unsafe extern "C" fn keyrx_config_save_profile(...)`.
4. **Result**:
    - Impossible to have "missing function" (compile error).
    - Impossible to return `void` when `Profile` is expected (type mismatch in macro).
    - Dart bindings are strictly typed matching the schema.

## 4. Next Steps
1. Create `config.ffi-contract.json` to lock in the current API.
2. Refactor `exports.rs` to use a trait-based approach that is easier to validate.
