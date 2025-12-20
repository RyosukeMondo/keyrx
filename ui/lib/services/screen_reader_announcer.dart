/// Screen reader announcer for accessibility.
library;

import 'package:flutter/material.dart';
import 'package:flutter/semantics.dart';

/// Service for announcing state changes to screen readers.
///
/// Provides context-aware announcements for various KeyFlow operations
/// to keep screen reader users informed of system state changes.
class ScreenReaderAnnouncer {
  ScreenReaderAnnouncer({required this.context});

  final BuildContext context;

  /// Announce device connection.
  void announceDeviceConnected(String deviceName) {
    _announce('$deviceName connected');
  }

  /// Announce device disconnection.
  void announceDeviceDisconnected(String deviceName) {
    _announce('$deviceName disconnected');
  }

  /// Announce slot added.
  void announceSlotAdded(
    String deviceName,
    String profileName,
    String keymapName,
  ) {
    _announce('Slot added to $deviceName: $profileName with $keymapName');
  }

  /// Announce slot removed.
  void announceSlotRemoved(String deviceName, int slotIndex) {
    _announce('Slot $slotIndex removed from $deviceName');
  }

  /// Announce slot reordered.
  void announceSlotReordered(
    String deviceName,
    int oldPriority,
    int newPriority,
  ) {
    _announce(
      'Slot priority changed from $oldPriority to $newPriority on $deviceName',
    );
  }

  /// Announce node moved.
  void announceNodeMoved(String nodeName) {
    _announce('$nodeName moved to new position');
  }

  /// Announce node focused.
  void announceNodeFocused(String nodeName, String nodeType) {
    _announce('$nodeName $nodeType focused');
  }

  /// Announce edge selected.
  void announceEdgeSelected(String fromNode, String toNode) {
    _announce('Connection selected from $fromNode to $toNode');
  }

  /// Announce edge deleted.
  void announceEdgeDeleted(String fromNode, String toNode) {
    _announce('Connection deleted from $fromNode to $toNode');
  }

  /// Announce save state.
  void announceSaveState(String state) {
    switch (state) {
      case 'saving':
        _announce('Saving changes');
        break;
      case 'saved':
        _announce('Changes saved successfully');
        break;
      case 'error':
        _announce('Failed to save changes');
        break;
    }
  }

  /// Announce validation error.
  void announceValidationError(String nodeName, String error) {
    _announce('Error on $nodeName: $error');
  }

  /// Announce zoom level changed.
  void announceZoomChanged(String zoomLevel) {
    _announce('Zoom level: $zoomLevel');
  }

  void _announce(String message) {
    if (!_canAnnounce) return;

    // Use SemanticsService to announce to screen readers
    SemanticsService.announce(message, TextDirection.ltr);
  }

  // Check if semantics are enabled (screen reader is active)
  bool get _canAnnounce =>
      SemanticsBinding.instance.accessibilityFeatures.accessibleNavigation;
}
