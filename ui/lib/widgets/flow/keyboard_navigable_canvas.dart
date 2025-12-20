/// Keyboard navigation wrapper for flow canvas.
library;

import 'package:core_logic/models/flow/flow_graph.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../services/keyboard_shortcuts.dart';
import '../../state/keyflow_providers.dart';
import '../../state/node_positions_provider.dart';
import '../../state/selection_state.dart';

/// Controller for managing keyboard-based zoom operations.
class ZoomController {
  ZoomController({required this.transformationController});

  final TransformationController transformationController;

  /// Reset zoom to 100% (scale 1.0).
  void resetZoom() {
    transformationController.value = Matrix4.identity();
  }

  /// Zoom in by 10%.
  void zoomIn() {
    _adjustZoom(1.1);
  }

  /// Zoom out by 10%.
  void zoomOut() {
    _adjustZoom(0.9);
  }

  void _adjustZoom(double factor) {
    final current = transformationController.value;
    final currentScale = current.getMaxScaleOnAxis();
    final newScale = (currentScale * factor).clamp(0.5, 2.5);
    final scaleFactor = newScale / currentScale;

    transformationController.value = current.clone()
      ..scaleByDouble(scaleFactor, scaleFactor, 1, 1);
  }
}

/// Focus node tracker for keyboard navigation.
/// Focus node tracker for keyboard navigation.
@immutable
class FocusTracker {
  const FocusTracker({required this.nodes, this.focusedIndex = 0});

  final List<FlowNode> nodes;
  final int focusedIndex;

  FocusTracker copyWith({List<FlowNode>? nodes, int? focusedIndex}) =>
      FocusTracker(nodes: nodes ?? this.nodes, focusedIndex: focusedIndex ?? this.focusedIndex);

  String? get focusedNodeId =>
      nodes.isEmpty ? null : nodes[focusedIndex.clamp(0, nodes.length - 1)].nodeId;

  /// Move focus to the next node.
  FocusTracker focusNext() {
    if (nodes.isEmpty) return this;
    final nextIndex = (focusedIndex + 1) % nodes.length;
    return copyWith(focusedIndex: nextIndex);
  }

  /// Move focus to the previous node.
  FocusTracker focusPrevious() {
    if (nodes.isEmpty) return this;
    final prevIndex = (focusedIndex - 1 + nodes.length) % nodes.length;
    return copyWith(focusedIndex: prevIndex);
  }

  /// Check if a node is focused.
  bool isNodeFocused(String nodeId) => focusedNodeId == nodeId;
}

/// Provider for focus tracking state.
final focusTrackerProvider = StateProvider<FocusTracker>((ref) => const FocusTracker(nodes: []));

/// Keyboard-navigable canvas wrapper.
///
/// Wraps the flow canvas with keyboard navigation capabilities:
/// - Tab/Shift+Tab to cycle through nodes
/// - Arrow keys to move focused node
/// - Enter to open node details
/// - Delete to remove selected edge
/// - Escape to clear selection
/// - Ctrl+0/+/- for zoom control
/// - Ctrl+? for help
class KeyboardNavigableCanvas extends ConsumerStatefulWidget {
  const KeyboardNavigableCanvas({
    required this.child,
    required this.transformationController,
    required this.canvasBounds,
    super.key,
    this.onNodeDetailsRequested,
    this.onHelpRequested,
  });

  final Widget child;
  final TransformationController transformationController;
  final Size canvasBounds;
  final ValueChanged<String>? onNodeDetailsRequested;
  final VoidCallback? onHelpRequested;

  @override
  ConsumerState<KeyboardNavigableCanvas> createState() => _KeyboardNavigableCanvasState();
}

class _KeyboardNavigableCanvasState extends ConsumerState<KeyboardNavigableCanvas> {
  final _focusNode = FocusNode();
  late final ZoomController _zoomController;

  static const _moveStep = 10.0;

