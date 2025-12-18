# Tasks Document

## 1. Core Generator Infrastructure

- [x] 1. Create `ipc_client_gen.rs` module structure
  - File: `core/tools/generate_dart_bindings/src/ipc_client_gen.rs`
  - Create the new module file and register it in `lib.rs`
  - Implement basic structs for holding generation state
  - _Leverage: core/tools/generate_dart_bindings/src/bindings_gen.rs_
  - _Requirements: 1.1, Requirement 1 (Non-Functional)_
  - _Prompt: Role: Rust Developer specializing in code generation tools | Task: Create the `ipc_client_gen.rs` module structure and register it in `lib.rs`, implementing basic structs for generation state following requirement 1.1 and non-functional requirement 1, leveraging patterns from `bindings_gen.rs` | Restrictions: Follow existing module patterns, ensure clean separation of concerns, no logic implementation yet | Success: Module compiles, struct definitions match the needs of the generator, registered in lib.rs_

- [x] 2. Implement `IpcClientGenerator` class generation logic
  - File: `core/tools/generate_dart_bindings/src/ipc_client_gen.rs`
  - Implement function to generate the `DaemonClient` class shell
  - Add imports (dart:async, dart:convert, models.dart)
  - _Leverage: core/tools/generate_dart_bindings/src/templates.rs_
  - _Requirements: 3.1, 2.1_
  - _Prompt: Role: Rust Developer with expertise in template-based code generation | Task: Implement the `DaemonClient` class shell generation logic in `ipc_client_gen.rs`, adding necessary imports including models, following requirements 3.1 and 2.1 and leveraging `templates.rs` | Restrictions: Must use existing template system, ensure correct imports for Dart, class must be extensible | Success: Generates valid Dart class code with correct imports and constructor_

## 2. Method and Event Generation

- [x] 3. Implement IPC Method generation
  - File: `core/tools/generate_dart_bindings/src/ipc_client_gen.rs`
  - Map `FfiContract` functions to Dart `Future<T>` methods
  - Generate code to serialize args -> `sendRequest` -> deserialize result
  - Handle `void` returns and custom model returns
  - _Leverage: core/tools/generate_dart_bindings/src/models_gen.rs (for type mapping)_
  - _Requirements: 1.2, 3.2, 3.3, 3.4_
  - _Prompt: Role: Rust Developer specializing in FFI/IPC code generation | Task: Implement IPC method generation logic mapping `FfiContract` functions to Dart `Future<T>` methods, handling serialization/deserialization and return types following requirements 1.2, 3.2, 3.3, and 3.4, leveraging `models_gen.rs` type mapping | Restrictions: Must correctly map all supported FFI types to JSON-compatible Dart types, ensure type safety in generated code | Success: Generates correct Dart methods that call `sendRequest` with properly serialized parameters and return types_

- [x] 4. Implement IPC Event generation
  - File: `core/tools/generate_dart_bindings/src/ipc_client_gen.rs`
  - Map `FfiContract` events to Dart `Stream<T>` getters
  - Generate code to expose stream from `DaemonService`
  - _Requirements: 1.3, 4.1, 4.2_
  - _Prompt: Role: Rust Developer with expertise in event-driven systems and code generation | Task: Implement IPC event generation logic mapping `FfiContract` events to Dart `Stream<T>` getters following requirements 1.3, 4.1, and 4.2 | Restrictions: Must ensure streams are properly typed, reuse model classes for event payloads | Success: Generates correct Dart getters returning typed Streams for all contract events_

## 3. Integration & Tooling

- [x] 5. Update `pipeline.rs` to include IPC generation
  - File: `core/tools/generate_dart_bindings/src/pipeline.rs`
  - Update `generate_code` to call `ipc_client_gen::generate`
  - Add `generate_ipc_client` function to pipeline
  - Write output to `ui/lib/services/generated/ipc_client.dart`
  - _Leverage: core/tools/generate_dart_bindings/src/writer.rs_
  - _Requirements: 1.1, 1.4_
  - _Prompt: Role: Rust Developer specializing in CLI tool architecture | Task: Update `pipeline.rs` to integrate `ipc_client_gen`, adding `generate_ipc_client` function and writing output to `ui/lib/services/generated/ipc_client.dart` following requirements 1.1 and 1.4 | Restrictions: Maintain backward compatibility with FFI generation, ensure clean error handling during file writing | Success: Pipeline successfully invokes IPC generator and writes the output file to the correct location_

- [x] 6. Update `DaemonService` to support generated clients
  - File: `ui/lib/services/daemon_service.dart`
  - Ensure `sendRequest` generic signature matches generated code expectations
  - Expose raw `notification` stream if not already available
  - _Requirements: Requirement 3 (Design)_
  - _Prompt: Role: Dart/Flutter Developer with expertise in asynchronous programming and services | Task: Refactor `DaemonService` to support generated clients, ensuring `sendRequest` signature matches expectations and exposing raw notification stream following Design Requirement 3 | Restrictions: Do not break existing manual usages yet, ensure backward compatibility | Success: DaemonService provides the necessary public API for the generated client to function_

## 4. Validation

- [x] 7. Verify with `consistency_check`
  - Run the generator
  - Verify `ui/lib/services/generated/ipc_client.dart` exists and is valid Dart
  - Run `dart analyze`
  - _Requirements: Non-Functional (Clean Output)_
  - _Prompt: Role: QA Engineer / Tooling Specialist | Task: Verify the full generation pipeline by running the tool, checking for file existence, and running `dart analyze` on the output following Non-Functional requirements | Restrictions: Must pass without any analyzer errors or warnings | Success: Generator runs without error, Dart code is valid and type-safe_
