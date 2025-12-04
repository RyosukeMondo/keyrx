import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/models/key_mapping.dart';
import 'package:keyrx_ui/widgets/editor/binding_panel.dart';
import 'package:keyrx_ui/widgets/common/styled_text_field.dart';

/// Integration tests for BindingPanel with form controls and callbacks.
void main() {
  group('BindingPanel Integration', () {
    late TextEditingController outputController;
    late TextEditingController layerController;
    late TextEditingController tapOutputController;
    late TextEditingController holdOutputController;

    setUp(() {
      outputController = TextEditingController();
      layerController = TextEditingController();
      tapOutputController = TextEditingController();
      holdOutputController = TextEditingController();
    });

    tearDown(() {
      outputController.dispose();
      layerController.dispose();
      tapOutputController.dispose();
      holdOutputController.dispose();
    });

    testWidgets('BindingPanel composes text fields correctly',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: BindingPanel(
              selectedKey: 'A',
              selectedAction: KeyActionType.remap,
              outputController: outputController,
              layerController: layerController,
              tapOutputController: tapOutputController,
              holdOutputController: holdOutputController,
              onActionChanged: (_) {},
              onApply: () {},
            ),
          ),
        ),
      );

      // Verify all text fields are present
      expect(find.byType(StyledTextField), findsNWidgets(4));
    });

    testWidgets('BindingPanel propagates text input to controllers',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: BindingPanel(
              selectedKey: 'A',
              selectedAction: KeyActionType.remap,
              outputController: outputController,
              layerController: layerController,
              tapOutputController: tapOutputController,
              holdOutputController: holdOutputController,
              onActionChanged: (_) {},
              onApply: () {},
            ),
          ),
        ),
      );

      // Enter text in remap field
      await tester.enterText(
        find.widgetWithText(StyledTextField, 'Remap to key'),
        'Escape',
      );
      await tester.pumpAndSettle();

      expect(outputController.text, equals('Escape'));

      // Enter text in layer field
      await tester.enterText(
        find.widgetWithText(StyledTextField, 'Layer (optional)'),
        'navigation',
      );
      await tester.pumpAndSettle();

      expect(layerController.text, equals('navigation'));
    });

    testWidgets('BindingPanel triggers apply callback',
        (WidgetTester tester) async {
      var applyCount = 0;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: BindingPanel(
              selectedKey: 'A',
              selectedAction: KeyActionType.remap,
              outputController: outputController,
              layerController: layerController,
              tapOutputController: tapOutputController,
              holdOutputController: holdOutputController,
              onActionChanged: (_) {},
              onApply: () => applyCount++,
            ),
          ),
        ),
      );

      // Tap apply button
      await tester.tap(find.widgetWithText(FilledButton, 'Apply'));
      await tester.pumpAndSettle();

      expect(applyCount, equals(1));
    });

    testWidgets('BindingPanel handles action type changes',
        (WidgetTester tester) async {
      KeyActionType? changedAction;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: StatefulBuilder(
              builder: (context, setState) {
                return BindingPanel(
                  selectedKey: 'A',
                  selectedAction: changedAction ?? KeyActionType.remap,
                  outputController: outputController,
                  layerController: layerController,
                  tapOutputController: tapOutputController,
                  holdOutputController: holdOutputController,
                  onActionChanged: (action) {
                    setState(() {
                      changedAction = action;
                    });
                  },
                  onApply: () {},
                );
              },
            ),
          ),
        ),
      );

      // Find and tap dropdown
      await tester.tap(find.byType(DropdownButton<KeyActionType>));
      await tester.pumpAndSettle();

      // Select 'Block' action
      await tester.tap(find.text('Block').last);
      await tester.pumpAndSettle();

      expect(changedAction, equals(KeyActionType.block));
    });

    testWidgets('BindingPanel disables output field for non-remap actions',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: BindingPanel(
              selectedKey: 'A',
              selectedAction: KeyActionType.block,
              outputController: outputController,
              layerController: layerController,
              tapOutputController: tapOutputController,
              holdOutputController: holdOutputController,
              onActionChanged: (_) {},
              onApply: () {},
            ),
          ),
        ),
      );

      // Find the output field
      final outputFields = find.byType(StyledTextField);
      final outputField =
          tester.widget<StyledTextField>(outputFields.first);

      // Should be disabled for block action
      expect(outputField.enabled, isFalse);
    });

    testWidgets('BindingPanel shows message when no key selected',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: BindingPanel(
              selectedKey: null,
              selectedAction: KeyActionType.remap,
              outputController: outputController,
              layerController: layerController,
              tapOutputController: tapOutputController,
              holdOutputController: holdOutputController,
              onActionChanged: (_) {},
              onApply: () {},
            ),
          ),
        ),
      );

      expect(find.text('Select a key to configure'), findsOneWidget);
      expect(find.byType(StyledTextField), findsNothing);
    });

    testWidgets('BindingPanel workflow: select key, configure, apply',
        (WidgetTester tester) async {
      String? selectedKey;
      KeyActionType selectedAction = KeyActionType.remap;
      var applyCount = 0;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: StatefulBuilder(
              builder: (context, setState) {
                return BindingPanel(
                  selectedKey: selectedKey,
                  selectedAction: selectedAction,
                  outputController: outputController,
                  layerController: layerController,
                  tapOutputController: tapOutputController,
                  holdOutputController: holdOutputController,
                  onActionChanged: (action) {
                    if (action != null) {
                      setState(() {
                        selectedAction = action;
                      });
                    }
                  },
                  onApply: () {
                    applyCount++;
                  },
                );
              },
            ),
          ),
        ),
      );

      // Initially no key selected
      expect(find.text('Select a key to configure'), findsOneWidget);

      // Select a key
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: StatefulBuilder(
              builder: (context, setState) {
                selectedKey = 'A';
                return BindingPanel(
                  selectedKey: selectedKey,
                  selectedAction: selectedAction,
                  outputController: outputController,
                  layerController: layerController,
                  tapOutputController: tapOutputController,
                  holdOutputController: holdOutputController,
                  onActionChanged: (action) {
                    if (action != null) {
                      setState(() {
                        selectedAction = action;
                      });
                    }
                  },
                  onApply: () {
                    applyCount++;
                  },
                );
              },
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();

      // Verify configuration UI is shown
      expect(find.text('Configuring: A'), findsOneWidget);

      // Enter remap value
      await tester.enterText(
        find.widgetWithText(StyledTextField, 'Remap to key'),
        'Escape',
      );
      await tester.pumpAndSettle();

      // Apply configuration
      await tester.tap(find.widgetWithText(FilledButton, 'Apply'));
      await tester.pumpAndSettle();

      expect(applyCount, equals(1));
      expect(outputController.text, equals('Escape'));
    });
  });
}
