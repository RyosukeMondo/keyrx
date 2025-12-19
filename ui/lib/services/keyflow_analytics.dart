/// Analytics service for tracking KeyFlow usage and events.
library;

import 'dart:async';

import 'package:core_logic/utils/logger.dart';

/// Event types for KeyFlow analytics.
enum KeyFlowEventType {
  /// Page opened
  pageView,

  /// Slot added to device
  slotAdded,

  /// Slot deleted from device
  slotDeleted,

  /// Slot priority reordered
  slotReordered,

  /// Node dragged on canvas
  nodeDragged,

  /// Error occurred
  errorOccurred,

  /// Edge selected/interacted
  edgeInteracted,

  /// Dialog opened
  dialogOpened,

  /// Keyboard shortcut used
  keyboardShortcut,

  /// Canvas zoomed/panned
  canvasManipulated,

  /// Auto-save triggered
  autoSaveTriggered,
}

/// Analytics event data.
class KeyFlowEvent {
  const KeyFlowEvent({
    required this.type,
    required this.timestamp,
    this.context = const {},
  });

  final KeyFlowEventType type;
  final DateTime timestamp;
  final Map<String, Object?> context;

  Map<String, dynamic> toJson() => {
    'event': type.name,
    'timestamp': timestamp.toUtc().toIso8601String(),
    if (context.isNotEmpty) 'context': context,
  };
}

/// Service for tracking KeyFlow analytics and metrics.
///
/// Tracks:
/// - Page views and user interactions
/// - Slot operations (add/delete/reorder)
/// - Node dragging and canvas manipulation
/// - Errors and failures
/// - Performance metrics
///
/// All events are logged using structured logging for analysis.
class KeyFlowAnalytics {
  KeyFlowAnalytics({StructuredLogger? logger, bool enabled = true})
    : _logger = logger ?? getLogger('keyflow_analytics'),
      _enabled = enabled {
    _logger.info('analytics.init', context: {'enabled': _enabled});
  }

  final StructuredLogger _logger;
  final bool _enabled;

  final _eventController = StreamController<KeyFlowEvent>.broadcast();

  /// Stream of analytics events.
  Stream<KeyFlowEvent> get eventStream => _eventController.stream;

  /// Track a KeyFlow event.
  void trackEvent(
    KeyFlowEventType type, {
    Map<String, Object?> context = const {},
  }) {
    if (!_enabled) return;

    final event = KeyFlowEvent(
      type: type,
      timestamp: DateTime.now(),
      context: context,
    );

    // Emit to stream for potential external analytics
    if (!_eventController.isClosed) {
      _eventController.add(event);
    }

    // Log event for structured logging
    _logger.info('keyflow.event.${type.name}', context: event.toJson());
  }

  /// Track page view.
  void trackPageView({int? deviceCount, int? slotCount, int? nodeCount}) {
    trackEvent(
      KeyFlowEventType.pageView,
      context: {
        if (deviceCount != null) 'device_count': deviceCount,
        if (slotCount != null) 'slot_count': slotCount,
        if (nodeCount != null) 'node_count': nodeCount,
      },
    );
  }

  /// Track slot added.
  void trackSlotAdded({
    required String deviceId,
    required String profileId,
    required String keymapId,
    int? priority,
    bool? autoAssigned,
  }) {
    trackEvent(
      KeyFlowEventType.slotAdded,
      context: {
        'device_id': deviceId,
        'profile_id': profileId,
        'keymap_id': keymapId,
        if (priority != null) 'priority': priority,
        if (autoAssigned != null) 'auto_assigned': autoAssigned,
      },
    );
  }

  /// Track slot deleted.
  void trackSlotDeleted({
    required String deviceId,
    required String slotId,
    int? priority,
  }) {
    trackEvent(
      KeyFlowEventType.slotDeleted,
      context: {
        'device_id': deviceId,
        'slot_id': slotId,
        if (priority != null) 'priority': priority,
      },
    );
  }

  /// Track slot reordered.
  void trackSlotReordered({
    required String deviceId,
    required String slotId,
    required int oldPriority,
    required int newPriority,
  }) {
    trackEvent(
      KeyFlowEventType.slotReordered,
      context: {
        'device_id': deviceId,
        'slot_id': slotId,
        'old_priority': oldPriority,
        'new_priority': newPriority,
      },
    );
  }

  /// Track node dragged.
  void trackNodeDragged({
    required String nodeId,
    required String nodeType,
    required double deltaX,
    required double deltaY,
  }) {
    trackEvent(
      KeyFlowEventType.nodeDragged,
      context: {
        'node_id': nodeId,
        'node_type': nodeType,
        'delta_x': deltaX,
        'delta_y': deltaY,
      },
    );
  }

  /// Track error occurred.
  void trackError({
    required String errorType,
    required String errorMessage,
    String? operation,
    Map<String, Object?>? additionalContext,
  }) {
    trackEvent(
      KeyFlowEventType.errorOccurred,
      context: {
        'error_type': errorType,
        'error_message': errorMessage,
        if (operation != null) 'operation': operation,
        if (additionalContext != null) ...additionalContext,
      },
    );
  }

  /// Track edge interaction.
  void trackEdgeInteraction({
    required String action,
    required String edgeId,
    bool? isActive,
  }) {
    trackEvent(
      KeyFlowEventType.edgeInteracted,
      context: {
        'action': action,
        'edge_id': edgeId,
        if (isActive != null) 'is_active': isActive,
      },
    );
  }

  /// Track dialog opened.
  void trackDialogOpened({required String dialogType}) {
    trackEvent(
      KeyFlowEventType.dialogOpened,
      context: {'dialog_type': dialogType},
    );
  }

  /// Track keyboard shortcut used.
  void trackKeyboardShortcut({
    required String shortcut,
    required String action,
  }) {
    trackEvent(
      KeyFlowEventType.keyboardShortcut,
      context: {'shortcut': shortcut, 'action': action},
    );
  }

  /// Track canvas manipulation.
  void trackCanvasManipulation({
    required String action,
    double? scale,
    double? deltaX,
    double? deltaY,
  }) {
    trackEvent(
      KeyFlowEventType.canvasManipulated,
      context: {
        'action': action,
        if (scale != null) 'scale': scale,
        if (deltaX != null) 'delta_x': deltaX,
        if (deltaY != null) 'delta_y': deltaY,
      },
    );
  }

  /// Track auto-save triggered.
  void trackAutoSave({
    required String operation,
    required bool success,
    int? retryCount,
    String? errorMessage,
  }) {
    trackEvent(
      KeyFlowEventType.autoSaveTriggered,
      context: {
        'operation': operation,
        'success': success,
        if (retryCount != null) 'retry_count': retryCount,
        if (errorMessage != null) 'error_message': errorMessage,
      },
    );
  }

  /// Clean up resources.
  void dispose() {
    _logger.info('analytics.dispose');
    _eventController.close();
  }
}
