import 'package:flutter/material.dart';

/// A standardized dialog supporting multiple action buttons.
///
/// Provides a consistent UI for dialogs that need more than simple
/// confirm/cancel actions. Actions are displayed in order with the
/// last action optionally styled as primary.
class MultiActionDialog<T> extends StatelessWidget {
  const MultiActionDialog({
    super.key,
    required this.title,
    required this.message,
    required this.actions,
    this.icon,
  });

  final String title;
  final String message;
  final List<DialogAction<T>> actions;
  final IconData? icon;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final scheme = theme.colorScheme;

    return AlertDialog(
      backgroundColor: scheme.surface,
      titlePadding: const EdgeInsets.fromLTRB(24, 20, 24, 12),
      contentPadding: const EdgeInsets.fromLTRB(24, 0, 24, 16),
      actionsPadding: const EdgeInsets.fromLTRB(16, 0, 16, 12),
      title: Row(
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          if (icon != null) ...[
            Icon(icon, color: scheme.primary),
            const SizedBox(width: 12),
          ],
          Expanded(child: Text(title, style: theme.textTheme.titleLarge)),
        ],
      ),
      content: Text(message, style: theme.textTheme.bodyMedium),
      actions: actions.map((action) {
        if (action.isPrimary) {
          return FilledButton(
            onPressed: () => Navigator.of(context).pop(action.value),
            style: action.isDestructive
                ? FilledButton.styleFrom(
                    backgroundColor: scheme.error,
                    foregroundColor: scheme.onError,
                  )
                : null,
            child: Text(action.label),
          );
        } else {
          return TextButton(
            onPressed: () => Navigator.of(context).pop(action.value),
            child: Text(action.label),
          );
        }
      }).toList(),
    );
  }

  /// Shows a multi-action dialog and returns the selected action's value.
  ///
  /// Returns the value associated with the selected action, or `null` if dismissed.
  static Future<T?> show<T>(
    BuildContext context, {
    required String title,
    required String message,
    required List<DialogAction<T>> actions,
    IconData? icon,
    bool barrierDismissible = true,
  }) {
    return showDialog<T>(
      context: context,
      barrierDismissible: barrierDismissible,
      builder: (_) => MultiActionDialog<T>(
        title: title,
        message: message,
        actions: actions,
        icon: icon,
      ),
    );
  }
}

/// Represents an action button in a multi-action dialog.
class DialogAction<T> {
  const DialogAction({
    required this.label,
    required this.value,
    this.isPrimary = false,
    this.isDestructive = false,
  });

  final String label;
  final T value;
  final bool isPrimary;
  final bool isDestructive;
}
