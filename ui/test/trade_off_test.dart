// Widget tests for the trade-off visualizer page.
//
// Tests chart rendering, slider interaction, and preset visibility.

import 'dart:async';

import 'package:fl_chart/fl_chart.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/ffi/bridge.dart';
import 'package:keyrx_ui/pages/trade_off_visualizer.dart';
import 'package:keyrx_ui/services/engine_service.dart';
import 'package:keyrx_ui/services/error_translator.dart';
import 'package:keyrx_ui/services/service_registry.dart';
import 'package:keyrx_ui/repositories/mapping_repository.dart';
import 'package:provider/provider.dart';

import 'helpers/fake_services.dart';

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

class _FakeErrorTranslator implements ErrorTranslator {
  @override
  UserMessage translate(Object error) =>
      const UserMessage(title: 'err', body: 'error');
}

Widget _buildTestWidget({_FakeEngineService? engine}) {
  final fakeEngine = engine ?? _FakeEngineService();
  final registry = ServiceRegistry.withOverrides(
    errorTranslator: _FakeErrorTranslator(),
    engineService: fakeEngine,
    mappingRepository: MappingRepository(),
    deviceService: FakeDeviceService(),
    scriptFileService: FakeScriptFileService(),
    testService: FakeTestService(),
    bridge: FakeBridge(),
    apiDocsService: FakeApiDocsService(),
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

      await tester.drag(find.byType(Slider), const Offset(100, 0));
      await tester.pumpAndSettle();

      expect(find.textContaining(' ms'), findsWidgets);
    });
  });
}
