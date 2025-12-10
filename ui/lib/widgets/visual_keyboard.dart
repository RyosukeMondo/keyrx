/// Visual keyboard widget for the keymap editor.
///
/// Renders an interactive ANSI keyboard layout with proper key sizing,
/// tap handling, drag-and-drop mapping, and visual feedback.
library;

import 'package:flutter/material.dart';

import '../config/config.dart';
import '../models/keyboard_layout.dart';
import 'mapping_overlay.dart';
import 'visual_keyboard_keys.dart';

export 'mapping_overlay.dart' show RemapConfig, MappingType;

/// Interactive visual keyboard widget with drag-and-drop mapping support.
///
/// Renders keys based on a [KeyboardLayout] and supports:
/// - Tap handling for key selection
/// - Drag-and-drop for creating mappings
/// - Visual feedback for mappings via [MappingOverlay]
class VisualKeyboard extends StatefulWidget {
  const VisualKeyboard({
    super.key,
    this.layout,
    this.onKeyTap,
    this.onKeyLongPress,
    this.onMappingCreated,
    this.onMappingDeleted,
    this.mappings = const [],
    this.selectedKeys = const {},
    this.highlightedKeys = const {},
    this.mappedKeys = const {},
    this.heldKeys = const {},
    this.showSecondaryLabels = true,
    this.showMappingOverlay = true,
    this.enableDragDrop = true,
    this.enabled = true,
  });

  /// The keyboard layout to render. Defaults to ANSI 104.
  final KeyboardLayout? layout;

  /// Callback when a key is tapped.
  final void Function(KeyDefinition key)? onKeyTap;

  /// Callback when a key is long-pressed.
  final void Function(KeyDefinition key)? onKeyLongPress;

  /// Callback when a mapping is created via drag-and-drop.
  final void Function(String sourceKeyId, String targetKeyId)? onMappingCreated;

  /// Callback when a mapping is deleted.
  final void Function(int index)? onMappingDeleted;

  /// Current list of mappings to display.
  final List<RemapConfig> mappings;

  /// Set of key IDs that are currently selected.
  final Set<String> selectedKeys;

  /// Set of key IDs that should be highlighted (e.g., combo in progress).
  final Set<String> highlightedKeys;

  /// Set of key IDs that have mappings configured.
  final Set<String> mappedKeys;

  /// Set of key IDs that are currently held down.
  final Set<String> heldKeys;

  /// Whether to show secondary labels (shifted characters).
  final bool showSecondaryLabels;

  /// Whether to show the mapping overlay with arrows.
  final bool showMappingOverlay;

  /// Whether drag-and-drop mapping is enabled.
  final bool enableDragDrop;

  /// Whether the keyboard is interactive.
  final bool enabled;

  @override
  State<VisualKeyboard> createState() => _VisualKeyboardState();
}

class _VisualKeyboardState extends State<VisualKeyboard> {
  String? _hoveredKeyId;
  String? _dragStartKeyId;
  Offset? _dragCurrentPosition;
  int? _selectedMappingIndex;

  KeyboardLayout get _layout => widget.layout ?? KeyboardLayout.ansi104();

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final scale = _calculateScale(constraints);
        final scaledLayout = _createScaledLayout(scale);

