import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/services/keyflow_analytics.dart';
import 'package:keyrx_ui/utils/logger.dart';

void main() {
  group('KeyFlowAnalytics', () {
    late KeyFlowAnalytics analytics;
    late List<String> logOutput;
    late StructuredLogger logger;

    setUp(() {
      logOutput = [];
      logger = getLogger('test_analytics', sink: (line) => logOutput.add(line));
      analytics = KeyFlowAnalytics(logger: logger);
    });

    tearDown(() {
      analytics.dispose();
      LoggingConfigurator.resetForTesting();
    });

    test('initializes with enabled flag', () {
      final analytics1 = KeyFlowAnalytics();
      final analytics2 = KeyFlowAnalytics(enabled: false);

      expect(analytics1, isNotNull);
      expect(analytics2, isNotNull);

      analytics1.dispose();
      analytics2.dispose();
    });

    test('tracks page view event', () async {
      analytics.trackPageView(deviceCount: 2, slotCount: 5, nodeCount: 10);

      // Wait for logging to process
      await Future.delayed(const Duration(milliseconds: 10));

      expect(logOutput, isNotEmpty);
      expect(
        logOutput.any((line) => line.contains('keyflow.event.pageView')),
        isTrue,
      );
      expect(logOutput.any((line) => line.contains('device_count')), isTrue);
    });

    test('tracks slot added event', () async {
      analytics.trackSlotAdded(
        deviceId: 'device1',
        profileId: 'profile1',
        keymapId: 'keymap1',
        priority: 0,
        autoAssigned: true,
      );

      await Future.delayed(const Duration(milliseconds: 10));

      expect(logOutput, isNotEmpty);
      expect(
        logOutput.any((line) => line.contains('keyflow.event.slotAdded')),
        isTrue,
      );
    });

    test('tracks slot deleted event', () async {
      analytics.trackSlotDeleted(
        deviceId: 'device1',
        slotId: 'slot1',
        priority: 0,
      );

      await Future.delayed(const Duration(milliseconds: 10));

      expect(logOutput, isNotEmpty);
      expect(
        logOutput.any((line) => line.contains('keyflow.event.slotDeleted')),
        isTrue,
      );
    });

    test('tracks slot reordered event', () async {
      analytics.trackSlotReordered(
        deviceId: 'device1',
        slotId: 'slot1',
        oldPriority: 0,
        newPriority: 1,
      );

      await Future.delayed(const Duration(milliseconds: 10));

      expect(logOutput, isNotEmpty);
      expect(
        logOutput.any((line) => line.contains('keyflow.event.slotReordered')),
        isTrue,
      );
    });

    test('tracks node dragged event', () async {
      analytics.trackNodeDragged(
        nodeId: 'node1',
        nodeType: 'device',
        deltaX: 10,
        deltaY: 20,
      );

      await Future.delayed(const Duration(milliseconds: 10));

      expect(logOutput, isNotEmpty);
      expect(
        logOutput.any((line) => line.contains('keyflow.event.nodeDragged')),
        isTrue,
      );
    });

    test('tracks error event', () async {
      analytics.trackError(
        errorType: 'ffi_error',
        errorMessage: 'Connection failed',
        operation: 'addSlot',
      );

      await Future.delayed(const Duration(milliseconds: 10));

      expect(logOutput, isNotEmpty);
      expect(
        logOutput.any((line) => line.contains('keyflow.event.errorOccurred')),
        isTrue,
      );
    });

    test('tracks edge interaction event', () async {
      analytics.trackEdgeInteraction(
        action: 'select',
        edgeId: 'edge1',
        isActive: true,
      );

      await Future.delayed(const Duration(milliseconds: 10));

      expect(logOutput, isNotEmpty);
      expect(
        logOutput.any((line) => line.contains('keyflow.event.edgeInteracted')),
        isTrue,
      );
    });

    test('tracks dialog opened event', () async {
      analytics.trackDialogOpened(dialogType: 'add_slot');

      await Future.delayed(const Duration(milliseconds: 10));

      expect(logOutput, isNotEmpty);
      expect(
        logOutput.any((line) => line.contains('keyflow.event.dialogOpened')),
        isTrue,
      );
    });

    test('tracks keyboard shortcut event', () async {
      analytics.trackKeyboardShortcut(shortcut: 'Ctrl+0', action: 'resetZoom');

      await Future.delayed(const Duration(milliseconds: 10));

      expect(logOutput, isNotEmpty);
      expect(
        logOutput.any(
          (line) => line.contains('keyflow.event.keyboardShortcut'),
        ),
        isTrue,
      );
    });

    test('tracks canvas manipulation event', () async {
      analytics.trackCanvasManipulation(action: 'zoom', scale: 1.5);

      await Future.delayed(const Duration(milliseconds: 10));

      expect(logOutput, isNotEmpty);
      expect(
        logOutput.any(
          (line) => line.contains('keyflow.event.canvasManipulated'),
        ),
        isTrue,
      );
    });

    test('tracks auto-save event', () async {
      analytics.trackAutoSave(
        operation: 'addSlot',
        success: true,
        retryCount: 0,
      );

      await Future.delayed(const Duration(milliseconds: 10));

      expect(logOutput, isNotEmpty);
      expect(
        logOutput.any(
          (line) => line.contains('keyflow.event.autoSaveTriggered'),
        ),
        isTrue,
      );
    });

    test('does not track events when disabled', () async {
      final disabledAnalytics = KeyFlowAnalytics(
        logger: logger,
        enabled: false,
      );

      logOutput.clear();
      disabledAnalytics.trackPageView(deviceCount: 1);

      await Future.delayed(const Duration(milliseconds: 10));

      expect(logOutput, isEmpty);

      disabledAnalytics.dispose();
    });

    test('emits events to stream', () async {
      final events = <KeyFlowEvent>[];
      final subscription = analytics.eventStream.listen(events.add);

      analytics.trackPageView(deviceCount: 1);
      analytics.trackSlotAdded(
        deviceId: 'device1',
        profileId: 'profile1',
        keymapId: 'keymap1',
      );

      await Future.delayed(const Duration(milliseconds: 10));

      expect(events, hasLength(2));
      expect(events[0].type, KeyFlowEventType.pageView);
      expect(events[1].type, KeyFlowEventType.slotAdded);

      await subscription.cancel();
    });

    test('disposes cleanly', () {
      analytics.dispose();

      // Should not throw
      analytics.trackPageView(deviceCount: 1);
    });
  });
}
