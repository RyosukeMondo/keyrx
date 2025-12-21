/// Observability service for accessing logs and metrics.
///
/// Provides a Flutter-friendly API to access structured logs and performance
/// metrics from the KeyRx Core engine through FFI.
library;

import 'dart:async';

import 'package:core_logic/models/log_entry.dart' as core;
import 'package:core_logic/services/metrics_service.dart';
import 'package:core_logic/services/observability_service.dart' as core_observability;

/// Log entry from the engine for UI display.
///
/// This is a UI-specific log entry model used by the observability service
/// to display logs in the UI. It is separate from core_logic's LogEntry model.
class ObservabilityLogEntry {
  const ObservabilityLogEntry({
    required this.timestamp,
    required this.level,
    required this.target,
    required this.message,
    this.fields = const {},
    this.span,
  });

  factory ObservabilityLogEntry.fromJson(Map<String, dynamic> json) => ObservabilityLogEntry(
    timestamp: json['timestamp'] as int,
    level: _parseLogLevel(json['level'] as String),
    target: json['target'] as String,
    message: json['message'] as String,
    fields: json['fields'] != null
        ? Map<String, String>.from(json['fields'] as Map)
        : const {},
    span: json['span'] as String?,
  );

  /// Convert from core LogEntry to UI ObservabilityLogEntry.
  factory ObservabilityLogEntry.fromCoreLogEntry(core.LogEntry coreEntry) => ObservabilityLogEntry(
    timestamp: coreEntry.timestamp.millisecondsSinceEpoch,
    level: _parseLogLevel(coreEntry.level),
    target: coreEntry.target,
    message: coreEntry.message,
    fields: coreEntry.fields.map<String, String>((key, value) => MapEntry(key, value.toString())),
  );

  final int timestamp;
  final LogLevel level;
  final String target;
  final String message;
  final Map<String, String> fields;
  final String? span;

  DateTime get dateTime =>
      DateTime.fromMillisecondsSinceEpoch(timestamp, isUtc: true);

  static LogLevel _parseLogLevel(String level) => switch (level.toUpperCase()) {
    'TRACE' => LogLevel.trace,
    'DEBUG' => LogLevel.debug,
    'INFO' => LogLevel.info,
    'WARN' => LogLevel.warn,
    'ERROR' => LogLevel.error,
    _ => LogLevel.info,
  };
}

/// Log severity level.
enum LogLevel {
  trace,
  debug,
  info,
  warn,
  error;

  bool operator >=(LogLevel other) => index >= other.index;
  bool operator <=(LogLevel other) => index <= other.index;
}

/// UI-specific abstraction for observability operations with polling streams.
///
/// Wraps core_logic ObservabilityService and adds UI-specific polling features.
abstract class UIObservabilityService {
  /// Stream of log entries.
  ///
  /// Emits new log entries as they are captured by the log bridge.
  Stream<List<ObservabilityLogEntry>> get logStream;

  /// Stream of metrics snapshots.
  ///
  /// Emits periodic metrics updates when metrics updates are enabled.
  Stream<MetricsSnapshot> get metricsStream;

  /// Initialize the observability service.
  Future<bool> initialize();

  /// Fetch all buffered log entries and clear the buffer.
  Future<List<ObservabilityLogEntry>> drainLogs();

  /// Get the number of buffered log entries.
  Future<int> getLogCount();

  /// Clear all buffered log entries without returning them.
  Future<void> clearLogs();

  /// Enable or disable log buffering.
  Future<void> setLogEnabled({required bool enabled});

  /// Fetch the current metrics snapshot.
  Future<MetricsSnapshot?> getMetrics();

  /// Start background metrics updates.
  ///
  /// When enabled, the metrics stream will emit periodic updates.
  Future<void> startMetricsUpdates();

  /// Stop background metrics updates.
  Future<void> stopMetricsUpdates();

  /// Dispose any held resources.
  Future<void> dispose();
}

/// UI ObservabilityService implementation that wraps core ObservabilityService with polling.
class UIObservabilityServiceImpl implements UIObservabilityService {
  UIObservabilityServiceImpl({
    required core_observability.ObservabilityService coreObservabilityService,
    required MetricsService metricsService,
  })  : _coreObservabilityService = coreObservabilityService,
        _metricsService = metricsService;

  final core_observability.ObservabilityService _coreObservabilityService;
  final MetricsService _metricsService;
  final StreamController<List<ObservabilityLogEntry>> _logController =
      StreamController<List<ObservabilityLogEntry>>.broadcast();
  final StreamController<MetricsSnapshot> _metricsController =
      StreamController<MetricsSnapshot>.broadcast();

  Timer? _logPollingTimer;
  Timer? _metricsPollingTimer;
  bool _initialized = false;

  @override
  Stream<List<ObservabilityLogEntry>> get logStream => _logController.stream;

  @override
  Stream<MetricsSnapshot> get metricsStream => _metricsController.stream;

  @override
  Future<bool> initialize() async {
    if (_initialized) {
      return true;
    }

    // Initialize the core observability service
    final result = await _coreObservabilityService.initialize();
    if (!result) {
      return false;
    }

    // Start polling for logs periodically
    _logPollingTimer = Timer.periodic(
      const Duration(milliseconds: 100),
      (_) => _pollLogs(),
    );

    _initialized = true;
    return true;
  }

  Future<void> _pollLogs() async {
    final count = await getLogCount();
    if (count > 0) {
      final logs = await drainLogs();
      if (logs.isNotEmpty) {
        _logController.add(logs);
      }
    }
  }

  @override
  Future<List<ObservabilityLogEntry>> drainLogs() async {
    final coreLogs = await _coreObservabilityService.drainLogs();
    return coreLogs.map(ObservabilityLogEntry.fromCoreLogEntry).toList();
  }

  @override
  Future<int> getLogCount() async => _coreObservabilityService.getLogCount();

  @override
  Future<void> clearLogs() async => _coreObservabilityService.clearLogs();

  @override
  Future<void> setLogEnabled({required bool enabled}) async =>
      _coreObservabilityService.setLogEnabled(enabled: enabled);

  @override
  Future<MetricsSnapshot?> getMetrics() async {
    return _metricsService.getSnapshot();
  }

  @override
  Future<void> startMetricsUpdates() async {
    await _metricsService.startUpdates();

    // Start polling for metrics periodically
    _metricsPollingTimer = Timer.periodic(
      const Duration(seconds: 1),
      (_) => _pollMetrics(),
    );
  }

  Future<void> _pollMetrics() async {
    final snapshot = await getMetrics();
    if (snapshot != null) {
      _metricsController.add(snapshot);
    }
  }

  @override
  Future<void> stopMetricsUpdates() async {
    await _metricsService.stopUpdates();

    _metricsPollingTimer?.cancel();
    _metricsPollingTimer = null;
  }

  @override
  Future<void> dispose() async {
    _logPollingTimer?.cancel();
    _metricsPollingTimer?.cancel();

    await stopMetricsUpdates();
    await _logController.close();
    await _metricsController.close();

    _initialized = false;
  }
}
