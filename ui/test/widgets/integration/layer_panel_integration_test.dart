import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/widgets/editor/layer_panel.dart';

/// Integration tests for LayerPanel with list interactions and callbacks.
void main() {
  group('LayerPanel Integration', () {
    final testLayers = [
      const LayerInfo(name: 'base', active: true, priority: 0),
      const LayerInfo(name: 'navigation', active: false, priority: 1),
      const LayerInfo(name: 'symbols', active: false, priority: 2),
    ];

    testWidgets('LayerPanel renders all layers', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(
              height: 400,
              child: LayerPanel(layers: testLayers),
            ),
          ),
        ),
      );

      // Verify all layers are displayed
      expect(find.text('base'), findsOneWidget);
      expect(find.text('navigation'), findsOneWidget);
      expect(find.text('symbols'), findsOneWidget);
    });

    testWidgets('LayerPanel shows layer priorities',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(
              height: 400,
              child: LayerPanel(layers: testLayers),
            ),
          ),
        ),
      );

      // Verify priorities are shown
      expect(find.text('Priority: 0'), findsOneWidget);
      expect(find.text('Priority: 1'), findsOneWidget);
      expect(find.text('Priority: 2'), findsOneWidget);
    });

    testWidgets('LayerPanel toggle callback fires correctly',
        (WidgetTester tester) async {
      String? toggledLayer;
      bool? toggledState;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(
              height: 400,
              child: LayerPanel(
                layers: testLayers,
                onToggleLayer: (name, active) {
                  toggledLayer = name;
                  toggledState = active;
                },
              ),
            ),
          ),
        ),
      );

      // Find and tap the switch for 'navigation' layer
      final switches = find.byType(Switch);
      // First switch is for 'base', second for 'navigation'
      await tester.tap(switches.at(1));
      await tester.pumpAndSettle();

      expect(toggledLayer, equals('navigation'));
      expect(toggledState, isTrue);
    });

    testWidgets('LayerPanel add layer callback fires',
        (WidgetTester tester) async {
      var addCalled = false;

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(
              height: 400,
              child: LayerPanel(
                layers: testLayers,
                onAddLayer: () => addCalled = true,
              ),
            ),
          ),
        ),
      );

      // Tap the add button
      await tester.tap(find.byIcon(Icons.add));
      await tester.pumpAndSettle();

      expect(addCalled, isTrue);
    });

    testWidgets('LayerPanel handles empty layer list',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(
              height: 400,
              child: LayerPanel(layers: const []),
            ),
          ),
        ),
      );

      // Panel should still render with header
      expect(find.text('Layers'), findsOneWidget);
      expect(find.byIcon(Icons.add), findsOneWidget);
    });

    testWidgets('LayerPanel shows correct icons for active/inactive layers',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(
              height: 400,
              child: LayerPanel(layers: testLayers),
            ),
          ),
        ),
      );

      // Find all list tiles
      final listTiles = find.byType(ListTile);
      expect(listTiles, findsNWidgets(3));

      // Find icons
      final layersIcons = find.byIcon(Icons.layers);
      final layersOutlinedIcons = find.byIcon(Icons.layers_outlined);

      // Active layer (base) should have filled icon
      expect(layersIcons, findsOneWidget);
      // Inactive layers should have outlined icons
      expect(layersOutlinedIcons, findsNWidgets(2));
    });

    testWidgets('LayerPanel workflow: add, toggle, and verify state',
        (WidgetTester tester) async {
      var layers = List<LayerInfo>.from(testLayers);
      var addCount = 0;
      final toggledLayers = <String, bool>{};

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: StatefulBuilder(
              builder: (context, setState) {
                return SizedBox(
                  height: 400,
                  child: LayerPanel(
                    layers: layers,
                    onToggleLayer: (name, active) {
                      toggledLayers[name] = active;
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
                      addCount++;
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
                );
              },
            ),
          ),
        ),
      );

      // Initially 3 layers
      expect(find.byType(ListTile), findsNWidgets(3));

      // Add a new layer
      await tester.tap(find.byIcon(Icons.add));
      await tester.pumpAndSettle();

      expect(addCount, equals(1));
      expect(find.byType(ListTile), findsNWidgets(4));
      expect(find.text('layer3'), findsOneWidget);

      // Toggle navigation layer
      final switches = find.byType(Switch);
      await tester.tap(switches.at(1));
      await tester.pumpAndSettle();

      expect(toggledLayers['navigation'], isTrue);

      // Verify the switch state updated
      final navigationSwitch = tester.widget<Switch>(switches.at(1));
      expect(navigationSwitch.value, isTrue);
    });

    testWidgets('LayerPanel handles null callbacks gracefully',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(
              height: 400,
              child: LayerPanel(
                layers: testLayers,
                onToggleLayer: null,
                onAddLayer: null,
              ),
            ),
          ),
        ),
      );

      // Tapping add button should not throw
      await tester.tap(find.byIcon(Icons.add));
      await tester.pumpAndSettle();

      // Tapping switch should not throw
      final switches = find.byType(Switch);
      await tester.tap(switches.first);
      await tester.pumpAndSettle();

      // No errors expected
    });

    testWidgets('LayerPanel scrolls with many layers',
        (WidgetTester tester) async {
      // Create many layers
      final manyLayers = List.generate(
        20,
        (i) => LayerInfo(
          name: 'layer$i',
          active: i == 0,
          priority: i,
        ),
      );

      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(
              height: 400,
              child: LayerPanel(layers: manyLayers),
            ),
          ),
        ),
      );

      // First layer should be visible
      expect(find.text('layer0'), findsOneWidget);

      // Last layer might not be visible initially
      expect(find.text('layer19'), findsNothing);

      // Scroll down
      await tester.drag(find.byType(ListView), const Offset(0, -500));
      await tester.pumpAndSettle();

      // Now last layer should be visible
      expect(find.text('layer19'), findsOneWidget);
    });
  });
}
