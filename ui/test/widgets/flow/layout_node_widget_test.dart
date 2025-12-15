import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/models/flow/flow_data.dart';
import 'package:keyrx_ui/widgets/flow/layout_node_widget.dart';

void main() {
  group('LayoutNodeWidget', () {
    group('basic rendering', () {
      testWidgets('displays layout profile info correctly', (tester) async {
        const data = LayoutNodeData(name: 'ANSI-104', type: 'ANSI');

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: LayoutNodeWidget(data: data)),
          ),
        );

        // Verify layout name is displayed
        expect(find.text('ANSI-104'), findsOneWidget);

        // Verify layout type is displayed
        expect(find.text('Type: ANSI'), findsOneWidget);

        // Verify layout icon is displayed
        expect(find.text('▦'), findsOneWidget);
      });

      testWidgets('displays ISO layout correctly', (tester) async {
        const data = LayoutNodeData(name: 'ISO-105', type: 'ISO');

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: LayoutNodeWidget(data: data)),
          ),
        );

        expect(find.text('ISO-105'), findsOneWidget);
        expect(find.text('Type: ISO'), findsOneWidget);
      });

      testWidgets('displays ergonomic layout correctly', (tester) async {
        const data = LayoutNodeData(name: 'Ergo-60', type: 'Ergonomic');

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: LayoutNodeWidget(data: data)),
          ),
        );

        expect(find.text('Ergo-60'), findsOneWidget);
        expect(find.text('Type: Ergonomic'), findsOneWidget);
      });
    });

    group('visual styling', () {
      testWidgets('has consistent dark theme styling', (tester) async {
        const data = LayoutNodeData(name: 'Test Layout', type: 'ANSI');

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: LayoutNodeWidget(data: data)),
          ),
        );

        final container = tester.widget<Container>(
          find.descendant(
            of: find.byType(LayoutNodeWidget),
            matching: find.byType(Container).first,
          ),
        );

        final decoration = container.decoration as BoxDecoration;

        // Verify background color
        expect(decoration.color, const Color(0xFF1E1E1E));

        // Verify border
        expect(decoration.border, isNotNull);
        final border = decoration.border as Border;
        expect(border.top.color, const Color(0xFF333333));

        // Verify border radius
        expect(decoration.borderRadius, BorderRadius.circular(8));

        // Verify shadow
        expect(decoration.boxShadow, isNotNull);
        expect(decoration.boxShadow!.length, 1);
        expect(decoration.boxShadow![0].blurRadius, 6);
        expect(decoration.boxShadow![0].offset, const Offset(0, 4));
      });

      testWidgets('displays icon with proper styling', (tester) async {
        const data = LayoutNodeData(name: 'Test Layout', type: 'ANSI');

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: LayoutNodeWidget(data: data)),
          ),
        );

        // Verify icon text styling
        final iconText = tester.widget<Text>(find.text('▦'));
        expect(iconText.style?.fontSize, 18);
        expect(iconText.style?.color, Colors.white);

        // Verify containers exist
        final containers = tester.widgetList<Container>(
          find.descendant(
            of: find.byType(LayoutNodeWidget),
            matching: find.byType(Container),
          ),
        );

        expect(containers.length, greaterThan(0));
      });

      testWidgets('displays mock layout grid visual', (tester) async {
        const data = LayoutNodeData(name: 'Test Layout', type: 'ANSI');

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: LayoutNodeWidget(data: data)),
          ),
        );

        // Find rows within the widget (the grid is a Row)
        final rows = tester.widgetList<Row>(
          find.descendant(
            of: find.byType(LayoutNodeWidget),
            matching: find.byType(Row),
          ),
        );

        // Find the row with 5 children (the grid row)
        final gridRows = rows.where((r) => r.children.length == 5).toList();

        expect(gridRows.length, 1);

        final gridRow = gridRows.first;
        expect(gridRow.children.length, 5);
      });

      testWidgets('has proper text styling for name and type', (tester) async {
        const data = LayoutNodeData(name: 'ANSI-104', type: 'ANSI');

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: LayoutNodeWidget(data: data)),
          ),
        );

        // Verify name styling
        final nameText = tester.widget<Text>(find.text('ANSI-104'));
        expect(nameText.style?.fontWeight, FontWeight.w600);
        expect(nameText.style?.fontSize, 14);
        expect(nameText.style?.color, Colors.white);

        // Verify type styling
        final typeText = tester.widget<Text>(find.text('Type: ANSI'));
        expect(typeText.style?.fontSize, 11);
        expect(typeText.style?.color, Colors.grey);
      });
    });

    group('dimensions and layout', () {
      testWidgets('has fixed width of 200 pixels', (tester) async {
        const data = LayoutNodeData(name: 'Test', type: 'ANSI');

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: LayoutNodeWidget(data: data)),
          ),
        );

        // Verify the widget has the expected width by checking the rendered box
        final size = tester.getSize(find.byType(LayoutNodeWidget));
        expect(size.width, 200);
      });

      testWidgets('uses proper padding', (tester) async {
        const data = LayoutNodeData(name: 'Test', type: 'ANSI');

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: LayoutNodeWidget(data: data)),
          ),
        );

        final container = tester.widget<Container>(
          find.descendant(
            of: find.byType(LayoutNodeWidget),
            matching: find.byType(Container).first,
          ),
        );

        expect(container.padding, const EdgeInsets.all(12));
      });

      testWidgets('content is laid out vertically', (tester) async {
        const data = LayoutNodeData(name: 'Test', type: 'ANSI');

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: LayoutNodeWidget(data: data)),
          ),
        );

        // Find the main column
        final column = tester.widget<Column>(
          find.descendant(
            of: find.byType(LayoutNodeWidget),
            matching: find.byType(Column).first,
          ),
        );

        expect(column.crossAxisAlignment, CrossAxisAlignment.start);
        expect(column.mainAxisSize, MainAxisSize.min);
      });
    });

    group('edge cases', () {
      testWidgets('handles long layout names gracefully', (tester) async {
        const data = LayoutNodeData(
          name: 'Very Long Layout Profile Name That Might Overflow',
          type: 'Custom',
        );

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: LayoutNodeWidget(data: data)),
          ),
        );

        // Widget should render without overflow errors
        expect(tester.takeException(), isNull);

        // Layout name should still be visible
        expect(
          find.text('Very Long Layout Profile Name That Might Overflow'),
          findsOneWidget,
        );
      });

      testWidgets('handles long type names', (tester) async {
        const data = LayoutNodeData(
          name: 'Test',
          type: 'Very Long Type Name That Is Unusual',
        );

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: LayoutNodeWidget(data: data)),
          ),
        );

        // Widget should render without overflow errors
        expect(tester.takeException(), isNull);

        expect(
          find.text('Type: Very Long Type Name That Is Unusual'),
          findsOneWidget,
        );
      });

      testWidgets('handles empty strings gracefully', (tester) async {
        const data = LayoutNodeData(name: '', type: '');

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: LayoutNodeWidget(data: data)),
          ),
        );

        // Widget should render without errors
        expect(tester.takeException(), isNull);
      });
    });

    group('common layout types', () {
      final layoutTypes = [
        ('ANSI-104', 'ANSI'),
        ('ISO-105', 'ISO'),
        ('JIS-109', 'JIS'),
        ('Ergo-60', 'Ergonomic'),
        ('Split-80', 'Split'),
        ('Ortho-40', 'Ortholinear'),
      ];

      for (final (name, type) in layoutTypes) {
        testWidgets('renders $name layout correctly', (tester) async {
          final data = LayoutNodeData(name: name, type: type);

          await tester.pumpWidget(
            MaterialApp(
              home: Scaffold(body: LayoutNodeWidget(data: data)),
            ),
          );

          expect(find.text(name), findsOneWidget);
          expect(find.text('Type: $type'), findsOneWidget);
          expect(tester.takeException(), isNull);
        });
      }
    });

    group('accessibility', () {
      testWidgets('has sufficient contrast for text', (tester) async {
        const data = LayoutNodeData(name: 'Test Layout', type: 'ANSI');

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: LayoutNodeWidget(data: data)),
          ),
        );

        // Name text should be white on dark background (good contrast)
        final nameText = tester.widget<Text>(find.text('Test Layout'));
        expect(nameText.style?.color, Colors.white);

        // Type text should be grey (secondary info, still readable)
        final typeText = tester.widget<Text>(find.text('Type: ANSI'));
        expect(typeText.style?.color, Colors.grey);
      });

      testWidgets('icon is visible against background', (tester) async {
        const data = LayoutNodeData(name: 'Test', type: 'ANSI');

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: LayoutNodeWidget(data: data)),
          ),
        );

        final iconText = tester.widget<Text>(find.text('▦'));
        expect(iconText.style?.color, Colors.white);
      });
    });
  });
}
