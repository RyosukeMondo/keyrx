// Tests for SoftKeyboard widget.

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/widgets/soft_keyboard.dart';

void main() {
  group('SoftKeyboard Widget', () {
    testWidgets('renders with all keys initially', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(home: Scaffold(body: SoftKeyboard())),
      );

      // Should show the count of all keys (110 total)
      expect(find.text('110 keys'), findsOneWidget);

      // Search bar should be present
      expect(find.byType(TextField), findsOneWidget);
      expect(find.text('Search keys...'), findsOneWidget);

      // Grid should be present
      expect(find.byType(GridView), findsOneWidget);
    });

    testWidgets('displays search icon and placeholder', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(home: Scaffold(body: SoftKeyboard())),
      );

      // Search icon should be visible
      expect(find.byIcon(Icons.search), findsOneWidget);

      // Placeholder text should be visible
      expect(find.text('Search keys...'), findsOneWidget);
    });

    testWidgets('filters keys based on search query', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(home: Scaffold(body: SoftKeyboard())),
      );

      // Enter search query
      await tester.enterText(find.byType(TextField), 'shift');
      await tester.pump();

      // Should show only 2 shift keys (LeftShift, RightShift)
      expect(find.text('2 keys'), findsOneWidget);

      // Clear icon should appear
      expect(find.byIcon(Icons.clear), findsOneWidget);
    });

    testWidgets('search is case-insensitive', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(home: Scaffold(body: SoftKeyboard())),
      );

      // Search with uppercase
      await tester.enterText(find.byType(TextField), 'CTRL');
      await tester.pump();

      // Should find both ctrl keys (LeftCtrl, RightCtrl)
      expect(find.text('2 keys'), findsOneWidget);
    });

    testWidgets('search filters by display name', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(home: Scaffold(body: SoftKeyboard())),
      );

      // Search for arrow keys using symbol
      await tester.enterText(find.byType(TextField), '↑');
      await tester.pump();

      // Should find the up arrow
      expect(find.text('1 keys'), findsOneWidget);
    });

    testWidgets('search filters by category', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(home: Scaffold(body: SoftKeyboard())),
      );

      // Search by category
      await tester.enterText(find.byType(TextField), 'media');
      await tester.pump();

      // Should find all 7 media keys
      expect(find.text('7 keys'), findsOneWidget);
    });

    testWidgets('shows empty state when no keys match', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(home: Scaffold(body: SoftKeyboard())),
      );

      // Enter query that matches nothing
      await tester.enterText(find.byType(TextField), 'zzzzz');
      await tester.pump();

      // Should show empty state
      expect(find.text('0 keys'), findsOneWidget);
      expect(find.byIcon(Icons.search_off), findsOneWidget);
      expect(find.text('No keys found'), findsOneWidget);
    });

    testWidgets('clear button resets search', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(home: Scaffold(body: SoftKeyboard())),
      );

      // Enter search query
      await tester.enterText(find.byType(TextField), 'ctrl');
      await tester.pump();

      expect(find.text('2 keys'), findsOneWidget);

      // Tap clear button
      await tester.tap(find.byIcon(Icons.clear));
      await tester.pump();

      // Should show all keys again
      expect(find.text('110 keys'), findsOneWidget);
    });

    testWidgets('calls onKeySelected when key is tapped', (tester) async {
      String? selectedKey;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SoftKeyboard(
              onKeySelected: (key) {
                selectedKey = key;
              },
            ),
          ),
        ),
      );

      // Search for a specific key to make it easier to find
      await tester.enterText(find.byType(TextField), 'escape');
      await tester.pump();

      // Tap the Escape key button
      await tester.tap(find.text('Esc'));
      await tester.pump();

      // Callback should be called with correct variant
      expect(selectedKey, equals('Escape'));
    });

    testWidgets('highlights selected key', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(body: SoftKeyboard(selectedKey: 'Enter')),
        ),
      );

      // Search for Enter to make it visible
      await tester.enterText(find.byType(TextField), 'enter');
      await tester.pump();

      // Enter key button should exist
      expect(find.text('Enter'), findsOneWidget);

      // The selected key should have elevated Material
      final material = tester.widget<Material>(
        find
            .ancestor(of: find.text('Enter'), matching: find.byType(Material))
            .first,
      );

      // Selected keys have elevation of 4.0
      expect(material.elevation, equals(4.0));
    });

    testWidgets('non-selected key has lower elevation', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(body: SoftKeyboard(selectedKey: 'Enter')),
        ),
      );

      // Search for a key that's not selected
      await tester.enterText(find.byType(TextField), 'space');
      await tester.pump();

      // Space key should exist but not be selected
      expect(find.text('Space'), findsOneWidget);

      final material = tester.widget<Material>(
        find
            .ancestor(of: find.text('Space'), matching: find.byType(Material))
            .first,
      );

      // Non-selected keys have elevation of 1.0
      expect(material.elevation, equals(1.0));
    });

    testWidgets('includes all letter keys A-Z', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(home: Scaffold(body: SoftKeyboard())),
      );

      // Search for letters category
      await tester.enterText(find.byType(TextField), 'letters');
      await tester.pump();

      // Should find 26 letter keys
      expect(find.text('26 keys'), findsOneWidget);
    });

    testWidgets('includes all number keys 0-9', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(home: Scaffold(body: SoftKeyboard())),
      );

      // Search for numbers category
      await tester.enterText(find.byType(TextField), 'numbers');
      await tester.pump();

      // Should find 10 number keys
      expect(find.text('10 keys'), findsOneWidget);
    });

    testWidgets('includes all function keys F1-F12', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(home: Scaffold(body: SoftKeyboard())),
      );

      // Search for function category
      await tester.enterText(find.byType(TextField), 'function');
      await tester.pump();

      // Should find 12 function keys
      expect(find.text('12 keys'), findsOneWidget);
    });

    testWidgets('includes all modifier keys', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(home: Scaffold(body: SoftKeyboard())),
      );

      // Search for modifiers category
      await tester.enterText(find.byType(TextField), 'modifiers');
      await tester.pump();

      // Should find 8 modifier keys (L/R Shift, Ctrl, Alt, Meta)
      expect(find.text('8 keys'), findsOneWidget);
    });

    testWidgets('includes all navigation keys', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(home: Scaffold(body: SoftKeyboard())),
      );

      // Search for navigation category
      await tester.enterText(find.byType(TextField), 'navigation');
      await tester.pump();

      // Should find 8 navigation keys (arrows, home, end, pgup, pgdn)
      expect(find.text('8 keys'), findsOneWidget);
    });

    testWidgets('includes numpad keys', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(home: Scaffold(body: SoftKeyboard())),
      );

      // Search for numpad category
      await tester.enterText(find.byType(TextField), 'numpad');
      await tester.pump();

      // Should find 16 numpad keys
      expect(find.text('16 keys'), findsOneWidget);
    });

    testWidgets('includes media control keys', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(home: Scaffold(body: SoftKeyboard())),
      );

      // Search for media category
      await tester.enterText(find.byType(TextField), 'media');
      await tester.pump();

      // Should find 7 media keys
      expect(find.text('7 keys'), findsOneWidget);
    });

    testWidgets('respects custom keySize parameter', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(home: Scaffold(body: SoftKeyboard(keySize: 80.0))),
      );

      // Get the GridView delegate
      final gridView = tester.widget<GridView>(find.byType(GridView));
      final delegate =
          gridView.gridDelegate as SliverGridDelegateWithMaxCrossAxisExtent;

      // Should use custom key size
      expect(delegate.maxCrossAxisExtent, equals(80.0));
    });

    testWidgets('respects custom keySpacing parameter', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(home: Scaffold(body: SoftKeyboard(keySpacing: 12.0))),
      );

      // Get the GridView delegate
      final gridView = tester.widget<GridView>(find.byType(GridView));
      final delegate =
          gridView.gridDelegate as SliverGridDelegateWithMaxCrossAxisExtent;

      // Should use custom spacing
      expect(delegate.mainAxisSpacing, equals(12.0));
      expect(delegate.crossAxisSpacing, equals(12.0));
    });
  });
}
