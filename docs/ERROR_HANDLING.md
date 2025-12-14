# Error Handling & Registry

KeyRx surfaces actionable, user-friendly errors with short codes that point to fixes. Dart-facing codes follow the `E####` format and map to Rust error categories documented in `docs/errors/`.

## Quick Usage
- Use `ErrorHandler` (or the default `ErrorTranslatorImpl`) to translate any error into a UI-ready `UserMessage`.
- Display the message title (includes code) and keep the body for actionable guidance.
- When logging, attach the error code and link so developers can jump to the right playbook section.

```dart
final translator = const ErrorTranslatorImpl();
final message = translator.translate(error);
await AppErrorDialog.show(
  context,
  title: message.title,
  message: message.body,
);
```

## Error Code Registry

| Code  | Scenario                     | User Message Focus                              | Suggested Fix                                              | Docs |
|-------|------------------------------|-------------------------------------------------|------------------------------------------------------------|------|
| E1000 | Generic FFI failure          | Native integration reported an unexpected fail  | Enable FFI tracing, retry, capture context                 | [FFI](#ffi-errors) |
| E1001 | Native library missing       | Failed to load native library                   | Rebuild bindings, ensure shared lib on PATH/LD_LIBRARY_PATH | [FFI](#ffi-errors) |
| E1002 | FFI argument/null pointer    | Invalid data passed to native                   | Validate pointers, buffer lengths, nullability             | [FFI](#ffi-errors) |
| E2001 | Engine not ready/initializing| Engine or bridge not initialized                | Run initialization and wait for ready status               | [Engine](#engine-state) |
| E2002 | Unsupported operation/state  | Operation not allowed in current mode           | Switch to supported workflow or disable incompatible mode  | [Engine](#engine-state) |
| E3001 | Validation/input issue       | Input validation failed                         | Fix required fields/formats/ranges                         | [Validation](#validation-and-input) |
| E3002 | File path/permission issue   | File operation failed                           | Verify path exists and permissions are granted             | [Files](#file-issues) |
| E4001 | Timeout/network instability  | Operation took too long or network unavailable  | Check connectivity and retry once stable                   | [Network](#timeouts-and-network) |
| E9000 | Unexpected error             | Unexpected condition occurred                   | Retry, attach logs, escalate if repeated                   | [Escalation](#escalation) |

## FFI Errors
- **Symptoms:** Missing native library, null pointer, argument/shape mismatch, FFI calls before initialization.
- **What to do:**
  - Enable tracing via `FfiDebugTools.trace*` and inspect structured logs.
  - Verify the shared library is present and matches platform architecture.
  - Re-run `just dev-watch` or `just codegen` to rebuild bindings.
  - Validate buffer sizes and nullability before calling into native.
- **See also:** `docs/FFI_DEBUGGING.md`, `docs/errors/ffi.md`.

## Engine State
- **Symptoms:** Engine not ready, unsupported actions in current mode, initialization races.
- **What to do:**
  - Call `KeyrxBridge.initialize()` once and await readiness before user actions.
  - Avoid calling runtime operations while the engine is starting or stopping.
  - Check `ServiceRegistry.real` initialization logs for bridge load failures.
- **See also:** `docs/DEVELOPMENT.md#runtime-workflows`, `docs/STATE_MANAGEMENT.md`.

## Validation and Input
- **Symptoms:** Missing required fields, invalid formats, out-of-range values.
- **What to do:**
  - Surface specific field names in UI copy.
  - Validate ahead of FFI calls to avoid native failures.
  - Use `FacadeError.validation` for consistent messaging.
- **See also:** `docs/errors/validation.md`.

## File Issues
- **Symptoms:** Missing script/config files, permission denied, read/write failures.
- **What to do:**
  - Confirm paths, file extensions, and casing (especially on Windows).
  - Request permissions where applicable; prefer app data directories.
  - Log the failing path and operation for quick reproduction.
- **See also:** `docs/errors/config.md`.

## Timeouts and Network
- **Symptoms:** Slow responses, offline network, dependency endpoints unreachable.
- **What to do:**
  - Surface the operation name and timeout threshold to the user.
  - Offer a quick retry path; backoff for repeated failures.
  - Capture latency metrics to tighten thresholds.
- **See also:** `docs/PROFILING.md`, `docs/LOGGING.md`.

## Escalation
- If an error repeats with code `E9000` or lacks a clear action:
  - Capture logs with request IDs and recent steps.
  - File an issue referencing the error code and reproduction steps.
  - Add a new registry entry in `ui/lib/utils/error_handler.dart` when a pattern is confirmed to keep codes stable.
