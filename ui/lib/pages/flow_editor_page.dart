import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:keyrx_ui/models/flow/flow_data.dart';
import 'package:keyrx_ui/models/flow/flow_graph.dart';
import 'package:keyrx_ui/models/flow/node_position.dart';
import 'package:keyrx_ui/models/flow/edge_style.dart';
import 'package:keyrx_ui/state/device_stream_provider.dart';
import 'package:keyrx_ui/state/keyflow_providers.dart';
import 'package:keyrx_ui/state/node_positions_provider.dart';
import 'package:keyrx_ui/widgets/flow/device_node_widget.dart';
import 'package:keyrx_ui/widgets/flow/draggable_node.dart';
import 'package:keyrx_ui/widgets/flow/edges_painter.dart';
import 'package:keyrx_ui/widgets/flow/enhanced_device_node.dart';
import 'package:keyrx_ui/widgets/flow/enhanced_keymap_node.dart';
import 'package:keyrx_ui/widgets/flow/enhanced_layout_node.dart';
import 'package:keyrx_ui/widgets/flow/interactive_edge.dart';
import 'package:keyrx_ui/widgets/flow/keymap_node_widget.dart';
import 'package:keyrx_ui/widgets/flow/layout_node_widget.dart';
import 'package:keyrx_ui/widgets/flow/save_indicator.dart';
import 'package:keyrx_ui/widgets/flow/keyboard_navigable_canvas.dart';
import 'package:keyrx_ui/widgets/flow/focus_indicator.dart';
import 'package:keyrx_ui/dialogs/keyboard_shortcuts_help_dialog.dart';

class FlowEditorPage extends ConsumerStatefulWidget {
  const FlowEditorPage({super.key});

  @override
  ConsumerState<FlowEditorPage> createState() => _FlowEditorPageState();
}

