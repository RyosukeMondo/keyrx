# Tasks: Move Config Services to core_logic

- [x] 1. Move config_result.dart to core_logic
  - File: packages/core_logic/lib/services/config_result.dart
  - Create directory if needed
  - Copy file from UI
  - Export from core_logic.dart
  - Purpose: Foundation for configuration services
  - _Requirements: REQ-1, REQ-2, REQ-4_

- [x] 2. Move runtime_service.dart to core_logic
  - File: packages/core_logic/lib/services/runtime_service.dart
  - Copy from UI
  - Update imports to relative paths
  - Export from core_logic.dart
  - Verify zero Flutter dependencies
  - Purpose: Decouple runtime config management from UI
  - _Requirements: REQ-1, REQ-3_

- [x] 3. Move keymap_service.dart to core_logic
  - File: packages/core_logic/lib/services/keymap_service.dart
  - Copy from UI
  - Update imports to relative paths
  - Export from core_logic.dart
  - Verify zero Flutter dependencies
  - Purpose: Decouple keymap management from UI
  - _Requirements: REQ-2, REQ-3_

- [x] 4. Update UI imports to use core_logic
  - File: ui/lib/services/service_registry.dart
  - Update imports in service_registry.dart
  - Update other UI files using these services
  - Run flutter pub get and dart analyze
  - Purpose: Finalize migration to core_logic services
  - _Requirements: REQ-3_

- [x] 5. Run tests and verify functionality
  - File: N/A
  - Run flutter test in UI
  - Run dart analyze
  - Check code metrics
  - Purpose: Ensure no regressions
  - _Requirements: REQ-3, NFR-2, NFR-3_

- [x] 6. Delete old service files from UI
  - File: ui/lib/services/runtime_service.dart, ui/lib/services/keymap_service.dart, ui/lib/services/config_result.dart
  - Delete original files after successful migration
  - Verify no references remain
  - Purpose: Cleanup duplicates
  - _Requirements: REQ-1, REQ-2_