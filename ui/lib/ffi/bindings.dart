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

typedef KeyrxSetBypassNative = Void Function(Bool active);
typedef KeyrxSetBypass = void Function(bool active);

typedef KeyrxFreeStringNative = Void Function(Pointer<Char> ptr);
typedef KeyrxFreeString = void Function(Pointer<Char> ptr);

// ────────────────────────────────────────────────────────────────
// Device Registry FFI Types (Revolutionary Mapping)
// ────────────────────────────────────────────────────────────────

typedef KeyrxDeviceRegistryListDevicesNative = Pointer<Char> Function();
typedef KeyrxDeviceRegistryListDevices = Pointer<Char> Function();

typedef KeyrxDeviceRegistrySetRemapEnabledNative = Pointer<Char> Function(
    Pointer<Char>, Int32);
typedef KeyrxDeviceRegistrySetRemapEnabled = Pointer<Char> Function(
    Pointer<Char>, int);

typedef KeyrxDeviceRegistryAssignProfileNative = Pointer<Char> Function(
    Pointer<Char>, Pointer<Char>);
typedef KeyrxDeviceRegistryAssignProfile = Pointer<Char> Function(
    Pointer<Char>, Pointer<Char>);

typedef KeyrxDeviceRegistrySetUserLabelNative = Pointer<Char> Function(
    Pointer<Char>, Pointer<Char>);
typedef KeyrxDeviceRegistrySetUserLabel = Pointer<Char> Function(
    Pointer<Char>, Pointer<Char>);

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
  late final KeyrxOnState? onState;

  late final KeyrxSetBypass? setBypass;

  // Memory management (required)
  late final KeyrxFreeString freeString;

  // Device Registry FFI functions (revolutionary mapping)
  late final KeyrxDeviceRegistryListDevices? deviceRegistryListDevices;
  late final KeyrxDeviceRegistrySetRemapEnabled? deviceRegistrySetRemapEnabled;
  late final KeyrxDeviceRegistryAssignProfile? deviceRegistryAssignProfile;
  late final KeyrxDeviceRegistrySetUserLabel? deviceRegistrySetUserLabel;

  KeyrxBindings(this._lib) : super(_lib) {
    // Required functions
    freeString = _lib.lookupFunction<KeyrxFreeStringNative, KeyrxFreeString>(
        'keyrx_free_string');

    // Optional legacy callbacks
    onState = _tryLookupOnState();

    setBypass = _tryLookupSetBypass();

    // Revolutionary mapping device registry functions
    deviceRegistryListDevices = _tryLookupDeviceRegistryListDevices();
    deviceRegistrySetRemapEnabled = _tryLookupDeviceRegistrySetRemapEnabled();
    deviceRegistryAssignProfile = _tryLookupDeviceRegistryAssignProfile();
    deviceRegistrySetUserLabel = _tryLookupDeviceRegistrySetUserLabel();
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

  KeyrxOnState? _tryLookupOnState() {
    try {
      return _lib.lookupFunction<KeyrxOnStateNative, KeyrxOnState>(
        'keyrx_on_state',
      );
    } on ArgumentError {
      return null;
    }
  }

  KeyrxDeviceRegistryListDevices? _tryLookupDeviceRegistryListDevices() {
    try {
      return _lib.lookupFunction<KeyrxDeviceRegistryListDevicesNative,
          KeyrxDeviceRegistryListDevices>(
        'keyrx_device_registry_list_devices',
      );
    } on ArgumentError {
      return null;
    }
  }

  KeyrxDeviceRegistrySetRemapEnabled?
      _tryLookupDeviceRegistrySetRemapEnabled() {
    try {
      return _lib.lookupFunction<KeyrxDeviceRegistrySetRemapEnabledNative,
          KeyrxDeviceRegistrySetRemapEnabled>(
        'keyrx_device_registry_set_remap_enabled',
      );
    } on ArgumentError {
      return null;
    }
  }

  KeyrxDeviceRegistryAssignProfile? _tryLookupDeviceRegistryAssignProfile() {
    try {
      return _lib.lookupFunction<KeyrxDeviceRegistryAssignProfileNative,
          KeyrxDeviceRegistryAssignProfile>(
        'keyrx_device_registry_assign_profile',
      );
    } on ArgumentError {
      return null;
    }
  }

  KeyrxDeviceRegistrySetUserLabel? _tryLookupDeviceRegistrySetUserLabel() {
    try {
      return _lib.lookupFunction<KeyrxDeviceRegistrySetUserLabelNative,
          KeyrxDeviceRegistrySetUserLabel>(
        'keyrx_device_registry_set_user_label',
      );
    } on ArgumentError {
      return null;
    }
  }
}
