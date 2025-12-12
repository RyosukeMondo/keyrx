/// High-level bridge to KeyRx Core.
///
/// Provides a Dart-friendly API over the raw FFI bindings.
/// This file composes all bridge functionality from modular mixins.
library;

import 'dart:async';
import 'dart:convert';
import 'dart:ffi';
import 'dart:typed_data';

import 'package:flutter/foundation.dart'; // For @visibleForTesting

import 'bindings.dart';
import 'bridge_core.dart';
import 'bridge_device_profile.dart';
import 'bridge_discovery.dart';
import 'bridge_engine.dart';
import 'bridge_migration.dart';
import 'bridge_session.dart';
import 'bridge_testing.dart';
import 'bridge_validation.dart';

import 'device_registry_ffi.dart';
import 'profile_registry_ffi.dart';
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
        BridgeStateUpdate,
        BridgeStateDelta,
        BridgeDeltaChange,
        BridgeDeltaChangeType,
        BridgeSnapshot,
        BridgeLayoutSnapshot,
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
/// - [ProfileRegistryFFIMixin]: Profile registry for revolutionary mapping
/// - [BridgeMigrationMixin]: Migration from V1 to V2 profiles
class KeyrxBridge
    with
        BridgeCoreMixin,
        BridgeEngineMixin,
        BridgeSessionMixin,
        BridgeDeviceProfileMixin,
        BridgeDiscoveryMixin,
        BridgeTestingMixin,
        BridgeValidationMixin,
        DeviceRegistryFFIMixin,
        ProfileRegistryFFIMixin,
        BridgeMigrationMixin {
  static const int expectedProtocolVersion = 1;

  static KeyrxBridge? _currentInstance;

  KeyrxBindings? _bindings;
  bool _initialized = false;
  Object? _loadFailure;
  StreamController<BridgeStateUpdate>? _stateController;

  // Unified event callback handlers
  final Map<EventType, void Function(Uint8List)> _eventHandlers = {};

  /// Keep NativeCallables alive for the duration of the registration.
  final Map<EventType, NativeCallable> _nativeCallables = {};
  NativeCallable<EventCallbackNative>? _stateCallable;

  // Manual binding for memory management
  late final FreeEventPayload? _freeEventPayload;

  KeyrxBridge._({
    KeyrxBindings? bindings,
    Object? loadFailure,
    DynamicLibrary? lib,
  }) : _bindings = bindings,
       _loadFailure = loadFailure {
    if (lib != null) {
      try {
        _freeEventPayload = lib
            .lookupFunction<FreeEventPayloadNative, FreeEventPayload>(
              'keyrx_free_event_payload',
            );
      } catch (_) {
        // Function might be missing in older binaries
        _freeEventPayload = null;
      }
    } else {
      _freeEventPayload = null;
    }

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
      final bindings = KeyrxBindings(lib);
      final bridge = KeyrxBridge._(bindings: bindings, lib: lib);

      // Perform handshake first to ensure binary compatibility
      bridge._checkProtocolVersion();

      return bridge;
    } catch (e) {
      return KeyrxBridge._(loadFailure: e);
    }
  }

  void _checkProtocolVersion() {
    if (_bindings == null) return;

    final versionFn = _bindings!.protocolVersion;
    final version = versionFn();
    if (version != expectedProtocolVersion) {
      _loadFailure ??= Exception(
        'Native library protocol mismatch. '
        'UI expects v$expectedProtocolVersion, Core is v$version. '
        'Please run "flutter clean" and rebuild.',
      );
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
  StreamController<BridgeStateUpdate>? get stateController => _stateController;

  /// Close any native resources and stop dispatching callbacks.
  Future<void> dispose() async {
    _currentInstance = null;
    await _stateController?.close();
    _stateController = null;

    _stateCallable?.close();
    _stateCallable = null;

    // Unregister all unified event handlers
    for (final eventType in _eventHandlers.keys.toList()) {
      unregisterEventCallback(eventType);
    }

    // Ensure all callables are closed (just in case)
    for (final callable in _nativeCallables.values) {
      callable.close();
    }
    _nativeCallables.clear();

    // Best-effort shutdown of the revolutionary runtime when available.
    if (_bindings != null) {
      _bindings!.revolutionaryRuntimeShutdown();
    }
  }

  @override
  bool initialize() {
    final result = super.initialize();
    if (result) {
      _initRevolutionaryRuntime();
    }
    return result;
  }

  /// Shutdown the revolutionary runtime (stop the engine).
  void shutdown() {
    _initialized = false;
    if (_bindings != null) {
      _bindings!.revolutionaryRuntimeShutdown();
    }
  }

  /// Start the engine loop (input capturing).
  bool startEngineLoop() {
    final startFn = bindings!.startLoop;
    if (startFn == null) {
      _loadFailure = 'startLoop function not found in shared library';
      return false;
    }

    final result = startFn();
    return result == 0;
  }

  /// Stop the engine loop (input capturing).
  bool stopEngineLoop() {
    final stopFn = bindings!.stopLoop;
    if (stopFn == null) {
      _loadFailure = 'stopLoop function not found in shared library';
      return false;
    }

    final result = stopFn();
    return result == 0;
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

    // Close existing callable for this type if any
    _nativeCallables[eventType]?.close();

    // Create a new NativeCallable.listener which allows the callback to be
    // invoked from any thread (e.g. background Rust threads).
    final nativeCallable = NativeCallable<EventCallbackNative>.listener(
      _handleUnifiedEvent,
    );
    _nativeCallables[eventType] = nativeCallable;

    final result = _bindings!.registerEventCallback(
      eventType.code,
      nativeCallable.nativeFunction,
    );

    return result == 0;
  }

  /// Returns true if unregistration succeeded, false otherwise.
  bool unregisterEventCallback(EventType eventType) {
    if (_bindings == null) return false;

    // Remove the handler
    _eventHandlers.remove(eventType);

    // Close the callable
    _nativeCallables[eventType]?.close();
    _nativeCallables.remove(eventType);

    // Unregister with the native code (pass nullptr)
    final result = _bindings!.registerEventCallback(eventType.code, nullptr);

    return result == 0;
  }

  /// Check if an event callback is registered.
  bool isEventCallbackRegistered(EventType eventType) {
    return _eventHandlers.containsKey(eventType);
  }

  void _setupStateStream() {
    // onState is a required binding now, but _bindings itself is nullable
    if (_bindings == null) {
      return;
    }

    _stateController ??= StreamController<BridgeStateUpdate>.broadcast();

    _stateCallable?.close();
    _stateCallable = NativeCallable<EventCallbackNative>.listener(_handleState);

    _bindings!.onState(_stateCallable!.nativeFunction);
  }

  /// Ask the native side to resend the next state update as a full snapshot.
  void requestFullStateResubscribe() {
    if (_bindings == null || _stateController?.isClosed == true) {
      return;
    }

    try {
      // We don't need to recreate the callable if it already exists,
      // but if we want to be robust we can just re-register the existing one.
      if (_stateCallable != null) {
        _bindings!.onState(_stateCallable!.nativeFunction);
      }
    } catch (_) {
      // Ignore resubscribe failures; consumer will retry on next event.
    }
  }

  void _initRevolutionaryRuntime() {
    if (_bindings == null) {
      return;
    }
    final code = _bindings!.revolutionaryRuntimeInit();
    // We ignore the return code here because it returns -1 if already initialized.
    // Since we call this blindly in initialize() as a recovery measure,
    // we don't want to overwrite `_loadFailure` if it was merely "already done".
    // Real initialization failures will surface when we try to use the runtime.
    if (code != 0 && _loadFailure == null) {
      // Optional: log this internally if we had a logger here.
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

  /// Helper to route events, extracted for testability.
  @visibleForTesting
  static void routeEvent(
    Uint8List bytes,
    Map<EventType, void Function(Uint8List)> handlers,
  ) {
    try {
      // Parse the JSON wrapper to extract event type
      final jsonString = utf8.decode(bytes);
      final eventWrapper = jsonDecode(jsonString) as Map<String, dynamic>;

      final typeCode = eventWrapper['eventType'] as int?;

      if (typeCode != null) {
        // Find the matching EventType enum
        EventType? targetType;
        for (final type in EventType.values) {
          if (type.code == typeCode) {
            targetType = type;
            break;
          }
        }

        if (targetType != null) {
          final handler = handlers[targetType];
          if (handler != null) {
            try {
              // Pass the original bytes to the handler to preserve the data contract
              // and avoid unnecessary re-serialization.
              handler(bytes);
            } catch (_) {
              // Swallow handler errors to avoid crashing
            }
          }
        }
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
      routeEvent(bytes, instance._eventHandlers);
    } catch (_) {
      // Swallow malformed payloads to avoid crashing listeners.
    } finally {
      // CRITICAL: Free the memory allocated by Rust.
      // Since EventRegistry::invoke leaks the buffer to ensure async safety,
      // we must explicitly free it here to avoid memory leaks.
      final free = instance._freeEventPayload;
      if (free != null && ptr != nullptr) {
        free(ptr, length);
      }
    }
  }
}

// Manual typedefs for keyrx_free_event_payload
typedef FreeEventPayloadNative = Void Function(Pointer<Uint8>, IntPtr);
typedef FreeEventPayload = void Function(Pointer<Uint8>, int);
