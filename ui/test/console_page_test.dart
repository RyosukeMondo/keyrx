import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/ffi/bridge.dart';
import 'package:keyrx_ui/pages/console.dart';
import 'package:keyrx_ui/services/engine_service.dart';

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

  Future<ConsoleEvalResult> Function(String command)? onEval;

  @override
  Future<ConsoleEvalResult> eval(String command) async =>
      onEval != null
          ? await onEval!(command)
          : ConsoleEvalResult(success: true, output: 'ok: $command');

  @override
  Stream<EngineSnapshot> get stateStream => _stateController.stream;

  @override
  Future<KeyRegistryResult> fetchKeyRegistry() async =>
      const KeyRegistryResult(entries: []);

  @override
  Future<void> dispose() async {
    await _stateController.close();
  }
}

void main() {
  testWidgets('Console executes commands via EngineService', (tester) async {
    final fakeEngine = _FakeEngineService();

    await tester.pumpWidget(
      MaterialApp(home: ConsolePage(engineService: fakeEngine)),
    );

    await tester.enterText(find.byType(TextField), 'print("hi")');
    await tester.pump();
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pump();

    expect(find.textContaining('print("hi")'), findsWidgets);
    expect(find.text('OK'), findsWidgets);
  });

  testWidgets('Console renders error styling for error-prefixed output', (tester) async {
    final fakeEngine = _FakeEngineService()
      ..onEval = (_) async => const ConsoleEvalResult(
            success: false,
            output: 'error: engine unavailable',
          );

    await tester.pumpWidget(
      MaterialApp(home: ConsolePage(engineService: fakeEngine)),
    );

    await tester.enterText(find.byType(TextField), 'status');
    await tester.pump();
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pump();

    expect(find.textContaining('status'), findsWidgets);
    expect(find.text('ERROR'), findsWidgets);
    expect(find.textContaining('engine unavailable'), findsWidgets);
  });
}
