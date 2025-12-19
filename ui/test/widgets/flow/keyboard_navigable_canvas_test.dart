import 'package:core_logic/models/flow/flow_graph.dart';
import 'package:core_logic/models/hardware_profile.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/state/keyflow_providers.dart';
import 'package:keyrx_ui/state/selection_state.dart';
import 'package:keyrx_ui/widgets/flow/keyboard_navigable_canvas.dart';

void main() {
  group('KeyboardNavigableCanvas', () {
    late TransformationController transformationController;

    setUp(() {
      transformationController = TransformationController();
    });

    tearDown(() {
      transformationController.dispose();
    });

    testWidgets('renders child widget', (tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: KeyboardNavigableCanvas(
              transformationController: transformationController,
              canvasBounds: const Size(1000, 1000),
              child: const Text('Test Child'),
            ),
          ),
        ),
      );

      expect(find.text('Test Child'), findsOneWidget);
    });

    testWidgets('Tab key cycles to next node', (tester) async {
      final nodes = [
        const FlowNode.device(
          id: 'device-1',
          deviceId: DeviceInstanceId(vendorId: 1, productId: 1, serial: '1'),
          label: 'Device 1',
        ),
        const FlowNode.device(
          id: 'device-2',
          deviceId: DeviceInstanceId(vendorId: 1, productId: 2, serial: '2'),
          label: 'Device 2',
        ),
      ];

      final graph = FlowGraph(nodes: nodes);

      await tester.pumpWidget(
        ProviderScope(
          overrides: [flowGraphProvider.overrideWith((ref) => graph)],
          child: MaterialApp(
            home: KeyboardNavigableCanvas(
              transformationController: transformationController,
              canvasBounds: const Size(1000, 1000),
              child: const SizedBox(),
            ),
          ),
        ),
      );

      // Allow focus tracker to initialize
      await tester.pumpAndSettle();

      // Press Tab key
      await tester.sendKeyEvent(LogicalKeyboardKey.tab);
      await tester.pumpAndSettle();

      // Verify focus moved to next node (wrapped around to first node)
      final container = tester.element(find.byType(KeyboardNavigableCanvas));
      final providerContainer = ProviderScope.containerOf(container);
      final tracker = providerContainer.read(focusTrackerProvider);

      expect(tracker.focusedIndex, 1);
    });

    testWidgets('Shift+Tab cycles to previous node', (tester) async {
      final nodes = [
        const FlowNode.device(
          id: 'device-1',
          deviceId: DeviceInstanceId(vendorId: 1, productId: 1, serial: '1'),
          label: 'Device 1',
        ),
        const FlowNode.device(
          id: 'device-2',
          deviceId: DeviceInstanceId(vendorId: 1, productId: 2, serial: '2'),
          label: 'Device 2',
        ),
      ];

      final graph = FlowGraph(nodes: nodes);

      await tester.pumpWidget(
        ProviderScope(
          overrides: [flowGraphProvider.overrideWith((ref) => graph)],
          child: MaterialApp(
            home: KeyboardNavigableCanvas(
              transformationController: transformationController,
              canvasBounds: const Size(1000, 1000),
              child: const SizedBox(),
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      // Press Shift+Tab
      await tester.sendKeyDownEvent(LogicalKeyboardKey.shift);
      await tester.sendKeyEvent(LogicalKeyboardKey.tab);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.shift);
      await tester.pumpAndSettle();

      // Verify focus moved to previous node (wrapped to last)
      final container = tester.element(find.byType(KeyboardNavigableCanvas));
      final providerContainer = ProviderScope.containerOf(container);
      final tracker = providerContainer.read(focusTrackerProvider);

      expect(tracker.focusedIndex, 1);
    });

    testWidgets('Escape key clears selection', (tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: KeyboardNavigableCanvas(
              transformationController: transformationController,
              canvasBounds: const Size(1000, 1000),
              child: const SizedBox(),
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      // Get selection notifier and select a node
      final container = tester.element(find.byType(KeyboardNavigableCanvas));
      final providerContainer = ProviderScope.containerOf(container);
      final selection = providerContainer.read(
        selectionNotifierProvider.notifier,
      );
      selection.selectNode('device-1');
      await tester.pumpAndSettle();

      // Verify initial selection
      expect(
        providerContainer.read(selectionNotifierProvider),
        isA<NodeSelection>(),
      );

      // Press Escape
      await tester.sendKeyEvent(LogicalKeyboardKey.escape);
      await tester.pumpAndSettle();

      // Verify selection cleared
      expect(
        providerContainer.read(selectionNotifierProvider),
        isA<NoSelection>(),
      );
    });

    testWidgets('Ctrl+0 resets zoom to 100%', (tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: KeyboardNavigableCanvas(
              transformationController: transformationController,
              canvasBounds: const Size(1000, 1000),
              child: const SizedBox(),
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      // Set zoom to 2x
      transformationController.value = Matrix4.identity()..scale(2.0);
      await tester.pumpAndSettle();

      // Verify zoom is 2x
      expect(transformationController.value.getMaxScaleOnAxis(), 2.0);

      // Press Ctrl+0
      await tester.sendKeyDownEvent(LogicalKeyboardKey.control);
      await tester.sendKeyEvent(LogicalKeyboardKey.digit0);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.control);
      await tester.pumpAndSettle();

      // Verify zoom reset to 1.0
      expect(transformationController.value.getMaxScaleOnAxis(), 1.0);
    });

    testWidgets('Ctrl++ zooms in by 10%', (tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: KeyboardNavigableCanvas(
              transformationController: transformationController,
              canvasBounds: const Size(1000, 1000),
              child: const SizedBox(),
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      final initialScale = transformationController.value.getMaxScaleOnAxis();

      // Press Ctrl++
      await tester.sendKeyDownEvent(LogicalKeyboardKey.control);
      await tester.sendKeyEvent(LogicalKeyboardKey.equal);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.control);
      await tester.pumpAndSettle();

      // Verify zoom increased by ~10%
      final newScale = transformationController.value.getMaxScaleOnAxis();
      expect(newScale, greaterThan(initialScale));
      expect(newScale / initialScale, closeTo(1.1, 0.01));
    });

    testWidgets('Ctrl+- zooms out by 10%', (tester) async {
      // Start with 2x zoom
      transformationController.value = Matrix4.identity()..scale(2.0);

      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: KeyboardNavigableCanvas(
              transformationController: transformationController,
              canvasBounds: const Size(1000, 1000),
              child: const SizedBox(),
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      final initialScale = transformationController.value.getMaxScaleOnAxis();

      // Press Ctrl+-
      await tester.sendKeyDownEvent(LogicalKeyboardKey.control);
      await tester.sendKeyEvent(LogicalKeyboardKey.minus);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.control);
      await tester.pumpAndSettle();

      // Verify zoom decreased by ~10%
      final newScale = transformationController.value.getMaxScaleOnAxis();
      expect(newScale, lessThan(initialScale));
      expect(newScale / initialScale, closeTo(0.9, 0.01));
    });

    testWidgets('Ctrl+? triggers help callback', (tester) async {
      var helpCalled = false;

      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: KeyboardNavigableCanvas(
              transformationController: transformationController,
              canvasBounds: const Size(1000, 1000),
              onHelpRequested: () => helpCalled = true,
              child: const SizedBox(),
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      // Press Ctrl+?
      await tester.sendKeyDownEvent(LogicalKeyboardKey.control);
      await tester.sendKeyEvent(LogicalKeyboardKey.slash);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.control);
      await tester.pumpAndSettle();

      // Verify help callback was called
      expect(helpCalled, isTrue);
    });

    testWidgets('Enter key triggers node details callback', (tester) async {
      String? requestedNodeId;

      final nodes = [
        const FlowNode.device(
          id: 'device-1',
          deviceId: DeviceInstanceId(vendorId: 1, productId: 1, serial: '1'),
          label: 'Device 1',
        ),
      ];

      final graph = FlowGraph(nodes: nodes);

      await tester.pumpWidget(
        ProviderScope(
          overrides: [flowGraphProvider.overrideWith((ref) => graph)],
          child: MaterialApp(
            home: KeyboardNavigableCanvas(
              transformationController: transformationController,
              canvasBounds: const Size(1000, 1000),
              onNodeDetailsRequested: (nodeId) => requestedNodeId = nodeId,
              child: const SizedBox(),
            ),
          ),
        ),
      );

      await tester.pumpAndSettle();

      // Press Enter
      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
      await tester.pumpAndSettle();

      // Verify node details callback was called with focused node
      expect(requestedNodeId, 'device-1');
    });
  });

  group('FocusTracker', () {
    test('initializes with first node focused', () {
      final nodes = [
        const FlowNode.device(
          id: 'device-1',
          deviceId: DeviceInstanceId(vendorId: 1, productId: 1, serial: '1'),
          label: 'D1',
        ),
        const FlowNode.device(
          id: 'device-2',
          deviceId: DeviceInstanceId(vendorId: 1, productId: 2, serial: '2'),
          label: 'D2',
        ),
      ];

      final tracker = FocusTracker(nodes: nodes);

      expect(tracker.focusedIndex, 0);
      expect(tracker.focusedNodeId, 'device-1');
    });

    test('focusNext cycles through nodes', () {
      final nodes = [
        const FlowNode.device(
          id: 'device-1',
          deviceId: DeviceInstanceId(vendorId: 1, productId: 1, serial: '1'),
          label: 'D1',
        ),
        const FlowNode.device(
          id: 'device-2',
          deviceId: DeviceInstanceId(vendorId: 1, productId: 2, serial: '2'),
          label: 'D2',
        ),
        const FlowNode.device(
          id: 'device-3',
          deviceId: DeviceInstanceId(vendorId: 1, productId: 3, serial: '3'),
          label: 'D3',
        ),
      ];

      final tracker = FocusTracker(nodes: nodes);

      tracker.focusNext();
      expect(tracker.focusedIndex, 1);
      expect(tracker.focusedNodeId, 'device-2');

      tracker.focusNext();
      expect(tracker.focusedIndex, 2);
      expect(tracker.focusedNodeId, 'device-3');

      // Should wrap around
      tracker.focusNext();
      expect(tracker.focusedIndex, 0);
      expect(tracker.focusedNodeId, 'device-1');
    });

    test('focusPrevious cycles backwards through nodes', () {
      final nodes = [
        const FlowNode.device(
          id: 'device-1',
          deviceId: DeviceInstanceId(vendorId: 1, productId: 1, serial: '1'),
          label: 'D1',
        ),
        const FlowNode.device(
          id: 'device-2',
          deviceId: DeviceInstanceId(vendorId: 1, productId: 2, serial: '2'),
          label: 'D2',
        ),
        const FlowNode.device(
          id: 'device-3',
          deviceId: DeviceInstanceId(vendorId: 1, productId: 3, serial: '3'),
          label: 'D3',
        ),
      ];

      final tracker = FocusTracker(nodes: nodes);

      // Should wrap around to last
      tracker.focusPrevious();
      expect(tracker.focusedIndex, 2);
      expect(tracker.focusedNodeId, 'device-3');

      tracker.focusPrevious();
      expect(tracker.focusedIndex, 1);
      expect(tracker.focusedNodeId, 'device-2');
    });

    test('handles empty node list gracefully', () {
      final tracker = FocusTracker(nodes: const []);

      expect(tracker.focusedNodeId, isNull);

      // Should not crash
      tracker.focusNext();
      tracker.focusPrevious();
    });

    test('isNodeFocused returns correct value', () {
      final nodes = [
        const FlowNode.device(
          id: 'device-1',
          deviceId: DeviceInstanceId(vendorId: 1, productId: 1, serial: '1'),
          label: 'D1',
        ),
        const FlowNode.device(
          id: 'device-2',
          deviceId: DeviceInstanceId(vendorId: 1, productId: 2, serial: '2'),
          label: 'D2',
        ),
      ];

      final tracker = FocusTracker(nodes: nodes);

      expect(tracker.isNodeFocused('device-1'), isTrue);
      expect(tracker.isNodeFocused('device-2'), isFalse);
    });
  });

  group('ZoomController', () {
    late TransformationController transformationController;
    late ZoomController zoomController;

    setUp(() {
      transformationController = TransformationController();
      zoomController = ZoomController(
        transformationController: transformationController,
      );
    });

    tearDown(() {
      transformationController.dispose();
    });

    test('resetZoom sets scale to 1.0', () {
      transformationController.value = Matrix4.identity()..scale(2.5);
      expect(transformationController.value.getMaxScaleOnAxis(), 2.5);

      zoomController.resetZoom();
      expect(transformationController.value.getMaxScaleOnAxis(), 1.0);
    });

    test('zoomIn increases scale by 10%', () {
      transformationController.value = Matrix4.identity();
      final initialScale = transformationController.value.getMaxScaleOnAxis();

      zoomController.zoomIn();

      final newScale = transformationController.value.getMaxScaleOnAxis();
      expect(newScale / initialScale, closeTo(1.1, 0.01));
    });

    test('zoomOut decreases scale by 10%', () {
      transformationController.value = Matrix4.identity()..scale(2.0);
      final initialScale = transformationController.value.getMaxScaleOnAxis();

      zoomController.zoomOut();

      final newScale = transformationController.value.getMaxScaleOnAxis();
      expect(newScale / initialScale, closeTo(0.9, 0.01));
    });

    test('zoom is clamped to min 0.5', () {
      transformationController.value = Matrix4.identity()..scale(0.5);

      zoomController.zoomOut();

      // Should stay at 0.5 minimum
      expect(transformationController.value.getMaxScaleOnAxis(), 0.5);
    });

    test('zoom is clamped to max 2.5', () {
      transformationController.value = Matrix4.identity()..scale(2.5);

      zoomController.zoomIn();

      // Should stay at 2.5 maximum
      expect(transformationController.value.getMaxScaleOnAxis(), 2.5);
    });
  });
}
