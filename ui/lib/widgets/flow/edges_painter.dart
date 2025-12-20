import 'package:core_logic/models/flow/edge_style.dart';
import 'package:core_logic/models/flow/flow_graph.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';

import '../../utils/core_ui_extensions.dart';

class PaintableEdge {
  const PaintableEdge({
    required this.start,
    required this.end,
    required this.status,
    required this.active,
    required this.priority,
  });

  final Offset start;
  final Offset end;
  final EdgeStatus status;
  final bool active;
  final int priority;

  @override
  bool operator ==(Object other) =>
      other is PaintableEdge &&
      other.start == start &&
      other.end == end &&
      other.status == status &&
      other.active == active &&
      other.priority == priority;

  @override
  int get hashCode => Object.hash(start, end, status, active, priority);
}

class EdgesPainter extends CustomPainter {
  EdgesPainter({required this.edges, required this.style, required Offset? hoverPosition})
    : _hoverPosition = hoverPosition,
      _computedEdges = edges.map((edge) => _ComputedEdge.from(edge, style)).toList();

  final List<PaintableEdge> edges;
  final EdgeStyle style;
  final Offset? _hoverPosition;
  final List<_ComputedEdge> _computedEdges;

  @override
  void paint(Canvas canvas, Size size) {
    final badgeTextStyle = TextStyle(
      color: style.palette.priorityTextColor,
      fontSize: 11,
      fontWeight: FontWeight.w700,
    );

    for (final edge in _computedEdges) {
      final isHovered = _hoverPosition != null && edge.isNear(_hoverPosition, style.hoverHitRadius);
      final strokeWidth = isHovered
          ? style.hoverWidth
          : edge.edge.active
          ? style.activeWidth
          : style.inactiveWidth;
      final color = style.colorForStatus(
        edge.edge.status,
        active: edge.edge.active,
        hovered: isHovered,
      );

      final paint = Paint()
        ..color = color
        ..strokeWidth = strokeWidth
        ..style = PaintingStyle.stroke
        ..strokeCap = StrokeCap.round;

      if (isHovered) {
        final glowPaint = Paint()
          ..color = color.withValues(alpha: 0.32)
          ..strokeWidth = strokeWidth + 2
          ..style = PaintingStyle.stroke
          ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 4);
        canvas.drawPath(edge.path, glowPaint);
      }

      final path = edge.edge.active ? edge.path : edge.dashedPath;
      canvas.drawPath(path, paint);

      _drawPriorityBadge(canvas, edge, color, badgeTextStyle);
    }
  }

  void _drawPriorityBadge(Canvas canvas, _ComputedEdge edge, Color color, TextStyle textStyle) {
    final label = edge.edge.priority.toString();
    final badgePoint = edge.sample(0.55, normalOffset: 16);

    final fillPaint = Paint()
      ..color = style.palette.priorityBackgroundColor.withValues(alpha: 0.9)
      ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 1.5);

    canvas
      ..drawCircle(badgePoint, style.priorityBadgeRadius, fillPaint)
      ..drawCircle(
        badgePoint,
        style.priorityBadgeRadius,
        Paint()
          ..color = color
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1.2,
      );

    final textPainter = TextPainter(
      text: TextSpan(text: label, style: textStyle),
      textAlign: TextAlign.center,
      textDirection: TextDirection.ltr,
    )..layout();

    textPainter.paint(canvas, badgePoint - Offset(textPainter.width / 2, textPainter.height / 2));
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) {
    if (oldDelegate is! EdgesPainter) return true;
    final sameEdges = listEquals(edges, oldDelegate.edges);
    return !sameEdges || _hoverPosition != oldDelegate._hoverPosition || style != oldDelegate.style;
  }
}

class _ComputedEdge {
  _ComputedEdge({
    required this.edge,
    required this.path,
    required this.dashedPath,
    required this.control1,
    required this.control2,
  });

  factory _ComputedEdge.from(PaintableEdge edge, EdgeStyle style) {
    final controls = _computeControls(edge.start, edge.end);
    final path = _buildPath(edge.start, edge.end, controls.$1, controls.$2);
    final dashedPath = edge.active ? path : _dashPath(path, style.dashLength, style.dashGap);
    return _ComputedEdge(
      edge: edge,
      path: path,
      dashedPath: dashedPath,
      control1: controls.$1,
      control2: controls.$2,
    );
  }

  final PaintableEdge edge;
  final Path path;
  final Path dashedPath;
  final Offset control1;
  final Offset control2;

  bool isNear(Offset point, double radius) => _distanceTo(point) <= radius;

  Offset sample(double t, {double normalOffset = 0}) {
    final clampedT = t.clamp(0.0, 1.0);
    final point = _cubicPoint(edge.start, control1, control2, edge.end, clampedT);
    if (normalOffset == 0) return point;

    final tangent = _cubicTangent(edge.start, control1, control2, edge.end, clampedT);
    final normal = Offset(-tangent.dy, tangent.dx);
    final normalized = normal == Offset.zero ? Offset.zero : normal / normal.distance;
    return point + normalized * normalOffset;
  }

  double _distanceTo(Offset point) {
    // Sample the cubic at discrete intervals to approximate distance.
    const samples = 20;
    var minDistance = double.infinity;
    for (var i = 0; i <= samples; i++) {
      final t = i / samples;
      final pos = _cubicPoint(edge.start, control1, control2, edge.end, t);
      final distance = (pos - point).distance;
      if (distance < minDistance) {
        minDistance = distance;
      }
    }
    return minDistance;
  }
}

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
