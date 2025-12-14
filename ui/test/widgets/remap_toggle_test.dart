import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:keyrx_ui/widgets/remap_toggle.dart';

import '../helpers/widget_test_harness.dart';

void main() {
  final harness = WidgetTestHarness();

  testWidgets('RemapToggle displays ON label when enabled', (
    WidgetTester tester,
  ) async {
    await harness.pumpWidget(
      tester,
      const RemapToggle(enabled: true),
    );

    expect(find.text('ON'), findsOneWidget);
    expect(find.text('OFF'), findsNothing);
    expect(find.byType(Switch), findsOneWidget);
  });

  testWidgets('RemapToggle displays OFF label when disabled', (
    WidgetTester tester,
  ) async {
    await harness.pumpWidget(
      tester,
      const RemapToggle(enabled: false),
    );

    expect(find.text('OFF'), findsOneWidget);
    expect(find.text('ON'), findsNothing);
  });

  testWidgets('RemapToggle calls onChanged when switch is toggled', (
    WidgetTester tester,
  ) async {
    bool? changedValue;

    await harness.pumpWidget(
      tester,
      RemapToggle(
        enabled: false,
        onChanged: (value) => changedValue = value,
      ),
    );

    await tester.tapAndSettle(find.byType(Switch));

    expect(changedValue, isTrue);
  });

  testWidgets('RemapToggle can be disabled when onChanged is null', (
    WidgetTester tester,
  ) async {
    await harness.pumpWidget(
      tester,
      const RemapToggle(enabled: true, onChanged: null),
    );

    final switchWidget = tester.widget<Switch>(find.byType(Switch));
    expect(switchWidget.onChanged, isNull);
  });

  testWidgets('RemapToggle switch reflects enabled state', (
    WidgetTester tester,
  ) async {
    await harness.pumpWidget(
      tester,
      const RemapToggle(enabled: true),
    );

    final switchWidget = tester.widget<Switch>(find.byType(Switch));
    expect(switchWidget.value, isTrue);
  });
}
