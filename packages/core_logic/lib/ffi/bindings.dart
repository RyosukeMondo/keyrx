/// FFI bindings for KeyRx Core.
///
/// This file extends the auto-generated bindings and adds backward-compatible
/// legacy callback support. For new code, use the generated bindings directly.
library;

// ignore_for_file: non_constant_identifier_names, avoid_catching_errors, sort_constructors_first, lines_longer_than_80_chars

import 'dart:ffi';
import 'package:ffi/ffi.dart';
import 'generated_bindings.dart';

// ────────────────────────────────────────────────────────────────
// Legacy Callback Types (Backward Compatibility)
// ────────────────────────────────────────────────────────────────

typedef EventCallbackNative = Void Function(Pointer<Uint8>, IntPtr);
typedef EventCallback = void Function(Pointer<Uint8>, int);

typedef KeyrxFreeStringNative = Void Function(Pointer<Char> ptr);
typedef KeyrxFreeString = void Function(Pointer<Char> ptr);

typedef KeyrxProtocolVersionNative = Uint32 Function();
typedef KeyrxProtocolVersion = int Function();

typedef KeyrxSetConfigRootNative = Int32 Function(Pointer<Char> path);
typedef KeyrxSetConfigRoot = int Function(Pointer<Char> path);

typedef KeyrxEngineStartLoopNative = Int32 Function();
typedef KeyrxEngineStartLoop = int Function();

typedef KeyrxEngineStopLoopNative = Int32 Function();
typedef KeyrxEngineStopLoop = int Function();

typedef KeyrxOnStateNative =
    Void Function(Pointer<NativeFunction<EventCallbackNative>>);
typedef KeyrxOnState =
    void Function(Pointer<NativeFunction<EventCallbackNative>>);

typedef KeyrxRegisterEventCallbackNative =
    Int32 Function(Int32, Pointer<NativeFunction<EventCallbackNative>>);
typedef KeyrxRegisterEventCallback =
    int Function(int, Pointer<NativeFunction<EventCallbackNative>>);

typedef KeyrxGetConfigRootNative = Pointer<Char> Function();
typedef KeyrxGetConfigRoot = Pointer<Char> Function();

// ────────────────────────────────────────────────────────────────
// FFI Tracing Types
// ────────────────────────────────────────────────────────────────

typedef KeyrxTraceEnableNative = Void Function();
typedef KeyrxTraceEnable = void Function();

typedef KeyrxTraceDisableNative = Void Function();
typedef KeyrxTraceDisable = void Function();

typedef KeyrxTraceIsEnabledNative = Bool Function();
typedef KeyrxTraceIsEnabled = bool Function();

typedef KeyrxTraceGetCountNative = Size Function();
typedef KeyrxTraceGetCount = int Function();

typedef KeyrxTraceClearNative = Void Function();
typedef KeyrxTraceClear = void Function();

typedef KeyrxTraceGetEventsJsonNative = Pointer<Char> Function();
typedef KeyrxTraceGetEventsJson = Pointer<Char> Function();

typedef KeyrxTraceGetFilteredEventsJsonNative =
    Pointer<Char> Function(Pointer<Char> domain, Uint64 minDurationUs);
typedef KeyrxTraceGetFilteredEventsJson =
    Pointer<Char> Function(Pointer<Char> domain, int minDurationUs);

typedef KeyrxTraceFreeJsonNative = Void Function(Pointer<Char> json);
typedef KeyrxTraceFreeJson = void Function(Pointer<Char> json);

// ────────────────────────────────────────────────────────────────
// Device Registry Types (for missing methods)
// ────────────────────────────────────────────────────────────────

typedef DeviceRegistrySetDeviceEnabledNative =
    Pointer<Char> Function(
      Pointer<Char> deviceKey,
      Int32 enabled,
      Pointer<Pointer<Utf8>> error,
    );
typedef DeviceRegistrySetDeviceEnabled =
    Pointer<Char> Function(
      Pointer<Char> deviceKey,
      int enabled,
      Pointer<Pointer<Utf8>> error,
    );

// ────────────────────────────────────────────────────────────────
// Main Bindings Class
// ────────────────────────────────────────────────────────────────

/// FFI bindings class that extends auto-generated bindings.
///
/// This class provides:
/// - All auto-generated bindings from KeyrxBindingsGenerated
/// - Legacy callback support for backward compatibility
/// - Optional functions that may not exist in all builds
class KeyrxBindings {
  final DynamicLibrary _lib;

  late final BaseBindings base;
  late final CompatBindings compat;
  late final ConfigBindings config;
  late final DeviceDefinitionsBindings deviceDefinitions;
  late final DeviceRegistryBindings deviceRegistry;
  late final DiscoveryBindings discovery;
  late final EngineBindings engine;
  late final MarshalBindings marshal;
  late final MigrationBindings migration;
  late final ObservabilityBindings observability;
  late final ProfileRegistryBindings profileRegistry;
  late final RuntimeBindings runtime;
  late final TelemetryBindings telemetry;
  late final TracingBindings tracing;
  late final TransitionLogBindings transitionLog;

  late final KeyrxEngineStartLoop? startLoop;
  late final KeyrxEngineStopLoop? stopLoop;
  late final KeyrxOnState onState;
  late final KeyrxFreeString freeString;

