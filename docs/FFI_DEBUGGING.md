# FFI Debugging Guide

This guide explains how to trace FFI interactions, capture Flutter Timeline events, and surface actionable error messages when crossing the Dart ↔ native boundary.

## Quick start
- Use `FfiDebugTools.traceSync` or `traceAsync` to wrap any call into native bindings.
- All traces emit structured JSON logs via `StructuredLogger` under the `ffi` service.
- Timeline events appear in Flutter DevTools with the label `FFI: <operation>`.
- Use `FfiDebugTools.formatError` to produce user-friendly messages for UI surfaces.

```dart
import 'package:keyrx_ui/ffi/debug_tools.dart';

final result = await FfiDebugTools.traceAsync(
  operation: 'engine.eval',
  context: {'command': ':stats'},
  call: () => bridge.eval(':stats'),
);
```

## Boundary logging
- `FfiDebugTools.logBoundary` records every crossing at the trace/debug level:
  - `dartToNative` (Dart invoking FFI)
  - `nativeToDart` (callbacks from Rust to Dart)
- Context fields include the operation name, event type, and payload size when available.
- View logs via stdout or any configured sink from `LoggingConfigurator`.

## Timeline instrumentation
- Synchronous calls use `Timeline.startSync`/`finishSync`.
- Async calls use `TimelineTask` to keep spans connected.
- Open Flutter DevTools → Performance → select the `FFI: <operation>` slices to inspect durations.

## Error formatting
`FfiDebugTools.formatError` classifies common failures and returns a `FfiErrorDetails` object:
- `FFI_LIBRARY_LOAD`: Native library missing or cannot be opened (check `LD_LIBRARY_PATH`/DLL location).
- `FFI_ARGUMENT`: Invalid pointer, buffer length, or nullability contract.
- `FFI_STATE`: Bridge used before `initialize()` completed.
- `FFI_NULL_POINTER`: Native call returned `nullptr`.
- `FFI_UNKNOWN`: Fallback with the raw error text.

Use `details.toContext()` to add error metadata to logs:

```dart
try {
  bridge.initialize();
} catch (e, st) {
  final details = FfiDebugTools.formatError(e);
  logger.error(
    'ffi.initialization_failed',
    message: details.message,
    context: details.toContext(),
    error: e,
    stackTrace: st,
  );
}
```

## Common issues and fixes
- **Protocol mismatch**: If logs show `protocol_version` mismatch, rebuild Rust core and regenerate bindings.
- **Null payloads**: `ffi.call.error` with `FFI_NULL_POINTER` usually means the native side did not allocate/return data; verify the Rust function contract.
- **Callbacks stalled**: No `nativeToDart` boundary logs for events; ensure `registerEventCallback` returns success and the state stream is open.
- **Timeline missing**: Run the app in profile/debug mode and open DevTools; timeline spans are not recorded in release mode.

## Workflow checklist
1. Wrap new FFI entry points with `traceSync/traceAsync`.
2. Watch stdout or configured sink for `ffi.call.*` and `ffi.boundary.*` events.
3. Inspect Timeline slices for expensive calls; add latency budgets to contexts where needed.
4. Surface formatted errors to users instead of raw exception strings.
