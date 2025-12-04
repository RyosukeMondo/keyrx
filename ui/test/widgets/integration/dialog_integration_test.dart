import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/widgets/common/dialogs/dialog_helpers.dart';
import 'package:keyrx_ui/widgets/common/dialogs/confirmation_dialog.dart';
import 'package:keyrx_ui/widgets/common/dialogs/input_dialog.dart';
import 'package:keyrx_ui/widgets/common/dialogs/selection_dialog.dart';
import 'package:keyrx_ui/widgets/common/styled_text_field.dart';

/// Integration tests for dialog helpers and dialog widgets.
void main() {
  group('Dialog Integration', () {
    testWidgets('DialogHelpers.confirm shows ConfirmationDialog',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Builder(
              builder: (context) {
                return ElevatedButton(
                  onPressed: () {
                    DialogHelpers.confirm(
                      context,
                      title: 'Test',
                      message: 'Confirm this action?',
                    );
                  },
                  child: const Text('Show Dialog'),
                );
              },
            ),
          ),
        ),
      );

      // Tap button to show dialog
      await tester.tap(find.text('Show Dialog'));
      await tester.pumpAndSettle();

      // Verify ConfirmationDialog is shown
      expect(find.byType(ConfirmationDialog), findsOneWidget);
      expect(find.text('Test'), findsOneWidget);
      expect(find.text('Confirm this action?'), findsOneWidget);
    });

    testWidgets('DialogHelpers.confirm returns true when confirmed',
        (WidgetTester tester) async {
      bool? result;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Builder(
              builder: (context) {
                return ElevatedButton(
                  onPressed: () async {
                    result = await DialogHelpers.confirm(
                      context,
                      title: 'Test',
                      message: 'Confirm?',
                    );
                  },
                  child: const Text('Show Dialog'),
                );
              },
            ),
          ),
        ),
      );

      await tester.tap(find.text('Show Dialog'));
      await tester.pumpAndSettle();

      // Tap confirm button
      await tester.tap(find.text('Confirm'));
      await tester.pumpAndSettle();

      expect(result, isTrue);
    });

    testWidgets('DialogHelpers.confirm returns false when cancelled',
        (WidgetTester tester) async {
      bool? result;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Builder(
              builder: (context) {
                return ElevatedButton(
                  onPressed: () async {
                    result = await DialogHelpers.confirm(
                      context,
                      title: 'Test',
                      message: 'Confirm?',
                    );
                  },
                  child: const Text('Show Dialog'),
                );
              },
            ),
          ),
        ),
      );

      await tester.tap(find.text('Show Dialog'));
      await tester.pumpAndSettle();

      // Tap cancel button
      await tester.tap(find.text('Cancel'));
      await tester.pumpAndSettle();

      expect(result, isFalse);
    });

    testWidgets('DialogHelpers.input shows InputDialog with StyledTextField',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Builder(
              builder: (context) {
                return ElevatedButton(
                  onPressed: () {
                    DialogHelpers.input(
                      context,
                      title: 'Enter Name',
                      labelText: 'Name',
                    );
                  },
                  child: const Text('Show Dialog'),
                );
              },
            ),
          ),
        ),
      );

      await tester.tap(find.text('Show Dialog'));
      await tester.pumpAndSettle();

      // Verify InputDialog and StyledTextField are shown
      expect(find.byType(InputDialog), findsOneWidget);
      expect(find.byType(StyledTextField), findsOneWidget);
    });

    testWidgets('DialogHelpers.input returns entered text',
        (WidgetTester tester) async {
      String? result;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Builder(
              builder: (context) {
                return ElevatedButton(
                  onPressed: () async {
                    result = await DialogHelpers.input(
                      context,
                      title: 'Enter Name',
                      labelText: 'Name',
                    );
                  },
                  child: const Text('Show Dialog'),
                );
              },
            ),
          ),
        ),
      );

      await tester.tap(find.text('Show Dialog'));
      await tester.pumpAndSettle();

      // Enter text
      await tester.enterText(find.byType(StyledTextField), 'John Doe');
      await tester.pumpAndSettle();

      // Tap OK button
      await tester.tap(find.text('OK'));
      await tester.pumpAndSettle();

      expect(result, equals('John Doe'));
    });

    testWidgets('DialogHelpers.input validates input',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Builder(
              builder: (context) {
                return ElevatedButton(
                  onPressed: () {
                    DialogHelpers.input(
                      context,
                      title: 'Enter Name',
                      labelText: 'Name',
                      validator: (value) {
                        if (value == null || value.isEmpty) {
                          return 'Name is required';
                        }
                        return null;
                      },
                    );
                  },
                  child: const Text('Show Dialog'),
                );
              },
            ),
          ),
        ),
      );

      await tester.tap(find.text('Show Dialog'));
      await tester.pumpAndSettle();

      // Try to submit without entering text
      await tester.tap(find.text('OK'));
      await tester.pumpAndSettle();

      // Validation error should be shown
      expect(find.text('Name is required'), findsOneWidget);
    });

    testWidgets('DialogHelpers.select shows SelectionDialog',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Builder(
              builder: (context) {
                return ElevatedButton(
                  onPressed: () {
                    DialogHelpers.select<String>(
                      context,
                      title: 'Choose Option',
                      options: [
                        const SelectionOption(label: 'Option 1', value: 'opt1'),
                        const SelectionOption(label: 'Option 2', value: 'opt2'),
                      ],
                    );
                  },
                  child: const Text('Show Dialog'),
                );
              },
            ),
          ),
        ),
      );

      await tester.tap(find.text('Show Dialog'));
      await tester.pumpAndSettle();

      // Verify SelectionDialog is shown with options
      expect(find.byType(SelectionDialog<String>), findsOneWidget);
      expect(find.text('Option 1'), findsOneWidget);
      expect(find.text('Option 2'), findsOneWidget);
    });

    testWidgets('DialogHelpers.select returns selected value',
        (WidgetTester tester) async {
      String? result;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Builder(
              builder: (context) {
                return ElevatedButton(
                  onPressed: () async {
                    result = await DialogHelpers.select<String>(
                      context,
                      title: 'Choose Option',
                      options: [
                        const SelectionOption(label: 'Option 1', value: 'opt1'),
                        const SelectionOption(label: 'Option 2', value: 'opt2'),
                      ],
                    );
                  },
                  child: const Text('Show Dialog'),
                );
              },
            ),
          ),
        ),
      );

      await tester.tap(find.text('Show Dialog'));
      await tester.pumpAndSettle();

      // Tap an option
      await tester.tap(find.text('Option 2'));
      await tester.pumpAndSettle();

      expect(result, equals('opt2'));
    });

    testWidgets('DialogHelpers.confirmDelete shows destructive dialog',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Builder(
              builder: (context) {
                return ElevatedButton(
                  onPressed: () {
                    DialogHelpers.confirmDelete(
                      context,
                      title: 'Delete Item',
                      message: 'This cannot be undone',
                    );
                  },
                  child: const Text('Show Dialog'),
                );
              },
            ),
          ),
        ),
      );

      await tester.tap(find.text('Show Dialog'));
      await tester.pumpAndSettle();

      // Verify destructive styling (delete button)
      expect(find.text('Delete'), findsOneWidget);
      expect(find.byIcon(Icons.delete_outline), findsOneWidget);
    });

    testWidgets('Dialog workflow: open, input, validate, submit',
        (WidgetTester tester) async {
      String? result;
      var submitCount = 0;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Builder(
              builder: (context) {
                return ElevatedButton(
                  onPressed: () async {
                    result = await DialogHelpers.input(
                      context,
                      title: 'Enter Path',
                      labelText: 'Path',
                      validator: (value) {
                        if (value == null || value.isEmpty) {
                          return 'Required';
                        }
                        if (!value.endsWith('.txt')) {
                          return 'Must be a .txt file';
                        }
                        return null;
                      },
                    );
                    if (result != null) {
                      submitCount++;
                    }
                  },
                  child: const Text('Show Dialog'),
                );
              },
            ),
          ),
        ),
      );

      // Open dialog
      await tester.tap(find.text('Show Dialog'));
      await tester.pumpAndSettle();

      // Try empty input
      await tester.tap(find.text('OK'));
      await tester.pumpAndSettle();
      expect(find.text('Required'), findsOneWidget);

      // Try invalid input
      final textField = find.byType(TextField);
      await tester.enterText(textField, 'file.md');
      await tester.pumpAndSettle();
      await tester.tap(find.text('OK'));
      await tester.pumpAndSettle();
      expect(find.text('Must be a .txt file'), findsOneWidget);

      // Enter valid input
      await tester.enterText(textField, 'document.txt');
      await tester.pumpAndSettle();
      await tester.tap(find.text('OK'));
      await tester.pumpAndSettle();

      expect(result, equals('document.txt'));
      expect(submitCount, equals(1));
    });

    testWidgets('Multiple dialogs can be chained',
        (WidgetTester tester) async {
      bool? confirmed;
      String? name;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: Builder(
              builder: (context) {
                return ElevatedButton(
                  onPressed: () async {
                    confirmed = await DialogHelpers.confirm(
                      context,
                      title: 'Proceed?',
                      message: 'Continue with action?',
                    );

                    if (confirmed == true) {
                      name = await DialogHelpers.input(
                        context,
                        title: 'Enter Name',
                        labelText: 'Name',
                      );
                    }
                  },
                  child: const Text('Start'),
                );
              },
            ),
          ),
        ),
      );

      // Start workflow
      await tester.tap(find.text('Start'));
      await tester.pumpAndSettle();

      // Confirm first dialog
      expect(find.text('Proceed?'), findsOneWidget);
      await tester.tap(find.text('Confirm'));
      await tester.pumpAndSettle();

      // Second dialog should appear
      expect(find.text('Enter Name'), findsOneWidget);
      final nameField = find.byType(TextField);
      await tester.enterText(nameField, 'Test User');
      await tester.pumpAndSettle();
      await tester.tap(find.text('OK'));
      await tester.pumpAndSettle();

      expect(confirmed, isTrue);
      expect(name, equals('Test User'));
    });
  });
}
