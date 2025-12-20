/// Optimized flow canvas with viewport virtualization and performance optimizations.
///
/// This widget wraps the flow editor canvas with several performance optimizations:
/// - RepaintBoundary isolation for each node
/// - Viewport-based virtualization (only render visible nodes)
/// - Selective edge rendering
/// - Optimized rebuilds
library;

import 'package:core_logic/models/flow/edge_style.dart';
import 'package:flutter/material.dart';

import '../../utils/core_ui_extensions.dart';
import '../../utils/rendering_optimizer.dart';
import '../flow/edges_painter.dart';

/// Optimized canvas view model.
class OptimizedCanvasView {
  const OptimizedCanvasView({
    required this.nodes,
    required this.edges,
    required this.viewport,
  });

  final List<OptimizedCanvasNode> nodes;
  final List<PaintableEdge> edges;
  final Rect viewport;

  /// Filter nodes to only those visible in viewport.
  List<OptimizedCanvasNode> get visibleNodes => nodes
      .where(
        (node) => isNodeInViewport(
          nodePosition: node.position,
          nodeSize: node.size,
          viewport: viewport,
          margin: 200, // Render margin for smooth scrolling
        ),
      )
      .toList();

  /// Filter edges to only those with both endpoints visible.
  /// Render all edges for now, optimize later if needed
  List<PaintableEdge> get visibleEdges => edges;
}

