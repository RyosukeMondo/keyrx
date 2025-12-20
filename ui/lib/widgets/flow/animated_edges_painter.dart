import 'package:core_logic/models/flow/edge_style.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';

import '../../animations/edge_animations.dart';
import '../../utils/core_ui_extensions.dart';
import 'edges_painter.dart';

/// Extended painter with animations for edges
class AnimatedEdgesPainter extends CustomPainter {
  AnimatedEdgesPainter({
    required this.edges,
    required this.style,
    required this.hoverPosition,
    required this.animationStates,
    required this.pulseEnabled,
    this.previousColors = const {},
  }) : _pathCache = {};

  final List<PaintableEdge> edges;
  final EdgeStyle style;
  final Offset? hoverPosition;

  /// Animation states for each edge (keyed by edge identifier)
  final Map<String, EdgeAnimationState> animationStates;

  /// Whether pulse animation is enabled for active slots
  final bool pulseEnabled;

  /// Previous colors for smooth transitions (keyed by edge identifier)
  final Map<String, Color> previousColors;

  /// Path cache to avoid recomputation
  final Map<EdgePathCacheKey, Path> _pathCache;

  @override
  void paint(Canvas canvas, Size size) {
    final badgeTextStyle = TextStyle(
      color: style.palette.priorityTextColor,
      fontSize: 11,
      fontWeight: FontWeight.w700,
    );

    for (var i = 0; i < edges.length; i++) {
      final edge = edges[i];
      final edgeId = _edgeId(edge);
      final animState = animationStates[edgeId] ?? EdgeAnimationState();

      // Skip if fully transparent (faded out)
      if (animState.opacity < 0.01) continue;

      final isHovered = _isEdgeHovered(i);
      final baseStrokeWidth = isHovered
          ? style.hoverWidth
          : edge.active
          ? style.activeWidth
          : style.inactiveWidth;

      // Apply pulse scaling to active edges
      final pulseScale = edge.active && pulseEnabled
          ? EdgeAnimations.pulseScale(animState.pulsePhase)
          : 1.0;
      final strokeWidth = baseStrokeWidth * pulseScale;

      // Calculate color with transitions
      final baseColor = style.colorForStatus(edge.status, active: edge.active, hovered: isHovered);

      final color = _applyColorTransition(edgeId, baseColor, animState.colorTransition);

      // Apply anti-aliasing and opacity
      final paint = Paint()
        ..color = color.withValues(alpha: color.a * animState.opacity)
        ..strokeWidth = strokeWidth
        ..style = PaintingStyle.stroke
        ..strokeCap = StrokeCap.round
        ..isAntiAlias = true;

      // Draw glow effect for hovered or pulsing edges
      if (isHovered || (edge.active && pulseEnabled)) {
        final glowOpacity = isHovered ? 0.32 : EdgeAnimations.pulseGlow(animState.pulsePhase);
        final glowPaint = Paint()
          ..color = color.withValues(alpha: glowOpacity * animState.opacity)
          ..strokeWidth = strokeWidth + 3
          ..style = PaintingStyle.stroke
          ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 5)
          ..isAntiAlias = true;

        final path = _getCachedPath(edge);
        canvas.drawPath(path, glowPaint);
      }

      // Draw main edge
      final path = _getCachedPath(edge);
      canvas.drawPath(path, paint);

      // Draw priority badge with opacity
      _drawPriorityBadge(
        canvas,
        edge,
        i,
        color.withValues(alpha: animState.opacity),
        badgeTextStyle,
        animState.opacity,
      );
    }
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
      return edge.active ? path : _dashPath(path, style.dashLength, style.dashGap);
    });
  }

  Color _applyColorTransition(String edgeId, Color targetColor, double transitionProgress) {
    final prevColor = previousColors[edgeId];
    if (prevColor == null || transitionProgress >= 1.0) {
      return targetColor;
    }

    return EdgeAnimations.lerpColor(prevColor, targetColor, transitionProgress);
  }

  void _drawPriorityBadge(
    Canvas canvas,
    PaintableEdge edge,
    int edgeIndex,
    Color color,
    TextStyle textStyle,
    double opacity,
  ) {
    final label = edge.priority.toString();
    final badgePoint = _sampleEdge(edge, edgeIndex, 0.55, normalOffset: 16);

    final fillPaint = Paint()
      ..color = style.palette.priorityBackgroundColor.withValues(alpha: 0.9 * opacity)
      ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 1.5);

    canvas
      ..drawCircle(badgePoint, style.priorityBadgeRadius, fillPaint)
      ..drawCircle(
        badgePoint,
        style.priorityBadgeRadius,
        Paint()
          ..color = color
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1.2
          ..isAntiAlias = true,
      );

    final textPainter = TextPainter(
      text: TextSpan(
        text: label,
        style: textStyle.copyWith(color: textStyle.color?.withValues(alpha: opacity)),
      ),
      textAlign: TextAlign.center,
      textDirection: TextDirection.ltr,
    )..layout();

    textPainter.paint(canvas, badgePoint - Offset(textPainter.width / 2, textPainter.height / 2));
  }

  Offset _sampleEdge(PaintableEdge edge, int edgeIndex, double t, {double normalOffset = 0}) {
    final controls = _computeControls(edge.start, edge.end);
    final clampedT = t.clamp(0.0, 1.0);
    final point = _cubicPoint(edge.start, controls.$1, controls.$2, edge.end, clampedT);

    if (normalOffset == 0) return point;

    final tangent = _cubicTangent(edge.start, controls.$1, controls.$2, edge.end, clampedT);
    final normal = Offset(-tangent.dy, tangent.dx);
    final normalized = normal == Offset.zero ? Offset.zero : normal / normal.distance;
    return point + normalized * normalOffset;
  }

  bool _isEdgeHovered(int edgeIndex) {
    if (hoverPosition == null || edgeIndex >= edges.length) return false;
    final edge = edges[edgeIndex];
    return _distanceToEdge(edge) <= style.hoverHitRadius;
  }

  double _distanceToEdge(PaintableEdge edge) {
    if (hoverPosition == null) return double.infinity;

    const samples = 20;
    var minDistance = double.infinity;
    final controls = _computeControls(edge.start, edge.end);

    for (var i = 0; i <= samples; i++) {
      final t = i / samples;
      final pos = _cubicPoint(edge.start, controls.$1, controls.$2, edge.end, t);
      final distance = (pos - hoverPosition!).distance;
      if (distance < minDistance) {
        minDistance = distance;
      }
    }
    return minDistance;
  }

  String _edgeId(PaintableEdge edge) {
    // Create a unique ID for the edge based on its properties
    return '${edge.start.dx}_${edge.start.dy}_${edge.end.dx}_${edge.end.dy}_${edge.priority}';
  }

  @override
  bool shouldRepaint(covariant AnimatedEdgesPainter oldDelegate) {
    // Always repaint if animation states changed
    if (!_mapEquals(animationStates, oldDelegate.animationStates)) {
      return true;
    }

    if (!_mapEquals(previousColors, oldDelegate.previousColors)) {
      return true;
    }

    if (pulseEnabled != oldDelegate.pulseEnabled) {
      return true;
    }

    // Check edge/style changes
    final sameEdges = listEquals(edges, oldDelegate.edges);
    return !sameEdges || hoverPosition != oldDelegate.hoverPosition || style != oldDelegate.style;
  }

  bool _mapEquals<K, V>(Map<K, V> a, Map<K, V> b) {
    if (a.length != b.length) return false;
    for (final key in a.keys) {
      if (a[key] != b[key]) return false;
    }
    return true;
  }
}

// Helper functions (duplicated from EdgesPainter to avoid private access)
Path _buildPath(Offset start, Offset end, Offset control1, Offset control2) => Path()
  ..moveTo(start.dx, start.dy)
  ..cubicTo(control1.dx, control1.dy, control2.dx, control2.dy, end.dx, end.dy);

Path _dashPath(Path source, double dashLength, double gap) {
  final dashed = Path();
  for (final metric in source.computeMetrics()) {
    var distance = 0.0;
    while (distance < metric.length) {
      dashed.addPath(metric.extractPath(distance, distance + dashLength), Offset.zero);
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
  return (p1 - p0) * (3 * mt * mt) + (p2 - p1) * (6 * mt * t) + (p3 - p2) * (3 * t * t);
}
