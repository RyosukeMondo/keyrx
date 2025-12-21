/// Service for accessing performance metrics from the KeyRx Core engine.
///
/// Provides a Flutter-friendly API to fetch, cache, and stream real-time
/// performance metrics. This UI layer wraps core_logic MetricsService
/// and adds UI-specific features like polling and violation streams.
library;

import 'dart:async';

import 'package:core_logic/services/metrics_service.dart' as core;

export 'package:core_logic/services/metrics_service.dart'
    show MetricsSnapshot, MetricsThresholds;

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

/// UI-specific abstraction for metrics operations with polling and caching.
///
/// Wraps core MetricsService and adds UI-specific features:
/// - Polling-based metrics stream
/// - Snapshot caching
/// - Threshold violation stream
abstract class UIMetricsService {
  /// Stream of metrics snapshots (polled periodically).
  Stream<core.MetricsSnapshot> get metricsStream;

  /// Stream of threshold violations.
  Stream<ThresholdViolation> get violationStream;

  /// Last cached metrics snapshot.
  core.MetricsSnapshot? get cachedSnapshot;

  /// Fetch the current metrics snapshot.
  Future<core.MetricsSnapshot?> getSnapshot();

  /// Start streaming metrics updates.
  Future<void> startUpdates();

  /// Stop streaming metrics updates.
  Future<void> stopUpdates();

  /// Configure performance thresholds.
  Future<void> setThresholds(core.MetricsThresholds thresholds);

  /// Get current threshold configuration.
  Future<core.MetricsThresholds?> getThresholds();

  /// Dispose any held resources.
  Future<void> dispose();
}

/// UI MetricsService implementation that delegates to core service.
class UIMetricsServiceImpl implements UIMetricsService {
  UIMetricsServiceImpl({required core.MetricsService coreMetricsService})
      : _coreMetricsService = coreMetricsService;

  final core.MetricsService _coreMetricsService;
  final StreamController<core.MetricsSnapshot> _metricsController =
      StreamController<core.MetricsSnapshot>.broadcast();
  final StreamController<ThresholdViolation> _violationController =
      StreamController<ThresholdViolation>.broadcast();

  Timer? _pollingTimer;
  core.MetricsSnapshot? _cachedSnapshot;
  bool _isUpdating = false;

  @override
  Stream<core.MetricsSnapshot> get metricsStream => _metricsController.stream;

  @override
  Stream<ThresholdViolation> get violationStream => _violationController.stream;

  @override
  core.MetricsSnapshot? get cachedSnapshot => _cachedSnapshot;

  @override
  Future<core.MetricsSnapshot?> getSnapshot() async {
    final snapshot = await _coreMetricsService.getSnapshot();
    if (snapshot != null) {
      _cachedSnapshot = snapshot;
    }
    return snapshot;
  }

  @override
  Future<void> startUpdates() async {
    if (_isUpdating) {
      return;
    }

    await _coreMetricsService.startUpdates();
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

    await _coreMetricsService.stopUpdates();

    _pollingTimer?.cancel();
    _pollingTimer = null;
    _isUpdating = false;
  }

  @override
  Future<void> setThresholds(core.MetricsThresholds thresholds) async {
    await _coreMetricsService.setThresholds(thresholds);
  }

  @override
  Future<core.MetricsThresholds?> getThresholds() async =>
      _coreMetricsService.getThresholds();

  @override
  Future<void> dispose() async {
    await stopUpdates();
    await _metricsController.close();
    await _violationController.close();
    _cachedSnapshot = null;
  }
}
