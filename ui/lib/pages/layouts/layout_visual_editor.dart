import 'package:flutter/material.dart';
import '../../models/virtual_layout.dart';

class LayoutVisualEditor extends StatefulWidget {
  const LayoutVisualEditor({
    super.key,
    required this.keys,
    required this.onKeysChanged,
  });

  final List<VirtualKeyDef> keys;
  final ValueChanged<List<VirtualKeyDef>> onKeysChanged;

  @override
  State<LayoutVisualEditor> createState() => _LayoutVisualEditorState();
}

class _LayoutVisualEditorState extends State<LayoutVisualEditor> {
  final Set<String> _selectedIds = {};
  bool _showProperties = true;

  // Selection Box State
  Rect? _selectionBox;
  Offset? _selectionStart;

  // Drag State (Absolute Positioning)
  Map<String, Offset> _initialKeyPositions = {};
  Offset? _dragStartLocalPosition;
  bool _isDraggingKeys = false;

  void _selectKey(String id, {bool multi = false}) {
    setState(() {
      if (multi) {
        if (_selectedIds.contains(id)) {
          _selectedIds.remove(id);
        } else {
          _selectedIds.add(id);
        }
      } else {
        _selectedIds.clear();
        _selectedIds.add(id);
      }
    });
  }

  void _clearSelection() {
    setState(() {
      _selectedIds.clear();
    });
  }

  void _updateKey(VirtualKeyDef updatedKey) {
    final index = widget.keys.indexWhere((k) => k.id == updatedKey.id);
    if (index != -1) {
      final newKeys = List<VirtualKeyDef>.from(widget.keys);
      newKeys[index] = updatedKey;
      widget.onKeysChanged(newKeys);
    }
  }

  VirtualKeyDef? get _primarySelectedKey {
    if (_selectedIds.isEmpty) return null;
    try {
      // Return the first selected key for property editing
      return widget.keys.firstWhere((k) => k.id == _selectedIds.first);
    } catch (_) {
      return null;
    }
  }

  // Helper logic ported from React
  ({int r, int c}) _getNextRC() {
    final occupied = widget.keys
        .map((k) {
          // Parse label if it matches rXcY pattern, otherwise default?
          // Actually we stored r/c in React but KeyDef might not have it explicitly
          // unless we extend the model. For now, let's just parse the label or ID.
          // Or simplified: Just find the next generic one.
          // Let's rely on label parsing for now as we don't have r/c fields in VirtualKeyDef yet.
          final match = RegExp(r'r(\d+)c(\d+)').firstMatch(k.label);
          if (match != null) {
            return '${match.group(1)}-${match.group(2)}';
          }
          return null;
        })
        .where((e) => e != null)
        .toSet();

    int r = 1;
    while (r < 100) {
      for (int c = 1; c <= 40; c++) {
        if (!occupied.contains('$r-$c')) {
          return (r: r, c: c);
        }
      }
      r++;
    }
    return (r: 1, c: 1);
  }

