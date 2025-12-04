import 'package:flutter/material.dart';

/// A standardized selection dialog for choosing from multiple options.
///
/// Provides a consistent UI for selecting a single item from a list.
class SelectionDialog<T> extends StatelessWidget {
  const SelectionDialog({
    super.key,
    required this.title,
    this.message,
    required this.options,
    this.selectedOption,
    this.icon,
  });

  final String title;
  final String? message;
  final List<SelectionOption<T>> options;
  final T? selectedOption;
  final IconData? icon;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final scheme = theme.colorScheme;

    return AlertDialog(
      backgroundColor: scheme.surface,
      titlePadding: const EdgeInsets.fromLTRB(24, 20, 24, 12),
      contentPadding: EdgeInsets.zero,
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
      content: SizedBox(
        width: double.maxFinite,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            if (message != null)
              Padding(
                padding: const EdgeInsets.fromLTRB(24, 0, 24, 16),
                child: Text(message!, style: theme.textTheme.bodyMedium),
              ),
            Flexible(
              child: ListView.builder(
                shrinkWrap: true,
                itemCount: options.length,
                itemBuilder: (context, index) {
                  final option = options[index];
                  final isSelected = option.value == selectedOption;
                  return ListTile(
                    leading: option.icon != null
                        ? Icon(option.icon)
                        : (isSelected ? const Icon(Icons.check) : null),
                    title: Text(option.label),
                    subtitle:
                        option.description != null ? Text(option.description!) : null,
                    selected: isSelected,
                    onTap: () => Navigator.of(context).pop(option.value),
                  );
                },
              ),
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Cancel'),
        ),
      ],
    );
  }

  /// Shows a selection dialog and returns the selected value.
  ///
  /// Returns the selected value if an option is chosen, `null` if cancelled or dismissed.
  static Future<T?> show<T>(
    BuildContext context, {
    required String title,
    String? message,
    required List<SelectionOption<T>> options,
    T? selectedOption,
    IconData? icon,
    bool barrierDismissible = true,
  }) {
    return showDialog<T>(
      context: context,
      barrierDismissible: barrierDismissible,
      builder: (_) => SelectionDialog<T>(
        title: title,
        message: message,
        options: options,
        selectedOption: selectedOption,
        icon: icon,
      ),
    );
  }
}

/// Represents an option in a selection dialog.
class SelectionOption<T> {
  const SelectionOption({
    required this.value,
    required this.label,
    this.description,
    this.icon,
  });

  final T value;
  final String label;
  final String? description;
  final IconData? icon;
}
