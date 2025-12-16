import 'dart:async';
import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:logging/logging.dart';

typedef LogSink = void Function(String line);

/// Configures the global logging pipeline to emit structured JSON lines.
class LoggingConfigurator {
  LoggingConfigurator._();

  static bool _initialized = false;
  static LogSink _sink = print;
  static StreamSubscription<LogRecord>? _subscription;

  /// Initialize the logger. Safe to call multiple times; the first call wins.
  static void configure({Level? level, LogSink? sink}) {
    if (_initialized) {
      return;
    }

    if (sink != null) {
      _sink = sink;
    }

    Logger.root.level = level ?? _defaultLevel();
    _subscription = Logger.root.onRecord.listen((record) {
      final payload = _buildPayload(record);
      _sink(jsonEncode(payload));
    });

    _initialized = true;
  }

  static Level _defaultLevel() {
    const envLevel = String.fromEnvironment('KEYRX_LOG_LEVEL');
    final parsed = _parseLevel(envLevel);
    if (parsed != null) {
      return parsed;
    }

    return kReleaseMode ? Level.INFO : Level.FINE;
  }

  static Level? _parseLevel(String? value) {
    if (value == null || value.isEmpty) {
      return null;
    }

    final normalized = value.toLowerCase();
    return switch (normalized) {
      'trace' => Level.FINEST,
      'debug' => Level.FINE,
      'info' => Level.INFO,
      'warn' || 'warning' => Level.WARNING,
      'error' || 'severe' => Level.SEVERE,
      _ => null,
    };
  }

  static Map<String, dynamic> _buildPayload(LogRecord record) {
    final base = <String, dynamic>{};

    if (record.object is Map) {
      base.addAll(_sanitizeMap(record.object as Map));
    } else if (record.message.isNotEmpty) {
      base['message'] = record.message;
    }

    return {
      'timestamp': record.time.toUtc().toIso8601String(),
      'level': record.level.name,
      'service': record.loggerName.isNotEmpty ? record.loggerName : 'ui',
      ...base,
      if (record.error != null) 'error': record.error.toString(),
      if (record.stackTrace != null) 'stack': record.stackTrace.toString(),
    };
  }

  static Map<String, dynamic> _sanitizeMap(Map<dynamic, dynamic> input) =>
      input.map((key, value) => MapEntry(key.toString(), _coerce(value)));

  static dynamic _coerce(dynamic value) {
    if (value == null ||
        value is num ||
        value is bool ||
        value is String ||
        value == null) {
      return value;
    }

    if (value is DateTime) {
      return value.toUtc().toIso8601String();
    }

    if (value is Enum) {
      return value.name;
    }

    if (value is Map) {
      return _sanitizeMap(value);
    }

    if (value is Iterable) {
      return value.map(_coerce).toList();
    }

    return value.toString();
  }

  /// Reset logging so tests can configure sinks cleanly.
  @visibleForTesting
  static Future<void> resetForTesting() async {
    await _subscription?.cancel();
    _subscription = null;
    _initialized = false;
    _sink = print;
  }

  /// Override sink for tests to capture log lines.
  @visibleForTesting
  static void useTestSink(LogSink sink) {
    _sink = sink;
  }
}

/// Structured logger that emits JSON lines with consistent fields.
class StructuredLogger {
  StructuredLogger._(this._logger);

  final Logger _logger;

  void trace(
    String event, {
    String? message,
    Map<String, Object?> context = const {},
    Object? error,
    StackTrace? stackTrace,
  }) => _emit(
    Level.FINEST,
    event,
    message: message,
    context: context,
    error: error,
    stackTrace: stackTrace,
  );

  void debug(
    String event, {
    String? message,
    Map<String, Object?> context = const {},
    Object? error,
    StackTrace? stackTrace,
  }) => _emit(
    Level.FINE,
    event,
    message: message,
    context: context,
    error: error,
    stackTrace: stackTrace,
  );

  void info(
    String event, {
    String? message,
    Map<String, Object?> context = const {},
    Object? error,
    StackTrace? stackTrace,
  }) => _emit(
    Level.INFO,
    event,
    message: message,
    context: context,
    error: error,
    stackTrace: stackTrace,
  );

  void warn(
    String event, {
    String? message,
    Map<String, Object?> context = const {},
    Object? error,
    StackTrace? stackTrace,
  }) => _emit(
    Level.WARNING,
    event,
    message: message,
    context: context,
    error: error,
    stackTrace: stackTrace,
  );

  void error(
    String event, {
    String? message,
    Map<String, Object?> context = const {},
    Object? error,
    StackTrace? stackTrace,
  }) => _emit(
    Level.SEVERE,
    event,
    message: message,
    context: context,
    error: error,
    stackTrace: stackTrace,
  );

  void _emit(
    Level level,
    String event, {
    String? message,
    Map<String, Object?> context = const {},
    Object? error,
    StackTrace? stackTrace,
  }) {
    LoggingConfigurator.configure();
    if (!_logger.isLoggable(level)) {
      return;
    }

    final payload = <String, dynamic>{
      'event': event,
      if (message != null) 'message': message,
      if (context.isNotEmpty) 'context': context,
    };

    _logger.log(level, payload, error, stackTrace);
  }
}

/// Factory for obtaining a structured logger for a specific service/module.
StructuredLogger getLogger(String service, {Level? level, LogSink? sink}) {
  LoggingConfigurator.configure(level: level, sink: sink);
  return StructuredLogger._(Logger(service));
}
