// Remap toggle switch widget for per-device remapping control.
//
// Displays a Material Switch with ON/OFF labels and color-coded state
// to control whether remapping is enabled for a specific device.

import 'package:flutter/material.dart';

/// Widget that provides a visual toggle for device remapping state.
///
/// Shows a Material Switch with clear ON/OFF visual indicators including
/// labels and color-coded states. When toggled, calls the provided callback.
class RemapToggle extends StatelessWidget {
  /// Current remap enabled state.
  final bool enabled;

  /// Callback invoked when toggle state changes.
  final ValueChanged<bool>? onChanged;

  const RemapToggle({
    super.key,
    required this.enabled,
    this.onChanged,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;

    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Text(
          enabled ? 'ON' : 'OFF',
          style: theme.textTheme.labelLarge?.copyWith(
            color: enabled ? colorScheme.primary : colorScheme.onSurfaceVariant,
            fontWeight: FontWeight.bold,
          ),
        ),
        const SizedBox(width: 8),
        Switch(
          value: enabled,
          onChanged: onChanged,
          activeColor: colorScheme.primary,
        ),
      ],
    );
  }
}
