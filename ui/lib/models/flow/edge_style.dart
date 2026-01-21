import 'package:flutter/material.dart';
import 'flow_graph.dart';

class EdgePalette {
  const EdgePalette({
    required this.valid,
    required this.invalid,
    required this.warning,
    required this.inactive,
    required this.hoverGlow,
    required this.priorityBackground,
    required this.priorityText,
  });

  final Color valid;
  final Color invalid;
  final Color warning;
  final Color inactive;
  final Color hoverGlow;
  final Color priorityBackground;
  final Color priorityText;
}

const defaultEdgePalette = EdgePalette(
  valid: Color(0xFF21D07A), // green
  invalid: Color(0xFFE74C3C), // red
  warning: Color(0xFFF1C40F), // yellow
  inactive: Color(0xFF6C7A89), // grey-blue for contrast
  hoverGlow: Color(0xFF8EE0FF), // cyan highlight
  priorityBackground: Color(0xFF1A1F2E),
  priorityText: Color(0xFFE8ECF5),
);

class EdgeStyle {
  const EdgeStyle({
    required this.palette, this.activeWidth = 3.0,
    this.inactiveWidth = 2.2,
    this.hoverWidth = 4.0,
    this.dashLength = 8.0,
    this.dashGap = 6.0,
    this.hoverHitRadius = 12.0,
    this.priorityBadgeRadius = 14.0,
  });

  final double activeWidth;
  final double inactiveWidth;
  final double hoverWidth;
  final double dashLength;
  final double dashGap;
  final double hoverHitRadius;
  final double priorityBadgeRadius;
  final EdgePalette palette;

  Color colorForStatus(
    EdgeStatus status, {
    required bool active,
    required bool hovered,
  }) {
    final base = switch (status) {
      EdgeStatus.invalid => palette.invalid,
      EdgeStatus.warning => palette.warning,
      EdgeStatus.valid => active ? palette.valid : palette.inactive,
    };
    if (hovered) {
      return Color.alphaBlend(palette.hoverGlow.withOpacity(0.28), base);
    }
    return base;
  }
}
