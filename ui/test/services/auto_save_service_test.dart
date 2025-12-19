library;

import 'dart:async';

import 'package:core_logic/models/hardware_profile.dart';
import 'package:core_logic/models/runtime_config.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/repositories/keyflow_repository.dart';
import 'package:keyrx_ui/services/auto_save_service.dart';
import 'package:keyrx_ui/services/config_result.dart';
import 'package:core_logic/utils/logger.dart';

// Fake implementations for testing
class FakeKeyFlowRepository implements KeyFlowRepository {
  FakeKeyFlowRepository({this.addSlotFn, this.updateSlotFn, this.removeSlotFn});

  Future<ConfigOperationResult<RuntimeConfig>> Function(
    DeviceInstanceId,
    ProfileSlot,
    int?,
  )?
  addSlotFn;

  Future<ConfigOperationResult<RuntimeConfig>> Function(
    DeviceInstanceId,
    ProfileSlot,
  )?
  updateSlotFn;

  Future<ConfigOperationResult<RuntimeConfig>> Function(
    DeviceInstanceId,
    String,
  )?
  removeSlotFn;

  @override
  Future<ConfigOperationResult<RuntimeConfig>> addSlot(
    DeviceInstanceId device,
    ProfileSlot slot, {
    int? priorityOverride,
  }) async {
    if (addSlotFn != null) {
      return addSlotFn!(device, slot, priorityOverride);
    }
    return ConfigOperationResult.error('Not configured');
  }

  @override
  Future<ConfigOperationResult<RuntimeConfig>> updateSlot(
    DeviceInstanceId device,
    ProfileSlot slot,
  ) async {
    if (updateSlotFn != null) {
      return updateSlotFn!(device, slot);
    }
    return ConfigOperationResult.error('Not configured');
  }

  @override
  Future<ConfigOperationResult<RuntimeConfig>> removeSlot(
    DeviceInstanceId device,
    String slotId,
  ) async {
    if (removeSlotFn != null) {
      return removeSlotFn!(device, slotId);
    }
    return ConfigOperationResult.error('Not configured');
  }

  @override
  Future<void> dispose() async {}

  @override
  Future<ConfigOperationResult<RuntimeConfig>> getRuntimeConfig({
    bool forceRefresh = false,
  }) async => ConfigOperationResult.error('Not implemented');

  @override
  Stream<RuntimeConfig> watchRuntimeConfig() => const Stream.empty();
}

class FakeLogger implements StructuredLogger {
  @override
  void trace(
    String event, {
    String? message,
    Map<String, Object?> context = const {},
    Object? error,
    StackTrace? stackTrace,
  }) {}

  @override
  void debug(
    String event, {
    String? message,
    Map<String, Object?> context = const {},
    Object? error,
    StackTrace? stackTrace,
  }) {}

  @override
  void error(
    String event, {
    String? message,
    Map<String, Object?> context = const {},
    Object? error,
    StackTrace? stackTrace,
  }) {}

  @override
  void info(
    String event, {
    String? message,
    Map<String, Object?> context = const {},
    Object? error,
    StackTrace? stackTrace,
  }) {}

  @override
  void warn(
    String event, {
    String? message,
    Map<String, Object?> context = const {},
    Object? error,
    StackTrace? stackTrace,
  }) {}
}

