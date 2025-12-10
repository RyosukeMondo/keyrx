/// Device profile FFI methods.
///
/// Provides device profile access for viewing row-column mappings.
library;

import 'dart:convert';
import 'dart:ffi';

import 'package:ffi/ffi.dart';
import 'package:flutter/foundation.dart';

import 'bindings.dart';

/// Physical key metadata from device profile.
class PhysicalKey {
  const PhysicalKey({
    required this.scanCode,
    required this.row,
    required this.col,
    this.alias,
  });

  factory PhysicalKey.fromJson(Map<String, dynamic> json) {
    return PhysicalKey(
      scanCode: (json['scan_code'] as num).toInt(),
      row: (json['row'] as num).toInt(),
      col: (json['col'] as num).toInt(),
      alias: json['alias']?.toString(),
    );
  }

  final int scanCode;
  final int row;
  final int col;
  final String? alias;

  Map<String, dynamic> toJson() => {
    'scan_code': scanCode,
    'row': row,
    'col': col,
    if (alias != null) 'alias': alias,
  };
}

/// Profile source enum.
enum ProfileSource {
  discovered('Discovered'),
  defaultSource('Default'),
  migrated('Migrated');

  const ProfileSource(this.label);
  final String label;

  factory ProfileSource.fromString(String source) {
    return switch (source.toLowerCase()) {
      'discovered' => ProfileSource.discovered,
      'default' => ProfileSource.defaultSource,
      'migrated' => ProfileSource.migrated,
      _ => ProfileSource.defaultSource,
    };
  }
}

/// Device profile containing layout and keymap information.
class DeviceProfile {
  const DeviceProfile({
    required this.schemaVersion,
    required this.vendorId,
    required this.productId,
    this.name,
    required this.discoveredAt,
    required this.rows,
    required this.colsPerRow,
    required this.keymap,
    required this.aliases,
    required this.source,
  });

  factory DeviceProfile.fromJson(Map<String, dynamic> json) {
    final keymapJson = json['keymap'] as Map<String, dynamic>? ?? {};
    final keymap = <int, PhysicalKey>{};
    for (final entry in keymapJson.entries) {
      final scanCode = int.parse(entry.key);
      keymap[scanCode] = PhysicalKey.fromJson(
        entry.value as Map<String, dynamic>,
      );
    }

    final aliasesJson = json['aliases'] as Map<String, dynamic>? ?? {};
    final aliases = <String, int>{};
    for (final entry in aliasesJson.entries) {
      aliases[entry.key] = (entry.value as num).toInt();
    }

    final colsPerRowJson = json['cols_per_row'] as List? ?? [];
    final colsPerRow = colsPerRowJson.map((e) => (e as num).toInt()).toList();

    return DeviceProfile(
      schemaVersion: (json['schema_version'] as num?)?.toInt() ?? 1,
      vendorId: (json['vendor_id'] as num).toInt(),
      productId: (json['product_id'] as num).toInt(),
      name: json['name']?.toString(),
      discoveredAt: DateTime.parse(json['discovered_at'] as String),
      rows: (json['rows'] as num).toInt(),
      colsPerRow: colsPerRow,
      keymap: keymap,
      aliases: aliases,
      source: ProfileSource.fromString(json['source']?.toString() ?? 'Default'),
    );
  }

  final int schemaVersion;
  final int vendorId;
  final int productId;
  final String? name;
  final DateTime discoveredAt;
  final int rows;
  final List<int> colsPerRow;
  final Map<int, PhysicalKey> keymap;
  final Map<String, int> aliases;
  final ProfileSource source;

  /// Get device ID in hex format (vendor:product).
  String get deviceId =>
      '${vendorId.toRadixString(16).padLeft(4, '0')}:${productId.toRadixString(16).padLeft(4, '0')}';

  /// Get total number of keys in the layout.
  int get totalKeys => colsPerRow.fold(0, (sum, cols) => sum + cols);

  Map<String, dynamic> toJson() => {
    'schema_version': schemaVersion,
    'vendor_id': vendorId,
    'product_id': productId,
    if (name != null) 'name': name,
    'discovered_at': discoveredAt.toIso8601String(),
    'rows': rows,
    'cols_per_row': colsPerRow,
    'keymap': keymap.map(
      (key, value) => MapEntry(key.toString(), value.toJson()),
    ),
    'aliases': aliases,
    'source': source.label,
  };
}

