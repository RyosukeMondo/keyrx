# Requirements: Move Utilities to Core

**Spec Name**: move-utilities-to-core
**Created**: 2025-12-20
**Status**: Draft
**Priority**: Low (SSOT completion)

---

## Overview

Move ErrorTranslator and StoragePathResolver from UI to core_logic to enable shared error messages and path resolution across CLI and UI.

---

## Business Goals

1. Share error translation logic between CLI and UI
2. Share storage path resolution logic
3. Reduce code duplication
4. Achieve complete SSOT for utilities

---

## Requirements

### REQ-1: Move ErrorTranslator to Core

**Description**: Move ErrorTranslator and ErrorTranslatorImpl from UI to core_logic.

**Current State**:
- Location: `ui/lib/services/error_translator.dart` (interface)
- Location: `ui/lib/services/error_translator_impl.dart` (implementation)
- Flutter deps: 0 (already pure Dart)
- Used by: UI only

**Target State**:
- Location: `packages/core_logic/lib/utils/error_translator.dart`
- Location: `packages/core_logic/lib/utils/error_translator_impl.dart`
- Exported from `core_logic.dart`
- Used by: CLI and UI

**Acceptance Criteria**:
- ✅ Files moved to core_logic/utils/
- ✅ Zero Flutter dependencies verified
- ✅ Exported from core_logic.dart
- ✅ UI imports from core_logic

---

### REQ-2: Move StoragePathResolver to Core

**Description**: Move StoragePathResolver from UI to core_logic.

**Current State**:
- Location: `ui/lib/services/storage_path_resolver.dart`
- Flutter deps: 0 (already pure Dart)
- Used by: UI only

**Target State**:
- Location: `packages/core_logic/lib/utils/storage_path_resolver.dart`
- Exported from `core_logic.dart`
- Used by: CLI and UI

**Acceptance Criteria**:
- ✅ File moved to core_logic/utils/
- ✅ Zero Flutter dependencies verified
- ✅ Exported from core_logic.dart
- ✅ UI imports from core_logic

---

### REQ-3: Update UI Imports

**Description**: Update all UI imports to use core_logic package.

**Acceptance Criteria**:
- ✅ ServiceRegistry imports from core_logic
- ✅ All UI files using utilities import from core_logic
- ✅ No broken imports
- ✅ UI builds successfully

---

### REQ-4: Verify and Test

**Description**: Ensure utilities work correctly in both CLI and UI.

**Acceptance Criteria**:
- ✅ UI tests pass
- ✅ Error messages work correctly
- ✅ Path resolution works
- ✅ No Flutter dependencies in core_logic
- ✅ No regressions

---

## Non-Functional Requirements

### NFR-1: Zero Flutter Dependencies

All moved utilities must have zero Flutter package dependencies.

### NFR-2: Backward Compatibility

UI functionality must remain unchanged.

### NFR-3: Code Quality

All code must pass pre-commit checks.

---

## Success Metrics

- ✅ 2 utilities moved to core_logic
- ✅ Zero Flutter deps verified
- ✅ UI and CLI both using shared utilities
- ✅ All tests passing
- ✅ No code duplication

---

## Out of Scope

- Adding new error messages
- Changing error translation logic
- Adding new path resolution features
- Modifying utility implementations
