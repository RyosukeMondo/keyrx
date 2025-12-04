import 'package:flutter/material.dart';

/// A standardized information dialog for displaying help or detailed content.
///
/// Provides a consistent UI for showing read-only information with
/// optional scrolling for long content.
class InfoDialog extends StatelessWidget {
  const InfoDialog({
    super.key,
    required this.title,
    required this.content,
    this.closeLabel = 'Close',
    this.icon,
    this.maxHeight = 500.0,
  });

  final String title;
  final Widget content;
  final String closeLabel;
  final IconData? icon;
  final double maxHeight;

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
      content: ConstrainedBox(
        constraints: BoxConstraints(maxHeight: maxHeight),
        child: SingleChildScrollView(
          child: content,
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: Text(closeLabel),
        ),
      ],
    );
  }

  /// Shows an information dialog.
  static Future<void> show(
    BuildContext context, {
    required String title,
    required Widget content,
    String closeLabel = 'Close',
    IconData? icon,
    double maxHeight = 500.0,
    bool barrierDismissible = true,
  }) {
    return showDialog<void>(
      context: context,
      barrierDismissible: barrierDismissible,
      builder: (_) => InfoDialog(
        title: title,
        content: content,
        closeLabel: closeLabel,
        icon: icon,
        maxHeight: maxHeight,
      ),
    );
  }
}
