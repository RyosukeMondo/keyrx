# Design Document

## Overview

The Rust FFI Code Generation feature uses procedural macros to automatically generate `extern "C"` FFI wrapper functions from JSON contracts. Developers write clean Rust implementation functions, and the macro generates all FFI boilerplate including panic catching, type marshaling, error handling, and JSON serialization.

This eliminates 300+ lines of repetitive FFI code and makes contract violations compile-time errors instead of runtime crashes.

## Steering Document Alignment

### Technical Standards (tech.md)

- **Dependency Injection**: Generated code preserves DI patterns; implementations remain testable
- **Error Handling**: Generated wrappers catch panics and convert `Result<T, E>` to FFI error pointers
- **Performance**: Zero-cost abstractions; generated code is as fast as hand-written FFI
- **Rust Stability**: Uses stable Rust proc macros; no nightly features required

### Project Structure (structure.md)

- **New Crate**: `keyrx_ffi_macro/` - Procedural macro crate (required for proc macros)
- **Runtime Crate**: `keyrx_ffi_runtime/` - Runtime marshaling helpers shared between macro and core
- **Integration**: Core uses `#[keyrx_ffi]` macro to generate FFI exports
- **Naming**: Generated functions follow `keyrx_{domain}_{function}` convention

## Code Reuse Analysis

### Existing Components to Leverage

- **ContractRegistry**: Load and parse JSON contracts at compile time
- **FfiContract / FunctionContract**: Already models contract structure
- **TypeDefinition**: Contract type information for code generation
- **Error Handling**: Existing `FfiError` and `FfiResult` types

### Integration Points

- **FFI Exports**: Replace manual `extern "C"` functions in `core/src/ffi/exports.rs` and domain modules
- **Contracts**: Read from `core/src/ffi/contracts/*.ffi-contract.json`
- **Serialization**: Use existing `serde_json` for JSON marshaling

## Architecture

The code generation system follows a compile-time transformation pipeline:

```
Contract JSON → Macro Expansion → AST Generation → Rust Compilation
```

### Modular Design Principles

- **Separation**: Macro crate (`keyrx_ffi_macro`) and runtime crate (`keyrx_ffi_runtime`) are separate
- **Single Responsibility**: Macro generates code; runtime provides marshaling utilities
- **Composability**: Individual functions can use the macro independently

```mermaid
graph TD
    A[Developer writes impl] --> B[#[keyrx_ffi] macro]
    B --> C[Load contract.json]
    C --> D[Validate impl signature]
    D --> E[Generate extern C wrapper]
    E --> F[Insert panic handling]
    F --> G[Insert type marshaling]
    G --> H[Compile to binary]
```

## Components and Interfaces

### Component 1: Procedural Macro (`keyrx_ffi_macro`)
- **Purpose:** Attribute macro that generates FFI wrappers from contracts
- **Interfaces:**
  - `#[keyrx_ffi(domain = "config")]` attribute on `impl` blocks or functions
- **Dependencies:** `syn`, `quote`, `proc_macro2`, `serde_json`
- **Reuses:** Contract loading logic from `ContractRegistry`

**Macro Expansion Example:**
```rust
// Developer writes:
#[keyrx_ffi(domain = "config")]
impl ConfigFfi {
    fn save_hardware_profile(profile_json: String) -> Result<HardwareProfile, String> {
        // Implementation
    }
}

// Macro generates:
#[no_mangle]
pub unsafe extern "C" fn keyrx_config_save_hardware_profile(
    profile_json: *const c_char,
    error: *mut *mut c_char,
) -> *const c_char {
    keyrx_ffi_runtime::ffi_wrapper(error, || {
        let profile_json = keyrx_ffi_runtime::parse_c_string(profile_json, "profile_json")?;
        let result = ConfigFfi::save_hardware_profile(profile_json)?;
        keyrx_ffi_runtime::serialize_to_c_string(&result)
    })
}
```

### Component 2: Runtime Marshaling Library (`keyrx_ffi_runtime`)
- **Purpose:** Provide reusable FFI utilities for generated code
- **Interfaces:**
  - `ffi_wrapper<F>(error: *mut *mut c_char, f: F) -> ReturnType`
  - `parse_c_string(ptr: *const c_char, name: &str) -> Result<String>`
  - `serialize_to_c_string<T: Serialize>(value: &T) -> Result<*const c_char>`
  - `handle_panic<F>(f: F) -> Result<F::Output>`
