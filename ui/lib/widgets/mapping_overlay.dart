/// Mapping overlay widget for drawing arrows between mapped keys.
///
/// Uses CustomPainter to render visual connections between source
/// and target keys on the visual keyboard.
library;

import 'package:flutter/material.dart';

import '../models/keyboard_layout.dart';

/// Configuration for a single key mapping.
@immutable
class RemapConfig {
  const RemapConfig({
    required this.sourceKeyId,
    required this.targetKeyId,
    this.type = MappingType.simple,
  });

  /// Source key ID (from key).
  final String sourceKeyId;

  /// Target key ID (to key).
  final String targetKeyId;

  /// Type of mapping.
  final MappingType type;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is RemapConfig &&
          runtimeType == other.runtimeType &&
          sourceKeyId == other.sourceKeyId &&
          targetKeyId == other.targetKeyId &&
          type == other.type;

  @override
  int get hashCode => Object.hash(sourceKeyId, targetKeyId, type);
}

/// Types of key mappings.
enum MappingType {
  /// Simple remap: press A outputs B.
  simple,

  /// Tap-hold: tap outputs tap action, hold outputs hold action.
  tapHold,

  /// Layer activation: activates a layer while held.
  layer,
}

/// Overlay widget that draws mapping arrows on top of the keyboard.
class MappingOverlay extends StatelessWidget {
  const MappingOverlay({
    super.key,
    required this.mappings,
    required this.layout,
    this.selectedMappingIndex,
    this.onMappingTap,
    this.onMappingDelete,
    this.dragStartKey,
    this.dragCurrentPosition,
  });

  /// List of current mappings to display.
  final List<RemapConfig> mappings;

  /// Keyboard layout for calculating positions.
  final KeyboardLayout layout;

  /// Index of selected mapping (for highlighting).
  final int? selectedMappingIndex;

  /// Callback when a mapping arrow is tapped.
  final void Function(int index)? onMappingTap;

  /// Callback when a mapping's delete button is pressed.
  final void Function(int index)? onMappingDelete;

  /// Key being dragged from (for in-progress drag visualization).
  final String? dragStartKey;

  /// Current drag position (for in-progress drag visualization).
  final Offset? dragCurrentPosition;

  @override
  Widget build(BuildContext context) {
    return Stack(
      children: [
        // Visuals layer (arrows) - Ignore pointer events so keys below can be clicked
        Positioned.fill(
          child: IgnorePointer(
            child: CustomPaint(
              painter: _MappingPainter(
                mappings: mappings,
                layout: layout,
                selectedIndex: selectedMappingIndex,
                dragStartKey: dragStartKey,
                dragCurrentPosition: dragCurrentPosition,
                colorScheme: Theme.of(context).colorScheme,
              ),
            ),
          ),
        ),
        // Interactive layer (delete buttons)
        Stack(
          children: [
            for (int i = 0; i < mappings.length; i++)
              _buildDeleteButton(context, i),
          ],
        ),
      ],
    );
  }

