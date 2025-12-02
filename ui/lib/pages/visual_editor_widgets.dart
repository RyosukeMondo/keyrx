/// Widgets for the visual editor page.
///
/// Contains reusable components for the visual keymap editor.
library;

import 'package:flutter/material.dart';

import '../services/rhai_generator.dart';
import '../widgets/visual_keyboard.dart';

export '../services/rhai_generator.dart' show TapHoldConfig;
export '../widgets/visual_keyboard.dart' show RemapConfig, MappingType;

/// Warning banner shown when script has advanced features.
class AdvancedFeaturesWarning extends StatelessWidget {
  const AdvancedFeaturesWarning({
    super.key,
    required this.onViewCode,
  });

  final VoidCallback onViewCode;

  @override
  Widget build(BuildContext context) {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      color: Theme.of(context).colorScheme.errorContainer,
      child: Row(
        children: [
          Icon(Icons.warning_amber,
              color: Theme.of(context).colorScheme.onErrorContainer),
          const SizedBox(width: 8),
          Expanded(
            child: Text(
              'This script contains advanced features that cannot be edited visually. '
              'Use the code view to edit.',
              style: TextStyle(
                  color: Theme.of(context).colorScheme.onErrorContainer),
            ),
          ),
          TextButton(
            onPressed: onViewCode,
            child: const Text('View Code'),
          ),
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
  });

  final List<RemapConfig> mappings;
  final List<TapHoldConfig> tapHoldConfigs;
  final void Function(int index) onMappingDeleted;
  final void Function(int index) onTapHoldDeleted;
  final void Function(String keyId) onMappingSelected;
  final VoidCallback onClearAll;

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: BoxDecoration(
        border: Border(
          left: BorderSide(
            color: Theme.of(context).colorScheme.outlineVariant,
          ),
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          _buildHeader(context),
          Expanded(child: _buildMappingList(context)),
          if (tapHoldConfigs.isNotEmpty) ...[
            _buildTapHoldHeader(context),
            ...List.generate(
                tapHoldConfigs.length, (i) => _buildTapHoldTile(context, i)),
          ],
        ],
      ),
    );
  }

  Widget _buildHeader(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(12),
      color: Theme.of(context).colorScheme.surfaceContainerHighest,
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(
            'Mappings (${mappings.length})',
            style: Theme.of(context).textTheme.titleSmall,
          ),
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
    return ListTile(
      dense: true,
      leading: Icon(
        Icons.keyboard_arrow_right,
        color: Theme.of(context).colorScheme.primary,
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

  Widget _buildTapHoldHeader(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(12),
      color: Theme.of(context).colorScheme.surfaceContainerHighest,
      child: Text(
        'Tap-Hold (${tapHoldConfigs.length})',
        style: Theme.of(context).textTheme.titleSmall,
      ),
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

/// Code editor view with toolbar.
class CodeEditorView extends StatelessWidget {
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
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        _buildToolbar(context),
        Expanded(child: _buildEditor(context)),
      ],
    );
  }

  Widget _buildToolbar(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      color: Theme.of(context).colorScheme.surfaceContainerHighest,
      child: Row(
        children: [
          Icon(Icons.code,
              size: 20, color: Theme.of(context).colorScheme.primary),
          const SizedBox(width: 8),
          Text(
            'Rhai Script',
            style: Theme.of(context).textTheme.titleSmall,
          ),
          const Spacer(),
          if (isModified)
            Padding(
              padding: const EdgeInsets.only(right: 8),
              child: Chip(
                label: const Text('Modified'),
                labelStyle: TextStyle(
                  fontSize: 11,
                  color: Theme.of(context).colorScheme.onSecondaryContainer,
                ),
                backgroundColor:
                    Theme.of(context).colorScheme.secondaryContainer,
                padding: EdgeInsets.zero,
                visualDensity: VisualDensity.compact,
              ),
            ),
          TextButton.icon(
            icon: const Icon(Icons.sync, size: 18),
            label: const Text('Parse to Visual'),
            onPressed: isModified ? onParseToVisual : null,
          ),
        ],
      ),
    );
  }

  Widget _buildEditor(BuildContext context) {
    return Container(
      color: Theme.of(context).colorScheme.surfaceContainerLowest,
      child: TextField(
        controller: controller,
        maxLines: null,
        expands: true,
        textAlignVertical: TextAlignVertical.top,
        style: const TextStyle(
          fontFamily: 'monospace',
          fontSize: 13,
        ),
        decoration: const InputDecoration(
          border: InputBorder.none,
          contentPadding: EdgeInsets.all(16),
        ),
        onChanged: onCodeChanged,
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
        title: const Text('Unsaved Code Changes'),
        content: const Text(
          'The code has been modified. Do you want to parse it back to visual mode, '
          'or discard changes and regenerate from visual config?',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx),
            child: const Text('Cancel'),
          ),
          TextButton(
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
    return showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Clear Configuration'),
        content: const Text(
          'Are you sure you want to clear all mappings? This cannot be undone.',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx, false),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () => Navigator.pop(ctx, true),
            child: const Text('Clear'),
          ),
        ],
      ),
    );
  }

  /// Show load script dialog.
  static Future<String?> showLoadDialog(BuildContext context) {
    final controller = TextEditingController(text: 'scripts/');
    return showDialog<String>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Load Script'),
        content: SizedBox(
          width: 400,
          child: TextField(
            controller: controller,
            decoration: const InputDecoration(
              labelText: 'Script path',
              hintText: 'scripts/my-config.rhai',
            ),
            autofocus: true,
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () => Navigator.pop(ctx, controller.text),
            child: const Text('Load'),
          ),
        ],
      ),
    );
  }

  /// Show save script dialog.
  static Future<String?> showSaveDialog(
    BuildContext context,
    String suggestedPath,
  ) {
    final controller = TextEditingController(text: suggestedPath);
    return showDialog<String>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Save Script'),
        content: SizedBox(
          width: 400,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              TextField(
                controller: controller,
                decoration: const InputDecoration(
                  labelText: 'Save path',
                  hintText: 'scripts/my-config.rhai',
                ),
                autofocus: true,
              ),
              const SizedBox(height: 16),
              Text(
                'The script will be saved and can be loaded with:\n'
                'keyrx run --script <path>',
                style: Theme.of(ctx).textTheme.bodySmall,
              ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () => Navigator.pop(ctx, controller.text),
            child: const Text('Save'),
          ),
        ],
      ),
    );
  }
}

/// Action to take when syncing code and visual views.
enum SyncAction { parse, discard }
