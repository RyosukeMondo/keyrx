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

typedef KeyrxSetBypassNative = Void Function(Bool active);
typedef KeyrxSetBypass = void Function(bool active);

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

  late final KeyrxSetBypass? setBypass;
  late final KeyrxFreeString freeString;

  KeyrxBindings(this._lib) : super(_lib) {
    // Required functions
    freeString = _lib.lookupFunction<KeyrxFreeStringNative, KeyrxFreeString>(
      'keyrx_free_string',
    );

    setBypass = _tryLookupSetBypass();
    startLoop = _tryLookupStartLoop();
    stopLoop = _tryLookupStopLoop();
  }

  KeyrxSetBypass? _tryLookupSetBypass() {
    try {
      return _lib.lookupFunction<KeyrxSetBypassNative, KeyrxSetBypass>(
        'keyrx_set_bypass',
      );
    } on ArgumentError {
      return null;
    }
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
}
