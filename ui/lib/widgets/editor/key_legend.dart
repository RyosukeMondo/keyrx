/// KeyLegend widget for displaying key color legend.
///
/// Provides a visual legend explaining the color coding used for keys
/// in the keyboard editor, supporting both horizontal and vertical layouts.
library;

import 'package:flutter/material.dart';

/// Visual legend displaying key color meanings.
///
/// Shows a list of legend items with color indicators and labels,
/// useful for explaining what different key colors represent in the editor.
class KeyLegend extends StatelessWidget {
  /// Creates a KeyLegend widget.
  ///
  /// [items] defines the legend entries to display.
  /// [orientation] controls layout direction (horizontal or vertical).
  const KeyLegend({
    super.key,
    required this.items,
    this.orientation = Axis.horizontal,
  });

  /// The list of legend items to display.
  final List<LegendItem> items;

  /// The layout orientation (horizontal or vertical).
  final Axis orientation;

  @override
  Widget build(BuildContext context) {
    final legendWidgets = items.map((item) => _buildLegendItem(item)).toList();

    if (orientation == Axis.horizontal) {
      return Wrap(
        spacing: 16,
        runSpacing: 8,
        children: legendWidgets,
      );
    } else {
      return Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: legendWidgets
            .map((widget) => Padding(
                  padding: const EdgeInsets.symmetric(vertical: 4),
                  child: widget,
                ))
            .toList(),
      );
    }
  }

  /// Builds a single legend item with color indicator and label.
  Widget _buildLegendItem(LegendItem item) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        if (item.icon != null) ...[
          Icon(item.icon, size: 16, color: item.color),
          const SizedBox(width: 6),
        ] else ...[
          Container(
            width: 16,
            height: 16,
            decoration: BoxDecoration(
              color: item.color,
              borderRadius: BorderRadius.circular(4),
              border: Border.all(
                color: item.color.withValues(alpha: 0.3),
                width: 1,
              ),
            ),
          ),
          const SizedBox(width: 8),
        ],
        Text(
          item.label,
          style: const TextStyle(fontSize: 13),
        ),
        if (item.tooltip != null)
          Padding(
            padding: const EdgeInsets.only(left: 4),
            child: Tooltip(
              message: item.tooltip!,
              child: Icon(
                Icons.info_outline,
                size: 14,
                color: Colors.grey[600],
              ),
            ),
          ),
      ],
    );
  }
}

/// Represents a single item in the key legend.
///
/// Contains display information for a legend entry including color,
/// label text, optional icon, and tooltip.
class LegendItem {
  /// Creates a LegendItem.
  ///
  /// [label] is the text displayed for this legend entry.
  /// [color] is the color indicator for this entry.
  /// [icon] optionally replaces the color box with an icon.
  /// [tooltip] provides additional information on hover.
  const LegendItem({
    required this.label,
    required this.color,
    this.icon,
    this.tooltip,
  });

  /// The text label for this legend item.
  final String label;

  /// The color associated with this legend item.
  final Color color;

  /// Optional icon to display instead of a color box.
  final IconData? icon;

  /// Optional tooltip text for additional information.
  final String? tooltip;
}
