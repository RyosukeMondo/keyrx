/// Visual keyboard widget.
///
/// Renders a keyboard layout with selectable keys
/// for the visual keymap editor.

import 'package:flutter/material.dart';

/// Visual keyboard widget for keymap editing.
class KeyboardWidget extends StatelessWidget {
  final void Function(String key)? onKeySelected;
  final String? selectedKey;

  const KeyboardWidget({
    super.key,
    this.onKeySelected,
    this.selectedKey,
  });

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

  Widget _buildRow(List<String> keys) {
    return Row(
      mainAxisAlignment: MainAxisAlignment.center,
      children: keys.map((key) => _buildKey(key)).toList(),
    );
  }

  Widget _buildKey(String label) {
    final isSelected = selectedKey == label;
    final width = _getKeyWidth(label);

    return Padding(
      padding: const EdgeInsets.all(2),
      child: Material(
        color: isSelected ? Colors.blue : Colors.grey[800],
        borderRadius: BorderRadius.circular(4),
        child: InkWell(
          onTap: () => onKeySelected?.call(label),
          borderRadius: BorderRadius.circular(4),
          child: Container(
            width: width,
            height: 40,
            alignment: Alignment.center,
            child: Text(
              label,
              style: TextStyle(
                color: isSelected ? Colors.white : Colors.grey[300],
                fontSize: 12,
              ),
            ),
          ),
        ),
      ),
    );
  }

  double _getKeyWidth(String label) {
    switch (label) {
      case 'Backspace':
      case 'Tab':
      case 'CapsLock':
      case 'Enter':
        return 70;
      case 'LShift':
      case 'RShift':
        return 90;
      case 'Space':
        return 200;
      case 'LCtrl':
      case 'RCtrl':
      case 'LAlt':
      case 'RAlt':
      case 'LWin':
      case 'RWin':
      case 'Menu':
        return 50;
      default:
        return 40;
    }
  }
}
