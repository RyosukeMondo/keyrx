// Widget tests for the trade-off visualizer page.
//
// Tests chart rendering, slider interaction, and preset visibility.

import 'dart:async';

import 'package:fl_chart/fl_chart.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/ffi/bridge.dart';
import 'package:keyrx_ui/pages/trade_off_visualizer.dart';
import 'package:keyrx_ui/services/audio_service.dart';
import 'package:keyrx_ui/services/engine_service.dart';
import 'package:keyrx_ui/services/error_translator.dart';
import 'package:keyrx_ui/services/permission_service.dart';
import 'package:keyrx_ui/services/service_registry.dart';
import 'package:keyrx_ui/repositories/mapping_repository.dart';
import 'package:provider/provider.dart';

class _FakeEngineService implements EngineService {
  final StreamController<EngineSnapshot> _stateController =
      StreamController.broadcast();

  @override
  bool get isInitialized => true;

  @override
  String get version => 'test';

  @override
  Future<bool> initialize() async => true;

  @override
  Future<bool> loadScript(String path) async => true;

  @override
  Future<ConsoleEvalResult> eval(String command) async =>
      const ConsoleEvalResult(success: true, output: 'ok');

  @override
  Stream<EngineSnapshot> get stateStream => _stateController.stream;

  @override
  Future<KeyRegistryResult> fetchKeyRegistry() async =>
      const KeyRegistryResult(entries: []);

  void emit(EngineSnapshot snapshot) {
    _stateController.add(snapshot);
  }

  @override
  Future<void> dispose() async {
    await _stateController.close();
  }
}

class _FakeAudioService implements AudioService {
  @override
  AudioState get state => AudioState.idle;

  @override
  Stream<ClassificationResult> get classificationStream => const Stream.empty();

  @override
  Future<AudioOperationResult> start({required int bpm}) async =>
      const AudioOperationResult(success: true);

  @override
  Future<AudioOperationResult> stop() async =>
      const AudioOperationResult(success: true);

  @override
  Future<AudioOperationResult> setBpm(int bpm) async =>
      const AudioOperationResult(success: true);

  @override
  Future<void> dispose() async {}
}

class _FakePermissionService implements PermissionService {
  @override
  Future<PermissionResult> checkMicrophone() async =>
      const PermissionResult(state: PermissionState.granted);

  @override
  Future<PermissionResult> requestMicrophone() async =>
      const PermissionResult(state: PermissionState.granted);
}

class _FakeErrorTranslator implements ErrorTranslator {
  @override
  UserMessage translate(Object error) =>
      const UserMessage(title: 'err', body: 'error');
}

Widget _buildTestWidget({_FakeEngineService? engine}) {
  final fakeEngine = engine ?? _FakeEngineService();
  final registry = ServiceRegistry.withOverrides(
    permissionService: _FakePermissionService(),
    audioService: _FakeAudioService(),
    errorTranslator: _FakeErrorTranslator(),
    engineService: fakeEngine,
    mappingRepository: MappingRepository(),
  );

  return MultiProvider(
    providers: [Provider<ServiceRegistry>.value(value: registry)],
    child: const MaterialApp(home: TradeOffVisualizerPage()),
  );
}