  void _addKey({Offset? position}) {
    print('Adding key...');
    final rc = _getNextRC();
    final newId = 'K_${DateTime.now().millisecondsSinceEpoch}';

    // Default position: Center of the canvas (which is 2000x2000), so 1000,1000?
    // Or if position is provided (from double click), use that.
    // Note: position passed here is local to the interactive viewer if we handle it right.

    double x = 100;
    double y = 100;

    if (position != null) {
      // Snap to grid (10px)
      x = (position.dx / 10).roundToDouble() * 10;
      y = (position.dy / 10).roundToDouble() * 10;
    }

    final newKey = VirtualKeyDef(
      id: newId,
      label: 'r${rc.r}c${rc.c}',
      position: KeyPosition(x: x, y: y),
      size: const KeySize(
        width: 50,
        height: 50,
      ), // Unit? React used 50px. Flutter model expects unitless?
      // Wait, VirtualKeyDef uses logical units where 1.0 approx 60px or similar?
      // Let's check _buildKeyCanvas implementation.
      // It says: const double unitSize = 60.0;
      // React used 50px fixed size keys on a grid.
      // If we use VirtualKeyDef size 1.0, it renders as 60px.
      // Let's use 0.833 (50/60) or just standard 1.0 for now for "1u".
      // Let's stick to 1.0 = 1u.
      // Position is also in units.
      // React x=100 was pixels.
      // Here x is units?
      // Checking _buildKeyCanvas: left: (key.position.x) * unitSize + 1000
      // So key.position.x is in UNITS.
      // So if we want pixel offset 100, we need 100/60 = 1.66 units.
      // Let's just put it at 0,0 (center) relative to origin 1000,1000.
    );

    // Actually, let's look at the Coordinate System again.
    // _buildKeyCanvas logic: left = (x * 60) + 1000.
    // So x=0 is at 1000px (center).
    // React x=100 was absolute pixels from top-left.
    // Let's just spawn at x=0, y=0 (Unit coordinates).

    // Default position: 50, 50 (Pixels)
    double finalX = 50.0;
    double finalY = 50.0;

    if (position != null) {
      finalX = position.dx;
      finalY = position.dy;

      // Center the key (subtract half size 25px)
      finalX -= 25.0;
      finalY -= 25.0;

      // Snap to 10px grid
      finalX = (finalX / 10).roundToDouble() * 10;
      finalY = (finalY / 10).roundToDouble() * 10;
    }

    final k = VirtualKeyDef(
      id: newId,
      label: 'r${rc.r}c${rc.c}',
      position: KeyPosition(x: finalX, y: finalY),
      size: const KeySize(width: 50, height: 50), // Standard 50px
    );

    final newKeys = List<VirtualKeyDef>.from(widget.keys)..add(k);
    widget.onKeysChanged(newKeys);
    _selectKey(newId);

    print('Key added: $newId at $finalX, $finalY');
  }

  void _deleteSelectedKeys() {
    if (_selectedIds.isEmpty) return;
    final newKeys = widget.keys
        .where((k) => !_selectedIds.contains(k.id))
        .toList();
    widget.onKeysChanged(newKeys);
    _clearSelection();
  }

