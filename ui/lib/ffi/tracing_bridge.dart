/// FFI Tracing Bridge for Dart DevTools Timeline Integration.
///
/// This module provides:
/// - FFI trace event collection from Rust
/// - Conversion to Dart Timeline events
/// - Integration with Flutter DevTools Timeline
/// - Filtering and search capabilities
library;

import 'dart:async';
import 'dart:convert';
import 'dart:developer' as developer;
import 'dart:ffi' as ffi;

import 'package:ffi/ffi.dart';

import 'bindings.dart';

/// Result status for FFI trace events (must match Rust enum)
enum FfiResultStatus {
  /// Event is pending (not yet completed)
  pending(0),

  /// Event completed successfully
  success(1),

  /// Event completed with error
  error(2),

  /// Event was skipped
  skipped(3);

  const FfiResultStatus(this.value);
  final int value;

  static FfiResultStatus fromValue(int value) {
    return FfiResultStatus.values.firstWhere(
      (e) => e.value == value,
      orElse: () => FfiResultStatus.pending,
    );
  }
}

/// A trace event representing an FFI call
class FfiTraceEvent {
  /// Unique event ID
  final int id;

  /// Function name (e.g., "keyrx_engine_start")
  final String name;

  /// Domain (e.g., "engine", "device", "profile")
  final String domain;

  /// Start time in microseconds since UNIX epoch
  final int startTimeUs;

  /// Duration in microseconds (null if not yet completed)
  final int? durationUs;

  /// Correlation ID for tracking request flows
  final int correlationId;

  /// Sanitized arguments (sensitive data removed)
  final Map<String, String> args;

  /// Result status
  final FfiResultStatus resultStatus;

  FfiTraceEvent({
    required this.id,
    required this.name,
    required this.domain,
    required this.startTimeUs,
    this.durationUs,
    required this.correlationId,
    required this.args,
    required this.resultStatus,
  });

  /// Create from JSON
  factory FfiTraceEvent.fromJson(Map<String, dynamic> json) {
    return FfiTraceEvent(
      id: json['id'] as int,
      name: json['name'] as String,
      domain: json['domain'] as String,
      startTimeUs: json['start_time_us'] as int,
      durationUs: json['duration_us'] as int?,
      correlationId: json['correlation_id'] as int,
      args: Map<String, String>.from(json['args'] as Map),
      resultStatus: FfiResultStatus.fromValue(json['result_status'] as int),
    );
  }

  /// Convert to JSON
  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'name': name,
      'domain': domain,
      'start_time_us': startTimeUs,
      'duration_us': durationUs,
      'correlation_id': correlationId,
      'args': args,
      'result_status': resultStatus.value,
    };
  }

  @override
  String toString() {
    return 'FfiTraceEvent(id: $id, name: $name, domain: $domain, '
        'duration: ${durationUs != null ? '${durationUs}μs' : 'pending'}, '
        'status: $resultStatus)';
  }
}

/// Filter options for trace events
class FfiTraceFilter {
  /// Filter by domain (null for all domains)
  final String? domain;

  /// Minimum duration in microseconds (0 for no filter)
  final int minDurationUs;

  /// Filter by function name pattern (null for all functions)
  final String? functionPattern;

  /// Filter by result status (null for all statuses)
  final FfiResultStatus? resultStatus;

  const FfiTraceFilter({
    this.domain,
    this.minDurationUs = 0,
    this.functionPattern,
    this.resultStatus,
  });

  /// No filtering
  static const none = FfiTraceFilter();
}

/// FFI Tracing Bridge
///
/// Provides integration between Rust FFI trace events and Dart DevTools Timeline.
///
/// Usage:
/// ```dart
/// final bridge = FfiTracingBridge(bindings);
///
/// // Enable tracing
/// bridge.enable();
///
/// // Start polling for events
/// await bridge.startTimelineIntegration();
///
/// // FFI calls will now appear in DevTools Timeline
///
/// // Get events manually
/// final events = bridge.getEvents();
///
/// // Get filtered events
/// final slowEvents = bridge.getFilteredEvents(
///   FfiTraceFilter(minDurationUs: 1000),
/// );
///
/// // Disable when done
/// bridge.disable();
/// ```
class FfiTracingBridge {
  final KeyrxBindings _bindings;

