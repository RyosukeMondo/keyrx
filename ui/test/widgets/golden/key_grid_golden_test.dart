import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/widgets/editor/key_grid.dart';

void main() {
  group('KeyGrid Golden Tests', () {
    testWidgets('renders full keyboard layout', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          theme: ThemeData.dark(),
          home: Scaffold(
            backgroundColor: Colors.black,
            body: Center(
              child: KeyGrid(
                onKeySelected: (_) {},
              ),
            ),
          ),
        ),
      );

      await expectLater(
        find.byType(KeyGrid),
        matchesGoldenFile('goldens/key_grid_full_layout.png'),
      );
    });

    testWidgets('renders with selected key', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          theme: ThemeData.dark(),
          home: Scaffold(
            backgroundColor: Colors.black,
            body: Center(
              child: KeyGrid(
                selectedKey: 'A',
                onKeySelected: (_) {},
              ),
            ),
          ),
        ),
      );

      await expectLater(
        find.byType(KeyGrid),
        matchesGoldenFile('goldens/key_grid_with_selection.png'),
      );
    });

    testWidgets('renders with modifier key selected', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          theme: ThemeData.dark(),
          home: Scaffold(
            backgroundColor: Colors.black,
            body: Center(
              child: KeyGrid(
                selectedKey: 'LShift',
                onKeySelected: (_) {},
              ),
            ),
          ),
        ),
      );

      await expectLater(
        find.byType(KeyGrid),
        matchesGoldenFile('goldens/key_grid_modifier_selected.png'),
      );
    });

    testWidgets('renders in light theme', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          theme: ThemeData.light(),
          home: Scaffold(
            backgroundColor: Colors.white,
            body: Center(
              child: KeyGrid(
                onKeySelected: (_) {},
              ),
            ),
          ),
        ),
      );

      await expectLater(
        find.byType(KeyGrid),
        matchesGoldenFile('goldens/key_grid_light_theme.png'),
      );
    });

    testWidgets('renders with spacebar selected', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          theme: ThemeData.dark(),
          home: Scaffold(
            backgroundColor: Colors.black,
            body: Center(
              child: KeyGrid(
                selectedKey: 'Space',
                onKeySelected: (_) {},
              ),
            ),
          ),
        ),
      );

      await expectLater(
        find.byType(KeyGrid),
        matchesGoldenFile('goldens/key_grid_spacebar_selected.png'),
      );
    });
  });
}
