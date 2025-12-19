# Tasks Document

- [x] 1. Create DeviceConfig model
  - File: packages/core_logic/lib/models/device_config.dart
  - Define `DeviceConfig` using Freezed.
  - Purpose: Data structure for persistence.
  - _Leverage: freezed_
  - _Requirements: 1_
  - _Prompt: Role: Dart Developer | Task: Create `DeviceConfig` model in `packages/core_logic` with a list of `DeviceIdentity`. Use Freezed. | Restrictions: None. | Success: Model generates without error._

- [x] 2. Implement DeviceConfigRepository
  - File: packages/core_logic/lib/repositories/device_config_repository.dart
  - Implement JSON file reading/writing using `dart:io`.
  - Handle platform-specific config paths (XDG on Linux).
  - Purpose: Persistence layer.
  - _Requirements: 1, Non-Functional (Modularity)_
  - _Prompt: Role: Backend Developer | Task: Implement `DeviceConfigRepository`. Use `path_provider` (or manual env var check if package not available in pure dart) to find `~/.config/keyrx`. Implement load/save with atomic write pattern. | Restrictions: Must work in pure Dart (no Flutter deps). | Success: Repository can save and load config from disk._

- [x] 3. Create Shared DeviceRegistryService
  - File: packages/core_logic/lib/services/device_registry_service.dart
  - Port logic from `ui/.../device_registry_service.dart`.
  - Replace `SharedPreferences` with `DeviceConfigRepository`.
  - Purpose: Unified logic.
  - _Leverage: ui/lib/services/device_registry_service.dart_
  - _Requirements: 2_
  - _Prompt: Role: Refactoring Specialist | Task: Port `DeviceRegistryService` to `packages/core_logic`. Inject `KeyrxBridge` and `DeviceConfigRepository`. Implement merging logic. | Restrictions: No Flutter imports. | Success: Service compiles and passes unit tests._

- [x] 4. Update CLI to use new Service
  - File: apps/cli_tool/lib/command_processor.dart
  - Inject `DeviceRegistryService` into the command processor.
  - Update `listDevices` to use the service instead of raw Bridge.
  - Add commands: `addVirtual`, `removeVirtual`, `setLabel`.
  - Purpose: CLI capabilities.
  - _Requirements: 3_
  - _Prompt: Role: Tooling Engineer | Task: Update CLI `CommandProcessor` to use `DeviceRegistryService`. Add support for `addVirtual`, `removeVirtual`, `setLabel` commands. | Restrictions: None. | Success: CLI commands modify the JSON config file._

- [x] 5. Refactor UI to use Shared Service
  - File: ui/lib/services/device_registry_service.dart
  - Delete old implementation.
  - Re-implement as a wrapper/provider around the `core_logic` service.
  - Purpose: UI consistency.
  - _Requirements: 4_
  - _Prompt: Role: Flutter Developer | Task: Update `ui/lib/services/device_registry_service.dart` to simply export or provide the `core_logic` service. Update Riverpod providers to construct the shared service with the repository. | Restrictions: Ensure UI still compiles. | Success: UI displays devices from the JSON config._

- [x] 6. Verify Consistency
  - File: scripts/verify_registry_consistency.sh
  - Script that adds a device via CLI and asserts existence in the config file.
  - Purpose: Verification.
  - _Requirements: 3_
  - _Prompt: Role: QA Engineer | Task: Create a verification script. | Restrictions: None. | Success: Script passes._
