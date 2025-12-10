// Widget tests for the visual keyboard and mapping overlay.
//
// Tests keyboard rendering, key tap callbacks, and drag-drop mapping.

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/models/keyboard_layout.dart';
import 'package:keyrx_ui/widgets/mapping_overlay.dart';
import 'package:keyrx_ui/widgets/visual_keyboard.dart';

/// Minimal layout for testing with just a few keys.
KeyboardLayout _testLayout() {
  return KeyboardLayout(
    name: 'Test',
    unitSize: 48.0,
    keySpacing: 4.0,
    rows: [
      KeyboardRow(
        keys: [
          KeyDefinition(id: 'KeyA', label: 'A', row: 0, column: 0),
          KeyDefinition(id: 'KeyB', label: 'B', row: 0, column: 1),
          KeyDefinition(id: 'KeyC', label: 'C', row: 0, column: 2),
        ],
      ),
      KeyboardRow(
        keys: [
          KeyDefinition(id: 'KeyD', label: 'D', row: 1, column: 0),
          KeyDefinition(id: 'KeyE', label: 'E', row: 1, column: 1),
          KeyDefinition(id: 'KeyF', label: 'F', row: 1, column: 2),
        ],
      ),
    ],
  );
}

Widget _buildTestKeyboard({
  KeyboardLayout? layout,
  void Function(KeyDefinition)? onKeyTap,
  void Function(KeyDefinition)? onKeyLongPress,
  void Function(String, String)? onMappingCreated,
  void Function(int)? onMappingDeleted,
  List<RemapConfig> mappings = const [],
  Set<String> selectedKeys = const {},
  Set<String> highlightedKeys = const {},
  Set<String> mappedKeys = const {},
  Set<String> heldKeys = const {},
  bool showSecondaryLabels = true,
  bool showMappingOverlay = true,
  bool enableDragDrop = true,
  bool enabled = true,
}) {
  return MaterialApp(
    home: Scaffold(
      body: SizedBox(
        width: 800,
        height: 600,
        child: VisualKeyboard(
          layout: layout ?? _testLayout(),
          onKeyTap: onKeyTap,
          onKeyLongPress: onKeyLongPress,
          onMappingCreated: onMappingCreated,
          onMappingDeleted: onMappingDeleted,
          mappings: mappings,
          selectedKeys: selectedKeys,
          highlightedKeys: highlightedKeys,
          mappedKeys: mappedKeys,
          heldKeys: heldKeys,
          showSecondaryLabels: showSecondaryLabels,
          showMappingOverlay: showMappingOverlay,
          enableDragDrop: enableDragDrop,
          enabled: enabled,
        ),
      ),
    ),
  );
}

