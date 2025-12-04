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

typedef KeyrxStateCallbackNative = Void Function(
  Pointer<Uint8> bytes,
  IntPtr length,
);
typedef KeyrxStateCallback = void Function(
  Pointer<Uint8> bytes,
  int length,
);

typedef KeyrxOnStateNative = Void Function(
  Pointer<NativeFunction<KeyrxStateCallbackNative>> callback,
);
typedef KeyrxOnState = void Function(
  Pointer<NativeFunction<KeyrxStateCallbackNative>> callback,
);

typedef KeyrxClassificationCallbackNative = Void Function(
  Pointer<Uint8> bytes,
  IntPtr length,
);
typedef KeyrxClassificationCallback = void Function(
  Pointer<Uint8> bytes,
  int length,
);

typedef KeyrxOnClassificationNative = Void Function(
  Pointer<NativeFunction<KeyrxClassificationCallbackNative>> callback,
);
typedef KeyrxOnClassification = void Function(
  Pointer<NativeFunction<KeyrxClassificationCallbackNative>> callback,
);

// ────────────────────────────────────────────────────────────────
// Legacy Audio Types (Not in generated bindings yet)
// ────────────────────────────────────────────────────────────────

typedef KeyrxStartAudioNative = Int32 Function(Int32 bpm);
typedef KeyrxStartAudio = int Function(int bpm);

typedef KeyrxStopAudioNative = Int32 Function();
typedef KeyrxStopAudio = int Function();

typedef KeyrxSetBpmNative = Int32 Function(Int32 bpm);
typedef KeyrxSetBpm = int Function(int bpm);

typedef KeyrxSetBypassNative = Void Function(Bool active);
typedef KeyrxSetBypass = void Function(bool active);

typedef KeyrxFreeStringNative = Void Function(Pointer<Char> ptr);
typedef KeyrxFreeString = void Function(Pointer<Char> ptr);

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

  // Legacy callback functions (optional)
  late final KeyrxOnClassification? onClassification;
  late final KeyrxOnState? onState;

  // Audio functions (optional, not yet in generated bindings)
  late final KeyrxStartAudio? startAudio;
  late final KeyrxStopAudio? stopAudio;
  late final KeyrxSetBpm? setBpm;
  late final KeyrxSetBypass? setBypass;

  // Memory management (required)
  late final KeyrxFreeString freeString;

  KeyrxBindings(this._lib) : super(_lib) {
    // Required functions
    freeString = _lib.lookupFunction<KeyrxFreeStringNative, KeyrxFreeString>(
        'keyrx_free_string');

    // Optional legacy callbacks
    onClassification = _tryLookupOnClassification();
    onState = _tryLookupOnState();

    // Optional audio functions
    startAudio = _tryLookupStartAudio();
    stopAudio = _tryLookupStopAudio();
    setBpm = _tryLookupSetBpm();
    setBypass = _tryLookupSetBypass();
  }

  KeyrxStartAudio? _tryLookupStartAudio() {
    try {
      return _lib.lookupFunction<KeyrxStartAudioNative, KeyrxStartAudio>(
        'keyrx_start_audio',
      );
    } on ArgumentError {
      return null;
    }
  }

  KeyrxStopAudio? _tryLookupStopAudio() {
    try {
      return _lib.lookupFunction<KeyrxStopAudioNative, KeyrxStopAudio>(
        'keyrx_stop_audio',
      );
    } on ArgumentError {
      return null;
    }
  }

  KeyrxSetBpm? _tryLookupSetBpm() {
    try {
      return _lib.lookupFunction<KeyrxSetBpmNative, KeyrxSetBpm>(
        'keyrx_set_bpm',
      );
    } on ArgumentError {
      return null;
    }
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

  KeyrxOnClassification? _tryLookupOnClassification() {
    try {
      return _lib.lookupFunction<KeyrxOnClassificationNative,
          KeyrxOnClassification>('keyrx_on_classification');
    } on ArgumentError {
      return null;
    }
  }

  KeyrxOnState? _tryLookupOnState() {
    try {
      return _lib.lookupFunction<KeyrxOnStateNative, KeyrxOnState>(
        'keyrx_on_state',
      );
    } on ArgumentError {
      return null;
    }
  }
}
