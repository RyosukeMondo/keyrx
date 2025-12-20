/// Keyboard shortcuts help dialog for the flow editor.
library;

import 'package:flutter/material.dart';

import '../services/keyboard_shortcuts.dart';

/// Shows a dialog displaying all available keyboard shortcuts.
class KeyboardShortcutsHelpDialog extends StatelessWidget {
  const KeyboardShortcutsHelpDialog({super.key});

  /// Show the keyboard shortcuts help dialog.
  static Future<void> show(BuildContext context) => showDialog<void>(
    context: context,
    builder: (context) => const KeyboardShortcutsHelpDialog(),
  );

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final shortcuts = ShortcutCatalog.byCategory;

    return Dialog(
      child: Container(
        width: 600,
        constraints: const BoxConstraints(maxHeight: 700),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            // Header
            Padding(
              padding: const EdgeInsets.all(24),
              child: Row(
                children: [
                  const Icon(Icons.keyboard_outlined, size: 28),
                  const SizedBox(width: 12),
                  Text(
                    'Keyboard Shortcuts',
                    style: theme.textTheme.headlineSmall?.copyWith(
                      fontWeight: FontWeight.w600,
                    ),
                  ),
                  const Spacer(),
                  IconButton(
                    icon: const Icon(Icons.close),
                    onPressed: () => Navigator.of(context).pop(),
                    tooltip: 'Close',
                  ),
                ],
              ),
            ),
            const Divider(height: 1),

            // Shortcuts list
            Flexible(
              child: SingleChildScrollView(
                padding: const EdgeInsets.all(24),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: shortcuts.entries
                      .map(
                        (entry) => _buildCategory(
                          theme,
                          category: entry.key,
                          shortcuts: entry.value,
                        ),
                      )
                      .toList(),
                ),
              ),
            ),

            // Footer
            const Divider(height: 1),
            Padding(
              padding: const EdgeInsets.all(16),
              child: Row(
                children: [
                  const Icon(Icons.info_outline, size: 18, color: Colors.grey),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      'Press Ctrl+? to show this dialog anytime',
                      style: theme.textTheme.bodySmall?.copyWith(
                        color: Colors.grey[600],
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildCategory(
    ThemeData theme, {
    required String category,
    required List<ShortcutInfo> shortcuts,
  }) => Padding(
    padding: const EdgeInsets.only(bottom: 24),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // Category header
        Text(
          category,
          style: theme.textTheme.titleMedium?.copyWith(
            fontWeight: FontWeight.w600,
            color: theme.colorScheme.primary,
          ),
        ),
        const SizedBox(height: 12),

        // Shortcuts in this category
        ...shortcuts.map(
          (shortcut) => _buildShortcutRow(
            theme,
            description: shortcut.description,
            keys: shortcut.shortcut,
          ),
        ),
      ],
    ),
  );

  Widget _buildShortcutRow(
    ThemeData theme, {
    required String description,
    required String keys,
  }) => Padding(
    padding: const EdgeInsets.symmetric(vertical: 6),
    child: Row(
      children: [
        // Description
        Expanded(
          flex: 3,
          child: Text(description, style: theme.textTheme.bodyMedium),
        ),

        // Keyboard shortcut
        Expanded(
          flex: 2,
          child: Align(
            alignment: Alignment.centerRight,
            child: _buildKeyBadge(theme, keys),
          ),
        ),
      ],
    ),
  );

  Widget _buildKeyBadge(ThemeData theme, String keys) {
    // Split combined keys (e.g., "Ctrl+Z" -> ["Ctrl", "Z"])
    final parts = keys.split('+');

    return Wrap(
      spacing: 4,
      children: parts
          .map(
            (key) => Container(
              padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
              decoration: BoxDecoration(
                color: theme.colorScheme.surfaceContainerHighest,
                borderRadius: BorderRadius.circular(6),
                border: Border.all(
                  color: theme.colorScheme.outline.withValues(alpha: 0.5),
                ),
                boxShadow: [
                  BoxShadow(
                    color: Colors.black.withValues(alpha: 0.1),
                    offset: const Offset(0, 1),
                    blurRadius: 2,
                  ),
                ],
              ),
              child: Text(
                key,
                style: theme.textTheme.labelMedium?.copyWith(
                  fontWeight: FontWeight.w600,
                  fontFamily: 'monospace',
                ),
              ),
            ),
          )
          .toList(),
    );
  }
}
