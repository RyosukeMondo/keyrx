import 'package:core_logic/models/profile.dart';
import 'package:flutter/material.dart';

/// A widget that renders a preview of a single key.
///
/// Used by matrix and split layout previews.
class KeyPreview extends StatelessWidget {
  const KeyPreview({
    required this.row,
    required this.col,
    super.key,
    this.profile,
    this.isSelected = false,
    this.isHighlighted = false,
    this.onTap,
  });

  /// The row of the key.
  final int row;

  /// The column of the key.
  final int col;

  /// The current profile.
  final Profile? profile;

  /// Whether the key is selected.
  final bool isSelected;

  /// Whether the key is highlighted.
  final bool isHighlighted;

  /// A callback that is invoked when the key is tapped.
  final void Function(int row, int col)? onTap;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final position = PhysicalPosition(row: row, col: col);

    // Get the mapping for this position
    final action = profile?.getAction(position);
    final hasMapping = action != null;

    // Determine key label
    var label = '$row,$col';
    if (hasMapping) {
      label = _getActionLabel(action);
    }

    return Material(
      color: _getKeyColor(theme, isSelected, hasMapping, isHighlighted),
      borderRadius: BorderRadius.circular(4),
      elevation: isSelected ? 4 : 1,
      child: InkWell(
        onTap: onTap != null ? () => onTap!(row, col) : null,
        borderRadius: BorderRadius.circular(4),
        child: Container(
          padding: const EdgeInsets.all(4),
          child: FittedBox(
            fit: BoxFit.scaleDown,
            child: Column(
              mainAxisSize: MainAxisSize.min,
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Text(
                  '$row,$col',
                  style: theme.textTheme.bodySmall?.copyWith(
                    fontSize: 10,
                    color: isSelected
                        ? theme.colorScheme.onPrimary.withAlpha(180)
                        : theme.colorScheme.onSurface.withAlpha(128),
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
  String _getActionLabel(KeyAction action) => keyActionLabel(action);

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
String keyActionLabel(KeyAction action) => action.when(
  key: (key) => key,
  chord: (keys) => keys.join('+'),
  script: (script) => 'Script',
  block: () => 'BLOCK',
  pass: () => 'PASS',
);
