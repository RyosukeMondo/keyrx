/// Tests for AddSlotDialog covering form validation, profile/keymap selection, submission, and cancellation.
library;

import 'package:core_logic/models/hardware_profile.dart';
import 'package:core_logic/models/keymap.dart';
import 'package:core_logic/models/runtime_config.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/dialogs/add_slot_dialog.dart';
import 'package:keyrx_ui/services/config_result.dart';
import 'package:keyrx_ui/state/keyflow_providers.dart';
import 'package:keyrx_ui/state/slot_actions.dart';

void main() {
  group('AddSlotDialog', () {
    late List<HardwareProfile> testProfiles;
    late List<Keymap> testKeymaps;
    late DeviceInstanceId testDevice;
    late _FakeSlotActions fakeSlotActions;

    setUp(() {
      testProfiles = [
        const HardwareProfile(
          id: 'profile-1',
          name: 'Ergo-60',
          virtualLayoutId: 'layout-1',
        ),
        const HardwareProfile(
          id: 'profile-2',
          name: 'TKL Layout',
          virtualLayoutId: 'layout-1',
        ),
        const HardwareProfile(
          id: 'profile-3',
          name: 'Full-size',
          virtualLayoutId: 'layout-1',
        ),
      ];

      testKeymaps = [
        const Keymap(
          id: 'keymap-1',
          name: 'Default',
          virtualLayoutId: 'layout-1',
        ),
        const Keymap(
          id: 'keymap-2',
          name: 'Gaming',
          virtualLayoutId: 'layout-1',
        ),
        const Keymap(
          id: 'keymap-3',
          name: 'Programming',
          virtualLayoutId: 'layout-1',
        ),
      ];

      testDevice = const DeviceInstanceId(
        vendorId: 1234,
        productId: 5678,
        serial: 'serial1',
      );
      fakeSlotActions = _FakeSlotActions();
    });

    Widget buildTestWidget({List<ProfileSlotSelection>? existingSlots}) =>
        ProviderScope(
          overrides: [
            availableHardwareProfilesProvider.overrideWith(
              (ref) => Stream.value(testProfiles).asyncMap((v) => v),
            ),
            availableKeymapsProvider.overrideWith(
              (ref) => Stream.value(testKeymaps).asyncMap((v) => v),
            ),
            slotActionsProvider.overrideWith((ref) => fakeSlotActions),
          ],
          child: MaterialApp(
            home: Scaffold(
              body: Builder(
                builder: (context) => ElevatedButton(
                  onPressed: () {
                    showAddSlotDialog(
                      context,
                      device: testDevice,
                      existingSlots: existingSlots ?? [],
                    );
                  },
                  child: const Text('Open Dialog'),
                ),
              ),
            ),
          ),
        );

    testWidgets('renders dialog with all elements', (tester) async {
      await tester.pumpWidget(buildTestWidget());
      await tester.tap(find.text('Open Dialog'));
      await tester.pumpAndSettle();

      expect(find.text('Add Slot'), findsOneWidget);
      expect(find.text('Hardware Profile'), findsOneWidget);
      expect(find.text('Keymap'), findsOneWidget);
      expect(find.text('Active'), findsOneWidget);
      expect(find.text('Cancel'), findsOneWidget);
      expect(find.text('Add'), findsOneWidget);
    });

    testWidgets('shows hardware profile dropdown with options', (tester) async {
      await tester.pumpWidget(buildTestWidget());
      await tester.tap(find.text('Open Dialog'));
      await tester.pumpAndSettle();

      // Open dropdown
      await tester.tap(find.text('Select a hardware profile'));
      await tester.pumpAndSettle();

      // Verify options
      expect(find.text('Ergo-60'), findsOneWidget);
      expect(find.text('TKL Layout'), findsOneWidget);
      expect(find.text('Full-size'), findsOneWidget);
    });

    testWidgets('shows keymap dropdown with options', (tester) async {
      await tester.pumpWidget(buildTestWidget());
      await tester.tap(find.text('Open Dialog'));
      await tester.pumpAndSettle();

      // Open dropdown
      await tester.tap(find.text('Select a keymap'));
      await tester.pumpAndSettle();

      // Verify options
      expect(find.text('Default'), findsOneWidget);
      expect(find.text('Gaming'), findsOneWidget);
      expect(find.text('Programming'), findsOneWidget);
    });

    testWidgets('validates that profile is selected before submission', (
      tester,
    ) async {
      await tester.pumpWidget(buildTestWidget());
      await tester.tap(find.text('Open Dialog'));
      await tester.pumpAndSettle();

      // Try to submit without selecting profile
      await tester.tap(find.text('Add'));
      await tester.pumpAndSettle();

      // Error message should appear
      expect(find.text('Please select a hardware profile'), findsOneWidget);
      expect(fakeSlotActions.addSlotCalls, isEmpty);
    });

    testWidgets('validates that keymap is selected before submission', (
      tester,
    ) async {
      await tester.pumpWidget(buildTestWidget());
      await tester.tap(find.text('Open Dialog'));
      await tester.pumpAndSettle();

      // Select profile but not keymap
      await tester.tap(find.text('Select a hardware profile'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Ergo-60').last);
      await tester.pumpAndSettle();

      // Try to submit
      await tester.tap(find.text('Add'));
      await tester.pumpAndSettle();

      // Error message should appear
      expect(find.text('Please select a keymap'), findsOneWidget);
      expect(fakeSlotActions.addSlotCalls, isEmpty);
    });

    testWidgets('validates against duplicate profile+keymap combinations', (
      tester,
    ) async {
      final existingSlots = [
        ProfileSlotSelection(
          hardwareProfileId: 'profile-1',
          keymapId: 'keymap-1',
        ),
      ];

      await tester.pumpWidget(buildTestWidget(existingSlots: existingSlots));
      await tester.tap(find.text('Open Dialog'));
      await tester.pumpAndSettle();

      // Select the same combination
      await tester.tap(find.text('Select a hardware profile'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Ergo-60').last);
      await tester.pumpAndSettle();

      await tester.tap(find.text('Select a keymap'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Default').last);
      await tester.pumpAndSettle();

      // Try to submit
      await tester.tap(find.text('Add'));
      await tester.pumpAndSettle();

      // Error message should appear
      expect(
        find.text(
          'This profile + keymap combination already exists for this device',
        ),
        findsOneWidget,
      );
      expect(fakeSlotActions.addSlotCalls, isEmpty);
    });

    testWidgets('allows duplicate check to pass with different combinations', (
      tester,
    ) async {
      final existingSlots = [
        ProfileSlotSelection(
          hardwareProfileId: 'profile-1',
          keymapId: 'keymap-1',
        ),
      ];

      fakeSlotActions.addSlotResult =
          ConfigOperationResult<RuntimeConfig>.success(const RuntimeConfig());

      await tester.pumpWidget(buildTestWidget(existingSlots: existingSlots));
      await tester.tap(find.text('Open Dialog'));
      await tester.pumpAndSettle();

      // Select different combination
      await tester.tap(find.text('Select a hardware profile'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('TKL Layout').last);
      await tester.pumpAndSettle();

      await tester.tap(find.text('Select a keymap'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Gaming').last);
      await tester.pumpAndSettle();

      // Submit
      await tester.tap(find.text('Add'));
      await tester.pumpAndSettle();

      // Should succeed
      expect(fakeSlotActions.addSlotCalls.length, 1);
    });

    testWidgets('submits slot when form is valid', (tester) async {
      fakeSlotActions.addSlotResult =
          ConfigOperationResult<RuntimeConfig>.success(const RuntimeConfig());

      await tester.pumpWidget(buildTestWidget());
      await tester.tap(find.text('Open Dialog'));
      await tester.pumpAndSettle();

      // Select profile
      await tester.tap(find.text('Select a hardware profile'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Ergo-60').last);
      await tester.pumpAndSettle();

      // Select keymap
      await tester.tap(find.text('Select a keymap'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Default').last);
      await tester.pumpAndSettle();

      // Submit
      await tester.tap(find.text('Add'));
      await tester.pumpAndSettle();

      // Verify call
      expect(fakeSlotActions.addSlotCalls.length, 1);
      expect(fakeSlotActions.addSlotCalls[0].device.vendorId, 1234);
      expect(fakeSlotActions.addSlotCalls[0].device.productId, 5678);
      expect(fakeSlotActions.addSlotCalls[0].hardwareProfileId, 'profile-1');
      expect(fakeSlotActions.addSlotCalls[0].keymapId, 'keymap-1');
      expect(fakeSlotActions.addSlotCalls[0].active, false); // default
    });

    testWidgets('submits with active=true when checkbox is checked', (
      tester,
    ) async {
      fakeSlotActions.addSlotResult =
          ConfigOperationResult<RuntimeConfig>.success(const RuntimeConfig());

      await tester.pumpWidget(buildTestWidget());
      await tester.tap(find.text('Open Dialog'));
      await tester.pumpAndSettle();

      // Select profile and keymap
      await tester.tap(find.text('Select a hardware profile'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Ergo-60').last);
      await tester.pumpAndSettle();

      await tester.tap(find.text('Select a keymap'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Default').last);
      await tester.pumpAndSettle();

      // Check active checkbox
      await tester.tap(find.byType(Checkbox));
      await tester.pumpAndSettle();

      // Submit
      await tester.tap(find.text('Add'));
      await tester.pumpAndSettle();

      // Verify active=true
      expect(fakeSlotActions.addSlotCalls[0].active, true);
    });

    testWidgets('closes dialog on successful submission', (tester) async {
      fakeSlotActions.addSlotResult =
          ConfigOperationResult<RuntimeConfig>.success(const RuntimeConfig());

      await tester.pumpWidget(buildTestWidget());
      await tester.tap(find.text('Open Dialog'));
      await tester.pumpAndSettle();

      // Fill form and submit
      await tester.tap(find.text('Select a hardware profile'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Ergo-60').last);
      await tester.pumpAndSettle();

      await tester.tap(find.text('Select a keymap'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Default').last);
      await tester.pumpAndSettle();

      await tester.tap(find.text('Add'));
      await tester.pumpAndSettle();

      // Dialog should be closed
      expect(find.text('Add Slot'), findsNothing);
      expect(find.text('Open Dialog'), findsOneWidget);
    });

    testWidgets('closes dialog on cancel button', (tester) async {
      await tester.pumpWidget(buildTestWidget());
      await tester.tap(find.text('Open Dialog'));
      await tester.pumpAndSettle();

      // Cancel
      await tester.tap(find.text('Cancel'));
      await tester.pumpAndSettle();

      // Dialog should be closed
      expect(find.text('Add Slot'), findsNothing);
      expect(fakeSlotActions.addSlotCalls, isEmpty);
    });

    testWidgets('shows error message on submission failure', (tester) async {
      fakeSlotActions.addSlotResult =
          ConfigOperationResult<RuntimeConfig>.error(
            'FFI error: device disconnected',
          );

      await tester.pumpWidget(buildTestWidget());
      await tester.tap(find.text('Open Dialog'));
      await tester.pumpAndSettle();

      // Fill and submit
      await tester.tap(find.text('Select a hardware profile'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Ergo-60').last);
      await tester.pumpAndSettle();

      await tester.tap(find.text('Select a keymap'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Default').last);
      await tester.pumpAndSettle();

      await tester.tap(find.text('Add'));
      await tester.pumpAndSettle();

      // Error message should appear in dialog
      expect(find.text('FFI error: device disconnected'), findsOneWidget);
      expect(find.byIcon(Icons.error_outline), findsOneWidget);
    });

    testWidgets('disables buttons while saving', (tester) async {
      // Use a completer to control when the async operation finishes
      fakeSlotActions.addSlotDelay = const Duration(seconds: 2);
      fakeSlotActions.addSlotResult =
          ConfigOperationResult<RuntimeConfig>.success(const RuntimeConfig());

      await tester.pumpWidget(buildTestWidget());
      await tester.tap(find.text('Open Dialog'));
      await tester.pumpAndSettle();

      // Fill form
      await tester.tap(find.text('Select a hardware profile'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Ergo-60').last);
      await tester.pumpAndSettle();

      await tester.tap(find.text('Select a keymap'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Default').last);
      await tester.pumpAndSettle();

      // Start submission
      await tester.tap(find.text('Add'));
      await tester.pump();

      // Buttons should be disabled and show loading indicator
      final addButton = tester.widget<FilledButton>(
        find.widgetWithText(FilledButton, 'Add'),
      );
      expect(addButton.onPressed, isNull);

      final cancelButton = tester.widget<TextButton>(
        find.widgetWithText(TextButton, 'Cancel'),
      );
      expect(cancelButton.onPressed, isNull);

      expect(find.byType(CircularProgressIndicator), findsOneWidget);
    });

    testWidgets('clears error message when selection changes', (tester) async {
      await tester.pumpWidget(buildTestWidget());
      await tester.tap(find.text('Open Dialog'));
      await tester.pumpAndSettle();

      // Try to submit without selection to trigger error
      await tester.tap(find.text('Add'));
      await tester.pumpAndSettle();
      expect(find.text('Please select a hardware profile'), findsOneWidget);

      // Change selection
      await tester.tap(find.text('Select a hardware profile'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Ergo-60').last);
      await tester.pumpAndSettle();

      // Error should be cleared
      expect(find.text('Please select a hardware profile'), findsNothing);
    });

    testWidgets('shows message when no profiles available', (tester) async {
      final widget = ProviderScope(
        overrides: [
          availableHardwareProfilesProvider.overrideWith(
            (ref) => Stream.value(<HardwareProfile>[]).asyncMap((v) => v),
          ),
          availableKeymapsProvider.overrideWith(
            (ref) => Stream.value(testKeymaps).asyncMap((v) => v),
          ),
          slotActionsProvider.overrideWith((ref) => fakeSlotActions),
        ],
        child: MaterialApp(
          home: Scaffold(
            body: Builder(
              builder: (context) => ElevatedButton(
                onPressed: () {
                  showAddSlotDialog(
                    context,
                    device: testDevice,
                    existingSlots: [],
                  );
                },
                child: const Text('Open Dialog'),
              ),
            ),
          ),
        ),
      );

      await tester.pumpWidget(widget);
      await tester.tap(find.text('Open Dialog'));
      await tester.pumpAndSettle();

      expect(find.text('No hardware profiles available'), findsOneWidget);
    });

    testWidgets('shows message when no keymaps available', (tester) async {
      final widget = ProviderScope(
        overrides: [
          availableHardwareProfilesProvider.overrideWith(
            (ref) => Stream.value(testProfiles).asyncMap((v) => v),
          ),
          availableKeymapsProvider.overrideWith(
            (ref) => Stream.value(<Keymap>[]).asyncMap((v) => v),
          ),
          slotActionsProvider.overrideWith((ref) => fakeSlotActions),
        ],
        child: MaterialApp(
          home: Scaffold(
            body: Builder(
              builder: (context) => ElevatedButton(
                onPressed: () {
                  showAddSlotDialog(
                    context,
                    device: testDevice,
                    existingSlots: [],
                  );
                },
                child: const Text('Open Dialog'),
              ),
            ),
          ),
        ),
      );

      await tester.pumpWidget(widget);
      await tester.tap(find.text('Open Dialog'));
      await tester.pumpAndSettle();

      expect(find.text('No keymaps available'), findsOneWidget);
    });
  });
}

/// Fake SlotActions for testing.
class _FakeSlotActions implements SlotActions {
  final List<_AddSlotCall> addSlotCalls = [];

  ConfigOperationResult<RuntimeConfig> addSlotResult =
      ConfigOperationResult<RuntimeConfig>.success(const RuntimeConfig());

  Duration addSlotDelay = Duration.zero;

  @override
  Future<ConfigOperationResult<RuntimeConfig>> addSlot({
    required DeviceInstanceId device,
    required String hardwareProfileId,
    required String keymapId,
    int? priorityOverride,
    bool active = false,
  }) async {
    addSlotCalls.add(
      _AddSlotCall(
        device: device,
        hardwareProfileId: hardwareProfileId,
        keymapId: keymapId,
        active: active,
        priorityOverride: priorityOverride,
      ),
    );

    if (addSlotDelay > Duration.zero) {
      await Future.delayed(addSlotDelay);
    }

    return addSlotResult;
  }

  @override
  Future<ConfigOperationResult<RuntimeConfig>> deleteSlot({
    required DeviceInstanceId device,
    required String slotId,
  }) async {
    throw UnimplementedError();
  }

  @override
  Future<List<ConfigOperationResult<RuntimeConfig>>> reorderSlots({
    required DeviceInstanceId device,
    required List<ProfileSlot> slotsInOrder,
  }) async {
    throw UnimplementedError();
  }

  @override
  Future<ConfigOperationResult<RuntimeConfig>> toggleSlotActive({
    required DeviceInstanceId device,
    required ProfileSlot slot,
  }) async {
    throw UnimplementedError();
  }

  @override
  Future<ConfigOperationResult<RuntimeConfig>> updateSlot({
    required DeviceInstanceId device,
    required ProfileSlot slot,
  }) async {
    throw UnimplementedError();
  }
}

class _AddSlotCall {
  _AddSlotCall({
    required this.device,
    required this.hardwareProfileId,
    required this.keymapId,
    required this.active,
    this.priorityOverride,
  });

  final DeviceInstanceId device;
  final String hardwareProfileId;
  final String keymapId;
  final bool active;
  final int? priorityOverride;
}
