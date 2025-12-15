/// Tests for InteractiveEdgeCanvas covering click detection, selection, hover, context menu, and keyboard shortcuts.
library;

import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/models/flow/edge_style.dart';
import 'package:keyrx_ui/models/flow/flow_graph.dart';
import 'package:keyrx_ui/models/hardware_profile.dart';
import 'package:keyrx_ui/models/runtime_config.dart';
import 'package:keyrx_ui/services/config_result.dart';
import 'package:keyrx_ui/state/keyflow_providers.dart';
import 'package:keyrx_ui/state/selection_state.dart';
import 'package:keyrx_ui/widgets/flow/edges_painter.dart';
import 'package:keyrx_ui/widgets/flow/interactive_edge.dart';

void main() {
  group('InteractiveEdgeCanvas', () {
    late List<FlowEdge> testEdges;
    late List<PaintableEdge> testPaintableEdges;
    late EdgeStyle testStyle;
    late _FakeKeyFlowActions fakeActions;

    setUp(() {
      testEdges = [
        FlowEdge(
          slotId: 'slot-1',
          from: '1234:5678:serial1',
          to: 'profile-1',
          priority: 0,
          active: true,
          status: EdgeStatus.valid,
        ),
        FlowEdge(
          slotId: 'slot-2',
          from: '1234:5678:serial1',
          to: 'profile-2',
          priority: 1,
          active: false,
          status: EdgeStatus.invalid,
        ),
        FlowEdge(
          slotId: 'slot-3',
          from: '9999:8888:serial2',
          to: 'profile-3',
          priority: 0,
          active: true,
          status: EdgeStatus.warning,
        ),
      ];

      testPaintableEdges = [
        const PaintableEdge(
          start: Offset(100, 100),
          end: Offset(300, 200),
          active: true,
          status: EdgeStatus.valid,
          priority: 0,
        ),
        const PaintableEdge(
          start: Offset(100, 150),
          end: Offset(300, 250),
          active: false,
          status: EdgeStatus.invalid,
          priority: 1,
        ),
        const PaintableEdge(
          start: Offset(400, 100),
          end: Offset(600, 200),
          active: true,
          status: EdgeStatus.warning,
          priority: 0,
        ),
      ];

      testStyle = EdgeStyle(palette: defaultEdgePalette);
    });

    Widget buildTestWidget({
      List<FlowEdge>? edges,
      List<PaintableEdge>? paintableEdges,
      EdgeStyle? style,
    }) {
      return ProviderScope(
        overrides: [
          selectionNotifierProvider.overrideWith(() => SelectionNotifier()),
          keyFlowActionsProvider.overrideWith((ref) {
            fakeActions = _FakeKeyFlowActions(ref);
            return fakeActions;
          }),
        ],
        child: MaterialApp(
          home: Scaffold(
            body: SizedBox(
              width: 800,
              height: 600,
              child: InteractiveEdgeCanvas(
                edges: edges ?? testEdges,
                paintableEdges: paintableEdges ?? testPaintableEdges,
                style: style ?? testStyle,
              ),
            ),
          ),
        ),
      );
    }

    testWidgets('renders edges via CustomPaint', (tester) async {
      await tester.pumpWidget(buildTestWidget());

      expect(find.byType(CustomPaint), findsOneWidget);
    });

    testWidgets('selects edge on tap', (tester) async {
      await tester.pumpWidget(buildTestWidget());

      final container = ProviderScope.containerOf(
        tester.element(find.byType(InteractiveEdgeCanvas)),
      );

      // Before tap
      expect(container.read(selectionNotifierProvider), isA<NoSelection>());

      // Tap near first edge (midpoint around 200, 150)
      await tester.tapAt(const Offset(200, 150));
      await tester.pump();

      // After tap
      final selection = container.read(selectionNotifierProvider);
      expect(selection, isA<EdgeSelection>());
      final edgeSelection = selection as EdgeSelection;
      expect(edgeSelection.slotId, 'slot-1');
      expect(edgeSelection.from, '1234:5678:serial1');
      expect(edgeSelection.to, 'profile-1');
    });

    testWidgets('clears selection when tapping empty space', (tester) async {
      await tester.pumpWidget(buildTestWidget());

      final container = ProviderScope.containerOf(
        tester.element(find.byType(InteractiveEdgeCanvas)),
      );

      // First select an edge
      await tester.tapAt(const Offset(200, 150));
      await tester.pump();
      expect(container.read(selectionNotifierProvider), isA<EdgeSelection>());

      // Tap empty space
      await tester.tapAt(const Offset(10, 10));
      await tester.pump();

      // Selection cleared
      expect(container.read(selectionNotifierProvider), isA<NoSelection>());
    });

    testWidgets('shows context menu on right-click', (tester) async {
      await tester.pumpWidget(buildTestWidget());

      // Right-click near first edge
      await tester.tapAt(const Offset(200, 150), buttons: kSecondaryButton);
      await tester.pumpAndSettle();

      // Context menu should appear
      expect(find.text('Deactivate slot'), findsOneWidget);
      expect(find.text('Delete slot'), findsOneWidget);
    });

    testWidgets('context menu shows "Activate slot" for inactive edge', (
      tester,
    ) async {
      await tester.pumpWidget(buildTestWidget());

      // Right-click near second edge (inactive)
      await tester.tapAt(const Offset(200, 200), buttons: kSecondaryButton);
      await tester.pumpAndSettle();

      // Context menu should show activate option
      expect(find.text('Activate slot'), findsOneWidget);
      expect(find.text('Delete slot'), findsOneWidget);
    });

    testWidgets('context menu delete option shows confirmation dialog', (
      tester,
    ) async {
      await tester.pumpWidget(buildTestWidget());

      // Right-click and select delete
      await tester.tapAt(const Offset(200, 150), buttons: kSecondaryButton);
      await tester.pumpAndSettle();

      await tester.tap(find.text('Delete slot'));
      await tester.pumpAndSettle();

      // Confirmation dialog appears
      expect(find.text('Delete Slot'), findsOneWidget);
      expect(
        find.text('Are you sure you want to delete this slot?'),
        findsOneWidget,
      );
      expect(find.text('Cancel'), findsOneWidget);
      expect(
        find.text('Delete'),
        findsNWidgets(2),
      ); // One in context menu, one in dialog
    });

    testWidgets('confirming delete calls removeSlot', (tester) async {
      await tester.pumpWidget(buildTestWidget());

      // Right-click and select delete
      await tester.tapAt(const Offset(200, 150), buttons: kSecondaryButton);
      await tester.pumpAndSettle();

      await tester.tap(find.text('Delete slot'));
      await tester.pumpAndSettle();

      // Confirm deletion
      await tester.tap(find.text('Delete').last);
      await tester.pumpAndSettle();

      // Verify removeSlot was called
      expect(fakeActions.removeSlotCalls.length, 1);
      expect(fakeActions.removeSlotCalls[0].$1.vendorId, 1234);
      expect(fakeActions.removeSlotCalls[0].$1.productId, 5678);
      expect(fakeActions.removeSlotCalls[0].$1.serial, 'serial1');
      expect(fakeActions.removeSlotCalls[0].$2, 'slot-1');
    });

    testWidgets('canceling delete does not call removeSlot', (tester) async {
      await tester.pumpWidget(buildTestWidget());

      // Right-click and select delete
      await tester.tapAt(const Offset(200, 150), buttons: kSecondaryButton);
      await tester.pumpAndSettle();

      await tester.tap(find.text('Delete slot'));
      await tester.pumpAndSettle();

      // Cancel deletion
      await tester.tap(find.text('Cancel'));
      await tester.pumpAndSettle();

      // Verify removeSlot was NOT called
      expect(fakeActions.removeSlotCalls, isEmpty);
    });

    testWidgets('toggle active calls updateSlot with toggled active state', (
      tester,
    ) async {
      // Create edges with keymap edge for finding keymap ID
      final edgesWithKeymap = [
        ...testEdges,
        FlowEdge(
          slotId: 'slot-1',
          from: 'profile-1',
          to: 'keymap-1',
          priority: 0,
          active: true,
          status: EdgeStatus.valid,
        ),
      ];

      await tester.pumpWidget(buildTestWidget(edges: edgesWithKeymap));

      // Right-click and select toggle
      await tester.tapAt(const Offset(200, 150), buttons: kSecondaryButton);
      await tester.pumpAndSettle();

      await tester.tap(find.text('Deactivate slot'));
      await tester.pumpAndSettle();

      // Verify updateSlot was called with toggled active state
      expect(fakeActions.updateSlotCalls.length, 1);
      expect(fakeActions.updateSlotCalls[0].$2.id, 'slot-1');
      expect(
        fakeActions.updateSlotCalls[0].$2.active,
        false,
      ); // toggled from true
    });

    testWidgets('Delete key deletes selected edge', (tester) async {
      await tester.pumpWidget(buildTestWidget());

      final container = ProviderScope.containerOf(
        tester.element(find.byType(InteractiveEdgeCanvas)),
      );

      // Select an edge first
      await tester.tapAt(const Offset(200, 150));
      await tester.pump();
      expect(container.read(selectionNotifierProvider), isA<EdgeSelection>());

      // Press Delete key
      await tester.sendKeyEvent(LogicalKeyboardKey.delete);
      await tester.pumpAndSettle();

      // Confirmation dialog should appear
      expect(find.text('Delete Slot'), findsOneWidget);

      // Confirm deletion
      await tester.tap(find.text('Delete').last);
      await tester.pumpAndSettle();

      // Verify removeSlot was called
      expect(fakeActions.removeSlotCalls.length, 1);
    });

    testWidgets('Delete key does nothing when no edge selected', (
      tester,
    ) async {
      await tester.pumpWidget(buildTestWidget());

      final container = ProviderScope.containerOf(
        tester.element(find.byType(InteractiveEdgeCanvas)),
      );

      // No selection
      expect(container.read(selectionNotifierProvider), isA<NoSelection>());

      // Press Delete key
      await tester.sendKeyEvent(LogicalKeyboardKey.delete);
      await tester.pump();

      // No dialog should appear
      expect(find.text('Delete Slot'), findsNothing);
      expect(fakeActions.removeSlotCalls, isEmpty);
    });

    testWidgets('hover updates hover position', (tester) async {
      await tester.pumpWidget(buildTestWidget());

      // Create mouse hover gesture
      final gesture = await tester.createGesture(kind: PointerDeviceKind.mouse);
      await gesture.addPointer(location: const Offset(200, 150));
      await tester.pump();

      // Hover position should be updated (tested indirectly via painter)
      expect(find.byType(CustomPaint), findsOneWidget);

      await gesture.removePointer();
    });

    testWidgets('shows success snackbar on successful delete', (tester) async {
      // This will be set when the provider is created in buildTestWidget

      await tester.pumpWidget(buildTestWidget());

      // Delete an edge
      await tester.tapAt(const Offset(200, 150), buttons: kSecondaryButton);
      await tester.pumpAndSettle();

      await tester.tap(find.text('Delete slot'));
      await tester.pumpAndSettle();

      await tester.tap(find.text('Delete').last);
      await tester.pumpAndSettle();

      // Success snackbar should appear
      expect(find.text('Slot deleted successfully'), findsOneWidget);
    });

    testWidgets('shows error snackbar on failed delete', (tester) async {
      _FakeKeyFlowActions? actions;

      final widget = ProviderScope(
        overrides: [
          selectionNotifierProvider.overrideWith(() => SelectionNotifier()),
          keyFlowActionsProvider.overrideWith((ref) {
            actions = _FakeKeyFlowActions(ref);
            actions!.removeSlotResult = ConfigOperationResult.error(
              'FFI error: connection lost',
            );
            return actions!;
          }),
        ],
        child: MaterialApp(
          home: Scaffold(
            body: SizedBox(
              width: 800,
              height: 600,
              child: InteractiveEdgeCanvas(
                edges: testEdges,
                paintableEdges: testPaintableEdges,
                style: testStyle,
              ),
            ),
          ),
        ),
      );

      await tester.pumpWidget(widget);

      // Delete an edge
      await tester.tapAt(const Offset(200, 150), buttons: kSecondaryButton);
      await tester.pumpAndSettle();

      await tester.tap(find.text('Delete slot'));
      await tester.pumpAndSettle();

      await tester.tap(find.text('Delete').last);
      await tester.pumpAndSettle();

      // Error snackbar should appear
      expect(find.textContaining('Failed to delete slot'), findsOneWidget);
      expect(find.textContaining('FFI error: connection lost'), findsOneWidget);
    });

    testWidgets('clears selection after successful delete', (tester) async {
      await tester.pumpWidget(buildTestWidget());

      final container = ProviderScope.containerOf(
        tester.element(find.byType(InteractiveEdgeCanvas)),
      );

      // Select and delete
      await tester.tapAt(const Offset(200, 150), buttons: kSecondaryButton);
      await tester.pumpAndSettle();

      await tester.tap(find.text('Delete slot'));
      await tester.pumpAndSettle();

      await tester.tap(find.text('Delete').last);
      await tester.pumpAndSettle();

      // Selection should be cleared
      expect(container.read(selectionNotifierProvider), isA<NoSelection>());
    });

    testWidgets('highlights selected edge', (tester) async {
      await tester.pumpWidget(buildTestWidget());

      // Select an edge
      await tester.tapAt(const Offset(200, 150));
      await tester.pump();

      // The CustomPaint should have selection state
      final customPaint = tester.widget<CustomPaint>(find.byType(CustomPaint));
      final painter = customPaint.painter as dynamic;
      expect(painter.selection, isA<EdgeSelection>());
    });

    testWidgets('can select different edges', (tester) async {
      await tester.pumpWidget(buildTestWidget());

      final container = ProviderScope.containerOf(
        tester.element(find.byType(InteractiveEdgeCanvas)),
      );

      // Select first edge
      await tester.tapAt(const Offset(200, 150));
      await tester.pump();
      EdgeSelection selection =
          container.read(selectionNotifierProvider) as EdgeSelection;
      expect(selection.slotId, 'slot-1');

      // Select second edge
      await tester.tapAt(const Offset(200, 200));
      await tester.pump();
      selection = container.read(selectionNotifierProvider) as EdgeSelection;
      expect(selection.slotId, 'slot-2');

      // Select third edge
      await tester.tapAt(const Offset(500, 150));
      await tester.pump();
      selection = container.read(selectionNotifierProvider) as EdgeSelection;
      expect(selection.slotId, 'slot-3');
    });
  });
}

