import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:keyrx_ui/ffi/bridge.dart';
import 'package:keyrx_ui/pages/console.dart';
import 'package:keyrx_ui/services/console_parser.dart';
import 'package:keyrx_ui/services/engine_service.dart';

class _FakeEngineService implements EngineService {
  final StreamController<EngineSnapshot> _stateController =
      StreamController.broadcast();

  bool initializeResult = true;
  List<String> evalCommands = [];
  Future<ConsoleEvalResult> Function(String command)? onEval;

  @override
  bool get isInitialized => initializeResult;

  @override
  String get version => 'test';

  @override
  Future<bool> initialize() async => initializeResult;

  @override
  Future<bool> loadScript(String path) async => true;

  @override
  Future<ConsoleEvalResult> eval(String command) async {
    evalCommands.add(command);
    if (onEval != null) {
      return onEval!(command);
    }
    return ConsoleEvalResult(success: true, output: 'ok: $command');
  }

  @override
  Stream<EngineSnapshot> get stateStream => _stateController.stream;

  @override
  Future<KeyRegistryResult> fetchKeyRegistry() async =>
      const KeyRegistryResult(entries: []);

  @override
  Future<void> dispose() async {
    await _stateController.close();
  }

  @override
  Future<void> stop() async {
    // No-op
  }
}

void main() {
  late _FakeEngineService fakeEngine;

  setUp(() {
    fakeEngine = _FakeEngineService();
  });

  tearDown(() async {
    await fakeEngine.dispose();
  });

  Widget buildTestWidget({ConsoleParser? parser}) {
    return MaterialApp(
      home: ConsolePage(
        engineService: fakeEngine,
        parser: parser ?? const ConsoleParser(),
      ),
    );
  }

  testWidgets('displays initial UI with input field and clear button', (
    tester,
  ) async {
    await tester.pumpWidget(buildTestWidget());

    expect(find.text('Rhai Console'), findsOneWidget);
    expect(find.byType(TextField), findsOneWidget);
    expect(find.byIcon(Icons.clear_all), findsOneWidget);
    // The prompt character is in a Text widget
    expect(find.textContaining('>'), findsWidgets);
  });

  testWidgets('executes command on submit and shows result', (tester) async {
    await tester.pumpWidget(buildTestWidget());

    await tester.enterText(find.byType(TextField), 'print("hello")');
    await tester.pump();
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pump();

    expect(fakeEngine.evalCommands, contains('print("hello")'));
    expect(find.textContaining('print("hello")'), findsWidgets);
    expect(find.text('OK'), findsWidgets);
  });

  testWidgets('shows error badge for failed commands', (tester) async {
    fakeEngine.onEval = (_) async =>
        const ConsoleEvalResult(success: false, output: 'error: syntax error');

    await tester.pumpWidget(buildTestWidget());

    await tester.enterText(find.byType(TextField), 'bad syntax');
    await tester.pump();
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pump();

    expect(find.text('ERROR'), findsWidgets);
    expect(find.textContaining('syntax error'), findsWidgets);
  });

  testWidgets('clear button removes all history', (tester) async {
    await tester.pumpWidget(buildTestWidget());

    // Execute a command
    await tester.enterText(find.byType(TextField), 'test command');
    await tester.pump();
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pump();

    expect(find.textContaining('test command'), findsWidgets);

    // Clear history
    await tester.tap(find.byIcon(Icons.clear_all));
    await tester.pump();

    expect(find.textContaining('test command'), findsNothing);
  });

  testWidgets('empty commands are not executed', (tester) async {
    await tester.pumpWidget(buildTestWidget());

    await tester.enterText(find.byType(TextField), '   ');
    await tester.pump();
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pump();

    expect(fakeEngine.evalCommands, isEmpty);
  });

  testWidgets('initialize engine button appears on init error', (tester) async {
    fakeEngine.initializeResult = false;
    fakeEngine.onEval = (_) async => const ConsoleEvalResult(
      success: false,
      output: 'error: Engine not initialized',
    );

    await tester.pumpWidget(buildTestWidget());

    await tester.enterText(find.byType(TextField), 'status');
    await tester.pump();
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pump();

    expect(find.text('Initialize Engine'), findsOneWidget);
  });

  testWidgets('initialize engine button works', (tester) async {
    fakeEngine.initializeResult = true;
    fakeEngine.onEval = (_) async => const ConsoleEvalResult(
      success: false,
      output: 'error: Engine not initialized',
    );

    await tester.pumpWidget(buildTestWidget());

    // Trigger the error that shows init button
    await tester.enterText(find.byType(TextField), 'status');
    await tester.pump();
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pump();

    // Click initialize
    await tester.tap(find.text('Initialize Engine'));
    await tester.pump();

    expect(find.textContaining('Engine initialized'), findsOneWidget);
  });

  testWidgets('multiple commands build history', (tester) async {
    await tester.pumpWidget(buildTestWidget());

    // Execute multiple commands
    await tester.enterText(find.byType(TextField), 'cmd1');
    await tester.pump();
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pump();

    await tester.enterText(find.byType(TextField), 'cmd2');
    await tester.pump();
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pump();

    expect(fakeEngine.evalCommands, equals(['cmd1', 'cmd2']));
    expect(find.textContaining('cmd1'), findsWidgets);
    expect(find.textContaining('cmd2'), findsWidgets);
  });

  testWidgets('input is cleared after command execution', (tester) async {
    await tester.pumpWidget(buildTestWidget());

    await tester.enterText(find.byType(TextField), 'test');
    await tester.pump();
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pump();

    final textField = tester.widget<TextField>(find.byType(TextField));
    expect(textField.controller?.text, isEmpty);
  });

  testWidgets('console shows ok output styling', (tester) async {
    fakeEngine.onEval = (_) async =>
        const ConsoleEvalResult(success: true, output: 'ok: command succeeded');

    await tester.pumpWidget(buildTestWidget());

    await tester.enterText(find.byType(TextField), 'test');
    await tester.pump();
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pump();

    expect(find.text('OK'), findsWidgets);
  });

  testWidgets('console handles command with special characters', (
    tester,
  ) async {
    await tester.pumpWidget(buildTestWidget());

    await tester.enterText(find.byType(TextField), 'remap("a", "b")');
    await tester.pump();
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pump();

    expect(fakeEngine.evalCommands, contains('remap("a", "b")'));
  });

  testWidgets('input field is disabled during command execution', (
    tester,
  ) async {
    final completer = Completer<ConsoleEvalResult>();
    fakeEngine.onEval = (_) => completer.future;

    await tester.pumpWidget(buildTestWidget());

    await tester.enterText(find.byType(TextField), 'slow command');
    await tester.pump();
    await tester.testTextInput.receiveAction(TextInputAction.done);
    await tester.pump();

    // Input should be disabled while waiting
    final textField = tester.widget<TextField>(find.byType(TextField));
    expect(textField.enabled, isFalse);

    // Complete the command
    completer.complete(const ConsoleEvalResult(success: true, output: 'ok'));
    await tester.pump();

    // Input should be enabled again
    final textField2 = tester.widget<TextField>(find.byType(TextField));
    expect(textField2.enabled, isTrue);
  });
}
