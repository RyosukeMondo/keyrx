// Layout grid widget for displaying device layouts dynamically.
//
// Renders Matrix, Standard, and Split keyboard layouts with current
// mappings displayed on keys. Supports interactive key selection for
// mapping configuration.

import 'package:flutter/material.dart';
import '../models/layout_type.dart';
import '../models/profile.dart';

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

  /// Build standard keyboard layout
  /// For now, uses a simplified grid approach
  /// TODO: Implement proper ANSI/ISO keyboard layout with correct key sizes
  Widget _buildStandardLayout(BuildContext context) {
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Icon(Icons.keyboard, size: 48),
          const SizedBox(height: 16),
          Text(
            'Standard keyboard layout',
            style: Theme.of(context).textTheme.titleMedium,
          ),
          const SizedBox(height: 8),
          Text(
            'Visual representation coming soon',
            style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                  color: Theme.of(context).colorScheme.onSurface.withOpacity(0.6),
                ),
          ),
          const SizedBox(height: 16),
          // Show mapped keys as a simple list for now
          if (profile != null && profile!.hasMapping) ...[
            const Divider(),
            const SizedBox(height: 8),
            Text(
              'Mapped keys: ${profile!.mappingCount}',
              style: Theme.of(context).textTheme.bodySmall,
            ),
          ],
        ],
      ),
    );
  }

  /// Build split keyboard layout
  /// Shows two independent grids side by side
  Widget _buildSplitLayout(BuildContext context) {
    final halfCols = layoutInfo.cols ~/ 2;

    return Row(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        // Left half
        Expanded(
          child: _buildSplitHalf(context, 0, 0, halfCols, isLeft: true),
        ),
        SizedBox(width: keySpacing * 4), // Gap between halves
        // Right half
        Expanded(
          child: _buildSplitHalf(context, 0, halfCols, layoutInfo.cols, isLeft: false),
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

    // Get the mapping for this position
    final action = profile?.getAction(position);
    final hasMapping = action != null;

    // Determine key label
    String label = '$row,$col';
    if (hasMapping) {
      label = _getActionLabel(action);
    }

    return Material(
      color: _getKeyColor(theme, isSelected, hasMapping),
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
                        ? theme.colorScheme.onPrimary.withOpacity(0.7)
                        : theme.colorScheme.onSurface.withOpacity(0.5),
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
              ],
            ),
          ),
        ),
      ),
    );
  }

  /// Get the display label for a key action
  String _getActionLabel(KeyAction action) {
    return action.when(
      key: (key) => key,
      chord: (keys) => keys.join('+'),
      script: (script) => 'Script',
      block: () => 'BLOCK',
      pass: () => 'PASS',
    );
  }

  /// Get the color for a key based on its state
  Color _getKeyColor(ThemeData theme, bool isSelected, bool hasMapping) {
    if (isSelected) {
      return theme.colorScheme.primary;
    }
    if (hasMapping) {
      return theme.colorScheme.secondaryContainer;
    }
    return theme.colorScheme.surfaceContainerHighest;
  }
}
