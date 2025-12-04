import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/models/key_mapping.dart';
import 'package:keyrx_ui/widgets/editor/key_grid.dart';
import 'package:keyrx_ui/widgets/editor/key_button.dart';
import 'package:keyrx_ui/widgets/editor/key_legend.dart';
import 'package:keyrx_ui/widgets/editor/binding_panel.dart';
import 'package:keyrx_ui/widgets/editor/layer_panel.dart';
import 'package:keyrx_ui/widgets/common/styled_text_field.dart';

/// Integration tests for complete editor workflows combining multiple widgets.
void main() {
  group('Editor Workflow Integration', () {
    testWidgets('Complete key mapping workflow',
        (WidgetTester tester) async {
      String? selectedKey;
      KeyActionType selectedAction = KeyActionType.remap;
      final outputController = TextEditingController();
      final layerController = TextEditingController();
      final tapOutputController = TextEditingController();
      final holdOutputController = TextEditingController();
      final appliedMappings = <String, String>{};

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: StatefulBuilder(
              builder: (context, setState) {
                return Row(
                  children: [
                    // Key grid for selection
                    Expanded(
                      flex: 2,
                      child: Column(
                        children: [
                          Expanded(
                            child: KeyGrid(
                              selectedKey: selectedKey,
                              onKeySelected: (key) {
                                setState(() {
                                  selectedKey = key;
                                });
                              },
                            ),
                          ),
                          const KeyLegend(
                            items: [
                              LegendItem(label: 'Unassigned', color: Colors.grey),
                              LegendItem(label: 'Remapped', color: Colors.blue),
                              LegendItem(label: 'Blocked', color: Colors.red),
                            ],
                          ),
                        ],
                      ),
                    ),
                    // Binding panel for configuration
                    Expanded(
                      child: BindingPanel(
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
                          if (selectedKey != null) {
                            appliedMappings[selectedKey!] =
                                outputController.text;
                          }
                        },
                      ),
                    ),
                  ],
                );
              },
            ),
          ),
        ),
      );

      // Initially no key selected
      expect(find.text('Select a key to configure'), findsOneWidget);

      // Select key 'A' from grid
      await tester.tap(find.text('A'));
      await tester.pumpAndSettle();

      // Binding panel should show configuration
      expect(find.text('Configuring: A'), findsOneWidget);

      // Enter remap target
      await tester.enterText(
        find.widgetWithText(StyledTextField, 'Remap to key'),
        'Escape',
      );
      await tester.pumpAndSettle();

      // Apply mapping
      await tester.tap(find.widgetWithText(FilledButton, 'Apply'));
      await tester.pumpAndSettle();

      expect(appliedMappings['A'], equals('Escape'));

      // Select another key
      await tester.tap(find.text('B'));
      await tester.pumpAndSettle();

      expect(find.text('Configuring: B'), findsOneWidget);

      // Change action to block
      await tester.tap(find.byType(DropdownButton<KeyActionType>));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Block').last);
      await tester.pumpAndSettle();

      // Apply
      await tester.tap(find.widgetWithText(FilledButton, 'Apply'));
      await tester.pumpAndSettle();

      expect(appliedMappings.length, equals(2));

      outputController.dispose();
      layerController.dispose();
      tapOutputController.dispose();
      holdOutputController.dispose();
    });

    testWidgets('KeyGrid with LayerPanel integration',
        (WidgetTester tester) async {
      String? selectedKey;
      var layers = [
        const LayerInfo(name: 'base', active: true, priority: 0),
        const LayerInfo(name: 'navigation', active: false, priority: 1),
      ];

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: StatefulBuilder(
              builder: (context, setState) {
                return Row(
                  children: [
                    Expanded(
                      flex: 2,
                      child: KeyGrid(
                        selectedKey: selectedKey,
                        onKeySelected: (key) {
                          setState(() {
                            selectedKey = key;
                          });
                        },
                      ),
                    ),
                    SizedBox(
                      width: 200,
                      child: LayerPanel(
                        layers: layers,
                        onToggleLayer: (name, active) {
                          setState(() {
                            layers = layers
                                .map((l) => l.name == name
                                    ? LayerInfo(
                                        name: l.name,
                                        active: active,
                                        priority: l.priority)
                                    : l)
                                .toList();
                          });
                        },
                      ),
                    ),
                  ],
                );
              },
            ),
          ),
        ),
      );

      // Both widgets visible
      expect(find.byType(KeyGrid), findsOneWidget);
      expect(find.byType(LayerPanel), findsOneWidget);

      // Select a key
      await tester.tap(find.text('A'));
      await tester.pumpAndSettle();

      // Verify selection
      final keyButtons = find.byType(KeyButton);
      final aButton = tester
          .widgetList<KeyButton>(keyButtons)
          .firstWhere((b) => b.label == 'A');
      expect(aButton.isSelected, isTrue);

      // Toggle layer
      final switches = find.byType(Switch);
      await tester.tap(switches.at(1));
      await tester.pumpAndSettle();

      // Verify layer toggled
      expect(layers[1].active, isTrue);
    });

    testWidgets('Multiple keys can be configured sequentially',
        (WidgetTester tester) async {
      String? selectedKey;
      final outputController = TextEditingController();
      final layerController = TextEditingController();
      final tapOutputController = TextEditingController();
      final holdOutputController = TextEditingController();
      final mappings = <String, String>{};

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: StatefulBuilder(
              builder: (context, setState) {
                return Row(
                  children: [
                    Expanded(
                      child: KeyGrid(
                        selectedKey: selectedKey,
                        onKeySelected: (key) {
                          setState(() {
                            selectedKey = key;
                            outputController.clear();
                          });
                        },
                      ),
                    ),
                    Expanded(
                      child: BindingPanel(
                        selectedKey: selectedKey,
                        selectedAction: KeyActionType.remap,
                        outputController: outputController,
                        layerController: layerController,
                        tapOutputController: tapOutputController,
                        holdOutputController: holdOutputController,
                        onActionChanged: (_) {},
                        onApply: () {
                          if (selectedKey != null) {
                            mappings[selectedKey!] = outputController.text;
                          }
                        },
                      ),
                    ),
                  ],
                );
              },
            ),
          ),
        ),
      );

      // Configure multiple keys
      final keysToMap = ['A', 'S', 'D', 'F'];
      final targets = ['Left', 'Down', 'Up', 'Right'];

      for (var i = 0; i < keysToMap.length; i++) {
        // Select key
        await tester.tap(find.text(keysToMap[i]));
        await tester.pumpAndSettle();

        // Enter mapping
        await tester.enterText(
          find.widgetWithText(StyledTextField, 'Remap to key'),
          targets[i],
        );
        await tester.pumpAndSettle();

        // Apply
        await tester.tap(find.widgetWithText(FilledButton, 'Apply'));
        await tester.pumpAndSettle();
      }

      // Verify all mappings applied
      expect(mappings.length, equals(4));
      expect(mappings['A'], equals('Left'));
      expect(mappings['S'], equals('Down'));
      expect(mappings['D'], equals('Up'));
      expect(mappings['F'], equals('Right'));

      outputController.dispose();
      layerController.dispose();
      tapOutputController.dispose();
      holdOutputController.dispose();
    });

    testWidgets('KeyGrid selection persists during panel interactions',
        (WidgetTester tester) async {
      String? selectedKey;
      final outputController = TextEditingController();
      final layerController = TextEditingController();
      final tapOutputController = TextEditingController();
      final holdOutputController = TextEditingController();

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: StatefulBuilder(
              builder: (context, setState) {
                return Row(
                  children: [
                    Expanded(
                      child: KeyGrid(
                        selectedKey: selectedKey,
                        onKeySelected: (key) {
                          setState(() {
                            selectedKey = key;
                          });
                        },
                      ),
                    ),
                    Expanded(
                      child: BindingPanel(
                        selectedKey: selectedKey,
                        selectedAction: KeyActionType.remap,
                        outputController: outputController,
                        layerController: layerController,
                        tapOutputController: tapOutputController,
                        holdOutputController: holdOutputController,
                        onActionChanged: (_) {},
                        onApply: () {},
                      ),
                    ),
                  ],
                );
              },
            ),
          ),
        ),
      );

      // Select a key
      await tester.tap(find.text('A'));
      await tester.pumpAndSettle();

      // Interact with panel
      await tester.enterText(
        find.widgetWithText(StyledTextField, 'Remap to key'),
        'Escape',
      );
      await tester.pumpAndSettle();

      // Key selection should persist
      final keyButtons = find.byType(KeyButton);
      final aButton = tester
          .widgetList<KeyButton>(keyButtons)
          .firstWhere((b) => b.label == 'A');
      expect(aButton.isSelected, isTrue);

      // Panel should still show correct key
      expect(find.text('Configuring: A'), findsOneWidget);

      outputController.dispose();
      layerController.dispose();
      tapOutputController.dispose();
      holdOutputController.dispose();
    });

    testWidgets('Complete editor UI composition',
        (WidgetTester tester) async {
      String? selectedKey;
      final outputController = TextEditingController();
      final layerController = TextEditingController();
      final tapOutputController = TextEditingController();
      final holdOutputController = TextEditingController();
      var layers = [
        const LayerInfo(name: 'base', active: true, priority: 0),
      ];

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: StatefulBuilder(
              builder: (context, setState) {
                return Column(
                  children: [
                    Expanded(
                      child: Row(
                        children: [
                          Expanded(
                            flex: 2,
                            child: Column(
                              children: [
                                Expanded(
                                  child: KeyGrid(
                                    selectedKey: selectedKey,
                                    onKeySelected: (key) {
                                      setState(() {
                                        selectedKey = key;
                                      });
                                    },
                                  ),
                                ),
                                const KeyLegend(
                            items: [
                              LegendItem(label: 'Unassigned', color: Colors.grey),
                              LegendItem(label: 'Remapped', color: Colors.blue),
                              LegendItem(label: 'Blocked', color: Colors.red),
                            ],
                          ),
                              ],
                            ),
                          ),
                          SizedBox(
                            width: 200,
                            child: LayerPanel(
                              layers: layers,
                              onToggleLayer: (name, active) {
                                setState(() {
                                  layers = layers
                                      .map((l) => l.name == name
                                          ? LayerInfo(
                                              name: l.name,
                                              active: active,
                                              priority: l.priority)
                                          : l)
                                      .toList();
                                });
                              },
                              onAddLayer: () {
                                setState(() {
                                  layers = [
                                    ...layers,
                                    LayerInfo(
                                      name: 'layer${layers.length}',
                                      active: false,
                                      priority: layers.length,
                                    ),
                                  ];
                                });
                              },
                            ),
                          ),
                        ],
                      ),
                    ),
                    BindingPanel(
                      selectedKey: selectedKey,
                      selectedAction: KeyActionType.remap,
                      outputController: outputController,
                      layerController: layerController,
                      tapOutputController: tapOutputController,
                      holdOutputController: holdOutputController,
                      onActionChanged: (_) {},
                      onApply: () {},
                    ),
                  ],
                );
              },
            ),
          ),
        ),
      );

      // Verify all components are present
      expect(find.byType(KeyGrid), findsOneWidget);
      expect(find.byType(KeyLegend), findsOneWidget);
      expect(find.byType(LayerPanel), findsOneWidget);
      expect(find.byType(BindingPanel), findsOneWidget);

      // Interact with each component
      await tester.tap(find.text('A'));
      await tester.pumpAndSettle();
      expect(find.text('Configuring: A'), findsOneWidget);

      await tester.tap(find.byIcon(Icons.add));
      await tester.pumpAndSettle();
      expect(layers.length, equals(2));

      outputController.dispose();
      layerController.dispose();
      tapOutputController.dispose();
      holdOutputController.dispose();
    });
  });
}
