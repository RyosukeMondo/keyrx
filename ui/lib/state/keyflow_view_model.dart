/// View model for the KeyFlow viewer.
///
/// Manages loading snapshots, runtime overlay updates, selection state,
/// and staleness tracking for UI consumers.
library;

import 'dart:async';

import 'package:flutter/foundation.dart';

import '../services/facade/keyflow_repository.dart';
import '../services/keyflow_telemetry.dart';

class KeyFlowViewModel extends ChangeNotifier {
  KeyFlowViewModel({
    required KeyFlowRepository repository,
    KeyFlowTelemetry? telemetry,
    Duration runtimeStaleAfter = const Duration(seconds: 5),
    Duration refreshDebounce = const Duration(milliseconds: 200),
    DateTime Function()? now,
  }) : _repository = repository,
       _telemetry = telemetry,
       _runtimeStaleAfter = runtimeStaleAfter,
       _refreshDebounce = refreshDebounce,
       _now = now ?? DateTime.now {
    _runtimeSub = _repository.runtimeStream().listen(
      _handleRuntimeOverlay,
      onError: (_) {
        // Ignore runtime stream errors; stale flag will inform UI.
      },
    );
  }

  final KeyFlowRepository _repository;
  final KeyFlowTelemetry? _telemetry;
  final Duration _runtimeStaleAfter;
  final Duration _refreshDebounce;
  final DateTime Function() _now;

  KeyFlowSnapshot? _snapshot;
  String? _selectedDeviceId;
  String? _selectedKeyId;
  RuntimeOverlay? _runtimeOverlay;
  bool _runtimeStale = true;

  bool _isLoading = false;
  bool _isRefreshing = false;
  bool _disposed = false;
  String? _loadError;
  int? _lastRenderStampMs;

  StreamSubscription<RuntimeOverlay>? _runtimeSub;
  Timer? _runtimeStaleTimer;
  Timer? _refreshTimer;
  Completer<void>? _pendingRefresh;

  /// Current snapshot.
  KeyFlowSnapshot? get snapshot => _snapshot;

  /// Selected device if present in the snapshot.
  KeyFlowDevice? get selectedDevice => _findDevice(_selectedDeviceId);

  /// Selected key identifier for the current device context.
  String? get selectedKeyId => _selectedKeyId;

  /// Aggregated wiring gaps across devices.
  int get totalGaps {
    return _snapshot?.devices.fold<int>(0, (sum, d) => sum + d.gaps) ?? 0;
  }

  /// Aggregated wiring conflicts across devices.
  int get totalConflicts {
    return _snapshot?.devices.fold<int>(0, (sum, d) => sum + d.conflicts) ?? 0;
  }

  /// Whether the view model is currently loading.
  bool get isLoading => _isLoading;

  /// Whether a debounced refresh is executing.
  bool get isRefreshing => _isRefreshing;

  /// Whether runtime overlay data is stale (>5s silence by default).
  bool get isRuntimeStale => _runtimeStale || runtimeOverlay == null;

  /// Latest runtime overlay (merged from snapshot and stream).
  RuntimeOverlay? get runtimeOverlay => _runtimeOverlay ?? _snapshot?.runtime;

  /// Timestamp of the last runtime event.
  DateTime? get lastRuntimeUpdate => runtimeOverlay?.lastUpdate;

  /// User-safe errors collected from repository calls.
  List<KeyFlowError> get errors {
    final base = List<KeyFlowError>.from(_snapshot?.errors ?? const []);
    if (_loadError != null) {
      base.add(KeyFlowError(source: 'view-model', message: _loadError!));
    }
    return base;
  }

  /// Telemetry summary for CLI-friendly automation loops.
  String get telemetrySummary =>
      _telemetry?.cliSummary() ?? 'telemetry-disabled';

  KeyFlowTelemetrySnapshot? get telemetrySnapshot => _telemetry?.snapshot();

  /// Load the full KeyFlow snapshot.
  Future<void> load() async {
    _isLoading = true;
    _notify();
    try {
      final result = await _repository.loadAll();
      _loadError = null;
      _applySnapshot(result);
    } catch (e) {
      _loadError = 'Failed to load KeyFlow data: $e';
    } finally {
      _isLoading = false;
      _notify();
    }
  }

