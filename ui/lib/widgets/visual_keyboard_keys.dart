/// Key rendering widgets for the visual keyboard.
///
/// Contains [KeyWidget] which renders individual keyboard keys with
/// proper styling, state visualization, and interaction handling.
library;

import 'package:flutter/material.dart';

import '../config/config.dart';
import '../models/keyboard_layout.dart';

/// Individual key widget within the visual keyboard.
///
/// Renders a single key with support for multiple visual states:
/// - Selected (user selected for editing)
/// - Highlighted (part of active combo)
/// - Mapped (has a remapping configured)
/// - Held (physically pressed)
/// - Hovered (mouse over)
/// - Drag source (being dragged for mapping)
class KeyWidget extends StatelessWidget {
  const KeyWidget({
    super.key,
    required this.keyDef,
    required this.width,
    required this.height,
    this.isSelected = false,
    this.isHighlighted = false,
    this.isMapped = false,
    this.isHeld = false,
    this.isHovered = false,
    this.isDragSource = false,
    this.showSecondaryLabel = true,
    this.enabled = true,
    this.onTap,
    this.onLongPress,
    this.onHoverChanged,
  });

  /// The key definition containing label and metadata.
  final KeyDefinition keyDef;

  /// Width of the key in pixels.
  final double width;

  /// Height of the key in pixels.
  final double height;

  /// Whether this key is selected.
  final bool isSelected;

  /// Whether this key is highlighted (e.g., combo in progress).
  final bool isHighlighted;

  /// Whether this key has a mapping configured.
  final bool isMapped;

  /// Whether this key is currently held down.
  final bool isHeld;

  /// Whether the mouse is hovering over this key.
  final bool isHovered;

  /// Whether this key is the source of a drag operation.
  final bool isDragSource;

  /// Whether to show the secondary label (shifted character).
  final bool showSecondaryLabel;

  /// Whether this key is interactive.
  final bool enabled;

  /// Callback when this key is tapped.
  final VoidCallback? onTap;

  /// Callback when this key is long-pressed.
  final VoidCallback? onLongPress;

  /// Callback when hover state changes.
  final void Function(bool hovered)? onHoverChanged;

  @override
  Widget build(BuildContext context) {
    final colorScheme = Theme.of(context).colorScheme;

    final backgroundColor = _getBackgroundColor(colorScheme);
    final borderColor = _getBorderColor(colorScheme);
    final textColor = _getTextColor(colorScheme);

    return MouseRegion(
      onEnter: (_) => onHoverChanged?.call(true),
      onExit: (_) => onHoverChanged?.call(false),
      child: GestureDetector(
        onTap: enabled ? onTap : null,
        onLongPress: enabled ? onLongPress : null,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: TimingConfig.keyAnimationMs),
          width: width,
          height: height,
          decoration: BoxDecoration(
            color: backgroundColor,
            borderRadius: BorderRadius.circular(6),
            border: Border.all(
              color: borderColor,
              width: isSelected || isHighlighted ? 2 : 1,
            ),
            boxShadow: isHeld || isSelected
                ? []
                : [
                    BoxShadow(
                      color: Colors.black.withValues(alpha: 0.3),
                      offset: const Offset(0, 2),
                      blurRadius: 2,
                    ),
                  ],
          ),
          child: Stack(
            children: [
              // Main label
              Center(
                child: Text(
                  keyDef.label,
                  style: TextStyle(
                    color: textColor,
                    fontSize: _getLabelFontSize(),
                    fontWeight: FontWeight.w500,
                  ),
                  textAlign: TextAlign.center,
                  overflow: TextOverflow.ellipsis,
                ),
              ),
              // Secondary label (shifted character)
              if (showSecondaryLabel && keyDef.secondaryLabel != null)
                Positioned(
                  top: 4,
                  left: 6,
                  child: Text(
                    keyDef.secondaryLabel!,
                    style: TextStyle(
                      color: textColor.withValues(alpha: 0.6),
                      fontSize: 10,
                    ),
                  ),
                ),
              // Mapped indicator
              if (isMapped)
                Positioned(
                  top: 4,
                  right: 4,
                  child: Container(
                    width: 8,
                    height: 8,
                    decoration: BoxDecoration(
                      color: colorScheme.tertiary,
                      shape: BoxShape.circle,
                    ),
                  ),
                ),
            ],
          ),
        ),
      ),
    );
  }

  Color _getBackgroundColor(ColorScheme colorScheme) {
    if (isDragSource) {
      return colorScheme.primary.withValues(alpha: 0.6);
    }
    if (isHeld) {
      return colorScheme.primary.withValues(alpha: 0.8);
    }
    if (isSelected) {
      return colorScheme.primaryContainer;
    }
    if (isHighlighted) {
      return colorScheme.secondaryContainer;
    }
    if (isHovered && enabled) {
      return colorScheme.surfaceContainerHighest;
    }
    return colorScheme.surfaceContainerHigh;
  }

  Color _getBorderColor(ColorScheme colorScheme) {
    if (isDragSource) {
      return colorScheme.primary;
    }
    if (isSelected) {
      return colorScheme.primary;
    }
    if (isHighlighted) {
      return colorScheme.secondary;
    }
    if (isMapped) {
      return colorScheme.tertiary.withValues(alpha: 0.5);
    }
    return colorScheme.outline.withValues(alpha: 0.3);
  }

  Color _getTextColor(ColorScheme colorScheme) {
    if (isHeld) {
      return colorScheme.onPrimary;
    }
    if (isSelected) {
      return colorScheme.onPrimaryContainer;
    }
    if (!enabled) {
      return colorScheme.onSurface.withValues(alpha: 0.38);
    }
    return colorScheme.onSurface;
  }

  double _getLabelFontSize() {
    // Adjust font size based on key width and label length
    if (keyDef.label.length > 5) {
      return 10;
    }
    if (keyDef.label.length > 3) {
      return 11;
    }
    if (width < 50) {
      return 12;
    }
    return 14;
  }
}