  // FFI Tracing functions
  late final KeyrxTraceEnable keyrx_trace_enable;
  late final KeyrxTraceDisable keyrx_trace_disable;
  late final KeyrxTraceIsEnabled keyrx_trace_is_enabled;
  late final KeyrxTraceGetCount keyrx_trace_get_count;
  late final KeyrxTraceClear keyrx_trace_clear;
  late final KeyrxTraceGetEventsJson keyrx_trace_get_events_json;
  late final KeyrxTraceGetFilteredEventsJson
  keyrx_trace_get_filtered_events_json;
  late final KeyrxTraceFreeJson keyrx_trace_free_json;

  // Device Registry functions (only the ones missing from KeyrxBindingsGenerated)
  late final DeviceRegistrySetDeviceEnabled? deviceRegistrySetDeviceEnabled;

  KeyrxBindings(this._lib) {
    base = BaseBindings(_lib);
    compat = CompatBindings(_lib);
    config = ConfigBindings(_lib);
    deviceDefinitions = DeviceDefinitionsBindings(_lib);
    deviceRegistry = DeviceRegistryBindings(_lib);
    discovery = DiscoveryBindings(_lib);
    engine = EngineBindings(_lib);
    marshal = MarshalBindings(_lib);
    migration = MigrationBindings(_lib);
    observability = ObservabilityBindings(_lib);
    profileRegistry = ProfileRegistryBindings(_lib);
    runtime = RuntimeBindings(_lib);
    telemetry = TelemetryBindings(_lib);
    tracing = TracingBindings(_lib);
    transitionLog = TransitionLogBindings(_lib);

    startLoop = _tryLookupStartLoop();
    stopLoop = _tryLookupStopLoop();
    onState = _lookupOnState();
    freeString = _lib
        .lookup<NativeFunction<KeyrxFreeStringNative>>('keyrx_free_string')
        .asFunction<KeyrxFreeString>();

    // Initialize tracing functions
    keyrx_trace_enable = _lib
        .lookup<NativeFunction<KeyrxTraceEnableNative>>('keyrx_trace_enable')
        .asFunction<KeyrxTraceEnable>();
    keyrx_trace_disable = _lib
        .lookup<NativeFunction<KeyrxTraceDisableNative>>('keyrx_trace_disable')
        .asFunction<KeyrxTraceDisable>();
    keyrx_trace_is_enabled = _lib
        .lookup<NativeFunction<KeyrxTraceIsEnabledNative>>(
          'keyrx_trace_is_enabled',
        )
        .asFunction<KeyrxTraceIsEnabled>();
    keyrx_trace_get_count = _lib
        .lookup<NativeFunction<KeyrxTraceGetCountNative>>(
          'keyrx_trace_get_count',
        )
        .asFunction<KeyrxTraceGetCount>();
    keyrx_trace_clear = _lib
        .lookup<NativeFunction<KeyrxTraceClearNative>>('keyrx_trace_clear')
        .asFunction<KeyrxTraceClear>();
    keyrx_trace_get_events_json = _lib
        .lookup<NativeFunction<KeyrxTraceGetEventsJsonNative>>(
          'keyrx_trace_get_events_json',
        )
        .asFunction<KeyrxTraceGetEventsJson>();
    keyrx_trace_get_filtered_events_json = _lib
        .lookup<NativeFunction<KeyrxTraceGetFilteredEventsJsonNative>>(
          'keyrx_trace_get_filtered_events_json',
        )
        .asFunction<KeyrxTraceGetFilteredEventsJson>();
    keyrx_trace_free_json = _lib
        .lookup<NativeFunction<KeyrxTraceFreeJsonNative>>(
          'keyrx_trace_free_json',
        )
        .asFunction<KeyrxTraceFreeJson>();

    // Initialize device registry functions (only the ones missing from KeyrxBindingsGenerated)
    deviceRegistrySetDeviceEnabled = _tryLookupDeviceRegistrySetDeviceEnabled();
  }

  KeyrxEngineStartLoop? _tryLookupStartLoop() {
    try {
      return _lib
          .lookupFunction<KeyrxEngineStartLoopNative, KeyrxEngineStartLoop>(
            'keyrx_engine_start_loop',
          );
    } on ArgumentError {
      return null;
    }
  }

  KeyrxEngineStopLoop? _tryLookupStopLoop() {
    try {
      return _lib
          .lookupFunction<KeyrxEngineStopLoopNative, KeyrxEngineStopLoop>(
            'keyrx_engine_stop_loop',
          );
    } on ArgumentError {
      return null;
    }
  }

  KeyrxOnState _lookupOnState() {
    try {
      return _lib.lookupFunction<KeyrxOnStateNative, KeyrxOnState>(
        'keyrx_on_state',
      );
    } catch (_) {
      // Return a no-op if missing
      return (Pointer<NativeFunction<EventCallbackNative>> callback) {};
    }
  }

  DeviceRegistrySetDeviceEnabled? _tryLookupDeviceRegistrySetDeviceEnabled() {
    try {
      return _lib.lookupFunction<
        DeviceRegistrySetDeviceEnabledNative,
        DeviceRegistrySetDeviceEnabled
      >('keyrx_device_registry_set_device_enabled');
    } on ArgumentError {
      return null;
    }
  }
}
