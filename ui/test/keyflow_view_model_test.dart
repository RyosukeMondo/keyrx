import 'dart:async';

import 'package:fake_async/fake_async.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/services/facade/keyflow_repository.dart';
import 'package:keyrx_ui/state/keyflow_view_model.dart';
import 'package:mocktail/mocktail.dart';

class _MockKeyFlowRepository extends Mock implements KeyFlowRepository {}

void main() {
  late _MockKeyFlowRepository repository;
  late StreamController<RuntimeOverlay> runtimeController;

  setUp(() {
    repository = _MockKeyFlowRepository();
    runtimeController = StreamController<RuntimeOverlay>.broadcast();

    when(
      () => repository.runtimeStream(),
    ).thenAnswer((_) => runtimeController.stream);
    when(() => repository.dispose()).thenAnswer((_) async {});
  });

  tearDown(() async {
    await runtimeController.close();
  });

  KeyFlowSnapshot _snapshot({
    List<KeyFlowDevice>? devices,
    RuntimeOverlay? runtime,
    List<KeyFlowError> errors = const [],
  }) {
    final deviceList =
        devices ??
        const [
          KeyFlowDevice(
            id: 'dev-a',
            displayName: 'A',
            remapEnabled: true,
            profileId: 'profile-1',
            layoutId: 'layout-1',
            wiring: {},
            gaps: 1,
            conflicts: 0,
          ),
        ];
    return KeyFlowSnapshot(
      devices: deviceList,
      profiles: const {},
      layouts: const {},
      keymaps: const {},
      generatedAt: DateTime.now(),
      runtime: runtime ?? RuntimeOverlay.initial(),
      runtimeConfig: null,
      errors: errors,
    );
  }

  group('KeyFlowViewModel', () {
    test('load sets snapshot, derived counts, and selection', () async {
      final snapshot = _snapshot(
        devices: const [
          KeyFlowDevice(
            id: 'dev-a',
            displayName: 'A',
            remapEnabled: true,
            profileId: 'profile-1',
            layoutId: 'layout-1',
            wiring: {},
            gaps: 2,
            conflicts: 3,
          ),
        ],
        runtime: RuntimeOverlay(
          activeLayers: const [0],
          activeModifiers: const [],
          heldKeys: const [],
          pendingCount: 0,
          lastUpdate: DateTime.now(),
        ),
      );
      when(() => repository.loadAll()).thenAnswer((_) async => snapshot);

      final viewModel = KeyFlowViewModel(
        repository: repository,
        runtimeStaleAfter: const Duration(milliseconds: 50),
      );

      await viewModel.load();

      expect(viewModel.snapshot, isNotNull);
      expect(viewModel.selectedDevice?.id, 'dev-a');
      expect(viewModel.totalGaps, 2);
      expect(viewModel.totalConflicts, 3);
      expect(viewModel.isRuntimeStale, isFalse);
      await viewModel.dispose();
    });

    test(
      'refresh debounces and reconciles selection when device disappears',
      () {
        fakeAsync((async) {
          final initial = _snapshot(
            devices: const [
              KeyFlowDevice(
                id: 'dev-a',
                displayName: 'A',
                remapEnabled: true,
                profileId: 'profile-1',
                layoutId: 'layout-1',
                wiring: {},
                gaps: 0,
                conflicts: 0,
              ),
              KeyFlowDevice(
                id: 'dev-b',
                displayName: 'B',
                remapEnabled: true,
                profileId: 'profile-1',
                layoutId: 'layout-1',
                wiring: {},
                gaps: 0,
                conflicts: 0,
              ),
            ],
          );
          final refreshed = _snapshot(
            devices: const [
              KeyFlowDevice(
                id: 'dev-a',
                displayName: 'A',
                remapEnabled: true,
                profileId: 'profile-1',
                layoutId: 'layout-1',
                wiring: {},
                gaps: 0,
                conflicts: 0,
              ),
            ],
          );

          when(() => repository.loadAll()).thenAnswer((_) async => initial);
          when(
            () => repository.refreshDevices(),
          ).thenAnswer((_) async => refreshed);

          final viewModel = KeyFlowViewModel(
            repository: repository,
            refreshDebounce: const Duration(milliseconds: 20),
          );

          viewModel.load();
          async.flushMicrotasks();
          viewModel.selectDevice('dev-b');

          viewModel.refresh();
          async.elapse(const Duration(milliseconds: 5));
          expect(viewModel.isRefreshing, isFalse);

          async.elapse(const Duration(milliseconds: 20));
          async.flushMicrotasks();
          expect(viewModel.selectedDevice?.id, 'dev-a');
          expect(viewModel.isRefreshing, isFalse);
          viewModel.dispose();
        });
      },
    );

    test(
      'runtime updates clear stale flag then mark stale after threshold',
      () {
        fakeAsync((async) {
          final now = DateTime.now();
          final overlay = RuntimeOverlay(
            activeLayers: const [0],
            activeModifiers: const ['M1'],
            heldKeys: const ['K1'],
            pendingCount: 0,
            lastUpdate: now,
          );

          final viewModel = KeyFlowViewModel(
            repository: repository,
            runtimeStaleAfter: const Duration(milliseconds: 30),
          );

          runtimeController.add(overlay);
          async.flushMicrotasks();
          expect(viewModel.isRuntimeStale, isFalse);

          async.elapse(const Duration(milliseconds: 40));
          expect(viewModel.isRuntimeStale, isTrue);
          viewModel.dispose();
        });
      },
    );
  });
}
