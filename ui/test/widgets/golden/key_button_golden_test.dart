import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/widgets/editor/key_button.dart';

void main() {
  group('KeyButton Golden Tests', () {
    testWidgets('renders standard key correctly', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          theme: ThemeData.dark(),
          home: Scaffold(
            backgroundColor: Colors.black,
            body: Center(
              child: KeyButton(
                label: 'A',
                onTap: () {},
              ),
            ),
          ),
        ),
      );

      await expectLater(
        find.byType(KeyButton),
        matchesGoldenFile('goldens/key_button_standard.png'),
      );
    });

    testWidgets('renders selected state correctly', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          theme: ThemeData.dark(),
          home: Scaffold(
            backgroundColor: Colors.black,
            body: Center(
              child: KeyButton(
                label: 'A',
                isSelected: true,
                onTap: () {},
              ),
            ),
          ),
        ),
      );

      await expectLater(
        find.byType(KeyButton),
        matchesGoldenFile('goldens/key_button_selected.png'),
      );
    });

    testWidgets('renders modifier key (wide) correctly', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          theme: ThemeData.dark(),
          home: Scaffold(
            backgroundColor: Colors.black,
            body: Center(
              child: KeyButton(
                label: 'Backspace',
                onTap: () {},
              ),
            ),
          ),
        ),
      );

      await expectLater(
        find.byType(KeyButton),
        matchesGoldenFile('goldens/key_button_modifier.png'),
      );
    });

    testWidgets('renders spacebar correctly', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          theme: ThemeData.dark(),
          home: Scaffold(
            backgroundColor: Colors.black,
            body: Center(
              child: KeyButton(
                label: 'Space',
                onTap: () {},
              ),
            ),
          ),
        ),
      );

      await expectLater(
        find.byType(KeyButton),
        matchesGoldenFile('goldens/key_button_spacebar.png'),
      );
    });

    testWidgets('renders custom size correctly', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          theme: ThemeData.dark(),
          home: Scaffold(
            backgroundColor: Colors.black,
            body: Center(
              child: KeyButton(
                label: 'X',
                width: 100,
                height: 60,
                onTap: () {},
              ),
            ),
          ),
        ),
      );

      await expectLater(
        find.byType(KeyButton),
        matchesGoldenFile('goldens/key_button_custom_size.png'),
      );
    });

    testWidgets('renders light theme correctly', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          theme: ThemeData.light(),
          home: Scaffold(
            backgroundColor: Colors.white,
            body: Center(
              child: KeyButton(
                label: 'B',
                onTap: () {},
              ),
            ),
          ),
        ),
      );

      await expectLater(
        find.byType(KeyButton),
        matchesGoldenFile('goldens/key_button_light_theme.png'),
      );
    });
  });
}
