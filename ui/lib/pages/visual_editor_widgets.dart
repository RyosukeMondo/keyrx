/// Widgets for the visual editor page.
///
/// Contains reusable components for the visual keymap editor.
library;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../services/rhai_generator.dart';
import '../services/api_docs_service.dart';
import '../widgets/visual_keyboard.dart';
import '../widgets/scripting/api_browser.dart';
import '../widgets/common/dialogs/dialogs.dart';

export '../services/rhai_generator.dart' show TapHoldConfig;
export '../widgets/visual_keyboard.dart' show RemapConfig, MappingType;

/// Lightweight inline message used for helper and validation text.
class InlineMessage extends StatelessWidget {
  const InlineMessage({
    super.key,
    required this.message,
    this.variant = InlineMessageVariant.info,
    this.icon,
  });

  final String message;
  final InlineMessageVariant variant;
  final IconData? icon;

  @override
  Widget build(BuildContext context) {
    final ColorScheme scheme = Theme.of(context).colorScheme;
    final Color foreground = switch (variant) {
      InlineMessageVariant.info => scheme.onSurfaceVariant,
      InlineMessageVariant.success => scheme.onSecondaryContainer,
      InlineMessageVariant.warning => scheme.onTertiaryContainer,
      InlineMessageVariant.error => scheme.onErrorContainer,
    };
    final Color background = switch (variant) {
      InlineMessageVariant.info => scheme.surfaceContainerLowest,
      InlineMessageVariant.success => scheme.secondaryContainer,
      InlineMessageVariant.warning => scheme.tertiaryContainer,
      InlineMessageVariant.error => scheme.errorContainer,
    };
    final IconData icon =
        this.icon ??
        switch (variant) {
          InlineMessageVariant.info => Icons.info_outline,
          InlineMessageVariant.success => Icons.check_circle_outline,
          InlineMessageVariant.warning => Icons.warning_amber_outlined,
          InlineMessageVariant.error => Icons.error_outline,
        };

    return Container(
      width: double.infinity,
      margin: const EdgeInsets.only(top: 8),
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      decoration: BoxDecoration(
        color: background,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: scheme.outlineVariant),
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Icon(icon, size: 18, color: foreground),
          const SizedBox(width: 8),
          Expanded(
            child: Text(
              message,
              style: Theme.of(
                context,
              ).textTheme.bodyMedium?.copyWith(color: foreground),
            ),
          ),
        ],
      ),
    );
  }
}

/// Message type for InlineMessage.
enum InlineMessageVariant { info, success, warning, error }

/// Warning banner shown when script has advanced features.
class AdvancedFeaturesWarning extends StatelessWidget {
  const AdvancedFeaturesWarning({super.key, required this.onViewCode});

  final VoidCallback onViewCode;

  @override
  Widget build(BuildContext context) {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      color: Theme.of(context).colorScheme.errorContainer,
      child: Row(
        children: [
          Icon(
            Icons.warning_amber,
            color: Theme.of(context).colorScheme.onErrorContainer,
          ),
          const SizedBox(width: 8),
          Expanded(
            child: Text(
              'This script contains advanced features that cannot be edited visually. '
              'Use the code view to edit.',
              style: TextStyle(
                color: Theme.of(context).colorScheme.onErrorContainer,
              ),
            ),
          ),
          TextButton(onPressed: onViewCode, child: const Text('View Code')),
        ],
      ),
    );
  }
}

/// Side panel showing list of mappings and tap-hold configs.
class MappingPanel extends StatelessWidget {
  const MappingPanel({
    super.key,
    required this.mappings,
    required this.tapHoldConfigs,
    required this.onMappingDeleted,
    required this.onTapHoldDeleted,
    required this.onMappingSelected,
    required this.onClearAll,
    this.selectedKeyId,
  });

