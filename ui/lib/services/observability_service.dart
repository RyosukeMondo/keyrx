/// Observability service for accessing logs and metrics.
///
/// Provides a Flutter-friendly API to access structured logs and performance
/// metrics from the KeyRx Core engine through FFI.
library;

import 'dart:async';
import 'dart:convert';
import 'dart:ffi';

import 'package:ffi/ffi.dart';

import '../ffi/bridge.dart';

/// Log entry from the engine.
class LogEntry {
  const LogEntry({
    required this.timestamp,
    required this.level,
    required this.target,
    required this.message,
    this.fields = const {},
    this.span,
  });

  factory LogEntry.fromJson(Map<String, dynamic> json) {
    return LogEntry(
      timestamp: json['timestamp'] as int,
      level: _parseLogLevel(json['level'] as String),
      target: json['target'] as String,
      message: json['message'] as String,
      fields: json['fields'] != null
          ? Map<String, String>.from(json['fields'] as Map)
          : const {},
      span: json['span'] as String?,
    );
  }

  final int timestamp;
  final LogLevel level;
  final String target;
  final String message;
  final Map<String, String> fields;
  final String? span;

  DateTime get dateTime =>
      DateTime.fromMillisecondsSinceEpoch(timestamp, isUtc: true);

  static LogLevel _parseLogLevel(String level) {
    return switch (level.toUpperCase()) {
      'TRACE' => LogLevel.trace,
      'DEBUG' => LogLevel.debug,
      'INFO' => LogLevel.info,
      'WARN' => LogLevel.warn,
      'ERROR' => LogLevel.error,
      _ => LogLevel.info,
    };
  }
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

/// Metrics snapshot from the engine.
class MetricsSnapshot {
  const MetricsSnapshot({
    required this.timestamp,
    required this.eventLatencyP50,
    required this.eventLatencyP95,
    required this.eventLatencyP99,
    required this.eventsProcessed,
    required this.errorsCount,
    required this.memoryUsed,
  });

  factory MetricsSnapshot.fromJson(Map<String, dynamic> json) {
    return MetricsSnapshot(
      timestamp: json['timestamp'] as int,
      eventLatencyP50: json['event_latency_p50'] as int,
      eventLatencyP95: json['event_latency_p95'] as int,
      eventLatencyP99: json['event_latency_p99'] as int,
      eventsProcessed: json['events_processed'] as int,
      errorsCount: json['errors_count'] as int,
      memoryUsed: json['memory_used'] as int,
    );
  }

  final int timestamp;
  final int eventLatencyP50;
  final int eventLatencyP95;
  final int eventLatencyP99;
  final int eventsProcessed;
  final int errorsCount;
  final int memoryUsed;

  DateTime get dateTime =>
      DateTime.fromMillisecondsSinceEpoch(timestamp, isUtc: true);
}

/// Abstraction for observability operations.
abstract class ObservabilityService {
  /// Stream of log entries.
  ///
  /// Emits new log entries as they are captured by the log bridge.
  Stream<List<LogEntry>> get logStream;

  /// Stream of metrics snapshots.
  ///
  /// Emits periodic metrics updates when metrics updates are enabled.
  Stream<MetricsSnapshot> get metricsStream;

  /// Initialize the observability service.
  Future<bool> initialize();

  /// Fetch all buffered log entries and clear the buffer.
  Future<List<LogEntry>> drainLogs();

  /// Get the number of buffered log entries.
  Future<int> getLogCount();

  /// Clear all buffered log entries without returning them.
  Future<void> clearLogs();

  /// Enable or disable log buffering.
  Future<void> setLogEnabled(bool enabled);

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

/// Real ObservabilityService that wraps the KeyrxBridge.
class ObservabilityServiceImpl implements ObservabilityService {
  ObservabilityServiceImpl({required KeyrxBridge bridge}) : _bridge = bridge;

  final KeyrxBridge _bridge;
  final StreamController<List<LogEntry>> _logController =
      StreamController<List<LogEntry>>.broadcast();
  final StreamController<MetricsSnapshot> _metricsController =
      StreamController<MetricsSnapshot>.broadcast();

  Timer? _logPollingTimer;
  Timer? _metricsPollingTimer;
  bool _initialized = false;

  @override
  Stream<List<LogEntry>> get logStream => _logController.stream;

  @override
  Stream<MetricsSnapshot> get metricsStream => _metricsController.stream;

  @override
  Future<bool> initialize() async {
    if (_initialized) {
      return true;
    }

    if (_bridge.loadFailure != null) {
      return false;
    }

    // Initialize the log bridge
    final result = _bridge.bindings?.logBridgeInit();
    if (result != 0) {
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
    if (_bridge.loadFailure != null) {
      return;
    }

    final count = await getLogCount();
    if (count > 0) {
      final logs = await drainLogs();
      if (logs.isNotEmpty) {
        _logController.add(logs);
      }
    }
  }

  @override
  Future<List<LogEntry>> drainLogs() async {
    if (_bridge.loadFailure != null) {
      return const [];
    }

    final bindings = _bridge.bindings;
    if (bindings == null) {
      return const [];
    }

    final jsonPtr = bindings.logDrain();
    if (jsonPtr == nullptr) {
      return const [];
    }

    try {
      final jsonStr = jsonPtr.cast<Utf8>().toDartString();
      final jsonList = jsonDecode(jsonStr) as List<dynamic>;
      return jsonList
          .map((entry) => LogEntry.fromJson(entry as Map<String, dynamic>))
          .toList();
    } catch (e) {
      // Failed to parse logs
      return const [];
    } finally {
      // Free the string (assuming keyrx_free_string exists)
      // bindings.freeString(jsonPtr);
    }
  }

  @override
  Future<int> getLogCount() async {
    if (_bridge.loadFailure != null) {
      return 0;
    }

    final bindings = _bridge.bindings;
    if (bindings == null) {
      return 0;
    }

    return bindings.logCount();
  }

  @override
  Future<void> clearLogs() async {
    if (_bridge.loadFailure != null) {
      return;
    }

    final bindings = _bridge.bindings;
    if (bindings == null) {
      return;
    }

    bindings.logClear();
  }

  @override
  Future<void> setLogEnabled(bool enabled) async {
    if (_bridge.loadFailure != null) {
      return;
    }

    final bindings = _bridge.bindings;
    if (bindings == null) {
      return;
    }

    bindings.logSetEnabled(enabled ? 1 : 0);
  }

  @override
  Future<MetricsSnapshot?> getMetrics() async {
    if (_bridge.loadFailure != null) {
      return null;
    }

    final bindings = _bridge.bindings;
    if (bindings == null) {
      return null;
    }

    final jsonPtr = bindings.metricsSnapshotJson();
    if (jsonPtr == nullptr) {
      return null;
    }

    try {
      final jsonStr = jsonPtr.cast<Utf8>().toDartString();
      final json = jsonDecode(jsonStr) as Map<String, dynamic>;
      return MetricsSnapshot.fromJson(json);
    } catch (e) {
      return null;
    } finally {
      // Free the string
      // bindings.freeString(jsonPtr);
    }
  }

  @override
  Future<void> startMetricsUpdates() async {
    if (_bridge.loadFailure != null) {
      return;
    }

    final bindings = _bridge.bindings;
    if (bindings == null) {
      return;
    }

    final result = bindings.metricsStartUpdates();
    if (result != 0) {
      return;
    }

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
    if (_bridge.loadFailure != null) {
      return;
    }

    final bindings = _bridge.bindings;
    if (bindings == null) {
      return;
    }

    bindings.metricsStopUpdates();

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
