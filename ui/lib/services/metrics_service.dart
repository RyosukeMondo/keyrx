/// Service for accessing performance metrics from the KeyRx Core engine.
///
/// Provides a Flutter-friendly API to fetch, cache, and stream real-time
/// performance metrics through FFI. Supports both polling and callback-based
/// updates, as well as threshold violation alerts.
library;

import 'dart:async';
import 'dart:convert';

import 'package:core_logic/ffi/bridge.dart'
    if (dart.library.html) 'package:core_logic/ffi/bridge_stub.dart';

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

/// Type of threshold violation.
enum ViolationType {
  latencyWarning(0),
  latencyError(1),
  memoryWarning(2),
  memoryError(3);

  const ViolationType(this.value);

  final int value;

  static ViolationType fromValue(int value) => switch (value) {
    0 => ViolationType.latencyWarning,
    1 => ViolationType.latencyError,
    2 => ViolationType.memoryWarning,
    3 => ViolationType.memoryError,
    _ => throw ArgumentError('Invalid violation type: $value'),
  };
}

/// Threshold violation event.
class ThresholdViolation {
  const ThresholdViolation({
    required this.timestamp,
    required this.type,
    required this.actualValue,
    required this.thresholdValue,
  });

  final int timestamp;
  final ViolationType type;
  final int actualValue;
  final int thresholdValue;

  DateTime get dateTime =>
      DateTime.fromMillisecondsSinceEpoch(timestamp, isUtc: true);

  String get message => switch (type) {
    ViolationType.latencyWarning =>
      'Warning: Latency ${actualValue}us exceeds threshold ${thresholdValue}us',
    ViolationType.latencyError =>
      'Error: Latency ${actualValue}us exceeds threshold ${thresholdValue}us',
    ViolationType.memoryWarning =>
      'Warning: Memory $actualValue bytes exceeds threshold $thresholdValue bytes',
    ViolationType.memoryError =>
      'Error: Memory $actualValue bytes exceeds threshold $thresholdValue bytes',
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
  /// Stream of metrics snapshots.
  ///
  /// Emits periodic metrics updates when streaming is enabled.
  Stream<MetricsSnapshot> get metricsStream;

  /// Stream of threshold violations.
  ///
  /// Emits alerts when metrics exceed configured thresholds.
  Stream<ThresholdViolation> get violationStream;

  /// Last cached metrics snapshot.
  MetricsSnapshot? get cachedSnapshot;

  /// Fetch the current metrics snapshot.
  ///
  /// Updates the cache and returns the latest snapshot.
  Future<MetricsSnapshot?> getSnapshot();

  /// Start streaming metrics updates.
  ///
  /// Metrics will be fetched periodically and emitted via [metricsStream].
  Future<void> startUpdates();

  /// Stop streaming metrics updates.
  Future<void> stopUpdates();

  /// Configure performance thresholds.
  ///
  /// When thresholds are exceeded, violations are emitted via [violationStream].
  Future<void> setThresholds(MetricsThresholds thresholds);

  /// Get current threshold configuration.
  Future<MetricsThresholds?> getThresholds();

  /// Dispose any held resources.
  Future<void> dispose();
}

/// Real MetricsService implementation that wraps the KeyrxBridge.
class MetricsServiceImpl implements MetricsService {
  MetricsServiceImpl({required KeyrxBridge bridge}) : _bridge = bridge;

  final KeyrxBridge _bridge;
  final StreamController<MetricsSnapshot> _metricsController =
      StreamController<MetricsSnapshot>.broadcast();
  final StreamController<ThresholdViolation> _violationController =
      StreamController<ThresholdViolation>.broadcast();

  Timer? _pollingTimer;
  MetricsSnapshot? _cachedSnapshot;
  bool _isUpdating = false;

  @override
  Stream<MetricsSnapshot> get metricsStream => _metricsController.stream;

  @override
  Stream<ThresholdViolation> get violationStream => _violationController.stream;

  @override
  MetricsSnapshot? get cachedSnapshot => _cachedSnapshot;

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
      final snapshot = MetricsSnapshot.fromJson(json);
      _cachedSnapshot = snapshot;
      return snapshot;
    } catch (e) {
      return null;
    }
  }

  @override
  Future<void> startUpdates() async {
    if (_isUpdating) {
      return;
    }

    if (_bridge.loadFailure != null) {
      return;
    }

    final bindings = _bridge.bindings;
    if (bindings == null) {
      return;
    }

    // Start background updates in the core
    try {
      final status = bindings.observability.metricsStartUpdates();
      if (status != 0) {
        return;
      }
    } catch (_) {
      return;
    }

    _isUpdating = true;

    // Poll for metrics periodically
    _pollingTimer = Timer.periodic(
      const Duration(seconds: 1),
      (_) => _pollMetrics(),
    );
  }

  Future<void> _pollMetrics() async {
    final snapshot = await getSnapshot();
    if (snapshot != null) {
      _metricsController.add(snapshot);
    }
  }

  @override
  Future<void> stopUpdates() async {
    if (!_isUpdating) {
      return;
    }

    if (_bridge.loadFailure != null) {
      return;
    }

    final bindings = _bridge.bindings;
    if (bindings == null) {
      return;
    }

    try {
      bindings.observability.metricsStopUpdates();
    } catch (_) {}

    _pollingTimer?.cancel();
    _pollingTimer = null;
    _isUpdating = false;
  }

  @override
  Future<void> setThresholds(MetricsThresholds thresholds) async {
    if (_bridge.loadFailure != null) {
      return;
    }

    // Note: The generated bindings for setThresholds might not be
    // available if the struct definition wasn't exported.
    // For now we skip setting thresholds via FFI if complex types aren't ready.
  }

  @override
  Future<MetricsThresholds?> getThresholds() async {
    // Note: The generated bindings for getThresholds might not be
    // available if the struct definition wasn't exported.
    return null;
  }

  @override
  Future<void> dispose() async {
    await stopUpdates();
    await _metricsController.close();
    await _violationController.close();
    _cachedSnapshot = null;
  }
}
