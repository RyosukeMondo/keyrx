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

/// FFI bindings class
class KeyrxBindings {
  final DynamicLibrary _lib;

  late final KeyrxInit init;
  late final KeyrxVersion version;
  late final KeyrxLoadScript loadScript;
  late final KeyrxFreeString freeString;

  KeyrxBindings(this._lib) {
    init = _lib.lookupFunction<KeyrxInitNative, KeyrxInit>('keyrx_init');
    version =
        _lib.lookupFunction<KeyrxVersionNative, KeyrxVersion>('keyrx_version');
    loadScript = _lib.lookupFunction<KeyrxLoadScriptNative, KeyrxLoadScript>(
        'keyrx_load_script');
    freeString = _lib.lookupFunction<KeyrxFreeStringNative, KeyrxFreeString>(
        'keyrx_free_string');
  }
}
