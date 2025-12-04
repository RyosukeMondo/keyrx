/// Widget for displaying the list of key mappings with a layer panel.
library;

import 'package:flutter/material.dart';

import '../../ffi/bridge.dart';
import '../../models/key_mapping.dart';
import '../../services/key_mappings_util.dart';
import '../common/styled_icon_button.dart';
import 'layer_panel.dart';

class MappingListPanel extends StatelessWidget {
  const MappingListPanel({
    super.key, required this.mappings, required this.layers,
    required this.onRemoveMapping, required this.onAddLayer,
    required this.onToggleLayer,
  });

  final Map<String, KeyMapping> mappings;
  final List<LayerInfo> layers;
  final ValueChanged<String> onRemoveMapping;
  final VoidCallback onAddLayer;
  final void Function(String name, bool active) onToggleLayer;

  @override
  Widget build(BuildContext context) {
    if (mappings.isEmpty) {
      return const Center(child: Text('No mappings yet. Add one to continue.'));
    }
    return Row(children: [
      Expanded(flex: 2, child: ListView(
        children: mappings.values.map((m) => ListTile(
          leading: const Icon(Icons.keyboard),
          title: Text('${m.from} → ${_describeMapping(m)}'),
          subtitle: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
            Text(_describeDetails(m)),
            const SizedBox(height: 4),
            Wrap(spacing: 6, runSpacing: 4, children: [
              KeyValidityChip(label: 'from ${m.from}',
                  isValid: KeyMappings.isKnownKey(m.from)),
              if (m.to != null && m.to!.isNotEmpty)
                KeyValidityChip(label: 'to ${m.to}',
                    isValid: KeyMappings.isKnownKey(m.to!)),
            ]),
          ]),
          trailing: StyledIconButton(
            icon: Icons.delete,
            onPressed: () => onRemoveMapping(m.from),
            tooltip: 'Delete mapping',
          ),
        )).toList(),
      )),
      const SizedBox(width: 12),
      Expanded(flex: 1, child: LayerPanel(
          layers: layers, onAddLayer: onAddLayer, onToggleLayer: onToggleLayer)),
    ]);
  }

  String _describeMapping(KeyMapping m) => switch (m.type) {
    KeyActionType.remap => m.to ?? '',
    KeyActionType.block => 'blocked',
    KeyActionType.pass => 'pass through',
  };

  String _describeDetails(KeyMapping m) {
    final parts = <String>[];
    if (m.layer != null && m.layer!.isNotEmpty) parts.add('Layer: ${m.layer}');
    if (m.tapHoldTap?.isNotEmpty == true && m.tapHoldHold?.isNotEmpty == true) {
      parts.add('Tap/Hold: ${m.tapHoldTap} / ${m.tapHoldHold}');
    }
    return parts.isEmpty ? m.type.label : parts.join(' • ');
  }
}

/// Chip widget for displaying key validity status.
class KeyValidityChip extends StatelessWidget {
  const KeyValidityChip({super.key, required this.label, required this.isValid});

  final String label;
  final bool isValid;

  @override
  Widget build(BuildContext context) {
    final baseColor = isValid ? Colors.green : Colors.redAccent;
    return Chip(
      label: Text(label),
      backgroundColor: baseColor.withValues(alpha: 0.12),
      labelStyle: TextStyle(color: baseColor, fontWeight: FontWeight.w600),
      visualDensity: VisualDensity.compact,
      side: BorderSide(color: baseColor.withValues(alpha: 0.5)),
    );
  }
}
