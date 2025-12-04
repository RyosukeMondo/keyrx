import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/widgets/editor/key_grid.dart';
import 'package:keyrx_ui/widgets/editor/key_button.dart';

void main() {
  group('KeyGrid', () {
    testWidgets('renders keyboard layout', (WidgetTester tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: KeyGrid(),
          ),
        ),
      );

      // Check for function keys
      expect(find.text('Esc'), findsOneWidget);
      expect(find.text('F1'), findsOneWidget);
      expect(find.text('F12'), findsOneWidget);

      // Check for number row
      expect(find.text('1'), findsOneWidget);
      expect(find.text('0'), findsOneWidget);
      expect(find.text('Backspace'), findsOneWidget);

      // Check for QWERTY row
      expect(find.text('Tab'), findsOneWidget);
      expect(find.text('Q'), findsOneWidget);
      expect(find.text('P'), findsOneWidget);

      // Check for home row
      expect(find.text('CapsLock'), findsOneWidget);
      expect(find.text('A'), findsOneWidget);
      expect(find.text('Enter'), findsOneWidget);

      // Check for bottom row
      expect(find.text('LShift'), findsOneWidget);
      expect(find.text('Z'), findsOneWidget);
      expect(find.text('RShift'), findsOneWidget);

      // Check for modifier row
      expect(find.text('LCtrl'), findsOneWidget);
      expect(find.text('Space'), findsOneWidget);
      expect(find.text('RCtrl'), findsOneWidget);
    });

    testWidgets('calls onKeySelected when key is tapped',
        (WidgetTester tester) async {
      String? selectedKey;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: KeyGrid(
              onKeySelected: (key) => selectedKey = key,
            ),
          ),
        ),
      );

      await tester.tap(find.text('A'));
      await tester.pumpAndSettle();

      expect(selectedKey, equals('A'));

      await tester.tap(find.text('Space'));
      await tester.pumpAndSettle();

      expect(selectedKey, equals('Space'));
    });

    testWidgets('highlights selected key', (WidgetTester tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: KeyGrid(
              selectedKey: 'A',
            ),
          ),
        ),
      );

      // Find the KeyButton with label 'A'
      final keyButtons = find.byType(KeyButton);
      final aButton = tester.widgetList<KeyButton>(keyButtons).firstWhere(
            (button) => button.label == 'A',
          );

      expect(aButton.isSelected, isTrue);

      // Verify other keys are not selected
      final bButton = tester.widgetList<KeyButton>(keyButtons).firstWhere(
            (button) => button.label == 'B',
          );

      expect(bButton.isSelected, isFalse);
    });

    testWidgets('updates selected key when state changes',
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
      var aButton = tester.widgetList<KeyButton>(keyButtons).firstWhere(
            (button) => button.label == 'A',
          );
      expect(aButton.isSelected, isFalse);

      // Tap 'A' key
      await tester.tap(find.text('A'));
      await tester.pumpAndSettle();

      // Now 'A' should be selected
      keyButtons = find.byType(KeyButton);
      aButton = tester.widgetList<KeyButton>(keyButtons).firstWhere(
            (button) => button.label == 'A',
          );
      expect(aButton.isSelected, isTrue);

      // Tap 'B' key
      await tester.tap(find.text('B'));
      await tester.pumpAndSettle();

      // Now 'B' should be selected and 'A' unselected
      keyButtons = find.byType(KeyButton);
      aButton = tester.widgetList<KeyButton>(keyButtons).firstWhere(
            (button) => button.label == 'A',
          );
      final bButton = tester.widgetList<KeyButton>(keyButtons).firstWhere(
            (button) => button.label == 'B',
          );
      expect(aButton.isSelected, isFalse);
      expect(bButton.isSelected, isTrue);
    });

    testWidgets('handles null onKeySelected', (WidgetTester tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: KeyGrid(
              onKeySelected: null,
            ),
          ),
        ),
      );

      // Should not throw error when tapping
      await tester.tap(find.text('A'));
      await tester.pumpAndSettle();

      expect(find.text('A'), findsOneWidget);
    });

    testWidgets('renders all modifier keys', (WidgetTester tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: KeyGrid(),
          ),
        ),
      );

      expect(find.text('LCtrl'), findsOneWidget);
      expect(find.text('RCtrl'), findsOneWidget);
      expect(find.text('LShift'), findsOneWidget);
      expect(find.text('RShift'), findsOneWidget);
      expect(find.text('LAlt'), findsOneWidget);
      expect(find.text('RAlt'), findsOneWidget);
      expect(find.text('LWin'), findsOneWidget);
      expect(find.text('RWin'), findsOneWidget);
      expect(find.text('Menu'), findsOneWidget);
    });

    testWidgets('renders correct number of KeyButton widgets',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: KeyGrid(),
          ),
        ),
      );

      final keyButtons = find.byType(KeyButton);

      // Count expected keys:
      // Function row: 13 keys
      // Number row: 14 keys
      // QWERTY row: 14 keys
      // Home row: 13 keys
      // Bottom row: 12 keys
      // Modifier row: 8 keys
      // Total: 74 keys
      expect(keyButtons, findsNWidgets(74));
    });
  });
}
