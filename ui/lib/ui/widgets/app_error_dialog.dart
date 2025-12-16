import 'package:flutter/material.dart';

/// Standardized error dialog for consistent user messaging.
class AppErrorDialog extends StatelessWidget {
  const AppErrorDialog({
    required this.title,
    required this.message,
    super.key,
    this.primaryActionLabel = 'OK',
    this.onPrimaryAction,
    this.secondaryActionLabel,
    this.onSecondaryAction,
    this.icon = Icons.error_outline,
  });
  final String title;
  final String message;
  final String primaryActionLabel;
  final VoidCallback? onPrimaryAction;
  final String? secondaryActionLabel;
  final VoidCallback? onSecondaryAction;
  final IconData icon;

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
        children: [
          Icon(icon, color: scheme.error),
          const SizedBox(width: 12),
          Expanded(child: Text(title, style: theme.textTheme.titleLarge)),
        ],
      ),
      content: Text(message, style: theme.textTheme.bodyMedium),
      actions: [
        if (secondaryActionLabel != null)
          TextButton(
            onPressed: () => _handleSecondary(context),
            child: Text(secondaryActionLabel!),
          ),
        FilledButton(
          onPressed: () => _handlePrimary(context),
          child: Text(primaryActionLabel),
        ),
      ],
    );
  }

  void _handlePrimary(BuildContext context) {
    Navigator.of(context).pop();
    onPrimaryAction?.call();
  }

  void _handleSecondary(BuildContext context) {
    Navigator.of(context).pop();
    onSecondaryAction?.call();
  }

  /// Helper to present the dialog with consistent options.
  static Future<void> show(
    BuildContext context, {
    required String title,
    required String message,
    String primaryActionLabel = 'OK',
    VoidCallback? onPrimaryAction,
    String? secondaryActionLabel,
    VoidCallback? onSecondaryAction,
    IconData icon = Icons.error_outline,
    bool barrierDismissible = false,
  }) => showDialog<void>(
    context: context,
    barrierDismissible: barrierDismissible,
    builder: (_) => AppErrorDialog(
      title: title,
      message: message,
      primaryActionLabel: primaryActionLabel,
      onPrimaryAction: onPrimaryAction,
      secondaryActionLabel: secondaryActionLabel,
      onSecondaryAction: onSecondaryAction,
      icon: icon,
    ),
  );
}
