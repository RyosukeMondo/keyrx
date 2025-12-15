import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:keyrx_ui/widgets/common/dialogs/confirmation_dialog.dart';
import 'package:keyrx_ui/widgets/common/dialogs/input_dialog.dart';

import '../../helpers/golden_test_harness.dart';
import '../../helpers/widget_test_harness.dart';

void main() {
  final darkHarness = GoldenTestHarness();
  final lightHarness = GoldenTestHarness(
    widgetHarness: WidgetTestHarness(
      theme: ThemeData.light(useMaterial3: true),
    ),
  );

  Widget dialogShell(Widget child, {Color? backgroundColor}) => Scaffold(
    backgroundColor: backgroundColor,
    body: Center(child: child),
  );

  group('ConfirmationDialog Goldens', () {
    testGoldens('renders basic confirmation dialog', (tester) async {
      await darkHarness.pump(
        tester,
        dialogShell(
          const ConfirmationDialog(
            title: 'Confirm Action',
            message: 'Are you sure you want to proceed?',
          ),
          backgroundColor: Colors.black,
        ),
      );

      await darkHarness.expectGolden(tester, 'confirmation_dialog_basic');
    });

    testGoldens('renders destructive confirmation dialog', (tester) async {
      await darkHarness.pump(
        tester,
        dialogShell(
          const ConfirmationDialog(
            title: 'Delete Item',
            message: 'This action cannot be undone.',
            confirmLabel: 'Delete',
            isDestructive: true,
            icon: Icons.warning,
          ),
          backgroundColor: Colors.black,
        ),
      );

      await darkHarness.expectGolden(tester, 'confirmation_dialog_destructive');
    });

    testGoldens('renders confirmation dialog with icon', (tester) async {
      await darkHarness.pump(
        tester,
        dialogShell(
          const ConfirmationDialog(
            title: 'Save Changes',
            message: 'Do you want to save your changes?',
            confirmLabel: 'Save',
            cancelLabel: 'Discard',
            icon: Icons.save,
          ),
          backgroundColor: Colors.black,
        ),
      );

      await darkHarness.expectGolden(tester, 'confirmation_dialog_with_icon');
    });

    testGoldens('renders confirmation dialog in light theme', (tester) async {
      await lightHarness.pump(
        tester,
        dialogShell(
          const ConfirmationDialog(
            title: 'Confirm Action',
            message: 'Are you sure you want to proceed?',
          ),
          backgroundColor: Colors.white,
        ),
      );

      await lightHarness.expectGolden(
        tester,
        'confirmation_dialog_light_theme',
      );
    });
  });

  group('InputDialog Goldens', () {
    testGoldens('renders basic input dialog', (tester) async {
      await darkHarness.pump(
        tester,
        dialogShell(
          const InputDialog(
            title: 'Enter Name',
            labelText: 'Name',
            hintText: 'Enter your name',
          ),
          backgroundColor: Colors.black,
        ),
      );

      await darkHarness.expectGolden(tester, 'input_dialog_basic');
    });

    testGoldens('renders input dialog with message', (tester) async {
      await darkHarness.pump(
        tester,
        dialogShell(
          const InputDialog(
            title: 'Create Layer',
            message: 'Enter a name for the new layer',
            labelText: 'Layer Name',
            hintText: 'e.g., Gaming',
          ),
          backgroundColor: Colors.black,
        ),
      );

      await darkHarness.expectGolden(tester, 'input_dialog_with_message');
    });

    testGoldens('renders input dialog with icon', (tester) async {
      await darkHarness.pump(
        tester,
        dialogShell(
          const InputDialog(
            title: 'Rename',
            labelText: 'New Name',
            icon: Icons.edit,
            initialValue: 'Old Name',
          ),
          backgroundColor: Colors.black,
        ),
      );

      await darkHarness.expectGolden(tester, 'input_dialog_with_icon');
    });

    testGoldens('renders multiline input dialog', (tester) async {
      await darkHarness.pump(
        tester,
        dialogShell(
          const InputDialog(
            title: 'Add Description',
            labelText: 'Description',
            hintText: 'Enter a description',
            maxLines: 3,
          ),
          backgroundColor: Colors.black,
        ),
      );

      await darkHarness.expectGolden(tester, 'input_dialog_multiline');
    });

    testGoldens('renders input dialog in light theme', (tester) async {
      await lightHarness.pump(
        tester,
        dialogShell(
          const InputDialog(
            title: 'Enter Name',
            labelText: 'Name',
            hintText: 'Enter your name',
          ),
          backgroundColor: Colors.white,
        ),
      );

      await lightHarness.expectGolden(tester, 'input_dialog_light_theme');
    });
  });
}
