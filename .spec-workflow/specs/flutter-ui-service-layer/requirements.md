# Requirements Document

## Introduction

Flutter-facing improvements to decouple UI from FFI, provide reusable presentation primitives, and surface user-friendly errors. This phase introduces a Dart service layer (audio/permission/error), shared widgets for dialogs/loading/containers, and baseline telemetry to make the UI observable and testable.

## Alignment with Product Vision

Delivers “CLI first, GUI later” parity by keeping UI thin over the already-capable Rust engine, while making the GUI debuggable and safe (no global state, explicit services). Improves visibility (“Visual > Abstract”) via reusable UI primitives and structured error handling that mirrors engine guarantees.

## Requirements

### Requirement 1

**User Story:** As a power user, I want the Flutter UI to call a typed service layer instead of raw FFI so that behaviors are consistent, testable, and mockable.

#### Acceptance Criteria

1. WHEN the UI starts training or calibration THEN it SHALL obtain microphone permission via a `PermissionService` abstraction and block start if denied.
2. IF audio start fails in Rust THEN the `AudioService` SHALL surface a structured error enum and a user-friendly message (no raw Rust strings in UI).
3. WHEN requesting classification stream THEN the UI SHALL subscribe via the service and be able to swap in a mock service in tests without code changes to screens.

### Requirement 2

**User Story:** As a user, I want consistent UI affordances (errors/loading/containers) so that the app looks coherent and is easy to reason about.

#### Acceptance Criteria

1. WHEN an operation is in progress THEN a shared loading overlay SHALL be used instead of bespoke spinners.
2. WHEN an error is shown THEN a shared error dialog component SHALL be used with standardized title/body/actions.
3. WHEN displaying grouped content THEN a shared container style helper SHALL be applied (padding, radius, surface color) instead of ad-hoc Containers.

### Requirement 3

**User Story:** As a developer, I want telemetry hooks for UI actions so that I can debug and analyze flows quickly.

#### Acceptance Criteria

1. WHEN audio start/stop succeeds or fails THEN a trace/log entry SHALL be emitted with outcome and duration.
2. WHEN a permission request completes THEN the result SHALL be logged (allow/deny/permanently denied) without leaking PII.
3. WHEN classification stream subscription starts/stops THEN events SHALL be logged for timing analysis.

## Non-Functional Requirements

### Code Architecture and Modularity
- **Single Responsibility Principle**: Services isolate FFI concerns; widgets isolate presentation; screens orchestrate only.
- **Modular Design**: Services in `lib/services`, widgets in `lib/ui/widgets`, styles in `lib/ui/styles`, no circular deps.
- **Dependency Management**: UI depends on services via interfaces; services depend on FFI bridge; no UI→FFI direct imports.
- **Clear Interfaces**: PermissionService/AudioService/ErrorTranslator exposed as interfaces with mock implementations for tests.

### Performance
- Service calls and error translation SHALL add <1ms overhead per call on a mid-tier device.
- Shared widgets SHALL not regress initial render time by more than 5%.

### Security
- No raw Rust error strings shown; user-facing messages are sanitized.
- Permission state is not persisted beyond session; no extra storage of PII.

### Reliability
- Service layer must be fully mockable for widget tests; failures are surfaced as typed results, not thrown exceptions.
- UI must handle stream cancellation gracefully (no unhandled exceptions on dispose).

### Usability
- Error dialogs use actionable, plain-language messages with a single primary action.
- Loading overlay blocks interaction and is dismissible only when operations complete.