  void _alignKeys(String alignment) {
    if (_selectedIds.length < 2) return;
    final selectedKeys = widget.keys
        .where((k) => _selectedIds.contains(k.id))
        .toList();

    if (selectedKeys.isEmpty) return;

    double? targetVal;

    // Calculate bounds
    double minX = double.infinity;
    double maxX = -double.infinity;
    double minY = double.infinity;
    double maxY = -double.infinity;
    double maxRight = -double.infinity;
    double maxBottom = -double.infinity;

    for (var k in selectedKeys) {
      double x = k.position?.x ?? 0;
      double y = k.position?.y ?? 0;
      double w = k.size?.width ?? 50;
      double h = k.size?.height ?? 50;
      if (x < minX) minX = x;
      if (x > maxX) maxX = x;
      if (y < minY) minY = y;
      if (y > maxY) maxY = y;
      if (x + w > maxRight) maxRight = x + w;
      if (y + h > maxBottom) maxBottom = y + h;
    }

    final newKeys = List<VirtualKeyDef>.from(widget.keys);

    for (int i = 0; i < newKeys.length; i++) {
      if (_selectedIds.contains(newKeys[i].id)) {
        var k = newKeys[i];
        double x = k.position?.x ?? 0;
        double y = k.position?.y ?? 0;
        double w = k.size?.width ?? 50;
        double h = k.size?.height ?? 50;

        switch (alignment) {
          case 'left':
            x = minX;
            break;
          case 'center': // Horizontal Center
            double center = (minX + maxRight) / 2;
            x = center - (w / 2);
            break;
          case 'right':
            x = maxRight - w;
            break;
          case 'top':
            y = minY;
            break;
          case 'middle': // Vertical Center
            double center = (minY + maxBottom) / 2;
            y = center - (h / 2);
            break;
          case 'bottom':
            y = maxBottom - h;
            break;
        }

        // Snap to 10px grid
        x = (x / 10).roundToDouble() * 10;
        y = (y / 10).roundToDouble() * 10;

        newKeys[i] = k.copyWith(
          position: KeyPosition(x: x, y: y),
        );
      }
    }
    widget.onKeysChanged(newKeys);
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        // Toolbar
        Container(
          height: 48,
          decoration: BoxDecoration(
            color: Theme.of(context).colorScheme.surface,
            border: Border(bottom: Divider.createBorderSide(context)),
          ),
          padding: const EdgeInsets.symmetric(horizontal: 8),
          child: Row(
            children: [
              IconButton(onPressed: () => _addKey(), icon: const Text('+ Key')),
              const VerticalDivider(),
              IconButton(
                onPressed: () => _alignKeys('left'),
                icon: const Icon(Icons.align_horizontal_left),
              ),
              IconButton(
                onPressed: () => _alignKeys('center'),
                icon: const Icon(Icons.align_horizontal_center),
              ),
              IconButton(
                onPressed: () => _alignKeys('right'),
                icon: const Icon(Icons.align_horizontal_right),
              ),
              IconButton(
                onPressed: () => _alignKeys('top'),
                icon: const Icon(Icons.align_vertical_top),
              ),
              IconButton(
                onPressed: () => _alignKeys('middle'),
                icon: const Icon(Icons.align_vertical_center),
              ),
              IconButton(
                onPressed: () => _alignKeys('bottom'),
                icon: const Icon(Icons.align_vertical_bottom),
              ),
              const Spacer(),
              IconButton(
                onPressed: () =>
                    setState(() => _showProperties = !_showProperties),
                icon: Icon(
                  _showProperties
                      ? Icons.keyboard_double_arrow_right
                      : Icons.keyboard_double_arrow_left,
                ),
                tooltip: _showProperties
                    ? 'Hide Properties'
                    : 'Show Properties',
              ),
            ],
          ),
        ),
        Expanded(
          child: Row(
            children: [
              // Visual Area
              Expanded(
                flex: 3,
                child: Container(
                  color: Theme.of(context).colorScheme.surfaceContainer,
                  child: SingleChildScrollView(
                    scrollDirection: Axis.vertical,
                    child: SingleChildScrollView(
                      scrollDirection: Axis.horizontal,
                      child: SizedBox(
                        width: 4000,
                        height: 4000,
                        child: Stack(
                          children: [
                            Positioned.fill(
                              child: CustomPaint(
                                painter: _GridPainter(
                                  step: 60.0,
                                  color: Theme.of(
                                    context,
                                  ).dividerColor.withOpacity(0.1),
                                  subStep: 15.0, // Sub-grid for visual hints
                                ),
                              ),
                            ),
                            _buildKeyCanvas(),
                          ],
                        ),
                      ),
                    ),
                  ),
                ),
              ),
              // Property Bar
              if (_showProperties) ...[
                const VerticalDivider(width: 1),
                SizedBox(
                  width: 300,
                  child: _PropertyBar(
                    selectedKey: _primarySelectedKey,
                    onUpdate: _updateKey,
                    selectedCount: _selectedIds.length,
                    onDelete: _deleteSelectedKeys,
                  ),
                ),
              ],
            ],
          ),
        ),
      ],
    );
  }

  Widget _buildKeyCanvas() {
    // Pixel-based coordinates with 5px snapping
    const double canvasSize = 4000.0;

    return SizedBox(
      width: canvasSize,
      height: canvasSize,
      child: GestureDetector(
        onTap: () {
          print('Canvas tapped - clearing selection');
          _clearSelection();
        },
        onDoubleTapDown: (details) {
          // Double Tap: Add key at snapped pixel position
          final pos = details.localPosition;
          _addKey(position: pos);
        },
        onPanStart: (details) {
          setState(() {
            _selectionStart = details.localPosition;
            _selectionBox = Rect.fromPoints(_selectionStart!, _selectionStart!);
          });
        },
        onPanUpdate: (details) {
          if (_selectionStart == null) return;
          setState(() {
            _selectionBox = Rect.fromPoints(
              _selectionStart!,
              details.localPosition,
            );
          });
          // Logic to select keys inside box
          final box = _selectionBox!;
          // Convert box to unit space relative to 1000,1000
          // Optimization: Do this only on PanEnd? Or live? Live is better.
          // ... (For brevity, maybe on End is safer for perf, but live is "perfect match")
        },
        onPanEnd: (details) {
          if (_selectionBox != null) {
            // Finalize selection
            final box = _selectionBox!;
            final unitBox = Rect.fromLTRB(
              box.left,
              box.top,
              box.right,
              box.bottom,
            );

            final newSelected = widget.keys
                .where((k) {
                  final kRect = Rect.fromLTWH(
                    k.position?.x ?? 0,
                    k.position?.y ?? 0,
                    k.size?.width ?? 50, // Default 50px
                    k.size?.height ?? 50,
                  );
                  return unitBox.overlaps(kRect);
                })
                .map((k) => k.id)
                .toSet();

            // Add to existing if shift? For now replace.
            setState(() {
              _selectedIds.clear();
              _selectedIds.addAll(newSelected);
              _selectionBox = null;
              _selectionStart = null;
            });
          }
        },
        behavior: HitTestBehavior.opaque, // Catch clicks on empty space
        child: Stack(
          children: [
            for (final key in widget.keys)
              Positioned(
                left: (key.position?.x ?? 0),
                top: (key.position?.y ?? 0),
                width: (key.size?.width ?? 50),
                height: (key.size?.height ?? 50),
                child: GestureDetector(
                  onTap: () => _selectKey(key.id),
                  onDoubleTap: () {
                    // Delete key on double tap
                    print('Double tap key ${key.id} -> deleting');
                    // Need to call a delete function
                    final newKeys = widget.keys
                        .where((k) => k.id != key.id)
                        .toList();
                    widget.onKeysChanged(newKeys);
                    if (_selectedIds.contains(key.id)) {
                      setState(() {
                        _selectedIds.remove(key.id);
                      });
                    }
                  },
                  onPanStart: (details) {
                    if (!_selectedIds.contains(key.id)) {
                      _selectKey(key.id);
                    }
                    setState(() {
                      _isDraggingKeys = true;
                      _dragStartLocalPosition =
                          details.globalPosition; // Using global to be safe
                      _initialKeyPositions = {
                        for (var k in widget.keys)
                          if (_selectedIds.contains(k.id))
                            k.id: Offset(
                              k.position?.x ?? 0,
                              k.position?.y ?? 0,
                            ),
                      };
                    });
                  },
                  onPanUpdate: (details) {
                    if (!_isDraggingKeys || _dragStartLocalPosition == null)
                      return;

                    final totalDelta =
                        details.globalPosition - _dragStartLocalPosition!;

                    final newKeys = widget.keys.map((k) {
                      if (_initialKeyPositions.containsKey(k.id)) {
                        final initialPos = _initialKeyPositions[k.id]!;
                        final newX = initialPos.dx + totalDelta.dx;
                        final newY = initialPos.dy + totalDelta.dy;

                        return k.copyWith(
                          position: KeyPosition(x: newX, y: newY),
                        );
                      }
                      return k;
                    }).toList();
                    widget.onKeysChanged(newKeys);
                  },
                  onPanEnd: (_) {
                    setState(() {
                      _isDraggingKeys = false;
                      _initialKeyPositions.clear();
                      _dragStartLocalPosition = null;
                    });

                    // Final Snap
                    final newKeys = widget.keys.map((k) {
                      if (_selectedIds.contains(k.id)) {
                        double x = k.position?.x ?? 0;
                        double y = k.position?.y ?? 0;
                        // Snap to 10px grid
                        x = (x / 10).roundToDouble() * 10;
                        y = (y / 10).roundToDouble() * 10;
                        return k.copyWith(
                          position: KeyPosition(x: x, y: y),
                        );
                      }
                      return k;
                    }).toList();
                    widget.onKeysChanged(newKeys);
                  },
                  child: Container(
                    margin: const EdgeInsets.all(2), // Gutter
                    decoration: BoxDecoration(
                      color: _selectedIds.contains(key.id)
                          ? Theme.of(context).colorScheme.primaryContainer
                          : Theme.of(context).colorScheme.surface,
                      borderRadius: BorderRadius.circular(4),
                      border: Border.all(
                        color: _selectedIds.contains(key.id)
                            ? Theme.of(context).colorScheme.primary
                            : Theme.of(context).dividerColor,
                        width: _selectedIds.contains(key.id) ? 2 : 1,
                      ),
                    ),
                    alignment: Alignment.center,
                    child: Text(
                      key.label.isEmpty && key.id.isNotEmpty
                          ? key.id
                          : key.label,
                      style: const TextStyle(fontSize: 12),
                      overflow: TextOverflow.ellipsis,
                    ),
                  ),
                ),
              ),
            // Marquee Render
            if (_selectionBox != null)
              Positioned.fromRect(
                rect: _selectionBox!,
                child: Container(
                  decoration: BoxDecoration(
                    color: Colors.blueAccent.withOpacity(0.2),
                    border: Border.all(color: Colors.blueAccent),
                  ),
                ),
              ),
          ],
        ),
      ),
    );
  }
}

