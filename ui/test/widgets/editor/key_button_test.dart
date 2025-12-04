import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/widgets/editor/key_button.dart';

void main() {
  group('KeyButton', () {
    testWidgets('renders label correctly', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: KeyButton(
              label: 'A',
              onTap: () {},
            ),
          ),
        ),
      );

      expect(find.text('A'), findsOneWidget);
    });

    testWidgets('calls onTap when tapped', (WidgetTester tester) async {
      var tapped = false;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: KeyButton(
              label: 'A',
              onTap: () => tapped = true,
            ),
          ),
        ),
      );

      await tester.tap(find.byType(KeyButton));
      await tester.pumpAndSettle();

      expect(tapped, isTrue);
    });

    testWidgets('shows selected state', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: KeyButton(
              label: 'A',
              isSelected: true,
              onTap: () {},
            ),
          ),
        ),
      );

      final material = tester.widget<Material>(
        find.descendant(
          of: find.byType(KeyButton),
          matching: find.byType(Material),
        ),
      );

      expect(material.color, equals(Colors.blue));
    });

    testWidgets('shows unselected state', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: KeyButton(
              label: 'A',
              isSelected: false,
              onTap: () {},
            ),
          ),
        ),
      );

      final material = tester.widget<Material>(
        find.descendant(
          of: find.byType(KeyButton),
          matching: find.byType(Material),
        ),
      );

      expect(material.color, equals(Colors.grey[800]));
    });

    testWidgets('uses custom width when provided', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: KeyButton(
              label: 'A',
              width: 100,
              onTap: () {},
            ),
          ),
        ),
      );

      final container = tester.widget<Container>(
        find.descendant(
          of: find.byType(InkWell),
          matching: find.byType(Container),
        ),
      );

      expect(container.constraints?.minWidth, equals(100));
    });

    testWidgets('uses custom height when provided', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: KeyButton(
              label: 'A',
              height: 60,
              onTap: () {},
            ),
          ),
        ),
      );

      final container = tester.widget<Container>(
        find.descendant(
          of: find.byType(InkWell),
          matching: find.byType(Container),
        ),
      );

      expect(container.constraints?.minHeight, equals(60));
    });

    testWidgets('handles null onTap', (WidgetTester tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: KeyButton(
              label: 'A',
              onTap: null,
            ),
          ),
        ),
      );

      expect(find.text('A'), findsOneWidget);
    });
  });

  group('KeyButton.getStandardWidth', () {
    test('returns correct width for Backspace', () {
      expect(KeyButton.getStandardWidth('Backspace'), equals(70));
    });

    test('returns correct width for Tab', () {
      expect(KeyButton.getStandardWidth('Tab'), equals(70));
    });

    test('returns correct width for CapsLock', () {
      expect(KeyButton.getStandardWidth('CapsLock'), equals(70));
    });

    test('returns correct width for Enter', () {
      expect(KeyButton.getStandardWidth('Enter'), equals(70));
    });

    test('returns correct width for LShift', () {
      expect(KeyButton.getStandardWidth('LShift'), equals(90));
    });

    test('returns correct width for RShift', () {
      expect(KeyButton.getStandardWidth('RShift'), equals(90));
    });

    test('returns correct width for Space', () {
      expect(KeyButton.getStandardWidth('Space'), equals(200));
    });

    test('returns correct width for LCtrl', () {
      expect(KeyButton.getStandardWidth('LCtrl'), equals(50));
    });

    test('returns correct width for RCtrl', () {
      expect(KeyButton.getStandardWidth('RCtrl'), equals(50));
    });

    test('returns correct width for LAlt', () {
      expect(KeyButton.getStandardWidth('LAlt'), equals(50));
    });

    test('returns correct width for RAlt', () {
      expect(KeyButton.getStandardWidth('RAlt'), equals(50));
    });

    test('returns correct width for LWin', () {
      expect(KeyButton.getStandardWidth('LWin'), equals(50));
    });

    test('returns correct width for RWin', () {
      expect(KeyButton.getStandardWidth('RWin'), equals(50));
    });

    test('returns correct width for Menu', () {
      expect(KeyButton.getStandardWidth('Menu'), equals(50));
    });

    test('returns default width for standard keys', () {
      expect(KeyButton.getStandardWidth('A'), equals(40));
      expect(KeyButton.getStandardWidth('1'), equals(40));
      expect(KeyButton.getStandardWidth('F1'), equals(40));
    });
  });
}
