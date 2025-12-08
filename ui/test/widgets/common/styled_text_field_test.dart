import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/widgets/common/styled_text_field.dart';

void main() {
  group('StyledTextField', () {
    testWidgets('renders with label', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(body: StyledTextField(labelText: 'Name')),
        ),
      );

      expect(find.text('Name'), findsOneWidget);
    });

    testWidgets('calls onChanged when text changes', (
      WidgetTester tester,
    ) async {
      String? changedValue;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: StyledTextField(onChanged: (value) => changedValue = value),
          ),
        ),
      );

      await tester.enterText(find.byType(TextField), 'test');
      expect(changedValue, equals('test'));
    });

    testWidgets('calls onSubmitted when submitted', (
      WidgetTester tester,
    ) async {
      String? submittedValue;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: StyledTextField(
              onSubmitted: (value) => submittedValue = value,
            ),
          ),
        ),
      );

      await tester.enterText(find.byType(TextField), 'test');
      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.pump();

      expect(submittedValue, equals('test'));
    });

    testWidgets('respects enabled state', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(home: Scaffold(body: StyledTextField(enabled: false))),
      );

      final textField = tester.widget<TextField>(find.byType(TextField));
      expect(textField.enabled, isFalse);
    });

    testWidgets('supports multiline', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(home: Scaffold(body: StyledTextField(maxLines: 5))),
      );

      final textField = tester.widget<TextField>(find.byType(TextField));
      expect(textField.maxLines, equals(5));
    });

    testWidgets('shows hint text', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(body: StyledTextField(hintText: 'Enter text here')),
        ),
      );

      expect(find.text('Enter text here'), findsOneWidget);
    });

    testWidgets('displays prefix and suffix icons', (
      WidgetTester tester,
    ) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: StyledTextField(
              prefixIcon: const Icon(Icons.search),
              suffixIcon: const Icon(Icons.clear),
            ),
          ),
        ),
      );

      expect(find.byIcon(Icons.search), findsOneWidget);
      expect(find.byIcon(Icons.clear), findsOneWidget);
    });

    testWidgets('uses custom decoration when provided', (
      WidgetTester tester,
    ) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: StyledTextField(
              decoration: const InputDecoration(
                labelText: 'Custom',
                helperText: 'Helper',
              ),
            ),
          ),
        ),
      );

      expect(find.text('Custom'), findsOneWidget);
      expect(find.text('Helper'), findsOneWidget);
    });
  });

  group('OutlinedTextField', () {
    testWidgets('renders with outline border', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(body: OutlinedTextField(labelText: 'Email')),
        ),
      );

      expect(find.text('Email'), findsOneWidget);
      final textField = tester.widget<TextField>(find.byType(TextField));
      expect(textField.decoration?.border, isA<OutlineInputBorder>());
    });

    testWidgets('calls onChanged when text changes', (
      WidgetTester tester,
    ) async {
      String? changedValue;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: OutlinedTextField(onChanged: (value) => changedValue = value),
          ),
        ),
      );

      await tester.enterText(find.byType(TextField), 'test@example.com');
      expect(changedValue, equals('test@example.com'));
    });

    testWidgets('respects enabled state', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(home: Scaffold(body: OutlinedTextField(enabled: false))),
      );

      final textField = tester.widget<TextField>(find.byType(TextField));
      expect(textField.enabled, isFalse);
    });
  });

  group('ExpandingTextField', () {
    testWidgets('expands to fill space', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(
              height: 200,
              child: ExpandingTextField(hintText: 'Enter code'),
            ),
          ),
        ),
      );

      final textField = tester.widget<TextField>(find.byType(TextField));
      expect(textField.expands, isTrue);
      expect(textField.maxLines, isNull);
    });

    testWidgets('calls onChanged when text changes', (
      WidgetTester tester,
    ) async {
      String? changedValue;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(
              height: 200,
              child: ExpandingTextField(
                onChanged: (value) => changedValue = value,
              ),
            ),
          ),
        ),
      );

      await tester.enterText(find.byType(TextField), 'multi\nline\ntext');
      expect(changedValue, equals('multi\nline\ntext'));
    });

    testWidgets('uses monospace font', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(height: 200, child: ExpandingTextField()),
          ),
        ),
      );

      final textField = tester.widget<TextField>(find.byType(TextField));
      expect(textField.style?.fontFamily, equals('monospace'));
    });

    testWidgets('respects enabled state', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(
              height: 200,
              child: ExpandingTextField(enabled: false),
            ),
          ),
        ),
      );

      final textField = tester.widget<TextField>(find.byType(TextField));
      expect(textField.enabled, isFalse);
    });
  });

  group('NumberTextField', () {
    testWidgets('accepts only digits by default', (WidgetTester tester) async {
      final controller = TextEditingController();

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(body: NumberTextField(controller: controller)),
        ),
      );

      await tester.enterText(find.byType(TextField), '123abc');
      expect(controller.text, equals('123'));
    });

    testWidgets('allows decimal when enabled', (WidgetTester tester) async {
      final controller = TextEditingController();

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: NumberTextField(controller: controller, allowDecimal: true),
          ),
        ),
      );

      await tester.enterText(find.byType(TextField), '123.45');
      expect(controller.text, equals('123.45'));
    });

    testWidgets('allows negative when enabled', (WidgetTester tester) async {
      final controller = TextEditingController();

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: NumberTextField(controller: controller, allowNegative: true),
          ),
        ),
      );

      await tester.enterText(find.byType(TextField), '-123');
      expect(controller.text, equals('-123'));
    });

    testWidgets('allows decimal and negative when both enabled', (
      WidgetTester tester,
    ) async {
      final controller = TextEditingController();

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: NumberTextField(
              controller: controller,
              allowDecimal: true,
              allowNegative: true,
            ),
          ),
        ),
      );

      await tester.enterText(find.byType(TextField), '-123.45');
      expect(controller.text, equals('-123.45'));
    });

    testWidgets('renders with outline border when outlined is true', (
      WidgetTester tester,
    ) async {
      await tester.pumpWidget(
        MaterialApp(home: Scaffold(body: NumberTextField(outlined: true))),
      );

      final textField = tester.widget<TextField>(find.byType(TextField));
      expect(textField.decoration?.border, isA<OutlineInputBorder>());
    });

    testWidgets('calls onChanged when text changes', (
      WidgetTester tester,
    ) async {
      String? changedValue;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: NumberTextField(onChanged: (value) => changedValue = value),
          ),
        ),
      );

      await tester.enterText(find.byType(TextField), '42');
      expect(changedValue, equals('42'));
    });
  });
}
