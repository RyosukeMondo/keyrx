import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:keyrx_ui/widgets/remap_toggle.dart';

void main() {
  testWidgets('RemapToggle displays ON label when enabled',
      (WidgetTester tester) async {
    await tester.pumpWidget(
      const MaterialApp(
        home: Scaffold(
          body: RemapToggle(
            enabled: true,
          ),
        ),
      ),
    );

    expect(find.text('ON'), findsOneWidget);
    expect(find.text('OFF'), findsNothing);
    expect(find.byType(Switch), findsOneWidget);
  });

  testWidgets('RemapToggle displays OFF label when disabled',
      (WidgetTester tester) async {
    await tester.pumpWidget(
      const MaterialApp(
        home: Scaffold(
          body: RemapToggle(
            enabled: false,
          ),
        ),
      ),
    );

    expect(find.text('OFF'), findsOneWidget);
    expect(find.text('ON'), findsNothing);
  });

  testWidgets('RemapToggle calls onChanged when switch is toggled',
      (WidgetTester tester) async {
    bool? changedValue;

    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: RemapToggle(
            enabled: false,
            onChanged: (value) => changedValue = value,
          ),
        ),
      ),
    );

    await tester.tap(find.byType(Switch));
    await tester.pumpAndSettle();

    expect(changedValue, isTrue);
  });

  testWidgets('RemapToggle can be disabled when onChanged is null',
      (WidgetTester tester) async {
    await tester.pumpWidget(
      const MaterialApp(
        home: Scaffold(
          body: RemapToggle(
            enabled: true,
            onChanged: null,
          ),
        ),
      ),
    );

    final switchWidget = tester.widget<Switch>(find.byType(Switch));
    expect(switchWidget.onChanged, isNull);
  });

  testWidgets('RemapToggle switch reflects enabled state',
      (WidgetTester tester) async {
    await tester.pumpWidget(
      const MaterialApp(
        home: Scaffold(
          body: RemapToggle(
            enabled: true,
          ),
        ),
      ),
    );

    final switchWidget = tester.widget<Switch>(find.byType(Switch));
    expect(switchWidget.value, isTrue);
  });
}
