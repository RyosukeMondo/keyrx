/// Service for accessing structured logs from the KeyRx Core engine.
library;

import 'dart:async';
import 'dart:convert';

import '../ffi/bridge.dart'
    if (dart.library.html) '../ffi/bridge_stub.dart';
import '../models/log_entry.dart';

/// Abstraction for observability operations.
abstract class ObservabilityService {
  /// Initialize the log bridge.
  Future<bool> initialize();

  /// Fetch all buffered log entries and clear the buffer.
  Future<List<LogEntry>> drainLogs();

  /// Get the number of buffered log entries.
  Future<int> getLogCount();

  /// Clear all buffered log entries without returning them.
  Future<void> clearLogs();

  /// Enable or disable log buffering.
  Future<void> setLogEnabled({required bool enabled});
}

/// Implementation of ObservabilityService that wraps FFI bindings.
class ObservabilityServiceImpl implements ObservabilityService {
  ObservabilityServiceImpl({required KeyrxBridge bridge}) : _bridge = bridge;

  final KeyrxBridge _bridge;

  @override
  Future<bool> initialize() async {
    if (_bridge.loadFailure != null) {
      return false;
    }

    final bindings = _bridge.bindings;
    if (bindings == null) {
      return false;
    }

    try {
      final result = bindings.observability.logBridgeInit();
      return result == 0;
    } on Object catch (_) {
      return false;
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

    try {
      final jsonStr = bindings.observability.logDrain();
      final jsonList = jsonDecode(jsonStr) as List<dynamic>;
      return jsonList
          .map((entry) => LogEntry.fromJson(entry as Map<String, dynamic>))
          .toList();
    } on Exception {
      return const [];
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

    try {
      return bindings.observability.logCount();
    } on Object catch (_) {
      return 0;
    }
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

    try {
      bindings.observability.logClear();
    } on Object catch (_) {}
  }

  @override
  Future<void> setLogEnabled({required bool enabled}) async {
    if (_bridge.loadFailure != null) {
      return;
    }

    final bindings = _bridge.bindings;
    if (bindings == null) {
      return;
    }

    try {
      bindings.observability.logSetEnabled(enabled ? 1 : 0);
    } on Object catch (_) {}
  }
}