  @override
  void initState() {
    super.initState();
    _zoomController = ZoomController(transformationController: widget.transformationController);

    // Request focus on mount for immediate keyboard control
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _focusNode.requestFocus();
    });
  }

  @override
  void dispose() {
    _focusNode.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    // Update focus tracker when graph changes
    final graphAsync = ref.watch(flowGraphProvider);
    if (graphAsync.hasValue) {
      final graph = graphAsync.value!;
      final currentTracker = ref.read(focusTrackerProvider);

      // Update tracker if nodes have changed
      if (currentTracker.nodes != graph.nodes) {
        WidgetsBinding.instance.addPostFrameCallback((_) {
          ref.read(focusTrackerProvider.notifier).state = currentTracker.copyWith(
            nodes: graph.nodes,
          );
        });
      }
    }

    return FocusableActionDetector(
      focusNode: _focusNode,
      autofocus: true,
      shortcuts: _shortcuts,
      actions: _actions,
      child: widget.child,
    );
  }

  Map<ShortcutActivator, Intent> get _shortcuts => {
    KeyboardShortcuts.tabToNextNode: const _FocusNextNodeIntent(),
    KeyboardShortcuts.tabToPreviousNode: const _FocusPreviousNodeIntent(),
    KeyboardShortcuts.moveNodeUp: const _MoveNodeIntent(Offset(0, -_moveStep)),
    KeyboardShortcuts.moveNodeDown: const _MoveNodeIntent(Offset(0, _moveStep)),
    KeyboardShortcuts.moveNodeLeft: const _MoveNodeIntent(Offset(-_moveStep, 0)),
    KeyboardShortcuts.moveNodeRight: const _MoveNodeIntent(Offset(_moveStep, 0)),
    KeyboardShortcuts.openNodeDetails: const _OpenNodeDetailsIntent(),
    KeyboardShortcuts.deleteSelectedEdge: const _DeleteSelectedEdgeIntent(),
    KeyboardShortcuts.clearSelection: const _ClearSelectionIntent(),
    KeyboardShortcuts.resetZoom: const _ResetZoomIntent(),
    KeyboardShortcuts.zoomIn: const _ZoomInIntent(),
    KeyboardShortcuts.zoomOut: const _ZoomOutIntent(),
    KeyboardShortcuts.showHelp: const _ShowHelpIntent(),
  };

  Map<Type, Action<Intent>> get _actions => {
    _FocusNextNodeIntent: CallbackAction<_FocusNextNodeIntent>(onInvoke: (_) => _handleFocusNext()),
    _FocusPreviousNodeIntent: CallbackAction<_FocusPreviousNodeIntent>(
      onInvoke: (_) => _handleFocusPrevious(),
    ),
    _MoveNodeIntent: CallbackAction<_MoveNodeIntent>(
      onInvoke: (intent) => _handleMoveNode(intent.delta),
    ),
    _OpenNodeDetailsIntent: CallbackAction<_OpenNodeDetailsIntent>(
      onInvoke: (_) => _handleOpenNodeDetails(),
    ),
    _DeleteSelectedEdgeIntent: CallbackAction<_DeleteSelectedEdgeIntent>(
      onInvoke: (_) => _handleDeleteSelectedEdge(),
    ),
    _ClearSelectionIntent: CallbackAction<_ClearSelectionIntent>(
      onInvoke: (_) => _handleClearSelection(),
    ),
    _ResetZoomIntent: CallbackAction<_ResetZoomIntent>(
      onInvoke: (_) => _zoomController.resetZoom(),
    ),
    _ZoomInIntent: CallbackAction<_ZoomInIntent>(onInvoke: (_) => _zoomController.zoomIn()),
    _ZoomOutIntent: CallbackAction<_ZoomOutIntent>(onInvoke: (_) => _zoomController.zoomOut()),
    _ShowHelpIntent: CallbackAction<_ShowHelpIntent>(
      onInvoke: (_) => widget.onHelpRequested?.call(),
    ),
  };

  void _handleFocusNext() {
    final tracker = ref.read(focusTrackerProvider.notifier);
    tracker.update((state) => state.focusNext());
    _selectFocusedNode();
  }

  void _handleFocusPrevious() {
    final tracker = ref.read(focusTrackerProvider.notifier);
    tracker.update((state) => state.focusPrevious());
    _selectFocusedNode();
  }

  void _selectFocusedNode() {
    final tracker = ref.read(focusTrackerProvider);
    final nodeId = tracker.focusedNodeId;
    if (nodeId != null) {
      ref.read(selectionNotifierProvider.notifier).selectNode(nodeId);
    }
  }

  void _handleMoveNode(Offset delta) {
    final tracker = ref.read(focusTrackerProvider);
    final nodeId = tracker.focusedNodeId;
    if (nodeId == null) return;

    final positionsNotifier = ref.read(nodePositionsProvider.notifier);
    positionsNotifier.moveNode(nodeId, delta, widget.canvasBounds);
  }

  void _handleOpenNodeDetails() {
    final tracker = ref.read(focusTrackerProvider);
    final nodeId = tracker.focusedNodeId;
    if (nodeId != null) {
      widget.onNodeDetailsRequested?.call(nodeId);
    }
  }

  void _handleDeleteSelectedEdge() {
    final selection = ref.read(selectionNotifierProvider);
    if (selection is! EdgeSelection) return;

    // Find the device node to get the actual DeviceInstanceId
    final graphAsync = ref.read(flowGraphProvider);
    if (!graphAsync.hasValue) return;

    final graph = graphAsync.value!;
    final deviceNode = graph.nodes.firstWhere(
      (node) => node.nodeId == selection.from,
      orElse: () => throw StateError('Device node not found'),
    );

    if (deviceNode is! DeviceFlowNode) return;

    final actions = ref.read(keyFlowActionsProvider);
    actions.removeSlot(deviceNode.deviceId, selection.slotId);
    ref.read(selectionNotifierProvider.notifier).clearSelection();
  }

  void _handleClearSelection() {
    ref.read(selectionNotifierProvider.notifier).clearSelection();
  }
}

// Intent classes
class _FocusNextNodeIntent extends Intent {
  const _FocusNextNodeIntent();
}

class _FocusPreviousNodeIntent extends Intent {
  const _FocusPreviousNodeIntent();
}

class _MoveNodeIntent extends Intent {
  const _MoveNodeIntent(this.delta);
  final Offset delta;
}

class _OpenNodeDetailsIntent extends Intent {
  const _OpenNodeDetailsIntent();
}

class _DeleteSelectedEdgeIntent extends Intent {
  const _DeleteSelectedEdgeIntent();
}

class _ClearSelectionIntent extends Intent {
  const _ClearSelectionIntent();
}

class _ResetZoomIntent extends Intent {
  const _ResetZoomIntent();
}

class _ZoomInIntent extends Intent {
  const _ZoomInIntent();
}

class _ZoomOutIntent extends Intent {
  const _ZoomOutIntent();
}

class _ShowHelpIntent extends Intent {
  const _ShowHelpIntent();
}

/// Extension to add moveNode method to NodePositions.
