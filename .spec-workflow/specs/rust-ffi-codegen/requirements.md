# Requirements Document

## Introduction

The Rust FFI Code Generation feature eliminates manual FFI boilerplate by automatically generating `extern "C"` functions from JSON contracts using procedural macros. This makes it impossible to have missing functions or type mismatches, as the contract becomes the single source of truth for all FFI exports.

**Current Problem**: Developers manually write FFI wrapper functions in `exports.rs`, handling JSON serialization, panic catching, and error pointer management. This is error-prone and leads to drift between contracts and implementations.

**Solution**: Implement a `#[keyrx_ffi]` procedural macro that reads JSON contracts and generates FFI wrappers automatically. Developers only write the core implementation logic; the macro handles all FFI concerns.

## Alignment with Product Vision

This feature embodies the "Logic > Configuration" principle by treating contracts as executable specifications. It supports the "Safety First" pillar by making FFI errors impossible through compile-time guarantees. By eliminating 300+ lines of repetitive FFI boilerplate, it allows developers to focus on core remapping logic rather than FFI plumbing.

This is the culmination of the contract-driven development plan, enabling KeyRx to scale FFI operations without increasing maintenance burden.

## Requirements

### Requirement 1: Procedural Macro for FFI Generation

**User Story:** As a developer, I want to annotate my implementation with `#[keyrx_ffi]`, so that FFI wrappers are generated automatically without manual boilerplate.

#### Acceptance Criteria

1. WHEN I annotate a module with `#[keyrx_ffi(domain = "config")]` THEN it SHALL:
   - Load `config.ffi-contract.json` at compile time
   - Generate `extern "C"` functions for all contract entries
   - Link generated functions to the annotated implementation
2. WHEN the contract file is missing THEN compilation SHALL fail with: "Contract file not found: {path}"
3. WHEN the contract JSON is invalid THEN compilation SHALL fail with: "Invalid contract JSON: {error}"
4. WHEN the implementation function is missing THEN compilation SHALL fail with: "No implementation found for contract function `{name}`"

### Requirement 2: Automatic Type Marshaling

**User Story:** As a developer, I want automatic conversion between Rust types and FFI types, so that I can work with safe Rust types in my implementation.

#### Acceptance Criteria

1. WHEN a contract parameter is `string` THEN the macro SHALL:
   - Accept `*const c_char` in the FFI signature
   - Convert to `String` or `&str` for the implementation
   - Handle null pointers safely
2. WHEN a contract return type is a custom struct THEN the macro SHALL:
   - Serialize the result to JSON
   - Allocate a C string
   - Return `*const c_char`
3. WHEN marshaling fails (invalid UTF-8, serialization error) THEN it SHALL populate the error pointer with a descriptive message
4. IF the contract specifies `nullable: true` THEN the macro SHALL handle `null` inputs gracefully

### Requirement 3: Panic and Error Handling

**User Story:** As a developer, I want automatic panic catching in FFI boundaries, so that Rust panics don't crash the Flutter application.

#### Acceptance Criteria

1. WHEN the implementation panics THEN the generated FFI wrapper SHALL:
   - Catch the panic using `catch_unwind`
   - Populate the error pointer with: "Panic: {message}"
   - Return a safe default value (null pointer or error code)
2. WHEN the implementation returns `Result<T, E>` THEN the macro SHALL:
   - On `Ok(value)`: Return the marshaled value, set error pointer to null
   - On `Err(e)`: Populate error pointer with `e.to_string()`, return null/error code
3. WHEN the error pointer is null THEN errors SHALL be logged but not dereferenced

### Requirement 4: Contract-Driven Signatures

**User Story:** As a developer, I want FFI function signatures to exactly match the contract, so that validation passes automatically.

#### Acceptance Criteria

1. WHEN the contract specifies `parameters: [{name: "profile_id", type: "string"}]` THEN the FFI signature SHALL be:
   ```rust
   extern "C" fn keyrx_domain_function(profile_id: *const c_char, error: *mut *mut c_char) -> *const c_char
   ```
2. WHEN the contract specifies `returns: "void"` THEN the FFI signature SHALL return `()` and omit JSON serialization
3. WHEN parameter order changes in the contract THEN the generated signature SHALL update automatically
4. WHEN the implementation signature doesn't match the contract THEN compilation SHALL fail with a type mismatch error

### Requirement 5: Developer-Friendly Implementation Interface

**User Story:** As a developer, I want to write clean Rust implementations without FFI concerns, so that my code is maintainable and testable.

#### Acceptance Criteria

1. WHEN implementing a contract function THEN I SHALL write:
   ```rust
   #[keyrx_ffi(domain = "config")]
   impl ConfigFfi {
       fn save_profile(profile_id: String, profile: HardwareProfile) -> Result<HardwareProfile, String> {
           // Pure Rust implementation
       }
   }
   ```
2. WHEN the macro expands THEN it SHALL generate:
   ```rust
   #[no_mangle]
   pub unsafe extern "C" fn keyrx_config_save_profile(
       profile_id: *const c_char,
       profile: *const c_char,
       error: *mut *mut c_char
   ) -> *const c_char {
       // Generated marshaling code
   }
   ```
3. WHEN testing my implementation THEN I SHALL call the Rust function directly without FFI layer
4. WHEN the contract updates THEN I SHALL update my implementation signature to match, and the macro will regenerate the FFI wrapper

### Requirement 6: Code Generation Observability

**User Story:** As a developer, I want to see the generated FFI code, so that I can debug issues and understand what the macro produces.

#### Acceptance Criteria

1. WHEN compiling with `RUSTFLAGS="--cfg keyrx_ffi_debug"` THEN the macro SHALL emit the generated code to `target/keyrx_ffi_generated.rs`
2. WHEN a generation error occurs THEN the macro SHALL provide a span pointing to the annotated implementation
3. WHEN the contract changes THEN recompilation SHALL regenerate the FFI code (no stale generated code)

## Non-Functional Requirements

### Code Architecture and Modularity

- **Separation of Concerns**: Procedural macro in `keyrx_ffi_macro` crate, runtime helpers in `keyrx_ffi_runtime` crate
- **Single Responsibility**: Macro handles code generation only; runtime crate handles marshaling and error handling
- **Testability**: Generated code SHALL be deterministic and testable via `trybuild` tests

### Performance

- Macro expansion SHALL complete in under 1 second per domain
- Generated code SHALL have zero runtime overhead compared to hand-written FFI
- JSON serialization SHALL use `serde_json` with optimizations enabled

### Reliability

- Generated code SHALL never introduce memory leaks (all allocations properly freed)
- Generated code SHALL be memory-safe (no undefined behavior)
- Panic handling SHALL never cause undefined behavior or crashes

### Usability

- Macro errors SHALL provide clear guidance on how to fix implementation issues
- Documentation SHALL include migration guide from manual FFI to macro-based FFI
- Examples SHALL cover common patterns (Result types, optional parameters, custom types)

### Security

- Generated code SHALL validate all input pointers before dereferencing
- Null checks SHALL be performed on all FFI parameters
- Buffer overflows SHALL be impossible (safe conversions only)
