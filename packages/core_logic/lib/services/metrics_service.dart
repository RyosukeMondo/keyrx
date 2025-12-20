/// Service for accessing performance metrics from the KeyRx Core engine.
library;

import 'dart:async';
import 'dart:convert';

import '../ffi/bridge.dart'
    if (dart.library.html) '../ffi/bridge_stub.dart';

/// Performance metrics snapshot from the engine.
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

  factory MetricsSnapshot.fromJson(Map<String, dynamic> json) =>
      MetricsSnapshot(
        timestamp: json['timestamp'] as int,
        eventLatencyP50: json['event_latency_p50'] as int,
        eventLatencyP95: json['event_latency_p95'] as int,
        eventLatencyP99: json['event_latency_p99'] as int,
        eventsProcessed: json['events_processed'] as int,
        errorsCount: json['errors_count'] as int,
        memoryUsed: json['memory_used'] as int,
      );

  final int timestamp;
  final int eventLatencyP50; // microseconds
  final int eventLatencyP95; // microseconds
  final int eventLatencyP99; // microseconds
  final int eventsProcessed;
  final int errorsCount;
  final int memoryUsed; // bytes

  DateTime get dateTime =>
      DateTime.fromMillisecondsSinceEpoch(timestamp, isUtc: true);

  Map<String, dynamic> toJson() => {
        'timestamp': timestamp,
        'event_latency_p50': eventLatencyP50,
        'event_latency_p95': eventLatencyP95,
        'event_latency_p99': eventLatencyP99,
        'events_processed': eventsProcessed,
        'errors_count': errorsCount,
        'memory_used': memoryUsed,
      };
}

/// Configuration for performance thresholds.
class MetricsThresholds {
  const MetricsThresholds({
    required this.latencyWarningUs,
    required this.latencyErrorUs,
    required this.memoryWarningBytes,
    required this.memoryErrorBytes,
  });

  /// Default thresholds: 50us warn, 100us error, 100MB warn, 500MB error.
  factory MetricsThresholds.defaults() => const MetricsThresholds(
        latencyWarningUs: 50,
        latencyErrorUs: 100,
        memoryWarningBytes: 100 * 1024 * 1024,
        memoryErrorBytes: 500 * 1024 * 1024,
      );

  final int latencyWarningUs;
  final int latencyErrorUs;
  final int memoryWarningBytes;
  final int memoryErrorBytes;
}

/// Abstraction for metrics operations.
abstract class MetricsService {
  /// Fetch the current metrics snapshot.
  Future<MetricsSnapshot?> getSnapshot();

  /// Start streaming metrics updates.
  Future<void> startUpdates();

  /// Stop streaming metrics updates.
  Future<void> stopUpdates();

  /// Configure performance thresholds.
  Future<void> setThresholds(MetricsThresholds thresholds);

  /// Get current threshold configuration.
  Future<MetricsThresholds?> getThresholds();
}

/// Implementation of MetricsService that wraps FFI bindings.
class MetricsServiceImpl implements MetricsService {
  MetricsServiceImpl({required KeyrxBridge bridge}) : _bridge = bridge;

  final KeyrxBridge _bridge;

  @override
  Future<MetricsSnapshot?> getSnapshot() async {
    if (_bridge.loadFailure != null) {
      return null;
    }

    final bindings = _bridge.bindings;
    if (bindings == null) {
      return null;
    }

    try {
      final jsonStr = bindings.observability.metricsSnapshotJson();
      final jsonPayload = jsonStr.startsWith('ok:')
          ? jsonStr.substring(3)
          : jsonStr.startsWith('error:')
              ? throw Exception(jsonStr)
              : jsonStr;

      final json = jsonDecode(jsonPayload) as Map<String, dynamic>;
      return MetricsSnapshot.fromJson(json);
    } on Exception {
      return null;
    }
  }

  @override
  Future<void> startUpdates() async {
    if (_bridge.loadFailure != null) {
      return;
    }

    final bindings = _bridge.bindings;
    if (bindings == null) {
      return;
    }

    try {
      bindings.observability.metricsStartUpdates();
    } on Object catch (_) {}
  }

  @override
  Future<void> stopUpdates() async {
    if (_bridge.loadFailure != null) {
      return;
    }

    final bindings = _bridge.bindings;
    if (bindings == null) {
      return;
    }

    try {
      bindings.observability.metricsStopUpdates();
    } on Object catch (_) {}
  }

  @override
  Future<void> setThresholds(MetricsThresholds thresholds) async {
    if (_bridge.loadFailure != null) {
      return;
    }

    final bindings = _bridge.bindings;
    if (bindings == null) {
      return;
    }

    try {
      bindings.observability.metricsSetThresholds(
        thresholds.latencyWarningUs,
        thresholds.latencyErrorUs,
        thresholds.memoryWarningBytes,
        thresholds.memoryErrorBytes,
      );
    } on Object catch (_) {}
  }

  @override
  Future<MetricsThresholds?> getThresholds() async {
    if (_bridge.loadFailure != null) {
      return null;
    }

    final bindings = _bridge.bindings;
    if (bindings == null) {
      return null;
    }

    try {
      // Note: FFI getThresholds requires out-pointers, which is complex.
      // For now, we return null and rely on the default thresholds.
      // This can be enhanced if needed.
      return null;
    } on Object catch (_) {
      return null;
    }
  }
}