  Widget _buildDeleteButton(BuildContext context, int index) {
    final mapping = mappings[index];
    final sourceKey = layout.findKey(mapping.sourceKeyId);
    final targetKey = layout.findKey(mapping.targetKeyId);

    if (sourceKey == null || targetKey == null) return const SizedBox.shrink();

    final sourcePos = layout.getKeyPosition(sourceKey);
    final sourceSize = layout.getKeySize(sourceKey);
    final targetPos = layout.getKeyPosition(targetKey);
    final targetSize = layout.getKeySize(targetKey);

    // Position delete button at midpoint of arrow
    final midX = (sourcePos.x + sourceSize.width / 2 +
            targetPos.x + targetSize.width / 2) /
        2;
    final midY = (sourcePos.y + sourceSize.height / 2 +
            targetPos.y + targetSize.height / 2) /
        2;

    return Positioned(
      left: midX - 12,
      top: midY - 12,
      child: GestureDetector(
        onTap: () => onMappingTap?.call(index),
        child: MouseRegion(
          cursor: SystemMouseCursors.click,
          child: Container(
            width: 24,
            height: 24,
            decoration: BoxDecoration(
              color: selectedMappingIndex == index
                  ? Theme.of(context).colorScheme.error
                  : Theme.of(context).colorScheme.surface,
              shape: BoxShape.circle,
              border: Border.all(
                color: Theme.of(context).colorScheme.outline,
                width: 1,
              ),
              boxShadow: [
                BoxShadow(
                  color: Colors.black.withValues(alpha: 0.2),
                  blurRadius: 4,
                  offset: const Offset(0, 2),
                ),
              ],
            ),
            child: selectedMappingIndex == index
                ? IconButton(
                    padding: EdgeInsets.zero,
                    iconSize: 14,
                    icon: const Icon(Icons.close, color: Colors.white),
                    onPressed: () => onMappingDelete?.call(index),
                  )
                : Center(
                    child: Icon(
                      Icons.arrow_forward,
                      size: 14,
                      color: Theme.of(context).colorScheme.onSurface,
                    ),
                  ),
          ),
        ),
      ),
    );
  }
}

/// Custom painter for drawing mapping arrows.
class _MappingPainter extends CustomPainter {
  _MappingPainter({
    required this.mappings,
    required this.layout,
    required this.colorScheme,
    this.selectedIndex,
    this.dragStartKey,
    this.dragCurrentPosition,
  });

  final List<RemapConfig> mappings;
  final KeyboardLayout layout;
  final ColorScheme colorScheme;
  final int? selectedIndex;
  final String? dragStartKey;
  final Offset? dragCurrentPosition;

  @override
  void paint(Canvas canvas, Size size) {
    // Draw existing mappings
    for (int i = 0; i < mappings.length; i++) {
      _drawMapping(canvas, mappings[i], isSelected: i == selectedIndex);
    }

    // Draw in-progress drag line
    if (dragStartKey != null && dragCurrentPosition != null) {
      _drawDragLine(canvas);
    }
  }

  void _drawMapping(Canvas canvas, RemapConfig mapping, {bool isSelected = false}) {
    final sourceKey = layout.findKey(mapping.sourceKeyId);
    final targetKey = layout.findKey(mapping.targetKeyId);

    if (sourceKey == null || targetKey == null) return;

    final sourcePos = layout.getKeyPosition(sourceKey);
    final sourceSize = layout.getKeySize(sourceKey);
    final targetPos = layout.getKeyPosition(targetKey);
    final targetSize = layout.getKeySize(targetKey);

    final sourceCenter = Offset(
      sourcePos.x + sourceSize.width / 2,
      sourcePos.y + sourceSize.height / 2,
    );
    final targetCenter = Offset(
      targetPos.x + targetSize.width / 2,
      targetPos.y + targetSize.height / 2,
    );

    final color = _getMappingColor(mapping.type, isSelected);
    final paint = Paint()
      ..color = color
      ..strokeWidth = isSelected ? 3.0 : 2.0
      ..style = PaintingStyle.stroke
      ..strokeCap = StrokeCap.round;

    // Draw curved arrow using quadratic bezier
    final controlPoint = _calculateControlPoint(sourceCenter, targetCenter);
    final path = Path()
      ..moveTo(sourceCenter.dx, sourceCenter.dy)
      ..quadraticBezierTo(
        controlPoint.dx,
        controlPoint.dy,
        targetCenter.dx,
        targetCenter.dy,
      );

    canvas.drawPath(path, paint);

    // Draw arrowhead at target
    _drawArrowhead(canvas, controlPoint, targetCenter, color, isSelected);
  }

