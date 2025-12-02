/// Keyboard layout model for ANSI 104-key layout.
///
/// Provides key definitions with proper sizing and positioning
/// for the visual keyboard widget.
library;

import 'package:flutter/foundation.dart';

/// Definition of a single key on the keyboard.
@immutable
class KeyDefinition {
  const KeyDefinition({
    required this.id,
    required this.label,
    this.width = 1.0,
    this.height = 1.0,
    this.row = 0,
    this.column = 0,
    this.secondaryLabel,
  });

  /// Unique identifier for the key (matches Rhai key names).
  final String id;

  /// Display label for the key.
  final String label;

  /// Width in units (1.0 = standard key width).
  final double width;

  /// Height in units (1.0 = standard key height).
  final double height;

  /// Row index (0 = top row).
  final int row;

  /// Column position within the row.
  final double column;

  /// Secondary label (e.g., shifted character).
  final String? secondaryLabel;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is KeyDefinition &&
          runtimeType == other.runtimeType &&
          id == other.id;

  @override
  int get hashCode => id.hashCode;
}

/// A row of keys on the keyboard.
@immutable
class KeyboardRow {
  const KeyboardRow({
    required this.keys,
    this.offsetX = 0.0,
  });

  /// Keys in this row.
  final List<KeyDefinition> keys;

  /// Horizontal offset for the entire row.
  final double offsetX;
}

/// Complete keyboard layout definition.
@immutable
class KeyboardLayout {
  const KeyboardLayout({
    required this.name,
    required this.rows,
    this.unitSize = 48.0,
    this.keySpacing = 4.0,
  });

  /// Layout name (e.g., "ANSI 104").
  final String name;

  /// Rows of keys.
  final List<KeyboardRow> rows;

  /// Base unit size in pixels (width of a 1.0u key).
  final double unitSize;

  /// Spacing between keys in pixels.
  final double keySpacing;

  /// Get total width of the keyboard in pixels.
  double get totalWidth {
    double maxWidth = 0;
    for (final row in rows) {
      double rowWidth = row.offsetX;
      for (final key in row.keys) {
        rowWidth += key.width * unitSize + keySpacing;
      }
      if (rowWidth > maxWidth) maxWidth = rowWidth;
    }
    return maxWidth;
  }

  /// Get total height of the keyboard in pixels.
  double get totalHeight {
    return rows.length * (unitSize + keySpacing);
  }

  /// Find a key by its ID.
  KeyDefinition? findKey(String id) {
    for (final row in rows) {
      for (final key in row.keys) {
        if (key.id == id) return key;
      }
    }
    return null;
  }

  /// Get all key IDs.
  List<String> get allKeyIds {
    final ids = <String>[];
    for (final row in rows) {
      for (final key in row.keys) {
        ids.add(key.id);
      }
    }
    return ids;
  }

