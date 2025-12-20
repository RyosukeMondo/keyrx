# Tasks: Move Profile & Test Services to Core

- [x] 1. Move ProfileRegistryService to core_logic
  - File: packages/core_logic/lib/services/profile_registry_service.dart
  - Copy from ui/lib/services/profile_registry_service.dart
  - Update imports to use relative paths
  - Add export to core_logic.dart
  - Verify zero Flutter dependencies
  - Purpose: Enable CLI profile management
  - _Leverage: packages/core_logic/lib/services/runtime_service.dart (move pattern from Spec 1)_
  - _Requirements: REQ-1_
  - _Prompt: Role: Package organizer | Task: Move ProfileRegistryService from UI to core_logic following same pattern as RuntimeService move in Spec 1, update imports to relative paths, add export to core_logic.dart | Restrictions: Do NOT modify logic, do NOT delete original yet, verify NO Flutter dependencies using grep | Success: File at packages/core_logic/lib/services/profile_registry_service.dart, imports use relative paths, exported from core_logic.dart, zero Flutter dependencies confirmed with grep_

- [x] 2. Move TestService to core_logic
  - File: packages/core_logic/lib/services/test_service.dart
  - Copy from ui/lib/services/test_service.dart
  - Update imports to use relative paths
  - Add export to core_logic.dart
  - Verify zero Flutter dependencies
  - Purpose: Enable CLI testing capabilities
  - _Leverage: packages/core_logic/lib/services/profile_registry_service.dart (Task 1 pattern)_
  - _Requirements: REQ-2_
  - _Prompt: Role: Package organizer | Task: Move TestService from UI to core_logic following same pattern as ProfileRegistryService in Task 1, update imports to relative paths, add export to core_logic.dart | Restrictions: Do NOT modify logic, do NOT delete original yet, verify NO Flutter dependencies using grep | Success: File at packages/core_logic/lib/services/test_service.dart, imports use relative paths, exported from core_logic.dart, zero Flutter dependencies confirmed_

- [x] 3. Move ScriptFileService to core_logic
  - File: packages/core_logic/lib/services/script_file_service.dart
  - Copy from ui/lib/services/script_file_service.dart
  - Update imports to use relative paths
  - Add export to core_logic.dart
  - Verify zero Flutter dependencies
  - Purpose: Enable CLI script file operations
  - _Leverage: packages/core_logic/lib/services/test_service.dart (Task 2 pattern)_
  - _Requirements: REQ-3_
  - _Prompt: Role: Package organizer | Task: Move ScriptFileService from UI to core_logic following same pattern as TestService in Task 2, update imports to relative paths, add export to core_logic.dart | Restrictions: Do NOT modify logic, do NOT delete original yet, verify NO Flutter dependencies | Success: File at packages/core_logic/lib/services/script_file_service.dart, imports use relative paths, exported from core_logic.dart, zero Flutter dependencies confirmed_

- [ ] 4. Update UI imports to use core_logic
  - File: ui/lib/services/service_registry.dart (and other UI files)
  - Change imports from relative to package:core_logic
  - Update ServiceRegistry to import from core_logic
  - Verify UI builds successfully
  - Purpose: Use shared services from core_logic
  - _Leverage: ui/lib/services/service_registry.dart (existing DeviceRegistryService import pattern)_
  - _Requirements: REQ-4_
  - _Prompt: Role: Import refactoring specialist | Task: Update all UI imports from relative paths to package:core_logic for ProfileRegistryService, TestService, ScriptFileService, following existing DeviceRegistryService import pattern in ServiceRegistry | Restrictions: Do NOT modify service usage, only import statements, ensure UI builds after changes | Success: All imports use package:core_logic, no broken imports, flutter build succeeds, no compilation errors_

- [ ] 5. Add CLI commands for profile and test management
  - File: apps/cli_tool/lib/command_processor.dart
  - Add ProfileRegistryService, TestService, ScriptFileService to constructor
  - Implement 8 new commands: listProfiles, getProfile, saveProfile, deleteProfile, findCompatibleProfiles, testScript, validateScript, runTests
  - Use JSON protocol for requests and responses
  - Handle errors gracefully
  - Purpose: Enable CLI profile and test operations
  - _Leverage: apps/cli_tool/lib/command_processor.dart (existing command patterns like listRuntime, listHardware)_
  - _Requirements: REQ-5_
  - _Prompt: Role: CLI developer | Task: Add 3 service parameters to CommandProcessor constructor, implement 8 new commands (5 profile + 3 test) following existing command patterns like listRuntime and listHardware, maintain JSON protocol, handle errors gracefully | Restrictions: Maintain JSON protocol format, validate all payloads, return proper error responses, do NOT break existing commands | Success: All 8 commands work, return JSON responses, handle errors with error field, services properly injected, existing commands still work_

- [ ] 6. Delete old UI service files and verify
  - File: ui/lib/services/profile_registry_service.dart (delete)
  - File: ui/lib/services/test_service.dart (delete)
  - File: ui/lib/services/script_file_service.dart (delete)
  - Verify no imports remain to old files using grep
  - Run flutter test to verify UI
  - Test CLI commands
  - Purpose: Clean up old files and ensure quality
  - _Leverage: grep for import verification, flutter test for UI verification_
  - _Requirements: REQ-6, NFR-3_
  - _Prompt: Role: QA and cleanup specialist | Task: Delete old service files after verifying no references remain using grep, run full UI test suite and verify CLI commands work, check zero Flutter dependencies in core_logic | Restrictions: Only delete after all imports updated, verify no broken imports with grep, run full test suite before marking complete | Success: Old files deleted, no broken imports verified with grep, UI tests pass, CLI commands work, zero Flutter dependencies in core_logic confirmed_
