import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/widgets/common/styled_icon_button.dart';

void main() {
  group('StyledIconButton', () {
    testWidgets('renders icon correctly', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: StyledIconButton(
              icon: Icons.delete,
              onPressed: () {},
            ),
          ),
        ),
      );

      expect(find.byIcon(Icons.delete), findsOneWidget);
    });

    testWidgets('calls onPressed when tapped', (WidgetTester tester) async {
      var pressed = false;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: StyledIconButton(
              icon: Icons.delete,
              onPressed: () => pressed = true,
            ),
          ),
        ),
      );

      await tester.tap(find.byType(StyledIconButton));
      await tester.pumpAndSettle();

      expect(pressed, isTrue);
    });

    testWidgets('displays tooltip when provided', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: StyledIconButton(
              icon: Icons.delete,
              onPressed: () {},
              tooltip: 'Delete item',
            ),
          ),
        ),
      );

      expect(find.byType(Tooltip), findsOneWidget);

      // Long press to show tooltip
      final gesture = await tester.startGesture(
        tester.getCenter(find.byIcon(Icons.delete)),
      );
      await tester.pump(const Duration(milliseconds: 500));
      await tester.pumpAndSettle();

      expect(find.text('Delete item'), findsOneWidget);

      await gesture.up();
      await tester.pumpAndSettle();
    });

    testWidgets('does not display tooltip when not provided',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: StyledIconButton(
              icon: Icons.delete,
              onPressed: () {},
            ),
          ),
        ),
      );

      expect(find.byType(Tooltip), findsNothing);
    });

    testWidgets('is disabled when onPressed is null',
        (WidgetTester tester) async {
      var pressed = false;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: StyledIconButton(
              icon: Icons.delete,
              onPressed: null,
            ),
          ),
        ),
      );

      await tester.tap(find.byType(StyledIconButton));
      await tester.pumpAndSettle();

      expect(pressed, isFalse);
    });

    testWidgets('uses custom color when provided',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: StyledIconButton(
              icon: Icons.delete,
              onPressed: () {},
              color: Colors.red,
            ),
          ),
        ),
      );

      final icon = tester.widget<Icon>(find.byIcon(Icons.delete));
      expect(icon.color, equals(Colors.red));
    });

    testWidgets('uses custom size when provided',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: StyledIconButton(
              icon: Icons.delete,
              onPressed: () {},
              size: 32.0,
            ),
          ),
        ),
      );

      final icon = tester.widget<Icon>(find.byIcon(Icons.delete));
      expect(icon.size, equals(32.0));
    });
  });

  group('CompactIconButton', () {
    testWidgets('renders with smaller size', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: CompactIconButton(
              icon: Icons.edit,
              onPressed: () {},
            ),
          ),
        ),
      );

      final icon = tester.widget<Icon>(find.byIcon(Icons.edit));
      expect(icon.size, equals(18.0));
    });

    testWidgets('calls onPressed when tapped', (WidgetTester tester) async {
      var pressed = false;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: CompactIconButton(
              icon: Icons.edit,
              onPressed: () => pressed = true,
            ),
          ),
        ),
      );

      await tester.tap(find.byType(CompactIconButton));
      await tester.pumpAndSettle();

      expect(pressed, isTrue);
    });

    testWidgets('displays tooltip when provided', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: CompactIconButton(
              icon: Icons.edit,
              onPressed: () {},
              tooltip: 'Edit',
            ),
          ),
        ),
      );

      expect(find.byType(Tooltip), findsOneWidget);
    });
  });

  group('FilledIconButton', () {
    testWidgets('renders icon correctly', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: FilledIconButton(
              icon: Icons.add,
              onPressed: () {},
            ),
          ),
        ),
      );

      expect(find.byIcon(Icons.add), findsOneWidget);
    });

    testWidgets('calls onPressed when tapped', (WidgetTester tester) async {
      var pressed = false;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: FilledIconButton(
              icon: Icons.add,
              onPressed: () => pressed = true,
            ),
          ),
        ),
      );

      await tester.tap(find.byType(FilledIconButton));
      await tester.pumpAndSettle();

      expect(pressed, isTrue);
    });

    testWidgets('displays tooltip when provided', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: FilledIconButton(
              icon: Icons.add,
              onPressed: () {},
              tooltip: 'Add item',
            ),
          ),
        ),
      );

      expect(find.byType(Tooltip), findsOneWidget);
    });

    testWidgets('uses custom background color when provided',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: FilledIconButton(
              icon: Icons.add,
              onPressed: () {},
              backgroundColor: Colors.green,
            ),
          ),
        ),
      );

      final container = tester.widget<AnimatedContainer>(
        find.descendant(
          of: find.byType(FilledIconButton),
          matching: find.byType(AnimatedContainer),
        ),
      );

      final decoration = container.decoration as BoxDecoration;
      expect(decoration.color, equals(Colors.green));
    });

    testWidgets('uses custom foreground color when provided',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: FilledIconButton(
              icon: Icons.add,
              onPressed: () {},
              foregroundColor: Colors.yellow,
            ),
          ),
        ),
      );

      final icon = tester.widget<Icon>(find.byIcon(Icons.add));
      expect(icon.color, equals(Colors.yellow));
    });

    testWidgets('is disabled when onPressed is null',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: FilledIconButton(
              icon: Icons.add,
              onPressed: null,
            ),
          ),
        ),
      );

      final icon = tester.widget<Icon>(find.byIcon(Icons.add));
      expect(icon.color!.alpha, lessThan(255));
    });
  });
}
