/// Binding configuration panel widget for the visual keymap editor.
library;

import 'package:flutter/material.dart';

import '../../models/key_mapping.dart';
import '../common/styled_icon_button.dart';
import '../common/styled_text_field.dart';

/// Widget for displaying and configuring a single key mapping (binding).
///
/// This panel shows configuration options for remapping keys, blocking keys,
/// or passing keys through. It supports advanced features like layers and
/// tap-hold configurations.
class BindingPanel extends StatelessWidget {
  const BindingPanel({
    super.key,
    required this.selectedKey,
    required this.selectedAction,
    required this.outputController,
    required this.layerController,
    required this.tapOutputController,
    required this.holdOutputController,
    required this.onActionChanged,
    required this.onApply,
  });

  final String? selectedKey;
  final KeyActionType selectedAction;
  final TextEditingController outputController;
  final TextEditingController layerController;
  final TextEditingController tapOutputController;
  final TextEditingController holdOutputController;
  final ValueChanged<KeyActionType?> onActionChanged;
  final VoidCallback onApply;

  @override
  Widget build(BuildContext context) {
    if (selectedKey == null) {
      return const Center(child: Text('Select a key to configure'));
    }
    final isRemap = selectedAction == KeyActionType.remap;
    return Padding(
      padding: const EdgeInsets.all(16),
      child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        Text('Configuring: $selectedKey',
            style: Theme.of(context).textTheme.titleLarge),
        if (!KeyMappings.isKnownKey(selectedKey!)) ...[
          const SizedBox(height: 4),
          Text('$selectedKey is not in the canonical key list.',
              style: TextStyle(color: Colors.orange.shade700)),
        ],
        const SizedBox(height: 16),
        Row(children: [
          DropdownButton<KeyActionType>(
            value: selectedAction,
            items: KeyActionType.values
                .map((t) => DropdownMenuItem(value: t, child: Text(t.label)))
                .toList(),
            onChanged: onActionChanged,
          ),
          const SizedBox(width: 16),
          Expanded(
            child: StyledTextField(
              controller: outputController,
              labelText: isRemap ? 'Remap to key' : 'Optional note',
              hintText: isRemap ? 'e.g., Escape' : 'Leave blank for block/pass',
              enabled: isRemap,
            ),
          ),
          const SizedBox(width: 12),
          FilledButton.icon(
              icon: const Icon(Icons.check), label: const Text('Apply'),
              onPressed: onApply),
        ]),
        const SizedBox(height: 12),
        Row(children: [
          Expanded(
            child: StyledTextField(
              controller: layerController,
              labelText: 'Layer (optional)',
              hintText: 'e.g., navigation',
            ),
          ),
          const SizedBox(width: 12),
          Expanded(
            child: StyledTextField(
              controller: tapOutputController,
              labelText: 'Tap output (tap-hold)',
              hintText: 'e.g., Escape',
            ),
          ),
          const SizedBox(width: 12),
          Expanded(
            child: StyledTextField(
              controller: holdOutputController,
              labelText: 'Hold output (tap-hold)',
              hintText: 'e.g., Ctrl',
            ),
          ),
        ]),
      ]),
    );
  }
}
