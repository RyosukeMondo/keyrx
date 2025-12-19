import 'package:core_logic/models/flow/edge_style.dart';
import 'package:core_logic/models/flow/flow_graph.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:keyrx_ui/widgets/flow/edges_painter.dart';

import '../../helpers/golden_test_harness.dart';
import '../../helpers/widget_test_harness.dart';

void main() {
  final darkHarness = GoldenTestHarness();
  final lightHarness = GoldenTestHarness(
    widgetHarness: WidgetTestHarness(
      theme: ThemeData.light(useMaterial3: true),
    ),
  );

  Widget edgeShell(List<PaintableEdge> edges, {Color? backgroundColor}) =>
      Scaffold(
        backgroundColor: backgroundColor ?? Colors.black,
        body: Center(
          child: SizedBox(
            width: 400,
            height: 300,
            child: CustomPaint(
              painter: EdgesPainter(
                edges: edges,
                style: const EdgeStyle(palette: defaultEdgePalette),
                hoverPosition: null,
              ),
            ),
          ),
        ),
      );

  group('Edge Rendering Goldens', () {
    testGoldens('renders edge - active valid', (tester) async {
      final edges = [
        const PaintableEdge(
          start: Offset(50, 150),
          end: Offset(350, 150),
          status: EdgeStatus.valid,
          active: true,
          priority: 1,
        ),
      ];

      await darkHarness.pump(tester, edgeShell(edges));
      await darkHarness.expectGolden(tester, 'edge_active_valid');
    });

    testGoldens('renders edge - inactive valid', (tester) async {
      final edges = [
        const PaintableEdge(
          start: Offset(50, 150),
          end: Offset(350, 150),
          status: EdgeStatus.valid,
          active: false,
          priority: 2,
        ),
      ];

      await darkHarness.pump(tester, edgeShell(edges));
      await darkHarness.expectGolden(tester, 'edge_inactive_valid');
    });

    testGoldens('renders edge - active invalid', (tester) async {
      final edges = [
        const PaintableEdge(
          start: Offset(50, 150),
          end: Offset(350, 150),
          status: EdgeStatus.invalid,
          active: true,
          priority: 1,
        ),
      ];

      await darkHarness.pump(tester, edgeShell(edges));
      await darkHarness.expectGolden(tester, 'edge_active_invalid');
    });

    testGoldens('renders edge - inactive invalid', (tester) async {
      final edges = [
        const PaintableEdge(
          start: Offset(50, 150),
          end: Offset(350, 150),
          status: EdgeStatus.invalid,
          active: false,
          priority: 3,
        ),
      ];

      await darkHarness.pump(tester, edgeShell(edges));
      await darkHarness.expectGolden(tester, 'edge_inactive_invalid');
    });

    testGoldens('renders edge - active warning', (tester) async {
      final edges = [
        const PaintableEdge(
          start: Offset(50, 150),
          end: Offset(350, 150),
          status: EdgeStatus.warning,
          active: true,
          priority: 2,
        ),
      ];

      await darkHarness.pump(tester, edgeShell(edges));
      await darkHarness.expectGolden(tester, 'edge_active_warning');
    });

    testGoldens('renders edge - inactive warning', (tester) async {
      final edges = [
        const PaintableEdge(
          start: Offset(50, 150),
          end: Offset(350, 150),
          status: EdgeStatus.warning,
          active: false,
          priority: 4,
        ),
      ];

      await darkHarness.pump(tester, edgeShell(edges));
      await darkHarness.expectGolden(tester, 'edge_inactive_warning');
    });

    testGoldens('renders edge - curved path', (tester) async {
      final edges = [
        const PaintableEdge(
          start: Offset(50, 50),
          end: Offset(350, 250),
          status: EdgeStatus.valid,
          active: true,
          priority: 1,
        ),
      ];

      await darkHarness.pump(tester, edgeShell(edges));
      await darkHarness.expectGolden(tester, 'edge_curved');
    });

    testGoldens('renders edge - priority badges', (tester) async {
      final edges = [
        const PaintableEdge(
          start: Offset(50, 60),
          end: Offset(350, 60),
          status: EdgeStatus.valid,
          active: true,
          priority: 1,
        ),
        const PaintableEdge(
          start: Offset(50, 120),
          end: Offset(350, 120),
          status: EdgeStatus.valid,
          active: true,
          priority: 5,
        ),
        const PaintableEdge(
          start: Offset(50, 180),
          end: Offset(350, 180),
          status: EdgeStatus.valid,
          active: true,
          priority: 10,
        ),
        const PaintableEdge(
          start: Offset(50, 240),
          end: Offset(350, 240),
          status: EdgeStatus.valid,
          active: true,
          priority: 99,
        ),
      ];

      await darkHarness.pump(tester, edgeShell(edges));
      await darkHarness.expectGolden(tester, 'edge_priority_badges');
    });

    testGoldens('renders multiple edges - mixed states', (tester) async {
      final edges = [
        const PaintableEdge(
          start: Offset(50, 50),
          end: Offset(350, 70),
          status: EdgeStatus.valid,
          active: true,
          priority: 1,
        ),
        const PaintableEdge(
          start: Offset(50, 100),
          end: Offset(350, 120),
          status: EdgeStatus.valid,
          active: false,
          priority: 2,
        ),
        const PaintableEdge(
          start: Offset(50, 150),
          end: Offset(350, 170),
          status: EdgeStatus.warning,
          active: true,
          priority: 3,
        ),
        const PaintableEdge(
          start: Offset(50, 200),
          end: Offset(350, 220),
          status: EdgeStatus.invalid,
          active: true,
          priority: 4,
        ),
        const PaintableEdge(
          start: Offset(50, 250),
          end: Offset(350, 270),
          status: EdgeStatus.invalid,
          active: false,
          priority: 5,
        ),
      ];

      await darkHarness.pump(tester, edgeShell(edges));
      await darkHarness.expectGolden(tester, 'edges_mixed_states');
    });

    testGoldens('renders edges - overlapping paths', (tester) async {
      final edges = [
        const PaintableEdge(
          start: Offset(50, 100),
          end: Offset(350, 200),
          status: EdgeStatus.valid,
          active: true,
          priority: 1,
        ),
        const PaintableEdge(
          start: Offset(50, 200),
          end: Offset(350, 100),
          status: EdgeStatus.warning,
          active: true,
          priority: 2,
        ),
      ];

      await darkHarness.pump(tester, edgeShell(edges));
      await darkHarness.expectGolden(tester, 'edges_overlapping');
    });

    testGoldens('renders edges - light theme', (tester) async {
      final edges = [
        const PaintableEdge(
          start: Offset(50, 80),
          end: Offset(350, 80),
          status: EdgeStatus.valid,
          active: true,
          priority: 1,
        ),
        const PaintableEdge(
          start: Offset(50, 150),
          end: Offset(350, 150),
          status: EdgeStatus.warning,
          active: true,
          priority: 2,
        ),
        const PaintableEdge(
          start: Offset(50, 220),
          end: Offset(350, 220),
          status: EdgeStatus.invalid,
          active: false,
          priority: 3,
        ),
      ];

      await lightHarness.pump(
        tester,
        edgeShell(edges, backgroundColor: Colors.white),
      );
      await lightHarness.expectGolden(tester, 'edges_light_theme');
    });

    testGoldens('renders edge - vertical', (tester) async {
      final edges = [
        const PaintableEdge(
          start: Offset(200, 50),
          end: Offset(200, 250),
          status: EdgeStatus.valid,
          active: true,
          priority: 1,
        ),
      ];

      await darkHarness.pump(tester, edgeShell(edges));
      await darkHarness.expectGolden(tester, 'edge_vertical');
    });

    testGoldens('renders edge - diagonal', (tester) async {
      final edges = [
        const PaintableEdge(
          start: Offset(50, 50),
          end: Offset(350, 250),
          status: EdgeStatus.valid,
          active: true,
          priority: 1,
        ),
      ];

      await darkHarness.pump(tester, edgeShell(edges));
      await darkHarness.expectGolden(tester, 'edge_diagonal');
    });

    testGoldens('renders edge - short distance', (tester) async {
      final edges = [
        const PaintableEdge(
          start: Offset(180, 150),
          end: Offset(220, 150),
          status: EdgeStatus.valid,
          active: true,
          priority: 1,
        ),
      ];

      await darkHarness.pump(tester, edgeShell(edges));
      await darkHarness.expectGolden(tester, 'edge_short_distance');
    });
  });

  group('Edge Styles Goldens', () {
    testGoldens('renders edge - dashed inactive style', (tester) async {
      final edges = [
        const PaintableEdge(
          start: Offset(50, 80),
          end: Offset(350, 80),
          status: EdgeStatus.valid,
          active: false,
          priority: 1,
        ),
        const PaintableEdge(
          start: Offset(50, 150),
          end: Offset(350, 150),
          status: EdgeStatus.warning,
          active: false,
          priority: 2,
        ),
        const PaintableEdge(
          start: Offset(50, 220),
          end: Offset(350, 220),
          status: EdgeStatus.invalid,
          active: false,
          priority: 3,
        ),
      ];

      await darkHarness.pump(tester, edgeShell(edges));
      await darkHarness.expectGolden(tester, 'edge_dashed_styles');
    });

    testGoldens('renders edge - solid active style', (tester) async {
      final edges = [
        const PaintableEdge(
          start: Offset(50, 80),
          end: Offset(350, 80),
          status: EdgeStatus.valid,
          active: true,
          priority: 1,
        ),
        const PaintableEdge(
          start: Offset(50, 150),
          end: Offset(350, 150),
          status: EdgeStatus.warning,
          active: true,
          priority: 2,
        ),
        const PaintableEdge(
          start: Offset(50, 220),
          end: Offset(350, 220),
          status: EdgeStatus.invalid,
          active: true,
          priority: 3,
        ),
      ];

      await darkHarness.pump(tester, edgeShell(edges));
      await darkHarness.expectGolden(tester, 'edge_solid_styles');
    });
  });
}
