import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:keyrx_ui/models/device_identity.dart';
import 'package:keyrx_ui/models/device_state.dart';
import 'package:keyrx_ui/models/flow/flow_graph.dart';
import 'package:keyrx_ui/models/flow/node_position.dart';
import 'package:keyrx_ui/models/flow/edge_style.dart';
import 'package:keyrx_ui/state/device_stream_provider.dart';
import 'package:keyrx_ui/state/drag_controller.dart';
import 'package:keyrx_ui/state/keyflow_providers.dart';
import 'package:keyrx_ui/state/node_positions_provider.dart';
import 'package:keyrx_ui/widgets/flow/draggable_node.dart';
import 'package:keyrx_ui/widgets/flow/enhanced_device_node.dart';
import 'package:keyrx_ui/widgets/flow/enhanced_keymap_node.dart';
import 'package:keyrx_ui/widgets/flow/enhanced_layout_node.dart';
import 'package:keyrx_ui/widgets/flow/save_indicator.dart';
import 'package:keyrx_ui/widgets/flow/keyboard_navigable_canvas.dart';
import 'package:keyrx_ui/widgets/flow/focus_indicator.dart';
import 'package:keyrx_ui/widgets/flow/optimized_flow_canvas.dart';
import 'package:keyrx_ui/widgets/flow/edges_painter.dart';
import 'package:keyrx_ui/widgets/flow/error_view.dart';
import 'package:keyrx_ui/widgets/flow/flow_canvas_internals.dart';
import 'package:keyrx_ui/widgets/flow/loading_skeleton.dart'
    hide FlowEmptyState, DisconnectedDevicesState;
import 'package:keyrx_ui/widgets/flow/empty_state.dart';
import 'package:keyrx_ui/widgets/flow/validation_overlay.dart';
import 'package:keyrx_ui/services/flow_validator.dart';
import 'package:keyrx_ui/utils/rendering_optimizer.dart';
import 'package:keyrx_ui/dialogs/keyboard_shortcuts_help_dialog.dart';
import 'package:keyrx_ui/dialogs/keyflow_onboarding_dialog.dart';
import 'package:keyrx_ui/dialogs/node_details_dialog.dart';
import 'package:keyrx_ui/state/analytics_providers.dart';

class FlowEditorPage extends ConsumerStatefulWidget {
  const FlowEditorPage({super.key});

  @override
  ConsumerState<FlowEditorPage> createState() => _FlowEditorPageState();
}

