/// Tests for DraggableNode widget covering drag gestures, position updates, and constraints.
library;

import 'package:core_logic/models/flow/node_position.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/state/drag_controller.dart';
import 'package:keyrx_ui/state/node_positions_provider.dart';
import 'package:keyrx_ui/widgets/flow/draggable_node.dart';

void main() {
  group('DraggableNode', () {
    late Map<String, Offset> positions;
    late List<Override> overrides;

    setUp(() {
      positions = {};
      overrides = [
        dragControllerProvider.overrideWith(DragController.new),
        nodePositionsProvider.overrideWith(() => _FakeNodePositions(positions)),
      ];
    });

    Widget buildTestWidget({
      required String nodeId,
      Offset? initialPosition,
      Size? size,
      Size? canvasBounds,
    }) => ProviderScope(
      overrides: overrides,
      child: MaterialApp(
        home: Scaffold(
          body: Stack(
            children: [
              DraggableNode(
                nodeId: nodeId,
                initialPosition: initialPosition ?? const Offset(100, 100),
                size: size ?? const Size(120, 80),
                canvasBounds: canvasBounds ?? const Size(800, 600),
                child: Container(
                  width: size?.width ?? 120,
                  height: size?.height ?? 80,
                  color: Colors.blue,
                  child: const Center(child: Text('Test Node')),
                ),
              ),
            ],
          ),
        ),
      ),
    );

    testWidgets('renders child widget at initial position', (tester) async {
      await tester.pumpWidget(
        buildTestWidget(
          nodeId: 'node-1',
          initialPosition: const Offset(150, 200),
        ),
      );

      final positioned = tester.widget<Positioned>(find.byType(Positioned));
      expect(positioned.left, 150);
      expect(positioned.top, 200);
      expect(find.text('Test Node'), findsOneWidget);
    });

    testWidgets('starts drag on pan start', (tester) async {
      await tester.pumpWidget(
        buildTestWidget(
          nodeId: 'node-1',
          initialPosition: const Offset(100, 100),
        ),
      );

      final container = ProviderScope.containerOf(
        tester.element(find.byType(DraggableNode)),
      );

      // Before drag
      expect(container.read(dragControllerProvider), isNull);

      // Start drag
      await tester.startGesture(const Offset(160, 140));
      await tester.pump();

      // After drag start
      final dragState = container.read(dragControllerProvider);
      expect(dragState, isNotNull);
      expect(dragState!.nodeId, 'node-1');
      expect(dragState.initialPosition, const Offset(100, 100));
      expect(dragState.isDragging, isTrue);
    });

    testWidgets('updates position during drag', (tester) async {
      await tester.pumpWidget(
        buildTestWidget(
          nodeId: 'node-1',
          initialPosition: const Offset(100, 100),
        ),
      );

      final container = ProviderScope.containerOf(
        tester.element(find.byType(DraggableNode)),
      );

      // Start and drag
      final gesture = await tester.startGesture(const Offset(160, 140));
      await tester.pump();

      await gesture.moveBy(const Offset(50, 30));
      await tester.pump();

      // Check drag state updated
      final dragState = container.read(dragControllerProvider);
      expect(dragState, isNotNull);
      expect(dragState!.currentOffset, const Offset(50, 30));
      expect(dragState.currentPosition, const Offset(150, 130));

      // Verify widget moved visually
      final positioned = tester.widget<Positioned>(find.byType(Positioned));
      expect(positioned.left, 150);
      expect(positioned.top, 130);

      await gesture.up();
    });

    testWidgets('persists position on drag end', (tester) async {
      await tester.pumpWidget(
        buildTestWidget(
          nodeId: 'node-1',
          initialPosition: const Offset(100, 100),
        ),
      );

      final gesture = await tester.startGesture(const Offset(160, 140));
      await gesture.moveBy(const Offset(50, 30));
      await tester.pump();
      await gesture.up();
      await tester.pumpAndSettle();

      // Check position was persisted
      expect(positions['node-1'], const Offset(150, 130));
    });

    testWidgets('constrains position within canvas bounds - right edge', (
      tester,
    ) async {
      await tester.pumpWidget(
        buildTestWidget(
          nodeId: 'node-1',
          initialPosition: const Offset(100, 100),
          size: const Size(120, 80),
          canvasBounds: const Size(300, 300),
        ),
      );

      // Try to drag beyond right edge (max allowed: 300 - 120 = 180)
      final gesture = await tester.startGesture(const Offset(160, 140));
      await gesture.moveBy(
        const Offset(150, 0),
      ); // Would go to x=250, but should be clamped to 180
      await tester.pump();
      await gesture.up();
      await tester.pumpAndSettle();

      // Check position was constrained
      expect(positions['node-1']!.dx, 180); // clamped to max
      expect(positions['node-1']!.dy, 100); // unchanged
    });

    testWidgets('constrains position within canvas bounds - left edge', (
      tester,
    ) async {
      await tester.pumpWidget(
        buildTestWidget(
          nodeId: 'node-1',
          initialPosition: const Offset(100, 100),
          size: const Size(120, 80),
          canvasBounds: const Size(300, 300),
        ),
      );

      // Try to drag beyond left edge
      final gesture = await tester.startGesture(const Offset(160, 140));
      await gesture.moveBy(
        const Offset(-150, 0),
      ); // Would go to x=-50, should be clamped to 0
      await tester.pump();
      await gesture.up();
      await tester.pumpAndSettle();

      // Check position was constrained
      expect(positions['node-1']!.dx, 0); // clamped to min
    });

    testWidgets('constrains position within canvas bounds - bottom edge', (
      tester,
    ) async {
      await tester.pumpWidget(
        buildTestWidget(
          nodeId: 'node-1',
          initialPosition: const Offset(100, 100),
          size: const Size(120, 80),
          canvasBounds: const Size(300, 250),
        ),
      );

      // Try to drag beyond bottom edge (max allowed: 250 - 80 = 170)
      final gesture = await tester.startGesture(const Offset(160, 140));
      await gesture.moveBy(
        const Offset(0, 100),
      ); // Would go to y=200, should be clamped to 170
      await tester.pump();
      await gesture.up();
      await tester.pumpAndSettle();

      // Check position was constrained
      expect(positions['node-1']!.dy, 170); // clamped to max
    });

    testWidgets('constrains position within canvas bounds - top edge', (
      tester,
    ) async {
      await tester.pumpWidget(
        buildTestWidget(
          nodeId: 'node-1',
          initialPosition: const Offset(100, 100),
          size: const Size(120, 80),
          canvasBounds: const Size(300, 300),
        ),
      );

      // Try to drag beyond top edge
      final gesture = await tester.startGesture(const Offset(160, 140));
      await gesture.moveBy(
        const Offset(0, -150),
      ); // Would go to y=-50, should be clamped to 0
      await tester.pump();
      await gesture.up();
      await tester.pumpAndSettle();

      // Check position was constrained
      expect(positions['node-1']!.dy, 0); // clamped to min
    });

    testWidgets('shows elevation shadow during drag', (tester) async {
      await tester.pumpWidget(
        buildTestWidget(
          nodeId: 'node-1',
          initialPosition: const Offset(100, 100),
        ),
      );

      // Before drag - no shadow or minimal shadow
      var container = tester.widget<AnimatedContainer>(
        find.byType(AnimatedContainer),
      );
      var decoration = container.decoration as BoxDecoration?;
      expect(decoration?.boxShadow, isNull);

      // During drag
      await tester.startGesture(const Offset(160, 140));
      await tester.pump();
      await tester.pump(
        const Duration(milliseconds: 200),
      ); // Wait for animation

      container = tester.widget<AnimatedContainer>(
        find.byType(AnimatedContainer),
      );
      decoration = container.decoration as BoxDecoration?;
      expect(decoration?.boxShadow, isNotNull);
      expect(decoration!.boxShadow!.length, 1);
      expect(decoration.boxShadow![0].blurRadius, 16);
      expect(decoration.boxShadow![0].offset, const Offset(0, 8));
    });

    testWidgets('scales up slightly during drag', (tester) async {
      await tester.pumpWidget(
        buildTestWidget(
          nodeId: 'node-1',
          initialPosition: const Offset(100, 100),
        ),
      );

      // Before drag
      var transform = tester.widget<Transform>(find.byType(Transform));
      expect(transform.transform.getMaxScaleOnAxis(), 1.0);

      // During drag
      await tester.startGesture(const Offset(160, 140));
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 200));

      transform = tester.widget<Transform>(find.byType(Transform));
      expect(transform.transform.getMaxScaleOnAxis(), 1.02);
    });

    testWidgets('cursor changes on hover', (tester) async {
      await tester.pumpWidget(
        buildTestWidget(
          nodeId: 'node-1',
          initialPosition: const Offset(100, 100),
        ),
      );

      // Hover over node
      final gesture = await tester.createGesture(kind: PointerDeviceKind.mouse);
      await gesture.addPointer(location: const Offset(160, 140));
      await tester.pump();

      // MouseRegion should show grab cursor on hover
      final mouseRegion = tester.widget<MouseRegion>(find.byType(MouseRegion));
      expect(mouseRegion.cursor, SystemMouseCursors.grab);

      await gesture.removePointer();
    });

    testWidgets('cursor changes to grabbing during drag', (tester) async {
      await tester.pumpWidget(
        buildTestWidget(
          nodeId: 'node-1',
          initialPosition: const Offset(100, 100),
        ),
      );

      // Start drag
      await tester.startGesture(const Offset(160, 140));
      await tester.pump();

      // Cursor should change to grabbing
      final mouseRegion = tester.widget<MouseRegion>(find.byType(MouseRegion));
      expect(mouseRegion.cursor, SystemMouseCursors.grabbing);
    });

    testWidgets('multiple nodes can be tracked independently', (tester) async {
      await tester.pumpWidget(
        ProviderScope(
          overrides: overrides,
          child: MaterialApp(
            home: Scaffold(
              body: Stack(
                children: [
                  DraggableNode(
                    nodeId: 'node-1',
                    initialPosition: const Offset(100, 100),
                    size: const Size(120, 80),
                    canvasBounds: const Size(800, 600),
                    child: Container(
                      width: 120,
                      height: 80,
                      color: Colors.blue,
                      child: const Text('Node 1'),
                    ),
                  ),
                  DraggableNode(
                    nodeId: 'node-2',
                    initialPosition: const Offset(300, 200),
                    size: const Size(120, 80),
                    canvasBounds: const Size(800, 600),
                    child: Container(
                      width: 120,
                      height: 80,
                      color: Colors.red,
                      child: const Text('Node 2'),
                    ),
                  ),
                ],
              ),
            ),
          ),
        ),
      );

      // Drag first node
      final gesture1 = await tester.startGesture(const Offset(160, 140));
      await gesture1.moveBy(const Offset(50, 50));
      await tester.pump();
      await gesture1.up();
      await tester.pumpAndSettle();

      expect(positions['node-1'], const Offset(150, 150));
      expect(positions.containsKey('node-2'), isFalse);

      // Drag second node
      final gesture2 = await tester.startGesture(const Offset(360, 240));
      await gesture2.moveBy(const Offset(-50, 30));
      await tester.pump();
      await gesture2.up();
      await tester.pumpAndSettle();

      expect(positions['node-2'], const Offset(250, 230));
      expect(positions['node-1'], const Offset(150, 150)); // unchanged
    });

    testWidgets('drag with zero delta does not change position', (
      tester,
    ) async {
      await tester.pumpWidget(
        buildTestWidget(
          nodeId: 'node-1',
          initialPosition: const Offset(100, 100),
        ),
      );

      final gesture = await tester.startGesture(const Offset(160, 140));
      await tester.pump();
      await gesture.up();
      await tester.pumpAndSettle();

      // Position should be unchanged
      expect(positions['node-1'], const Offset(100, 100));
    });
  });
}

/// Fake node positions provider for testing.
class _FakeNodePositions extends NodePositions {
  _FakeNodePositions(this.positions);

  final Map<String, Offset> positions;

  @override
  Future<Map<String, NodePosition>> build() async => positions.map(
    (key, value) => MapEntry(
      key,
      NodePosition(
        x: value.dx,
        y: value.dy,
        isManual: true,
        updatedAt: DateTime.now(),
      ),
    ),
  );

  @override
  Future<void> setPosition(
    String nodeId,
    Offset position, {
    bool manual = true,
  }) async {
    positions[nodeId] = position;
    state = AsyncValue.data(
      positions.map(
        (key, value) => MapEntry(
          key,
          NodePosition(
            x: value.dx,
            y: value.dy,
            isManual: true,
            updatedAt: DateTime.now(),
          ),
        ),
      ),
    );
  }
}
