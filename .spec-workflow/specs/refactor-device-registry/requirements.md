# Requirements Document

## Introduction

The current Device Registry logic, which handles "Virtual Devices" and "User Labels", is tightly coupled to the Flutter UI layer (`ui/lib/services/device_registry_service.dart`) and uses `SharedPreferences`. This causes inconsistency where the Dart CLI (Headless Harness) cannot see or manage these persisted settings, leading to "split brain" behavior between the UI and CLI.

This specification outlines the refactoring of the Device Registry logic into the shared `core_logic` package, enabling a Unified Device Registry that persists state to the file system (JSON), ensuring consistency across all interfaces (UI, CLI, Automation).

## Alignment with Product Vision

This aligns with the **Headless Dart Harness Strategy** by ensuring that the CLI can fully replicate the application state, including user configurations for devices. It also reinforces the **Single Source of Truth (SSOT)** principle.

## Requirements

### Requirement 1: Unified Persistence Layer

**User Story:** As a user, when I rename a device in the UI, I want to see that new name when I list devices in the CLI.

#### Acceptance Criteria
1.  Device configuration (Virtual Devices, User Labels) SHALL be stored in a platform-agnostic file format (JSON) accessible to both Dart VM (CLI) and Flutter (UI).
2.  The persistence logic SHALL reside in `packages/core_logic`.
3.  The storage location SHALL follow the project's config standards (e.g., `~/.config/keyrx/devices.json` or relative to the executable).

### Requirement 2: Shared Device Registry Service

**User Story:** As a developer, I want to use the same `DeviceRegistryService` class in both the CLI and the App to ensure identical behavior.

#### Acceptance Criteria
1.  A new `DeviceRegistryService` class SHALL be implemented in `packages/core_logic`.
2.  This service SHALL merge "Real" devices (from FFI) with "Virtual" devices (from Persistence).
3.  The UI's `DeviceRegistryService` SHALL be replaced or refactored to wrap this shared service.

### Requirement 3: CLI Device Management

**User Story:** As a QA engineer, I want to add a virtual device via the CLI and have it appear in the App.

#### Acceptance Criteria
1.  The CLI `listDevices` command SHALL return the merged list of devices (Real + Virtual).
2.  The CLI SHALL support commands to `addVirtualDevice`, `removeVirtualDevice`, and `setUserLabel`.

### Requirement 4: UI Consistency

**User Story:** As a user, I expect the UI to behave exactly as before, but backed by the new shared service.

#### Acceptance Criteria
1.  The "Managed Devices" list in the UI SHALL populate correctly using the new service.
2.  Changes made in the UI SHALL be persisted to the new file location.
3.  (Migration) Existing `SharedPreferences` data SHOULD be migrated to the new file if possible, or we accept a reset (Decision: Reset is acceptable for this refactor if migration is too complex, but migration is preferred). *Decision: Support basic migration if easy, otherwise reset.*

## Non-Functional Requirements

### Modularity
- `core_logic` must NOT depend on `shared_preferences` (Flutter plugin). It should use `dart:io` for file access.

### Reliability
- The file persistence must handle concurrent reads/writes safely (basic locking or atomic writes).
