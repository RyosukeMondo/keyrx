import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/models/flow/edge_style.dart';
import 'package:keyrx_ui/models/flow/flow_graph.dart';
import 'package:keyrx_ui/widgets/flow/edges_painter.dart';
import 'package:keyrx_ui/widgets/flow/interactive_edge.dart';

void main() {
  group('InteractiveEdgeCanvas', () {
    testWidgets('renders without errors', (tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: Scaffold(
              body: InteractiveEdgeCanvas(
                edges: const [],
                paintableEdges: const [],
                style: const EdgeStyle(palette: defaultEdgePalette),
              ),
            ),
          ),
        ),
      );

      expect(find.byType(InteractiveEdgeCanvas), findsOneWidget);
    });

    testWidgets('detects mouse hover', (tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: Scaffold(
              body: SizedBox(
                width: 400,
                height: 400,
                child: InteractiveEdgeCanvas(
                  edges: const [],
                  paintableEdges: const [],
                  style: const EdgeStyle(palette: defaultEdgePalette),
                ),
              ),
            ),
          ),
        ),
      );

      final gesture = await tester.createGesture(kind: PointerDeviceKind.mouse);
      await gesture.addPointer(location: const Offset(200, 200));
      await tester.pump();
      await gesture.moveTo(const Offset(250, 250));
      await tester.pump();

      expect(find.byType(InteractiveEdgeCanvas), findsOneWidget);
    });

    testWidgets('handles tap on empty canvas', (tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            home: Scaffold(
              body: InteractiveEdgeCanvas(
                edges: const [],
                paintableEdges: const [],
                style: const EdgeStyle(palette: defaultEdgePalette),
              ),
            ),
          ),
        ),
      );

      await tester.tap(find.byType(InteractiveEdgeCanvas));
      await tester.pump();

      expect(find.byType(InteractiveEdgeCanvas), findsOneWidget);
    });
  });
}
