import 'package:flutter/material.dart';
import 'package:keyrx_ui/widgets/flow/device_node_widget.dart';
import 'package:keyrx_ui/widgets/flow/layout_node_widget.dart';
import 'package:keyrx_ui/widgets/flow/keymap_node_widget.dart';
import 'package:keyrx_ui/models/flow/flow_data.dart';

class FlowEditorPage extends StatefulWidget {
  const FlowEditorPage({super.key});

  @override
  State<FlowEditorPage> createState() => _FlowEditorPageState();
}

class _FlowEditorPageState extends State<FlowEditorPage> {
  // Simple State for now
  final List<FlowNodeWrapper> nodes = [];
  final List<FlowEdge> edges = [];

  @override
  void initState() {
    super.initState();
    _loadInitialData();
  }

  void _loadInitialData() {
    // Device 1
    nodes.add(
      FlowNodeWrapper(
        id: 'device-1',
        position: const Offset(50, 100),
        child: const DeviceNodeWidget(
          data: DeviceNodeData(
            name: 'Keychron Q1',
            vid: '0x3434',
            pid: '0x0120',
            isConnected: true,
            wirings: [
              DeviceWiring(id: 'w1', name: 'Main Config', isActive: true),
              DeviceWiring(id: 'w2', name: 'Fallback Config', isActive: false),
            ],
          ),
        ),
      ),
    );

    // Layout 1
    nodes.add(
      FlowNodeWrapper(
        id: 'layout-1',
        position: const Offset(450, 50),
        child: const LayoutNodeWidget(
          data: LayoutNodeData(name: 'ANSI 104', type: 'preset'),
        ),
      ),
    );

    // Keymap 1
    nodes.add(
      FlowNodeWrapper(
        id: 'keymap-1',
        position: const Offset(750, 50),
        child: const KeymapNodeWidget(
          data: KeymapNodeData(name: 'Gaming Layer', layers: 4, isActive: true),
        ),
      ),
    );

    // Layout 2 (Fallback)
    nodes.add(
      FlowNodeWrapper(
        id: 'layout-2',
        position: const Offset(450, 300),
        child: const LayoutNodeWidget(
          data: LayoutNodeData(name: 'ISO 105', type: 'preset'),
        ),
      ),
    );

    // Keymap 2
    nodes.add(
      FlowNodeWrapper(
        id: 'keymap-2',
        position: const Offset(750, 300),
        child: const KeymapNodeWidget(
          data: KeymapNodeData(
            name: 'Office Layer',
            layers: 2,
            isActive: false,
          ),
        ),
      ),
    );

    // Edges
    // Note: Offsets are approximate for now. A real system would calculate handle positions dynamically.
    edges.add(
      FlowEdge(
        start: const Offset(290, 190), // Device w1 handle
        end: const Offset(450, 120), // Layout 1 in
        color: const Color(0xFF1AB394),
        isActive: true,
      ),
    );
    edges.add(
      FlowEdge(
        start: const Offset(650, 120), // Layout 1 out
        end: const Offset(750, 120), // Keymap 1 in
        color: const Color(0xFF9B59B6),
        isActive: true,
      ),
    );

    edges.add(
      FlowEdge(
        start: const Offset(290, 220), // Device w2 handle
        end: const Offset(450, 370), // Layout 2 in
        color: Colors.grey,
        isActive: false,
        isDashed: true,
      ),
    );
    edges.add(
      FlowEdge(
        start: const Offset(650, 370), // Layout 2 out
        end: const Offset(750, 370), // Keymap 2 in
        color: Colors.grey,
        isActive: false,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: const Color(0xFF121212),
      body: InteractiveViewer(
        boundaryMargin: const EdgeInsets.all(double.infinity),
        minScale: 0.1,
        maxScale: 2.0,
        child: SizedBox(
          width: 2000,
          height: 1500,
          child: Stack(
            children: [
              // Background Grid
              Positioned.fill(child: CustomPaint(painter: GridPainter())),

              // Edges (Painted below nodes)
              Positioned.fill(child: CustomPaint(painter: EdgesPainter(edges))),

              // Nodes
              ...nodes.map(
                (node) => Positioned(
                  left: node.position.dx,
                  top: node.position.dy,
                  child: node.child,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class FlowNodeWrapper {
  final String id;
  final Offset position;
  final Widget child;

  FlowNodeWrapper({
    required this.id,
    required this.position,
    required this.child,
  });
}

class FlowEdge {
  final Offset start;
  final Offset end;
  final Color color;
  final bool isActive;
  final bool isDashed;

  FlowEdge({
    required this.start,
    required this.end,
    required this.color,
    this.isActive = true,
    this.isDashed = false,
  });
}

class GridPainter extends CustomPainter {
  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = const Color(0xFF333333).withValues(alpha: 0.5)
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

class EdgesPainter extends CustomPainter {
  final List<FlowEdge> edges;

  EdgesPainter(this.edges);

  @override
  void paint(Canvas canvas, Size size) {
    for (final edge in edges) {
      final paint = Paint()
        ..color = edge.color
        ..strokeWidth = 2
        ..style = PaintingStyle.stroke;

      final path = Path();
      path.moveTo(edge.start.dx, edge.start.dy);

      // Bezier curve
      final midX = (edge.start.dx + edge.end.dx) / 2;
      path.cubicTo(
        midX,
        edge.start.dy,
        midX,
        edge.end.dy,
        edge.end.dx,
        edge.end.dy,
      );

      if (edge.isDashed) {
        // Simple dashed line implementation
        final dashPath = Path();
        final metrics = path.computeMetrics();
        for (final metric in metrics) {
          var distance = 0.0;
          while (distance < metric.length) {
            dashPath.addPath(
              metric.extractPath(distance, distance + 5),
              Offset.zero,
            );
            distance += 10;
          }
        }
        canvas.drawPath(dashPath, paint);
      } else {
        canvas.drawPath(path, paint);
      }
    }
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => true;
}
