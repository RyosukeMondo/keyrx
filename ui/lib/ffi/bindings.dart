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

  late final KeyrxSetBypass? setBypass;

  // Memory management (required)
  late final KeyrxFreeString freeString;

  // Protocol version (optional but recommended)
  late final KeyrxProtocolVersion? protocolVersion;

  // Config root override (optional)
  late final KeyrxSetConfigRoot? setConfigRoot;

  KeyrxBindings(this._lib) : super(_lib) {
    // Required functions
    freeString = _lib.lookupFunction<KeyrxFreeStringNative, KeyrxFreeString>(
      'keyrx_free_string',
    );

    setBypass = _tryLookupSetBypass();
    protocolVersion = _tryLookupProtocolVersion();
    setConfigRoot = _tryLookupSetConfigRoot();
  }

  KeyrxSetConfigRoot? _tryLookupSetConfigRoot() {
    try {
      return _lib.lookupFunction<KeyrxSetConfigRootNative, KeyrxSetConfigRoot>(
        'keyrx_set_config_root',
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

  KeyrxProtocolVersion? _tryLookupProtocolVersion() {
    try {
      return _lib
          .lookupFunction<KeyrxProtocolVersionNative, KeyrxProtocolVersion>(
            'keyrx_protocol_version',
          );
    } on ArgumentError {
      return null;
    }
  }
}