- **Dependencies:** `serde_json`, `libc`
- **Reuses:** Existing FFI error handling patterns

**Key Functions:**
```rust
// Panic-catching wrapper
pub fn ffi_wrapper<F, T>(error: *mut *mut c_char, f: F) -> *const c_char
where
    F: FnOnce() -> Result<T, String> + std::panic::UnwindSafe,
    T: Serialize,
{
    match handle_panic(|| f()) {
        Ok(Ok(value)) => match serialize_to_c_string(&value) {
            Ok(ptr) => ptr,
            Err(e) => {
                set_error_pointer(error, &e);
                ptr::null()
            }
        },
        Ok(Err(e)) => {
            set_error_pointer(error, &e);
            ptr::null()
        }
        Err(panic_msg) => {
            set_error_pointer(error, &format!("Panic: {}", panic_msg));
            ptr::null()
        }
    }
}

// Parse C string safely
pub fn parse_c_string(ptr: *const c_char, param_name: &str) -> Result<String, String> {
    if ptr.is_null() {
        return Err(format!("Null pointer for parameter '{}'", param_name));
    }
    unsafe {
        CStr::from_ptr(ptr)
            .to_str()
            .map(|s| s.to_string())
            .map_err(|_| format!("Invalid UTF-8 in parameter '{}'", param_name))
    }
}
```

### Component 3: Contract Loader (Compile-time)
- **Purpose:** Load and parse contracts during macro expansion
- **Interfaces:**
  - `load_contract(domain: &str) -> Result<FfiContract, Error>`
- **Dependencies:** File I/O, `serde_json`
- **Reuses:** `ContractRegistry` parsing logic

**Implementation:**
```rust
fn load_contract_for_domain(domain: &str) -> Result<FfiContract, syn::Error> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .map_err(|_| syn::Error::new(Span::call_site(), "CARGO_MANIFEST_DIR not set"))?;

    let contract_path = PathBuf::from(manifest_dir)
        .join("src/ffi/contracts")
        .join(format!("{}.ffi-contract.json", domain));

    let content = std::fs::read_to_string(&contract_path)
        .map_err(|e| syn::Error::new(
            Span::call_site(),
            format!("Failed to load contract {}: {}", contract_path.display(), e)
        ))?;

    serde_json::from_str(&content)
        .map_err(|e| syn::Error::new(
            Span::call_site(),
            format!("Invalid contract JSON: {}", e)
        ))
}
```

### Component 4: Code Generator
- **Purpose:** Generate Rust AST for FFI wrapper functions
- **Interfaces:**
  - `generate_ffi_function(func_contract: &FunctionContract, impl_fn: &ImplItemFn) -> TokenStream`
- **Dependencies:** `quote`, `syn`
- **Reuses:** None (new functionality)

**Generation Strategy:**
1. Extract function name from contract
2. Map parameter types from contract to FFI types
3. Generate parameter parsing code
4. Generate implementation call
5. Generate return value serialization
6. Wrap everything in panic handler

**Code Template:**
```rust
#[no_mangle]
pub unsafe extern "C" fn {ffi_name}(
    {params},
    error: *mut *mut c_char,
) -> {return_type} {
    keyrx_ffi_runtime::ffi_wrapper(error, || {
        {parse_params}
        {call_impl}
        {serialize_result}
    })
}
```

### Component 5: Type Mapper
- **Purpose:** Convert contract types to Rust FFI types
- **Interfaces:**
  - `map_contract_type_to_ffi(contract_type: &str) -> TokenStream`
  - `generate_parser_for_type(contract_type: &str, param_name: &str) -> TokenStream`
  - `generate_serializer_for_type(contract_type: &str) -> TokenStream`
- **Dependencies:** `quote`
- **Reuses:** Type mapping rules from validation

**Type Mapping:**
| Contract Type | FFI Parameter | Parser | Serializer |
|--------------|---------------|--------|------------|
| `string` | `*const c_char` | `parse_c_string(ptr, name)?` | `string_to_c_char(s)` |
| `int` | `i32` | Direct pass | Direct pass |
| `bool` | `bool` | Direct pass | Direct pass |
| `void` | - | - | Return `()` |
| Custom struct | `*const c_char` | `serde_json::from_str(&parse_c_string(ptr, name)?)?` | `serialize_to_c_string(&value)?` |
| `Vec<T>` | `*const c_char` | `serde_json::from_str(&parse_c_string(ptr, name)?)?` | `serialize_to_c_string(&vec)?` |

