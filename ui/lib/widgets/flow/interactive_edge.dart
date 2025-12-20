/// Interactive edge widget with click detection, selection, and context menu.
library;

import 'dart:async';

import 'package:core_logic/models/flow/edge_style.dart';
import 'package:core_logic/models/flow/flow_graph.dart';
import 'package:core_logic/models/hardware_profile.dart';
import 'package:core_logic/models/runtime_config.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../animations/edge_animations.dart';
import '../../state/selection_state.dart';
import '../../state/slot_actions.dart';
import '../../utils/core_ui_extensions.dart';
import 'animated_edges_painter.dart';
import 'edges_painter.dart';

/// Interactive wrapper for EdgesPainter with selection and context menu.
class InteractiveEdgeCanvas extends ConsumerStatefulWidget {
  const InteractiveEdgeCanvas({
    required this.edges,
    required this.paintableEdges,
    required this.style,
    super.key,
  });

  final List<FlowEdge> edges;
  final List<PaintableEdge> paintableEdges;
  final EdgeStyle style;

  @override
  ConsumerState<InteractiveEdgeCanvas> createState() => _InteractiveEdgeCanvasState();
}

class _InteractiveEdgeCanvasState extends ConsumerState<InteractiveEdgeCanvas>
    with TickerProviderStateMixin {
  Offset? _hoverPosition;
  late final AnimationController _pulseController;
  final Map<String, EdgeAnimationState> _animationStates = {};
  final Map<String, Color> _previousColors = {};
  final Map<String, AnimationController> _transitionControllers = {};

  @override
  void initState() {
    super.initState();

    // Initialize pulse controller for continuous animation
    _pulseController = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 2000),
    )..repeat();

    // Update pulse phase on each tick
    _pulseController.addListener(_updatePulsePhase);

    // Initialize states for all edges
    _initializeEdgeStates();
  }

  @override
  void didUpdateWidget(InteractiveEdgeCanvas oldWidget) {
    super.didUpdateWidget(oldWidget);

    // Handle edge additions/removals with fade animations
    if (oldWidget.paintableEdges.length != widget.paintableEdges.length ||
        !_edgesEqual(oldWidget.paintableEdges, widget.paintableEdges)) {
      _handleEdgeChanges(oldWidget.paintableEdges, widget.paintableEdges);
    }

    // Handle validation state color transitions
    _handleColorTransitions(oldWidget.paintableEdges, widget.paintableEdges);
  }

  @override
  void dispose() {
    _pulseController.dispose();

    for (final controller in _transitionControllers.values) {
      controller.dispose();
    }
    _transitionControllers.clear();

    super.dispose();
  }

  void _initializeEdgeStates() {
    for (final edge in widget.paintableEdges) {
      final edgeId = _edgeId(edge);
      _animationStates[edgeId] = EdgeAnimationState();
    }
  }

  void _updatePulsePhase() {
    final phase = _pulseController.value;
    var needsRepaint = false;

    for (final edge in widget.paintableEdges) {
      if (edge.active) {
        final edgeId = _edgeId(edge);
        final current = _animationStates[edgeId];
        if (current != null) {
          _animationStates[edgeId] = current.copyWith(pulsePhase: phase);
          needsRepaint = true;
        }
      }
    }

    if (needsRepaint) {
      setState(() {});
    }
  }

  void _handleEdgeChanges(List<PaintableEdge> oldEdges, List<PaintableEdge> newEdges) {
    final oldIds = oldEdges.map(_edgeId).toSet();
    final newIds = newEdges.map(_edgeId).toSet();

    // Handle new edges (fade in)
    for (final edge in newEdges) {
      final edgeId = _edgeId(edge);
      if (!oldIds.contains(edgeId)) {
        _fadeInEdge(edgeId);
      }
    }

    // Handle removed edges (fade out)
    for (final edge in oldEdges) {
      final edgeId = _edgeId(edge);
      if (!newIds.contains(edgeId)) {
        _fadeOutEdge(edgeId);
      }
    }
  }

  void _fadeInEdge(String edgeId) {
    _animationStates[edgeId] = EdgeAnimationState(opacity: 0);

    final controller = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 300),
    );

    controller.addListener(() {
      setState(() {
        final current = _animationStates[edgeId];
        if (current != null) {
          _animationStates[edgeId] = current.copyWith(
            opacity: EdgeAnimations.fadeEasing(controller.value),
          );
        }
      });
    });

    _transitionControllers[edgeId] = controller;
    controller
      ..addStatusListener((status) {
        if (status == AnimationStatus.completed) {
          controller.dispose();
          _transitionControllers.remove(edgeId);
        }
      })
      ..forward();
  }

  void _fadeOutEdge(String edgeId) {
    final current = _animationStates[edgeId];
    if (current == null) return;

    final controller = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 200),
    );

    controller.addListener(() {
      setState(() {
        _animationStates[edgeId] = current.copyWith(
          opacity: 1.0 - EdgeAnimations.fadeEasing(controller.value),
        );
      });
    });

    _transitionControllers[edgeId] = controller;
    controller
      ..addStatusListener((status) {
        if (status == AnimationStatus.completed) {
          setState(() {
            _animationStates.remove(edgeId);
          });
          controller.dispose();
          _transitionControllers.remove(edgeId);
        }
      })
      ..forward();
  }

  void _handleColorTransitions(List<PaintableEdge> oldEdges, List<PaintableEdge> newEdges) {
    for (var i = 0; i < newEdges.length && i < oldEdges.length; i++) {
      final oldEdge = oldEdges[i];
      final newEdge = newEdges[i];
      final edgeId = _edgeId(newEdge);

      if (oldEdge.status != newEdge.status || oldEdge.active != newEdge.active) {
        final oldColor = widget.style.colorForStatus(oldEdge.status, active: oldEdge.active);
        _previousColors[edgeId] = oldColor;
        _startColorTransition(edgeId);
      }
    }
  }

  void _startColorTransition(String edgeId) {
    _transitionControllers[edgeId]?.dispose();

    final controller = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 300),
    );

    controller.addListener(() {
      setState(() {
        final current = _animationStates[edgeId];
        if (current != null) {
          _animationStates[edgeId] = current.copyWith(colorTransition: controller.value);
        }
      });
    });

    _transitionControllers[edgeId] = controller;
    controller
      ..addStatusListener((status) {
        if (status == AnimationStatus.completed) {
          setState(() {
            _previousColors.remove(edgeId);
          });
          controller.dispose();
          _transitionControllers.remove(edgeId);
        }
      })
      ..forward();
  }

  String _edgeId(PaintableEdge edge) =>
      '${edge.start.dx}_${edge.start.dy}_${edge.end.dx}_${edge.end.dy}_${edge.priority}';

  bool _edgesEqual(List<PaintableEdge> a, List<PaintableEdge> b) {
    if (a.length != b.length) return false;
    for (var i = 0; i < a.length; i++) {
      if (a[i] != b[i]) return false;
    }
    return true;
  }

  @override
  Widget build(BuildContext context) {
    final selection = ref.watch(selectionNotifierProvider);

    return RepaintBoundary(
      child: MouseRegion(
        onHover: _handleHover,
        onExit: (_) => setState(() => _hoverPosition = null),
        child: GestureDetector(
          behavior: HitTestBehavior.translucent,
          onTapDown: _handleTapDown,
          onSecondaryTapDown: _handleSecondaryTapDown,
          child: Focus(
            onKeyEvent: _handleKeyEvent,
            child: CustomPaint(
              painter: _SelectableAnimatedEdgesPainter(
                edges: widget.paintableEdges,
                style: widget.style,
                hoverPosition: _hoverPosition,
                selection: selection,
                flowEdges: widget.edges,
                animationStates: _animationStates,
                pulseEnabled: true,
                previousColors: _previousColors,
              ),
            ),
          ),
        ),
      ),
    );
  }

  void _handleHover(PointerHoverEvent event) {
    setState(() => _hoverPosition = event.localPosition);
  }

  void _handleTapDown(TapDownDetails details) {
    final edgeIndex = _findEdgeAtPosition(details.localPosition);
    if (edgeIndex != null) {
      final edge = widget.edges[edgeIndex];
      ref
          .read(selectionNotifierProvider.notifier)
          .selectEdge(slotId: edge.slotId, from: edge.from, to: edge.to);
    } else {
      ref.read(selectionNotifierProvider.notifier).clearSelection();
    }
  }

  void _handleSecondaryTapDown(TapDownDetails details) {
    final edgeIndex = _findEdgeAtPosition(details.localPosition);
    if (edgeIndex != null) {
      final edge = widget.edges[edgeIndex];

      // Select the edge first
      ref
          .read(selectionNotifierProvider.notifier)
          .selectEdge(slotId: edge.slotId, from: edge.from, to: edge.to);

      // Show context menu
      _showContextMenu(details.globalPosition, edge);
    }
  }

  KeyEventResult _handleKeyEvent(FocusNode node, KeyEvent event) {
    if (event is KeyDownEvent && event.logicalKey == LogicalKeyboardKey.delete) {
      final selection = ref.read(selectionNotifierProvider);
      if (selection is EdgeSelection) {
        _deleteEdge(selection);
        return KeyEventResult.handled;
      }
    }
    return KeyEventResult.ignored;
  }

  int? _findEdgeAtPosition(Offset position) {
    final hitRadius = widget.style.hoverHitRadius;

    for (var i = 0; i < widget.paintableEdges.length; i++) {
      final paintableEdge = widget.paintableEdges[i];
      final distance = _distanceToEdge(paintableEdge, position);
      if (distance <= hitRadius) {
        return i;
      }
    }
    return null;
  }

  double _distanceToEdge(PaintableEdge edge, Offset point) {
    // Sample the cubic Bezier curve at discrete intervals
    const samples = 20;
    var minDistance = double.infinity;

    final controls = _computeControls(edge.start, edge.end);

    for (var i = 0; i <= samples; i++) {
      final t = i / samples;
      final pos = _cubicPoint(edge.start, controls.$1, controls.$2, edge.end, t);
      final distance = (pos - point).distance;
      if (distance < minDistance) {
        minDistance = distance;
      }
    }
    return minDistance;
  }

  (Offset, Offset) _computeControls(Offset start, Offset end) {
    final midX = (start.dx + end.dx) / 2;
    return (Offset(midX, start.dy), Offset(midX, end.dy));
  }

  Offset _cubicPoint(Offset p0, Offset p1, Offset p2, Offset p3, double t) {
    final mt = 1 - t;
    return (p0 * (mt * mt * mt)) +
        (p1 * (3 * mt * mt * t)) +
        (p2 * (3 * mt * t * t)) +
        (p3 * (t * t * t));
  }

  void _showContextMenu(Offset position, FlowEdge edge) {
    final overlay = Overlay.of(context).context.findRenderObject() as RenderBox?;

    showMenu<String>(
      context: context,
      position: RelativeRect.fromRect(
        Rect.fromLTWH(position.dx, position.dy, 0, 0),
        Rect.fromLTWH(0, 0, overlay?.size.width ?? 800, overlay?.size.height ?? 600),
      ),
      items: [
        PopupMenuItem<String>(
          value: 'toggle',
          child: Row(
            children: [
              Icon(edge.active ? Icons.toggle_on : Icons.toggle_off, size: 18),
              const SizedBox(width: 8),
              Text(edge.active ? 'Deactivate slot' : 'Activate slot'),
            ],
          ),
        ),
        const PopupMenuItem<String>(
          value: 'delete',
          child: Row(
            children: [
              Icon(Icons.delete_outline, size: 18, color: Colors.redAccent),
              SizedBox(width: 8),
              Text('Delete slot', style: TextStyle(color: Colors.redAccent)),
            ],
          ),
        ),
      ],
    ).then((value) {
      if (value == 'delete') {
        _deleteEdge(EdgeSelection(slotId: edge.slotId, from: edge.from, to: edge.to));
      } else if (value == 'toggle') {
        _toggleEdgeActive(edge);
      }
    });
  }

  Future<void> _deleteEdge(EdgeSelection selection) async {
    // Find the device ID from the edge
    final edge = widget.edges.firstWhere(
      (e) => e.slotId == selection.slotId && e.from == selection.from && e.to == selection.to,
      orElse: () => throw StateError('Edge not found'),
    );

    // Parse device ID from the "from" field (format: vendorId:productId:serial)
    final deviceId = _parseDeviceId(edge.from);

    // Show confirmation dialog
    final confirmed = await _showDeleteConfirmation();
    if (!confirmed) return;

    final actions = ref.read(slotActionsProvider);
    final result = await actions.deleteSlot(device: deviceId, slotId: edge.slotId);

    if (!mounted) return;

    if (result.hasError) {
      _showErrorSnackBar('Failed to delete slot: ${result.errorMessage}');
    } else {
      _showSuccessSnackBar('Slot deleted successfully');
      ref.read(selectionNotifierProvider.notifier).clearSelection();
    }
  }

  Future<void> _toggleEdgeActive(FlowEdge edge) async {
    final deviceId = _parseDeviceId(edge.from);

    // Create updated slot with toggled active state
    final updatedSlot = ProfileSlot(
      id: edge.slotId,
      hardwareProfileId: edge.to,
      keymapId: _findKeymapForSlot(edge.slotId),
      priority: edge.priority,
      active: !edge.active,
    );

    final actions = ref.read(slotActionsProvider);
    final result = await actions.updateSlot(device: deviceId, slot: updatedSlot);

    if (!mounted) return;

    if (result.hasError) {
      _showErrorSnackBar('Failed to toggle slot: ${result.errorMessage}');
    } else {
      _showSuccessSnackBar(edge.active ? 'Slot deactivated' : 'Slot activated');
    }
  }

  String _findKeymapForSlot(String slotId) {
    // Find the second edge with the same slotId (device -> profile -> keymap)
    final keymapEdge = widget.edges.firstWhere(
      (e) => e.slotId == slotId && e.from != widget.edges.first.from,
      orElse: () => throw StateError('Keymap edge not found for slot'),
    );
    return keymapEdge.to;
  }

  DeviceInstanceId _parseDeviceId(String deviceKey) {
    final parts = deviceKey.split(':');
    if (parts.length != 3) {
      throw FormatException('Invalid device key format: $deviceKey');
    }

    return DeviceInstanceId(
      vendorId: int.parse(parts[0]),
      productId: int.parse(parts[1]),
      serial: parts[2] == 'unknown' ? null : parts[2],
    );
  }

  Future<bool> _showDeleteConfirmation() async {
    final result = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Delete Slot'),
        content: const Text('Are you sure you want to delete this slot?'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(false),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () => Navigator.of(context).pop(true),
            style: TextButton.styleFrom(foregroundColor: Colors.redAccent),
            child: const Text('Delete'),
          ),
        ],
      ),
    );
    return result ?? false;
  }

  void _showErrorSnackBar(String message) {
    ScaffoldMessenger.of(
      context,
    ).showSnackBar(SnackBar(content: Text(message), backgroundColor: Colors.redAccent));
  }

  void _showSuccessSnackBar(String message) {
    ScaffoldMessenger.of(
      context,
    ).showSnackBar(SnackBar(content: Text(message), backgroundColor: Colors.green));
  }
}

