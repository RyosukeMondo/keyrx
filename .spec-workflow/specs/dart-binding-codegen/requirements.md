# Requirements Document

## Introduction

The Dart Binding Code Generation feature automatically generates type-safe Dart FFI bindings from JSON contracts, eliminating manual synchronization between Rust exports and Dart imports. This ensures that the Dart UI always has correct function signatures and type definitions that match the Rust core.

**Current Problem**: Developers manually write Dart FFI bindings in `ui/lib/ffi/bindings.dart`, duplicating type information and function signatures. When Rust FFI changes, Dart bindings must be manually updated, leading to potential mismatches and runtime errors.

**Solution**: Create a code generator tool that reads JSON contracts and produces Dart FFI bindings with proper type mappings, null safety, and error handling.

## Alignment with Product Vision

This feature completes the contract-driven development vision by establishing JSON contracts as the single source of truth for both Rust and Dart. It supports KeyRx's "Performance > Features" principle by enabling compile-time verification of the entire FFI boundary, catching errors before they reach the user.

By automating binding generation, this feature enables rapid iteration on FFI interfaces without the maintenance burden of manual synchronization.

## Requirements

### Requirement 1: Generate Dart FFI Function Signatures

**User Story:** As a UI developer, I want Dart FFI bindings generated automatically, so that I don't have to manually keep them in sync with Rust.

#### Acceptance Criteria

1. WHEN the generator runs THEN it SHALL:
   - Read all `*.ffi-contract.json` files from the contracts directory
   - Generate a Dart file `lib/ffi/generated_bindings.dart` with all FFI function signatures
   - Use `dart:ffi` types for C interop
2. WHEN a contract specifies `function: "keyrx_config_save_profile"` THEN the generated signature SHALL be:
   ```dart
   late final _keyrx_config_save_profile _save_profile_ptr = _dylib
       .lookup<NativeFunction<_keyrx_config_save_profile_native>>('keyrx_config_save_profile')
       .asFunction();
   ```
3. WHEN parameter types are defined THEN they SHALL be mapped to appropriate Dart FFI types
4. WHEN the generator fails THEN it SHALL exit with a non-zero code and descriptive error message

### Requirement 2: Type Mapping from Contract to Dart

**User Story:** As a UI developer, I want automatic type conversion between Dart and FFI types, so that I can work with idiomatic Dart types.

#### Acceptance Criteria

1. WHEN the contract type is `string` THEN the generator SHALL:
   - Use `Pointer<Utf8>` for C strings
   - Provide helper methods for `String` ↔ `Pointer<Utf8>` conversion
2. WHEN the contract type is a custom struct THEN the generator SHALL:
   - Use `Pointer<Utf8>` for JSON-serialized data
   - Generate Dart classes with `fromJson` and `toJson` methods
3. WHEN the contract type is `int` THEN it SHALL map to Dart `int`
4. WHEN the contract type is `bool` THEN it SHALL map to Dart `bool`
5. WHEN the contract type is `void` THEN it SHALL map to Dart `void`

### Requirement 3: Error Handling Wrappers

**User Story:** As a UI developer, I want automatic error handling for FFI calls, so that I don't have to manually check error pointers.

#### Acceptance Criteria

1. WHEN an FFI function has an error pointer THEN the generator SHALL create a wrapper that:
   - Allocates the error pointer
   - Calls the native function
   - Checks if the error pointer is non-null
   - Throws a Dart exception with the error message if an error occurred
   - Frees the error pointer
2. WHEN no error occurs THEN the wrapper SHALL return the result normally
3. WHEN an FFI call panics THEN the Dart wrapper SHALL throw `FfiException` with the panic message
4. Example generated wrapper:
   ```dart
   HardwareProfile saveProfile(String profileId, HardwareProfile profile) {
     final errorPtr = calloc<Pointer<Utf8>>();
     final result = _save_profile_ptr(
       profileId.toNativeUtf8(),
       jsonEncode(profile).toNativeUtf8(),
       errorPtr
     );
     if (errorPtr.value.address != 0) {
       final error = errorPtr.value.toDartString();
       calloc.free(errorPtr);
       throw FfiException(error);
     }
     calloc.free(errorPtr);
     return HardwareProfile.fromJson(jsonDecode(result.toDartString()));
   }
   ```

### Requirement 4: Null Safety and Memory Management

**User Story:** As a UI developer, I want automatic memory management for FFI calls, so that I don't cause memory leaks or crashes.