## Data Models

### MacroContext
```rust
struct MacroContext {
    domain: String,
    contract: FfiContract,
    impl_items: Vec<ImplItemFn>,
}
```

### GeneratedFunction
```rust
struct GeneratedFunction {
    ffi_name: Ident,
    params: Vec<FnArg>,
    return_type: ReturnType,
    body: Block,
}
```

### TypeMapping
```rust
enum TypeMapping {
    Direct { rust_type: Type },
    CString { requires_parse: bool },
    Json { inner_type: Type },
}
```

## Error Handling

### Error Scenarios

1. **Contract File Not Found**
   - **Handling:** Compile error with clear message about missing contract
   - **User Impact:** `error: Contract file not found: core/src/ffi/contracts/config.ffi-contract.json`

2. **Invalid Contract JSON**
   - **Handling:** Compile error with JSON parse error
   - **User Impact:** `error: Invalid contract JSON: expected '}' at line 45`

3. **Implementation Function Missing**
   - **Handling:** Compile error indicating which contract function has no implementation
   - **User Impact:** `error: No implementation found for contract function 'save_profile'`

4. **Type Mismatch Between Contract and Impl**
   - **Handling:** Compile error with type mismatch details
   - **User Impact:** `error: Implementation returns 'Result<Profile, String>' but contract expects 'void'`

5. **Runtime Panic in Implementation**
   - **Handling:** Catch panic, set error pointer, return null/error code
   - **User Impact:** Dart receives `FfiException: Panic: index out of bounds`

6. **Serialization Error**
   - **Handling:** Set error pointer with serialization error message
   - **User Impact:** Dart receives `FfiException: Failed to serialize result`

## Testing Strategy

### Unit Testing

- **Macro Expansion Tests**: Use `trybuild` to test macro expansion
  - Input: Rust code with `#[keyrx_ffi]`
  - Output: Verify generated code compiles
  - Cases: Valid cases, error cases (missing contract, type mismatches)

- **Runtime Helper Tests**: Test marshaling functions
  - `parse_c_string`: Test null handling, valid strings, invalid UTF-8
  - `serialize_to_c_string`: Test serialization of various types
  - `ffi_wrapper`: Test panic catching, error handling

### Integration Testing

- **End-to-End Generation**: Test full macro expansion with real contracts
  - Create sample contract
  - Write implementation
  - Verify generated FFI function works correctly
  - Call from "FFI" side and verify results

- **Error Propagation**: Test that errors flow correctly
  - Implementation returns `Err`
  - Verify error pointer is set
  - Verify null is returned

### End-to-End Testing

- **Dart Integration**: Test generated FFI from Flutter
  - Call generated FFI functions from Dart
  - Verify results are correct
  - Test error scenarios

## Implementation Phases

### Phase 1: Runtime Library
1. Create `keyrx_ffi_runtime` crate
2. Implement marshaling utilities
3. Implement panic handling
4. Unit test all runtime functions

### Phase 2: Basic Macro
1. Create `keyrx_ffi_macro` crate
2. Implement attribute parsing (`domain` parameter)
3. Load contract files
4. Validate contract exists

### Phase 3: Code Generation
1. Implement type mapping
2. Generate parameter parsing
3. Generate function calls
4. Generate return serialization

### Phase 4: Advanced Features
1. Handle `void` returns
2. Handle optional parameters
3. Handle various error types
4. Support custom serialization

### Phase 5: Testing & Documentation
1. Add `trybuild` tests
2. Test with real contracts
3. Document macro usage
4. Migration guide from manual FFI

## Migration Strategy

### Step-by-Step Migration

1. **Keep Existing FFI Functions**: Don't break existing code
2. **Add Macro to One Domain**: Start with `config` domain
3. **Test Thoroughly**: Ensure generated code works identically
4. **Deprecate Manual Functions**: Mark old functions as deprecated
5. **Migrate Remaining Domains**: Apply macro to other domains
6. **Remove Manual Code**: Delete old FFI boilerplate

### Compatibility

- Generated functions have same signatures as manual functions
- No breaking changes to Dart bindings
- Can run both side-by-side during migration