/// Custom painter that highlights selected edges with animations.
class _SelectableAnimatedEdgesPainter extends AnimatedEdgesPainter {
  _SelectableAnimatedEdgesPainter({
    required super.edges,
    required super.style,
    required super.hoverPosition,
    required super.animationStates,
    required super.pulseEnabled,
    required super.previousColors,
    required this.selection,
    required this.flowEdges,
  });

  final Selection selection;
  final List<FlowEdge> flowEdges;

  @override
  void paint(Canvas canvas, Size size) {
    // First paint animated edges
    super.paint(canvas, size);

    // Then highlight selected edge
    if (selection is EdgeSelection) {
      _paintSelectedEdge(canvas, selection as EdgeSelection);
    }
  }

  void _paintSelectedEdge(Canvas canvas, EdgeSelection selection) {
    // Find the matching edge
    for (var i = 0; i < flowEdges.length; i++) {
      final edge = flowEdges[i];
      if (edge.slotId == selection.slotId &&
          edge.from == selection.from &&
          edge.to == selection.to) {
        final paintableEdge = edges[i];
        final edgeId = _edgeId(paintableEdge);
        final animState = animationStates[edgeId] ?? EdgeAnimationState();

        // Draw selection highlight with animation opacity
        final selectionPaint = Paint()
          ..color = Colors.blueAccent.withValues(alpha: 0.6 * animState.opacity)
          ..strokeWidth = style.activeWidth + 4
          ..style = PaintingStyle.stroke
          ..strokeCap = StrokeCap.round
          ..isAntiAlias = true
          ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 2);

        final controls = _computeControls(paintableEdge.start, paintableEdge.end);
        final path = _buildPath(paintableEdge.start, paintableEdge.end, controls.$1, controls.$2);

        canvas.drawPath(path, selectionPaint);
        break;
      }
    }
  }

  String _edgeId(PaintableEdge edge) =>
      '${edge.start.dx}_${edge.start.dy}_${edge.end.dx}_${edge.end.dy}_${edge.priority}';

  (Offset, Offset) _computeControls(Offset start, Offset end) {
    final midX = (start.dx + end.dx) / 2;
    return (Offset(midX, start.dy), Offset(midX, end.dy));
  }

  Path _buildPath(Offset start, Offset end, Offset control1, Offset control2) => Path()
    ..moveTo(start.dx, start.dy)
    ..cubicTo(control1.dx, control1.dy, control2.dx, control2.dy, end.dx, end.dy);

  @override
  bool shouldRepaint(covariant _SelectableAnimatedEdgesPainter oldDelegate) =>
      super.shouldRepaint(oldDelegate) || selection != oldDelegate.selection;
}