#### Acceptance Criteria

1. WHEN allocating C strings THEN the wrapper SHALL use `calloc` from `package:ffi`
2. WHEN the FFI call completes THEN the wrapper SHALL free all allocated memory
3. WHEN a null pointer is returned THEN the wrapper SHALL handle it gracefully (return null or throw exception based on contract)
4. WHEN the contract specifies `nullable: true` THEN the Dart type SHALL be nullable (`String?`)
5. WHEN the contract specifies `nullable: false` THEN null results SHALL throw `FfiException("Unexpected null return")`

### Requirement 5: Type-Safe Dart Classes from Contracts

**User Story:** As a UI developer, I want Dart classes generated for custom types, so that I have type safety and auto-complete in the IDE.

#### Acceptance Criteria

1. WHEN a contract defines a custom type (e.g., `HardwareProfile`) THEN the generator SHALL:
   - Create a Dart class with fields matching the contract schema
   - Generate `fromJson` factory constructor
   - Generate `toJson` method
   - Use `@JsonSerializable` annotation for code generation compatibility
2. WHEN nested types exist THEN the generator SHALL recursively generate classes
3. WHEN optional fields exist THEN they SHALL be nullable Dart types
4. Example generated class:
   ```dart
   class HardwareProfile {
     final String id;
     final String name;
     final int vendorId;
     final int productId;

     HardwareProfile({
       required this.id,
       required this.name,
       required this.vendorId,
       required this.productId,
     });

     factory HardwareProfile.fromJson(Map<String, dynamic> json) => HardwareProfile(
       id: json['id'] as String,
       name: json['name'] as String,
       vendorId: json['vendor_id'] as int,
       productId: json['product_id'] as int,
     );

     Map<String, dynamic> toJson() => {
       'id': id,
       'name': name,
       'vendor_id': vendorId,
       'product_id': productId,
     };
   }
   ```

### Requirement 6: CLI Tool for Generation

**User Story:** As a developer, I want a CLI tool to regenerate Dart bindings, so that I can update them whenever contracts change.

#### Acceptance Criteria

1. WHEN I run `cargo run --bin generate-dart-bindings` THEN it SHALL:
   - Read all contracts from `core/contracts/*.ffi-contract.json`
   - Generate `ui/lib/ffi/generated_bindings.dart`
   - Generate `ui/lib/models/generated_models.dart` for custom types
   - Run `dart format` on generated files
2. WHEN generation succeeds THEN it SHALL exit with code 0 and print: "Dart bindings generated successfully"
3. WHEN generation fails THEN it SHALL exit with code 1 and print a descriptive error
4. WHEN contracts are unchanged THEN the tool SHALL skip regeneration (check file timestamps)
5. WHEN contracts change THEN the tool SHALL regenerate only affected files

### Requirement 7: Integration with Build Process

**User Story:** As a developer, I want Dart bindings to be regenerated automatically during the build, so that I never have stale bindings.

#### Acceptance Criteria

1. WHEN running `just build` THEN the Dart binding generator SHALL run before compiling Dart code
2. WHEN running `just dev` THEN the generator SHALL watch contract files for changes and regenerate automatically
3. WHEN CI runs THEN it SHALL verify that generated bindings are up-to-date (fail if regeneration would produce different output)
4. WHEN pre-commit hooks run THEN they SHALL verify bindings are current

## Non-Functional Requirements

### Code Architecture and Modularity

- **Single Responsibility**: Generator tool only handles Dart code generation; contract parsing is a separate module
- **Modular Design**: Separate modules for type mapping, code generation, and file writing
- **Clear Interfaces**: Define structs for `DartFunction`, `DartType`, and `DartClass`

### Performance

- Generation SHALL complete in under 3 seconds for all contracts
- Generated code SHALL have minimal runtime overhead
- Generated code SHALL be formatted and ready for version control

### Reliability

- Generator SHALL produce deterministic output (same input → same output)
- Generator SHALL never produce invalid Dart code
- Generated bindings SHALL pass `dart analyze` without warnings

### Usability

- Generated code SHALL include comments referencing the source contract
- Error messages SHALL indicate which contract file and function caused the issue
- Documentation SHALL include examples of using generated bindings

### Security

- Generated code SHALL validate all JSON deserialization
- Generated code SHALL handle malformed JSON gracefully (throw exception, not crash)
- Generated code SHALL never expose raw pointers to Flutter UI