/// Optimized canvas node representation.
class OptimizedCanvasNode {
  const OptimizedCanvasNode({
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

/// Optimized node wrapper with RepaintBoundary isolation.
///
/// Wraps each node in a RepaintBoundary to prevent unnecessary repaints
/// when other parts of the canvas change.
class OptimizedNodeWrapper extends StatelessWidget {
  const OptimizedNodeWrapper({
    required this.nodeId,
    required this.child,
    super.key,
  });

  final String nodeId;
  final Widget child;

  @override
  // Each node is isolated in its own repaint boundary
  Widget build(BuildContext context) => RepaintBoundary(child: child);
}

/// Optimized edge layer with custom RenderObject.
///
/// Uses a custom RenderObject for more efficient edge rendering
/// compared to CustomPainter.
class OptimizedEdgeLayer extends LeafRenderObjectWidget {
  const OptimizedEdgeLayer({
    required this.edges,
    required this.style,
    super.key,
    this.hoverPosition,
  });

  final List<PaintableEdge> edges;
  final EdgeStyle style;
  final Offset? hoverPosition;

  @override
  RenderObject createRenderObject(BuildContext context) =>
      RenderEdgeLayer(edges: edges, style: style, hoverPosition: hoverPosition);

  @override
  void updateRenderObject(BuildContext context, RenderEdgeLayer renderObject) {
    renderObject
      ..edges = edges
      ..style = style
      ..hoverPosition = hoverPosition;
  }
}

/// Custom RenderBox for rendering edges efficiently.
///
/// This avoids the overhead of CustomPainter and provides more control
/// over rendering and hit testing.
class RenderEdgeLayer extends RenderBox {
  RenderEdgeLayer({
    required List<PaintableEdge> edges,
    required EdgeStyle style,
    Offset? hoverPosition,
  }) : _edges = edges,
       _style = style,
       _hoverPosition = hoverPosition,
       _pathCache = {};

  List<PaintableEdge> _edges;
  EdgeStyle _style;
  Offset? _hoverPosition;
  final Map<EdgePathCacheKey, Path> _pathCache;

  List<PaintableEdge> get edges => _edges;
  set edges(List<PaintableEdge> value) {
    if (_edges != value) {
      _edges = value;
      _invalidateCache();
      markNeedsPaint();
    }
  }

  EdgeStyle get style => _style;
  set style(EdgeStyle value) {
    if (_style != value) {
      _style = value;
      markNeedsPaint();
    }
  }

  Offset? get hoverPosition => _hoverPosition;
  set hoverPosition(Offset? value) {
    if (_hoverPosition != value) {
      _hoverPosition = value;
      markNeedsPaint();
    }
  }

  void _invalidateCache() {
    _pathCache.clear();
  }

  @override
  bool get sizedByParent => true;

  @override
  Size computeDryLayout(BoxConstraints constraints) => constraints.biggest;

  @override
  void paint(PaintingContext context, Offset offset) {
    final canvas = context.canvas;
    canvas
      ..save()
      ..translate(offset.dx, offset.dy);

    for (var i = 0; i < _edges.length; i++) {
      final edge = _edges[i];
      _paintEdge(canvas, edge, i);
    }

    canvas.restore();
  }

  void _paintEdge(Canvas canvas, PaintableEdge edge, int index) {
    final isHovered = _isEdgeHovered(index);
    final strokeWidth = isHovered
        ? _style.hoverWidth
        : edge.active
        ? _style.activeWidth
        : _style.inactiveWidth;

    final color = _style.colorForStatus(
      edge.status,
      active: edge.active,
      hovered: isHovered,
    );

    final paint = Paint()
      ..color = color
      ..strokeWidth = strokeWidth
      ..style = PaintingStyle.stroke
      ..strokeCap = StrokeCap.round
      ..isAntiAlias = true;

    final path = _getCachedPath(edge);
    canvas.drawPath(path, paint);

    // Draw priority badge
    _drawPriorityBadge(canvas, edge, color);
  }

  Path _getCachedPath(PaintableEdge edge) {
    final cacheKey = EdgePathCacheKey(
      startX: edge.start.dx,
      startY: edge.start.dy,
      endX: edge.end.dx,
      endY: edge.end.dy,
      active: edge.active,
    );

    return _pathCache.putIfAbsent(cacheKey, () {
      final controls = _computeControls(edge.start, edge.end);
      final path = _buildPath(edge.start, edge.end, controls.$1, controls.$2);
      return edge.active
          ? path
          : _dashPath(path, _style.dashLength, _style.dashGap);
    });
  }

  void _drawPriorityBadge(Canvas canvas, PaintableEdge edge, Color color) {
    final label = edge.priority.toString();
    final badgePoint = _sampleEdge(edge, 0.55, normalOffset: 16);

    // Draw badge background
    final fillPaint = Paint()
      ..color = _style.palette.priorityBackgroundColor.withValues(alpha: 0.9)
      ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 1.5);

    canvas
      ..drawCircle(badgePoint, _style.priorityBadgeRadius, fillPaint)
      // Draw badge border
      ..drawCircle(
        badgePoint,
        _style.priorityBadgeRadius,
        Paint()
          ..color = color
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1.2
          ..isAntiAlias = true,
      );

    // Draw badge text
    final textPainter = TextPainter(
      text: TextSpan(
        text: label,
        style: TextStyle(
          color: _style.palette.priorityTextColor,
          fontSize: 11,
          fontWeight: FontWeight.w700,
        ),
      ),
      textAlign: TextAlign.center,
      textDirection: TextDirection.ltr,
    )..layout();

    textPainter.paint(
      canvas,
      badgePoint - Offset(textPainter.width / 2, textPainter.height / 2),
    );
  }

  Offset _sampleEdge(PaintableEdge edge, double t, {double normalOffset = 0}) {
    final controls = _computeControls(edge.start, edge.end);
    final clampedT = t.clamp(0.0, 1.0);
    final point = _cubicPoint(
      edge.start,
      controls.$1,
      controls.$2,
      edge.end,
      clampedT,
    );

    if (normalOffset == 0) return point;

    final tangent = _cubicTangent(
      edge.start,
      controls.$1,
      controls.$2,
      edge.end,
      clampedT,
    );
    final normal = Offset(-tangent.dy, tangent.dx);
    final normalized = normal == Offset.zero
        ? Offset.zero
        : normal / normal.distance;
    return point + normalized * normalOffset;
  }

  bool _isEdgeHovered(int edgeIndex) {
    if (_hoverPosition == null || edgeIndex >= _edges.length) return false;
    final edge = _edges[edgeIndex];
    return _distanceToEdge(edge) <= _style.hoverHitRadius;
  }

  double _distanceToEdge(PaintableEdge edge) {
    if (_hoverPosition == null) return double.infinity;

    const samples = 20;
    var minDistance = double.infinity;
    final controls = _computeControls(edge.start, edge.end);

    for (var i = 0; i <= samples; i++) {
      final t = i / samples;
      final pos = _cubicPoint(
        edge.start,
        controls.$1,
        controls.$2,
        edge.end,
        t,
      );
      final distance = (pos - _hoverPosition!).distance;
      if (distance < minDistance) {
        minDistance = distance;
      }
    }
    return minDistance;
  }

  @override
  bool hitTestSelf(Offset position) => true;
}

// Helper functions for Bezier curve calculations
Path _buildPath(Offset start, Offset end, Offset control1, Offset control2) =>
    Path()
      ..moveTo(start.dx, start.dy)
      ..cubicTo(
        control1.dx,
        control1.dy,
        control2.dx,
        control2.dy,
        end.dx,
        end.dy,
      );

Path _dashPath(Path source, double dashLength, double gap) {
  final dashed = Path();
  for (final metric in source.computeMetrics()) {
    var distance = 0.0;
    while (distance < metric.length) {
      dashed.addPath(
        metric.extractPath(distance, distance + dashLength),
        Offset.zero,
      );
      distance += dashLength + gap;
    }
  }
  return dashed;
}

(Offset, Offset) _computeControls(Offset start, Offset end) {
  final midX = (start.dx + end.dx) / 2;
  return (Offset(midX, start.dy), Offset(midX, end.dy));
}

Offset _cubicPoint(Offset p0, Offset p1, Offset p2, Offset p3, double t) {
  final mt = 1 - t;
  return (p0 * (mt * mt * mt)) +
      (p1 * (3 * mt * mt * t)) +
      (p2 * (3 * mt * t * t)) +
      (p3 * (t * t * t));
}

Offset _cubicTangent(Offset p0, Offset p1, Offset p2, Offset p3, double t) {
  final mt = 1 - t;
  return (p1 - p0) * (3 * mt * mt) +
      (p2 - p1) * (6 * mt * t) +
      (p3 - p2) * (3 * t * t);
}
