/// FFI bindings for KeyRx Core.
///
/// This file extends the auto-generated bindings and adds backward-compatible
/// legacy callback support. For new code, use the generated bindings directly.
library;

import 'dart:ffi';

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

  @override
  late final KeyrxFreeString freeString;
  @override
  late final KeyrxGetConfigRoot getConfigRoot;

  KeyrxBindings(this._lib) : super(_lib) {
    // Required functions
    freeString = _lib.lookupFunction<KeyrxFreeStringNative, KeyrxFreeString>(
      'keyrx_free_string',
    );
    getConfigRoot = _lib
        .lookupFunction<KeyrxGetConfigRootNative, KeyrxGetConfigRoot>(
          'keyrx_get_config_root',
        );

    startLoop = _tryLookupStartLoop();
    stopLoop = _tryLookupStopLoop();
    registerEventCallback = _lib
        .lookupFunction<
          KeyrxRegisterEventCallbackNative,
          KeyrxRegisterEventCallback
        >('keyrx_register_event_callback');
  }

  @override
  late final KeyrxRegisterEventCallback registerEventCallback;

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
}