/// Fake KeyFlowActions for testing.
class _FakeKeyFlowActions implements KeyFlowActions {
  _FakeKeyFlowActions(this._ref);

  final Ref _ref;

  final List<(DeviceInstanceId, String)> removeSlotCalls = [];
  final List<(DeviceInstanceId, ProfileSlot)> updateSlotCalls = [];

  ConfigOperationResult<RuntimeConfig> removeSlotResult =
      ConfigOperationResult.success(RuntimeConfig(devices: []));
  ConfigOperationResult<RuntimeConfig> updateSlotResult =
      ConfigOperationResult.success(RuntimeConfig(devices: []));

  @override
  Future<ConfigOperationResult<RuntimeConfig>> removeSlot(
    DeviceInstanceId device,
    String slotId,
  ) async {
    removeSlotCalls.add((device, slotId));
    return removeSlotResult;
  }

  @override
  Future<ConfigOperationResult<RuntimeConfig>> updateSlot(
    DeviceInstanceId device,
    ProfileSlot slot,
  ) async {
    updateSlotCalls.add((device, slot));
    return updateSlotResult;
  }

  @override
  Future<ConfigOperationResult<RuntimeConfig>> addSlot(
    DeviceInstanceId device,
    ProfileSlot slot, {
    int? priorityOverride,
  }) async {
    return ConfigOperationResult.success(RuntimeConfig(devices: []));
  }

  @override
  Future<ConfigOperationResult<RuntimeConfig>> refreshRuntimeConfig() async {
    return ConfigOperationResult.success(RuntimeConfig(devices: []));
  }
}