  Timer? _pollingTimer;
  bool _timelineIntegrationActive = false;
  final Set<int> _processedEventIds = {};

  FfiTracingBridge(this._bindings);

  /// Enable FFI trace event collection
  void enable() {
    _bindings.keyrx_trace_enable();
  }

  /// Disable FFI trace event collection
  void disable() {
    _bindings.keyrx_trace_disable();
    stopTimelineIntegration();
  }

  /// Check if trace event collection is enabled
  bool isEnabled() {
    return _bindings.keyrx_trace_is_enabled();
  }

  /// Get the number of buffered trace events
  int getCount() {
    return _bindings.keyrx_trace_get_count();
  }

  /// Clear all buffered trace events
  void clear() {
    _bindings.keyrx_trace_clear();
    _processedEventIds.clear();
  }

  /// Get all trace events
  List<FfiTraceEvent> getEvents() {
    final jsonPtr = _bindings.keyrx_trace_get_events_json();
    if (jsonPtr == ffi.nullptr) {
      return [];
    }

    try {
      final jsonStr = jsonPtr.cast<Utf8>().toDartString();
      final List<dynamic> jsonList = json.decode(jsonStr) as List<dynamic>;
      return jsonList
          .map((e) => FfiTraceEvent.fromJson(e as Map<String, dynamic>))
          .toList();
    } finally {
      _bindings.keyrx_trace_free_json(jsonPtr);
    }
  }

  /// Get filtered trace events
  List<FfiTraceEvent> getFilteredEvents(FfiTraceFilter filter) {
    // Get domain filter as C string
    ffi.Pointer<ffi.Char>? domainPtr;
    if (filter.domain != null) {
      domainPtr = filter.domain!.toNativeUtf8().cast<ffi.Char>();
    }

    try {
      final jsonPtr = _bindings.keyrx_trace_get_filtered_events_json(
        domainPtr ?? ffi.nullptr,
        filter.minDurationUs,
      );

      if (jsonPtr == ffi.nullptr) {
        return [];
      }

      try {
        final jsonStr = jsonPtr.cast<Utf8>().toDartString();
        final List<dynamic> jsonList = json.decode(jsonStr) as List<dynamic>;
        var events = jsonList
            .map((e) => FfiTraceEvent.fromJson(e as Map<String, dynamic>))
            .toList();

        // Apply additional Dart-side filters
        if (filter.functionPattern != null) {
          final pattern = RegExp(filter.functionPattern!);
          events = events.where((e) => pattern.hasMatch(e.name)).toList();
        }

        if (filter.resultStatus != null) {
          events = events
              .where((e) => e.resultStatus == filter.resultStatus)
              .toList();
        }

        return events;
      } finally {
        _bindings.keyrx_trace_free_json(jsonPtr);
      }
    } finally {
      if (domainPtr != null) {
        malloc.free(domainPtr);
      }
    }
  }

  /// Start Timeline integration
  ///
  /// Polls for new FFI trace events and adds them to the Dart Timeline.
  /// Events will be visible in Flutter DevTools Timeline with category "FFI".
  ///
  /// Args:
  ///   pollInterval: How often to poll for events (default: 100ms)
  Future<void> startTimelineIntegration({
    Duration pollInterval = const Duration(milliseconds: 100),
  }) async {
    if (_timelineIntegrationActive) {
      return; // Already active
    }

    if (!isEnabled()) {
      throw StateError(
        'FFI tracing must be enabled before starting timeline integration. '
        'Call enable() first.',
      );
    }

    _timelineIntegrationActive = true;
    _processedEventIds.clear();

    // Start polling for events
    _pollingTimer = Timer.periodic(pollInterval, (_) {
      _pollAndAddToTimeline();
    });
  }

  /// Stop Timeline integration
  void stopTimelineIntegration() {
    _pollingTimer?.cancel();
    _pollingTimer = null;
    _timelineIntegrationActive = false;
    _processedEventIds.clear();
  }

