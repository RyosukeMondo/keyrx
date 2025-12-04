import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/widgets/common/dialogs/dialog_helpers.dart';
import 'package:keyrx_ui/widgets/common/dialogs/selection_dialog.dart';
import 'package:keyrx_ui/widgets/common/dialogs/multi_action_dialog.dart';

void main() {
  group('DialogHelpers.confirm', () {
    testWidgets('shows confirmation dialog', (WidgetTester tester) async {
      late BuildContext testContext;

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) {
              testContext = context;
              return const Scaffold(body: SizedBox());
            },
          ),
        ),
      );

      final future = DialogHelpers.confirm(
        testContext,
        title: 'Confirm Action',
        message: 'Are you sure?',
      );

      await tester.pumpAndSettle();

      expect(find.text('Confirm Action'), findsOneWidget);
      expect(find.text('Are you sure?'), findsOneWidget);
      expect(find.text('Confirm'), findsOneWidget);
      expect(find.text('Cancel'), findsOneWidget);

      await tester.tap(find.text('Confirm'));
      await tester.pumpAndSettle();

      expect(await future, isTrue);
    });

    testWidgets('returns false when cancelled', (WidgetTester tester) async {
      late BuildContext testContext;

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) {
              testContext = context;
              return const Scaffold(body: SizedBox());
            },
          ),
        ),
      );

      final future = DialogHelpers.confirm(
        testContext,
        title: 'Confirm Action',
        message: 'Are you sure?',
      );

      await tester.pumpAndSettle();

      await tester.tap(find.text('Cancel'));
      await tester.pumpAndSettle();

      expect(await future, isFalse);
    });

    testWidgets('uses custom labels', (WidgetTester tester) async {
      late BuildContext testContext;

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) {
              testContext = context;
              return const Scaffold(body: SizedBox());
            },
          ),
        ),
      );

      DialogHelpers.confirm(
        testContext,
        title: 'Custom',
        message: 'Message',
        confirmLabel: 'Yes',
        cancelLabel: 'No',
      );

      await tester.pumpAndSettle();

      expect(find.text('Yes'), findsOneWidget);
      expect(find.text('No'), findsOneWidget);
    });
  });

  group('DialogHelpers.confirmDelete', () {
    testWidgets('shows delete confirmation', (WidgetTester tester) async {
      late BuildContext testContext;

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) {
              testContext = context;
              return const Scaffold(body: SizedBox());
            },
          ),
        ),
      );

      DialogHelpers.confirmDelete(
        testContext,
        title: 'Delete Item',
        message: 'This cannot be undone',
      );

      await tester.pumpAndSettle();

      expect(find.text('Delete Item'), findsOneWidget);
      expect(find.text('This cannot be undone'), findsOneWidget);
      expect(find.text('Delete'), findsOneWidget);
      expect(find.byIcon(Icons.delete_outline), findsOneWidget);
    });
  });

  group('DialogHelpers.confirmClear', () {
    testWidgets('shows clear confirmation', (WidgetTester tester) async {
      late BuildContext testContext;

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) {
              testContext = context;
              return const Scaffold(body: SizedBox());
            },
          ),
        ),
      );

      DialogHelpers.confirmClear(
        testContext,
        title: 'Clear Data',
        message: 'All data will be removed',
      );

      await tester.pumpAndSettle();

      expect(find.text('Clear Data'), findsOneWidget);
      expect(find.text('All data will be removed'), findsOneWidget);
      expect(find.text('Clear'), findsOneWidget);
      expect(find.byIcon(Icons.warning_amber), findsOneWidget);
    });
  });

  group('DialogHelpers.input', () {
    testWidgets('shows input dialog', (WidgetTester tester) async {
      late BuildContext testContext;

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) {
              testContext = context;
              return const Scaffold(body: SizedBox());
            },
          ),
        ),
      );

      final future = DialogHelpers.input(
        testContext,
        title: 'Enter Name',
        labelText: 'Name',
        hintText: 'John Doe',
      );

      await tester.pumpAndSettle();

      expect(find.text('Enter Name'), findsOneWidget);
      expect(find.text('Name'), findsOneWidget);
      expect(find.text('John Doe'), findsOneWidget);

      await tester.enterText(find.byType(TextField), 'Test User');
      await tester.tap(find.text('OK'));
      await tester.pumpAndSettle();

      expect(await future, equals('Test User'));
    });

    testWidgets('returns null when cancelled', (WidgetTester tester) async {
      late BuildContext testContext;

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) {
              testContext = context;
              return const Scaffold(body: SizedBox());
            },
          ),
        ),
      );

      final future = DialogHelpers.input(
        testContext,
        title: 'Enter Name',
        labelText: 'Name',
      );

      await tester.pumpAndSettle();

      await tester.tap(find.text('Cancel'));
      await tester.pumpAndSettle();

      expect(await future, isNull);
    });

    testWidgets('validates input', (WidgetTester tester) async {
      late BuildContext testContext;

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) {
              testContext = context;
              return const Scaffold(body: SizedBox());
            },
          ),
        ),
      );

      DialogHelpers.input(
        testContext,
        title: 'Enter Name',
        labelText: 'Name',
        validator: (value) {
          if (value == null || value.isEmpty) {
            return 'Name is required';
          }
          return null;
        },
      );

      await tester.pumpAndSettle();

      await tester.tap(find.text('OK'));
      await tester.pumpAndSettle();

      expect(find.text('Name is required'), findsOneWidget);
    });
  });

  group('DialogHelpers.inputPath', () {
    testWidgets('shows path input dialog', (WidgetTester tester) async {
      late BuildContext testContext;

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) {
              testContext = context;
              return const Scaffold(body: SizedBox());
            },
          ),
        ),
      );

      DialogHelpers.inputPath(
        testContext,
        title: 'Enter Path',
      );

      await tester.pumpAndSettle();

      expect(find.text('Enter Path'), findsOneWidget);
      expect(find.text('Path'), findsOneWidget);
      expect(find.byIcon(Icons.folder_outlined), findsOneWidget);
    });

    testWidgets('validates empty path', (WidgetTester tester) async {
      late BuildContext testContext;

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) {
              testContext = context;
              return const Scaffold(body: SizedBox());
            },
          ),
        ),
      );

      DialogHelpers.inputPath(
        testContext,
        title: 'Enter Path',
      );

      await tester.pumpAndSettle();

      await tester.tap(find.text('OK'));
      await tester.pumpAndSettle();

      expect(find.text('Path cannot be empty'), findsOneWidget);
    });
  });

  group('DialogHelpers.select', () {
    testWidgets('shows selection dialog', (WidgetTester tester) async {
      late BuildContext testContext;

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) {
              testContext = context;
              return const Scaffold(body: SizedBox());
            },
          ),
        ),
      );

      final future = DialogHelpers.select<String>(
        testContext,
        title: 'Choose Color',
        options: [
          SelectionOption(label: 'Red', value: 'red'),
          SelectionOption(label: 'Green', value: 'green'),
          SelectionOption(label: 'Blue', value: 'blue'),
        ],
      );

      await tester.pumpAndSettle();

      expect(find.text('Choose Color'), findsOneWidget);
      expect(find.text('Red'), findsOneWidget);
      expect(find.text('Green'), findsOneWidget);
      expect(find.text('Blue'), findsOneWidget);

      await tester.tap(find.text('Green'));
      await tester.pumpAndSettle();

      expect(await future, equals('green'));
    });
  });

  group('DialogHelpers.info', () {
    testWidgets('shows info dialog', (WidgetTester tester) async {
      late BuildContext testContext;

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) {
              testContext = context;
              return const Scaffold(body: SizedBox());
            },
          ),
        ),
      );

      DialogHelpers.info(
        testContext,
        title: 'Information',
        content: const Text('Some information here'),
      );

      await tester.pumpAndSettle();

      expect(find.text('Information'), findsOneWidget);
      expect(find.text('Some information here'), findsOneWidget);
      expect(find.text('Close'), findsOneWidget);
    });
  });

  group('DialogHelpers.help', () {
    testWidgets('shows help dialog', (WidgetTester tester) async {
      late BuildContext testContext;

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) {
              testContext = context;
              return const Scaffold(body: SizedBox());
            },
          ),
        ),
      );

      DialogHelpers.help(
        testContext,
        title: 'Help',
        content: const Text('Help content'),
      );

      await tester.pumpAndSettle();

      expect(find.text('Help'), findsOneWidget);
      expect(find.text('Help content'), findsOneWidget);
      expect(find.text('Got it'), findsOneWidget);
      expect(find.byIcon(Icons.help_outline), findsOneWidget);
    });
  });

  group('DialogHelpers.multiAction', () {
    testWidgets('shows multi-action dialog', (WidgetTester tester) async {
      late BuildContext testContext;

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) {
              testContext = context;
              return const Scaffold(body: SizedBox());
            },
          ),
        ),
      );

      final future = DialogHelpers.multiAction<String>(
        testContext,
        title: 'Choose Action',
        message: 'What would you like to do?',
        actions: [
          DialogAction(label: 'Save', value: 'save', isPrimary: true),
          DialogAction(label: 'Discard', value: 'discard'),
          DialogAction(label: 'Cancel', value: 'cancel'),
        ],
      );

      await tester.pumpAndSettle();

      expect(find.text('Choose Action'), findsOneWidget);
      expect(find.text('What would you like to do?'), findsOneWidget);
      expect(find.text('Save'), findsOneWidget);
      expect(find.text('Discard'), findsOneWidget);
      expect(find.text('Cancel'), findsOneWidget);

      await tester.tap(find.text('Save'));
      await tester.pumpAndSettle();

      expect(await future, equals('save'));
    });
  });

  group('DialogHelpers.threeWayChoice', () {
    testWidgets('shows three-way choice dialog', (WidgetTester tester) async {
      late BuildContext testContext;

      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) {
              testContext = context;
              return const Scaffold(body: SizedBox());
            },
          ),
        ),
      );

      final future = DialogHelpers.threeWayChoice(
        testContext,
        title: 'Sync Conflict',
        message: 'Choose resolution',
        primaryLabel: 'Use Local',
        secondaryLabel: 'Use Remote',
      );

      await tester.pumpAndSettle();

      expect(find.text('Sync Conflict'), findsOneWidget);
      expect(find.text('Choose resolution'), findsOneWidget);
      expect(find.text('Use Local'), findsOneWidget);
      expect(find.text('Use Remote'), findsOneWidget);
      expect(find.text('Cancel'), findsOneWidget);

      await tester.tap(find.text('Use Local'));
      await tester.pumpAndSettle();

      expect(await future, equals(1));
    });
  });
}
