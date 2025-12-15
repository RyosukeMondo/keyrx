import 'dart:math';
import 'dart:ui';

import 'package:flutter/animation.dart';

/// Controls the animation state of an edge
class EdgeAnimationState {
  EdgeAnimationState({
    this.opacity = 1.0,
    this.pulsePhase = 0.0,
    this.colorTransition = 1.0,
  });

  /// Opacity for fade in/out animations (0.0 to 1.0)
  final double opacity;

  /// Phase for pulse animation (0.0 to 1.0, cycles)
  final double pulsePhase;

  /// Color transition progress (0.0 to 1.0, for validation state changes)
  final double colorTransition;

  EdgeAnimationState copyWith({
    double? opacity,
    double? pulsePhase,
    double? colorTransition,
  }) {
    return EdgeAnimationState(
      opacity: opacity ?? this.opacity,
      pulsePhase: pulsePhase ?? this.pulsePhase,
      colorTransition: colorTransition ?? this.colorTransition,
    );
  }

  @override
  bool operator ==(Object other) {
    return other is EdgeAnimationState &&
        other.opacity == opacity &&
        other.pulsePhase == pulsePhase &&
        other.colorTransition == colorTransition;
  }

  @override
  int get hashCode => Object.hash(opacity, pulsePhase, colorTransition);
}

/// Animation controller for edge lifecycle transitions
class EdgeLifecycleAnimation {
  EdgeLifecycleAnimation({
    required this.edgeId,
    required this.controller,
    required this.type,
  });

  final String edgeId;
  final AnimationController controller;
  final EdgeAnimationType type;

  Animation<double> get animation => controller;
}

enum EdgeAnimationType { fadeIn, fadeOut, colorTransition }

/// Utility functions for edge animations
class EdgeAnimations {
  /// Creates a pulse effect for active slots
  /// Returns a value between 0.8 and 1.2 for subtle pulsing
  static double pulseScale(double phase) {
    // Use sine wave for smooth pulsing
    final normalizedPhase = phase * 2 * pi; // Convert to radians
    return 1.0 + (0.15 * (sin(normalizedPhase) * 0.5 + 0.5));
  }

  /// Computes glow intensity for pulse animation
  /// Returns opacity value between 0.0 and 0.4
  static double pulseGlow(double phase) {
    final normalizedPhase = phase * 2 * pi;
    return 0.25 * (sin(normalizedPhase) * 0.5 + 0.5);
  }

  /// Applies easing curve for fade animations
  static double fadeEasing(double t) {
    return Curves.easeInOut.transform(t);
  }

  /// Applies easing curve for color transitions
  static double colorTransitionEasing(double t) {
    return Curves.easeInOutCubic.transform(t);
  }

  /// Interpolates between two colors with easing
  static Color lerpColor(Color a, Color b, double t) {
    final easedT = colorTransitionEasing(t);
    return Color.lerp(a, b, easedT) ?? a;
  }

  /// Calculates anti-aliased stroke width adjustment
  /// Adds slight blur for smoother edges
  static Paint applyAntiAliasing(Paint paint) {
    return paint
      ..strokeCap = StrokeCap.round
      ..strokeJoin = StrokeJoin.round
      ..isAntiAlias = true;
  }
}

/// Cache key for edge paths to avoid recomputation
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
  bool operator ==(Object other) {
    return other is EdgePathCacheKey &&
        other.startX == startX &&
        other.startY == startY &&
        other.endX == endX &&
        other.endY == endY &&
        other.active == active;
  }

  @override
  int get hashCode => Object.hash(startX, startY, endX, endY, active);
}
