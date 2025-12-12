// Layout grid widget for displaying device layouts dynamically.
//
// Renders Matrix, Standard, and Split keyboard layouts with current
// mappings displayed on keys. Supports interactive key selection for
// mapping configuration.

import 'package:flutter/material.dart';
import '../models/keyboard_layout.dart';
import '../models/layout_type.dart';
import '../models/profile.dart';
import 'visual_keyboard.dart';

/// Layout information for rendering grids
class LayoutInfo {
  final int rows;
  final int cols;
  final LayoutType type;
  final List<int>? colsPerRow;

  const LayoutInfo({
    required this.rows,
    required this.cols,
    required this.type,
    this.colsPerRow,
  });
}

/// Widget that dynamically renders device layouts as interactive grids.
///
/// Supports Matrix (macro pads), Standard (keyboards), and Split layouts.
/// Shows current key mappings and allows key selection for remapping.
class LayoutGrid extends StatelessWidget {
  /// Layout information (rows, columns, type)
  final LayoutInfo layoutInfo;

  /// Current profile with mappings to display
  final Profile? profile;

  /// Callback when a key is tapped
  /// Arguments: (row, col)
  final void Function(int row, int col)? onKeyTap;

  /// Currently selected physical position (highlighted)
  final PhysicalPosition? selectedPosition;

  /// Positions to highlight (e.g., search matches).
  final Set<PhysicalPosition> highlightedPositions;

  /// Key size in pixels
  final double keySize;

  /// Spacing between keys in pixels
  final double keySpacing;

  const LayoutGrid({
    super.key,
    required this.layoutInfo,
    this.profile,
    this.onKeyTap,
    this.selectedPosition,
    this.highlightedPositions = const {},
    this.keySize = 48.0,
    this.keySpacing = 4.0,
  });

  @override
  Widget build(BuildContext context) {
    switch (layoutInfo.type) {
      case LayoutType.matrix:
        return _buildMatrixLayout(context);
      case LayoutType.standard:
        return _buildStandardLayout(context);
      case LayoutType.split:
        return _buildSplitLayout(context);
    }
  }

