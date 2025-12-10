import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/services/facade/facade_state.dart';

void main() {
  group('FacadeState', () {
    test('initial() creates state with all uninitialized/idle statuses', () {
      final state = FacadeState.initial();

      expect(state.engine, EngineStatus.uninitialized);
      expect(state.device, DeviceStatus.none);
      expect(state.validation, ValidationStatus.none);
      expect(state.discovery, DiscoveryStatus.idle);
      expect(state.scriptPath, isNull);
      expect(state.selectedDevicePath, isNull);
      expect(state.validationErrorCount, isNull);
      expect(state.validationWarningCount, isNull);
      expect(state.discoveredDeviceCount, isNull);
      expect(state.lastError, isNull);
    });

    test('copyWith updates specific fields', () {
      final initial = FacadeState.initial();
      final updated = initial.copyWith(
        engine: EngineStatus.ready,
        scriptPath: '/path/to/script.lua',
      );

      expect(updated.engine, EngineStatus.ready);
      expect(updated.scriptPath, '/path/to/script.lua');
      expect(updated.device, DeviceStatus.none); // unchanged
      expect(updated.validation, ValidationStatus.none); // unchanged
    });

    test('withEngineStatus updates engine status and timestamp', () {
      final initial = FacadeState.initial();
      final updated = initial.withEngineStatus(
        EngineStatus.running,
        scriptPath: '/path/to/script.lua',
      );

      expect(updated.engine, EngineStatus.running);
      expect(updated.scriptPath, '/path/to/script.lua');
      expect(updated.timestamp.isAfter(initial.timestamp), isTrue);
    });

    test('withDeviceStatus updates device status and timestamp', () {
      final initial = FacadeState.initial();
      final updated = initial.withDeviceStatus(
        DeviceStatus.connected,
        devicePath: '/dev/input/event0',
      );

      expect(updated.device, DeviceStatus.connected);
      expect(updated.selectedDevicePath, '/dev/input/event0');
      expect(updated.timestamp.isAfter(initial.timestamp), isTrue);
    });

    test('withValidationStatus updates validation status and counts', () {
      final initial = FacadeState.initial();
      final updated = initial.withValidationStatus(
        ValidationStatus.invalid,
        errorCount: 5,
        warningCount: 2,
      );

      expect(updated.validation, ValidationStatus.invalid);
      expect(updated.validationErrorCount, 5);
      expect(updated.validationWarningCount, 2);
      expect(updated.timestamp.isAfter(initial.timestamp), isTrue);
    });

    test('withDiscoveryStatus updates discovery status and device count', () {
      final initial = FacadeState.initial();
      final updated = initial.withDiscoveryStatus(
        DiscoveryStatus.completed,
        deviceCount: 3,
      );

      expect(updated.discovery, DiscoveryStatus.completed);
      expect(updated.discoveredDeviceCount, 3);
      expect(updated.timestamp.isAfter(initial.timestamp), isTrue);
    });

    test('canStartEngine returns true only when ready and valid', () {
      final state = FacadeState.initial().copyWith(
        engine: EngineStatus.ready,
        validation: ValidationStatus.valid,
      );

      expect(state.canStartEngine, isTrue);

      // Not ready
      expect(
        state.copyWith(engine: EngineStatus.uninitialized).canStartEngine,
        isFalse,
      );

      // Not valid
      expect(
        state.copyWith(validation: ValidationStatus.none).canStartEngine,
        isFalse,
      );
    });

    test('isEngineRunning returns true only when running', () {
      expect(
        FacadeState.initial()
            .copyWith(engine: EngineStatus.running)
            .isEngineRunning,
        isTrue,
      );
      expect(
        FacadeState.initial()
            .copyWith(engine: EngineStatus.ready)
            .isEngineRunning,
        isFalse,
      );
    });

    test('isDeviceReady returns true only when connected', () {
      expect(
        FacadeState.initial()
            .copyWith(device: DeviceStatus.connected)
            .isDeviceReady,
        isTrue,
      );
      expect(
        FacadeState.initial()
            .copyWith(device: DeviceStatus.selected)
            .isDeviceReady,
        isFalse,
      );
    });

    test('isDiscovering returns true when starting or active', () {
      expect(
        FacadeState.initial()
            .copyWith(discovery: DiscoveryStatus.starting)
            .isDiscovering,
        isTrue,
      );
      expect(
        FacadeState.initial()
            .copyWith(discovery: DiscoveryStatus.active)
            .isDiscovering,
        isTrue,
      );
      expect(
        FacadeState.initial()
            .copyWith(discovery: DiscoveryStatus.idle)
            .isDiscovering,
        isFalse,
      );
    });

    test('hasError returns true when any subsystem has error', () {
      expect(
        FacadeState.initial().copyWith(engine: EngineStatus.error).hasError,
        isTrue,
      );
      expect(
        FacadeState.initial().copyWith(device: DeviceStatus.error).hasError,
        isTrue,
      );
      expect(
        FacadeState.initial()
            .copyWith(discovery: DiscoveryStatus.error)
            .hasError,
        isTrue,
      );
      expect(FacadeState.initial().hasError, isFalse);
    });
  });

  group('EngineStatus', () {
    test('has all expected statuses', () {
      expect(EngineStatus.values, [
        EngineStatus.uninitialized,
        EngineStatus.initializing,
        EngineStatus.ready,
        EngineStatus.loading,
        EngineStatus.running,
        EngineStatus.stopping,
        EngineStatus.paused,
        EngineStatus.error,
      ]);
    });
  });

  group('DeviceStatus', () {
    test('has all expected statuses', () {
      expect(DeviceStatus.values, [
        DeviceStatus.none,
        DeviceStatus.scanning,
        DeviceStatus.available,
        DeviceStatus.selected,
        DeviceStatus.connected,
        DeviceStatus.error,
        DeviceStatus.disconnected,
      ]);
    });
  });

  group('ValidationStatus', () {
    test('has all expected statuses', () {
      expect(ValidationStatus.values, [
        ValidationStatus.none,
        ValidationStatus.validating,
        ValidationStatus.valid,
        ValidationStatus.invalid,
        ValidationStatus.validWithWarnings,
      ]);
    });
  });

  group('DiscoveryStatus', () {
    test('has all expected statuses', () {
      expect(DiscoveryStatus.values, [
        DiscoveryStatus.idle,
        DiscoveryStatus.starting,
        DiscoveryStatus.active,
        DiscoveryStatus.completing,
        DiscoveryStatus.completed,
        DiscoveryStatus.cancelled,
        DiscoveryStatus.error,
      ]);
    });
  });
}
