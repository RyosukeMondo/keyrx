/// High-level bridge to KeyRx Core.
///
/// Provides a Dart-friendly API over the raw FFI bindings.
/// This file composes all bridge functionality from modular mixins.
library;

import 'dart:async';
import 'dart:ffi';
import 'dart:typed_data';

import 'bindings.dart';
import 'bridge_core.dart';
import 'bridge_device_profile.dart';
import 'bridge_discovery.dart';
import 'bridge_engine.dart';
import 'bridge_session.dart';
import 'bridge_testing.dart';
import 'bridge_validation.dart';
import 'device_registry_ffi.dart';
import 'event_types.dart';
import 'generated/bindings_generated.dart';

// Re-export all types from mixin modules for public API compatibility.
export 'bridge_device_profile.dart'
    show DeviceProfile, DeviceProfileResult, PhysicalKey, ProfileSource;
export 'bridge_discovery.dart'
    show DeviceListResult, DiscoveryStartResult, KeyboardDevice;
export 'event_types.dart' show EventType;
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
/// - [BridgeSessionMixin]: Session recording, analysis, replay
/// - [BridgeDeviceProfileMixin]: Device profile access
/// - [BridgeDiscoveryMixin]: Device listing and discovery
/// - [BridgeTestingMixin]: Testing, simulation, benchmarks, diagnostics
/// - [BridgeValidationMixin]: Script validation and key suggestions
/// - [DeviceRegistryFFIMixin]: Device registry for revolutionary mapping
class KeyrxBridge
    with
        BridgeCoreMixin,
        BridgeEngineMixin,
        BridgeSessionMixin,
        BridgeDeviceProfileMixin,
        BridgeDiscoveryMixin,
        BridgeTestingMixin,
        BridgeValidationMixin,
        DeviceRegistryFFIMixin {
  static KeyrxBridge? _currentInstance;

  KeyrxBindings? _bindings;
  bool _initialized = false;
  Object? _loadFailure;
  StreamController<BridgeState>? _stateController;

  // Unified event callback handlers
  final Map<EventType, void Function(Uint8List)> _eventHandlers = {};

  KeyrxBridge._({KeyrxBindings? bindings, Object? loadFailure})
      : _bindings = bindings,
        _loadFailure = loadFailure {
    if (_bindings != null) {
      _setupStateStream();
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
  StreamController<BridgeState>? get stateController => _stateController;

  /// Close any native resources and stop dispatching callbacks.
  Future<void> dispose() async {
    _currentInstance = null;
    await _stateController?.close();
    _stateController = null;

    // Unregister all unified event handlers
    for (final eventType in _eventHandlers.keys.toList()) {
      unregisterEventCallback(eventType);
    }
  }

  /// Register a unified event callback for a specific event type.
  ///
  /// This is the new unified API that replaces domain-specific callback
  /// registration functions. The handler receives the raw JSON payload
  /// as bytes.
  ///
  /// Returns true if registration succeeded, false otherwise.
  bool registerEventCallback(
    EventType eventType,
    void Function(Uint8List jsonPayload) handler,
  ) {
    if (_bindings == null) return false;

    // Store the handler
    _eventHandlers[eventType] = handler;

    // Register the callback with the native code
    // All event types use the same static handler
    final callback = Pointer.fromFunction<EventCallbackNative>(
      _handleUnifiedEvent,
    );

    final result = _bindings!.registerEventCallback(
      eventType.code,
      callback,
    );

    return result == 0;
  }

  /// Unregister a unified event callback for a specific event type.
  ///
  /// Returns true if unregistration succeeded, false otherwise.
  bool unregisterEventCallback(EventType eventType) {
    if (_bindings == null) return false;

    // Remove the handler
    _eventHandlers.remove(eventType);

    // Unregister with the native code (pass nullptr)
    final result = _bindings!.registerEventCallback(
      eventType.code,
      nullptr,
    );

    return result == 0;
  }

  /// Check if an event callback is registered.
  bool isEventCallbackRegistered(EventType eventType) {
    return _eventHandlers.containsKey(eventType);
  }

  void _setupStateStream() {
    if (_bindings?.onState == null) {
      return;
    }

    _stateController ??= StreamController<BridgeState>.broadcast();
    _bindings!.onState!(
      Pointer.fromFunction<KeyrxStateCallbackNative>(
        _handleState,
      ),
    );
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

  /// Handle unified event callbacks from the native core.
  ///
  /// This static method is called by the Rust core when an event occurs.
  /// It forwards the event to the appropriate Dart handler based on
  /// event type information in the JSON payload.
  ///
  /// Note: Since this is a static callback, we rely on _currentInstance
  /// to access the event handlers map.
  static void _handleUnifiedEvent(Pointer<Uint8> ptr, int length) {
    final instance = _currentInstance;
    if (instance == null) {
      return;
    }

    try {
      final bytes = ptr.asTypedList(length);

      // The Rust core sends events in the format:
      // {"eventType": <code>, "payload": {...}}
      // We need to parse the eventType to route to the correct handler.
      //
      // For now, we'll invoke all registered handlers with the full payload.
      // This is not ideal but maintains functionality. A better approach
      // would be to parse the JSON to extract the event type code.
      //
      // TODO: Parse event type from JSON and route to specific handler

      // For now, just forward to all handlers (temporary implementation)
      for (final handler in instance._eventHandlers.values) {
        try {
          handler(bytes);
        } catch (_) {
          // Swallow handler errors to avoid crashing other handlers
        }
      }
    } catch (_) {
      // Swallow malformed payloads to avoid crashing listeners.
    }
  }
}