  final List<RemapConfig> mappings;
  final List<TapHoldConfig> tapHoldConfigs;
  final void Function(int index) onMappingDeleted;
  final void Function(int index) onTapHoldDeleted;
  final void Function(String keyId) onMappingSelected;
  final VoidCallback onClearAll;
  final String? selectedKeyId;

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: BoxDecoration(
        border: Border(
          top: BorderSide(color: Theme.of(context).colorScheme.outlineVariant),
        ),
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          // Remaps Column
          Expanded(
            flex: 3,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                _buildHeader(
                  context,
                  'Mappings',
                  mappings.length,
                  showClear: true,
                ),
                Expanded(child: _buildMappingList(context)),
              ],
            ),
          ),
          const VerticalDivider(width: 1),
          // Tap-Hold / Combos Column
          Expanded(
            flex: 2,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                _buildHeader(
                  context,
                  'Tap-Hold & Combos',
                  tapHoldConfigs.length,
                ),
                Expanded(
                  child: tapHoldConfigs.isEmpty
                      ? Center(
                          child: Text(
                            'No advanced configs',
                            style: Theme.of(context).textTheme.bodySmall
                                ?.copyWith(
                                  color: Theme.of(
                                    context,
                                  ).colorScheme.onSurfaceVariant,
                                ),
                          ),
                        )
                      : ListView.builder(
                          itemCount: tapHoldConfigs.length,
                          itemBuilder: (context, index) =>
                              _buildTapHoldTile(context, index),
                        ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildHeader(
    BuildContext context,
    String title,
    int count, {
    bool showClear = false,
  }) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      color: Theme.of(context).colorScheme.surfaceContainerHighest,
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(
            '$title ($count)',
            style: Theme.of(context).textTheme.titleSmall,
          ),
          if (showClear)
            IconButton(
              icon: const Icon(Icons.delete_sweep, size: 20),
              tooltip: 'Clear All',
              onPressed: mappings.isEmpty ? null : onClearAll,
            ),
        ],
      ),
    );
  }

  Widget _buildMappingList(BuildContext context) {
    if (mappings.isEmpty) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Text(
            'No mappings yet.\n\nDrag from one key to another on the keyboard to create a mapping.',
            textAlign: TextAlign.center,
            style: Theme.of(context).textTheme.bodySmall?.copyWith(
              color: Theme.of(context).colorScheme.onSurfaceVariant,
            ),
          ),
        ),
      );
    }
    return ListView.builder(
      itemCount: mappings.length,
      itemBuilder: (context, index) => _buildMappingTile(context, index),
    );
  }

  Widget _buildMappingTile(BuildContext context, int index) {
    final mapping = mappings[index];
    final isSelected = mapping.sourceKeyId == selectedKeyId;

    return ListTile(
      dense: true,
      selected: isSelected,
      selectedTileColor: Theme.of(context).colorScheme.primaryContainer,
      leading: Icon(
        Icons.keyboard_arrow_right,
        color: isSelected
            ? Theme.of(context).colorScheme.onPrimaryContainer
            : Theme.of(context).colorScheme.primary,
      ),
      title: Text('${mapping.sourceKeyId} → ${mapping.targetKeyId}'),
      subtitle: Text(_getMappingTypeLabel(mapping.type)),
      trailing: IconButton(
        icon: const Icon(Icons.close, size: 18),
        onPressed: () => onMappingDeleted(index),
      ),
      onTap: () => onMappingSelected(mapping.sourceKeyId),
    );
  }

  Widget _buildTapHoldTile(BuildContext context, int index) {
    final config = tapHoldConfigs[index];
    return ListTile(
      dense: true,
      leading: Icon(
        Icons.touch_app,
        color: Theme.of(context).colorScheme.secondary,
      ),
      title: Text(config.triggerKey),
      subtitle: Text('Tap: ${config.tapAction}, Hold: ${config.holdAction}'),
      trailing: IconButton(
        icon: const Icon(Icons.close, size: 18),
        onPressed: () => onTapHoldDeleted(index),
      ),
    );
  }

  String _getMappingTypeLabel(MappingType type) {
    return switch (type) {
      MappingType.simple => 'Remap',
      MappingType.tapHold => 'Tap-Hold',
      MappingType.layer => 'Layer',
    };
  }
}

/// Code editor view with toolbar and optional documentation panel.
class CodeEditorView extends StatefulWidget {
  const CodeEditorView({
    super.key,
    required this.controller,
    required this.isModified,
    required this.onCodeChanged,
    required this.onParseToVisual,
  });

  final TextEditingController controller;
  final bool isModified;
  final void Function(String) onCodeChanged;
  final VoidCallback onParseToVisual;

  @override
  State<CodeEditorView> createState() => _CodeEditorViewState();
}

