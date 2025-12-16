/// Internal components for the flow canvas.
///
/// Contains helper classes and widgets used internally by FlowEditorPage:
/// - GridPainter for canvas grid background
/// - CanvasView and CanvasNode data structures
/// - AnimatedNodeWrapper for node animations
library;

import 'package:flutter/material.dart';
import 'edges_painter.dart';

/// Grid painter for flow canvas background.
class GridPainter extends CustomPainter {
  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = const Color(0xFF333333).withOpacity(0.4)
      ..strokeWidth = 1;

    const gridSize = 20.0;

    for (double x = 0; x < size.width; x += gridSize) {
      canvas.drawLine(Offset(x, 0), Offset(x, size.height), paint);
    }
    for (double y = 0; y < size.height; y += gridSize) {
      canvas.drawLine(Offset(0, y), Offset(size.width, y), paint);
    }
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => false;
}

/// View model for canvas rendering.
class CanvasView {
  const CanvasView({required this.nodes, required this.edges});

  final List<CanvasNode> nodes;
  final List<PaintableEdge> edges;
}

/// Canvas node data structure.
class CanvasNode {
  const CanvasNode({
    required this.id,
    required this.position,
    required this.size,
    required this.child,
  });

  final String id;
  final Offset position;
  final Size size;
  final Widget child;
}

/// Wrapper widget that animates node appearance/disappearance.
///
/// Provides fade-in animation for new nodes and smooth transitions
/// for node updates.
class AnimatedNodeWrapper extends StatefulWidget {
  const AnimatedNodeWrapper({
    required this.nodeId,
    required this.isNew,
    required this.child,
    super.key,
  });

  final String nodeId;
  final bool isNew;
  final Widget child;

  @override
  State<AnimatedNodeWrapper> createState() => _AnimatedNodeWrapperState();
}

class _AnimatedNodeWrapperState extends State<AnimatedNodeWrapper>
    with SingleTickerProviderStateMixin {
  late final AnimationController _controller;
  late final Animation<double> _fadeAnimation;
  late final Animation<double> _scaleAnimation;

  @override
  void initState() {
    super.initState();

    _controller = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 300),
    );

    _fadeAnimation = Tween<double>(
      begin: widget.isNew ? 0 : 1,
      end: 1,
    ).animate(CurvedAnimation(parent: _controller, curve: Curves.easeOut));

    _scaleAnimation = Tween<double>(
      begin: widget.isNew ? 0.9 : 1,
      end: 1,
    ).animate(CurvedAnimation(parent: _controller, curve: Curves.easeOutBack));

    if (widget.isNew) {
      _controller.forward();
    } else {
      _controller.value = 1;
    }
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => FadeTransition(
    opacity: _fadeAnimation,
    child: ScaleTransition(scale: _scaleAnimation, child: widget.child),
  );
}
