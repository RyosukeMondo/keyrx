# Structured Logging (Dart/Flutter)

KeyRx UI now uses structured JSON logs to match the Rust engine’s tracing output. Every log line is a single JSON object that can be shipped to aggregators and filtered by level, service, and event.

## Format
- `timestamp` (UTC ISO8601)
- `level` (`TRACE`, `DEBUG`, `INFO`, `WARNING`, `SEVERE`)
- `service` (logical module name passed to `getLogger`)
- `event` (snake_case, past tense)
- `message` (optional, human-readable summary)
- `context` (optional, key/value fields; values are sanitized to strings/numbers/bools/arrays)
- `error` / `stack` (optional when logging failures)

Example:
```json
{"timestamp":"2025-12-14T15:00:00.000Z","level":"INFO","service":"device_service","event":"device_selected","message":"Device selection succeeded","context":{"path":"/dev/input/event3"}}
```

## Usage
```dart
import 'package:keyrx_ui/utils/logger.dart';

final log = getLogger('device_service');

Future<void> connect(String path) async {
  log.info('device_selected', message: 'Device selection succeeded', context: {'path': path});
}

Future<void> handleError(Object error, StackTrace stack) async {
  log.error(
    'device_selection_error',
    message: 'Device selection failed',
    context: {'path': '/dev/input/event3'},
    error: error,
    stackTrace: stack,
  );
}
```

## Log Levels
- `trace` → verbose internals (mapped to `Level.FINEST`)
- `debug` → diagnostics during development (`Level.FINE`)
- `info` → lifecycle and business events (`Level.INFO`)
- `warn` → recoverable issues or degraded behavior (`Level.WARNING`)
- `error` → failures that need action (`Level.SEVERE`)

The default level is `DEBUG` in debug/profile builds and `INFO` in release. Override via:
```
flutter run --dart-define=KEYRX_LOG_LEVEL=trace
```
Accepted values: `trace`, `debug`, `info`, `warn`, `error`.

## Guidance
- Prefer structured `context` fields over string interpolation.
- Do **not** log secrets, tokens, or PII.
- Use consistent event names (snake_case, past tense).
- Keep log lines single-line JSON for ingestion.
- Include identifiers that help correlate with Rust logs when relevant (device path, profile id, etc.).
- Use `KEYRX_LOG_LEVEL` for temporary verbosity during debugging (`trace` for deep FFI issues, drop back to `info`/`debug` afterward).
- FFI utilities (`FfiDebugTools.*`) already emit structured logs; avoid double-logging the same boundary events.
- If you add new services, create a dedicated logger via `getLogger('service_name')` so downstream aggregation can filter cleanly.
