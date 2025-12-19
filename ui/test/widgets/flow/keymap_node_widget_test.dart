import 'package:core_logic/models/flow/flow_data.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/widgets/flow/keymap_node_widget.dart';

void main() {
  group('KeymapNodeWidget', () {
    group('active keymap', () {
      testWidgets('displays keymap info correctly', (tester) async {
        const data = KeymapNodeData(name: 'Default', layers: 3, isActive: true);

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: KeymapNodeWidget(data: data)),
          ),
        );

        // Verify keymap name is displayed
        expect(find.text('Default'), findsOneWidget);

        // Verify layer count is displayed
        expect(find.text('3 Layers'), findsOneWidget);

        // Verify active status
        expect(find.text('● Active'), findsOneWidget);

        // Verify lightning icon
        expect(find.text('⚡'), findsOneWidget);
      });

      testWidgets('displays active state with proper styling', (tester) async {
        const data = KeymapNodeData(
          name: 'Active Keymap',
          layers: 5,
          isActive: true,
        );

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: KeymapNodeWidget(data: data)),
          ),
        );

        final container = tester.widget<Container>(
          find.descendant(
            of: find.byType(KeymapNodeWidget),
            matching: find.byType(Container).first,
          ),
        );

        final decoration = container.decoration as BoxDecoration;

        // Verify active border color (purple)
        expect(decoration.border, isNotNull);
        final border = decoration.border as Border;
        expect(border.top.color, const Color(0xFF9B59B6));
        expect(border.top.width, 2);

        // Verify status badge shows purple color for active
        final statusText = tester.widget<Text>(find.text('● Active'));
        expect(statusText.style?.color, const Color(0xFF9B59B6));
      });

      testWidgets('displays purple accent on active keymap', (tester) async {
        const data = KeymapNodeData(name: 'Test', layers: 1, isActive: true);

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: KeymapNodeWidget(data: data)),
          ),
        );

        // Icon should be purple
        final iconText = tester.widget<Text>(find.text('⚡'));
        expect(iconText.style?.color, const Color(0xFF9B59B6));

        // Verify containers exist
        final containers = tester.widgetList<Container>(
          find.descendant(
            of: find.byType(KeymapNodeWidget),
            matching: find.byType(Container),
          ),
        );

        expect(containers.length, greaterThan(0));
      });
    });

    group('inactive/pending keymap', () {
      testWidgets('displays pending status correctly', (tester) async {
        const data = KeymapNodeData(
          name: 'Pending Keymap',
          layers: 2,
          isActive: false,
        );

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: KeymapNodeWidget(data: data)),
          ),
        );

        // Verify keymap name
        expect(find.text('Pending Keymap'), findsOneWidget);

        // Verify layer count
        expect(find.text('2 Layers'), findsOneWidget);

        // Verify pending status
        expect(find.text('○ Pending'), findsOneWidget);
      });

      testWidgets('displays inactive state with proper styling', (
        tester,
      ) async {
        const data = KeymapNodeData(
          name: 'Inactive Keymap',
          layers: 4,
          isActive: false,
        );

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: KeymapNodeWidget(data: data)),
          ),
        );

        final container = tester.widget<Container>(
          find.descendant(
            of: find.byType(KeymapNodeWidget),
            matching: find.byType(Container).first,
          ),
        );

        final decoration = container.decoration as BoxDecoration;

        // Verify inactive border is grey and thinner
        expect(decoration.border, isNotNull);
        final border = decoration.border as Border;
        expect(border.top.color, const Color(0xFF444444));
        expect(border.top.width, 1);

        // Verify status badge shows grey color for pending
        final statusText = tester.widget<Text>(find.text('○ Pending'));
        expect(statusText.style?.color, const Color(0xFFBDC3C7));
      });
    });

    group('layer count variations', () {
      testWidgets('displays single layer correctly', (tester) async {
        const data = KeymapNodeData(name: 'Simple', layers: 1, isActive: true);

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: KeymapNodeWidget(data: data)),
          ),
        );

        expect(find.text('1 Layers'), findsOneWidget);
      });

      testWidgets('displays multiple layers correctly', (tester) async {
        const data = KeymapNodeData(name: 'Complex', layers: 8, isActive: true);

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: KeymapNodeWidget(data: data)),
          ),
        );

        expect(find.text('8 Layers'), findsOneWidget);
      });

      testWidgets('displays zero layers', (tester) async {
        const data = KeymapNodeData(name: 'Empty', layers: 0, isActive: false);

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: KeymapNodeWidget(data: data)),
          ),
        );

        expect(find.text('0 Layers'), findsOneWidget);
      });
    });

    group('visual styling', () {
      testWidgets('has consistent dark theme styling', (tester) async {
        const data = KeymapNodeData(name: 'Test', layers: 3, isActive: true);

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: KeymapNodeWidget(data: data)),
          ),
        );

        final container = tester.widget<Container>(
          find.descendant(
            of: find.byType(KeymapNodeWidget),
            matching: find.byType(Container).first,
          ),
        );

        final decoration = container.decoration as BoxDecoration;

        // Verify background color
        expect(decoration.color, const Color(0xFF1E1E1E));

        // Verify border radius
        expect(decoration.borderRadius, BorderRadius.circular(8));

        // Verify shadow
        expect(decoration.boxShadow, isNotNull);
        expect(decoration.boxShadow!.length, 1);
        expect(decoration.boxShadow![0].blurRadius, 6);
        expect(decoration.boxShadow![0].offset, const Offset(0, 4));
      });

      testWidgets('has proper text styling for name and layers', (
        tester,
      ) async {
        const data = KeymapNodeData(name: 'Gaming', layers: 4, isActive: true);

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: KeymapNodeWidget(data: data)),
          ),
        );

        // Verify name styling
        final nameText = tester.widget<Text>(find.text('Gaming'));
        expect(nameText.style?.fontWeight, FontWeight.w600);
        expect(nameText.style?.fontSize, 14);
        expect(nameText.style?.color, Colors.white);

        // Verify layers styling
        final layersText = tester.widget<Text>(find.text('4 Layers'));
        expect(layersText.style?.fontSize, 11);
        expect(layersText.style?.color, Colors.grey);
      });

      testWidgets('status badge has proper styling for active', (tester) async {
        const data = KeymapNodeData(name: 'Test', layers: 3, isActive: true);

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: KeymapNodeWidget(data: data)),
          ),
        );

        // Find the status badge container
        final containers = tester.widgetList<Container>(
          find.descendant(
            of: find.byType(KeymapNodeWidget),
            matching: find.byType(Container),
          ),
        );

        // Find container with status badge
        final badgeContainers = containers.where((c) {
          final decoration = c.decoration;
          if (decoration is BoxDecoration) {
            return decoration.borderRadius == BorderRadius.circular(10);
          }
          return false;
        }).toList();

        expect(badgeContainers.length, 1);

        final badgeDecoration =
            badgeContainers.first.decoration as BoxDecoration;
        expect(badgeDecoration.color, const Color(0x339B59B6));

        // Status text
        final statusText = tester.widget<Text>(find.text('● Active'));
        expect(statusText.style?.fontSize, 10);
        expect(statusText.style?.color, const Color(0xFF9B59B6));
      });

      testWidgets('status badge has proper styling for pending', (
        tester,
      ) async {
        const data = KeymapNodeData(name: 'Test', layers: 3, isActive: false);

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: KeymapNodeWidget(data: data)),
          ),
        );

        // Find the status badge container
        final containers = tester.widgetList<Container>(
          find.descendant(
            of: find.byType(KeymapNodeWidget),
            matching: find.byType(Container),
          ),
        );

        final badgeContainers = containers.where((c) {
          final decoration = c.decoration;
          if (decoration is BoxDecoration) {
            return decoration.borderRadius == BorderRadius.circular(10);
          }
          return false;
        }).toList();

        expect(badgeContainers.length, 1);

        final badgeDecoration =
            badgeContainers.first.decoration as BoxDecoration;
        expect(badgeDecoration.color, const Color(0x337F8C8D));

        // Status text
        final statusText = tester.widget<Text>(find.text('○ Pending'));
        expect(statusText.style?.fontSize, 10);
        expect(statusText.style?.color, const Color(0xFFBDC3C7));
      });
    });

    group('dimensions and layout', () {
      testWidgets('has fixed width of 180 pixels', (tester) async {
        const data = KeymapNodeData(name: 'Test', layers: 3, isActive: true);

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: KeymapNodeWidget(data: data)),
          ),
        );

        // Verify the widget has the expected width by checking the rendered box
        final size = tester.getSize(find.byType(KeymapNodeWidget));
        expect(size.width, 180);
      });

      testWidgets('uses proper padding', (tester) async {
        const data = KeymapNodeData(name: 'Test', layers: 3, isActive: true);

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: KeymapNodeWidget(data: data)),
          ),
        );

        final container = tester.widget<Container>(
          find.descendant(
            of: find.byType(KeymapNodeWidget),
            matching: find.byType(Container).first,
          ),
        );

        expect(container.padding, const EdgeInsets.all(12));
      });

      testWidgets('content is laid out vertically', (tester) async {
        const data = KeymapNodeData(name: 'Test', layers: 3, isActive: true);

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: KeymapNodeWidget(data: data)),
          ),
        );

        // Find the main column
        final column = tester.widget<Column>(
          find.descendant(
            of: find.byType(KeymapNodeWidget),
            matching: find.byType(Column).first,
          ),
        );

        expect(column.crossAxisAlignment, CrossAxisAlignment.start);
        expect(column.mainAxisSize, MainAxisSize.min);
      });
    });

    group('edge cases', () {
      testWidgets('handles long keymap names gracefully', (tester) async {
        const data = KeymapNodeData(
          name: 'Very Long Keymap Name That Might Overflow The Widget',
          layers: 3,
          isActive: true,
        );

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: KeymapNodeWidget(data: data)),
          ),
        );

        // Widget should render without overflow errors
        expect(tester.takeException(), isNull);

        // Keymap name should still be visible
        expect(
          find.text('Very Long Keymap Name That Might Overflow The Widget'),
          findsOneWidget,
        );
      });

      testWidgets('handles large layer counts', (tester) async {
        const data = KeymapNodeData(
          name: 'Complex',
          layers: 999,
          isActive: true,
        );

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: KeymapNodeWidget(data: data)),
          ),
        );

        expect(find.text('999 Layers'), findsOneWidget);
        expect(tester.takeException(), isNull);
      });

      testWidgets('handles empty keymap name', (tester) async {
        const data = KeymapNodeData(name: '', layers: 1, isActive: true);

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: KeymapNodeWidget(data: data)),
          ),
        );

        // Widget should render without errors
        expect(tester.takeException(), isNull);
      });
    });

    group('common keymap names', () {
      final keymaps = [
        ('Default', 3),
        ('Gaming', 4),
        ('Vim', 5),
        ('Emacs', 6),
        ('Colemak', 2),
        ('QWERTY', 1),
      ];

      for (final (name, layers) in keymaps) {
        testWidgets('renders $name keymap correctly', (tester) async {
          final data = KeymapNodeData(
            name: name,
            layers: layers,
            isActive: true,
          );

          await tester.pumpWidget(
            MaterialApp(
              home: Scaffold(body: KeymapNodeWidget(data: data)),
            ),
          );

          expect(find.text(name), findsOneWidget);
          expect(find.text('$layers Layers'), findsOneWidget);
          expect(tester.takeException(), isNull);
        });
      }
    });

    group('state transitions', () {
      testWidgets('can toggle between active and pending states', (
        tester,
      ) async {
        // Start with active
        const activeData = KeymapNodeData(
          name: 'Test',
          layers: 3,
          isActive: true,
        );

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: KeymapNodeWidget(data: activeData)),
          ),
        );

        expect(find.text('● Active'), findsOneWidget);

        // Rebuild with pending
        const pendingData = KeymapNodeData(
          name: 'Test',
          layers: 3,
          isActive: false,
        );

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: KeymapNodeWidget(data: pendingData)),
          ),
        );

        expect(find.text('○ Pending'), findsOneWidget);
        expect(find.text('● Active'), findsNothing);
      });
    });

    group('accessibility', () {
      testWidgets('has sufficient contrast for text', (tester) async {
        const data = KeymapNodeData(
          name: 'Test Keymap',
          layers: 3,
          isActive: true,
        );

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: KeymapNodeWidget(data: data)),
          ),
        );

        // Name text should be white on dark background (good contrast)
        final nameText = tester.widget<Text>(find.text('Test Keymap'));
        expect(nameText.style?.color, Colors.white);

        // Layers text should be grey (secondary info, still readable)
        final layersText = tester.widget<Text>(find.text('3 Layers'));
        expect(layersText.style?.color, Colors.grey);

        // Active status should have purple color (sufficient contrast)
        final statusText = tester.widget<Text>(find.text('● Active'));
        expect(statusText.style?.color, const Color(0xFF9B59B6));
      });

      testWidgets('icon is visible against background', (tester) async {
        const data = KeymapNodeData(name: 'Test', layers: 3, isActive: true);

        await tester.pumpWidget(
          const MaterialApp(
            home: Scaffold(body: KeymapNodeWidget(data: data)),
          ),
        );

        final iconText = tester.widget<Text>(find.text('⚡'));
        expect(iconText.style?.color, const Color(0xFF9B59B6));
      });
    });
  });
}
