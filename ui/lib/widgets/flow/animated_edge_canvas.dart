import 'dart:async';

import 'package:core_logic/models/flow/edge_style.dart';
import 'package:flutter/material.dart';

import '../../animations/edge_animations.dart';
import '../../utils/core_ui_extensions.dart';
import 'animated_edges_painter.dart';
import 'edges_painter.dart';

/// Animated wrapper for edge rendering with pulse and transition effects
class AnimatedEdgeCanvas extends StatefulWidget {
  const AnimatedEdgeCanvas({
    required this.edges,
    required this.style,
    this.hoverPosition,
    this.pulseEnabled = true,
    this.child,
    super.key,
  });

  final List<PaintableEdge> edges;
  final EdgeStyle style;
  final Offset? hoverPosition;
  final bool pulseEnabled;
  final Widget? child;

  @override
  State<AnimatedEdgeCanvas> createState() => _AnimatedEdgeCanvasState();
}

class _AnimatedEdgeCanvasState extends State<AnimatedEdgeCanvas> with TickerProviderStateMixin {
  late final AnimationController _pulseController;
  final Map<String, EdgeAnimationState> _animationStates = {};
  final Map<String, Color> _previousColors = {};
  final Map<String, AnimationController> _transitionControllers = {};
  Timer? _pulseTimer;

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
  void didUpdateWidget(AnimatedEdgeCanvas oldWidget) {
    super.didUpdateWidget(oldWidget);

    // Handle edge additions/removals with fade animations
    if (oldWidget.edges.length != widget.edges.length ||
        !_edgesEqual(oldWidget.edges, widget.edges)) {
      _handleEdgeChanges(oldWidget.edges, widget.edges);
    }

    // Handle validation state color transitions
    _handleColorTransitions(oldWidget.edges, widget.edges);
  }

  @override
  void dispose() {
    _pulseController.dispose();
    _pulseTimer?.cancel();

    for (final controller in _transitionControllers.values) {
      controller.dispose();
    }
    _transitionControllers.clear();

    super.dispose();
  }

  void _initializeEdgeStates() {
    for (final edge in widget.edges) {
      final edgeId = _edgeId(edge);
      _animationStates[edgeId] = EdgeAnimationState();
    }
  }

  void _updatePulsePhase() {
    if (!widget.pulseEnabled) return;

    setState(() {
      final phase = _pulseController.value;
      for (final edge in widget.edges) {
        if (edge.active) {
          final edgeId = _edgeId(edge);
          final current = _animationStates[edgeId];
          if (current != null) {
            _animationStates[edgeId] = current.copyWith(pulsePhase: phase);
          }
        }
      }
    });
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
    // Start with transparent
    _animationStates[edgeId] = EdgeAnimationState(opacity: 0);

    // Create fade-in animation
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

    // Create fade-out animation
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

      // Check if status or active state changed
      if (oldEdge.status != newEdge.status || oldEdge.active != newEdge.active) {
        final oldColor = widget.style.colorForStatus(oldEdge.status, active: oldEdge.active);
        _previousColors[edgeId] = oldColor;

        // Start color transition
        _startColorTransition(edgeId);
      }
    }
  }

  void _startColorTransition(String edgeId) {
    // Cancel existing transition if any
    _transitionControllers[edgeId]?.dispose();

    final controller = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 300),
    );

    controller
      ..addListener(() {
        setState(() {
          final current = _animationStates[edgeId];
          if (current != null) {
            _animationStates[edgeId] = current.copyWith(colorTransition: controller.value);
          }
        });
      })
      ..addStatusListener((status) {
        if (status == AnimationStatus.completed) {
          setState(() {
            _previousColors.remove(edgeId);
          });
          controller.dispose();
          _transitionControllers.remove(edgeId);
        }
      });

    // Reset transition progress
    setState(() {
      final current = _animationStates[edgeId];
      if (current != null) {
        _animationStates[edgeId] = current.copyWith(colorTransition: 0);
      }
    });

    _transitionControllers[edgeId] = controller;
    controller.forward();
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
  Widget build(BuildContext context) => RepaintBoundary(
    child: CustomPaint(
      painter: AnimatedEdgesPainter(
        edges: widget.edges,
        style: widget.style,
        hoverPosition: widget.hoverPosition,
        animationStates: _animationStates,
        pulseEnabled: widget.pulseEnabled,
        previousColors: _previousColors,
      ),
      child: widget.child,
    ),
  );
}