void main() {
  group('AutoSaveService', () {
    late FakeKeyFlowRepository fakeRepo;
    late FakeLogger fakeLogger;
    late AutoSaveService service;

    const testDevice = DeviceInstanceId(
      vendorId: 0x1234,
      productId: 0x5678,
      serial: 'TEST-001',
    );

    const testSlot = ProfileSlot(
      id: 'slot-1',
      hardwareProfileId: 'profile-1',
      keymapId: 'keymap-1',
      active: true,
    );

    const testConfig = RuntimeConfig(
      devices: [
        DeviceSlots(device: testDevice, slots: [testSlot]),
      ],
    );

    setUp(() {
      fakeRepo = FakeKeyFlowRepository();
      fakeLogger = FakeLogger();
      service = AutoSaveService(
        repository: fakeRepo,
        debounceDelay: const Duration(milliseconds: 50),
        logger: fakeLogger,
      );
    });

    tearDown(() {
      service.dispose();
    });

    test('initializes with idle state', () {
      expect(service.currentState.status, SaveStatus.idle);
      expect(service.currentState.isIdle, true);
    });

    test('debounces save operations', () async {
      var callCount = 0;
      fakeRepo.addSlotFn = (_, __, ___) async {
        callCount++;
        return ConfigOperationResult.success(testConfig);
      };

      service.scheduleSave(const AddSlotOperation(testDevice, testSlot));
      service.scheduleSave(const AddSlotOperation(testDevice, testSlot));
      service.scheduleSave(const AddSlotOperation(testDevice, testSlot));

      await Future.delayed(const Duration(milliseconds: 100));

      expect(callCount, 1);
    });

    test('transitions through saving -> saved states', () async {
      fakeRepo.addSlotFn = (_, __, ___) async =>
          ConfigOperationResult.success(testConfig);

      final states = <SaveState>[];
      final subscription = service.stateStream.listen(states.add);

      service.scheduleSave(const AddSlotOperation(testDevice, testSlot));

      await Future.delayed(const Duration(milliseconds: 150));

      expect(states.length, greaterThanOrEqualTo(2));
      expect(states[0].status, SaveStatus.saving);
      expect(states[1].status, SaveStatus.saved);

      await subscription.cancel();
    });

    test('handles save errors with retry logic', () async {
      var callCount = 0;
      fakeRepo.addSlotFn = (_, __, ___) async {
        callCount++;
        if (callCount < 3) {
          return ConfigOperationResult.error('Network error');
        }
        return ConfigOperationResult.success(testConfig);
      };

      service.scheduleSave(const AddSlotOperation(testDevice, testSlot));

      await Future.delayed(const Duration(milliseconds: 1000));

      expect(callCount, 3);
      expect(service.currentState.status, SaveStatus.saved);
    });

    test('transitions to error state after max retries', () async {
      fakeRepo.addSlotFn = (_, __, ___) async =>
          ConfigOperationResult.error('Persistent error');

      final states = <SaveState>[];
      final subscription = service.stateStream.listen(states.add);

      service.scheduleSave(const AddSlotOperation(testDevice, testSlot));

      await Future.delayed(const Duration(milliseconds: 1500));

      final errorStates = states
          .where((s) => s.status == SaveStatus.error)
          .toList();
      expect(errorStates, isNotEmpty);
      expect(errorStates.last.errorMessage, isNotNull);

      await subscription.cancel();
    });

    test('supports manual retry after error', () async {
      var callCount = 0;
      fakeRepo.addSlotFn = (_, __, ___) async {
        callCount++;
        if (callCount <= 3) {
          return ConfigOperationResult.error('Network error');
        }
        return ConfigOperationResult.success(testConfig);
      };

      service.scheduleSave(const AddSlotOperation(testDevice, testSlot));

      await Future.delayed(const Duration(milliseconds: 1500));
      expect(service.currentState.status, SaveStatus.error);

      await service.retry();
      await Future.delayed(const Duration(milliseconds: 150));

      expect(service.currentState.status, SaveStatus.saved);
    });

    test('handles updateSlot operation', () async {
      var called = false;
      fakeRepo.updateSlotFn = (_, __) async {
        called = true;
        return ConfigOperationResult.success(testConfig);
      };

      service.scheduleSave(const UpdateSlotOperation(testDevice, testSlot));

      await Future.delayed(const Duration(milliseconds: 150));

      expect(called, true);
      expect(service.currentState.status, SaveStatus.saved);
    });

    test('handles removeSlot operation', () async {
      var called = false;
      fakeRepo.removeSlotFn = (_, __) async {
        called = true;
        return ConfigOperationResult.success(testConfig);
      };

      service.scheduleSave(const RemoveSlotOperation(testDevice, 'slot-1'));

      await Future.delayed(const Duration(milliseconds: 150));

      expect(called, true);
      expect(service.currentState.status, SaveStatus.saved);
    });

    test('SaveState equality works correctly', () {
      final state1 = SaveState.idle();
      final state2 = SaveState.idle();
      final state3 = SaveState.saving();
      final state4 = SaveState.error('Test error');
      final state5 = SaveState.error('Test error');

      expect(state1, equals(state2));
      expect(state1, isNot(equals(state3)));
      expect(state4, equals(state5));
    });

    test('SaveState toString provides useful debug info', () {
      final idle = SaveState.idle();
      final saving = SaveState.saving();
      final saved = SaveState.saved();
      final error = SaveState.error('Network error');

      expect(idle.toString(), contains('idle'));
      expect(saving.toString(), contains('saving'));
      expect(saved.toString(), contains('saved'));
      expect(error.toString(), contains('error'));
      expect(error.toString(), contains('Network error'));
    });
  });
}