class _FlowEditorPageState extends ConsumerState<FlowEditorPage>
    with SingleTickerProviderStateMixin {
  static const _canvasWidth = 1800.0;
  static const _canvasHeight = 1200.0;
  static const _edgeStyle = EdgeStyle(palette: defaultEdgePalette);

  late final AnimationController _shimmerController;
  late final Animation<double> _shimmer;
  late final TransformationController _transformationController;

  // Track previous node IDs to detect additions/removals
  Set<String> _previousNodeIds = {};

  @override
  void initState() {
    super.initState();
    _shimmerController = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 1200),
    )..repeat();
    _shimmer = Tween<double>(begin: -1, end: 2).animate(
      CurvedAnimation(parent: _shimmerController, curve: Curves.easeInOut),
    );
    _transformationController = TransformationController();

    // Listen to device events for real-time updates
    ref.read(deviceStreamNotifierProvider);
  }

  @override
  void dispose() {
    _shimmerController.dispose();
    _transformationController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final graphAsync = ref.watch(flowGraphProvider);
    final positionsAsync = ref.watch(nodePositionsProvider);

    if (graphAsync.isLoading || positionsAsync.isLoading) {
      return Scaffold(
        backgroundColor: const Color(0xFF121212),
        body: _FlowLoadingSkeleton(
          key: const Key('flow-loading-skeleton'),
          animation: _shimmer,
        ),
      );
    }

    if (graphAsync.hasError) {
      return _FlowErrorView(error: graphAsync.error, onRetry: _retryLoad);
    }

    if (positionsAsync.hasError) {
      return _FlowErrorView(error: positionsAsync.error, onRetry: _retryLoad);
    }

    final graph = graphAsync.value ?? const FlowGraph();
    final positions = positionsAsync.value ?? const <String, NodePosition>{};

    if (graph.nodes.isEmpty) {
      return const _FlowEmptyState();
    }

    final view = _buildCanvas(graph, positions);

    final canvasWidth = view.nodes.fold<double>(
      _canvasWidth,
      (width, node) =>
          math.max(width, node.position.dx + node.size.width + 200),
    );
    final canvasHeight = view.nodes.fold<double>(
      _canvasHeight,
      (height, node) =>
          math.max(height, node.position.dy + node.size.height + 200),
    );

    return Scaffold(
      backgroundColor: const Color(0xFF121212),
      body: Stack(
        children: [
          KeyboardNavigableCanvas(
            transformationController: _transformationController,
            canvasBounds: Size(canvasWidth, canvasHeight),
            onNodeDetailsRequested: _handleNodeDetailsRequest,
            onHelpRequested: () => _showKeyboardShortcutsHelp(context),
            child: InteractiveViewer(
              transformationController: _transformationController,
              boundaryMargin: const EdgeInsets.all(240),
              minScale: 0.5,
              maxScale: 2.5,
              child: SizedBox(
                width: canvasWidth,
                height: canvasHeight,
                child: Stack(
                  children: [
                    Positioned.fill(
                      child: CustomPaint(painter: _GridPainter()),
                    ),
                    Positioned.fill(
                      child: InteractiveEdgeCanvas(
                        edges: graph.edges,
                        paintableEdges: view.edges,
                        style: _edgeStyle,
                      ),
                    ),
                    ...view.nodes.map(
                      (node) => _AnimatedNodeWrapper(
                        key: ValueKey(node.id),
                        nodeId: node.id,
                        isNew: !_previousNodeIds.contains(node.id),
                        child: FocusIndicator(
                          nodeId: node.id,
                          child: DraggableNode(
                            nodeId: node.id,
                            initialPosition: node.position,
                            size: node.size,
                            canvasBounds: Size(canvasWidth, canvasHeight),
                            child: node.child,
                          ),
                        ),
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ),
          // Save indicator positioned in bottom-right corner
          const Positioned(bottom: 16, right: 16, child: SaveIndicator()),
        ],
      ),
    );
  }

  void _retryLoad() {
    ref.invalidate(flowGraphProvider);
    ref.invalidate(runtimeConfigProvider);
    ref.invalidate(nodePositionsProvider);
  }

  void _handleNodeDetailsRequest(String nodeId) {
    // TODO: Implement node details dialog in a future task
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text('Node details for $nodeId (to be implemented)'),
        duration: const Duration(seconds: 2),
      ),
    );
  }

  void _showKeyboardShortcutsHelp(BuildContext context) {
    KeyboardShortcutsHelpDialog.show(context);
  }

  _CanvasView _buildCanvas(
    FlowGraph graph,
    Map<String, NodePosition> storedPositions,
  ) {
    final resolvedPositions = _resolvePositions(graph, storedPositions);

    final nodes = graph.nodes.map((node) {
      final position = resolvedPositions[node.nodeId] ?? Offset.zero;
      return _CanvasNode(
        id: node.nodeId,
        position: position,
        size: _nodeSizeFor(node),
        child: _buildNodeWidget(node, graph),
      );
    }).toList();

    // Update tracking for next render
    _previousNodeIds = nodes.map((n) => n.id).toSet();

    final nodeMap = {for (final node in nodes) node.id: node};
    final edges = <PaintableEdge>[];
    for (final edge in graph.edges) {
      final fromNode = nodeMap[edge.from];
      final toNode = nodeMap[edge.to];
      if (fromNode == null || toNode == null) continue;

      edges.add(
        PaintableEdge(
          start: _edgeHandle(fromNode, isSource: true),
          end: _edgeHandle(toNode, isSource: false),
          status: edge.status,
          active: edge.active,
          priority: edge.priority,
        ),
      );
    }

    return _CanvasView(nodes: nodes, edges: edges);
  }

  Map<String, Offset> _resolvePositions(
    FlowGraph graph,
    Map<String, NodePosition> storedPositions,
  ) {
    final positions = <String, Offset>{};

    for (final node in graph.nodes) {
      final saved = storedPositions[node.nodeId];
      positions[node.nodeId] = saved?.offset ?? Offset.zero;
    }

    return positions;
  }

  Widget _buildNodeWidget(FlowNode node, FlowGraph graph) {
    return node.map(
      device: (value) {
        final edges = graph.edges
            .where((edge) => edge.from == value.id)
            .toList();
        final activeSlotCount = edges.where((e) => e.active).length;
        final totalSlotCount = edges.length;
        final hasError = edges.any((e) => e.status == EdgeStatus.invalid);

        return EnhancedDeviceNode(
          node: value,
          activeSlotCount: activeSlotCount,
          totalSlotCount: totalSlotCount,
          hasError: hasError,
          // onAddSlot can be added later when slot management is implemented
        );
      },
      hardwareProfile: (value) {
        final hasInvalidEdges = graph.edges.any(
          (edge) =>
              (edge.from == value.id || edge.to == value.id) &&
              edge.status == EdgeStatus.invalid,
        );

        return EnhancedLayoutNode(
          node: value,
          layoutType: 'Profile', // Can be enhanced later with actual type
          isValid: !hasInvalidEdges,
          errorMessage: hasInvalidEdges
              ? 'This profile has invalid connections'
              : null,
        );
      },
      keymap: (value) {
        final hasInvalidEdges = graph.edges.any(
          (edge) => edge.to == value.id && edge.status == EdgeStatus.invalid,
        );

        return EnhancedKeymapNode(
          node: value,
          isValid: !hasInvalidEdges,
          compilationFailed: hasInvalidEdges,
          errorMessage: hasInvalidEdges
              ? 'Keymap has compilation errors or invalid connections'
              : null,
        );
      },
    );
  }

  String _formatId(int id) => '0x${id.toRadixString(16).padLeft(4, '0')}';

  Offset _edgeHandle(_CanvasNode node, {required bool isSource}) {
    final dx = isSource ? node.size.width : 0.0;
    return node.position + Offset(dx, node.size.height / 2);
  }

  Size _nodeSizeFor(FlowNode node) {
    return node.map(
      device: (_) => const Size(260, 220),
      hardwareProfile: (_) => const Size(220, 160),
      keymap: (_) => const Size(200, 140),
    );
  }
}

class _GridPainter extends CustomPainter {
  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = const Color(0xFF333333).withOpacity(0.4)
      ..strokeWidth = 1;

    const gridSize = 20.0;

    for (double x = 0; x < size.width; x += gridSize) {
      canvas.drawLine(Offset(x, 0), Offset(x, size.height), paint);
    }
    for (double y = 0; y < size.height; y += gridSize) {
      canvas.drawLine(Offset(0, y), Offset(size.width, y), paint);
    }
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => false;
}

class _FlowLoadingSkeleton extends StatelessWidget {
  const _FlowLoadingSkeleton({super.key, required this.animation});

  final Animation<double> animation;

  @override
  Widget build(BuildContext context) {
    return AnimatedBuilder(
      animation: animation,
      builder: (context, child) {
        return Container(
          color: const Color(0xFF121212),
          padding: const EdgeInsets.all(32),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              _shimmerBar(width: 220),
              const SizedBox(height: 20),
              Expanded(
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    _shimmerCard(width: 240, height: 180),
                    _shimmerCard(width: 200, height: 140),
                    _shimmerCard(width: 180, height: 120),
                  ],
                ),
              ),
            ],
          ),
        );
      },
    );
  }

  Widget _shimmerBar({required double width}) {
    return _shimmerBox(width: width, height: 16, radius: 6);
  }

  Widget _shimmerCard({required double width, required double height}) {
    return _shimmerBox(width: width, height: height, radius: 10);
  }

  Widget _shimmerBox({
    required double width,
    required double height,
    required double radius,
  }) {
    final gradient = LinearGradient(
      begin: Alignment(-1, 0),
      end: Alignment(1, 0),
      colors: const [Color(0xFF1E1E1E), Color(0xFF2C2C2C), Color(0xFF1E1E1E)],
      stops: const [0.1, 0.5, 0.9],
      transform: _SlidingGradientTransform(slidePercent: animation.value),
    );

    return Container(
      width: width,
      height: height,
      decoration: BoxDecoration(
        color: const Color(0xFF1E1E1E),
        borderRadius: BorderRadius.circular(radius),
        gradient: gradient,
      ),
    );
  }
}

