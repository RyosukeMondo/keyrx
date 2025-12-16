/// Rendering optimization utilities for the KeyFlow editor.
///
/// Provides utilities for optimizing canvas rendering performance,
/// including viewport calculations, path caching, and selective repainting.
library;

import 'package:flutter/widgets.dart';

/// Determines which nodes are visible in the current viewport.
///
/// Returns true if the node bounds intersect with the visible viewport.
/// Includes a margin to handle nodes partially outside the viewport.
bool isNodeInViewport({
  required Offset nodePosition,
  required Size nodeSize,
  required Rect viewport,
  double margin = 100.0,
}) {
  final expandedViewport = viewport.inflate(margin);
  final nodeRect = nodePosition & nodeSize;
  return expandedViewport.overlaps(nodeRect);
}

/// Calculates the visible viewport from a transformation matrix.
///
/// Takes the current transformation (from InteractiveViewer) and screen size
/// to calculate what portion of the canvas is currently visible.
Rect calculateViewport({required Matrix4 transform, required Size screenSize}) {
  // Get the inverse transform to map screen space to canvas space
  final inverse = Matrix4.inverted(transform);

  // Transform the four corners of the screen to canvas space
  final topLeft = MatrixUtils.transformPoint(inverse, Offset.zero);
  final bottomRight = MatrixUtils.transformPoint(
    inverse,
    Offset(screenSize.width, screenSize.height),
  );

  return Rect.fromPoints(topLeft, bottomRight);
}

/// Cache key for edge paths.
///
/// Used to cache computed Bezier paths to avoid recalculation on every frame.
class EdgePathCacheKey {
  const EdgePathCacheKey({
    required this.startX,
    required this.startY,
    required this.endX,
    required this.endY,
    required this.active,
  });

  final double startX;
  final double startY;
  final double endX;
  final double endY;
  final bool active;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is EdgePathCacheKey &&
          runtimeType == other.runtimeType &&
          startX == other.startX &&
          startY == other.startY &&
          endX == other.endX &&
          endY == other.endY &&
          active == other.active;

  @override
  int get hashCode =>
      startX.hashCode ^
      startY.hashCode ^
      endX.hashCode ^
      endY.hashCode ^
      active.hashCode;
}

/// Mixin for widgets that need selective repainting based on viewport changes.
///
/// Tracks the previous viewport and only triggers rebuilds when the viewport
/// changes significantly (beyond the threshold).
mixin ViewportAware<T extends StatefulWidget> on State<T> {
  Rect? _previousViewport;

  /// Threshold for viewport change detection (in pixels).
  /// Only trigger rebuild if viewport moved/resized beyond this value.
  double get viewportChangeThreshold => 50;

  /// Check if viewport has changed significantly.
  bool hasViewportChanged(Rect newViewport) {
    if (_previousViewport == null) {
      _previousViewport = newViewport;
      return true;
    }

    final prev = _previousViewport!;
    final dx = (newViewport.left - prev.left).abs();
    final dy = (newViewport.top - prev.top).abs();
    final dw = (newViewport.width - prev.width).abs();
    final dh = (newViewport.height - prev.height).abs();

    if (dx > viewportChangeThreshold ||
        dy > viewportChangeThreshold ||
        dw > viewportChangeThreshold ||
        dh > viewportChangeThreshold) {
      _previousViewport = newViewport;
      return true;
    }

    return false;
  }

  /// Reset viewport tracking (call when canvas is reset).
  void resetViewportTracking() {
    _previousViewport = null;
  }
}

/// Performance metrics for profiling rendering performance.
class RenderingMetrics {
  const RenderingMetrics({
    required this.totalNodes,
    required this.visibleNodes,
    required this.totalEdges,
    required this.visibleEdges,
    required this.frameTime,
  });

  final int totalNodes;
  final int visibleNodes;
  final int totalEdges;
  final int visibleEdges;
  final Duration frameTime;

  /// Check if performance is within target (60fps = 16.67ms per frame).
  bool get meetsPerformanceTarget => frameTime.inMilliseconds <= 16;

  /// Virtualization efficiency (percentage of nodes virtualized).
  double get virtualizationEfficiency =>
      totalNodes > 0 ? (totalNodes - visibleNodes) / totalNodes : 0.0;

  @override
  String toString() =>
      'RenderingMetrics(\n'
      '  nodes: $visibleNodes/$totalNodes visible\n'
      '  edges: $visibleEdges/$totalEdges visible\n'
      '  frameTime: ${frameTime.inMilliseconds}ms\n'
      '  target: ${meetsPerformanceTarget ? "MET" : "EXCEEDED"}\n'
      '  virtualization: ${(virtualizationEfficiency * 100).toStringAsFixed(1)}%\n'
      ')';
}

/// Stopwatch-based frame timer for measuring rendering performance.
class FrameTimer {
  final Stopwatch _stopwatch = Stopwatch();

  /// Start timing a frame.
  void start() {
    _stopwatch.reset();
    _stopwatch.start();
  }

  /// Stop timing and return elapsed duration.
  Duration stop() {
    _stopwatch.stop();
    return _stopwatch.elapsed;
  }

  /// Get current elapsed time without stopping.
  Duration get elapsed => _stopwatch.elapsed;
}

/// Debouncer for reducing rebuild frequency.
///
/// Useful for expensive operations that shouldn't happen on every frame.
class RebuildDebouncer {
  RebuildDebouncer({required this.delay});

  final Duration delay;
  DateTime? _lastRebuild;

  /// Check if enough time has passed since last rebuild.
  bool shouldRebuild() {
    final now = DateTime.now();
    if (_lastRebuild == null || now.difference(_lastRebuild!) > delay) {
      _lastRebuild = now;
      return true;
    }
    return false;
  }

  /// Reset the debouncer.
  void reset() {
    _lastRebuild = null;
  }
}
