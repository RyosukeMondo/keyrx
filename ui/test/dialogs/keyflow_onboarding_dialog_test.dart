import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/dialogs/keyflow_onboarding_dialog.dart';
import 'package:shared_preferences/shared_preferences.dart';

void main() {
  setUp(() {
    // Reset shared preferences before each test
    SharedPreferences.setMockInitialValues({});
  });

  group('KeyFlowOnboardingDialog', () {
    testWidgets('should show dialog with welcome page', (
      WidgetTester tester,
    ) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Builder(
              builder: (context) => ElevatedButton(
                onPressed: () => KeyFlowOnboardingDialog.show(context),
                child: const Text('Show Onboarding'),
              ),
            ),
          ),
        ),
      );

      // Tap button to show dialog
      await tester.tap(find.text('Show Onboarding'));
      await tester.pumpAndSettle();

      // Verify welcome page is shown
      expect(find.text('Welcome to KeyFlow'), findsOneWidget);
      expect(
        find.text(
          'KeyFlow visualizes how your keyboard devices connect to profiles and keymaps.',
        ),
        findsOneWidget,
      );
    });

    testWidgets('should navigate through all pages', (
      WidgetTester tester,
    ) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Builder(
              builder: (context) => ElevatedButton(
                onPressed: () => KeyFlowOnboardingDialog.show(context),
                child: const Text('Show Onboarding'),
              ),
            ),
          ),
        ),
      );

      await tester.tap(find.text('Show Onboarding'));
      await tester.pumpAndSettle();

      // Page 1: Welcome
      expect(find.text('Welcome to KeyFlow'), findsOneWidget);

      // Navigate to page 2
      await tester.tap(find.text('Next'));
      await tester.pumpAndSettle();
      expect(find.text('Devices, Profiles & Keymaps'), findsOneWidget);

      // Navigate to page 3
      await tester.tap(find.text('Next'));
      await tester.pumpAndSettle();
      expect(find.text('Slots & Priority'), findsOneWidget);

      // Navigate to page 4
      await tester.tap(find.text('Next'));
      await tester.pumpAndSettle();
      expect(find.text('Interact with the Flow'), findsOneWidget);

      // Navigate to page 5
      await tester.tap(find.text('Next'));
      await tester.pumpAndSettle();
      expect(find.text('Keyboard Shortcuts'), findsOneWidget);

      // Last page should show "Get Started" button
      expect(find.text('Get Started'), findsOneWidget);
    });

    testWidgets('should go back to previous page', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Builder(
              builder: (context) => ElevatedButton(
                onPressed: () => KeyFlowOnboardingDialog.show(context),
                child: const Text('Show Onboarding'),
              ),
            ),
          ),
        ),
      );

      await tester.tap(find.text('Show Onboarding'));
      await tester.pumpAndSettle();

      // Go to page 2
      await tester.tap(find.text('Next'));
      await tester.pumpAndSettle();
      expect(find.text('Devices, Profiles & Keymaps'), findsOneWidget);

      // Go back to page 1
      await tester.tap(find.text('Previous'));
      await tester.pumpAndSettle();
      expect(find.text('Welcome to KeyFlow'), findsOneWidget);
    });

    testWidgets('should skip onboarding', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Builder(
              builder: (context) => ElevatedButton(
                onPressed: () => KeyFlowOnboardingDialog.show(context),
                child: const Text('Show Onboarding'),
              ),
            ),
          ),
        ),
      );

      await tester.tap(find.text('Show Onboarding'));
      await tester.pumpAndSettle();

      // Skip the onboarding
      await tester.tap(find.text('Skip'));
      await tester.pumpAndSettle();

      // Dialog should be closed
      expect(find.text('Welcome to KeyFlow'), findsNothing);
    });

    testWidgets('should mark as seen when "Don\'t show again" is checked', (
      WidgetTester tester,
    ) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Builder(
              builder: (context) => ElevatedButton(
                onPressed: () => KeyFlowOnboardingDialog.show(context),
                child: const Text('Show Onboarding'),
              ),
            ),
          ),
        ),
      );

      await tester.tap(find.text('Show Onboarding'));
      await tester.pumpAndSettle();

      // Check "Don't show again"
      await tester.tap(find.byType(Checkbox));
      await tester.pumpAndSettle();

      // Navigate to last page and finish
      for (var i = 0; i < 4; i++) {
        await tester.tap(find.text('Next'));
        await tester.pumpAndSettle();
      }

      await tester.tap(find.text('Get Started'));
      await tester.pumpAndSettle();

      // Verify it was marked as seen
      final shouldShow = await KeyFlowOnboardingDialog.shouldShow();
      expect(shouldShow, isFalse);
    });

    testWidgets('should show page indicators', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Builder(
              builder: (context) => ElevatedButton(
                onPressed: () => KeyFlowOnboardingDialog.show(context),
                child: const Text('Show Onboarding'),
              ),
            ),
          ),
        ),
      );

      await tester.tap(find.text('Show Onboarding'));
      await tester.pumpAndSettle();

      // Find all page indicator dots (5 pages)
      final indicators = find.byType(Container).evaluate().where((element) {
        final widget = element.widget as Container;
        final decoration = widget.decoration as BoxDecoration?;
        return decoration?.shape == BoxShape.circle;
      });

      expect(indicators.length, 5);
    });

    testWidgets('should close dialog when close button is tapped', (
      WidgetTester tester,
    ) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Builder(
              builder: (context) => ElevatedButton(
                onPressed: () => KeyFlowOnboardingDialog.show(context),
                child: const Text('Show Onboarding'),
              ),
            ),
          ),
        ),
      );

      await tester.tap(find.text('Show Onboarding'));
      await tester.pumpAndSettle();

      // Tap close button
      final closeButton = find.byIcon(Icons.close);
      await tester.tap(closeButton);
      await tester.pumpAndSettle();

      // Dialog should be closed
      expect(find.text('Welcome to KeyFlow'), findsNothing);
    });

    test('shouldShow returns true when not seen', () async {
      SharedPreferences.setMockInitialValues({});
      final shouldShow = await KeyFlowOnboardingDialog.shouldShow();
      expect(shouldShow, isTrue);
    });

    test('shouldShow returns false when already seen', () async {
      SharedPreferences.setMockInitialValues({'keyflow_onboarding_seen': true});
      final shouldShow = await KeyFlowOnboardingDialog.shouldShow();
      expect(shouldShow, isFalse);
    });

    test('markAsSeen sets the preference', () async {
      SharedPreferences.setMockInitialValues({});
      await KeyFlowOnboardingDialog.markAsSeen();

      final shouldShow = await KeyFlowOnboardingDialog.shouldShow();
      expect(shouldShow, isFalse);
    });
  });
}
