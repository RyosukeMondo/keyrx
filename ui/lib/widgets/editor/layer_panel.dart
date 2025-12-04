/// Layer management panel widget.
///
/// Displays and manages keyboard layers with
/// activation/deactivation controls.

import 'package:flutter/material.dart';

/// Layer information model.
class LayerInfo {
  final String name;
  final bool active;
  final int priority;

  const LayerInfo({
    required this.name,
    required this.active,
    required this.priority,
  });
}

/// Layer management panel widget.
class LayerPanel extends StatelessWidget {
  final List<LayerInfo> layers;
  final void Function(String name, bool active)? onToggleLayer;
  final VoidCallback? onAddLayer;

  const LayerPanel({
    super.key,
    required this.layers,
    this.onToggleLayer,
    this.onAddLayer,
  });

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Padding(
            padding: const EdgeInsets.all(12),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(
                  'Layers',
                  style: Theme.of(context).textTheme.titleMedium,
                ),
                IconButton(
                  icon: const Icon(Icons.add),
                  onPressed: onAddLayer,
                  tooltip: 'Add Layer',
                ),
              ],
            ),
          ),
          const Divider(height: 1),
          Expanded(
            child: ListView.builder(
              itemCount: layers.length,
              itemBuilder: (context, index) {
                final layer = layers[index];
                return _buildLayerTile(context, layer);
              },
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildLayerTile(BuildContext context, LayerInfo layer) {
    return ListTile(
      leading: Icon(
        layer.active ? Icons.layers : Icons.layers_outlined,
        color: layer.active ? Colors.blue : Colors.grey,
      ),
      title: Text(layer.name),
      subtitle: Text('Priority: ${layer.priority}'),
      trailing: Switch(
        value: layer.active,
        onChanged: (value) => onToggleLayer?.call(layer.name, value),
      ),
    );
  }
}
