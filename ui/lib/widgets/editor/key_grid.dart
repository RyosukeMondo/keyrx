/// KeyGrid widget for rendering keyboard layout.
///
/// Provides a visual grid representation of a keyboard with configurable
/// key layout and selection handling.
library;

import 'package:flutter/material.dart';

import 'key_button.dart';

/// Visual keyboard grid layout for keymap editing.
///
/// Renders a standard keyboard layout with customizable key selection
/// and interaction callbacks. Uses [KeyButton] for individual key rendering.
class KeyGrid extends StatelessWidget {
  /// Creates a KeyGrid widget.
  ///
  /// [onKeySelected] is called when a key is tapped.
  /// [selectedKey] highlights the currently selected key.
  const KeyGrid({
    super.key,
    this.onKeySelected,
    this.selectedKey,
  });

  /// Callback invoked when a key is selected.
  final void Function(String key)? onKeySelected;

  /// The currently selected key label.
  final String? selectedKey;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(16),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          _buildRow(['Esc', 'F1', 'F2', 'F3', 'F4', 'F5', 'F6', 'F7', 'F8', 'F9', 'F10', 'F11', 'F12']),
          const SizedBox(height: 8),
          _buildRow(['`', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=', 'Backspace']),
          const SizedBox(height: 4),
          _buildRow(['Tab', 'Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', '[', ']', '\\']),
          const SizedBox(height: 4),
          _buildRow(['CapsLock', 'A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L', ';', "'", 'Enter']),
          const SizedBox(height: 4),
          _buildRow(['LShift', 'Z', 'X', 'C', 'V', 'B', 'N', 'M', ',', '.', '/', 'RShift']),
          const SizedBox(height: 4),
          _buildRow(['LCtrl', 'LWin', 'LAlt', 'Space', 'RAlt', 'RWin', 'Menu', 'RCtrl']),
        ],
      ),
    );
  }

  /// Builds a single row of keys.
  Widget _buildRow(List<String> keys) {
    return Row(
      mainAxisAlignment: MainAxisAlignment.center,
      children: keys.map((key) => _buildKey(key)).toList(),
    );
  }

  /// Builds an individual key button.
  Widget _buildKey(String label) {
    final isSelected = selectedKey == label;

    return KeyButton(
      label: label,
      isSelected: isSelected,
      onTap: () => onKeySelected?.call(label),
    );
  }
}
