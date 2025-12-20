# Tasks: Move Utilities to Core

- [x] 1. Move ErrorTranslator to core_logic
  - File: packages/core_logic/lib/utils/error_translator.dart
  - File: packages/core_logic/lib/utils/error_translator_impl.dart
  - Copy both interface and implementation from ui/lib/services/ to core_logic/lib/utils/
  - Update imports to use relative paths
  - Add exports to core_logic.dart
  - Verify zero Flutter dependencies
  - Purpose: Enable shared error translation across CLI and UI
  - _Leverage: packages/core_logic/lib/services/runtime_service.dart (move pattern)_
  - _Requirements: REQ-1_
  - _Prompt: Role: Package organizer | Task: Move ErrorTranslator interface and ErrorTranslatorImpl from ui/lib/services/ to core_logic/lib/utils/ maintaining both files, update imports to relative paths, add exports to core_logic.dart | Restrictions: Do NOT modify logic, do NOT delete originals yet, verify NO Flutter dependencies using grep, maintain interface/implementation separation | Success: Both files at packages/core_logic/lib/utils/, imports use relative paths, both exported from core_logic.dart, zero Flutter dependencies confirmed with grep_

- [-] 2. Move StoragePathResolver to core_logic
  - File: packages/core_logic/lib/utils/storage_path_resolver.dart
  - Copy from ui/lib/services/storage_path_resolver.dart to core_logic/lib/utils/
  - Update imports to use relative paths
  - Add export to core_logic.dart
  - Verify zero Flutter dependencies
  - Purpose: Enable shared path resolution across CLI and UI
  - _Leverage: packages/core_logic/lib/utils/error_translator.dart (Task 1 pattern)_
  - _Requirements: REQ-2_
  - _Prompt: Role: Package organizer | Task: Move StoragePathResolver from ui/lib/services/ to core_logic/lib/utils/ following same pattern as ErrorTranslator in Task 1, update imports to relative paths, add export to core_logic.dart | Restrictions: Do NOT modify logic, do NOT delete original yet, verify NO Flutter dependencies using grep | Success: File at packages/core_logic/lib/utils/storage_path_resolver.dart, imports use relative paths, exported from core_logic.dart, zero Flutter dependencies confirmed_

- [ ] 3. Update UI imports to use core_logic
  - File: ui/lib/services/service_registry.dart (and other UI files)
  - Change imports from relative to package:core_logic/utils/
  - Update ServiceRegistry to import utilities from core_logic
  - Find and update all other UI files using these utilities
  - Verify UI builds successfully
  - Purpose: Use shared utilities from core_logic
  - _Leverage: ui/lib/services/service_registry.dart (existing core_logic import patterns)_
  - _Requirements: REQ-3_
  - _Prompt: Role: Import refactoring specialist | Task: Update all UI imports from ui/lib/services/ to package:core_logic/utils/ for ErrorTranslator, ErrorTranslatorImpl, and StoragePathResolver, ensure all files using these utilities are updated | Restrictions: Do NOT modify utility usage, only import statements, ensure UI builds after changes, find ALL files using these utilities with grep | Success: All imports use package:core_logic/utils/, no broken imports, flutter build succeeds, all files using utilities updated, no compilation errors_

- [ ] 4. Delete old utility files and verify
  - File: ui/lib/services/error_translator.dart (delete)
  - File: ui/lib/services/error_translator_impl.dart (delete)
  - File: ui/lib/services/storage_path_resolver.dart (delete)
  - Verify no imports remain to old files using grep
  - Run flutter test to verify UI
  - Verify zero Flutter dependencies in core_logic/utils/
  - Purpose: Clean up old files and ensure quality
  - _Leverage: grep for import verification, flutter test for UI verification_
  - _Requirements: REQ-4, NFR-1, NFR-3_
  - _Prompt: Role: QA and cleanup specialist | Task: Delete old utility files from ui/lib/services/ after verifying no references remain using grep, run full UI test suite and verify zero Flutter dependencies in core_logic/utils/ | Restrictions: Only delete after all imports updated, verify no broken imports with grep, run full test suite before marking complete | Success: Old files deleted, no broken imports verified with grep, UI tests pass, zero Flutter dependencies in core_logic/utils/ confirmed, error messages work correctly, path resolution works_
