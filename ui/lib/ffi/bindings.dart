/// Auto-generated FFI bindings for KeyRx Core.
///
/// This file should be regenerated when the Rust API changes.
/// Use `ffigen` or manual definition based on core/src/ffi/exports.rs
library;

import 'dart:ffi';
import 'package:ffi/ffi.dart';

/// Native function signatures
typedef KeyrxInitNative = Int32 Function();
typedef KeyrxInit = int Function();

typedef KeyrxVersionNative = Pointer<Char> Function();
typedef KeyrxVersion = Pointer<Char> Function();

typedef KeyrxLoadScriptNative = Int32 Function(Pointer<Utf8> path);
typedef KeyrxLoadScript = int Function(Pointer<Utf8> path);

typedef KeyrxFreeStringNative = Void Function(Pointer<Char> ptr);
typedef KeyrxFreeString = void Function(Pointer<Char> ptr);

typedef KeyrxStartAudioNative = Int32 Function(Int32 bpm);
typedef KeyrxStartAudio = int Function(int bpm);

typedef KeyrxStopAudioNative = Int32 Function();
typedef KeyrxStopAudio = int Function();

typedef KeyrxSetBpmNative = Int32 Function(Int32 bpm);
typedef KeyrxSetBpm = int Function(int bpm);

typedef KeyrxEvalNative = Pointer<Char> Function(Pointer<Utf8> command);
typedef KeyrxEval = Pointer<Char> Function(Pointer<Utf8> command);

typedef KeyrxListKeysNative = Pointer<Char> Function();
typedef KeyrxListKeys = Pointer<Char> Function();

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

/// FFI bindings class
class KeyrxBindings {
  final DynamicLibrary _lib;

  late final KeyrxInit init;
  late final KeyrxVersion version;
  late final KeyrxLoadScript loadScript;
  late final KeyrxFreeString freeString;
  late final KeyrxStartAudio? startAudio;
  late final KeyrxStopAudio? stopAudio;
  late final KeyrxSetBpm? setBpm;
  late final KeyrxOnClassification? onClassification;
  late final KeyrxEval? eval;
  late final KeyrxOnState? onState;
  late final KeyrxListKeys? listKeys;

  KeyrxBindings(this._lib) {
    init = _lib.lookupFunction<KeyrxInitNative, KeyrxInit>('keyrx_init');
    version =
        _lib.lookupFunction<KeyrxVersionNative, KeyrxVersion>('keyrx_version');
    loadScript = _lib.lookupFunction<KeyrxLoadScriptNative, KeyrxLoadScript>(
        'keyrx_load_script');
    freeString = _lib.lookupFunction<KeyrxFreeStringNative, KeyrxFreeString>(
        'keyrx_free_string');
    startAudio = _tryLookupStartAudio();
    stopAudio = _tryLookupStopAudio();
    setBpm = _tryLookupSetBpm();
    onClassification = _tryLookupOnClassification();
    eval = _tryLookupEval();
    onState = _tryLookupOnState();
    listKeys = _tryLookupListKeys();
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

  KeyrxOnClassification? _tryLookupOnClassification() {
    try {
      return _lib.lookupFunction<KeyrxOnClassificationNative,
          KeyrxOnClassification>('keyrx_on_classification');
    } on ArgumentError {
      return null;
    }
  }

  KeyrxEval? _tryLookupEval() {
    try {
      return _lib.lookupFunction<KeyrxEvalNative, KeyrxEval>('keyrx_eval');
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

  KeyrxListKeys? _tryLookupListKeys() {
    try {
      return _lib.lookupFunction<KeyrxListKeysNative, KeyrxListKeys>(
        'keyrx_list_keys',
      );
    } on ArgumentError {
      return null;
    }
  }
}
