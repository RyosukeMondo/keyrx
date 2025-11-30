# Design Document

## Overview

Introduce a Flutter-facing service layer and shared UI primitives so screens no longer talk directly to FFI. Define `PermissionService`, `AudioService`, and `ErrorTranslator` interfaces with real and mock implementations. Extract reusable widgets (error dialog, loading overlay) and container styles. Add lightweight telemetry hooks for start/stop/permission/stream events to aid debugging and analysis.

## Steering Document Alignment

### Technical Standards (tech.md)
- No global state: services are injected, FFI stays behind service boundaries.
- Event-sourcing mindset: UI subscribes to streams via `AudioService`, not manual FFI calls.
- Dependency injection: services exposed as interfaces; screens depend on abstractions.
- Observability: trace/log hooks on service calls to support debugging/analysis.

### Project Structure (structure.md)
- Place services in `ui/lib/services/`.
- Shared widgets in `ui/lib/ui/widgets/`.
- Styles in `ui/lib/ui/styles/`.
- Keep imports layered: UI -> services -> FFI bridge; no UI -> FFI direct.

## Code Reuse Analysis

### Existing Components to Leverage
- Existing FFI bridge in `ui/lib/ffi` for actual Rust calls.
- Current pages (editor/debugger/console) patterns for routing/state.
- Existing models for classification results/errors.

### Integration Points
- Services wrap existing FFI functions (`startAudio`, `stopAudio`, `classificationStream`).
- PermissionService uses `permission_handler`.
- ErrorTranslator maps Rust error strings/enums to user-facing messages.

## Architecture

Service layer wraps FFI and platform concerns:
- `PermissionService`: request/check microphone permission, return enum.
- `AudioService`: start/stop/set BPM, expose `Stream<ClassificationResult>`.
- `ErrorTranslator`: map engine errors to friendly text + categories.
- `ServiceRegistry` (simple provider) to inject mocks vs. real.

Shared UI:
- `AppErrorDialog`: standardized error display.
- `LoadingOverlay`: full-screen blocking overlay.
- `SurfaceContainer`: styled container helper.

Telemetry:
- Trace/log in AudioService for start/stop/stream subscribe.
- Trace/log in PermissionService for outcomes.

### Modular Design Principles
- Single File Responsibility: one interface/impl per file; widgets isolated.
- Component Isolation: widgets stateless where possible; services handle side effects.
- Service Layer Separation: UI calls services; services call FFI.
- Utility Modularity: styles in a dedicated styles file with constants.

## Components and Interfaces

### PermissionService
- **Purpose:** Encapsulate microphone permission flow.
- **Interfaces:** `Future<PermissionResult> requestMicrophone();`
- **Dependencies:** `permission_handler`.
- **Reuses:** None; adapters only.

### AudioService
- **Purpose:** Wrap audio lifecycle + classification stream.
- **Interfaces:** `Future<void> start(int bpm); Future<void> stop(); Stream<ClassificationResult> stream();`
- **Dependencies:** FFI bridge; ErrorTranslator; PermissionService (composition).
- **Reuses:** Existing FFI methods.

### ErrorTranslator
- **Purpose:** Map Rust errors to user-facing messages and categories.
- **Interfaces:** `UserMessage translate(Object error);`
- **Dependencies:** None.
- **Reuses:** Known Rust error strings/constants.

### Shared Widgets
- **AppErrorDialog:** `Widget build(BuildContext, String title, String message)`
- **LoadingOverlay:** `Widget overlay({bool active, Widget child})`
- **SurfaceContainer:** `Widget surface({Widget child})`

## Data Models

### PermissionResult (enum)
- `granted | denied | permanentlyDenied | restricted`

### UserMessage
- `title: String`
- `body: String`
- `category: enum { info, warning, error }`

## Error Handling

### Error Scenarios
1. **Audio start fails (engine busy/device issue)**
   - **Handling:** AudioService catches, translates via ErrorTranslator, returns failure to UI; UI shows AppErrorDialog.
   - **User Impact:** Clear message, no crash; state stays idle.
2. **Permission denied**
   - **Handling:** PermissionService returns denied; UI shows dialog with guidance; AudioService not invoked.
   - **User Impact:** No attempt to start audio; actionable copy.
3. **Stream drops**
   - **Handling:** AudioService exposes stream errors as translated messages; UI can resubscribe or show toast/dialog.
   - **User Impact:** User informed; app continues running.

## Testing Strategy

### Unit Testing
- Service interfaces mocked; real implementations tested with fake FFI/mocks.
- ErrorTranslator table-driven tests.

### Integration Testing
- Widget tests: screens using mock services (granted/denied/start fail/start success) ensure dialogs/overlays appear.
- Stream handling: verify subscription/dispose paths don’t throw.

### End-to-End Testing
- Happy path: permission granted -> audio start -> stream emits -> UI renders classification.
- Denied path: permission denied -> no start -> dialog shown.
- Failure path: start error -> dialog shown -> state reset.
