import 'package:flutter/material.dart';

/// A pure visual representation of a virtual key.
///
/// Does not handle positioning or interaction.
class VirtualKeyVisual extends StatelessWidget {
  const VirtualKeyVisual({
    super.key,
    required this.label,
    required this.id,
    this.isSelected = false,
    this.isMapped = false,
    this.isHovered = false,
  });

  final String label;
  final String id;
  final bool isSelected;
  final bool isMapped;
  final bool isHovered;

  @override
  Widget build(BuildContext context) {
    // Determine colors based on state priority: Selected > Hovered > Mapped > Default
    // But Hovered should probably overlay mapped?
    // Let's keep logic simple and consistent with previous "Wiring" and "Editor" logic.

    final colorScheme = Theme.of(context).colorScheme;

    Color? backgroundColor;
    Color borderColor;
    double borderWidth = 1.0;
    FontWeight fontWeight = FontWeight.normal;
    Color? textColor;

    if (isSelected) {
      backgroundColor = colorScheme.primaryContainer;
      borderColor = colorScheme.primary;
      borderWidth = 2.0;
      fontWeight = FontWeight.bold;
      textColor = colorScheme.onPrimaryContainer;
    } else if (isMapped) {
      backgroundColor = colorScheme.secondaryContainer.withValues(alpha: 0.5);
      borderColor = colorScheme.secondary;
      textColor = colorScheme.onSecondaryContainer;
    } else {
      backgroundColor = colorScheme.surface;
      borderColor = Theme.of(context).dividerColor;
    }

    if (isHovered && !isSelected) {
      // Lighten/Highlight on hover if needed, or handled by parent InkWell?
      // Since this is just a Container, we can add visual feedback here.
      backgroundColor = Color.alphaBlend(
        colorScheme.onSurface.withValues(alpha: 0.05),
        backgroundColor,
      );
    }

    return Container(
      margin: const EdgeInsets.all(2), // Gutter
      decoration: BoxDecoration(
        color: backgroundColor,
        borderRadius: BorderRadius.circular(4),
        border: Border.all(color: borderColor, width: borderWidth),
      ),
      alignment: Alignment.center,
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Text(
            label.isEmpty && id.isNotEmpty ? id : label,
            style: TextStyle(
              fontSize: 12,
              fontWeight: fontWeight,
              color: textColor,
            ),
            overflow: TextOverflow.ellipsis,
            textAlign: TextAlign.center,
          ),
        ],
      ),
    );
  }
}

/// Paints a grid background.
class VirtualLayoutGridPainter extends CustomPainter {
  final double step;
  final double? subStep;
  final Color color;

  VirtualLayoutGridPainter({
    required this.step,
    required this.color,
    this.subStep,
  });

  @override
  void paint(Canvas canvas, Size size) {
    if (subStep != null) {
      final paintMinor = Paint()
        ..color = color
            .withValues(alpha: 0.1) // Assumes color passed is already low opacity?
        // Logic in consumers was: color: dividerColor.withValues(alpha: 0.1).
        // Let's assume 'color' is the MAJOR grid color. Minor is dimmer.
        // Or better: Use the color provided for major, and derive minor.
        ..color = color.withValues(alpha: color.opacity * 0.5)
        ..strokeWidth = 1;

      for (double x = 0; x <= size.width; x += subStep!) {
        canvas.drawLine(Offset(x, 0), Offset(x, size.height), paintMinor);
      }
      for (double y = 0; y <= size.height; y += subStep!) {
        canvas.drawLine(Offset(0, y), Offset(size.width, y), paintMinor);
      }
    }

    final paintMajor = Paint()
      ..color = color
      ..strokeWidth = 1;

    for (double x = 0; x <= size.width; x += step) {
      canvas.drawLine(Offset(x, 0), Offset(x, size.height), paintMajor);
    }

    for (double y = 0; y <= size.height; y += step) {
      canvas.drawLine(Offset(0, y), Offset(size.width, y), paintMajor);
    }
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => false;
}

