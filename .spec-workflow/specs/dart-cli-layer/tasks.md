# Tasks Document

- [x] 1. Create core_logic package
  - File: packages/core_logic/pubspec.yaml
  - Create a new Dart package using `dart create -t package`.
  - Add dependencies: `ffi`, `freezed_annotation`, `json_annotation`.
  - Purpose: Establish the shared library.
  - _Requirements: 1, Non-Functional (Package Separation)_
  - _Prompt: Role: Dart Developer | Task: Create the 'core_logic' package structure and pubspec.yaml following Requirement 1. Include dependencies for ffi, freezed_annotation, and json_annotation. | Restrictions: Do not include flutter dependencies. | Success: Package created and `dart pub get` succeeds._

- [x] 2. Move Models to core_logic
  - File: packages/core_logic/lib/models/
  - Move model files from `ui/lib/models/` to `packages/core_logic/lib/models/`.
  - Update imports within the files to be relative or package-absolute.
  - Purpose: Share data structures.
  - _Leverage: ui/lib/models/_
  - _Requirements: 4_
  - _Prompt: Role: Refactoring Specialist | Task: Move data models from `ui/lib/models` to `packages/core_logic/lib/models`. Update imports to be self-contained within the package. | Restrictions: Ensure no Flutter dependencies remain in the models. | Success: Models compile in the new package._

- [ ] 3. Move FFI Bridge to core_logic
  - File: packages/core_logic/lib/ffi/
  - Move FFI binding code from `ui/lib/ffi/` to `packages/core_logic/lib/ffi/`.
  - Update paths to load the dynamic library (might need logic to find it relative to script vs app).
  - Purpose: Share access to Rust core.
  - _Leverage: ui/lib/ffi/_
  - _Requirements: 1_
  - _Prompt: Role: Systems Programmer (Dart/Rust) | Task: Migrate FFI bridge code to `packages/core_logic/lib/ffi`. Implement robust library loading logic that works for both Flutter (AOT/JIT) and Dart CLI (JIT/exe). | Restrictions: Handle different platform paths for the dylib. | Success: FFI initializes correctly in a standalone Dart script._

- [ ] 4. Refactor UI to use core_logic
  - File: ui/pubspec.yaml, ui/lib/**/*
  - Add `core_logic` as a path dependency in `ui`.
  - Update UI imports to use `package:core_logic/...`.
  - Purpose: Restore UI functionality.
  - _Requirements: Non-Functional (Package Separation)_
  - _Prompt: Role: Flutter Developer | Task: Update `ui/pubspec.yaml` to depend on `packages/core_logic`. mass-replace imports in `ui/lib` to point to the new package. | Restrictions: Verify the app still compiles. | Success: `flutter build linux` succeeds._

- [ ] 5. Create CLI Tool Structure
  - File: apps/cli_tool/pubspec.yaml, apps/cli_tool/bin/keyrx_cli.dart
  - Create `apps/cli_tool` package.
  - specificy dependency on `core_logic`.
  - Implement basic argument parsing.
  - Purpose: The harness entry point.
  - _Requirements: 2_
  - _Prompt: Role: Tooling Engineer | Task: Scaffold the `apps/cli_tool` package and a main entry point that parses arguments. Depend on `core_logic`. | Restrictions: Keep it lightweight. | Success: `dart run bin/keyrx_cli.dart --help` runs._

- [ ] 6. Implement JSON Command Processor
  - File: apps/cli_tool/lib/command_processor.dart
  - Implement logic to decode JSON, map to `core_logic` calls, and encode responses.
  - Purpose: The "Brain" of the harness.
  - _Requirements: 2, 4_
  - _Prompt: Role: Backend Developer | Task: Implement the JSON command processing loop. Map at least one command (e.g., 'status' or 'echo') to prove the concept. | Restrictions: Handle JSON errors gracefully. | Success: Inputting `{"command": "echo", "payload": "test"}` returns valid JSON._

- [ ] 7. Create End-to-End Test Script
  - File: scripts/e2e_test_headless.sh
  - Create a shell script that compiles the CLI and runs a sequence of JSON tests.
  - Purpose: Verify the "Headless Harness" strategy.
  - _Requirements: 3_
  - _Prompt: Role: QA Automation Engineer | Task: Write a shell script `scripts/e2e_test_headless.sh` that builds the Rust core, runs the Dart CLI, and feeds it a test scenario. | Restrictions: Fail if output doesn't match expected JSON. | Success: The script runs and passes._
