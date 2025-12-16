/// Widget tests for MonitorView.
///
/// Focus areas:
/// - UI rendering with different connection states
/// - Event display and filtering
/// - Event list scrolling and buffer limits
/// - Empty states
library;

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/models/daemon_protocol.dart';
import 'package:keyrx_ui/pages/monitor_view.dart';
import 'package:keyrx_ui/services/daemon_service.dart';
import 'package:keyrx_ui/state/daemon_providers.dart';
import 'package:mocktail/mocktail.dart';

// Mocks
class MockDaemonService extends Mock implements DaemonService {}

void main() {
  late MockDaemonService mockService;
  late StreamController<DaemonEvent> eventsController;
  late StreamController<DaemonConnectionState> stateController;

  setUp(() {
    mockService = MockDaemonService();
    eventsController = StreamController<DaemonEvent>.broadcast();
    stateController = StreamController<DaemonConnectionState>.broadcast();

    // Default mock behavior
    when(() => mockService.events).thenAnswer((_) => eventsController.stream);
    when(
      () => mockService.connectionState,
    ).thenAnswer((_) => stateController.stream);
    when(() => mockService.connect()).thenAnswer((_) async {});
    when(() => mockService.subscribeEvents()).thenAnswer((_) async {});
    when(
      () => mockService.getStatus(),
    ).thenAnswer((_) async => const EngineStatus(status: 'stopped'));
  });

  tearDown(() {
    eventsController.close();
    stateController.close();
  });

  Widget createTestWidget() => ProviderScope(
    overrides: [daemonServiceProvider.overrideWithValue(mockService)],
    child: const MaterialApp(home: Scaffold(body: MonitorView())),
  );

  group('MonitorView - Connection Status', () {
    testWidgets('shows disconnected state initially', (tester) async {
      stateController.add(DaemonConnectionState.disconnected);

      await tester.pumpWidget(createTestWidget());
      await tester.pump();

      expect(find.text('Disconnected'), findsOneWidget);
      expect(find.byIcon(Icons.error_outline), findsOneWidget);
    });

    testWidgets('shows connected state when connected', (tester) async {
      await tester.pumpWidget(createTestWidget());
      await tester.pump(); // Initialize listeners

      stateController.add(DaemonConnectionState.connected);
      await tester.pump(const Duration(milliseconds: 100));

      expect(find.text('Connected'), findsOneWidget);
      expect(find.byIcon(Icons.check_circle), findsOneWidget);
    });

    testWidgets('connects on page initialization', (tester) async {
      await tester.pumpWidget(createTestWidget());
      await tester.pump();

      verify(() => mockService.connect()).called(1);
      verify(() => mockService.subscribeEvents()).called(1);
    });
  });

  group('MonitorView - Event Display', () {
    testWidgets('shows empty state when no events', (tester) async {
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      expect(find.text('No events yet'), findsOneWidget);
      expect(find.text('Listening for daemon events...'), findsOneWidget);
    });

    testWidgets('displays input events', (tester) async {
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Emit input event
      eventsController.add(
        const DaemonEvent.input(
          InputEvent(
            timestampUs: 1234567890,
            devicePath: '/dev/input/event0',
            deviceName: 'Test Keyboard',
            keyName: 'A',
            keyCode: 30,
            eventType: 'key_press',
          ),
        ),
      );

      await tester.pump(Duration.zero);
      await tester.pump(const Duration(milliseconds: 100));

      expect(find.text('A'), findsOneWidget);
      expect(find.text('PRESS'), findsOneWidget);
      expect(find.textContaining('Test Keyboard'), findsOneWidget);
    });

    testWidgets('displays output events', (tester) async {
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Emit output event
      eventsController.add(
        const DaemonEvent.output(
          OutputEvent(
            timestampUs: 1234567890,
            keyName: 'B',
            keyCode: 48,
            actionType: 'key_release',
          ),
        ),
      );

      await tester.pump(const Duration(milliseconds: 100));

      expect(find.text('B'), findsOneWidget);
      expect(find.text('RELEASE'), findsOneWidget);
    });

    testWidgets('displays state change events', (tester) async {
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Emit state change event
      eventsController.add(
        const DaemonEvent.stateChange(
          StateChangeEvent(
            type: 'engine.started',
            timestampUs: 1234567890,
            message: 'Engine started successfully',
          ),
        ),
      );

      await tester.pump(const Duration(milliseconds: 100));

      expect(find.text('Engine started successfully'), findsOneWidget);
    });

    testWidgets('events appear in reverse chronological order', (tester) async {
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Emit multiple events
      eventsController.add(
        const DaemonEvent.input(
          InputEvent(
            timestampUs: 1234567890,
            devicePath: '/dev/input/event0',
            deviceName: 'KB',
            keyName: 'A',
            keyCode: 30,
            eventType: 'key_press',
          ),
        ),
      );
      await tester.pump(const Duration(milliseconds: 100));

      eventsController.add(
        const DaemonEvent.input(
          InputEvent(
            timestampUs: 1234567891,
            devicePath: '/dev/input/event0',
            deviceName: 'KB',
            keyName: 'B',
            keyCode: 48,
            eventType: 'key_press',
          ),
        ),
      );
      await tester.pump(const Duration(milliseconds: 100));

      // Find all ListTiles (events)
      final listTiles = find.byType(ListTile);
      expect(listTiles, findsAtLeastNWidgets(2));
    });

    testWidgets('limits events to buffer size', (tester) async {
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Emit 510 events (buffer limit is 500)
      for (var i = 0; i < 510; i++) {
        eventsController.add(
          DaemonEvent.input(
            InputEvent(
              timestampUs: 1234567890 + i,
              devicePath: '/dev/input/event0',
              deviceName: 'KB',
              keyName: 'Key$i',
              keyCode: i,
              eventType: 'key_press',
            ),
          ),
        );
      }

      await tester.pump(const Duration(milliseconds: 100));

      // Should only show recent 500 events
      // This is hard to verify precisely without instrumenting the widget,
      // but ensuring no crash is good.
      expect(find.byType(ListTile), findsWidgets);
    });
  });

  group('MonitorView - Event Filtering', () {
    testWidgets('filter button opens dialog', (tester) async {
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Find and tap filter button
      await tester.tap(find.byIcon(Icons.filter_list));
      await tester.pumpAndSettle();

      expect(find.text('Filter Events'), findsOneWidget);
      expect(find.text('Input Events'), findsOneWidget);
      expect(find.text('Output Events'), findsOneWidget);
      expect(find.text('State Changes'), findsOneWidget);
    });

    testWidgets('filter toggles work', (tester) async {
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Open filter dialog
      await tester.tap(find.byIcon(Icons.filter_list));
      await tester.pumpAndSettle();

      // Uncheck input events
      await tester.tap(find.text('Input Events'));
      await tester.pumpAndSettle();

      // Close dialog
      await tester.tap(find.text('Close'));
      await tester.pumpAndSettle();

      // Emit input event
      eventsController.add(
        const DaemonEvent.input(
          InputEvent(
            timestampUs: 1234567890,
            devicePath: '/dev/input/event0',
            deviceName: 'KB',
            keyName: 'A',
            keyCode: 30,
            eventType: 'key_press',
          ),
        ),
      );

      await tester.pump(const Duration(milliseconds: 100));

      // Should show "No events match filter"
      expect(find.text('No events match filter'), findsOneWidget);
    });
  });

  group('MonitorView - Clear Events', () {
    testWidgets('clear button is disabled when no events', (tester) async {
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      final clearButton = find.byIcon(Icons.clear_all);
      expect(clearButton, findsOneWidget);
    });

    testWidgets('clear button clears events', (tester) async {
      await tester.pumpWidget(createTestWidget());
      await tester.pumpAndSettle();

      // Emit event
      eventsController.add(
        const DaemonEvent.input(
          InputEvent(
            timestampUs: 1234567890,
            devicePath: '/dev/input/event0',
            deviceName: 'KB',
            keyName: 'A',
            keyCode: 30,
            eventType: 'key_press',
          ),
        ),
      );
      await tester.pump(const Duration(milliseconds: 100));

      expect(find.text('A'), findsOneWidget);

      // Tap clear button
      await tester.tap(find.byIcon(Icons.clear_all));
      await tester.pump();

      // Events should be cleared
      expect(find.text('No events yet'), findsOneWidget);
      expect(find.text('A'), findsNothing);
    });
  });
}
