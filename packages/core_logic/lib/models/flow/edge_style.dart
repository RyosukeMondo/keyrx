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

  final int valid;
  final int invalid;
  final int warning;
  final int inactive;
  final int hoverGlow;
  final int priorityBackground;
  final int priorityText;
}

const defaultEdgePalette = EdgePalette(
  valid: 0xFF21D07A, // green
  invalid: 0xFFE74C3C, // red
  warning: 0xFFF1C40F, // yellow
  inactive: 0xFF6C7A89, // grey-blue for contrast
  hoverGlow: 0xFF8EE0FF, // cyan highlight
  priorityBackground: 0xFF1A1F2E,
  priorityText: 0xFFE8ECF5,
);

class EdgeStyle {
  const EdgeStyle({
    required this.palette,
    this.activeWidth = 3.0,
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
}