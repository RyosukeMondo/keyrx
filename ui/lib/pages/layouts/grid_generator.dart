import 'package:flutter/material.dart';
import '../../models/virtual_layout.dart';

class GridGenerator extends StatefulWidget {
  const GridGenerator({super.key, required this.onGenerate});

  final ValueChanged<List<VirtualKeyDef>> onGenerate;

  @override
  State<GridGenerator> createState() => _GridGeneratorState();
}

class _GridGeneratorState extends State<GridGenerator> {
  static const int maxRows = 6;
  static const int maxCols = 20;

  int _hoverRows = 0;
  int _hoverCols = 0;

  void _handleHover(int row, int col) {
    if (_hoverRows != row || _hoverCols != col) {
      if (!mounted) return;
      setState(() {
        _hoverRows = row;
        _hoverCols = col;
      });
    }
  }

  void _handleTap() {
    if (_hoverRows == 0 || _hoverCols == 0) return;

    final List<VirtualKeyDef> keys = [];

    const double keySize = 50.0;
    const double gap = 10.0;
    const double step = keySize + gap;

    // Start at 50, 50 to match the visible area ("+ Key" default)
    final double startX = 50.0;
    final double startY = 50.0;

    for (int r = 0; r < _hoverRows; r++) {
      for (int c = 0; c < _hoverCols; c++) {
        // 1-based indexing for user facing labels/IDs
        final rowIdx = r + 1;
        final colIdx = c + 1;
        final id = 'r${rowIdx}c${colIdx}';

        keys.add(
          VirtualKeyDef(
            id: id,
            label: id,
            row: rowIdx,
            column: colIdx,
            position: KeyPosition(
              x: startX + (c * step),
              y: startY + (r * step),
            ),
            size: const KeySize(width: keySize, height: keySize),
          ),
        );
      }
    }
    widget.onGenerate(keys);
  }

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        // Calculate box size to fit both width and height
        // Available width for maxCols
        final widthBasedSize = (constraints.maxWidth / maxCols);

        // Available height for maxRows + label space (approx 2 rows equivalent)
        // We subtract padding or text height roughly
        final heightBasedSize = (constraints.maxHeight / (maxRows + 2));

        // Use the smaller of the two to ensure it fits, clamped to reasonable limits
        final double boxSize =
            (widthBasedSize < heightBasedSize
                    ? widthBasedSize
                    : heightBasedSize)
                .clamp(10.0, 30.0);

        // Center the grid content
        return Center(
          child: FittedBox(
            fit: BoxFit.scaleDown,
            child: MouseRegion(
              onExit: (_) => setState(() => _hoverRows = _hoverCols = 0),
              child: GestureDetector(
                onTap: _handleTap,
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    Text(
                      _hoverRows > 0
                          ? '${_hoverRows}x$_hoverCols'
                          : 'Select Size',
                      style: Theme.of(context).textTheme.labelMedium?.copyWith(
                        color: Theme.of(context).colorScheme.onSurfaceVariant,
                      ),
                    ),
                    const SizedBox(height: 8),
                    Row(
                      mainAxisSize: MainAxisSize.min,
                      children: List.generate(maxCols, (c) {
                        return Column(
                          mainAxisSize: MainAxisSize.min,
                          children: List.generate(maxRows, (r) {
                            final isSelected = r < _hoverRows && c < _hoverCols;
                            return MouseRegion(
                              onEnter: (_) => _handleHover(r + 1, c + 1),
                              child: AnimatedContainer(
                                duration: const Duration(milliseconds: 50),
                                width: boxSize,
                                height: boxSize,
                                margin: const EdgeInsets.all(1),
                                decoration: BoxDecoration(
                                  color: isSelected
                                      ? Theme.of(context).colorScheme.primary
                                      : Theme.of(
                                          context,
                                        ).colorScheme.surfaceContainerHighest,
                                  border: Border.all(
                                    color: isSelected
                                        ? Theme.of(context).colorScheme.primary
                                        : Theme.of(
                                            context,
                                          ).colorScheme.outlineVariant,
                                    width: 1,
                                  ),
                                  borderRadius: BorderRadius.circular(2),
                                ),
                              ),
                            );
                          }),
                        );
                      }),
                    ),
                  ],
                ),
              ),
            ),
          ),
        );
      },
    );
  }
}
