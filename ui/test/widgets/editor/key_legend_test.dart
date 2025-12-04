import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/widgets/editor/key_legend.dart';

void main() {
  group('KeyLegend', () {
    testWidgets('renders legend items horizontally by default',
        (WidgetTester tester) async {
      final items = [
        const LegendItem(label: 'Modified', color: Colors.blue),
        const LegendItem(label: 'Default', color: Colors.grey),
        const LegendItem(label: 'Error', color: Colors.red),
      ];

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: KeyLegend(items: items),
          ),
        ),
      );

      expect(find.text('Modified'), findsOneWidget);
      expect(find.text('Default'), findsOneWidget);
      expect(find.text('Error'), findsOneWidget);
      expect(find.byType(Wrap), findsOneWidget);
    });

    testWidgets('renders legend items vertically when specified',
        (WidgetTester tester) async {
      final items = [
        const LegendItem(label: 'Modified', color: Colors.blue),
        const LegendItem(label: 'Default', color: Colors.grey),
      ];

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: KeyLegend(
              items: items,
              orientation: Axis.vertical,
            ),
          ),
        ),
      );

      expect(find.text('Modified'), findsOneWidget);
      expect(find.text('Default'), findsOneWidget);
      expect(find.byType(Column), findsOneWidget);
    });

    testWidgets('displays color boxes for legend items',
        (WidgetTester tester) async {
      final items = [
        const LegendItem(label: 'Modified', color: Colors.blue),
      ];

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: KeyLegend(items: items),
          ),
        ),
      );

      final containers = find.descendant(
        of: find.byType(Row),
        matching: find.byType(Container),
      );

      expect(containers, findsWidgets);

      final container = tester.widget<Container>(containers.first);
      final decoration = container.decoration as BoxDecoration;
      expect(decoration.color, equals(Colors.blue));
    });

    testWidgets('displays icon instead of color box when icon is provided',
        (WidgetTester tester) async {
      final items = [
        const LegendItem(
          label: 'Warning',
          color: Colors.orange,
          icon: Icons.warning,
        ),
      ];

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: KeyLegend(items: items),
          ),
        ),
      );

      expect(find.byIcon(Icons.warning), findsOneWidget);

      final icon = tester.widget<Icon>(find.byIcon(Icons.warning));
      expect(icon.color, equals(Colors.orange));
    });

    testWidgets('displays tooltip when provided', (WidgetTester tester) async {
      final items = [
        const LegendItem(
          label: 'Modified',
          color: Colors.blue,
          tooltip: 'Keys with custom mappings',
        ),
      ];

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: KeyLegend(items: items),
          ),
        ),
      );

      expect(find.byIcon(Icons.info_outline), findsOneWidget);
      expect(find.byType(Tooltip), findsOneWidget);
    });

    testWidgets('does not display tooltip when not provided',
        (WidgetTester tester) async {
      final items = [
        const LegendItem(
          label: 'Modified',
          color: Colors.blue,
        ),
      ];

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: KeyLegend(items: items),
          ),
        ),
      );

      expect(find.byIcon(Icons.info_outline), findsNothing);
    });

    testWidgets('renders multiple legend items', (WidgetTester tester) async {
      final items = [
        const LegendItem(label: 'Item 1', color: Colors.red),
        const LegendItem(label: 'Item 2', color: Colors.green),
        const LegendItem(label: 'Item 3', color: Colors.blue),
        const LegendItem(label: 'Item 4', color: Colors.yellow),
      ];

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: KeyLegend(items: items),
          ),
        ),
      );

      expect(find.text('Item 1'), findsOneWidget);
      expect(find.text('Item 2'), findsOneWidget);
      expect(find.text('Item 3'), findsOneWidget);
      expect(find.text('Item 4'), findsOneWidget);
    });

    testWidgets('handles empty items list', (WidgetTester tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: KeyLegend(items: []),
          ),
        ),
      );

      expect(find.byType(KeyLegend), findsOneWidget);
    });

    testWidgets('applies correct spacing in horizontal layout',
        (WidgetTester tester) async {
      final items = [
        const LegendItem(label: 'Item 1', color: Colors.red),
        const LegendItem(label: 'Item 2', color: Colors.blue),
      ];

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: KeyLegend(items: items),
          ),
        ),
      );

      final wrap = tester.widget<Wrap>(find.byType(Wrap));
      expect(wrap.spacing, equals(16));
      expect(wrap.runSpacing, equals(8));
    });

    testWidgets('applies correct padding in vertical layout',
        (WidgetTester tester) async {
      final items = [
        const LegendItem(label: 'Item 1', color: Colors.red),
        const LegendItem(label: 'Item 2', color: Colors.blue),
      ];

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: KeyLegend(
              items: items,
              orientation: Axis.vertical,
            ),
          ),
        ),
      );

      final paddings = find.descendant(
        of: find.byType(Column),
        matching: find.byType(Padding),
      );

      expect(paddings, findsWidgets);

      final padding = tester.widget<Padding>(paddings.first);
      expect(
        padding.padding,
        equals(const EdgeInsets.symmetric(vertical: 4)),
      );
    });

    testWidgets('legend item with icon and tooltip',
        (WidgetTester tester) async {
      final items = [
        const LegendItem(
          label: 'Warning',
          color: Colors.orange,
          icon: Icons.warning,
          tooltip: 'This is a warning',
        ),
      ];

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: KeyLegend(items: items),
          ),
        ),
      );

      expect(find.text('Warning'), findsOneWidget);
      expect(find.byIcon(Icons.warning), findsOneWidget);
      expect(find.byIcon(Icons.info_outline), findsOneWidget);
    });
  });

  group('LegendItem', () {
    test('creates legend item with required fields', () {
      const item = LegendItem(
        label: 'Test',
        color: Colors.red,
      );

      expect(item.label, equals('Test'));
      expect(item.color, equals(Colors.red));
      expect(item.icon, isNull);
      expect(item.tooltip, isNull);
    });

    test('creates legend item with all fields', () {
      const item = LegendItem(
        label: 'Test',
        color: Colors.red,
        icon: Icons.star,
        tooltip: 'Test tooltip',
      );

      expect(item.label, equals('Test'));
      expect(item.color, equals(Colors.red));
      expect(item.icon, equals(Icons.star));
      expect(item.tooltip, equals('Test tooltip'));
    });
  });
}
