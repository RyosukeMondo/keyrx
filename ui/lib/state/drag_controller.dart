import 'dart:ui';

import 'package:riverpod_annotation/riverpod_annotation.dart';

part 'drag_controller.g.dart';

/// Represents the current drag state for a node.
class DragState {
  const DragState({
    required this.nodeId,
    required this.initialPosition,
    required this.currentOffset,
    required this.isDragging,
  });

  final String nodeId;
  final Offset initialPosition;
  final Offset currentOffset;
  final bool isDragging;

  Offset get currentPosition => initialPosition + currentOffset;

  DragState copyWith({
    String? nodeId,
    Offset? initialPosition,
    Offset? currentOffset,
    bool? isDragging,
  }) {
    return DragState(
      nodeId: nodeId ?? this.nodeId,
      initialPosition: initialPosition ?? this.initialPosition,
      currentOffset: currentOffset ?? this.currentOffset,
      isDragging: isDragging ?? this.isDragging,
    );
  }
}

/// Controller for managing drag state across the flow canvas.
/// Tracks which node is being dragged and its current position.
@riverpod
class DragController extends _$DragController {
  @override
  DragState? build() => null;

  /// Start dragging a node from the given initial position.
  void startDrag(String nodeId, Offset initialPosition) {
    state = DragState(
      nodeId: nodeId,
      initialPosition: initialPosition,
      currentOffset: Offset.zero,
      isDragging: true,
    );
  }

  /// Update the current drag offset.
  void updateDrag(Offset delta) {
    final current = state;
    if (current == null || !current.isDragging) return;

    state = current.copyWith(currentOffset: current.currentOffset + delta);
  }

  /// End the drag operation and return the final position.
  Offset? endDrag() {
    final current = state;
    if (current == null || !current.isDragging) return null;

    final finalPosition = current.currentPosition;
    state = null;
    return finalPosition;
  }

  /// Cancel the drag operation without committing changes.
  void cancelDrag() {
    state = null;
  }

  /// Check if a specific node is currently being dragged.
  bool isDragging(String nodeId) {
    return state?.nodeId == nodeId && state?.isDragging == true;
  }

  /// Get the current position for a specific node if it's being dragged.
  Offset? getDragPosition(String nodeId) {
    if (!isDragging(nodeId)) return null;
    return state?.currentPosition;
  }
}
