import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../state/drag_controller.dart';
import '../../state/node_positions_provider.dart';

/// A wrapper widget that makes a flow node draggable.
///
/// Features:
/// - Smooth dragging with visual feedback (elevation and shadow)
/// - Constrained dragging (keeps nodes within canvas bounds)
/// - Integrates with nodePositionsProvider to persist positions
/// - Works with InteractiveViewer transformations
class DraggableNode extends ConsumerStatefulWidget {
  const DraggableNode({
    required this.nodeId,
    required this.initialPosition,
    required this.size,
    required this.canvasBounds,
    required this.child,
    super.key,
  });

  final String nodeId;
  final Offset initialPosition;
  final Size size;
  final Size canvasBounds;
  final Widget child;

  @override
  ConsumerState<DraggableNode> createState() => _DraggableNodeState();
}

class _DraggableNodeState extends ConsumerState<DraggableNode> {
  bool _isHovering = false;

  @override
  Widget build(BuildContext context) {
    final dragState = ref.watch(dragControllerProvider);
    final isDragging =
        dragState?.nodeId == widget.nodeId && dragState?.isDragging == true;

    // Just return the draggable widget without Positioned
    // The parent (flow_editor_page) will wrap this in Positioned
    return Semantics(
      // Ensure minimum touch target size for accessibility
      explicitChildNodes: true,
      child: MouseRegion(
        onEnter: (_) => setState(() => _isHovering = true),
        onExit: (_) => setState(() => _isHovering = false),
        cursor: isDragging
            ? SystemMouseCursors.grabbing
            : (_isHovering
                  ? SystemMouseCursors.grab
                  : SystemMouseCursors.basic),
        child: GestureDetector(
          onPanStart: _onPanStart,
          onPanUpdate: _onPanUpdate,
          onPanEnd: _onPanEnd,
          // Ensure minimum touch target size (48x48)
          behavior: HitTestBehavior.opaque,
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 150),
            // Ensure minimum touch target constraints
            constraints: const BoxConstraints(minWidth: 48, minHeight: 48),
            decoration: BoxDecoration(
              borderRadius: BorderRadius.circular(8),
              boxShadow: isDragging
                  ? [
                      BoxShadow(
                        color: Colors.black.withValues(alpha: 0.5),
                        blurRadius: 16,
                        offset: const Offset(0, 8),
                      ),
                    ]
                  : _isHovering
                  ? [
                      BoxShadow(
                        color: Colors.black.withValues(alpha: 0.4),
                        blurRadius: 8,
                        offset: const Offset(0, 4),
                      ),
                    ]
                  : null,
            ),
            child: Transform.scale(
              scale: isDragging ? 1.02 : 1.0,
              child: widget.child,
            ),
          ),
        ),
      ),
    );
  }

  void _onPanStart(DragStartDetails details) {
    ref
        .read(dragControllerProvider.notifier)
        .startDrag(widget.nodeId, widget.initialPosition);
  }

  void _onPanUpdate(DragUpdateDetails details) {
    ref.read(dragControllerProvider.notifier).updateDrag(details.delta);
  }

  void _onPanEnd(DragEndDetails details) {
    final dragController = ref.read(dragControllerProvider.notifier);
    final finalPosition = dragController.endDrag();

    if (finalPosition == null) return;

    // Apply constraints to keep node within canvas bounds
    final constrainedPosition = _constrainPosition(finalPosition);

    // Persist the new position
    ref
        .read(nodePositionsProvider.notifier)
        .setPosition(widget.nodeId, constrainedPosition);
  }

  /// Constrains the position to keep the node within canvas bounds.
  Offset _constrainPosition(Offset position) {
    const minX = 0.0;
    const minY = 0.0;
    final maxX = widget.canvasBounds.width - widget.size.width;
    final maxY = widget.canvasBounds.height - widget.size.height;

    return Offset(position.dx.clamp(minX, maxX), position.dy.clamp(minY, maxY));
  }
}
