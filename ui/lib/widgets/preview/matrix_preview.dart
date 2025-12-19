import 'package:core_logic/models/profile.dart';
import 'package:flutter/material.dart';

import 'key_preview.dart';

/// A widget that renders a matrix layout preview.
class MatrixPreview extends StatelessWidget {
  const MatrixPreview({
    required this.rows,
    required this.cols,
    super.key,
    this.colsPerRow,
    this.profile,
    this.onKeyTap,
    this.selectedPosition,
    this.highlightedPositions = const {},
    this.keySize = 48.0,
    this.keySpacing = 4.0,
  });

  /// The number of rows in the matrix.
  final int rows;

  /// The number of columns in the matrix.
  final int cols;

  /// An optional list specifying the number of columns for each row.
  final List<int>? colsPerRow;

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
    final perRow = colsPerRow;

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
      ),
      itemCount: rows * cols,
      itemBuilder: (context, index) {
        final row = index ~/ cols;
        final col = index % cols;
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
