import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/widgets/editor/key_grid.dart';
import 'package:keyrx_ui/widgets/editor/key_button.dart';
import 'package:keyrx_ui/widgets/editor/key_legend.dart';

/// Integration tests for KeyGrid with KeyButton composition and state flow.
void main() {
  group('KeyGrid Integration', () {
    testWidgets('KeyGrid composes KeyButtons correctly',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: KeyGrid(),
          ),
        ),
      );

      // Verify KeyGrid contains KeyButton widgets
      final keyButtons = find.byType(KeyButton);
      expect(keyButtons, findsWidgets);

      // Verify all KeyButtons are properly rendered
      final firstButton = tester.widget<KeyButton>(keyButtons.first);
      expect(firstButton.label, isNotEmpty);
    });

    testWidgets('KeyGrid propagates callbacks to KeyButtons',
        (WidgetTester tester) async {
      final selectedKeys = <String>[];

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: KeyGrid(
              onKeySelected: (key) => selectedKeys.add(key),
            ),
          ),
        ),
      );

      // Tap multiple keys and verify callbacks fire
      await tester.tap(find.text('A'));
      await tester.pumpAndSettle();
      expect(selectedKeys, contains('A'));

      await tester.tap(find.text('B'));
      await tester.pumpAndSettle();
      expect(selectedKeys, contains('B'));

      expect(selectedKeys.length, equals(2));
    });

    testWidgets('KeyGrid with KeyLegend integration',
        (WidgetTester tester) async {
      String? selectedKey;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Column(
              children: [
                Expanded(
                  child: KeyGrid(
                    selectedKey: selectedKey,
                    onKeySelected: (key) {
                      selectedKey = key;
                    },
                  ),
                ),
                const KeyLegend(
                  items: [
                    LegendItem(label: 'Unassigned', color: Colors.grey),
                    LegendItem(label: 'Remapped', color: Colors.blue),
                    LegendItem(label: 'Blocked', color: Colors.red),
                  ],
                ),
              ],
            ),
          ),
        ),
      );

      // Verify both widgets are present
      expect(find.byType(KeyGrid), findsOneWidget);
      expect(find.byType(KeyLegend), findsOneWidget);

      // Verify legend displays color categories
      expect(find.text('Unassigned'), findsOneWidget);
      expect(find.text('Remapped'), findsOneWidget);
      expect(find.text('Blocked'), findsOneWidget);
    });

    testWidgets('KeyGrid selection state flows correctly',
        (WidgetTester tester) async {
      String? selectedKey;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: StatefulBuilder(
              builder: (context, setState) {
                return KeyGrid(
                  selectedKey: selectedKey,
                  onKeySelected: (key) {
                    setState(() {
                      selectedKey = key;
                    });
                  },
                );
              },
            ),
          ),
        ),
      );

      // Initially no key selected
      var keyButtons = find.byType(KeyButton);
      var widgets = tester.widgetList<KeyButton>(keyButtons);
      expect(widgets.where((b) => b.isSelected).length, equals(0));

      // Select first key
      await tester.tap(find.text('Esc'));
      await tester.pumpAndSettle();

      // Verify selection state updated
      keyButtons = find.byType(KeyButton);
      widgets = tester.widgetList<KeyButton>(keyButtons);
      final selectedButton =
          widgets.firstWhere((b) => b.label == 'Esc', orElse: () => widgets.first);
      expect(selectedButton.isSelected, isTrue);

      // Select different key
      await tester.tap(find.text('A'));
      await tester.pumpAndSettle();

      // Verify only new key is selected
      keyButtons = find.byType(KeyButton);
      widgets = tester.widgetList<KeyButton>(keyButtons);
      final escButton =
          widgets.firstWhere((b) => b.label == 'Esc', orElse: () => widgets.first);
      final aButton =
          widgets.firstWhere((b) => b.label == 'A', orElse: () => widgets.first);

      expect(escButton.isSelected, isFalse);
      expect(aButton.isSelected, isTrue);
    });

    testWidgets('KeyGrid handles rapid taps correctly',
        (WidgetTester tester) async {
      final tappedKeys = <String>[];

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: KeyGrid(
              onKeySelected: (key) => tappedKeys.add(key),
            ),
          ),
        ),
      );

      // Rapidly tap multiple keys
      await tester.tap(find.text('A'));
      await tester.tap(find.text('B'));
      await tester.tap(find.text('C'));
      await tester.pumpAndSettle();

      // All taps should be registered
      expect(tappedKeys, containsAll(['A', 'B', 'C']));
    });

    testWidgets('KeyGrid rebuilds efficiently on selection change',
        (WidgetTester tester) async {
      String? selectedKey;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: StatefulBuilder(
              builder: (context, setState) {
                return KeyGrid(
                  selectedKey: selectedKey,
                  onKeySelected: (key) {
                    setState(() {
                      selectedKey = key;
                    });
                  },
                );
              },
            ),
          ),
        ),
      );

      // Initial render
      await tester.pump();
      final initialButtons = find.byType(KeyButton);
      final initialCount = tester.widgetList(initialButtons).length;

      // Change selection
      await tester.tap(find.text('A'));
      await tester.pumpAndSettle();

      // Same number of buttons after selection change
      final afterButtons = find.byType(KeyButton);
      final afterCount = tester.widgetList(afterButtons).length;
      expect(afterCount, equals(initialCount));
    });
  });
}