  void _drawDragLine(Canvas canvas) {
    final sourceKey = layout.findKey(dragStartKey!);
    if (sourceKey == null) return;

    final sourcePos = layout.getKeyPosition(sourceKey);
    final sourceSize = layout.getKeySize(sourceKey);
    final sourceCenter = Offset(
      sourcePos.x + sourceSize.width / 2,
      sourcePos.y + sourceSize.height / 2,
    );

    final paint = Paint()
      ..color = colorScheme.primary.withValues(alpha: 0.6)
      ..strokeWidth = 2.0
      ..style = PaintingStyle.stroke
      ..strokeCap = StrokeCap.round;

    // Dashed line for in-progress drag
    final path = Path()
      ..moveTo(sourceCenter.dx, sourceCenter.dy)
      ..lineTo(dragCurrentPosition!.dx, dragCurrentPosition!.dy);

    _drawDashedPath(canvas, path, paint);

    // Draw circle at drag position
    canvas.drawCircle(
      dragCurrentPosition!,
      8,
      Paint()..color = colorScheme.primary.withValues(alpha: 0.4),
    );
  }

  void _drawDashedPath(Canvas canvas, Path path, Paint paint) {
    const dashWidth = 8.0;
    const dashSpace = 4.0;

    final pathMetrics = path.computeMetrics();
    for (final metric in pathMetrics) {
      var distance = 0.0;
      while (distance < metric.length) {
        final segment = metric.extractPath(
          distance,
          (distance + dashWidth).clamp(0, metric.length),
        );
        canvas.drawPath(segment, paint);
        distance += dashWidth + dashSpace;
      }
    }
  }

  Offset _calculateControlPoint(Offset source, Offset target) {
    // Calculate perpendicular offset for curved arrow
    final dx = target.dx - source.dx;
    final dy = target.dy - source.dy;
    final distance = (dx * dx + dy * dy).clamp(1, double.infinity);
    final curvature = (distance / 4).clamp(20, 80);

    // Perpendicular direction
    final perpX = -dy / distance.toDouble().clamp(1, double.infinity);
    final perpY = dx / distance.toDouble().clamp(1, double.infinity);

    return Offset(
      (source.dx + target.dx) / 2 + perpX * curvature,
      (source.dy + target.dy) / 2 + perpY * curvature,
    );
  }

  void _drawArrowhead(
    Canvas canvas,
    Offset from,
    Offset to,
    Color color,
    bool isSelected,
  ) {
    final paint = Paint()
      ..color = color
      ..style = PaintingStyle.fill;

    // Calculate direction from control point to target
    final dx = to.dx - from.dx;
    final dy = to.dy - from.dy;
    final len = (dx * dx + dy * dy).clamp(1, double.infinity);
    final unitDx = dx / len.toDouble().clamp(1, double.infinity);
    final unitDy = dy / len.toDouble().clamp(1, double.infinity);

    final arrowSize = isSelected ? 12.0 : 10.0;

    // Arrowhead points
    final tip = to;
    final left = Offset(
      to.dx - unitDx * arrowSize - unitDy * arrowSize / 2,
      to.dy - unitDy * arrowSize + unitDx * arrowSize / 2,
    );
    final right = Offset(
      to.dx - unitDx * arrowSize + unitDy * arrowSize / 2,
      to.dy - unitDy * arrowSize - unitDx * arrowSize / 2,
    );

    final path = Path()
      ..moveTo(tip.dx, tip.dy)
      ..lineTo(left.dx, left.dy)
      ..lineTo(right.dx, right.dy)
      ..close();

    canvas.drawPath(path, paint);
  }

  Color _getMappingColor(MappingType type, bool isSelected) {
    if (isSelected) {
      return colorScheme.primary;
    }

    return switch (type) {
      MappingType.simple => colorScheme.tertiary,
      MappingType.tapHold => colorScheme.secondary,
      MappingType.layer => colorScheme.primary.withValues(alpha: 0.7),
    };
  }

  @override
  bool shouldRepaint(_MappingPainter oldDelegate) {
    return mappings != oldDelegate.mappings ||
        selectedIndex != oldDelegate.selectedIndex ||
        dragStartKey != oldDelegate.dragStartKey ||
        dragCurrentPosition != oldDelegate.dragCurrentPosition;
  }
}