void main() {
  group('Chart rendering', () {
    testWidgets('renders LineChart widget', (tester) async {
      await tester.pumpWidget(_buildTestWidget());
      await tester.pumpAndSettle();

      expect(find.byType(LineChart), findsOneWidget);
    });

    testWidgets('displays chart title', (tester) async {
      await tester.pumpWidget(_buildTestWidget());
      await tester.pumpAndSettle();

      expect(find.text('Miss Rate vs. Timeout'), findsOneWidget);
    });

    testWidgets('shows axis labels', (tester) async {
      await tester.pumpWidget(_buildTestWidget());
      await tester.pumpAndSettle();

      expect(find.text('Tap-Hold Timeout (ms)'), findsOneWidget);
      expect(find.text('Miss Rate (%)'), findsOneWidget);
    });
  });

  group('Slider interaction', () {
    testWidgets('displays slider with initial value', (tester) async {
      await tester.pumpWidget(_buildTestWidget());
      await tester.pumpAndSettle();

      expect(find.byType(Slider), findsOneWidget);
      // Default value is 200ms
      expect(find.text('200 ms'), findsOneWidget);
    });

    testWidgets('slider updates displayed timeout value', (tester) async {
      await tester.pumpWidget(_buildTestWidget());
      await tester.pumpAndSettle();

      // Scroll to make slider visible
      await tester.dragUntilVisible(
        find.text('Adjust Timeout'),
        find.byType(SingleChildScrollView),
        const Offset(0, -50),
      );
      await tester.pumpAndSettle();

      // Find the slider
      final slider = find.byType(Slider);
      expect(slider, findsOneWidget);

      // Drag slider to the right (higher value)
      // The slider goes from 100 to 1000, so dragging right increases value
      await tester.drag(slider, const Offset(200, 0), warnIfMissed: false);
      await tester.pumpAndSettle();

      // The value should have changed from 200ms
      // Since we dragged right, expect a higher value
      expect(find.text('200 ms'), findsNothing);
    });

    testWidgets('shows estimated miss rate for current timeout', (tester) async {
      await tester.pumpWidget(_buildTestWidget());
      await tester.pumpAndSettle();

      expect(find.text('Estimated Miss Rate'), findsOneWidget);
      // Should show a percentage value
      expect(find.textContaining('%'), findsWidgets);
    });

    testWidgets('shows category label based on timeout', (tester) async {
      await tester.pumpWidget(_buildTestWidget());
      await tester.pumpAndSettle();

      expect(find.text('Category'), findsOneWidget);
      // Default 200ms is in Typing range
      expect(find.text('Typing'), findsWidgets);
    });
  });

  group('Preset visibility', () {
    testWidgets('displays preset configurations section', (tester) async {
      await tester.pumpWidget(_buildTestWidget());
      await tester.pumpAndSettle();

      expect(find.text('Preset Configurations'), findsOneWidget);
    });

    testWidgets('shows Gaming preset', (tester) async {
      await tester.pumpWidget(_buildTestWidget());
      await tester.pumpAndSettle();

      expect(find.text('Gaming'), findsWidgets);
      expect(find.text('100-150ms'), findsOneWidget);
      expect(find.textContaining('Fast response'), findsOneWidget);
    });

    testWidgets('shows Typing preset', (tester) async {
      await tester.pumpWidget(_buildTestWidget());
      await tester.pumpAndSettle();

      expect(find.text('Typing'), findsWidgets);
      expect(find.text('175-250ms'), findsOneWidget);
      expect(find.textContaining('Balanced'), findsOneWidget);
    });

    testWidgets('shows Relaxed preset', (tester) async {
      await tester.pumpWidget(_buildTestWidget());
      await tester.pumpAndSettle();

      // Scroll to see presets
      await tester.dragUntilVisible(
        find.text('Preset Configurations'),
        find.byType(SingleChildScrollView),
        const Offset(0, -100),
      );
      await tester.pumpAndSettle();

      expect(find.text('Relaxed'), findsWidgets);
      expect(find.text('300-500ms'), findsOneWidget);
      // "Slower" appears in preset description and slider labels
      expect(find.textContaining('Slower'), findsWidgets);
    });

    testWidgets('tapping preset updates slider to preset range', (tester) async {
      await tester.pumpWidget(_buildTestWidget());
      await tester.pumpAndSettle();

      // Scroll to make presets visible
      await tester.dragUntilVisible(
        find.text('Preset Configurations'),
        find.byType(SingleChildScrollView),
        const Offset(0, -100),
      );
      await tester.pumpAndSettle();

      // Find and tap the Gaming preset row
      final gamingRow = find.ancestor(
        of: find.text('Gaming'),
        matching: find.byType(InkWell),
      );
      expect(gamingRow, findsWidgets);

      await tester.tap(gamingRow.first);
      await tester.pumpAndSettle();

      // Scroll back up to see the timeout display
      await tester.dragUntilVisible(
        find.text('Adjust Timeout'),
        find.byType(SingleChildScrollView),
        const Offset(0, 100),
      );
      await tester.pumpAndSettle();

      // Should update to middle of Gaming range (125ms)
      expect(find.text('125 ms'), findsOneWidget);
    });

    testWidgets('selected preset shows checkmark', (tester) async {
      await tester.pumpWidget(_buildTestWidget());
      await tester.pumpAndSettle();

      // Scroll to make presets visible
      await tester.dragUntilVisible(
        find.text('Preset Configurations'),
        find.byType(SingleChildScrollView),
        const Offset(0, -100),
      );
      await tester.pumpAndSettle();

      // Default 200ms is in Typing range, should show checkmark
      expect(find.byIcon(Icons.check_circle), findsOneWidget);
    });
  });

  group('Explanation and help', () {
    testWidgets('displays explanation card', (tester) async {
      await tester.pumpWidget(_buildTestWidget());
      await tester.pumpAndSettle();

      expect(find.text('Understanding Tap-Hold Timing'), findsOneWidget);
      expect(find.textContaining('tap-hold timeout determines'), findsOneWidget);
    });

    testWidgets('shows help button in app bar', (tester) async {
      await tester.pumpWidget(_buildTestWidget());
      await tester.pumpAndSettle();

      expect(find.byIcon(Icons.help_outline), findsOneWidget);
    });

    testWidgets('help button opens dialog', (tester) async {
      await tester.pumpWidget(_buildTestWidget());
      await tester.pumpAndSettle();

      await tester.tap(find.byIcon(Icons.help_outline));
      await tester.pumpAndSettle();

      expect(find.text('Understanding Trade-offs'), findsOneWidget);
      expect(find.text('Got it'), findsOneWidget);
    });
  });

  group('Statistics card', () {
    testWidgets('displays typing model parameters', (tester) async {
      await tester.pumpWidget(_buildTestWidget());
      await tester.pumpAndSettle();

      expect(find.text('Typing Model Parameters'), findsOneWidget);
      expect(find.text('Mean Key Duration'), findsOneWidget);
      expect(find.text('Std Deviation'), findsOneWidget);
    });

    testWidgets('shows simulate button', (tester) async {
      await tester.pumpWidget(_buildTestWidget());
      await tester.pumpAndSettle();

      expect(find.text('Simulate My Typing Speed'), findsOneWidget);
    });

    testWidgets('simulate button opens dialog', (tester) async {
      await tester.pumpWidget(_buildTestWidget());
      await tester.pumpAndSettle();

      // Scroll down to reveal the simulate button
      await tester.dragUntilVisible(
        find.text('Simulate My Typing Speed'),
        find.byType(SingleChildScrollView),
        const Offset(0, -100),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.text('Simulate My Typing Speed'));
      await tester.pumpAndSettle();

      expect(find.text('Typing Speed Simulation'), findsOneWidget);
      expect(find.text('Cancel'), findsWidgets);
    });

    testWidgets('shows edit parameters button', (tester) async {
      await tester.pumpWidget(_buildTestWidget());
      await tester.pumpAndSettle();

      // Scroll down to reveal the statistics card
      await tester.dragUntilVisible(
        find.text('Typing Model Parameters'),
        find.byType(SingleChildScrollView),
        const Offset(0, -100),
      );
      await tester.pumpAndSettle();

      expect(find.byIcon(Icons.edit), findsOneWidget);
    });

    testWidgets('edit button opens model edit dialog', (tester) async {
      await tester.pumpWidget(_buildTestWidget());
      await tester.pumpAndSettle();

      // Scroll down to reveal the edit button
      await tester.dragUntilVisible(
        find.text('Typing Model Parameters'),
        find.byType(SingleChildScrollView),
        const Offset(0, -100),
      );
      await tester.pumpAndSettle();

      await tester.tap(find.byIcon(Icons.edit));
      await tester.pumpAndSettle();

      expect(find.text('Edit Typing Model'), findsOneWidget);
      expect(find.textContaining('Mean Key Duration'), findsWidgets);
      expect(find.textContaining('Standard Deviation'), findsWidgets);
    });
  });

  group('Page structure', () {
    testWidgets('has correct app bar title', (tester) async {
      await tester.pumpWidget(_buildTestWidget());
      await tester.pumpAndSettle();

      expect(find.text('Timing Trade-offs'), findsOneWidget);
    });

    testWidgets('page is scrollable', (tester) async {
      await tester.pumpWidget(_buildTestWidget());
      await tester.pumpAndSettle();

      expect(find.byType(SingleChildScrollView), findsOneWidget);
    });

    testWidgets('displays all main cards', (tester) async {
      await tester.pumpWidget(_buildTestWidget());
      await tester.pumpAndSettle();

      // Count Card widgets - should have 5 main cards
      expect(find.byType(Card), findsAtLeast(4));
    });
  });
}
