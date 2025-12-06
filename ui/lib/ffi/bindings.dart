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

// Revolutionary runtime lifecycle
typedef KeyrxRevolutionaryRuntimeInitNative = Int32 Function();
typedef KeyrxRevolutionaryRuntimeInit = int Function();

typedef KeyrxRevolutionaryRuntimeShutdownNative = Int32 Function();
typedef KeyrxRevolutionaryRuntimeShutdown = int Function();

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
// Profile Registry FFI Types (Revolutionary Mapping)
// ────────────────────────────────────────────────────────────────

typedef KeyrxProfileRegistryListProfilesNative = Pointer<Char> Function();
typedef KeyrxProfileRegistryListProfiles = Pointer<Char> Function();

typedef KeyrxProfileRegistryGetProfileNative = Pointer<Char> Function(
    Pointer<Char>);
typedef KeyrxProfileRegistryGetProfile = Pointer<Char> Function(Pointer<Char>);

typedef KeyrxProfileRegistrySaveProfileNative = Pointer<Char> Function(
    Pointer<Char>);
typedef KeyrxProfileRegistrySaveProfile = Pointer<Char> Function(
    Pointer<Char>);

typedef KeyrxProfileRegistryDeleteProfileNative = Pointer<Char> Function(
    Pointer<Char>);
typedef KeyrxProfileRegistryDeleteProfile = Pointer<Char> Function(
    Pointer<Char>);

typedef KeyrxProfileRegistryFindCompatibleProfilesNative = Pointer<Char>
    Function(Pointer<Char>);
typedef KeyrxProfileRegistryFindCompatibleProfiles = Pointer<Char> Function(
    Pointer<Char>);

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

  // Revolutionary runtime lifecycle (optional; may not exist in older builds)
  late final KeyrxRevolutionaryRuntimeInit? revolutionaryRuntimeInit;
  late final KeyrxRevolutionaryRuntimeShutdown? revolutionaryRuntimeShutdown;

  // Device Registry FFI functions (revolutionary mapping)
  late final KeyrxDeviceRegistryListDevices? deviceRegistryListDevices;
  late final KeyrxDeviceRegistrySetRemapEnabled? deviceRegistrySetRemapEnabled;
  late final KeyrxDeviceRegistryAssignProfile? deviceRegistryAssignProfile;
  late final KeyrxDeviceRegistrySetUserLabel? deviceRegistrySetUserLabel;

  // Profile Registry FFI functions (revolutionary mapping)
  late final KeyrxProfileRegistryListProfiles? profileRegistryListProfiles;
  late final KeyrxProfileRegistryGetProfile? profileRegistryGetProfile;
  late final KeyrxProfileRegistrySaveProfile? profileRegistrySaveProfile;
  late final KeyrxProfileRegistryDeleteProfile? profileRegistryDeleteProfile;
  late final KeyrxProfileRegistryFindCompatibleProfiles?
      profileRegistryFindCompatibleProfiles;

  KeyrxBindings(this._lib) : super(_lib) {
    // Required functions
    freeString = _lib.lookupFunction<KeyrxFreeStringNative, KeyrxFreeString>(
        'keyrx_free_string');

    revolutionaryRuntimeInit = _tryLookupRuntimeInit();
    revolutionaryRuntimeShutdown = _tryLookupRuntimeShutdown();

    // Optional legacy callbacks
    onState = _tryLookupOnState();

    setBypass = _tryLookupSetBypass();

    // Revolutionary mapping device registry functions
    deviceRegistryListDevices = _tryLookupDeviceRegistryListDevices();
    deviceRegistrySetRemapEnabled = _tryLookupDeviceRegistrySetRemapEnabled();
    deviceRegistryAssignProfile = _tryLookupDeviceRegistryAssignProfile();
    deviceRegistrySetUserLabel = _tryLookupDeviceRegistrySetUserLabel();

    // Revolutionary mapping profile registry functions
    profileRegistryListProfiles = _tryLookupProfileRegistryListProfiles();
    profileRegistryGetProfile = _tryLookupProfileRegistryGetProfile();
    profileRegistrySaveProfile = _tryLookupProfileRegistrySaveProfile();
    profileRegistryDeleteProfile = _tryLookupProfileRegistryDeleteProfile();
    profileRegistryFindCompatibleProfiles =
        _tryLookupProfileRegistryFindCompatibleProfiles();
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

  KeyrxRevolutionaryRuntimeInit? _tryLookupRuntimeInit() {
    try {
      return _lib.lookupFunction<KeyrxRevolutionaryRuntimeInitNative,
          KeyrxRevolutionaryRuntimeInit>(
        'keyrx_revolutionary_runtime_init',
      );
    } on ArgumentError {
      return null;
    }
  }

  KeyrxRevolutionaryRuntimeShutdown? _tryLookupRuntimeShutdown() {
    try {
      return _lib.lookupFunction<KeyrxRevolutionaryRuntimeShutdownNative,
          KeyrxRevolutionaryRuntimeShutdown>(
        'keyrx_revolutionary_runtime_shutdown',
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

  KeyrxProfileRegistryListProfiles? _tryLookupProfileRegistryListProfiles() {
    try {
      return _lib.lookupFunction<KeyrxProfileRegistryListProfilesNative,
          KeyrxProfileRegistryListProfiles>(
        'keyrx_profile_registry_list_profiles',
      );
    } on ArgumentError {
      return null;
    }
  }

  KeyrxProfileRegistryGetProfile? _tryLookupProfileRegistryGetProfile() {
    try {
      return _lib.lookupFunction<KeyrxProfileRegistryGetProfileNative,
          KeyrxProfileRegistryGetProfile>(
        'keyrx_profile_registry_get_profile',
      );
    } on ArgumentError {
      return null;
    }
  }

  KeyrxProfileRegistrySaveProfile? _tryLookupProfileRegistrySaveProfile() {
    try {
      return _lib.lookupFunction<KeyrxProfileRegistrySaveProfileNative,
          KeyrxProfileRegistrySaveProfile>(
        'keyrx_profile_registry_save_profile',
      );
    } on ArgumentError {
      return null;
    }
  }

  KeyrxProfileRegistryDeleteProfile? _tryLookupProfileRegistryDeleteProfile() {
    try {
      return _lib.lookupFunction<KeyrxProfileRegistryDeleteProfileNative,
          KeyrxProfileRegistryDeleteProfile>(
        'keyrx_profile_registry_delete_profile',
      );
    } on ArgumentError {
      return null;
    }
  }

  KeyrxProfileRegistryFindCompatibleProfiles?
      _tryLookupProfileRegistryFindCompatibleProfiles() {
    try {
      return _lib.lookupFunction<
          KeyrxProfileRegistryFindCompatibleProfilesNative,
          KeyrxProfileRegistryFindCompatibleProfiles>(
        'keyrx_profile_registry_find_compatible_profiles',
      );
    } on ArgumentError {
      return null;
    }
  }
}