  /// Refresh devices with cached config, debounced to avoid UI thrash.
  Future<void> refresh() {
    if (_pendingRefresh != null) {
      return _pendingRefresh!.future;
    }

    _refreshTimer?.cancel();
    _pendingRefresh = Completer<void>();
    _refreshTimer = Timer(_refreshDebounce, () async {
      _refreshTimer = null;
      _isRefreshing = true;
      _notify();
      try {
        final result = await _repository.refreshDevices();
        _loadError = null;
        _applySnapshot(result);
      } catch (e) {
        _loadError = 'Failed to refresh KeyFlow data: $e';
      } finally {
        _isRefreshing = false;
        _notify();
        if (!(_pendingRefresh?.isCompleted ?? true)) {
          _pendingRefresh?.complete();
        }
        _pendingRefresh = null;
      }
    });
    return _pendingRefresh!.future;
  }

  /// Select a device by ID; no-ops if not found.
  void selectDevice(String? deviceId) {
    _selectedDeviceId = deviceId;
    _reconcileSelection();
    _notify();
  }

  /// Select a virtual key identifier for details panes.
  void selectKey(String? keyId) {
    _selectedKeyId = keyId;
    _notify();
  }

  @override
  Future<void> dispose() async {
    _disposed = true;
    _refreshTimer?.cancel();
    _runtimeStaleTimer?.cancel();
    if (!(_pendingRefresh?.isCompleted ?? true)) {
      _pendingRefresh?.complete();
    }
    await _runtimeSub?.cancel();
    await _repository.dispose();
    super.dispose();
  }

  void _applySnapshot(KeyFlowSnapshot snapshot) {
    _snapshot = snapshot;
    _runtimeOverlay = snapshot.runtime ?? _runtimeOverlay;
    _reconcileSelection();
    _updateStaleness();
  }

  void _reconcileSelection() {
    final devices = _snapshot?.devices ?? const [];
    if (_selectedDeviceId != null &&
        devices.every((d) => d.id != _selectedDeviceId)) {
      _selectedDeviceId = devices.isNotEmpty ? devices.first.id : null;
    } else if (_selectedDeviceId == null && devices.isNotEmpty) {
      _selectedDeviceId = devices.first.id;
    }
  }

  void _handleRuntimeOverlay(RuntimeOverlay overlay) {
    _runtimeOverlay = overlay;
    _snapshot = _snapshot == null
        ? null
        : KeyFlowSnapshot(
            devices: _snapshot!.devices,
            profiles: _snapshot!.profiles,
            layouts: _snapshot!.layouts,
            keymaps: _snapshot!.keymaps,
            runtime: overlay,
            runtimeConfig: _snapshot!.runtimeConfig,
            generatedAt: _snapshot!.generatedAt,
            errors: _snapshot!.errors,
          );
    _runtimeStale = false;
    _updateStaleness();
    _notify();
  }

  void _updateStaleness() {
    _runtimeStaleTimer?.cancel();
    final overlay = runtimeOverlay;
    if (overlay == null) {
      _runtimeStale = true;
      return;
    }

    final elapsed = _now().difference(overlay.lastUpdate);
    if (elapsed >= _runtimeStaleAfter) {
      _runtimeStale = true;
      return;
    }

    _runtimeStale = false;
    final delay = _runtimeStaleAfter - elapsed;
    _runtimeStaleTimer = Timer(delay, () {
      _runtimeStale = true;
      _notify();
    });
  }

  /// Record render durations for large layouts; no-ops when telemetry disabled.
  void recordRender(
    Duration duration, {
    required int keyCount,
    required int layerCount,
  }) {
    final snapshot = _snapshot;
    if (_telemetry == null || snapshot == null) {
      return;
    }
    final renderStamp = snapshot.generatedAt.millisecondsSinceEpoch;
    if (_lastRenderStampMs == renderStamp) {
      return;
    }
    _lastRenderStampMs = renderStamp;
    _telemetry!.recordRender(
      duration: duration,
      keyCount: keyCount,
      layerCount: layerCount,
    );
  }

  void _notify() {
    if (!_disposed) {
      notifyListeners();
    }
  }

  KeyFlowDevice? _findDevice(String? id) {
    if (id == null || _snapshot == null) return null;
    for (final device in _snapshot!.devices) {
      if (device.id == id) return device;
    }
    return null;
  }
}
