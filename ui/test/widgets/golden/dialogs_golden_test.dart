import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/widgets/common/dialogs/confirmation_dialog.dart';
import 'package:keyrx_ui/widgets/common/dialogs/input_dialog.dart';

void main() {
  group('ConfirmationDialog Golden Tests', () {
    testWidgets('renders basic confirmation dialog', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          theme: ThemeData.dark(),
          home: Scaffold(
            backgroundColor: Colors.black,
            body: ConfirmationDialog(
              title: 'Confirm Action',
              message: 'Are you sure you want to proceed?',
            ),
          ),
        ),
      );

      await expectLater(
        find.byType(ConfirmationDialog),
        matchesGoldenFile('goldens/confirmation_dialog_basic.png'),
      );
    });

    testWidgets('renders destructive confirmation dialog', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          theme: ThemeData.dark(),
          home: Scaffold(
            backgroundColor: Colors.black,
            body: ConfirmationDialog(
              title: 'Delete Item',
              message: 'This action cannot be undone.',
              confirmLabel: 'Delete',
              isDestructive: true,
              icon: Icons.warning,
            ),
          ),
        ),
      );

      await expectLater(
        find.byType(ConfirmationDialog),
        matchesGoldenFile('goldens/confirmation_dialog_destructive.png'),
      );
    });

    testWidgets('renders confirmation dialog with icon', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          theme: ThemeData.dark(),
          home: Scaffold(
            backgroundColor: Colors.black,
            body: ConfirmationDialog(
              title: 'Save Changes',
              message: 'Do you want to save your changes?',
              confirmLabel: 'Save',
              cancelLabel: 'Discard',
              icon: Icons.save,
            ),
          ),
        ),
      );

      await expectLater(
        find.byType(ConfirmationDialog),
        matchesGoldenFile('goldens/confirmation_dialog_with_icon.png'),
      );
    });

    testWidgets('renders confirmation dialog in light theme', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          theme: ThemeData.light(),
          home: Scaffold(
            backgroundColor: Colors.white,
            body: ConfirmationDialog(
              title: 'Confirm Action',
              message: 'Are you sure you want to proceed?',
            ),
          ),
        ),
      );

      await expectLater(
        find.byType(ConfirmationDialog),
        matchesGoldenFile('goldens/confirmation_dialog_light_theme.png'),
      );
    });
  });

  group('InputDialog Golden Tests', () {
    testWidgets('renders basic input dialog', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          theme: ThemeData.dark(),
          home: Scaffold(
            backgroundColor: Colors.black,
            body: InputDialog(
              title: 'Enter Name',
              labelText: 'Name',
              hintText: 'Enter your name',
            ),
          ),
        ),
      );

      await expectLater(
        find.byType(InputDialog),
        matchesGoldenFile('goldens/input_dialog_basic.png'),
      );
    });

    testWidgets('renders input dialog with message', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          theme: ThemeData.dark(),
          home: Scaffold(
            backgroundColor: Colors.black,
            body: InputDialog(
              title: 'Create Layer',
              message: 'Enter a name for the new layer',
              labelText: 'Layer Name',
              hintText: 'e.g., Gaming',
            ),
          ),
        ),
      );

      await expectLater(
        find.byType(InputDialog),
        matchesGoldenFile('goldens/input_dialog_with_message.png'),
      );
    });

    testWidgets('renders input dialog with icon', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          theme: ThemeData.dark(),
          home: Scaffold(
            backgroundColor: Colors.black,
            body: InputDialog(
              title: 'Rename',
              labelText: 'New Name',
              icon: Icons.edit,
              initialValue: 'Old Name',
            ),
          ),
        ),
      );

      await expectLater(
        find.byType(InputDialog),
        matchesGoldenFile('goldens/input_dialog_with_icon.png'),
      );
    });

    testWidgets('renders multiline input dialog', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          theme: ThemeData.dark(),
          home: Scaffold(
            backgroundColor: Colors.black,
            body: InputDialog(
              title: 'Add Description',
              labelText: 'Description',
              hintText: 'Enter a description',
              maxLines: 3,
            ),
          ),
        ),
      );

      await expectLater(
        find.byType(InputDialog),
        matchesGoldenFile('goldens/input_dialog_multiline.png'),
      );
    });

    testWidgets('renders input dialog in light theme', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          theme: ThemeData.light(),
          home: Scaffold(
            backgroundColor: Colors.white,
            body: InputDialog(
              title: 'Enter Name',
              labelText: 'Name',
              hintText: 'Enter your name',
            ),
          ),
        ),
      );

      await expectLater(
        find.byType(InputDialog),
        matchesGoldenFile('goldens/input_dialog_light_theme.png'),
      );
    });
  });
}
