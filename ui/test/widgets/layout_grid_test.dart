import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/models/layout_type.dart';
import 'package:keyrx_ui/models/profile.dart';
import 'package:keyrx_ui/widgets/layout_grid.dart';

void main() {
  group('LayoutGrid Widget Tests', () {
    // Helper function to wrap widget in MaterialApp for testing
    Widget makeTestableWidget(Widget child) {
      return MaterialApp(home: Scaffold(body: child));
    }

    // Sample profile for testing
    Profile createTestProfile({
      LayoutType layoutType = LayoutType.matrix,
      Map<String, KeyAction>? mappings,
    }) {
      return Profile(
        id: 'test-profile-1',
        name: 'Test Profile',
        layoutType: layoutType,
        mappings: mappings ?? {},
        createdAt: DateTime.now().toIso8601String(),
        updatedAt: DateTime.now().toIso8601String(),
      );
    }

    group('Matrix Layout', () {
      testWidgets('renders correct number of keys for 2x3 matrix', (
        WidgetTester tester,
      ) async {
        final layoutInfo = LayoutInfo(
          rows: 2,
          cols: 3,
          type: LayoutType.matrix,
        );

        await tester.pumpWidget(
          makeTestableWidget(LayoutGrid(layoutInfo: layoutInfo)),
        );

        // Should render 6 keys (2 rows × 3 cols)
        expect(find.byType(InkWell), findsNWidgets(6));
      });

      testWidgets('renders correct number of keys for 3x5 matrix', (
        WidgetTester tester,
      ) async {
        final layoutInfo = LayoutInfo(
          rows: 3,
          cols: 5,
          type: LayoutType.matrix,
        );

        await tester.pumpWidget(
          makeTestableWidget(LayoutGrid(layoutInfo: layoutInfo)),
        );

        // Should render 15 keys (3 rows × 5 cols)
        expect(find.byType(InkWell), findsNWidgets(15));
      });

      testWidgets('displays position labels correctly', (
        WidgetTester tester,
      ) async {
        final layoutInfo = LayoutInfo(
          rows: 2,
          cols: 2,
          type: LayoutType.matrix,
        );

        await tester.pumpWidget(
          makeTestableWidget(LayoutGrid(layoutInfo: layoutInfo)),
        );

        // Check for position labels
        expect(find.text('0,0'), findsOneWidget);
        expect(find.text('0,1'), findsOneWidget);
        expect(find.text('1,0'), findsOneWidget);
        expect(find.text('1,1'), findsOneWidget);
      });

      testWidgets('calls onKeyTap with correct coordinates', (
        WidgetTester tester,
      ) async {
        int? tappedRow;
        int? tappedCol;

        final layoutInfo = LayoutInfo(
          rows: 2,
          cols: 2,
          type: LayoutType.matrix,
        );

        await tester.pumpWidget(
          makeTestableWidget(
            SizedBox(
              width: 200,
              height: 200,
              child: LayoutGrid(
                layoutInfo: layoutInfo,
                keySize: 40.0,
                keySpacing: 4.0,
                onKeyTap: (row, col) {
                  tappedRow = row;
                  tappedCol = col;
                },
              ),
            ),
          ),
        );

        // Tap the first key (0,0)
        await tester.tap(find.byType(InkWell).first);
        await tester.pumpAndSettle();

        expect(tappedRow, equals(0));
        expect(tappedCol, equals(0));

        // Tap the last key (1,1)
        await tester.tap(find.byType(InkWell).last);
        await tester.pumpAndSettle();

        expect(tappedRow, equals(1));
        expect(tappedCol, equals(1));
      });

      testWidgets('displays mapped keys with action labels', (
        WidgetTester tester,
      ) async {
        final profile = createTestProfile(
          layoutType: LayoutType.matrix,
          mappings: {
            '0,0': const KeyAction.key(key: 'A'),
            '0,1': const KeyAction.chord(keys: ['Ctrl', 'C']),
            '1,0': const KeyAction.block(),
          },
        );

        final layoutInfo = LayoutInfo(
          rows: 2,
          cols: 2,
          type: LayoutType.matrix,
        );

        await tester.pumpWidget(
          makeTestableWidget(
            LayoutGrid(layoutInfo: layoutInfo, profile: profile),
          ),
        );

        // Check for mapped key labels
        expect(find.text('A'), findsOneWidget);
        expect(find.text('Ctrl+C'), findsOneWidget);
        expect(find.text('BLOCK'), findsOneWidget);
      });

      testWidgets('highlights selected position', (WidgetTester tester) async {
        final layoutInfo = LayoutInfo(
          rows: 2,
          cols: 2,
          type: LayoutType.matrix,
        );

        final selectedPos = PhysicalPosition(row: 0, col: 1);

        await tester.pumpWidget(
          makeTestableWidget(
            LayoutGrid(layoutInfo: layoutInfo, selectedPosition: selectedPos),
          ),
        );

        await tester.pumpAndSettle();

        // Verify widget builds without error
        // Detailed color verification would require golden tests
        expect(find.byType(LayoutGrid), findsOneWidget);
      });
    });

    group('Standard Layout', () {
      testWidgets('renders placeholder for standard layout', (
        WidgetTester tester,
      ) async {
        final layoutInfo = LayoutInfo(
          rows: 6,
          cols: 20,
          type: LayoutType.standard,
        );

        await tester.pumpWidget(
          makeTestableWidget(LayoutGrid(layoutInfo: layoutInfo)),
        );

        // Should show placeholder text
        expect(find.text('Standard keyboard layout'), findsOneWidget);
        expect(find.text('Visual representation coming soon'), findsOneWidget);
        expect(find.byIcon(Icons.keyboard), findsOneWidget);
      });

      testWidgets('shows mapping count for standard layout with profile', (
        WidgetTester tester,
      ) async {
        final profile = createTestProfile(
          layoutType: LayoutType.standard,
          mappings: {
            '0,0': const KeyAction.key(key: 'A'),
            '0,1': const KeyAction.key(key: 'B'),
            '0,2': const KeyAction.key(key: 'C'),
          },
        );

        final layoutInfo = LayoutInfo(
          rows: 6,
          cols: 20,
          type: LayoutType.standard,
        );

        await tester.pumpWidget(
          makeTestableWidget(
            LayoutGrid(layoutInfo: layoutInfo, profile: profile),
          ),
        );

        // Should show mapping count
        expect(find.text('Mapped keys: 3'), findsOneWidget);
      });
    });

    group('Split Layout', () {
      testWidgets('renders two halves for split layout', (
        WidgetTester tester,
      ) async {
        final layoutInfo = LayoutInfo(
          rows: 2,
          cols: 4, // 2 cols per half
          type: LayoutType.split,
        );

        await tester.pumpWidget(
          makeTestableWidget(LayoutGrid(layoutInfo: layoutInfo)),
        );

        // Should render 8 keys total (2 halves × 2 rows × 2 cols)
        expect(find.byType(InkWell), findsNWidgets(8));
      });

      testWidgets('displays correct positions in split layout', (
        WidgetTester tester,
      ) async {
        final layoutInfo = LayoutInfo(rows: 2, cols: 4, type: LayoutType.split);

        await tester.pumpWidget(
          makeTestableWidget(LayoutGrid(layoutInfo: layoutInfo)),
        );

        // Left half
        expect(find.text('0,0'), findsOneWidget);
        expect(find.text('0,1'), findsOneWidget);
        expect(find.text('1,0'), findsOneWidget);
        expect(find.text('1,1'), findsOneWidget);

        // Right half
        expect(find.text('0,2'), findsOneWidget);
        expect(find.text('0,3'), findsOneWidget);
        expect(find.text('1,2'), findsOneWidget);
        expect(find.text('1,3'), findsOneWidget);
      });
    });

    group('KeyAction Label Generation', () {
      testWidgets('displays correct labels for different action types', (
        WidgetTester tester,
      ) async {
        final profile = createTestProfile(
          mappings: {
            '0,0': const KeyAction.key(key: 'Space'),
            '0,1': const KeyAction.chord(keys: ['Ctrl', 'Alt', 'Del']),
            '0,2': const KeyAction.script(script: 'my_script.sh'),
            '1,0': const KeyAction.block(),
            '1,1': const KeyAction.pass(),
          },
        );

        final layoutInfo = LayoutInfo(
          rows: 2,
          cols: 3,
          type: LayoutType.matrix,
        );

        await tester.pumpWidget(
          makeTestableWidget(
            LayoutGrid(layoutInfo: layoutInfo, profile: profile),
          ),
        );

        // Verify action labels
        expect(find.text('Space'), findsOneWidget);
        expect(find.text('Ctrl+Alt+Del'), findsOneWidget);
        expect(find.text('Script'), findsOneWidget);
        expect(find.text('BLOCK'), findsOneWidget);
        expect(find.text('PASS'), findsOneWidget);
      });
    });

    group('Custom Key Size and Spacing', () {
      testWidgets('accepts custom key size', (WidgetTester tester) async {
        final layoutInfo = LayoutInfo(
          rows: 2,
          cols: 2,
          type: LayoutType.matrix,
        );

        await tester.pumpWidget(
          makeTestableWidget(
            LayoutGrid(layoutInfo: layoutInfo, keySize: 64.0, keySpacing: 8.0),
          ),
        );

        // Should build without errors
        expect(find.byType(LayoutGrid), findsOneWidget);
      });
    });

    group('Edge Cases', () {
      testWidgets('handles empty profile (no mappings)', (
        WidgetTester tester,
      ) async {
        final profile = createTestProfile(mappings: {});

        final layoutInfo = LayoutInfo(
          rows: 2,
          cols: 2,
          type: LayoutType.matrix,
        );

        await tester.pumpWidget(
          makeTestableWidget(
            LayoutGrid(layoutInfo: layoutInfo, profile: profile),
          ),
        );

        // Should render keys with only position labels
        expect(find.text('0,0'), findsOneWidget);
        expect(find.text('1,1'), findsOneWidget);
      });

      testWidgets('handles null profile', (WidgetTester tester) async {
        final layoutInfo = LayoutInfo(
          rows: 2,
          cols: 2,
          type: LayoutType.matrix,
        );

        await tester.pumpWidget(
          makeTestableWidget(LayoutGrid(layoutInfo: layoutInfo, profile: null)),
        );

        // Should render without errors
        expect(find.byType(LayoutGrid), findsOneWidget);
      });

      testWidgets('handles single key layout', (WidgetTester tester) async {
        final layoutInfo = LayoutInfo(
          rows: 1,
          cols: 1,
          type: LayoutType.matrix,
        );

        await tester.pumpWidget(
          makeTestableWidget(LayoutGrid(layoutInfo: layoutInfo)),
        );

        expect(find.byType(InkWell), findsOneWidget);
        expect(find.text('0,0'), findsOneWidget);
      });

      testWidgets('handles large matrix layout', (WidgetTester tester) async {
        final layoutInfo = LayoutInfo(
          rows: 5,
          cols: 5,
          type: LayoutType.matrix,
        );

        await tester.pumpWidget(
          makeTestableWidget(
            SingleChildScrollView(
              child: LayoutGrid(
                layoutInfo: layoutInfo,
                keySize: 40.0,
                keySpacing: 2.0,
              ),
            ),
          ),
        );

        // Should render 25 keys (smaller size to avoid viewport issues)
        expect(find.byType(InkWell), findsNWidgets(25));
      });

      testWidgets('onKeyTap is optional', (WidgetTester tester) async {
        final layoutInfo = LayoutInfo(
          rows: 2,
          cols: 2,
          type: LayoutType.matrix,
        );

        await tester.pumpWidget(
          makeTestableWidget(
            LayoutGrid(layoutInfo: layoutInfo, onKeyTap: null),
          ),
        );

        // Should render without errors and taps should do nothing
        await tester.tap(find.byType(InkWell).first);
        await tester.pumpAndSettle();

        expect(find.byType(LayoutGrid), findsOneWidget);
      });
    });
  });
}
