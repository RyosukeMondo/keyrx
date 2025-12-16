/// Node details dialog for inspecting flow graph nodes.
library;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../models/flow/flow_graph.dart';
import '../state/keyflow_providers.dart';

/// Shows detailed information about a flow graph node.
class NodeDetailsDialog extends ConsumerWidget {
  const NodeDetailsDialog({required this.nodeId, super.key});

  final String nodeId;

  /// Show the node details dialog.
  static Future<void> show(BuildContext context, String nodeId) =>
      showDialog<void>(
        context: context,
        builder: (context) => NodeDetailsDialog(nodeId: nodeId),
      );

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final theme = Theme.of(context);
    final graphAsync = ref.watch(flowGraphProvider);

    return Dialog(
      child: Container(
        width: 500,
        constraints: const BoxConstraints(maxHeight: 600),
        child: graphAsync.when(
          data: (graph) {
            final node = graph.nodes.firstWhere(
              (n) => n.nodeId == nodeId,
              orElse: () => throw Exception('Node $nodeId not found'),
            );

            return Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                _buildHeader(context, theme, node),
                const Divider(height: 1),
                Flexible(
                  child: SingleChildScrollView(
                    padding: const EdgeInsets.all(24),
                    child: _buildContent(theme, node, graph),
                  ),
                ),
                const Divider(height: 1),
                _buildFooter(context),
              ],
            );
          },
          loading: () => const Center(
            child: Padding(
              padding: EdgeInsets.all(48),
              child: CircularProgressIndicator(),
            ),
          ),
          error: (error, _) => _buildError(context, theme, error),
        ),
      ),
    );
  }

  Widget _buildHeader(BuildContext context, ThemeData theme, FlowNode node) =>
      Padding(
        padding: const EdgeInsets.all(24),
        child: Row(
          children: [
            Icon(
              _getNodeIcon(node),
              size: 28,
              color: theme.colorScheme.primary,
            ),
            const SizedBox(width: 12),
            Expanded(
              child: Text(
                _getNodeTitle(node),
                style: theme.textTheme.headlineSmall?.copyWith(
                  fontWeight: FontWeight.w600,
                ),
              ),
            ),
            IconButton(
              icon: const Icon(Icons.close),
              onPressed: () => Navigator.of(context).pop(),
              tooltip: 'Close',
            ),
          ],
        ),
      );

  Widget _buildContent(ThemeData theme, FlowNode node, FlowGraph graph) =>
      node.map(
        device: (device) => _buildDeviceDetails(theme, device, graph),
        hardwareProfile: (profile) => _buildProfileDetails(theme, profile),
        keymap: (keymap) => _buildKeymapDetails(theme, keymap),
      );

  Widget _buildDeviceDetails(
    ThemeData theme,
    DeviceFlowNode device,
    FlowGraph graph,
  ) {
    final connections = graph.edges.where((e) => e.from == device.id).length;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _buildDetailRow(theme, 'Node ID', device.id),
        const SizedBox(height: 12),
        _buildDetailRow(
          theme,
          'Device ID',
          '${device.deviceId.vendorId}:${device.deviceId.productId}',
        ),
        const SizedBox(height: 12),
        _buildDetailRow(
          theme,
          'Serial Number',
          device.deviceId.serial ?? 'N/A',
        ),
        if (device.label != null) ...[
          const SizedBox(height: 12),
          _buildDetailRow(theme, 'Label', device.label!),
        ],
        const SizedBox(height: 12),
        _buildDetailRow(
          theme,
          'Status',
          device.isConnected ? 'Connected' : 'Disconnected',
        ),
        const SizedBox(height: 12),
        _buildDetailRow(theme, 'Active Slots', '$connections'),
      ],
    );
  }

  Widget _buildProfileDetails(
    ThemeData theme,
    HardwareProfileFlowNode profile,
  ) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      _buildDetailRow(theme, 'Profile ID', profile.id),
      if (profile.label != null) ...[
        const SizedBox(height: 12),
        _buildDetailRow(theme, 'Label', profile.label!),
      ],
    ],
  );

  Widget _buildKeymapDetails(ThemeData theme, KeymapFlowNode keymap) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      _buildDetailRow(theme, 'Keymap ID', keymap.id),
      if (keymap.label != null) ...[
        const SizedBox(height: 12),
        _buildDetailRow(theme, 'Label', keymap.label!),
      ],
      if (keymap.layers != null) ...[
        const SizedBox(height: 12),
        _buildDetailRow(theme, 'Layers', '${keymap.layers}'),
      ],
    ],
  );

  Widget _buildDetailRow(ThemeData theme, String label, String value) => Row(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      SizedBox(
        width: 140,
        child: Text(
          label,
          style: theme.textTheme.bodyMedium?.copyWith(
            fontWeight: FontWeight.w600,
            color: theme.colorScheme.onSurfaceVariant,
          ),
        ),
      ),
      Expanded(child: Text(value, style: theme.textTheme.bodyMedium)),
    ],
  );

  Widget _buildFooter(BuildContext context) => Padding(
    padding: const EdgeInsets.all(16),
    child: Row(
      mainAxisAlignment: MainAxisAlignment.end,
      children: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Close'),
        ),
      ],
    ),
  );

  Widget _buildError(BuildContext context, ThemeData theme, Object error) =>
      Padding(
        padding: const EdgeInsets.all(48),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.error_outline, size: 64, color: theme.colorScheme.error),
            const SizedBox(height: 16),
            Text(
              'Failed to load node details',
              style: theme.textTheme.titleLarge,
            ),
            const SizedBox(height: 8),
            Text(
              error.toString(),
              style: theme.textTheme.bodyMedium,
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: 24),
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('Close'),
            ),
          ],
        ),
      );

  IconData _getNodeIcon(FlowNode node) => node.map(
    device: (_) => Icons.keyboard_outlined,
    hardwareProfile: (_) => Icons.widgets_outlined,
    keymap: (_) => Icons.map_outlined,
  );

  String _getNodeTitle(FlowNode node) => node.map(
    device: (d) => d.label ?? 'Device Node',
    hardwareProfile: (p) => p.label ?? 'Hardware Profile',
    keymap: (k) => k.label ?? 'Keymap',
  );
}