class _SlidingGradientTransform extends GradientTransform {
  const _SlidingGradientTransform({required this.slidePercent});

  final double slidePercent;

  @override
  Matrix4 transform(Rect bounds, {TextDirection? textDirection}) {
    return Matrix4.translationValues(bounds.width * slidePercent, 0, 0);
  }
}

class _FlowErrorView extends StatelessWidget {
  const _FlowErrorView({required this.error, required this.onRetry});

  final Object? error;
  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: const Color(0xFF121212),
      body: Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.error_outline, color: Colors.redAccent, size: 48),
            const SizedBox(height: 12),
            const Text(
              'Something went wrong',
              style: TextStyle(color: Colors.white, fontSize: 18),
            ),
            const SizedBox(height: 8),
            Text(
              '$error',
              style: const TextStyle(color: Colors.grey, fontSize: 12),
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: 16),
            ElevatedButton(onPressed: onRetry, child: const Text('Retry')),
          ],
        ),
      ),
    );
  }
}

class _FlowEmptyState extends StatelessWidget {
  const _FlowEmptyState();

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: const Color(0xFF121212),
      body: Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: const [
            Icon(Icons.usb_off_rounded, color: Colors.grey, size: 52),
            SizedBox(height: 12),
            Text(
              'No devices connected',
              style: TextStyle(color: Colors.white, fontSize: 18),
            ),
            SizedBox(height: 6),
            Text(
              'Connect a device to populate your flow.',
              style: TextStyle(color: Colors.grey, fontSize: 13),
            ),
          ],
        ),
      ),
    );
  }
}