        return SingleChildScrollView(
          scrollDirection: Axis.horizontal,
          child: SingleChildScrollView(
            child: SizedBox(
              width: scaledLayout.totalWidth,
              height: scaledLayout.totalHeight,
              child: Stack(
                children: [
                  // Key widgets layer
                  ..._buildKeyWidgets(scaledLayout),
                  // Mapping overlay layer
                  if (widget.showMappingOverlay)
                    Positioned.fill(
                      child: IgnorePointer(
                        ignoring: false,
                        child: MappingOverlay(
                          mappings: widget.mappings,
                          layout: scaledLayout,
                          selectedMappingIndex: _selectedMappingIndex,
                          onMappingTap: (index) {
                            setState(() {
                              _selectedMappingIndex =
                                  _selectedMappingIndex == index ? null : index;
                            });
                          },
                          onMappingDelete: widget.onMappingDeleted,
                          dragStartKey: _dragStartKeyId,
                          dragCurrentPosition: _dragCurrentPosition,
                        ),
                      ),
                    ),
                ],
              ),
            ),
          ),
        );
      },
    );
  }

  double _calculateScale(BoxConstraints constraints) {
    final availableWidth = constraints.maxWidth;
    final layoutWidth = _layout.totalWidth;

    if (layoutWidth <= availableWidth) {
      return 1.0;
    }
    return (availableWidth / layoutWidth).clamp(
      UiConstants.minKeyboardScale,
      UiConstants.maxKeyboardScale,
    );
  }

  KeyboardLayout _createScaledLayout(double scale) {
    if (scale == 1.0) return _layout;

    return KeyboardLayout(
      name: _layout.name,
      rows: _layout.rows,
      unitSize: _layout.unitSize * scale,
      keySpacing: _layout.keySpacing * scale,
    );
  }

  List<Widget> _buildKeyWidgets(KeyboardLayout layout) {
    final widgets = <Widget>[];

    for (final row in layout.rows) {
      for (final key in row.keys) {
        final position = layout.getKeyPosition(key);
        final size = layout.getKeySize(key);
        final isDragTarget =
            _dragStartKeyId != null && _dragStartKeyId != key.id;
        final isDragSource = _dragStartKeyId == key.id;

        final keyWidget = KeyWidget(
          keyDef: key,
          width: size.width,
          height: size.height,
          isSelected: widget.selectedKeys.contains(key.id),
          isHighlighted:
              widget.highlightedKeys.contains(key.id) || isDragTarget,
          isMapped: widget.mappedKeys.contains(key.id),
          isHeld: widget.heldKeys.contains(key.id),
          isHovered: _hoveredKeyId == key.id,
          isDragSource: isDragSource,
          showSecondaryLabel: widget.showSecondaryLabels,
          enabled: widget.enabled,
          onTap: widget.onKeyTap != null ? () => widget.onKeyTap!(key) : null,
          onLongPress: widget.onKeyLongPress != null
              ? () => widget.onKeyLongPress!(key)
              : null,
          onHoverChanged: (hovered) {
            setState(() {
              _hoveredKeyId = hovered ? key.id : null;
            });
          },
        );

        // Wrap with drag-drop if enabled
        final wrappedWidget = widget.enableDragDrop
            ? _wrapWithDragDrop(key, keyWidget, size, layout)
            : keyWidget;

        widgets.add(
          Positioned(left: position.x, top: position.y, child: wrappedWidget),
        );
      }
    }

    return widgets;
  }

  Widget _wrapWithDragDrop(
    KeyDefinition key,
    Widget child,
    ({double width, double height}) size,
    KeyboardLayout layout,
  ) {
    return DragTarget<String>(
      onWillAcceptWithDetails: (details) {
        // Accept if source is different from target
        return details.data != key.id;
      },
      onAcceptWithDetails: (details) {
        // Create mapping
        widget.onMappingCreated?.call(details.data, key.id);
        setState(() {
          _dragStartKeyId = null;
          _dragCurrentPosition = null;
          _selectedMappingIndex = null;
        });
      },
      builder: (context, candidateData, rejectedData) {
        final isAccepting = candidateData.isNotEmpty;

        return LongPressDraggable<String>(
          data: key.id,
          delay: const Duration(milliseconds: 150),
          feedback: Material(
            elevation: 4,
            borderRadius: BorderRadius.circular(6),
            child: Container(
              width: size.width,
              height: size.height,
              decoration: BoxDecoration(
                color: Theme.of(context).colorScheme.primaryContainer,
                borderRadius: BorderRadius.circular(6),
                border: Border.all(
                  color: Theme.of(context).colorScheme.primary,
                  width: 2,
                ),
              ),
              child: Center(
                child: Text(
                  key.label,
                  style: TextStyle(
                    color: Theme.of(context).colorScheme.onPrimaryContainer,
                    fontSize: 14,
                    fontWeight: FontWeight.bold,
                  ),
                ),
              ),
            ),
          ),
          childWhenDragging: Opacity(opacity: 0.4, child: child),
          onDragStarted: () {
            setState(() {
              _dragStartKeyId = key.id;
              _selectedMappingIndex = null;
            });
          },
          onDragUpdate: (details) {
            // Get the RenderBox of this widget to convert global to local
            final renderBox = context.findRenderObject() as RenderBox?;
            if (renderBox != null) {
              setState(() {
                _dragCurrentPosition = renderBox.globalToLocal(
                  details.globalPosition,
                );
              });
            }
          },
          onDraggableCanceled: (_, __) {
            setState(() {
              _dragStartKeyId = null;
              _dragCurrentPosition = null;
            });
          },
          onDragEnd: (_) {
            setState(() {
              _dragStartKeyId = null;
              _dragCurrentPosition = null;
            });
          },
          child: AnimatedContainer(
            duration: const Duration(
              milliseconds: TimingConfig.animationDurationMs,
            ),
            decoration: isAccepting
                ? BoxDecoration(
                    borderRadius: BorderRadius.circular(6),
                    boxShadow: [
                      BoxShadow(
                        color: Theme.of(
                          context,
                        ).colorScheme.primary.withValues(alpha: 0.4),
                        blurRadius: 8,
                        spreadRadius: 2,
                      ),
                    ],
                  )
                : null,
            child: child,
          ),
        );
      },
    );
  }
}

/// A compact visual keyboard for embedding in smaller spaces.
class CompactVisualKeyboard extends StatelessWidget {
  const CompactVisualKeyboard({
    super.key,
    this.onKeyTap,
    this.selectedKeys = const {},
    this.highlightedKeys = const {},
  });

  final void Function(KeyDefinition key)? onKeyTap;
  final Set<String> selectedKeys;
  final Set<String> highlightedKeys;

  @override
  Widget build(BuildContext context) {
    return VisualKeyboard(
      layout: KeyboardLayout.compact(),
      onKeyTap: onKeyTap,
      selectedKeys: selectedKeys,
      highlightedKeys: highlightedKeys,
      showSecondaryLabels: false,
    );
  }
}
