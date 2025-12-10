import 'package:flutter/material.dart';

/// A standardized input dialog for text entry.
///
/// Provides a consistent UI for collecting text input from users.
class InputDialog extends StatefulWidget {
  const InputDialog({
    super.key,
    required this.title,
    this.message,
    required this.labelText,
    this.hintText,
    this.initialValue = '',
    this.confirmLabel = 'OK',
    this.cancelLabel = 'Cancel',
    this.validator,
    this.icon,
    this.maxLines = 1,
  });

  final String title;
  final String? message;
  final String labelText;
  final String? hintText;
  final String initialValue;
  final String confirmLabel;
  final String cancelLabel;
  final String? Function(String?)? validator;
  final IconData? icon;
  final int maxLines;

  @override
  State<InputDialog> createState() => _InputDialogState();

  /// Shows an input dialog and returns the entered text.
  ///
  /// Returns the entered text if confirmed, `null` if cancelled or dismissed.
  static Future<String?> show(
    BuildContext context, {
    required String title,
    String? message,
    required String labelText,
    String? hintText,
    String initialValue = '',
    String confirmLabel = 'OK',
    String cancelLabel = 'Cancel',
    String? Function(String?)? validator,
    IconData? icon,
    int maxLines = 1,
    bool barrierDismissible = true,
  }) {
    return showDialog<String>(
      context: context,
      barrierDismissible: barrierDismissible,
      builder: (_) => InputDialog(
        title: title,
        message: message,
        labelText: labelText,
        hintText: hintText,
        initialValue: initialValue,
        confirmLabel: confirmLabel,
        cancelLabel: cancelLabel,
        validator: validator,
        icon: icon,
        maxLines: maxLines,
      ),
    );
  }
}

class _InputDialogState extends State<InputDialog> {
  late final TextEditingController _controller;
  String? _errorText;

  @override
  void initState() {
    super.initState();
    _controller = TextEditingController(text: widget.initialValue);
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  void _handleConfirm() {
    final value = _controller.text;
    if (widget.validator != null) {
      final error = widget.validator!(value);
      if (error != null) {
        setState(() => _errorText = error);
        return;
      }
    }
    Navigator.of(context).pop(value);
  }

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
          if (widget.icon != null) ...[
            Icon(widget.icon, color: scheme.primary),
            const SizedBox(width: 12),
          ],
          Expanded(
            child: Text(widget.title, style: theme.textTheme.titleLarge),
          ),
        ],
      ),
      content: SizedBox(
        width: 400,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            if (widget.message != null) ...[
              Text(widget.message!, style: theme.textTheme.bodyMedium),
              const SizedBox(height: 16),
            ],
            TextField(
              controller: _controller,
              decoration: InputDecoration(
                labelText: widget.labelText,
                hintText: widget.hintText,
                errorText: _errorText,
                border: const OutlineInputBorder(),
              ),
              autofocus: true,
              maxLines: widget.maxLines,
              onSubmitted: (_) => _handleConfirm(),
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: Text(widget.cancelLabel),
        ),
        FilledButton(
          onPressed: _handleConfirm,
          child: Text(widget.confirmLabel),
        ),
      ],
    );
  }
}
