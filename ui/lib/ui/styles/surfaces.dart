import 'package:flutter/material.dart';

import '../../config/config.dart';

/// Centralized surface styling for shared containers.
class SurfaceStyles {
  const SurfaceStyles._();

  /// Default padding applied to surfaces.
  static const EdgeInsets defaultPadding = EdgeInsets.all(
    UiConstants.defaultPadding,
  );

  /// Default outer margin for separation between surfaces.
  static const EdgeInsets defaultMargin = EdgeInsets.all(
    UiConstants.defaultMargin,
  );

  /// Standard border radius for rounded surfaces.
  static const BorderRadius defaultRadius = BorderRadius.all(
    Radius.circular(UiConstants.surfaceBorderRadius),
  );

  /// Standard elevation for subtle depth.
  static const double defaultElevation = UiConstants.defaultElevation;

  /// Standard surface color that respects the active theme.
  static Color surfaceColor(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    return scheme.surfaceContainerHighest.withValues(alpha: 0.85);
  }

  /// Border color tuned for dark and light modes.
  static Color borderColor(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    return scheme.outlineVariant.withValues(alpha: 0.6);
  }
}

/// Reusable, theme-aware container for grouped content.
class SurfaceContainer extends StatelessWidget {
  final Widget child;
  final EdgeInsetsGeometry padding;
  final EdgeInsetsGeometry margin;
  final BorderRadiusGeometry borderRadius;
  final Color? color;
  final double elevation;

  const SurfaceContainer({
    super.key,
    required this.child,
    this.padding = SurfaceStyles.defaultPadding,
    this.margin = SurfaceStyles.defaultMargin,
    this.borderRadius = SurfaceStyles.defaultRadius,
    this.color,
    this.elevation = SurfaceStyles.defaultElevation,
  });

  @override
  Widget build(BuildContext context) {
    final surfaceColor = color ?? SurfaceStyles.surfaceColor(context);

    return Padding(
      padding: margin,
      child: Material(
        color: surfaceColor,
        elevation: elevation,
        borderRadius: borderRadius,
        child: Container(
          decoration: BoxDecoration(
            borderRadius: borderRadius,
            border: Border.all(color: SurfaceStyles.borderColor(context)),
          ),
          child: Padding(padding: padding, child: child),
        ),
      ),
    );
  }
}
