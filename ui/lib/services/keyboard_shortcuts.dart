/// Keyboard shortcut definitions and management for flow editor.
library;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

/// Defines all keyboard shortcuts available in the flow editor.
class KeyboardShortcuts {
  KeyboardShortcuts._();

  // Navigation shortcuts
  static const tabToNextNode = SingleActivator(LogicalKeyboardKey.tab);
  static const tabToPreviousNode = SingleActivator(
    LogicalKeyboardKey.tab,
    shift: true,
  );

  // Arrow key movement (10px per press)
  static const moveNodeUp = SingleActivator(LogicalKeyboardKey.arrowUp);
  static const moveNodeDown = SingleActivator(LogicalKeyboardKey.arrowDown);
  static const moveNodeLeft = SingleActivator(LogicalKeyboardKey.arrowLeft);
  static const moveNodeRight = SingleActivator(LogicalKeyboardKey.arrowRight);

  // Action shortcuts
  static const openNodeDetails = SingleActivator(LogicalKeyboardKey.enter);
  static const deleteSelectedEdge = SingleActivator(LogicalKeyboardKey.delete);
  static const clearSelection = SingleActivator(LogicalKeyboardKey.escape);

  // Zoom shortcuts
  static const resetZoom = SingleActivator(
    LogicalKeyboardKey.digit0,
    control: true,
  );
  static const zoomIn = SingleActivator(
    LogicalKeyboardKey.equal,
    control: true,
  );
  static const zoomOut = SingleActivator(
    LogicalKeyboardKey.minus,
    control: true,
  );

  // Help shortcut
  static const showHelp = SingleActivator(
    LogicalKeyboardKey.slash,
    control: true,
  );

  /// Get a human-readable description of a shortcut.
  static String describe(SingleActivator activator) {
    final buffer = StringBuffer();
    if (activator.control) buffer.write('Ctrl+');
    if (activator.shift) buffer.write('Shift+');
    if (activator.alt) buffer.write('Alt+');
    if (activator.meta) buffer.write('Meta+');

    buffer.write(_keyName(activator.trigger));
    return buffer.toString();
  }

  static String _keyName(LogicalKeyboardKey key) {
    if (key == LogicalKeyboardKey.tab) return 'Tab';
    if (key == LogicalKeyboardKey.arrowUp) return 'Up';
    if (key == LogicalKeyboardKey.arrowDown) return 'Down';
    if (key == LogicalKeyboardKey.arrowLeft) return 'Left';
    if (key == LogicalKeyboardKey.arrowRight) return 'Right';
    if (key == LogicalKeyboardKey.enter) return 'Enter';
    if (key == LogicalKeyboardKey.delete) return 'Delete';
    if (key == LogicalKeyboardKey.escape) return 'Escape';
    if (key == LogicalKeyboardKey.digit0) return '0';
    if (key == LogicalKeyboardKey.equal) return '+';
    if (key == LogicalKeyboardKey.minus) return '-';
    if (key == LogicalKeyboardKey.slash) return '?';
    return key.keyLabel;
  }
}

/// Shortcut information for display in help dialog.
class ShortcutInfo {
  const ShortcutInfo({
    required this.category,
    required this.description,
    required this.activator,
  });

  final String category;
  final String description;
  final SingleActivator activator;

  String get shortcut => KeyboardShortcuts.describe(activator);
}

/// All shortcuts grouped by category for help display.
class ShortcutCatalog {
  ShortcutCatalog._();

  static const List<ShortcutInfo> all = [
    // Navigation
    ShortcutInfo(
      category: 'Navigation',
      description: 'Navigate to next node',
      activator: KeyboardShortcuts.tabToNextNode,
    ),
    ShortcutInfo(
      category: 'Navigation',
      description: 'Navigate to previous node',
      activator: KeyboardShortcuts.tabToPreviousNode,
    ),
    ShortcutInfo(
      category: 'Navigation',
      description: 'Move focused node up',
      activator: KeyboardShortcuts.moveNodeUp,
    ),
    ShortcutInfo(
      category: 'Navigation',
      description: 'Move focused node down',
      activator: KeyboardShortcuts.moveNodeDown,
    ),
    ShortcutInfo(
      category: 'Navigation',
      description: 'Move focused node left',
      activator: KeyboardShortcuts.moveNodeLeft,
    ),
    ShortcutInfo(
      category: 'Navigation',
      description: 'Move focused node right',
      activator: KeyboardShortcuts.moveNodeRight,
    ),

    // Actions
    ShortcutInfo(
      category: 'Actions',
      description: 'Open node details',
      activator: KeyboardShortcuts.openNodeDetails,
    ),
    ShortcutInfo(
      category: 'Actions',
      description: 'Delete selected edge',
      activator: KeyboardShortcuts.deleteSelectedEdge,
    ),
    ShortcutInfo(
      category: 'Actions',
      description: 'Clear selection',
      activator: KeyboardShortcuts.clearSelection,
    ),

    // Zoom
    ShortcutInfo(
      category: 'Zoom',
      description: 'Reset zoom to 100%',
      activator: KeyboardShortcuts.resetZoom,
    ),
    ShortcutInfo(
      category: 'Zoom',
      description: 'Zoom in by 10%',
      activator: KeyboardShortcuts.zoomIn,
    ),
    ShortcutInfo(
      category: 'Zoom',
      description: 'Zoom out by 10%',
      activator: KeyboardShortcuts.zoomOut,
    ),

    // Help
    ShortcutInfo(
      category: 'Help',
      description: 'Show keyboard shortcuts',
      activator: KeyboardShortcuts.showHelp,
    ),
  ];

  /// Get shortcuts grouped by category.
  static Map<String, List<ShortcutInfo>> get byCategory {
    final map = <String, List<ShortcutInfo>>{};
    for (final info in all) {
      map.putIfAbsent(info.category, () => []).add(info);
    }
    return map;
  }
}
