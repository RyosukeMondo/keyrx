/// Accessible flow canvas wrapper with screen reader support.
library;

import 'package:core_logic/models/flow/flow_graph.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../state/keyflow_providers.dart';
import '../../state/selection_state.dart';

/// Wrapper that adds semantic labels and announcements to the flow canvas.
///
/// This widget ensures that all interactive elements have proper semantic
/// labels for screen readers, and announces state changes appropriately.
class AccessibleFlowCanvas extends ConsumerWidget {
  const AccessibleFlowCanvas({required this.child, super.key});

  final Widget child;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final graphAsync = ref.watch(flowGraphProvider);
    final selection = ref.watch(selectionNotifierProvider);

    // Build semantic label for the canvas
    final semanticLabel = _buildCanvasSemanticLabel(graphAsync, selection);

    return Semantics(
      label: semanticLabel,
      container: true,
      explicitChildNodes: true,
      child: child,
    );
  }

  String _buildCanvasSemanticLabel(
    AsyncValue<FlowGraph> graphAsync,
    Selection selection,
  ) {
    if (graphAsync.isLoading) {
      return 'KeyFlow editor loading';
    }

    if (graphAsync.hasError) {
      return 'KeyFlow editor error: ${graphAsync.error}';
    }

    final graph = graphAsync.value;
    if (graph == null || graph.nodes.isEmpty) {
      return 'KeyFlow editor, no devices connected';
    }

    final deviceCount = graph.nodes.whereType<DeviceFlowNode>().length;
    final profileCount = graph.nodes
        .whereType<HardwareProfileFlowNode>()
        .length;
    final keymapCount = graph.nodes.whereType<KeymapFlowNode>().length;
    final edgeCount = graph.edges.length;

    final buffer = StringBuffer('KeyFlow editor with ')
      ..write('$deviceCount device${deviceCount != 1 ? 's' : ''}, ')
      ..write('$profileCount profile${profileCount != 1 ? 's' : ''}, ')
      ..write('$keymapCount keymap${keymapCount != 1 ? 's' : ''}, ')
      ..write('and $edgeCount connection${edgeCount != 1 ? 's' : ''}');

    if (selection is NodeSelection) {
      buffer.write('. Node ${selection.nodeId} selected');
    } else if (selection is EdgeSelection) {
      buffer.write(
        '. Connection from ${selection.from} to ${selection.to} selected',
      );
    }

    return buffer.toString();
  }
}

/// Extension to add semantic labels to flow nodes.
extension FlowNodeSemantics on FlowNode {
  /// Generate semantic label for this node.
  String toSemanticLabel({
    required bool isConnected,
    int activeSlotCount = 0,
    int totalSlotCount = 0,
    bool hasError = false,
  }) => map(
    device: (node) {
      final buffer = StringBuffer()
        ..write(node.label ?? 'Unknown device')
        ..write(' keyboard');

      if (isConnected) {
        buffer.write(', connected');
        if (totalSlotCount > 0) {
          buffer.write(
            ', $totalSlotCount slot${totalSlotCount != 1 ? 's' : ''}',
          );
          if (activeSlotCount > 0) {
            buffer.write(', $activeSlotCount active');
          }
        }
      } else {
        buffer.write(', disconnected');
      }

      if (hasError) {
        buffer.write(', has errors');
      }

      return buffer.toString();
    },
    hardwareProfile: (node) {
      final buffer = StringBuffer()
        ..write(node.label ?? 'Unknown profile')
        ..write(' hardware profile');

      if (hasError) {
        buffer.write(', invalid');
      } else {
        buffer.write(', valid');
      }

      return buffer.toString();
    },
    keymap: (node) {
      final buffer = StringBuffer()
        ..write(node.label ?? 'Unknown keymap')
        ..write(' keymap');

      final layers = node.layers;
      if (layers != null && layers > 0) {
        buffer.write(', $layers layer${layers != 1 ? 's' : ''}');
      }

      if (hasError) {
        buffer.write(', compilation failed');
      } else {
        buffer.write(', valid');
      }

      return buffer.toString();
    },
  );
}

/// Extension to add semantic labels to flow edges.
extension FlowEdgeSemantics on FlowEdge {
  /// Generate semantic label for this edge.
  String toSemanticLabel(FlowGraph graph) {
    final fromNode = graph.nodes.firstWhere(
      (n) => n.nodeId == from,
      orElse: () => throw StateError('From node not found'),
    );
    final toNode = graph.nodes.firstWhere(
      (n) => n.nodeId == to,
      orElse: () => throw StateError('To node not found'),
    );

    final buffer = StringBuffer();

    if (active) {
      buffer.write('Active slot');
    } else {
      buffer.write('Inactive slot');
    }

    buffer
      ..write(' connecting ')
      ..write(fromNode.label ?? 'unknown device')
      ..write(' to ')
      ..write(toNode.label ?? 'unknown keymap')
      ..write(', priority ${priority + 1}');

    switch (status) {
      case EdgeStatus.valid:
        buffer.write(', valid');
        break;
      case EdgeStatus.invalid:
        buffer.write(', invalid');
        break;
      case EdgeStatus.warning:
        buffer.write(', warning');
        break;
    }

    return buffer.toString();
  }
}
