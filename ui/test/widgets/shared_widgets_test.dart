import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:provider/provider.dart';

import 'package:keyrx_ui/pages/training_screen.dart';
import 'package:keyrx_ui/services/audio_service.dart';
import 'package:keyrx_ui/services/error_translator.dart';
import 'package:keyrx_ui/services/permission_service.dart';
import 'package:keyrx_ui/services/service_registry.dart';
import 'package:keyrx_ui/ui/widgets/app_error_dialog.dart';
import 'package:keyrx_ui/ui/widgets/loading_overlay.dart';

class _FakeAudioService implements AudioService {
  AudioState _state = AudioState.idle;
  AudioOperationResult startResult =
      const AudioOperationResult(success: true);
  AudioOperationResult stopResult = const AudioOperationResult(success: true);
  AudioOperationResult setBpmResult = const AudioOperationResult(success: true);
  final _controller = StreamController<ClassificationResult>.broadcast();
  int startCalls = 0;
  int stopCalls = 0;
  int? lastBpm;

  @override
  AudioState get state => _state;

  @override
  Stream<ClassificationResult> get classificationStream => _controller.stream;

  @override
  Future<AudioOperationResult> start({required int bpm}) async {
    startCalls += 1;
    lastBpm = bpm;
    _state = startResult.success ? AudioState.running : AudioState.idle;
    return startResult;
  }

  @override
  Future<AudioOperationResult> stop() async {
    stopCalls += 1;
    _state = AudioState.idle;
    return stopResult;
  }

  @override
  Future<AudioOperationResult> setBpm(int bpm) async {
    lastBpm = bpm;
    return setBpmResult;
  }

  @override
  Future<void> dispose() async {
    await _controller.close();
  }
}

class _StubPermissionService implements PermissionService {
  const _StubPermissionService([this.result = const PermissionResult(
    state: PermissionState.granted,
  )]);

  final PermissionResult result;

  @override
  Future<PermissionResult> checkMicrophone() async => result;

  @override
  Future<PermissionResult> requestMicrophone() async => result;
}

class _TestTranslator implements ErrorTranslator {
  const _TestTranslator(this.message);

  final UserMessage message;

  @override
  UserMessage translate(Object error) => message;
}

void main() {
  testWidgets('AppErrorDialog calls actions and closes dialog',
      (WidgetTester tester) async {
    var primaryCalled = false;
    var secondaryCalled = false;

    await tester.pumpWidget(
      MaterialApp(
        home: Builder(
          builder: (context) => Scaffold(
            body: Center(
              child: ElevatedButton(
                onPressed: () => AppErrorDialog.show(
                  context,
                  title: 'Error title',
                  message: 'Error body',
                  primaryActionLabel: 'Confirm',
                  onPrimaryAction: () => primaryCalled = true,
                  secondaryActionLabel: 'Cancel',
                  onSecondaryAction: () => secondaryCalled = true,
                ),
                child: const Text('Open'),
              ),
            ),
          ),
        ),
      ),
    );

    await tester.tap(find.text('Open'));
    await tester.pumpAndSettle();

    expect(find.text('Error title'), findsOneWidget);

    await tester.tap(find.text('Cancel'));
    await tester.pumpAndSettle();

    expect(secondaryCalled, isTrue);
    expect(find.byType(AlertDialog), findsNothing);

    await tester.tap(find.text('Open'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Confirm'));
    await tester.pumpAndSettle();

    expect(primaryCalled, isTrue);
    expect(find.byType(AlertDialog), findsNothing);
  });

  testWidgets('LoadingOverlay blocks interaction while loading',
      (WidgetTester tester) async {
    var tapped = false;

    await tester.pumpWidget(
      MaterialApp(
        home: LoadingOverlay(
          isLoading: true,
          message: 'Please wait',
          child: ElevatedButton(
            onPressed: () => tapped = true,
            child: const Text('Tap me'),
          ),
        ),
      ),
    );

    expect(find.text('Please wait'), findsOneWidget);
    await tester.tap(find.text('Tap me'), warnIfMissed: false);
    expect(tapped, isFalse);

    await tester.pumpWidget(
      MaterialApp(
        home: LoadingOverlay(
          isLoading: false,
          child: ElevatedButton(
            onPressed: () => tapped = true,
            child: const Text('Tap me'),
          ),
        ),
      ),
    );

    await tester.tap(find.text('Tap me'));
    expect(tapped, isTrue);
  });

  testWidgets('TrainingScreen surfaces service error via dialog',
      (WidgetTester tester) async {
    final audio = _FakeAudioService()
      ..startResult = const AudioOperationResult(
        success: false,
        userMessage: UserMessage(
          title: 'Unable to start',
          body: 'Engine failed',
        ),
      );

    final registry = ServiceRegistry.withOverrides(
      permissionService: const _StubPermissionService(),
      audioService: audio,
      errorTranslator: const _TestTranslator(
        UserMessage(title: 'Translated', body: 'Unused'),
      ),
    );

    addTearDown(registry.dispose);

    await tester.pumpWidget(
      Provider<ServiceRegistry>.value(
        value: registry,
        child: const MaterialApp(home: TrainingScreen()),
      ),
    );

    await tester.tap(find.text('Start'));
    await tester.pump();
    await tester.pumpAndSettle();

    expect(find.text('Unable to start'), findsOneWidget);
    expect(audio.startCalls, 1);
  });
}