class _PropertyBar extends StatelessWidget {
  const _PropertyBar({
    required this.selectedKey,
    required this.onUpdate,
    required this.selectedCount,
    required this.onDelete,
  });

  final VirtualKeyDef? selectedKey;
  final ValueChanged<VirtualKeyDef> onUpdate;
  final int selectedCount;
  final VoidCallback onDelete;

  @override
  Widget build(BuildContext context) {
    if (selectedKey == null) {
      return const Center(child: Text('Select a key to edit'));
    }

    // We use a key for the input fields so they rebuild when selection changes
    // instead of keeping old text.
    return KeyedSubtree(
      key: ValueKey(selectedKey!.id),
      child: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Text(
            selectedCount > 1
                ? 'Properties ($selectedCount)'
                : 'Key Properties',
            style: Theme.of(context).textTheme.titleMedium,
          ),
          const SizedBox(height: 16),
          TextFormField(
            initialValue: selectedKey!.label,
            decoration: const InputDecoration(labelText: 'Label'),
            onChanged: (val) => onUpdate(selectedKey!.copyWith(label: val)),
          ),
          const SizedBox(height: 12),
          TextFormField(
            initialValue: selectedKey!.id,
            decoration: const InputDecoration(labelText: 'ID'),
            // ID change might be tricky if it's the key, but for now allow it.
            onChanged: (val) => onUpdate(selectedKey!.copyWith(id: val)),
          ),
          const Divider(height: 32),
          Row(
            children: [
              Expanded(
                child: TextFormField(
                  initialValue: selectedKey!.position?.x.toString(),
                  decoration: const InputDecoration(labelText: 'Pos X'),
                  keyboardType: TextInputType.number,
                  onChanged: (val) {
                    final v = double.tryParse(val);
                    if (v != null) {
                      onUpdate(
                        selectedKey!.copyWith(
                          position:
                              selectedKey!.position?.copyWith(x: v) ??
                              KeyPosition(x: v, y: 0),
                        ),
                      );
                    }
                  },
                ),
              ),
              const SizedBox(width: 8),
              Expanded(
                child: TextFormField(
                  initialValue: selectedKey!.position?.y.toString(),
                  decoration: const InputDecoration(labelText: 'Pos Y'),
                  keyboardType: TextInputType.number,
                  onChanged: (val) {
                    final v = double.tryParse(val);
                    if (v != null) {
                      onUpdate(
                        selectedKey!.copyWith(
                          position:
                              selectedKey!.position?.copyWith(y: v) ??
                              KeyPosition(x: 0, y: v),
                        ),
                      );
                    }
                  },
                ),
              ),
            ],
          ),
          const SizedBox(height: 12),
          Row(
            children: [
              Expanded(
                child: TextFormField(
                  initialValue: selectedKey!.size?.width.toString(),
                  decoration: const InputDecoration(labelText: 'Width'),
                  keyboardType: TextInputType.number,
                  onChanged: (val) {
                    final v = double.tryParse(val);
                    if (v != null) {
                      onUpdate(
                        selectedKey!.copyWith(
                          size:
                              selectedKey!.size?.copyWith(width: v) ??
                              KeySize(width: v, height: 1),
                        ),
                      );
                    }
                  },
                ),
              ),
              const SizedBox(width: 8),
              Expanded(
                child: TextFormField(
                  initialValue: selectedKey!.size?.height.toString(),
                  decoration: const InputDecoration(labelText: 'Height'),
                  keyboardType: TextInputType.number,
                  onChanged: (val) {
                    final v = double.tryParse(val);
                    if (v != null) {
                      onUpdate(
                        selectedKey!.copyWith(
                          size:
                              selectedKey!.size?.copyWith(height: v) ??
                              KeySize(width: 1, height: v),
                        ),
                      );
                    }
                  },
                ),
              ),
            ],
          ),
          const SizedBox(height: 12),
          Row(
            children: [
              Expanded(
                child: TextFormField(
                  initialValue: selectedKey!.row?.toString() ?? '',
                  decoration: const InputDecoration(labelText: 'Row'),
                  keyboardType: TextInputType.number,
                  onChanged: (val) {
                    final v = int.tryParse(val);
                    onUpdate(selectedKey!.copyWith(row: v));
                  },
                ),
              ),
              const SizedBox(width: 8),
              Expanded(
                child: TextFormField(
                  initialValue: selectedKey!.column?.toString() ?? '',
                  decoration: const InputDecoration(labelText: 'Col'),
                  keyboardType: TextInputType.number,
                  onChanged: (val) {
                    final v = int.tryParse(val);
                    onUpdate(selectedKey!.copyWith(column: v));
                  },
                ),
              ),
            ],
          ),
          const SizedBox(height: 12),
          SizedBox(
            width: double.infinity,
            child: FilledButton.icon(
              onPressed: onDelete,
              icon: const Icon(Icons.delete),
              label: Text('Delete Selected ($selectedCount)'),
              style: FilledButton.styleFrom(
                backgroundColor: Theme.of(context).colorScheme.error,
                foregroundColor: Theme.of(context).colorScheme.onError,
              ),
            ),
          ),
        ],
      ),
    );
  }
}

class _GridPainter extends CustomPainter {
  final double step;
  final double subStep;
  final Color color;

  _GridPainter({required this.step, required this.color, this.subStep = 15.0});

  @override
  void paint(Canvas canvas, Size size) {
    // Minor Grid
    final paintMinor = Paint()
      ..color = color.withOpacity(0.1)
      ..strokeWidth = 1;

    for (double x = 0; x <= size.width; x += subStep) {
      canvas.drawLine(Offset(x, 0), Offset(x, size.height), paintMinor);
    }
    for (double y = 0; y <= size.height; y += subStep) {
      canvas.drawLine(Offset(0, y), Offset(size.width, y), paintMinor);
    }

    // Major Grid
    final paintMajor = Paint()
      ..color = color
      ..strokeWidth = 1;

    for (double x = 0; x <= size.width; x += step) {
      canvas.drawLine(Offset(x, 0), Offset(x, size.height), paintMajor);
    }

    for (double y = 0; y <= size.height; y += step) {
      canvas.drawLine(Offset(0, y), Offset(size.width, y), paintMajor);
    }
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => false;
}