class _FlowEditorPageState extends ConsumerState<FlowEditorPage>
    with ViewportAware {
  static const _canvasWidth = 8000.0;
  static const _canvasHeight = 6000.0;
  static const _edgeStyle = EdgeStyle(palette: defaultEdgePalette);

  late final TransformationController _transformationController;

  // Track previous node IDs to detect additions/removals
  Set<String> _previousNodeIds = {};

  // Track if we've done initial fit to viewport
  bool _hasPerformedInitialFit = false;

  // Performance tracking
  Rect _currentViewport = Rect.zero;
  final FrameTimer _frameTimer = FrameTimer();
  RenderingMetrics? _lastMetrics;

  // Validation
  final _validator = const FlowValidator();

  @override
  void initState() {
    super.initState();
    _transformationController = TransformationController();

    // Initial zoom will be calculated by _fitNodesToViewport() after first build
    _transformationController.value = Matrix4.identity();

    _transformationController.addListener(_onTransformChanged);

    // Listen to device events for real-time updates
    ref.read(deviceStreamNotifierProvider);

    // Show onboarding dialog on first visit
    WidgetsBinding.instance.addPostFrameCallback((_) {
      KeyFlowOnboardingDialog.showIfNeeded(context);

      // Track page view with initial metrics
      _trackPageViewMetrics();
    });
  }

  void _trackPageViewMetrics() {
    final graphAsync = ref.read(flowGraphProvider);
    graphAsync.whenData((graph) {
      final deviceCount = graph.nodes.whereType<DeviceFlowNode>().length;
      final slotCount = graph.edges.length;
      final nodeCount = graph.nodes.length;

      ref
          .read(analyticsTrackerProvider.notifier)
          .trackPageView(
            deviceCount: deviceCount,
            slotCount: slotCount,
            nodeCount: nodeCount,
          );
    });
  }

  void _onTransformChanged() {
    // Calculate viewport from current transformation
    final screenSize = MediaQuery.of(context).size;
    final newViewport = calculateViewport(
      transform: _transformationController.value,
      screenSize: screenSize,
    );

    // Only rebuild if viewport changed significantly
    if (hasViewportChanged(newViewport)) {
      setState(() {
        _currentViewport = newViewport;
      });
    }
  }

  void _fitNodesToViewport(List<CanvasNode> nodes) {
    if (nodes.isEmpty) return;

    // Calculate bounding box of all nodes
    double minX = double.infinity;
    double minY = double.infinity;
    double maxX = double.negativeInfinity;
    double maxY = double.negativeInfinity;

    for (final node in nodes) {
      minX = math.min(minX, node.position.dx);
      minY = math.min(minY, node.position.dy);
      maxX = math.max(maxX, node.position.dx + node.size.width);
      maxY = math.max(maxY, node.position.dy + node.size.height);
    }

    // Add padding around the content
    const padding = 100.0;
    final contentWidth = maxX - minX + (padding * 2);
    final contentHeight = maxY - minY + (padding * 2);

    // Get available viewport size
    final screenSize = MediaQuery.of(context).size;

    // Calculate zoom to fit content in viewport
    final scaleX = screenSize.width / contentWidth;
    final scaleY = screenSize.height / contentHeight;
    final scale = math.min(scaleX, scaleY).clamp(0.5, 2.5);

    // Center the content
    final offsetX = (minX - padding) * scale;
    final offsetY = (minY - padding) * scale;

    // Apply transformation
    _transformationController.value = Matrix4.identity()
      ..translate(-offsetX, -offsetY)
      ..scale(scale);
  }

  @override
  void dispose() {
    _transformationController.removeListener(_onTransformChanged);
    _transformationController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final graphAsync = ref.watch(flowGraphProvider);
    final positionsAsync = ref.watch(nodePositionsProvider);
    final deviceStatesAsync = ref.watch(deviceStatesProvider);

    if (graphAsync.isLoading || positionsAsync.isLoading || deviceStatesAsync.isLoading) {
      return const FlowLoadingSkeleton(key: Key('flow-loading-skeleton'));
    }

    if (graphAsync.hasError) {
      return FlowErrorView(error: graphAsync.error, onRetry: _retryLoad);
    }

    if (positionsAsync.hasError) {
      return FlowErrorView(error: positionsAsync.error, onRetry: _retryLoad);
    }

    final graph = graphAsync.value ?? const FlowGraph();
    final positions = positionsAsync.value ?? const <String, NodePosition>{};
    final deviceStates = deviceStatesAsync.value ?? <DeviceState>[];

    // Check for empty state scenarios
    if (graph.nodes.isEmpty) {
      return const FlowEmptyState();
    }

    // Check if all devices are disconnected
    final deviceNodes = graph.nodes.whereType<DeviceFlowNode>().toList();
    final connectedDevices = deviceNodes.where((d) => d.isConnected).toList();

    if (deviceNodes.isNotEmpty && connectedDevices.isEmpty) {
      return FlowWaitingState(deviceCount: deviceNodes.length);
    }

    final view = _buildCanvas(graph, positions, deviceStates);

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

    // Auto-fit zoom to show all nodes on first load only
    if (!_hasPerformedInitialFit && view.nodes.isNotEmpty) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) {
          _fitNodesToViewport(view.nodes);
          _hasPerformedInitialFit = true;
        }
      });
    }

    return Scaffold(
      backgroundColor: const Color(0xFF121212),
      body: Stack(
        children: [
          Positioned.fill(
            child: KeyboardNavigableCanvas(
              transformationController: _transformationController,
              canvasBounds: Size(canvasWidth, canvasHeight),
              onNodeDetailsRequested: _handleNodeDetailsRequest,
              onHelpRequested: () => _showKeyboardShortcutsHelp(context),
              child: InteractiveViewer(
                constrained: false,
                transformationController: _transformationController,
                boundaryMargin: const EdgeInsets.all(240),
                minScale: 0.5,
                maxScale: 2.5,
                child: SizedBox(
                  width: canvasWidth,
                  height: canvasHeight,
                  child: Stack(
                  children: [
                    // Grid layer with RepaintBoundary
                    Positioned.fill(
                      child: RepaintBoundary(
                        child: CustomPaint(painter: GridPainter()),
                      ),
                    ),
                    // Optimized edge layer with custom RenderObject
                    Positioned.fill(
                      child: RepaintBoundary(
                        child: OptimizedEdgeLayer(
                          edges: view.edges,
                          style: _edgeStyle,
                        ),
                      ),
                    ),
                    // Virtualized nodes - only render visible ones
                    ..._buildVirtualizedNodes(
                      view.nodes,
                      canvasWidth,
                      canvasHeight,
                    ),
                  ],
                ),
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
    NodeDetailsDialog.show(context, nodeId);
  }

  void _showKeyboardShortcutsHelp(BuildContext context) {
    KeyboardShortcutsHelpDialog.show(context);
  }

  /// Build virtualized nodes - only render nodes visible in viewport.
  List<Widget> _buildVirtualizedNodes(
    List<CanvasNode> allNodes,
    double canvasWidth,
    double canvasHeight,
  ) {
    _frameTimer.start();

    // Initialize viewport if needed
    if (_currentViewport == Rect.zero) {
      final screenSize = MediaQuery.of(context).size;
      _currentViewport = Rect.fromLTWH(
        0,
        0,
        screenSize.width,
        screenSize.height,
      );
    }

    // Filter visible nodes
    final visibleNodes = allNodes.where((node) {
      return isNodeInViewport(
        nodePosition: node.position,
        nodeSize: node.size,
        viewport: _currentViewport,
        margin: 300.0, // Large margin for smooth scrolling
      );
    }).toList();

    // Build widgets with Positioned at top level (direct child of Stack)
    final widgets = visibleNodes.map((node) {
      return Consumer(
        key: ValueKey(node.id),
        builder: (context, ref, _) {
          // Watch drag state to update position dynamically
          final dragState = ref.watch(dragControllerProvider);
          final isDragging = dragState?.nodeId == node.id && dragState?.isDragging == true;
          final currentPosition = isDragging
              ? (dragState?.currentPosition ?? node.position)
              : node.position;

          return Positioned(
            left: currentPosition.dx,
            top: currentPosition.dy,
            child: AnimatedNodeWrapper(
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
          );
        },
      );
    }).toList();

    // Track performance metrics
    final frameTime = _frameTimer.stop();
    _lastMetrics = RenderingMetrics(
      totalNodes: allNodes.length,
      visibleNodes: visibleNodes.length,
      totalEdges: 0, // Will be filled when edge virtualization is added
      visibleEdges: 0,
      frameTime: frameTime,
    );

    // Log performance issues (in debug mode only)
    assert(() {
      if (!_lastMetrics!.meetsPerformanceTarget) {
        debugPrint('Performance warning: ${_lastMetrics.toString()}');
      }
      return true;
    }());

    return widgets;
  }

  CanvasView _buildCanvas(
    FlowGraph graph,
    Map<String, NodePosition> storedPositions,
    List<DeviceState> deviceStates,
  ) {
    final resolvedPositions = _resolvePositions(graph, storedPositions);

    final nodes = graph.nodes.map((node) {
      final position = resolvedPositions[node.nodeId] ?? Offset.zero;
      return CanvasNode(
        id: node.nodeId,
        position: position,
        size: _nodeSizeFor(node),
        child: _buildNodeWidget(node, graph, deviceStates),
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

    return CanvasView(nodes: nodes, edges: edges);
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

  Widget _buildNodeWidget(FlowNode node, FlowGraph graph, List<DeviceState> deviceStates) {
    // Get validation errors for this node
    final validationErrors = _validator.getNodeErrors(graph, node.nodeId);

    final nodeWidget = node.map(
      device: (value) {
        final edges = graph.edges
            .where((edge) => edge.from == value.id)
            .toList();
        final activeSlotCount = edges.where((e) => e.active).length;
        final totalSlotCount = edges.length;
        final hasError = edges.any((e) => e.status == EdgeStatus.invalid);

        // Find matching device state to get deviceEnabled
        final matchingState = deviceStates.firstWhere(
          (state) {
            final matchesVidPid = state.identity.vendorId == value.deviceId.vendorId &&
                state.identity.productId == value.deviceId.productId;

            // If device instance has no serial or empty serial, match any serial
            final matchesSerial = (value.deviceId.serial == null ||
                                   value.deviceId.serial!.isEmpty) ||
                                  state.identity.serialNumber == value.deviceId.serial;

            return matchesVidPid && matchesSerial;
          },
          orElse: () => DeviceState(
            identity: DeviceIdentity(
              vendorId: value.deviceId.vendorId,
              productId: value.deviceId.productId,
              serialNumber: value.deviceId.serial ?? '',
            ),
            deviceEnabled: true, // Default to enabled if not found
            remapEnabled: false,
            connectedAt: '',
            updatedAt: DateTime.now().toIso8601String(),
          ),
        );

        return EnhancedDeviceNode(
          node: value,
          activeSlotCount: activeSlotCount,
          totalSlotCount: totalSlotCount,
          hasError: hasError,
          deviceEnabled: matchingState.deviceEnabled,
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

    // Wrap with validation badge if there are errors
    if (validationErrors.isEmpty) {
      return nodeWidget;
    }

    return Stack(
      clipBehavior: Clip.none,
      children: [
        nodeWidget,
        Positioned(
          top: -8,
          right: -8,
          child: ValidationBadge(
            errors: validationErrors,
            onTap: () => _showValidationErrors(node.nodeId, validationErrors),
          ),
        ),
      ],
    );
  }

  void _showValidationErrors(String nodeId, List<ValidationError> errors) {
    showDialog<void>(
      context: context,
      builder: (context) => ValidationErrorsDialog(
        title: 'Validation Errors for $nodeId',
        errors: errors,
      ),
    );
  }

  String _formatId(int id) => '0x${id.toRadixString(16).padLeft(4, '0')}';

  Offset _edgeHandle(CanvasNode node, {required bool isSource}) {
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
