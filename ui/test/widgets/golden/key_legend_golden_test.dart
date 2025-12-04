import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/widgets/editor/key_legend.dart';

void main() {
  group('KeyLegend Golden Tests', () {
    final testItems = [
      const LegendItem(label: 'Standard', color: Colors.blue),
      const LegendItem(label: 'Modified', color: Colors.orange),
      const LegendItem(label: 'Error', color: Colors.red),
      const LegendItem(label: 'Success', color: Colors.green),
    ];

    testWidgets('renders horizontal layout', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          theme: ThemeData.dark(),
          home: Scaffold(
            backgroundColor: Colors.black,
            body: Center(
              child: KeyLegend(
                items: testItems,
                orientation: Axis.horizontal,
              ),
            ),
          ),
        ),
      );

      await expectLater(
        find.byType(KeyLegend),
        matchesGoldenFile('goldens/key_legend_horizontal.png'),
      );
    });

    testWidgets('renders vertical layout', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          theme: ThemeData.dark(),
          home: Scaffold(
            backgroundColor: Colors.black,
            body: Center(
              child: KeyLegend(
                items: testItems,
                orientation: Axis.vertical,
              ),
            ),
          ),
        ),
      );

      await expectLater(
        find.byType(KeyLegend),
        matchesGoldenFile('goldens/key_legend_vertical.png'),
      );
    });

    testWidgets('renders with icons', (WidgetTester tester) async {
      final itemsWithIcons = [
        const LegendItem(
          label: 'Locked',
          color: Colors.grey,
          icon: Icons.lock,
        ),
        const LegendItem(
          label: 'Unlocked',
          color: Colors.green,
          icon: Icons.lock_open,
        ),
        const LegendItem(
          label: 'Warning',
          color: Colors.orange,
          icon: Icons.warning,
        ),
      ];

      await tester.pumpWidget(
        MaterialApp(
          theme: ThemeData.dark(),
          home: Scaffold(
            backgroundColor: Colors.black,
            body: Center(
              child: KeyLegend(
                items: itemsWithIcons,
                orientation: Axis.horizontal,
              ),
            ),
          ),
        ),
      );

      await expectLater(
        find.byType(KeyLegend),
        matchesGoldenFile('goldens/key_legend_with_icons.png'),
      );
    });

    testWidgets('renders with tooltips', (WidgetTester tester) async {
      final itemsWithTooltips = [
        const LegendItem(
          label: 'Active',
          color: Colors.blue,
          tooltip: 'This key is currently active',
        ),
        const LegendItem(
          label: 'Inactive',
          color: Colors.grey,
          tooltip: 'This key is not active',
        ),
      ];

      await tester.pumpWidget(
        MaterialApp(
          theme: ThemeData.dark(),
          home: Scaffold(
            backgroundColor: Colors.black,
            body: Center(
              child: KeyLegend(
                items: itemsWithTooltips,
                orientation: Axis.horizontal,
              ),
            ),
          ),
        ),
      );

      await expectLater(
        find.byType(KeyLegend),
        matchesGoldenFile('goldens/key_legend_with_tooltips.png'),
      );
    });

    testWidgets('renders in light theme', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          theme: ThemeData.light(),
          home: Scaffold(
            backgroundColor: Colors.white,
            body: Center(
              child: KeyLegend(
                items: testItems,
                orientation: Axis.horizontal,
              ),
            ),
          ),
        ),
      );

      await expectLater(
        find.byType(KeyLegend),
        matchesGoldenFile('goldens/key_legend_light_theme.png'),
      );
    });

    testWidgets('renders many items with wrapping', (WidgetTester tester) async {
      final manyItems = List.generate(
        10,
        (i) => LegendItem(
          label: 'Item $i',
          color: Colors.primaries[i % Colors.primaries.length],
        ),
      );

      await tester.pumpWidget(
        MaterialApp(
          theme: ThemeData.dark(),
          home: Scaffold(
            backgroundColor: Colors.black,
            body: Center(
              child: SizedBox(
                width: 400,
                child: KeyLegend(
                  items: manyItems,
                  orientation: Axis.horizontal,
                ),
              ),
            ),
          ),
        ),
      );

      await expectLater(
        find.byType(KeyLegend),
        matchesGoldenFile('goldens/key_legend_wrapping.png'),
      );
    });
  });
}
