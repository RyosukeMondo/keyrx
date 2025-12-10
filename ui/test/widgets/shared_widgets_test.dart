import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:keyrx_ui/ui/widgets/app_error_dialog.dart';
import 'package:keyrx_ui/ui/widgets/loading_overlay.dart';

void main() {
  testWidgets('AppErrorDialog calls actions and closes dialog', (
    WidgetTester tester,
  ) async {
    var primaryCalled = false;
    var secondaryCalled = false;

    await tester.pumpWidget(
      MaterialApp(
        home: Builder(
          builder: (context) => Scaffold(
            body: Center(
              child: ElevatedButton(
                onPressed: () => AppErrorDialog.show(
                  context,
                  title: 'Error title',
                  message: 'Error body',
                  primaryActionLabel: 'Confirm',
                  onPrimaryAction: () => primaryCalled = true,
                  secondaryActionLabel: 'Cancel',
                  onSecondaryAction: () => secondaryCalled = true,
                ),
                child: const Text('Open'),
              ),
            ),
          ),
        ),
      ),
    );

    await tester.tap(find.text('Open'));
    await tester.pumpAndSettle();

    expect(find.text('Error title'), findsOneWidget);

    await tester.tap(find.text('Cancel'));
    await tester.pumpAndSettle();

    expect(secondaryCalled, isTrue);
    expect(find.byType(AlertDialog), findsNothing);

    await tester.tap(find.text('Open'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Confirm'));
    await tester.pumpAndSettle();

    expect(primaryCalled, isTrue);
    expect(find.byType(AlertDialog), findsNothing);
  });

  testWidgets('LoadingOverlay blocks interaction while loading', (
    WidgetTester tester,
  ) async {
    var tapped = false;

    await tester.pumpWidget(
      MaterialApp(
        home: LoadingOverlay(
          isLoading: true,
          message: 'Please wait',
          child: ElevatedButton(
            onPressed: () => tapped = true,
            child: const Text('Tap me'),
          ),
        ),
      ),
    );

    expect(find.text('Please wait'), findsOneWidget);
    await tester.tap(find.text('Tap me'), warnIfMissed: false);
    expect(tapped, isFalse);

    await tester.pumpWidget(
      MaterialApp(
        home: LoadingOverlay(
          isLoading: false,
          child: ElevatedButton(
            onPressed: () => tapped = true,
            child: const Text('Tap me'),
          ),
        ),
      ),
    );

    await tester.tap(find.text('Tap me'));
    expect(tapped, isTrue);
  });
}
