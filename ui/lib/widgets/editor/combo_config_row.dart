/// Widget for displaying the combo configuration row.
library;

import 'package:flutter/material.dart';

import '../../models/key_mapping.dart';
import '../common/styled_icon_button.dart';
import '../common/styled_text_field.dart';

class ComboConfigRow extends StatelessWidget {
  const ComboConfigRow({
    super.key, required this.comboKeysController, required this.comboOutputController,
    required this.combos, required this.onAddCombo, required this.onRemoveCombo,
  });

  final TextEditingController comboKeysController;
  final TextEditingController comboOutputController;
  final List<ComboMapping> combos;
  final VoidCallback onAddCombo;
  final ValueChanged<int> onRemoveCombo;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
          Text('Combos', style: Theme.of(context).textTheme.titleMedium),
          const SizedBox(height: 8),
          Row(children: [
            Expanded(
              child: StyledTextField(
                controller: comboKeysController,
                labelText: 'Keys (comma-separated)',
                hintText: 'e.g., A,S',
              ),
            ),
            const SizedBox(width: 12),
            Expanded(
              child: StyledTextField(
                controller: comboOutputController,
                labelText: 'Output',
                hintText: 'e.g., Ctrl',
              ),
            ),
            const SizedBox(width: 12),
            FilledButton.icon(onPressed: onAddCombo, icon: const Icon(Icons.add),
                label: const Text('Add Combo')),
          ]),
          const SizedBox(height: 8),
          if (combos.isEmpty) const Text('No combos defined.')
          else ...combos.asMap().entries.map((e) => ListTile(
            dense: true, leading: const Icon(Icons.merge_type),
            title: Text('${e.value.keys.join(" + ")} → ${e.value.output}'),
            trailing: StyledIconButton(
              icon: Icons.delete,
              onPressed: () => onRemoveCombo(e.key),
              tooltip: 'Delete combo',
            ),
          )),
        ]),
      ),
    );
  }
}
