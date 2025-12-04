/// Aggregated state model for the KeyRx facade.
///
/// Combines state from multiple services (engine, device, validation, discovery)
/// into a single unified state representation. This enables widgets to observe
/// all relevant application state through a single stream subscription.
library;

import 'package:freezed_annotation/freezed_annotation.dart';

part 'facade_state.freezed.dart';

/// Status of the engine subsystem.
enum EngineStatus {
  /// Engine is not initialized.
  uninitialized,

  /// Engine is initializing.
  initializing,

  /// Engine is ready but not running.
  ready,

  /// Script is being loaded.
  loading,

  /// Engine is running with a loaded script.
  running,

  /// Engine is stopping.
  stopping,

  /// Engine is paused (if pause functionality exists).
  paused,

  /// Engine encountered an error.
  error,
}

/// Status of device connection and selection.
enum DeviceStatus {
  /// No device selected or available.
  none,

  /// Scanning for available devices.
  scanning,

  /// Devices are available but none selected.
  available,

  /// A device is selected but not yet connected.
  selected,

  /// Device is connected and ready.
  connected,

  /// Device connection error.
  error,

  /// Device was disconnected.
  disconnected,
}

/// Status of script validation.
enum ValidationStatus {
  /// No script loaded or validation not performed.
  none,

  /// Validation is in progress.
  validating,

  /// Script is valid.
  valid,

  /// Script has validation errors.
  invalid,

  /// Script has validation warnings but is valid.
  validWithWarnings,
}

/// Status of device discovery operations.
enum DiscoveryStatus {
  /// No discovery operation in progress.
  idle,

  /// Discovery is starting.
  starting,

  /// Discovery is actively running.
  active,

  /// Discovery is completing.
  completing,

  /// Discovery completed successfully.
  completed,

  /// Discovery was cancelled.
  cancelled,

  /// Discovery encountered an error.
  error,
}

/// Aggregated state from all facade-managed services.
///
/// This combines state from engine, device, validation, and discovery
/// subsystems into a single observable state model. Widgets can subscribe
/// to state changes through the facade's state stream.
///
/// Example:
/// ```dart
/// facade.stateStream.listen((state) {
///   if (state.engine == EngineStatus.running) {
///     print('Engine is running');
///   }
///   if (state.device == DeviceStatus.connected) {
///     print('Device connected: ${state.selectedDevicePath}');
///   }
/// });
/// ```
@freezed
class FacadeState with _$FacadeState {
  const FacadeState._();

  const factory FacadeState({
    /// Current engine status.
    required EngineStatus engine,

    /// Current device connection status.
    required DeviceStatus device,

    /// Current script validation status.
    required ValidationStatus validation,

    /// Current device discovery status.
    required DiscoveryStatus discovery,

    /// Path to the currently loaded script, if any.
    String? scriptPath,

    /// Path to the selected/connected device, if any.
    String? selectedDevicePath,

    /// Number of validation errors, if validation was performed.
    int? validationErrorCount,

    /// Number of validation warnings, if validation was performed.
    int? validationWarningCount,

    /// Number of discovered devices during last discovery.
    int? discoveredDeviceCount,

    /// Last error message, if any operation failed.
    String? lastError,

    /// Timestamp of this state snapshot.
    required DateTime timestamp,
  }) = _FacadeState;

  /// Create the initial facade state.
  ///
  /// All subsystems start in their idle/uninitialized state.
  factory FacadeState.initial() => FacadeState(
        engine: EngineStatus.uninitialized,
        device: DeviceStatus.none,
        validation: ValidationStatus.none,
        discovery: DiscoveryStatus.idle,
        timestamp: DateTime.now(),
      );

  /// Check if the engine is in a state where it can be started.
  bool get canStartEngine =>
      engine == EngineStatus.ready && validation == ValidationStatus.valid;

  /// Check if the engine is running.
  bool get isEngineRunning => engine == EngineStatus.running;

  /// Check if a device is connected and ready.
  bool get isDeviceReady => device == DeviceStatus.connected;

  /// Check if discovery is currently active.
  bool get isDiscovering =>
      discovery == DiscoveryStatus.starting ||
      discovery == DiscoveryStatus.active;

  /// Check if any subsystem is in an error state.
  bool get hasError =>
      engine == EngineStatus.error ||
      device == DeviceStatus.error ||
      discovery == DiscoveryStatus.error;

  /// Create a copy with updated engine status.
  FacadeState withEngineStatus(
    EngineStatus status, {
    String? scriptPath,
    String? error,
  }) {
    return copyWith(
      engine: status,
      scriptPath: scriptPath ?? this.scriptPath,
      lastError: error,
      timestamp: DateTime.now(),
    );
  }

  /// Create a copy with updated device status.
  FacadeState withDeviceStatus(
    DeviceStatus status, {
    String? devicePath,
    String? error,
  }) {
    return copyWith(
      device: status,
      selectedDevicePath: devicePath ?? selectedDevicePath,
      lastError: error,
      timestamp: DateTime.now(),
    );
  }

  /// Create a copy with updated validation status.
  FacadeState withValidationStatus(
    ValidationStatus status, {
    int? errorCount,
    int? warningCount,
    String? error,
  }) {
    return copyWith(
      validation: status,
      validationErrorCount: errorCount,
      validationWarningCount: warningCount,
      lastError: error,
      timestamp: DateTime.now(),
    );
  }

  /// Create a copy with updated discovery status.
  FacadeState withDiscoveryStatus(
    DiscoveryStatus status, {
    int? deviceCount,
    String? error,
  }) {
    return copyWith(
      discovery: status,
      discoveredDeviceCount: deviceCount,
      lastError: error,
      timestamp: DateTime.now(),
    );
  }
}