  /// Standard ANSI 104-key layout.
  static KeyboardLayout ansi104({
    double unitSize = 48.0,
    double keySpacing = 4.0,
  }) {
    return KeyboardLayout(
      name: 'ANSI 104',
      unitSize: unitSize,
      keySpacing: keySpacing,
      rows: [
        // Row 0: Function row (Esc, F1-F12)
        const KeyboardRow(
          keys: [
            KeyDefinition(id: 'Escape', label: 'Esc', row: 0, column: 0),
            KeyDefinition(id: 'F1', label: 'F1', row: 0, column: 2),
            KeyDefinition(id: 'F2', label: 'F2', row: 0, column: 3),
            KeyDefinition(id: 'F3', label: 'F3', row: 0, column: 4),
            KeyDefinition(id: 'F4', label: 'F4', row: 0, column: 5),
            KeyDefinition(id: 'F5', label: 'F5', row: 0, column: 6.5),
            KeyDefinition(id: 'F6', label: 'F6', row: 0, column: 7.5),
            KeyDefinition(id: 'F7', label: 'F7', row: 0, column: 8.5),
            KeyDefinition(id: 'F8', label: 'F8', row: 0, column: 9.5),
            KeyDefinition(id: 'F9', label: 'F9', row: 0, column: 11),
            KeyDefinition(id: 'F10', label: 'F10', row: 0, column: 12),
            KeyDefinition(id: 'F11', label: 'F11', row: 0, column: 13),
            KeyDefinition(id: 'F12', label: 'F12', row: 0, column: 14),
          ],
        ),
        // Row 1: Number row
        const KeyboardRow(
          keys: [
            KeyDefinition(
              id: 'Grave',
              label: '`',
              secondaryLabel: '~',
              row: 1,
              column: 0,
            ),
            KeyDefinition(
              id: 'Key1',
              label: '1',
              secondaryLabel: '!',
              row: 1,
              column: 1,
            ),
            KeyDefinition(
              id: 'Key2',
              label: '2',
              secondaryLabel: '@',
              row: 1,
              column: 2,
            ),
            KeyDefinition(
              id: 'Key3',
              label: '3',
              secondaryLabel: '#',
              row: 1,
              column: 3,
            ),
            KeyDefinition(
              id: 'Key4',
              label: '4',
              secondaryLabel: r'$',
              row: 1,
              column: 4,
            ),
            KeyDefinition(
              id: 'Key5',
              label: '5',
              secondaryLabel: '%',
              row: 1,
              column: 5,
            ),
            KeyDefinition(
              id: 'Key6',
              label: '6',
              secondaryLabel: '^',
              row: 1,
              column: 6,
            ),
            KeyDefinition(
              id: 'Key7',
              label: '7',
              secondaryLabel: '&',
              row: 1,
              column: 7,
            ),
            KeyDefinition(
              id: 'Key8',
              label: '8',
              secondaryLabel: '*',
              row: 1,
              column: 8,
            ),
            KeyDefinition(
              id: 'Key9',
              label: '9',
              secondaryLabel: '(',
              row: 1,
              column: 9,
            ),
            KeyDefinition(
              id: 'Key0',
              label: '0',
              secondaryLabel: ')',
              row: 1,
              column: 10,
            ),
            KeyDefinition(
              id: 'Minus',
              label: '-',
              secondaryLabel: '_',
              row: 1,
              column: 11,
            ),
            KeyDefinition(
              id: 'Equal',
              label: '=',
              secondaryLabel: '+',
              row: 1,
              column: 12,
            ),
            KeyDefinition(
              id: 'Backspace',
              label: 'Backspace',
              width: 2.0,
              row: 1,
              column: 13,
            ),
          ],
        ),
        // Row 2: QWERTY row
        const KeyboardRow(
          keys: [
            KeyDefinition(
              id: 'Tab',
              label: 'Tab',
              width: 1.5,
              row: 2,
              column: 0,
            ),
            KeyDefinition(id: 'KeyQ', label: 'Q', row: 2, column: 1.5),
            KeyDefinition(id: 'KeyW', label: 'W', row: 2, column: 2.5),
            KeyDefinition(id: 'KeyE', label: 'E', row: 2, column: 3.5),
            KeyDefinition(id: 'KeyR', label: 'R', row: 2, column: 4.5),
            KeyDefinition(id: 'KeyT', label: 'T', row: 2, column: 5.5),
            KeyDefinition(id: 'KeyY', label: 'Y', row: 2, column: 6.5),
            KeyDefinition(id: 'KeyU', label: 'U', row: 2, column: 7.5),
            KeyDefinition(id: 'KeyI', label: 'I', row: 2, column: 8.5),
            KeyDefinition(id: 'KeyO', label: 'O', row: 2, column: 9.5),
            KeyDefinition(id: 'KeyP', label: 'P', row: 2, column: 10.5),
            KeyDefinition(
              id: 'BracketLeft',
              label: '[',
              secondaryLabel: '{',
              row: 2,
              column: 11.5,
            ),
            KeyDefinition(
              id: 'BracketRight',
              label: ']',
              secondaryLabel: '}',
              row: 2,
              column: 12.5,
            ),
            KeyDefinition(
              id: 'Backslash',
              label: '\\',
              secondaryLabel: '|',
              width: 1.5,
              row: 2,
              column: 13.5,
            ),
          ],
        ),
        // Row 3: Home row (ASDF)
        const KeyboardRow(
          keys: [
            KeyDefinition(
              id: 'CapsLock',
              label: 'Caps',
              width: 1.75,
              row: 3,
              column: 0,
            ),
            KeyDefinition(id: 'KeyA', label: 'A', row: 3, column: 1.75),
            KeyDefinition(id: 'KeyS', label: 'S', row: 3, column: 2.75),
            KeyDefinition(id: 'KeyD', label: 'D', row: 3, column: 3.75),
            KeyDefinition(id: 'KeyF', label: 'F', row: 3, column: 4.75),
            KeyDefinition(id: 'KeyG', label: 'G', row: 3, column: 5.75),
            KeyDefinition(id: 'KeyH', label: 'H', row: 3, column: 6.75),
            KeyDefinition(id: 'KeyJ', label: 'J', row: 3, column: 7.75),
            KeyDefinition(id: 'KeyK', label: 'K', row: 3, column: 8.75),
            KeyDefinition(id: 'KeyL', label: 'L', row: 3, column: 9.75),
            KeyDefinition(
              id: 'Semicolon',
              label: ';',
              secondaryLabel: ':',
              row: 3,
              column: 10.75,
            ),
            KeyDefinition(
              id: 'Quote',
              label: "'",
              secondaryLabel: '"',
              row: 3,
              column: 11.75,
            ),
            KeyDefinition(
              id: 'Enter',
              label: 'Enter',
              width: 2.25,
              row: 3,
              column: 12.75,
            ),
          ],
        ),
        // Row 4: Bottom letter row (ZXCV)
        const KeyboardRow(
          keys: [
            KeyDefinition(
              id: 'ShiftLeft',
              label: 'Shift',
              width: 2.25,
              row: 4,
              column: 0,
            ),
            KeyDefinition(id: 'KeyZ', label: 'Z', row: 4, column: 2.25),
            KeyDefinition(id: 'KeyX', label: 'X', row: 4, column: 3.25),
            KeyDefinition(id: 'KeyC', label: 'C', row: 4, column: 4.25),
            KeyDefinition(id: 'KeyV', label: 'V', row: 4, column: 5.25),
            KeyDefinition(id: 'KeyB', label: 'B', row: 4, column: 6.25),
            KeyDefinition(id: 'KeyN', label: 'N', row: 4, column: 7.25),
            KeyDefinition(id: 'KeyM', label: 'M', row: 4, column: 8.25),
            KeyDefinition(
              id: 'Comma',
              label: ',',
              secondaryLabel: '<',
              row: 4,
              column: 9.25,
            ),
            KeyDefinition(
              id: 'Period',
              label: '.',
              secondaryLabel: '>',
              row: 4,
              column: 10.25,
            ),
            KeyDefinition(
              id: 'Slash',
              label: '/',
              secondaryLabel: '?',
              row: 4,
              column: 11.25,
            ),
            KeyDefinition(
              id: 'ShiftRight',
              label: 'Shift',
              width: 2.75,
              row: 4,
              column: 12.25,
            ),
          ],
        ),
        // Row 5: Bottom row (Ctrl, Win, Alt, Space, etc.)
        const KeyboardRow(
          keys: [
            KeyDefinition(
              id: 'ControlLeft',
              label: 'Ctrl',
              width: 1.25,
              row: 5,
              column: 0,
            ),
            KeyDefinition(
              id: 'MetaLeft',
              label: 'Win',
              width: 1.25,
              row: 5,
              column: 1.25,
            ),
            KeyDefinition(
              id: 'AltLeft',
              label: 'Alt',
              width: 1.25,
              row: 5,
              column: 2.5,
            ),
            KeyDefinition(
              id: 'Space',
              label: 'Space',
              width: 6.25,
              row: 5,
              column: 3.75,
            ),
            KeyDefinition(
              id: 'AltRight',
              label: 'Alt',
              width: 1.25,
              row: 5,
              column: 10,
            ),
            KeyDefinition(
              id: 'MetaRight',
              label: 'Win',
              width: 1.25,
              row: 5,
              column: 11.25,
            ),
            KeyDefinition(
              id: 'ContextMenu',
              label: 'Menu',
              width: 1.25,
              row: 5,
              column: 12.5,
            ),
            KeyDefinition(
              id: 'ControlRight',
              label: 'Ctrl',
              width: 1.25,
              row: 5,
              column: 13.75,
            ),
          ],
        ),
      ],
    );
  }

  /// Get a compact layout without navigation cluster for space efficiency.
  static KeyboardLayout compact({
    double unitSize = 40.0,
    double keySpacing = 3.0,
  }) {
    // Uses the same ANSI layout but with smaller unit size
    return ansi104(unitSize: unitSize, keySpacing: keySpacing);
  }
}

/// Extension methods for working with keyboard layouts.
extension KeyboardLayoutHelpers on KeyboardLayout {
  /// Calculate the pixel position for a key.
  ({double x, double y}) getKeyPosition(KeyDefinition key) {
    final row = key.row;
    final x = key.column * (unitSize + keySpacing);
    final y = row * (unitSize + keySpacing);
    return (x: x, y: y);
  }

  /// Calculate the pixel size for a key.
  ({double width, double height}) getKeySize(KeyDefinition key) {
    final width = key.width * unitSize + (key.width - 1) * keySpacing;
    final height = key.height * unitSize + (key.height - 1) * keySpacing;
    return (width: width, height: height);
  }
}
