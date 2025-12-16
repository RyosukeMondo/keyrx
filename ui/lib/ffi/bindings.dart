/// FFI bindings for KeyRx Core.
///
/// This file extends the auto-generated bindings and adds backward-compatible
/// legacy callback support. For new code, use the generated bindings directly.
library;

import 'dart:ffi';
import 'package:ffi/ffi.dart';

import 'generated/bindings_generated.dart';

// ────────────────────────────────────────────────────────────────
// Legacy Callback Types (Backward Compatibility)
// ────────────────────────────────────────────────────────────────

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

typedef DeviceRegistrySetDeviceEnabledNative = Pointer<Char> Function(
  Pointer<Char> deviceKey,
  Int32 enabled,
  Pointer<Pointer<Utf8>> error,
);
typedef DeviceRegistrySetDeviceEnabled = Pointer<Char> Function(
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
class KeyrxBindings extends KeyrxBindingsGenerated {
  final DynamicLibrary _lib;

  late final KeyrxEngineStartLoop? startLoop;
  late final KeyrxEngineStopLoop? stopLoop;
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

  KeyrxBindings(this._lib) : super(_lib) {
    startLoop = _tryLookupStartLoop();
    stopLoop = _tryLookupStopLoop();
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

  DeviceRegistrySetDeviceEnabled? _tryLookupDeviceRegistrySetDeviceEnabled() {
    try {
      return _lib
          .lookupFunction<DeviceRegistrySetDeviceEnabledNative, DeviceRegistrySetDeviceEnabled>(
            'keyrx_device_registry_set_device_enabled',
          );
    } on ArgumentError {
      return null;
    }
  }
}
