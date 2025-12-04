/// Styled icon button variants with hover and press states.
///
/// Provides reusable icon button components with consistent styling,
/// hover effects, and press states for use throughout the application.
library;

import 'package:flutter/material.dart';

/// A styled icon button with hover and press states.
///
/// This widget wraps Flutter's IconButton with additional visual feedback
/// for hover and press states, providing a more polished user experience.
///
/// Example:
/// ```dart
/// StyledIconButton(
///   icon: Icons.delete,
///   onPressed: () => print('Delete clicked'),
///   tooltip: 'Delete item',
/// )
/// ```
class StyledIconButton extends StatefulWidget {
  const StyledIconButton({
    super.key,
    required this.icon,
    required this.onPressed,
    this.tooltip,
    this.color,
    this.hoverColor,
    this.size = 24.0,
    this.padding = const EdgeInsets.all(8.0),
  });

  /// The icon to display.
  final IconData icon;

  /// Callback when the button is pressed.
  final VoidCallback? onPressed;

  /// Optional tooltip text shown on hover.
  final String? tooltip;

  /// The color of the icon. Defaults to theme's icon color.
  final Color? color;

  /// The color of the icon when hovered. Defaults to theme's primary color.
  final Color? hoverColor;

  /// The size of the icon in logical pixels.
  final double size;

  /// The padding around the icon.
  final EdgeInsets padding;

  @override
  State<StyledIconButton> createState() => _StyledIconButtonState();
}

class _StyledIconButtonState extends State<StyledIconButton> {
  bool _isHovered = false;
  bool _isPressed = false;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;

    final bool enabled = widget.onPressed != null;

    final iconColor = enabled
        ? (_isPressed
            ? (widget.hoverColor ?? colorScheme.primary).withValues(alpha: 0.7)
            : _isHovered
                ? (widget.hoverColor ?? colorScheme.primary)
                : (widget.color ?? colorScheme.onSurface))
        : colorScheme.onSurface.withValues(alpha: 0.38);

    final button = MouseRegion(
      onEnter: enabled ? (_) => setState(() => _isHovered = true) : null,
      onExit: enabled ? (_) => setState(() => _isHovered = false) : null,
      cursor: enabled ? SystemMouseCursors.click : SystemMouseCursors.basic,
      child: GestureDetector(
        onTapDown: enabled ? (_) => setState(() => _isPressed = true) : null,
        onTapUp: enabled ? (_) => setState(() => _isPressed = false) : null,
        onTapCancel: enabled ? () => setState(() => _isPressed = false) : null,
        onTap: widget.onPressed,
        child: Padding(
          padding: widget.padding,
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 150),
            child: Icon(
              widget.icon,
              size: widget.size,
              color: iconColor,
            ),
          ),
        ),
      ),
    );

    if (widget.tooltip != null) {
      return Tooltip(
        message: widget.tooltip!,
        child: button,
      );
    }

    return button;
  }
}

/// A compact styled icon button for use in toolbars and tight spaces.
///
/// This variant has smaller default size and padding compared to [StyledIconButton].
///
/// Example:
/// ```dart
/// CompactIconButton(
///   icon: Icons.edit,
///   onPressed: () => print('Edit clicked'),
/// )
/// ```
class CompactIconButton extends StatelessWidget {
  const CompactIconButton({
    super.key,
    required this.icon,
    required this.onPressed,
    this.tooltip,
    this.color,
    this.hoverColor,
  });

  /// The icon to display.
  final IconData icon;

  /// Callback when the button is pressed.
  final VoidCallback? onPressed;

  /// Optional tooltip text shown on hover.
  final String? tooltip;

  /// The color of the icon.
  final Color? color;

  /// The color of the icon when hovered.
  final Color? hoverColor;

  @override
  Widget build(BuildContext context) {
    return StyledIconButton(
      icon: icon,
      onPressed: onPressed,
      tooltip: tooltip,
      color: color,
      hoverColor: hoverColor,
      size: 18.0,
      padding: const EdgeInsets.all(4.0),
    );
  }
}

/// A styled icon button with a background fill.
///
/// This variant includes a background color that changes on hover,
/// useful for primary actions or emphasized buttons.
///
/// Example:
/// ```dart
/// FilledIconButton(
///   icon: Icons.add,
///   onPressed: () => print('Add clicked'),
///   tooltip: 'Add item',
/// )
/// ```
class FilledIconButton extends StatefulWidget {
  const FilledIconButton({
    super.key,
    required this.icon,
    required this.onPressed,
    this.tooltip,
    this.backgroundColor,
    this.foregroundColor,
    this.size = 24.0,
    this.padding = const EdgeInsets.all(12.0),
  });

  /// The icon to display.
  final IconData icon;

  /// Callback when the button is pressed.
  final VoidCallback? onPressed;

  /// Optional tooltip text shown on hover.
  final String? tooltip;

  /// The background color. Defaults to theme's primary color.
  final Color? backgroundColor;

  /// The icon color. Defaults to theme's onPrimary color.
  final Color? foregroundColor;

  /// The size of the icon in logical pixels.
  final double size;

  /// The padding around the icon.
  final EdgeInsets padding;

  @override
  State<FilledIconButton> createState() => _FilledIconButtonState();
}

class _FilledIconButtonState extends State<FilledIconButton> {
  bool _isHovered = false;
  bool _isPressed = false;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;

    final bool enabled = widget.onPressed != null;

    final backgroundColor = enabled
        ? (_isPressed
            ? (widget.backgroundColor ?? colorScheme.primary)
                .withValues(alpha: 0.7)
            : _isHovered
                ? (widget.backgroundColor ?? colorScheme.primary)
                    .withValues(alpha: 0.9)
                : (widget.backgroundColor ?? colorScheme.primary))
        : colorScheme.onSurface.withValues(alpha: 0.12);

    final foregroundColor = enabled
        ? (widget.foregroundColor ?? colorScheme.onPrimary)
        : colorScheme.onSurface.withValues(alpha: 0.38);

    final button = MouseRegion(
      onEnter: enabled ? (_) => setState(() => _isHovered = true) : null,
      onExit: enabled ? (_) => setState(() => _isHovered = false) : null,
      cursor: enabled ? SystemMouseCursors.click : SystemMouseCursors.basic,
      child: GestureDetector(
        onTapDown: enabled ? (_) => setState(() => _isPressed = true) : null,
        onTapUp: enabled ? (_) => setState(() => _isPressed = false) : null,
        onTapCancel: enabled ? () => setState(() => _isPressed = false) : null,
        onTap: widget.onPressed,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 150),
          padding: widget.padding,
          decoration: BoxDecoration(
            color: backgroundColor,
            borderRadius: BorderRadius.circular(8),
          ),
          child: Icon(
            widget.icon,
            size: widget.size,
            color: foregroundColor,
          ),
        ),
      ),
    );

    if (widget.tooltip != null) {
      return Tooltip(
        message: widget.tooltip!,
        child: button,
      );
    }

    return button;
  }
}