  /// Poll for new events and add to Timeline
  void _pollAndAddToTimeline() {
    try {
      final events = getEvents();

      for (final event in events) {
        // Skip already processed events
        if (_processedEventIds.contains(event.id)) {
          continue;
        }

        _addEventToTimeline(event);
        _processedEventIds.add(event.id);
      }
    } catch (e) {
      // Log error but don't crash
      developer.log(
        'Error polling FFI trace events: $e',
        name: 'FfiTracingBridge',
        level: 900, // ERROR
      );
    }
  }

  /// Add a single event to the Dart Timeline
  void _addEventToTimeline(FfiTraceEvent event) {
    // Build arguments for Timeline
    final Map<String, dynamic> arguments = {
      'domain': event.domain,
      'correlation_id': event.correlationId,
      'status': event.resultStatus.name,
      ...event.args,
    };

    if (event.durationUs != null) {
      arguments['duration_us'] = event.durationUs;
    }

    // For completed events, use instantSync with duration
    if (event.durationUs != null) {
      developer.Timeline.instantSync(event.name, arguments: arguments);

      // Also emit a start/finish pair for better visualization
      developer.Timeline.startSync(event.name, arguments: arguments);
      // Simulate the duration by finishing immediately
      // (DevTools will show the instantSync timing)
      developer.Timeline.finishSync();
    } else {
      // For pending events, just use instantSync
      developer.Timeline.instantSync(event.name, arguments: arguments);
    }
  }

  /// Get statistics about trace events
  FfiTraceStats getStats() {
    final events = getEvents();

    if (events.isEmpty) {
      return FfiTraceStats.empty();
    }

    final Map<String, int> byDomain = {};
    final Map<FfiResultStatus, int> byStatus = {};
    int totalDurationUs = 0;
    int completedCount = 0;
    int? minDurationUs;
    int? maxDurationUs;

    for (final event in events) {
      // Count by domain
      byDomain[event.domain] = (byDomain[event.domain] ?? 0) + 1;

      // Count by status
      byStatus[event.resultStatus] = (byStatus[event.resultStatus] ?? 0) + 1;

      // Calculate duration stats
      if (event.durationUs != null) {
        completedCount++;
        totalDurationUs += event.durationUs!;

        if (minDurationUs == null || event.durationUs! < minDurationUs) {
          minDurationUs = event.durationUs;
        }
        if (maxDurationUs == null || event.durationUs! > maxDurationUs) {
          maxDurationUs = event.durationUs;
        }
      }
    }

    return FfiTraceStats(
      totalEvents: events.length,
      completedEvents: completedCount,
      pendingEvents: events.length - completedCount,
      byDomain: byDomain,
      byStatus: byStatus,
      avgDurationUs: completedCount > 0 ? totalDurationUs ~/ completedCount : 0,
      minDurationUs: minDurationUs,
      maxDurationUs: maxDurationUs,
    );
  }
}

/// Statistics about FFI trace events
class FfiTraceStats {
  final int totalEvents;
  final int completedEvents;
  final int pendingEvents;
  final Map<String, int> byDomain;
  final Map<FfiResultStatus, int> byStatus;
  final int avgDurationUs;
  final int? minDurationUs;
  final int? maxDurationUs;

  const FfiTraceStats({
    required this.totalEvents,
    required this.completedEvents,
    required this.pendingEvents,
    required this.byDomain,
    required this.byStatus,
    required this.avgDurationUs,
    this.minDurationUs,
    this.maxDurationUs,
  });

  factory FfiTraceStats.empty() {
    return const FfiTraceStats(
      totalEvents: 0,
      completedEvents: 0,
      pendingEvents: 0,
      byDomain: {},
      byStatus: {},
      avgDurationUs: 0,
    );
  }

  @override
  String toString() {
    return 'FfiTraceStats(\n'
        '  total: $totalEvents, completed: $completedEvents, pending: $pendingEvents\n'
        '  by domain: $byDomain\n'
        '  by status: $byStatus\n'
        '  duration: avg=${avgDurationUs}μs, min=$minDurationUs, max=$maxDurationUs\n'
        ')';
  }
}