class _CodeEditorViewState extends State<CodeEditorView> {
  bool _showDocs = false;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        _buildToolbar(context),
        Expanded(child: _buildContent(context)),
      ],
    );
  }

  Widget _buildContent(BuildContext context) {
    if (!_showDocs) {
      return _buildEditor(context);
    }

    return Row(
      children: [
        Expanded(flex: 2, child: _buildEditor(context)),
        const VerticalDivider(width: 1),
        Expanded(flex: 1, child: _buildDocsPanel(context)),
      ],
    );
  }

  Widget _buildDocsPanel(BuildContext context) {
    final docsService = context.watch<ApiDocsService>();

    return Container(
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface,
        border: Border(left: BorderSide(color: Theme.of(context).dividerColor)),
      ),
      child: ApiBrowser(docsService: docsService),
    );
  }

  Widget _buildToolbar(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      color: Theme.of(context).colorScheme.surfaceContainerHighest,
      child: Row(
        children: [
          Icon(
            Icons.code,
            size: 20,
            color: Theme.of(context).colorScheme.primary,
          ),
          const SizedBox(width: 8),
          Text('Rhai Script', style: Theme.of(context).textTheme.titleSmall),
          const Spacer(),
          if (widget.isModified)
            Padding(
              padding: const EdgeInsets.only(right: 8),
              child: Chip(
                label: const Text('Modified'),
                labelStyle: TextStyle(
                  fontSize: 11,
                  color: Theme.of(context).colorScheme.onSecondaryContainer,
                ),
                backgroundColor: Theme.of(
                  context,
                ).colorScheme.secondaryContainer,
                padding: EdgeInsets.zero,
                visualDensity: VisualDensity.compact,
              ),
            ),
          IconButton(
            icon: Icon(_showDocs ? Icons.menu_book : Icons.menu_book_outlined),
            tooltip: _showDocs ? 'Hide Documentation' : 'Show Documentation',
            onPressed: () {
              setState(() {
                _showDocs = !_showDocs;
              });
            },
          ),
          TextButton.icon(
            icon: const Icon(Icons.sync, size: 18),
            label: const Text('Parse to Visual'),
            onPressed: widget.isModified ? widget.onParseToVisual : null,
          ),
        ],
      ),
    );
  }

  Widget _buildEditor(BuildContext context) {
    return Container(
      color: Theme.of(context).colorScheme.surfaceContainerLowest,
      child: TextField(
        controller: widget.controller,
        maxLines: null,
        expands: true,
        textAlignVertical: TextAlignVertical.top,
        style: const TextStyle(fontFamily: 'monospace', fontSize: 13),
        decoration: const InputDecoration(
          border: InputBorder.none,
          contentPadding: EdgeInsets.all(16),
        ),
        onChanged: widget.onCodeChanged,
      ),
    );
  }
}

/// Dialog helpers for visual editor.
class VisualEditorDialogs {
  /// Show sync warning when code is modified.
  static Future<SyncAction?> showSyncWarning(BuildContext context) {
    return showDialog<SyncAction>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Row(
          children: [
            Icon(Icons.warning_amber),
            SizedBox(width: 12),
            Text('Unsaved Code Changes'),
          ],
        ),
        content: const Text(
          'The code has been modified. Do you want to parse it back to visual mode, '
          'or discard changes and regenerate from visual config?',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(ctx, SyncAction.parse),
            child: const Text('Parse Code'),
          ),
          TextButton(
            onPressed: () => Navigator.pop(ctx, SyncAction.discard),
            child: const Text('Discard'),
          ),
        ],
      ),
    );
  }

  /// Show confirmation dialog for clearing config.
  static Future<bool?> showClearConfirmation(BuildContext context) {
    return DialogHelpers.confirmClear(
      context,
      title: 'Clear Configuration',
      message:
          'Are you sure you want to clear all mappings? This cannot be undone.',
      confirmLabel: 'Clear',
    );
  }

  /// Show load script dialog.
  static Future<String?> showLoadDialog(BuildContext context) {
    return DialogHelpers.inputPath(
      context,
      title: 'Load Script',
      message: 'Enter the path to the script to load',
      initialValue: 'scripts/',
      hintText: 'scripts/my-config.rhai',
    );
  }

  /// Show save script dialog.
  static Future<String?> showSaveDialog(
    BuildContext context,
    String suggestedPath,
  ) {
    return DialogHelpers.inputPath(
      context,
      title: 'Save Script',
      message:
          'Enter the path where the script should be saved.\n'
          'The script can be loaded with: keyrx run --script <path>',
      initialValue: suggestedPath,
      hintText: 'scripts/my-config.rhai',
    );
  }
}

/// Action to take when syncing code and visual views.
enum SyncAction { parse, discard }