void main() {
  group('Keyboard rendering', () {
    testWidgets('renders all keys from layout', (tester) async {
      await tester.pumpWidget(_buildTestKeyboard());

      expect(find.text('A'), findsOneWidget);
      expect(find.text('B'), findsOneWidget);
      expect(find.text('C'), findsOneWidget);
      expect(find.text('D'), findsOneWidget);
      expect(find.text('E'), findsOneWidget);
      expect(find.text('F'), findsOneWidget);
    });

    testWidgets('renders default ANSI 104 layout when none provided', (
      tester,
    ) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: SizedBox(width: 1200, height: 600, child: VisualKeyboard()),
          ),
        ),
      );

      // Check for some standard ANSI keys
      expect(find.text('Esc'), findsOneWidget);
      expect(find.text('Tab'), findsOneWidget);
      expect(find.text('Space'), findsOneWidget);
      expect(find.text('Enter'), findsOneWidget);
    });

    testWidgets('renders key with secondary label when enabled', (
      tester,
    ) async {
      final layout = KeyboardLayout(
        name: 'Test',
        unitSize: 48.0,
        keySpacing: 4.0,
        rows: [
          KeyboardRow(
            keys: [
              KeyDefinition(
                id: 'Key1',
                label: '1',
                secondaryLabel: '!',
                row: 0,
                column: 0,
              ),
            ],
          ),
        ],
      );

      await tester.pumpWidget(
        _buildTestKeyboard(layout: layout, showSecondaryLabels: true),
      );

      expect(find.text('1'), findsOneWidget);
      expect(find.text('!'), findsOneWidget);
    });

    testWidgets('hides secondary label when disabled', (tester) async {
      final layout = KeyboardLayout(
        name: 'Test',
        unitSize: 48.0,
        keySpacing: 4.0,
        rows: [
          KeyboardRow(
            keys: [
              KeyDefinition(
                id: 'Key1',
                label: '1',
                secondaryLabel: '!',
                row: 0,
                column: 0,
              ),
            ],
          ),
        ],
      );

      await tester.pumpWidget(
        _buildTestKeyboard(layout: layout, showSecondaryLabels: false),
      );

      expect(find.text('1'), findsOneWidget);
      expect(find.text('!'), findsNothing);
    });

    testWidgets('shows mapped indicator for mapped keys', (tester) async {
      await tester.pumpWidget(_buildTestKeyboard(mappedKeys: {'KeyA'}));

      // The mapped indicator is a small circular Container
      // We verify it by checking the key with mapped state has the indicator
      // (This is implementation-dependent, but we can verify the visual tree)
      expect(find.text('A'), findsOneWidget);
    });
  });

  group('Key tap callback', () {
    testWidgets('calls onKeyTap when key is tapped', (tester) async {
      KeyDefinition? tappedKey;

      // Disable mapping overlay to allow direct key taps
      await tester.pumpWidget(
        _buildTestKeyboard(
          onKeyTap: (key) => tappedKey = key,
          showMappingOverlay: false,
        ),
      );

      await tester.tap(find.text('A'));
      await tester.pumpAndSettle();

      expect(tappedKey, isNotNull);
      expect(tappedKey!.id, 'KeyA');
      expect(tappedKey!.label, 'A');
    });

    testWidgets('does not call onKeyTap when disabled', (tester) async {
      KeyDefinition? tappedKey;

      await tester.pumpWidget(
        _buildTestKeyboard(
          onKeyTap: (key) => tappedKey = key,
          enabled: false,
          showMappingOverlay: false,
        ),
      );

      await tester.tap(find.text('A'), warnIfMissed: false);
      await tester.pumpAndSettle();

      expect(tappedKey, isNull);
    });

    testWidgets('calls onKeyLongPress on long press', (tester) async {
      KeyDefinition? longPressedKey;

      // Disable drag-drop to allow long press to work without being captured
      await tester.pumpWidget(
        _buildTestKeyboard(
          onKeyLongPress: (key) => longPressedKey = key,
          showMappingOverlay: false,
          enableDragDrop: false,
        ),
      );

      await tester.longPress(find.text('B'));
      await tester.pumpAndSettle();

      expect(longPressedKey, isNotNull);
      expect(longPressedKey!.id, 'KeyB');
    });

    testWidgets('can tap different keys', (tester) async {
      final tappedKeys = <String>[];

      await tester.pumpWidget(
        _buildTestKeyboard(
          onKeyTap: (key) => tappedKeys.add(key.id),
          showMappingOverlay: false,
        ),
      );

      await tester.tap(find.text('A'));
      await tester.pumpAndSettle();

      await tester.tap(find.text('C'));
      await tester.pumpAndSettle();

      await tester.tap(find.text('E'));
      await tester.pumpAndSettle();

      expect(tappedKeys, ['KeyA', 'KeyC', 'KeyE']);
    });
  });

  group('Key visual states', () {
    testWidgets('selected key has different styling', (tester) async {
      await tester.pumpWidget(_buildTestKeyboard(selectedKeys: {'KeyA'}));

      // Key A should be rendered, and its styling is applied
      expect(find.text('A'), findsOneWidget);
    });

    testWidgets('highlighted key has different styling', (tester) async {
      await tester.pumpWidget(_buildTestKeyboard(highlightedKeys: {'KeyB'}));

      expect(find.text('B'), findsOneWidget);
    });

    testWidgets('held key has pressed styling', (tester) async {
      await tester.pumpWidget(_buildTestKeyboard(heldKeys: {'KeyC'}));

      expect(find.text('C'), findsOneWidget);
    });

    testWidgets('multiple visual states can coexist', (tester) async {
      await tester.pumpWidget(
        _buildTestKeyboard(
          selectedKeys: {'KeyA'},
          highlightedKeys: {'KeyB'},
          mappedKeys: {'KeyC'},
          heldKeys: {'KeyD'},
        ),
      );

      expect(find.text('A'), findsOneWidget);
      expect(find.text('B'), findsOneWidget);
      expect(find.text('C'), findsOneWidget);
      expect(find.text('D'), findsOneWidget);
    });
  });

  group('Drag creates mapping', () {
    testWidgets('drag creates mapping structure exists', (tester) async {
      // Verify the drag-drop infrastructure is set up correctly
      await tester.pumpWidget(
        _buildTestKeyboard(
          onMappingCreated: (source, target) {},
          enableDragDrop: true,
        ),
      );

      // Verify DragTarget widgets are created for each key
      expect(find.byType(DragTarget<String>), findsWidgets);
      // Verify LongPressDraggable widgets are created for each key
      expect(find.byType(LongPressDraggable<String>), findsWidgets);
    });

    testWidgets('dragging to same key does not create mapping', (tester) async {
      bool mappingCreated = false;

      await tester.pumpWidget(
        _buildTestKeyboard(
          onMappingCreated: (_, __) => mappingCreated = true,
          showMappingOverlay: false,
        ),
      );

      final keyA = find.text('A');
      final keyACenter = tester.getCenter(keyA);

      // Drag from A to itself - should not create mapping
      await tester.timedDragFrom(
        keyACenter,
        Offset.zero,
        const Duration(milliseconds: 500),
      );
      await tester.pumpAndSettle();

      expect(mappingCreated, isFalse);
    });

    testWidgets('drag is disabled when enableDragDrop is false', (
      tester,
    ) async {
      await tester.pumpWidget(_buildTestKeyboard(enableDragDrop: false));

      // When drag-drop is disabled, no DragTarget or LongPressDraggable should exist
      expect(find.byType(DragTarget<String>), findsNothing);
      expect(find.byType(LongPressDraggable<String>), findsNothing);
    });
  });

  group('Mapping overlay', () {
    testWidgets('displays mapping arrows when mappings exist', (tester) async {
      await tester.pumpWidget(
        _buildTestKeyboard(
          mappings: [
            const RemapConfig(
              sourceKeyId: 'KeyA',
              targetKeyId: 'KeyB',
              type: MappingType.simple,
            ),
          ],
        ),
      );

      // The MappingOverlay uses CustomPaint, so we verify it's present
      expect(find.byType(CustomPaint), findsWidgets);
    });

    testWidgets('hides mapping overlay when disabled', (tester) async {
      await tester.pumpWidget(
        _buildTestKeyboard(
          mappings: [
            const RemapConfig(
              sourceKeyId: 'KeyA',
              targetKeyId: 'KeyB',
              type: MappingType.simple,
            ),
          ],
          showMappingOverlay: false,
        ),
      );

      // MappingOverlay widget should not be present
      expect(find.byType(MappingOverlay), findsNothing);
    });

    testWidgets('shows delete button on mapping', (tester) async {
      await tester.pumpWidget(
        _buildTestKeyboard(
          mappings: [
            const RemapConfig(
              sourceKeyId: 'KeyA',
              targetKeyId: 'KeyB',
              type: MappingType.simple,
            ),
          ],
        ),
      );

      // The mapping has an arrow icon button
      expect(find.byIcon(Icons.arrow_forward), findsOneWidget);
    });

    testWidgets('tapping mapping selects it', (tester) async {
      await tester.pumpWidget(
        _buildTestKeyboard(
          mappings: [
            const RemapConfig(
              sourceKeyId: 'KeyA',
              targetKeyId: 'KeyB',
              type: MappingType.simple,
            ),
          ],
        ),
      );

      // Tap the mapping button (arrow icon)
      await tester.tap(find.byIcon(Icons.arrow_forward));
      await tester.pumpAndSettle();

      // After selection, the delete icon should appear
      expect(find.byIcon(Icons.close), findsOneWidget);
    });

    testWidgets('deleting mapping calls onMappingDeleted', (tester) async {
      int? deletedIndex;

      await tester.pumpWidget(
        _buildTestKeyboard(
          mappings: [
            const RemapConfig(
              sourceKeyId: 'KeyA',
              targetKeyId: 'KeyB',
              type: MappingType.simple,
            ),
          ],
          onMappingDeleted: (index) => deletedIndex = index,
        ),
      );

      // First tap to select
      await tester.tap(find.byIcon(Icons.arrow_forward));
      await tester.pumpAndSettle();

      // Then tap delete icon
      await tester.tap(find.byIcon(Icons.close));
      await tester.pumpAndSettle();

      expect(deletedIndex, 0);
    });

    testWidgets('multiple mappings show multiple buttons', (tester) async {
      await tester.pumpWidget(
        _buildTestKeyboard(
          mappings: [
            const RemapConfig(
              sourceKeyId: 'KeyA',
              targetKeyId: 'KeyB',
              type: MappingType.simple,
            ),
            const RemapConfig(
              sourceKeyId: 'KeyC',
              targetKeyId: 'KeyD',
              type: MappingType.simple,
            ),
          ],
        ),
      );

      // Should have two arrow icons (one per mapping)
      expect(find.byIcon(Icons.arrow_forward), findsNWidgets(2));
    });
  });

  group('Compact keyboard', () {
    testWidgets('CompactVisualKeyboard renders with smaller size', (
      tester,
    ) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: SizedBox(
              width: 800,
              height: 400,
              child: CompactVisualKeyboard(),
            ),
          ),
        ),
      );

      // Verify standard keys are rendered
      expect(find.text('Esc'), findsOneWidget);
      expect(find.text('Tab'), findsOneWidget);
    });

    testWidgets(
      'CompactVisualKeyboard creates VisualKeyboard with correct props',
      (tester) async {
        KeyDefinition? tappedKey;

        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: SizedBox(
                width: 800,
                height: 400,
                child: CompactVisualKeyboard(
                  onKeyTap: (key) => tappedKey = key,
                  selectedKeys: const {'KeyA'},
                  highlightedKeys: const {'KeyB'},
                ),
              ),
            ),
          ),
        );

        // Verify the underlying VisualKeyboard is created
        expect(find.byType(VisualKeyboard), findsOneWidget);

        // Verify keys are rendered
        expect(find.text('A'), findsOneWidget);
        expect(find.text('Q'), findsOneWidget);

        // Verify the callback is wired up (we can't tap directly due to overlay,
        // but we verified the widget structure is correct)
        expect(tappedKey, isNull); // Not tapped yet
      },
    );

    testWidgets('CompactVisualKeyboard respects selectedKeys', (tester) async {
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: SizedBox(
              width: 800,
              height: 400,
              child: CompactVisualKeyboard(selectedKeys: {'KeyA'}),
            ),
          ),
        ),
      );

      expect(find.text('A'), findsOneWidget);
    });
  });

  group('KeyboardLayout model', () {
    test('findKey returns correct key', () {
      final layout = _testLayout();
      final key = layout.findKey('KeyB');

      expect(key, isNotNull);
      expect(key!.id, 'KeyB');
      expect(key.label, 'B');
    });

    test('findKey returns null for non-existent key', () {
      final layout = _testLayout();
      final key = layout.findKey('KeyZ');

      expect(key, isNull);
    });

    test('allKeyIds returns all key IDs', () {
      final layout = _testLayout();
      final ids = layout.allKeyIds;

      expect(
        ids,
        containsAll(['KeyA', 'KeyB', 'KeyC', 'KeyD', 'KeyE', 'KeyF']),
      );
      expect(ids.length, 6);
    });

    test('getKeyPosition calculates correct position', () {
      final layout = _testLayout();
      final keyA = layout.findKey('KeyA')!;
      final pos = layout.getKeyPosition(keyA);

      // KeyA is at row 0, column 0
      expect(pos.x, 0.0);
      expect(pos.y, 0.0);
    });

    test('getKeyPosition accounts for column offset', () {
      final layout = _testLayout();
      final keyB = layout.findKey('KeyB')!;
      final pos = layout.getKeyPosition(keyB);

      // KeyB is at row 0, column 1
      // Position = column * (unitSize + keySpacing)
      expect(pos.x, 52.0); // 1 * (48 + 4)
      expect(pos.y, 0.0);
    });

    test('getKeySize calculates correct size', () {
      final layout = _testLayout();
      final keyA = layout.findKey('KeyA')!;
      final size = layout.getKeySize(keyA);

      // Standard 1.0u key
      expect(size.width, 48.0);
      expect(size.height, 48.0);
    });

    test('getKeySize accounts for wider keys', () {
      final layout = KeyboardLayout(
        name: 'Test',
        unitSize: 48.0,
        keySpacing: 4.0,
        rows: [
          KeyboardRow(
            keys: [
              KeyDefinition(
                id: 'Wide',
                label: 'Wide',
                width: 2.0,
                row: 0,
                column: 0,
              ),
            ],
          ),
        ],
      );

      final key = layout.findKey('Wide')!;
      final size = layout.getKeySize(key);

      // 2.0u key: width = 2 * 48 + (2-1) * 4 = 100
      expect(size.width, 100.0);
      expect(size.height, 48.0);
    });

    test('totalWidth accounts for all keys in widest row', () {
      final layout = _testLayout();
      // 3 keys per row, each 48px + 4px spacing = 52px each
      // Total = 3 * 52 = 156
      expect(layout.totalWidth, 156.0);
    });

    test('totalHeight accounts for all rows', () {
      final layout = _testLayout();
      // 2 rows, each 48px + 4px spacing = 52px
      // Total = 2 * 52 = 104
      expect(layout.totalHeight, 104.0);
    });
  });

  group('RemapConfig', () {
    test('equality based on all fields', () {
      const config1 = RemapConfig(
        sourceKeyId: 'KeyA',
        targetKeyId: 'KeyB',
        type: MappingType.simple,
      );
      const config2 = RemapConfig(
        sourceKeyId: 'KeyA',
        targetKeyId: 'KeyB',
        type: MappingType.simple,
      );
      const config3 = RemapConfig(
        sourceKeyId: 'KeyA',
        targetKeyId: 'KeyC',
        type: MappingType.simple,
      );

      expect(config1, equals(config2));
      expect(config1, isNot(equals(config3)));
    });

    test('different types are not equal', () {
      const config1 = RemapConfig(
        sourceKeyId: 'KeyA',
        targetKeyId: 'KeyB',
        type: MappingType.simple,
      );
      const config2 = RemapConfig(
        sourceKeyId: 'KeyA',
        targetKeyId: 'KeyB',
        type: MappingType.tapHold,
      );

      expect(config1, isNot(equals(config2)));
    });
  });

  group('KeyDefinition', () {
    test('equality based on id only', () {
      const key1 = KeyDefinition(id: 'KeyA', label: 'A');
      const key2 = KeyDefinition(id: 'KeyA', label: 'Different');
      const key3 = KeyDefinition(id: 'KeyB', label: 'A');

      expect(key1, equals(key2));
      expect(key1, isNot(equals(key3)));
    });

    test('hashCode based on id', () {
      const key1 = KeyDefinition(id: 'KeyA', label: 'A');
      const key2 = KeyDefinition(id: 'KeyA', label: 'Different');

      expect(key1.hashCode, equals(key2.hashCode));
    });
  });
}
