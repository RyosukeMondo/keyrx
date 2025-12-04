import 'package:flutter/material.dart';
import 'confirmation_dialog.dart';
import 'input_dialog.dart';
import 'selection_dialog.dart';
import 'info_dialog.dart';
import 'multi_action_dialog.dart';

/// Convenient helper functions for showing common dialogs.
///
/// Provides quick access to standard dialog patterns without
/// needing to call the dialog classes directly.
class DialogHelpers {
  DialogHelpers._();

  /// Shows a simple confirmation dialog.
  ///
  /// Returns `true` if confirmed, `false` if cancelled, `null` if dismissed.
  static Future<bool?> confirm(
    BuildContext context, {
    required String title,
    required String message,
    String confirmLabel = 'Confirm',
    String cancelLabel = 'Cancel',
    bool isDestructive = false,
    IconData? icon,
  }) {
    return ConfirmationDialog.show(
      context,
      title: title,
      message: message,
      confirmLabel: confirmLabel,
      cancelLabel: cancelLabel,
      isDestructive: isDestructive,
      icon: icon,
    );
  }

  /// Shows a destructive action confirmation dialog.
  ///
  /// Styled with error colors to indicate a dangerous action.
  /// Returns `true` if confirmed, `false` if cancelled, `null` if dismissed.
  static Future<bool?> confirmDelete(
    BuildContext context, {
    required String title,
    required String message,
    String confirmLabel = 'Delete',
    String cancelLabel = 'Cancel',
  }) {
    return ConfirmationDialog.show(
      context,
      title: title,
      message: message,
      confirmLabel: confirmLabel,
      cancelLabel: cancelLabel,
      isDestructive: true,
      icon: Icons.delete_outline,
    );
  }

  /// Shows a clear/reset confirmation dialog.
  ///
  /// Styled with warning colors for clearing data.
  /// Returns `true` if confirmed, `false` if cancelled, `null` if dismissed.
  static Future<bool?> confirmClear(
    BuildContext context, {
    required String title,
    required String message,
    String confirmLabel = 'Clear',
    String cancelLabel = 'Cancel',
  }) {
    return ConfirmationDialog.show(
      context,
      title: title,
      message: message,
      confirmLabel: confirmLabel,
      cancelLabel: cancelLabel,
      isDestructive: true,
      icon: Icons.warning_amber,
    );
  }

  /// Shows an input dialog for collecting text.
  ///
  /// Returns the entered text if confirmed, `null` if cancelled or dismissed.
  static Future<String?> input(
    BuildContext context, {
    required String title,
    required String labelText,
    String? message,
    String? hintText,
    String initialValue = '',
    String? Function(String?)? validator,
    IconData? icon,
    int maxLines = 1,
  }) {
    return InputDialog.show(
      context,
      title: title,
      message: message,
      labelText: labelText,
      hintText: hintText,
      initialValue: initialValue,
      validator: validator,
      icon: icon,
      maxLines: maxLines,
    );
  }

  /// Shows an input dialog for file paths.
  ///
  /// Pre-configured for entering file paths with validation.
  /// Returns the entered path if confirmed, `null` if cancelled or dismissed.
  static Future<String?> inputPath(
    BuildContext context, {
    required String title,
    String message = 'Enter the file path',
    String initialValue = '',
    String? hintText,
  }) {
    return InputDialog.show(
      context,
      title: title,
      message: message,
      labelText: 'Path',
      hintText: hintText ?? 'e.g., scripts/my-config.rhai',
      initialValue: initialValue,
      validator: (value) {
        if (value == null || value.trim().isEmpty) {
          return 'Path cannot be empty';
        }
        return null;
      },
      icon: Icons.folder_outlined,
    );
  }

  /// Shows a selection dialog for choosing from a list.
  ///
  /// Returns the selected value if an option is chosen, `null` if cancelled or dismissed.
  static Future<T?> select<T>(
    BuildContext context, {
    required String title,
    String? message,
    required List<SelectionOption<T>> options,
    T? selectedOption,
    IconData? icon,
  }) {
    return SelectionDialog.show<T>(
      context,
      title: title,
      message: message,
      options: options,
      selectedOption: selectedOption,
      icon: icon,
    );
  }

  /// Shows an information/help dialog.
  ///
  /// Displays read-only content with optional scrolling.
  static Future<void> info(
    BuildContext context, {
    required String title,
    required Widget content,
    String closeLabel = 'Close',
    IconData? icon,
    double maxHeight = 500.0,
  }) {
    return InfoDialog.show(
      context,
      title: title,
      content: content,
      closeLabel: closeLabel,
      icon: icon,
      maxHeight: maxHeight,
    );
  }

  /// Shows a help dialog with an info icon.
  ///
  /// Convenience method for showing help content.
  static Future<void> help(
    BuildContext context, {
    required String title,
    required Widget content,
    double maxHeight = 500.0,
  }) {
    return InfoDialog.show(
      context,
      title: title,
      content: content,
      closeLabel: 'Got it',
      icon: Icons.help_outline,
      maxHeight: maxHeight,
    );
  }

  /// Shows a multi-action dialog with custom actions.
  ///
  /// Returns the value associated with the selected action, or `null` if dismissed.
  static Future<T?> multiAction<T>(
    BuildContext context, {
    required String title,
    required String message,
    required List<DialogAction<T>> actions,
    IconData? icon,
  }) {
    return MultiActionDialog.show<T>(
      context,
      title: title,
      message: message,
      actions: actions,
      icon: icon,
    );
  }

  /// Shows a dialog with three actions: Cancel, Primary, Secondary.
  ///
  /// Common pattern for sync/conflict resolution dialogs.
  /// Returns 1 for primary action, 2 for secondary action, null for cancel/dismiss.
  static Future<int?> threeWayChoice(
    BuildContext context, {
    required String title,
    required String message,
    required String primaryLabel,
    required String secondaryLabel,
    String cancelLabel = 'Cancel',
    IconData? icon,
  }) {
    return MultiActionDialog.show<int>(
      context,
      title: title,
      message: message,
      actions: [
        DialogAction<int>(label: cancelLabel, value: 0),
        DialogAction<int>(label: primaryLabel, value: 1, isPrimary: true),
        DialogAction<int>(label: secondaryLabel, value: 2),
      ],
      icon: icon,
    );
  }
}
