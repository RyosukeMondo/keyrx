/// Selection state management for flow graph edges and nodes.
library;

import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'selection_state.g.dart';

/// Represents the currently selected element in the flow graph.
sealed class Selection {
  const Selection();
}

/// No element is selected.
class NoSelection extends Selection {
  const NoSelection();

  @override
  bool operator ==(Object other) => other is NoSelection;

  @override
  int get hashCode => 0;
}

/// An edge is selected (identified by slotId and from/to node IDs).
class EdgeSelection extends Selection {
  const EdgeSelection({
    required this.slotId,
    required this.from,
    required this.to,
  });

  final String slotId;
  final String from;
  final String to;

  @override
  bool operator ==(Object other) =>
      other is EdgeSelection &&
      other.slotId == slotId &&
      other.from == from &&
      other.to == to;

  @override
  int get hashCode => Object.hash(slotId, from, to);
}

/// A node is selected.
class NodeSelection extends Selection {
  const NodeSelection(this.nodeId);

  final String nodeId;

  @override
  bool operator ==(Object other) =>
      other is NodeSelection && other.nodeId == nodeId;

  @override
  int get hashCode => nodeId.hashCode;
}

/// Provider managing the current selection state.
@riverpod
class SelectionNotifier extends _$SelectionNotifier {
  @override
  Selection build() => const NoSelection();

  /// Select an edge.
  void selectEdge({
    required String slotId,
    required String from,
    required String to,
  }) {
    state = EdgeSelection(slotId: slotId, from: from, to: to);
  }

  /// Select a node.
  void selectNode(String nodeId) {
    state = NodeSelection(nodeId);
  }

  /// Clear selection.
  void clearSelection() {
    state = const NoSelection();
  }

  /// Check if a specific edge is selected.
  bool isEdgeSelected({
    required String slotId,
    required String from,
    required String to,
  }) {
    final current = state;
    return current is EdgeSelection &&
        current.slotId == slotId &&
        current.from == from &&
        current.to == to;
  }

  /// Check if a specific node is selected.
  bool isNodeSelected(String nodeId) {
    final current = state;
    return current is NodeSelection && current.nodeId == nodeId;
  }
}