  /// Build matrix layout using GridView
  Widget _buildMatrixLayout(BuildContext context) {
    final perRow = layoutInfo.colsPerRow;

    if (perRow != null && perRow.isNotEmpty) {
      return Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          for (int row = 0; row < perRow.length; row++) ...[
            Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                for (int col = 0; col < perRow[row]; col++)
                  Padding(
                    padding: EdgeInsets.all(keySpacing / 2),
                    child: SizedBox(
                      width: keySize,
                      height: keySize,
                      child: _buildKey(context, row, col),
                    ),
                  ),
              ],
            ),
            if (row != perRow.length - 1) SizedBox(height: keySpacing),
          ],
        ],
      );
    }

    return GridView.builder(
      shrinkWrap: true,
      physics: const NeverScrollableScrollPhysics(),
      gridDelegate: SliverGridDelegateWithMaxCrossAxisExtent(
        maxCrossAxisExtent: keySize + keySpacing * 2,
        mainAxisSpacing: keySpacing,
        crossAxisSpacing: keySpacing,
        childAspectRatio: 1.0,
      ),
      itemCount: layoutInfo.rows * layoutInfo.cols,
      itemBuilder: (context, index) {
        final row = index ~/ layoutInfo.cols;
        final col = index % layoutInfo.cols;
        return _buildKey(context, row, col);
      },
    );
  }

  /// Build standard keyboard layout using VisualKeyboard
  Widget _buildStandardLayout(BuildContext context) {
    final layout = KeyboardLayout.ansi104();

    // Map selected position to key ID
    final Set<String> selectedKeys = {};
    if (selectedPosition != null) {
      final key = _getKeyFromPosition(selectedPosition!, layout);
      if (key != null) {
        selectedKeys.add(key.id);
      }
    }

    // Map highlighted positions to key IDs
    final Set<String> highlightedKeys = {};
    for (final pos in highlightedPositions) {
      final key = _getKeyFromPosition(pos, layout);
      if (key != null) {
        highlightedKeys.add(key.id);
      }
    }

    // Map profile mappings to key IDs
    final Set<String> mappedKeys = {};
    if (profile != null) {
      for (final keyString in profile!.mappings.keys) {
        final pos = PhysicalPosition.fromKey(keyString);
        if (pos != null) {
          final key = _getKeyFromPosition(pos, layout);
          if (key != null) {
            mappedKeys.add(key.id);
          }
        }
      }
    }

    return VisualKeyboard(
      layout: layout,
      selectedKeys: selectedKeys,
      highlightedKeys: highlightedKeys,
      mappedKeys: mappedKeys,
      enableDragDrop: false,
      showMappingOverlay: false,
      onKeyTap: (key) {
        final pos = _getPositionFromKey(key, layout);
        if (pos != null && onKeyTap != null) {
          onKeyTap!(pos.row, pos.col);
        }
      },
    );
  }

  KeyDefinition? _getKeyFromPosition(
    PhysicalPosition pos,
    KeyboardLayout layout,
  ) {
    if (pos.row < 0 || pos.row >= layout.rows.length) {
      return null;
    }
    final row = layout.rows[pos.row];
    if (pos.col < 0 || pos.col >= row.keys.length) {
      return null;
    }
    return row.keys[pos.col];
  }

  PhysicalPosition? _getPositionFromKey(
    KeyDefinition key,
    KeyboardLayout layout,
  ) {
    for (int r = 0; r < layout.rows.length; r++) {
      final row = layout.rows[r];
      for (int c = 0; c < row.keys.length; c++) {
        if (row.keys[c].id == key.id) {
          return PhysicalPosition(row: r, col: c);
        }
      }
    }
    return null;
  }

  /// Build split keyboard layout
  /// Shows two independent grids side by side
  Widget _buildSplitLayout(BuildContext context) {
    final halfCols = layoutInfo.cols ~/ 2;

    return Row(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        // Left half
        Expanded(child: _buildSplitHalf(context, 0, 0, halfCols, isLeft: true)),
        SizedBox(width: keySpacing * 4), // Gap between halves
        // Right half
        Expanded(
          child: _buildSplitHalf(
            context,
            0,
            halfCols,
            layoutInfo.cols,
            isLeft: false,
          ),
        ),
      ],
    );
  }

  /// Build one half of a split keyboard
  Widget _buildSplitHalf(
    BuildContext context,
    int startRow,
    int startCol,
    int endCol, {
    required bool isLeft,
  }) {
    final halfCols = endCol - startCol;

    return GridView.builder(
      shrinkWrap: true,
      physics: const NeverScrollableScrollPhysics(),
      gridDelegate: SliverGridDelegateWithMaxCrossAxisExtent(
        maxCrossAxisExtent: keySize + keySpacing * 2,
        mainAxisSpacing: keySpacing,
        crossAxisSpacing: keySpacing,
        childAspectRatio: 1.0,
      ),
      itemCount: layoutInfo.rows * halfCols,
      itemBuilder: (context, index) {
        final row = index ~/ halfCols;
        final col = startCol + (index % halfCols);
        return _buildKey(context, row, col);
      },
    );
  }

  /// Build a single key widget
  Widget _buildKey(BuildContext context, int row, int col) {
    final theme = Theme.of(context);
    final position = PhysicalPosition(row: row, col: col);
    final isSelected = selectedPosition == position;
    final isHighlighted = highlightedPositions.contains(position);

    // Get the mapping for this position
    final action = profile?.getAction(position);
    final hasMapping = action != null;

    // Determine key label
    String label = '$row,$col';
    if (hasMapping) {
      label = _getActionLabel(action);
    }

    return Material(
      color: _getKeyColor(theme, isSelected, hasMapping, isHighlighted),
      borderRadius: BorderRadius.circular(4),
      elevation: isSelected ? 4 : 1,
      child: InkWell(
        onTap: onKeyTap != null ? () => onKeyTap!(row, col) : null,
        borderRadius: BorderRadius.circular(4),
        child: Container(
          padding: const EdgeInsets.all(4),
          child: FittedBox(
            fit: BoxFit.scaleDown,
            alignment: Alignment.center,
            child: Column(
              mainAxisSize: MainAxisSize.min,
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Text(
                  '$row,$col',
                  style: theme.textTheme.bodySmall?.copyWith(
                    fontSize: 10,
                    color: isSelected
                        ? theme.colorScheme.onPrimary.withValues(alpha: 0.7)
                        : theme.colorScheme.onSurface.withValues(alpha: 0.5),
                  ),
                ),
                if (hasMapping) ...[
                  const SizedBox(height: 2),
                  Text(
                    label,
                    style: theme.textTheme.bodyMedium?.copyWith(
                      fontWeight: FontWeight.bold,
                      fontSize: 12,
                      color: isSelected
                          ? theme.colorScheme.onPrimary
                          : theme.colorScheme.onSurface,
                    ),
                    textAlign: TextAlign.center,
                    overflow: TextOverflow.ellipsis,
                    maxLines: 2,
                  ),
                ],
                if (isHighlighted && !isSelected)
                  Padding(
                    padding: const EdgeInsets.only(top: 4),
                    child: Icon(
                      Icons.search,
                      size: 14,
                      color: theme.colorScheme.onTertiaryContainer,
                    ),
                  ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  /// Get the display label for a key action
  String _getActionLabel(KeyAction action) {
    return keyActionLabel(action);
  }

  /// Get the color for a key based on its state
  Color _getKeyColor(
    ThemeData theme,
    bool isSelected,
    bool hasMapping,
    bool isHighlighted,
  ) {
    if (isSelected) {
      return theme.colorScheme.primary;
    }
    if (isHighlighted) {
      return theme.colorScheme.tertiaryContainer;
    }
    if (hasMapping) {
      return theme.colorScheme.secondaryContainer;
    }
    return theme.colorScheme.surfaceContainerHighest;
  }
}

/// Public helper to describe a [KeyAction] for UI labels.
String keyActionLabel(KeyAction action) {
  return action.when(
    key: (key) => key,
    chord: (keys) => keys.join('+'),
    script: (script) => 'Script',
    block: () => 'BLOCK',
    pass: () => 'PASS',
  );
}
