import 'dart:async';
import 'dart:developer';

import '../utils/logger.dart';

/// Direction for FFI boundary crossings.
enum FfiBoundaryDirection { dartToNative, nativeToDart }

/// Structured details for FFI errors to surface helpful hints.
class FfiErrorDetails {
  const FfiErrorDetails({required this.code, required this.message, this.hint});

  factory FfiErrorDetails.from(Object error) {
    final raw = error.toString();

    if (error is ArgumentError || error is RangeError) {
      return const FfiErrorDetails(
        code: 'FFI_ARGUMENT',
        message: 'Invalid argument passed to native layer',
        hint: 'Check pointer lifetimes, buffer lengths, and nullability.',
      );
    }

    if (error is StateError) {
      return const FfiErrorDetails(
        code: 'FFI_STATE',
        message: 'FFI called before initialization',
        hint: 'Ensure KeyrxBridge.initialize() succeeds before calling FFI.',
      );
    }

    if (raw.contains('DynamicLibrary.open')) {
      return const FfiErrorDetails(
        code: 'FFI_LIBRARY_LOAD',
        message: 'Failed to load native library',
        hint: 'Verify binaries are built and placed on PATH/LD_LIBRARY_PATH.',
      );
    }

    if (raw.contains('nullptr') || raw.contains('null pointer')) {
      return const FfiErrorDetails(
        code: 'FFI_NULL_POINTER',
        message: 'Native call returned a null pointer',
        hint: 'Validate native return values and guard against missing data.',
      );
    }

    return FfiErrorDetails(
      code: 'FFI_UNKNOWN',
      message: raw,
      hint: 'Enable tracing to capture context and inspect native logs.',
    );
  }

  final String code;
  final String message;
  final String? hint;

  Map<String, Object?> toContext() => {
    'error_code': code,
    'error_message': message,
    if (hint != null) 'hint': hint,
  };
}

/// Lightweight helpers for tracing and logging FFI interactions.
class FfiDebugTools {
  FfiDebugTools._();

  static final StructuredLogger _logger = getLogger('ffi');

  /// Trace a synchronous FFI call with optional timeline instrumentation.
  static T traceSync<T>({
    required String operation,
    required T Function() call,
    Map<String, Object?> context = const {},
    bool includeTimeline = true,
  }) {
    final stopwatch = Stopwatch()..start();
    if (includeTimeline) {
      Timeline.startSync('FFI: $operation', arguments: context);
    }

    _logger.debug(
      'ffi.call.start',
      context: {'operation': operation, ...context},
    );

    try {
      final result = call();
      final elapsed = stopwatch.elapsed;
      _logger.info(
        'ffi.call.success',
        context: {
          'operation': operation,
          'duration_ms': elapsed.inMilliseconds,
          ...context,
        },
      );
      return result;
    } catch (e, st) {
      final details = FfiErrorDetails.from(e);
      final elapsed = stopwatch.elapsed;
      _logger.error(
        'ffi.call.error',
        message: details.message,
        context: {
          'operation': operation,
          'duration_ms': elapsed.inMilliseconds,
          ...context,
          ...details.toContext(),
        },
        error: e,
        stackTrace: st,
      );
      rethrow;
    } finally {
      if (includeTimeline) {
        Timeline.finishSync();
      }
    }
  }

  /// Trace an async FFI call with optional timeline instrumentation.
  static Future<T> traceAsync<T>({
    required String operation,
    required FutureOr<T> Function() call,
    Map<String, Object?> context = const {},
    bool includeTimeline = true,
  }) async {
    final stopwatch = Stopwatch()..start();
    TimelineTask? task;
    if (includeTimeline) {
      task = TimelineTask()..start('FFI: $operation', arguments: context);
    }

    _logger.debug(
      'ffi.call.start',
      context: {'operation': operation, ...context},
    );

    try {
      final result = await Future<T>.sync(call);
      final elapsed = stopwatch.elapsed;
      _logger.info(
        'ffi.call.success',
        context: {
          'operation': operation,
          'duration_ms': elapsed.inMilliseconds,
          ...context,
        },
      );
      return result;
    } catch (e, st) {
      final details = FfiErrorDetails.from(e);
      final elapsed = stopwatch.elapsed;
      _logger.error(
        'ffi.call.error',
        message: details.message,
        context: {
          'operation': operation,
          'duration_ms': elapsed.inMilliseconds,
          ...context,
          ...details.toContext(),
        },
        error: e,
        stackTrace: st,
      );
      rethrow;
    } finally {
      task?.finish();
    }
  }

  /// Log a boundary crossing between Dart and native code.
  static void logBoundary({
    required FfiBoundaryDirection direction,
    required String operation,
    Map<String, Object?> context = const {},
    Duration? duration,
  }) {
    _logger.debug(
      'ffi.boundary.${direction.name}',
      context: {
        'operation': operation,
        if (duration != null) 'duration_ms': duration.inMilliseconds,
        ...context,
      },
    );
  }

  /// Format an error for user-facing surfaces.
  static FfiErrorDetails formatError(Object error) =>
      FfiErrorDetails.from(error);
}
