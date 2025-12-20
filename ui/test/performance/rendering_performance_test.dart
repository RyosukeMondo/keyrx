/// Performance tests for KeyFlow canvas rendering.
///
/// Tests rendering performance with large numbers of nodes and edges
/// to ensure 60fps target is met.
library;

import 'package:core_logic/models/flow/edge_style.dart';
import 'package:core_logic/models/flow/flow_graph.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/utils/rendering_optimizer.dart';
import 'package:keyrx_ui/widgets/flow/edges_painter.dart';
import 'package:keyrx_ui/widgets/flow/optimized_flow_canvas.dart';

void main() {
  group('Rendering Performance', () {
    test('viewport calculation performance', () {
      final transform = Matrix4.identity();
      const screenSize = Size(1920, 1080);

      final stopwatch = Stopwatch()..start();
      for (var i = 0; i < 1000; i++) {
        calculateViewport(transform: transform, screenSize: screenSize);
      }
      stopwatch.stop();

      final avgTime = stopwatch.elapsedMicroseconds / 1000;

      // Should be very fast - less than 100 microseconds
      expect(
        avgTime,
        lessThan(100),
        reason: 'Viewport calculation should be under 100μs',
      );
    });

    test('node visibility check performance', () {
      const viewport = Rect.fromLTWH(0, 0, 1920, 1080);
      const nodePosition = Offset(100, 100);
      const nodeSize = Size(200, 150);

      final stopwatch = Stopwatch()..start();
      for (var i = 0; i < 10000; i++) {
        isNodeInViewport(
          nodePosition: nodePosition,
          nodeSize: nodeSize,
          viewport: viewport,
        );
      }
      stopwatch.stop();

      final avgTime = stopwatch.elapsedMicroseconds / 10000;

      // Should be extremely fast - less than 10 microseconds
      expect(
        avgTime,
        lessThan(10),
        reason: 'Node visibility check should be under 10μs',
      );
    });

    test('edge path cache key equality performance', () {
      const key1 = EdgePathCacheKey(
        startX: 100,
        startY: 200,
        endX: 300,
        endY: 400,
        active: true,
      );
      const key2 = EdgePathCacheKey(
        startX: 100,
        startY: 200,
        endX: 300,
        endY: 400,
        active: true,
      );

      final stopwatch = Stopwatch()..start();
      for (var i = 0; i < 100000; i++) {
        // ignore: unnecessary_statements
        key1 == key2;
      }
      stopwatch.stop();

      final avgTime = stopwatch.elapsedMicroseconds / 100000;

      // Should be very fast - less than 1 microsecond
      expect(
        avgTime,
        lessThan(1),
        reason: 'Cache key equality should be under 1μs',
      );
    });

    test('frame timer accuracy', () {
      final timer = FrameTimer();

      timer.start();
      // Simulate some work
      var sum = 0;
      for (var i = 0; i < 1000; i++) {
        sum += i;
      }
      final elapsed = timer.stop();

      // Use sum to prevent optimization
      expect(sum, greaterThan(0));

      expect(elapsed.inMicroseconds, greaterThan(0));
      expect(
        elapsed.inMilliseconds,
        lessThan(100),
        reason: 'Simple loop should complete quickly',
      );
    });

    test('rendering metrics calculation', () {
      const metrics = RenderingMetrics(
        totalNodes: 250,
        visibleNodes: 50,
        totalEdges: 500,
        visibleEdges: 100,
        frameTime: Duration(milliseconds: 12),
      );

      expect(
        metrics.meetsPerformanceTarget,
        isTrue,
        reason: '12ms frame time should meet 60fps target',
      );
      expect(
        metrics.virtualizationEfficiency,
        closeTo(0.8, 0.01),
        reason: '80% of nodes should be virtualized',
      );
    });

    test('rendering metrics with poor performance', () {
      const metrics = RenderingMetrics(
        totalNodes: 250,
        visibleNodes: 50,
        totalEdges: 500,
        visibleEdges: 100,
        frameTime: Duration(milliseconds: 20),
      );

      expect(
        metrics.meetsPerformanceTarget,
        isFalse,
        reason: '20ms frame time should exceed 60fps target',
      );
    });

    test('rebuild debouncer timing', () {
      final debouncer = RebuildDebouncer(
        delay: const Duration(milliseconds: 100),
      );

      expect(
        debouncer.shouldRebuild(),
        isTrue,
        reason: 'First call should always rebuild',
      );
      expect(
        debouncer.shouldRebuild(),
        isFalse,
        reason: 'Immediate second call should be debounced',
      );

      // Wait for debounce period
      return Future<void>.delayed(const Duration(milliseconds: 150), () {
        expect(
          debouncer.shouldRebuild(),
          isTrue,
          reason: 'Should rebuild after debounce period',
        );
      });
    });

    test('large node set virtualization', () {
      // Simulate 250 nodes (50 devices × 5 slots)
      final allNodes = List.generate(
        250,
        (i) => OptimizedCanvasNode(
          id: 'node_$i',
          position: Offset(100.0 * (i % 10), 100.0 * (i ~/ 10)),
          size: const Size(200, 150),
          child: Container(),
        ),
      );

      const viewport = Rect.fromLTWH(0, 0, 1920, 1080);

      final stopwatch = Stopwatch()..start();
      final visibleNodes = allNodes
          .where(
            (node) => isNodeInViewport(
              nodePosition: node.position,
              nodeSize: node.size,
              viewport: viewport,
              margin: 200,
            ),
          )
          .toList();
      stopwatch.stop();

      // Virtualization should filter out most nodes
      expect(
        visibleNodes.length,
        lessThan(allNodes.length),
        reason: 'Should virtualize off-screen nodes',
      );

      // Should complete very quickly
      expect(
        stopwatch.elapsedMilliseconds,
        lessThan(5),
        reason: 'Virtualization of 250 nodes should take <5ms',
      );

      print(
        'Virtualized ${allNodes.length} nodes to ${visibleNodes.length} '
        'in ${stopwatch.elapsedMicroseconds}μs',
      );
    });

    test('edge path cache reduces computation', () {
      final cache = <EdgePathCacheKey, Path>{};
      final edges = List.generate(
        500,
        (i) => PaintableEdge(
          start: Offset(100.0 * i, 200),
          end: Offset(100.0 * i + 50, 400),
          status: EdgeStatus.valid,
          active: i % 2 == 0,
          priority: i % 5,
        ),
      );

      // First pass - populate cache
      final stopwatch1 = Stopwatch()..start();
      for (final edge in edges) {
        final key = EdgePathCacheKey(
          startX: edge.start.dx,
          startY: edge.start.dy,
          endX: edge.end.dx,
          endY: edge.end.dy,
          active: edge.active,
        );
        if (!cache.containsKey(key)) {
          // Simulate path computation
          cache[key] = Path()..moveTo(edge.start.dx, edge.start.dy);
        }
      }
      final time1 = stopwatch1.elapsedMicroseconds;

      // Second pass - use cache
      final stopwatch2 = Stopwatch()..start();
      for (final edge in edges) {
        final key = EdgePathCacheKey(
          startX: edge.start.dx,
          startY: edge.start.dy,
          endX: edge.end.dx,
          endY: edge.end.dy,
          active: edge.active,
        );
        final _ = cache[key];
      }
      final time2 = stopwatch2.elapsedMicroseconds;

      // Cached access should be much faster
      expect(
        time2,
        lessThan(time1 / 2),
        reason: 'Cached path access should be 50% faster',
      );

      print(
        'Edge path cache: first=$time1μs, cached=$time2μs, '
        'speedup=${(time1 / time2).toStringAsFixed(2)}x',
      );
    });

    testWidgets('optimized edge layer renders without errors', (tester) async {
      final edges = List.generate(
        100,
        (i) => PaintableEdge(
          start: Offset(100.0 * i, 200),
          end: Offset(100.0 * i + 50, 400),
          status: EdgeStatus.valid,
          active: i % 2 == 0,
          priority: i % 5,
        ),
      );

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(
              width: 2000,
              height: 2000,
              child: OptimizedEdgeLayer(
                edges: edges,
                style: const EdgeStyle(palette: defaultEdgePalette),
              ),
            ),
          ),
        ),
      );

      // Should render without errors
      expect(find.byType(OptimizedEdgeLayer), findsOneWidget);
    });
  });

  group('Performance Benchmarks', () {
    test('target: 250 nodes virtualization <5ms', () {
      final nodes = List.generate(
        250,
        (i) => OptimizedCanvasNode(
          id: 'node_$i',
          position: Offset(100.0 * (i % 10), 100.0 * (i ~/ 10)),
          size: const Size(200, 150),
          child: Container(),
        ),
      );

      const viewport = Rect.fromLTWH(0, 0, 1920, 1080);

      final stopwatch = Stopwatch()..start();
      final visible = nodes
          .where(
            (node) => isNodeInViewport(
              nodePosition: node.position,
              nodeSize: node.size,
              viewport: viewport,
              margin: 200,
            ),
          )
          .toList();
      stopwatch.stop();

      expect(stopwatch.elapsedMilliseconds, lessThan(5));
      print(
        '✓ 250 nodes virtualized in ${stopwatch.elapsedMicroseconds}μs '
        '(${visible.length} visible)',
      );
    });

    test('target: 500 edge cache lookups <1ms', () {
      final cache = <EdgePathCacheKey, Path>{};

      // Populate cache
      for (var i = 0; i < 500; i++) {
        cache[EdgePathCacheKey(
              startX: i.toDouble(),
              startY: i.toDouble(),
              endX: i.toDouble() + 100,
              endY: i.toDouble() + 100,
              active: true,
            )] =
            Path();
      }

      final stopwatch = Stopwatch()..start();
      for (var i = 0; i < 500; i++) {
        final _ =
            cache[EdgePathCacheKey(
              startX: i.toDouble(),
              startY: i.toDouble(),
              endX: i.toDouble() + 100,
              endY: i.toDouble() + 100,
              active: true,
            )];
      }
      stopwatch.stop();

      expect(stopwatch.elapsedMilliseconds, lessThan(1));
      print('✓ 500 edge cache lookups in ${stopwatch.elapsedMicroseconds}μs');
    });
  });
}
