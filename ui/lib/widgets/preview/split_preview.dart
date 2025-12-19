import 'package:core_logic/models/profile.dart';
import 'package:flutter/material.dart';

import 'key_preview.dart';

/// A widget that renders a split layout preview.
class SplitPreview extends StatelessWidget {
  const SplitPreview({
    required this.rows,
    required this.cols,
    super.key,
    this.profile,
    this.onKeyTap,
    this.selectedPosition,
    this.highlightedPositions = const {},
    this.keySize = 48.0,
    this.keySpacing = 4.0,
  });

  /// The number of rows in the layout.
  final int rows;

  /// The total number of columns in the layout.
  final int cols;

  /// The current profile.
  final Profile? profile;

  /// A callback that is invoked when a key is tapped.
  final void Function(int row, int col)? onKeyTap;

  /// The currently selected physical position.
  final PhysicalPosition? selectedPosition;

  /// A set of physical positions to highlight.
  final Set<PhysicalPosition> highlightedPositions;

  /// The size of each key.
  final double keySize;

  /// The spacing between keys.
  final double keySpacing;

  @override
  Widget build(BuildContext context) {
    final halfCols = cols ~/ 2;

    return Row(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        // Left half
        Expanded(child: _buildSplitHalf(context, 0, 0, halfCols, isLeft: true)),
        SizedBox(width: keySpacing * 4), // Gap between halves
        // Right half
        Expanded(
          child: _buildSplitHalf(context, 0, halfCols, cols, isLeft: false),
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
      ),
      itemCount: rows * halfCols,
      itemBuilder: (context, index) {
        final row = index ~/ halfCols;
        final col = startCol + (index % halfCols);
        return _buildKey(context, row, col);
      },
    );
  }

  /// Build a single key widget using KeyPreview
  Widget _buildKey(BuildContext context, int row, int col) {
    final position = PhysicalPosition(row: row, col: col);
    return KeyPreview(
      row: row,
      col: col,
      profile: profile,
      isSelected: selectedPosition == position,
      isHighlighted: highlightedPositions.contains(position),
      onTap: onKeyTap,
    );
  }
}
