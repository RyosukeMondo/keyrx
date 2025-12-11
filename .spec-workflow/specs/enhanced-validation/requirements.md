# Requirements Document

## Introduction

The Enhanced Validation feature upgrades the existing `contract_adherence` test from a simple existence check to comprehensive static analysis that verifies FFI contract correctness. This eliminates runtime FFI errors by ensuring that Rust implementations match their JSON contract definitions at compile time.

**Current Problem**: The existing `tests/contract_adherence.rs` only checks if functions exist but doesn't verify that their signatures match the contract. This allows mismatches like returning `void` when `HardwareProfile` is expected, leading to runtime crashes.

**Solution**: Parse Rust AST using `syn` to extract actual function signatures and compare them against the JSON contract, failing the build if mismatches are detected.

## Alignment with Product Vision

This feature directly supports KeyRx's "Safety First" principle by preventing FFI crashes before they reach production. It aligns with the CLI-first development philosophy by providing compile-time guarantees without requiring manual testing. This eliminates an entire class of bugs that could undermine user trust in the remapping engine.

By establishing the JSON Contract as the Single Source of Truth (SSOT), this feature lays the foundation for future code generation (Phase 3), making KeyRx's FFI layer more maintainable and reliable.

## Requirements

### Requirement 1: Parse Rust Function Signatures

**User Story:** As a developer, I want the build system to automatically parse Rust FFI function signatures, so that type mismatches are detected without manual inspection.

#### Acceptance Criteria

1. WHEN the test runs THEN it SHALL use `syn` to parse `core/src/ffi/exports.rs`
2. WHEN parsing completes THEN it SHALL extract all `extern "C"` function signatures including:
   - Function name
   - Parameter names and types
   - Return type
   - Whether parameters are pointers (`*const`, `*mut`)
3. IF parsing fails (invalid Rust syntax) THEN the test SHALL fail with a clear error message indicating the file and line number

### Requirement 2: Validate Function Signatures Against Contracts

**User Story:** As a developer, I want function signatures to be validated against their JSON contracts, so that I'm immediately notified when they drift out of sync.

#### Acceptance Criteria

1. WHEN validation runs THEN it SHALL load all `*.ffi-contract.json` files from the contracts directory
2. FOR EACH function in the contract THEN it SHALL:
   - Verify the function exists in `exports.rs`
   - Verify parameter count matches
   - Verify parameter types match (with FFI type mappings)
   - Verify return type matches the expected contract type
3. WHEN a function is missing THEN the test SHALL fail with message: "Contract function `{name}` not found in exports.rs"
4. WHEN parameter count mismatches THEN the test SHALL fail with: "Function `{name}` expects {expected} parameters, found {actual}"
5. WHEN parameter type mismatches THEN the test SHALL fail with: "Function `{name}` parameter `{param}` expects type `{expected}`, found `{actual}`"
6. WHEN return type mismatches THEN the test SHALL fail with: "Function `{name}` expects return type `{expected}`, found `{actual}`"

### Requirement 3: FFI Type Mapping Rules

**User Story:** As a developer, I want clear rules for how Rust FFI types map to contract types, so that validation is consistent and predictable.

#### Acceptance Criteria

1. WHEN validating types THEN it SHALL apply these mappings:
   - Contract `string` → Rust `*const c_char`
   - Contract `int` → Rust `i32`
   - Contract `bool` → Rust `bool`
   - Contract `{CustomType}` → Rust `*const c_char` (JSON serialized)
   - Contract `void` → Rust `()`
   - Contract result wrapper → Rust `*mut *mut c_char` (error out parameter)
2. WHEN a contract type has no mapping THEN the test SHALL fail with: "Unknown contract type `{type}` in function `{name}`"
3. WHEN a custom type is expected THEN it SHALL accept JSON string pointers (`*const c_char`)

### Requirement 4: Comprehensive Error Reporting

**User Story:** As a developer, I want detailed error messages when validation fails, so that I can quickly identify and fix the issue.

#### Acceptance Criteria

1. WHEN any validation fails THEN the test SHALL output:
   - Which contract file contained the violation
   - Which function failed validation
   - What was expected vs. what was found
   - The file and line number of the Rust function
2. WHEN multiple violations exist THEN it SHALL report ALL violations, not just the first one
3. WHEN validation succeeds THEN it SHALL output: "All {count} FFI functions validated successfully against contracts"

### Requirement 5: Contract Completeness Check

**User Story:** As a developer, I want to ensure all FFI functions have contracts, so that no function is left undocumented.

#### Acceptance Criteria

1. WHEN validation runs THEN it SHALL identify all `extern "C"` functions in `exports.rs`
2. FOR EACH FFI function THEN it SHALL verify a corresponding contract entry exists
3. WHEN a function has no contract THEN the test SHALL warn: "Function `{name}` in exports.rs has no contract definition"
4. IF more than 10% of functions are missing contracts THEN the test SHALL fail

## Non-Functional Requirements

### Code Architecture and Modularity

- **Single Responsibility**: `contract_adherence.rs` handles only validation logic; parsing and type mapping are separate modules
- **Modular Design**: Use separate functions for parsing, loading contracts, validating, and reporting
- **Clear Interfaces**: Define structs for `ParsedFunction`, `ContractFunction`, and `ValidationError`

### Performance

- Validation SHALL complete in under 5 seconds on a typical development machine
- Parsing SHALL cache AST results to avoid re-parsing unchanged files

### Reliability

- Validation SHALL never produce false positives (reporting an error when the contract is correct)
- Validation SHALL never miss genuine mismatches (false negatives)
- The test SHALL be deterministic and produce the same results on all platforms

### Usability

- Error messages SHALL be actionable and include file locations
- The test SHALL integrate seamlessly with `cargo test` and CI pipelines
- Documentation SHALL include examples of common contract errors and how to fix them
