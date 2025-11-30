- [x] 1. Define service contracts (Dart)
  - File: ui/lib/services/audio_service.dart, ui/lib/services/permission_service.dart, ui/lib/services/error_translator.dart
  - Define interfaces for AudioService, PermissionService, ErrorTranslator (enums, signatures, UserMessage model).
  - Purpose: Establish DI-ready contracts and typed results.
  - _Leverage: ui/lib/ffi/bridge.dart (for types), permission_handler package_
  - _Requirements: 1_
  - _Prompt: Implement the task for spec flutter-ui-service-layer, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Dart architect | Task: Define AudioService/PermissionService/ErrorTranslator interfaces and data models following requirement 1, with enums for permission and error categories | Restrictions: No concrete FFI calls here, interface-only; keep files under ui/lib/services | _Leverage: ui/lib/ffi, permission_handler | _Requirements: 1 | Success: Interfaces compile, enums cover cases (granted/denied/permanentlyDenied/restricted; info/warning/error), no direct FFI calls in interfaces.

- [x] 2. Implement real services
  - File: ui/lib/services/audio_service_impl.dart, ui/lib/services/permission_service_impl.dart, ui/lib/services/error_translator_impl.dart
  - Implement interfaces using FFI bridge and permission_handler; translate Rust errors to UserMessage.
  - Purpose: Concrete, observable services wrapping FFI.
  - _Leverage: ui/lib/ffi/bridge.dart, permission_handler, tracing/log utilities_
  - _Requirements: 1, 3_
  - _Prompt: Implement the task for spec flutter-ui-service-layer, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter engineer with FFI experience | Task: Implement concrete AudioService/PermissionService/ErrorTranslator with telemetry hooks; start/stop/set BPM, stream subscription, permission requests; map engine errors to user-friendly messages | Restrictions: No UI imports; log trace events for start/stop/permission/stream; handle dispose; no globals | _Leverage: ffi bridge, permission_handler | _Requirements: 1,3 | Success: Services usable via interfaces, logs emitted, errors translated, stream cancel safe.

- [x] 3. Add service registry/provider
  - File: ui/lib/services/service_registry.dart
  - Provide factory for real vs. mock services; simple DI entry point.
  - Purpose: Allow injection/testing without globals.
  - _Leverage: new service interfaces and impls_
  - _Requirements: 1_
  - _Prompt: Implement the task for spec flutter-ui-service-layer, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Software architect | Task: Add ServiceRegistry that wires real PermissionService/AudioService/ErrorTranslator and allows overriding with mocks for tests | Restrictions: No singletons with implicit globals; pass registry down; keep constructors cheap | _Requirements: 1 | Success: Screens can obtain services from registry; tests can supply mocks easily.

- [x] 4. Shared UI widgets and styles
  - File: ui/lib/ui/widgets/app_error_dialog.dart, ui/lib/ui/widgets/loading_overlay.dart, ui/lib/ui/styles/surfaces.dart
  - Implement standardized error dialog, loading overlay, and surface container helper.
  - Purpose: DRY/KISS for presentation, consistent UX.
  - _Leverage: existing theme, widgets patterns in ui/lib/widgets_
  - _Requirements: 2_
  - _Prompt: Implement the task for spec flutter-ui-service-layer, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter UI engineer | Task: Build reusable error dialog, loading overlay, and surface style helper per requirement 2 | Restrictions: Stateless where possible; theming consistent; no business logic | _Requirements: 2 | Success: Widgets reusable across screens; compile without screen dependencies.

- [x] 5. Wire services into screens
  - File: ui/lib/pages/training_screen.dart (and related screens if present)
  - Replace direct FFI calls with service layer; use shared widgets for errors/loading.
  - Purpose: Enforce new architecture and UX consistency.
  - _Leverage: new services, shared widgets_
  - _Requirements: 1,2,3_
  - _Prompt: Implement the task for spec flutter-ui-service-layer, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter integrator | Task: Refactor training (and relevant) screens to use PermissionService/AudioService/ErrorTranslator and shared widgets; add telemetry hooks usage | Restrictions: No raw FFI imports in screens; keep stateful logic lean; handle dispose of streams | _Requirements: 1,2,3 | Success: Screens compile, use services, show standardized dialogs/overlays, telemetry fired on start/stop/permission/stream.

- [ ] 6. Tests (unit + widget)
  - File: ui/test/services/audio_service_test.dart; ui/test/widgets/shared_widgets_test.dart (or similar)
  - Add unit tests for services with mocks; widget tests for shared widgets and service-wired screen flows.
  - Purpose: Ensure testability and regression safety.
  - _Leverage: mocktail or existing test utils_
  - _Requirements: 1,2,3_
  - _Prompt: Implement the task for spec flutter-ui-service-layer, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter QA | Task: Write unit tests for services (permission outcomes, start/stop errors, stream) and widget tests for dialogs/overlays/service-wired screen paths | Restrictions: Use mocks for FFI/permissions; no network; ensure dispose coverage | _Requirements: 1,2,3 | Success: Tests pass, cover success/failure paths, no leaks.
