/// KeyButton widget for rendering individual keyboard keys.
///
/// Provides a reusable button component for displaying keyboard keys
/// with various states (selected, hovered, pressed) and size variants.
library;

import 'package:flutter/material.dart';

/// Visual button representing a keyboard key.
///
/// Supports multiple states and customizable sizing for different
/// key types (standard keys, modifiers, spacebar, etc.).
class KeyButton extends StatelessWidget {
  /// Creates a KeyButton widget.
  ///
  /// The [label] is displayed on the key face.
  /// [isSelected] highlights the key with accent color.
  /// [onTap] callback is invoked when the key is pressed.
  /// [width] and [height] customize the key dimensions.
  const KeyButton({
    super.key,
    required this.label,
    this.isSelected = false,
    this.onTap,
    this.width,
    this.height = 40,
  });

  /// The text label displayed on the key.
  final String label;

  /// Whether this key is currently selected.
  final bool isSelected;

  /// Callback invoked when the key is tapped.
  final VoidCallback? onTap;

  /// Custom width for the key. If null, uses [getStandardWidth].
  final double? width;

  /// Height of the key. Defaults to 40.
  final double height;

  @override
  Widget build(BuildContext context) {
    final effectiveWidth = width ?? getStandardWidth(label);

    return Padding(
      padding: const EdgeInsets.all(2),
      child: Material(
        color: isSelected ? Colors.blue : Colors.grey[800],
        borderRadius: BorderRadius.circular(4),
        child: InkWell(
          onTap: onTap,
          borderRadius: BorderRadius.circular(4),
          child: Container(
            width: effectiveWidth,
            height: height,
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

  /// Returns standard width for common key types.
  ///
  /// This provides default sizing for special keys like Backspace,
  /// modifiers, spacebar, etc. Standard letter/number keys get 40px width.
  static double getStandardWidth(String label) {
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