/// Result of getting a device profile.
class DeviceProfileResult {
  const DeviceProfileResult({this.profile, this.errorMessage});

  factory DeviceProfileResult.success(DeviceProfile profile) =>
      DeviceProfileResult(profile: profile);

  factory DeviceProfileResult.error(String message) =>
      DeviceProfileResult(errorMessage: message);

  factory DeviceProfileResult.parse(String raw) {
    final trimmed = raw.trim();
    if (trimmed.toLowerCase().startsWith('error:')) {
      return DeviceProfileResult.error(
        trimmed.substring('error:'.length).trim(),
      );
    }

    final payload = trimmed.toLowerCase().startsWith('ok:')
        ? trimmed.substring(trimmed.indexOf(':') + 1)
        : trimmed;

    try {
      final decoded = json.decode(payload);
      if (decoded is! Map<String, dynamic>) {
        return DeviceProfileResult.error('invalid device profile payload');
      }

      final profile = DeviceProfile.fromJson(decoded);
      return DeviceProfileResult.success(profile);
    } catch (e) {
      return DeviceProfileResult.error('$e');
    }
  }

  final DeviceProfile? profile;
  final String? errorMessage;

  bool get hasError => errorMessage != null;
  bool get isSuccess => profile != null && !hasError;
}

/// Mixin providing device profile FFI methods.
mixin BridgeDeviceProfileMixin {
  KeyrxBindings? get bindings;
  Object? get loadFailure;

  /// Get device profile for a specific device.
  ///
  /// Returns the complete device profile including keymap and layout.
  DeviceProfileResult getDeviceProfile(int vendorId, int productId) {
    final getFn = bindings?.getDeviceProfile;
    if (getFn == null) {
      return DeviceProfileResult.error('getDeviceProfile not available');
    }

    Pointer<Char>? ptr;
    try {
      ptr = getFn(vendorId, productId);
      if (ptr == nullptr) {
        return DeviceProfileResult.error('getDeviceProfile returned null');
      }

      final raw = ptr.cast<Utf8>().toDartString();
      return DeviceProfileResult.parse(raw);
    } catch (e) {
      return DeviceProfileResult.error('$e');
    } finally {
      if (ptr != null && ptr != nullptr) {
        try {
          bindings?.freeString(ptr);
        } catch (_) {}
      }
    }
  }

  /// Check if a device profile exists.
  ///
  /// Returns true if a profile has been created for this device.
  bool hasDeviceProfile(int vendorId, int productId) {
    final hasFn = bindings?.hasDeviceProfile;
    if (hasFn == null) {
      return false;
    }

    Pointer<Char>? ptr;
    try {
      ptr = hasFn(vendorId, productId);
      if (ptr == nullptr) {
        return false;
      }

      final raw = ptr.cast<Utf8>().toDartString();
      final trimmed = raw.trim();

      // Parse "ok:true" or "ok:false" response
      if (trimmed.toLowerCase().startsWith('ok:')) {
        final payload = trimmed.substring(trimmed.indexOf(':') + 1).trim();
        return payload.toLowerCase() == 'true';
      }

      return false;
    } catch (e) {
      return false;
    } finally {
      if (ptr != null && ptr != nullptr) {
        try {
          bindings?.freeString(ptr);
        } catch (_) {}
      }
    }
  }

  /// Save a device profile to disk.
  ///
  /// [profileJson] - JSON representation of the DeviceProfile.
  ///
  /// Returns true if successful.
  bool saveDeviceProfile(String profileJson) {
    final saveFn = bindings?.saveDeviceProfile;
    if (saveFn == null) return false;

    final jsonPtr = profileJson.toNativeUtf8();
    Pointer<Char>? ptr;
    try {
      ptr = saveFn(jsonPtr.cast<Char>());
      if (ptr == nullptr) return false;

      final raw = ptr.cast<Utf8>().toDartString();
      final trimmed = raw.trim();

      // Parse "ok:" response (or empty ok for void result)
      if (trimmed.toLowerCase().startsWith('ok:')) {
        return true;
      }
      debugPrint('saveDeviceProfile failed. Rust returned: $trimmed');
      return false;
    } catch (e) {
      debugPrint('saveDeviceProfile exception: $e');
      return false;
    } finally {
      calloc.free(jsonPtr);
      if (ptr != null && ptr != nullptr) {
        try {
          bindings?.freeString(ptr);
        } catch (_) {}
      }
    }
  }
}
