// Layout grid widget for displaying device layouts dynamically.
//
// Renders Matrix, Standard, and Split keyboard layouts with current
// mappings displayed on keys. Supports interactive key selection for
// mapping configuration.

import 'package:core_logic/models/keyboard_layout.dart';
import 'package:core_logic/models/layout_type.dart';
import 'package:core_logic/models/profile.dart';
import 'package:flutter/material.dart';

import 'preview/preview.dart';
import 'visual_keyboard.dart';

/// Layout information for rendering grids
class LayoutInfo {
  const LayoutInfo({
    required this.rows,
    required this.cols,
    required this.type,
    this.colsPerRow,
  });
  final int rows;
  final int cols;
  final LayoutType type;
  final List<int>? colsPerRow;
}

/// Widget that dynamically renders device layouts as interactive grids.
///
/// Supports Matrix (macro pads), Standard (keyboards), and Split layouts.
/// Shows current key mappings and allows key selection for remapping.
class LayoutGrid extends StatelessWidget {
  const LayoutGrid({
    required this.layoutInfo,
    super.key,
    this.profile,
    this.onKeyTap,
    this.selectedPosition,
    this.highlightedPositions = const {},
    this.keySize = 48.0,
    this.keySpacing = 4.0,
  });

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

  @override
  Widget build(BuildContext context) {
    switch (layoutInfo.type) {
      case LayoutType.matrix:
        return MatrixPreview(
          rows: layoutInfo.rows,
          cols: layoutInfo.cols,
          colsPerRow: layoutInfo.colsPerRow,
          profile: profile,
          onKeyTap: onKeyTap,
          selectedPosition: selectedPosition,
          highlightedPositions: highlightedPositions,
          keySize: keySize,
          keySpacing: keySpacing,
        );
      case LayoutType.standard:
        return _buildStandardLayout(context);
      case LayoutType.split:
        return SplitPreview(
          rows: layoutInfo.rows,
          cols: layoutInfo.cols,
          profile: profile,
          onKeyTap: onKeyTap,
          selectedPosition: selectedPosition,
          highlightedPositions: highlightedPositions,
          keySize: keySize,
          keySpacing: keySpacing,
        );
    }
  }

  /// Build standard keyboard layout using VisualKeyboard
  Widget _buildStandardLayout(BuildContext context) {
    final layout = KeyboardLayout.ansi104();

    // Map selected position to key ID
    final selectedKeys = <String>{};
    if (selectedPosition != null) {
      final key = _getKeyFromPosition(selectedPosition!, layout);
      if (key != null) {
        selectedKeys.add(key.id);
      }
    }

    // Map highlighted positions to key IDs
    final highlightedKeys = <String>{};
    for (final pos in highlightedPositions) {
      final key = _getKeyFromPosition(pos, layout);
      if (key != null) {
        highlightedKeys.add(key.id);
      }
    }

    // Map profile mappings to key IDs
    final mappedKeys = <String>{};
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
    for (var r = 0; r < layout.rows.length; r++) {
      final row = layout.rows[r];
      for (var c = 0; c < row.keys.length; c++) {
        if (row.keys[c].id == key.id) {
          return PhysicalPosition(row: r, col: c);
        }
      }
    }
    return null;
  }
}
