# Tasks Document

- [x] 0. Set up iteration-speed dev loop
  - Files: justfile (or scripts), docs snippet in ui/README or CONTRIBUTING section
  - Add commands for hot reload with mock bridge (`just keyflow:dev`), real FFI minimal rebuild (`just keyflow:ffi` using sccache/incremental), and fast test lanes (nextest/unit + widget subset); document stub data toggle (dart-define)
  - _Leverage: docs/AUTONOMOUS-DEV-GUIDE.md, existing justfile patterns, flutter run configs_
  - _Requirements: 5_
  - _Prompt: Implement the task for spec keyflow-configuration-viewer, first run spec-workflow-guide to get the workflow guide then implement the task: Role: DX engineer. Task: Create documented commands for hot-reload with mock bridge, minimal-rebuild real FFI runs, and fast test lanes (nextest/unit/widget subset) plus a short doc note. Restrictions: No destructive changes; keep commands opt-in and fast; ensure mock bridge path does not require native lib. _Leverage: AUTONOMOUS-DEV-GUIDE, justfile. _Requirements: 5. Success: developers can run a documented one-liner for UI-only work without native lib, another for real FFI with cached builds, and a fast test lane, with instructions checked into repo._

- [x] 1. Create KeyFlow repository facade
  - File: ui/lib/services/facade/keyflow_repository.dart
  - Aggregate devices, profiles, layouts, keymaps, runtime overlay into `KeyFlowSnapshot`
  - Provide `loadAll()`, `refreshDevices()`, `runtimeStream()` using existing services (DeviceRegistryService, HardwareService, LayoutService, KeymapService, RuntimeService)
  - _Leverage: ui/lib/services/device_registry_service.dart, ui/lib/services/hardware_service.dart, ui/lib/services/layout_service.dart, ui/lib/services/keymap_service.dart, ui/lib/services/runtime_service.dart_
  - _Requirements: 1, 2, 3, 4, 5_
  - _Prompt: Implement the task for spec keyflow-configuration-viewer, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter/Dart service engineer. Task: Build a KeyFlowRepository that composes existing services to produce KeyFlowSnapshot data and exposes a runtime overlay stream. Restrictions: Do not call FFI directly; use existing services; keep it read-only; ensure per-call error isolation; keep iteration speed in mind (mock-friendly, no heavy init). _Leverage: listed services. _Requirements: 1,2,3,4,5. Success: repository returns aggregated data with gap/conflict counts, handles partial failures with errors collected, provides runtime overlay updates without blocking UI, and can run with a mock bridge._

- [x] 2. Add KeyFlow view model
  - File: ui/lib/state/keyflow_view_model.dart
  - Manage loading, selection, staleness timers, gap/conflict derivation, error states; expose ChangeNotifier API
  - _Leverage: ui/lib/services/facade/keyflow_repository.dart_
  - _Requirements: 1, 2, 3, 4, 5_
  - _Prompt: Implement the task for spec keyflow-configuration-viewer, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter state-management developer. Task: Create KeyFlowViewModel that loads snapshots, subscribes to runtime stream, tracks selection and staleness, exposes derived counts and UI-safe errors. Restrictions: No widget logic; keep under 300 lines; debounce refreshes; staleness at 5s; support mock-bridge mode for fast reload. _Leverage: KeyFlowRepository. _Requirements: 1,2,3,4,5. Success: view model notifies listeners on data/runtime updates, maintains selection without resets on device updates, marks runtime stale after 5s silence, and can operate with mock data._

- [x] 3. Build KeyFlow screen and panes
  - File: ui/lib/pages/keyflow_page.dart
  - Compose DeviceListPane, FlowGraphPane, DetailsPane, RuntimeStrip; wire to KeyFlowViewModel
  - _Leverage: ui/lib/widgets/virtual_layout_renderer.dart, ui/lib/pages/wiring.dart (patterns), ui/lib/pages/mapping/mapping_dashboard.dart (load patterns)_
  - _Requirements: 1, 2, 3, 5_
  - _Prompt: Implement the task for spec keyflow-configuration-viewer, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter UI engineer. Task: Create KeyFlowPage with panes for devices, wiring graph, key details, and runtime strip, consuming KeyFlowViewModel. Restrictions: Read-only; preserve scroll/selection on updates; show banners for bridge errors and stale runtime; keep widgets split into panes in same file or subwidgets; support mock-bridge hot reload without native lib. _Leverage: listed widgets/pages. _Requirements: 1,2,3,5. Success: screen renders live data, flags gaps/conflicts, shows runtime overlays, handles empty/error states clearly, and supports the hot-reload mock loop._

- [x] 4. Add telemetry hooks for KeyFlow
  - File: ui/lib/services/metrics_service.dart (extend) and/or ui/lib/services/observability_service.dart usage within keyflow repository/view model
  - Record fetch latency (p95), success/failure counts, render timing for large layouts; capture parse errors with source + payload size only
  - _Leverage: ui/lib/services/metrics_service.dart, ui/lib/services/observability_service.dart_
  - _Requirements: 4, 5_
  - _Prompt: Implement the task for spec keyflow-configuration-viewer, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Observability-focused engineer. Task: Add metrics around KeyFlowRepository calls and rendering hooks (exposed via view model) capturing success/failure counts, p95 latency, and render timing for large layouts (>150 keys or 5+ layers), with CLI-friendly summary surfaces for the automation loop in docs/AUTONOMOUS-DEV-GUIDE.md. Restrictions: No PII; redact serials; avoid heavy logging on UI thread; keep metrics lightweight for fast dev loops. _Leverage: metrics/observability services. _Requirements: 4,5. Success: metrics emitted when enabled, lightweight when disabled, errors include operation + payload size only, CLI agents can consume summary output without UI, and telemetry does not slow hot reload._

- [x] 5. Tests for repository and view model
  - Files: ui/test/keyflow_repository_test.dart, ui/test/keyflow_view_model_test.dart
  - Mock services to cover aggregation, partial failures, staleness, gap/conflict computation
  - _Leverage: existing test fixtures/patterns in ui/test_
  - _Requirements: 1, 2, 3, 4, 5_
  - _Prompt: Implement the task for spec keyflow-configuration-viewer, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter/Dart QA engineer. Task: Write unit tests for KeyFlowRepository and KeyFlowViewModel using mocked services to validate aggregation, error propagation, staleness, selection preservation, and mock-bridge mode. Restrictions: No real FFI; use fakes; keep tests deterministic. _Leverage: ui/test helpers. _Requirements: 1,2,3,4,5. Success: tests cover happy/failure paths, staleness timer at 5s, gap/conflict counts, runtime updates propagate correctly, and mock-mode works without native libs._

- [ ] 6. Widget tests for KeyFlow page
  - File: ui/test/keyflow_page_test.dart
  - Verify rendering of panes, empty/error states, runtime stale banner, selection persistence
  - _Leverage: Flutter widget testing patterns in repo_
  - _Requirements: 1, 2, 3, 5_
  - _Prompt: Implement the task for spec keyflow-configuration-viewer, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter widget test engineer. Task: Add widget tests for KeyFlowPage to cover empty/error states, device list updates, gap/conflict badges, runtime overlays, stale indicator, and mock-bridge rendering. Restrictions: Use mock view model/provider; avoid golden tests unless already used. _Leverage: existing widget test utilities. _Requirements: 1,2,3,5. Success: widget tests assert correct banners, panes render with mock data, selections persist across updates, runtime stale state shows expected UI, and mock-bridge mode works without native libs._
