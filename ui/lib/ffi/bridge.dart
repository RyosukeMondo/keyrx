/// High-level bridge to KeyRx Core.
///
/// Provides a Dart-friendly API over the raw FFI bindings.
/// This file composes all bridge functionality from modular mixins.
library;

import 'dart:async';
import 'dart:ffi';

import 'bindings.dart';
import 'bridge_audio.dart';
import 'bridge_core.dart';
import 'bridge_discovery.dart';
import 'bridge_engine.dart';
import 'bridge_session.dart';
import 'bridge_testing.dart';
import 'bridge_validation.dart';

// Re-export all types from mixin modules for public API compatibility.
export 'bridge_audio.dart' show BridgeClassification;
export 'bridge_discovery.dart'
    show DeviceListResult, DiscoveryStartResult, KeyboardDevice;
export 'bridge_engine.dart'
    show
        BridgeState,
        BridgeTiming,
        KeyRegistryEntry,
        KeyRegistryResult,
        ScriptValidationError,
        ScriptValidationResult;
export 'bridge_session.dart'
    show
        DecisionBreakdown,
        RecordingStartResult,
        RecordingStopResult,
        ReplayMismatch,
        ReplayResult,
        SessionAnalysis,
        SessionAnalysisResult,
        SessionInfo,
        SessionListResult;
export 'bridge_testing.dart'
    show
        BenchmarkResult,
        DiagnosticCheck,
        DiscoveredTest,
        DoctorResult,
        KeyInput,
        SimulationMapping,
        SimulationResult,
        TestDiscoveryResult,
        TestResult,
        TestRunResult;
export 'bridge_validation.dart'
    show
        CoverageReport,
        LayerCoverage,
        SourceLocation,
        ValidationError,
        ValidationOptions,
        ValidationResult,
        ValidationWarning,
        WarningCategory;

/// Bridge to the KeyRx Core library.
///
/// Composes FFI functionality from modular mixins:
/// - [BridgeCoreMixin]: Initialization, version, disposal
/// - [BridgeEngineMixin]: Script loading, evaluation, key registry, bypass
/// - [BridgeAudioMixin]: Audio capture and classification streams
/// - [BridgeSessionMixin]: Session recording, analysis, replay
/// - [BridgeDiscoveryMixin]: Device listing and discovery
/// - [BridgeTestingMixin]: Testing, simulation, benchmarks, diagnostics
/// - [BridgeValidationMixin]: Script validation and key suggestions
class KeyrxBridge
    with
        BridgeCoreMixin,
        BridgeEngineMixin,
        BridgeAudioMixin,
        BridgeSessionMixin,
        BridgeDiscoveryMixin,
        BridgeTestingMixin,
        BridgeValidationMixin {
  static KeyrxBridge? _currentInstance;

  KeyrxBindings? _bindings;
  bool _initialized = false;
  Object? _loadFailure;
  StreamController<BridgeClassification>? _classificationController;
  StreamController<BridgeState>? _stateController;

  KeyrxBridge._({KeyrxBindings? bindings, Object? loadFailure})
      : _bindings = bindings,
        _loadFailure = loadFailure {
    if (_bindings != null) {
      _setupClassificationStream();
    }
    _currentInstance = this;
  }

  /// Create a new bridge instance.
  ///
  /// Gracefully handles missing native libraries so the UI can surface errors
  /// instead of crashing at startup.
  factory KeyrxBridge.open() {
    try {
      final lib = BridgeCoreMixin.loadLibrary();
      return KeyrxBridge._(bindings: KeyrxBindings(lib));
    } catch (e) {
      return KeyrxBridge._(loadFailure: e);
    }
  }

  // Mixin interface implementations.
  @override
  KeyrxBindings? get bindings => _bindings;

  @override
  bool get initialized => _initialized;

  @override
  set initialized(bool value) => _initialized = value;

  @override
  Object? get loadFailure => _loadFailure;

  @override
  StreamController<BridgeClassification>? get classificationController =>
      _classificationController;

  @override
  StreamController<BridgeState>? get stateController => _stateController;

  /// Close any native resources and stop dispatching callbacks.
  Future<void> dispose() async {
    _currentInstance = null;
    await _classificationController?.close();
    _classificationController = null;
    await _stateController?.close();
    _stateController = null;
  }

  void _setupClassificationStream() {
    if (_bindings?.onClassification == null) {
      return;
    }

    _classificationController ??=
        StreamController<BridgeClassification>.broadcast();

    _bindings!.onClassification!(
      Pointer.fromFunction<KeyrxClassificationCallbackNative>(
        _handleClassification,
      ),
    );

    if (_bindings?.onState != null) {
      _stateController ??= StreamController<BridgeState>.broadcast();
      _bindings!.onState!(
        Pointer.fromFunction<KeyrxStateCallbackNative>(
          _handleState,
        ),
      );
    }
  }

  static void _handleClassification(Pointer<Uint8> ptr, int length) {
    final instance = _currentInstance;
    final controller = instance?._classificationController;
    if (instance == null || controller == null || controller.isClosed) {
      return;
    }

    try {
      final bytes = ptr.asTypedList(length);
      final classification =
          BridgeAudioClassificationParser.parseClassificationPayload(bytes);
      if (classification != null) {
        controller.add(classification);
      }
    } catch (_) {
      // Swallow malformed payloads to avoid crashing listeners.
    }
  }

  static void _handleState(Pointer<Uint8> ptr, int length) {
    final instance = _currentInstance;
    final controller = instance?._stateController;
    if (instance == null || controller == null || controller.isClosed) {
      return;
    }

    try {
      final bytes = ptr.asTypedList(length);
      final state = BridgeEngineStateSetup.parseStatePayload(bytes);
      if (state != null) {
        controller.add(state);
      }
    } catch (_) {
      // Swallow malformed payloads to avoid crashing listeners.
    }
  }
}