class _CanvasView {
  const _CanvasView({required this.nodes, required this.edges});

  final List<_CanvasNode> nodes;
  final List<PaintableEdge> edges;
}

class _CanvasNode {
  const _CanvasNode({
    required this.id,
    required this.position,
    required this.size,
    required this.child,
  });

  final String id;
  final Offset position;
  final Size size;
  final Widget child;
}

/// Wrapper widget that animates node appearance/disappearance.
///
/// Provides fade-in animation for new nodes and smooth transitions
/// for node updates.
class _AnimatedNodeWrapper extends StatefulWidget {
  const _AnimatedNodeWrapper({
    super.key,
    required this.nodeId,
    required this.isNew,
    required this.child,
  });

  final String nodeId;
  final bool isNew;
  final Widget child;

  @override
  State<_AnimatedNodeWrapper> createState() => _AnimatedNodeWrapperState();
}

class _AnimatedNodeWrapperState extends State<_AnimatedNodeWrapper>
    with SingleTickerProviderStateMixin {
  late final AnimationController _controller;
  late final Animation<double> _fadeAnimation;
  late final Animation<double> _scaleAnimation;

  @override
  void initState() {
    super.initState();

    _controller = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 300),
    );

    _fadeAnimation = Tween<double>(
      begin: widget.isNew ? 0.0 : 1.0,
      end: 1.0,
    ).animate(CurvedAnimation(parent: _controller, curve: Curves.easeOut));

    _scaleAnimation = Tween<double>(
      begin: widget.isNew ? 0.9 : 1.0,
      end: 1.0,
    ).animate(CurvedAnimation(parent: _controller, curve: Curves.easeOutBack));

    if (widget.isNew) {
      _controller.forward();
    } else {
      _controller.value = 1.0;
    }
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return FadeTransition(
      opacity: _fadeAnimation,
      child: ScaleTransition(scale: _scaleAnimation, child: widget.child),
    );
  }
}
