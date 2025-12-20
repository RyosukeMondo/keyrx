/// Pure data models extracted from FFI bridge files.
///
/// These models contain no FFI dependencies and can be used on all platforms
/// including web. They represent the data structures returned by the FFI bridge
/// but are defined separately to enable platform-agnostic code.
library;

import 'dart:convert';

// ============================================================================
// Device Discovery Models (from bridge_discovery.dart)
// ============================================================================

/// A discovered keyboard device.
class KeyboardDevice {
  const KeyboardDevice({
    required this.name,
    required this.vendorId,
    required this.productId,
    required this.path,
    required this.hasProfile,
  });

  factory KeyboardDevice.fromJson(Map<String, dynamic> json) => KeyboardDevice(
    name: json['name'] as String? ?? 'Unknown',
    vendorId: (json['vendorId'] ?? json['vendor_id'] as num?)?.toInt() ?? 0,
    productId: (json['productId'] ?? json['product_id'] as num?)?.toInt() ?? 0,
    path: json['path'] as String? ?? '',
    hasProfile: (json['hasProfile'] ?? json['has_profile']) as bool? ?? false,
  );

  final String name;
  final int vendorId;
  final int productId;
  final String path;
  final bool hasProfile;

  String get deviceId =>
      '${vendorId.toRadixString(16).padLeft(4, '0')}:${productId.toRadixString(16).padLeft(4, '0')}';

  Map<String, dynamic> toJson() => {
    'name': name,
    'vendor_id': vendorId,
    'product_id': productId,
    'path': path,
    'has_profile': hasProfile,
  };

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is KeyboardDevice &&
          runtimeType == other.runtimeType &&
          vendorId == other.vendorId &&
          productId == other.productId &&
          path == other.path;

  @override
  int get hashCode => Object.hash(vendorId, productId, path);

  @override
  String toString() => 'KeyboardDevice($name, $deviceId, hasProfile: $hasProfile)';
}

/// Result of device listing operation.
class DeviceListResult {
  const DeviceListResult({required this.devices, this.errorMessage});

  factory DeviceListResult.success(List<KeyboardDevice> devices) =>
      DeviceListResult(devices: devices);

  factory DeviceListResult.error(String message) =>
      DeviceListResult(devices: const [], errorMessage: message);

  /// Parse FFI response string into DeviceListResult.
  /// Used by FFI bridge implementations to parse raw responses.
  factory DeviceListResult.parse(String raw) {
    final trimmed = raw.trim();
    if (trimmed.toLowerCase().startsWith('error:')) {
      return DeviceListResult.error(trimmed.substring('error:'.length).trim());
    }

    final payload = trimmed.toLowerCase().startsWith('ok:')
        ? trimmed.substring(trimmed.indexOf(':') + 1)
        : trimmed;

    try {
      final decoded = json.decode(payload);
      if (decoded is! List) {
        return DeviceListResult.error('invalid device list payload');
      }

      final devices = decoded
          .map((entry) {
            if (entry is! Map<String, dynamic>) {
              return null;
            }
            return KeyboardDevice.fromJson(entry);
          })
          .whereType<KeyboardDevice>()
          .toList();

      return DeviceListResult(devices: devices);
    } catch (e) {
      return DeviceListResult.error('$e');
    }
  }

  final List<KeyboardDevice> devices;
  final String? errorMessage;

  bool get hasError => errorMessage != null;
  bool get isSuccess => !hasError;
}

/// Result of starting device discovery.
class DiscoveryStartResult {
  const DiscoveryStartResult({required this.success, this.totalKeys, this.errorMessage});

  factory DiscoveryStartResult.ok({int? totalKeys}) =>
      DiscoveryStartResult(success: true, totalKeys: totalKeys);

  factory DiscoveryStartResult.error(String message) =>
      DiscoveryStartResult(success: false, errorMessage: message);

  /// Parse FFI response string into DiscoveryStartResult.
  /// Used by FFI bridge implementations to parse raw responses.
  factory DiscoveryStartResult.parse(String raw) {
    final trimmed = raw.trim();
    if (trimmed.toLowerCase().startsWith('error:')) {
      return DiscoveryStartResult.error(trimmed.substring('error:'.length).trim());
    }

    final payload = trimmed.toLowerCase().startsWith('ok:')
        ? trimmed.substring(trimmed.indexOf(':') + 1)
        : trimmed;

    try {
      final decoded = json.decode(payload);
      if (decoded is! Map<String, dynamic>) {
        return DiscoveryStartResult.error('invalid discovery payload');
      }

      return DiscoveryStartResult(
        success: decoded['success'] as bool? ?? false,
        totalKeys: (decoded['totalKeys'] as num?)?.toInt(),
        errorMessage: decoded['error']?.toString(),
      );
    } catch (e) {
      return DiscoveryStartResult.error('$e');
    }
  }

  final bool success;
  final int? totalKeys;
  final String? errorMessage;

  bool get hasError => errorMessage != null || !success;
}

// ============================================================================
// Device Profile Models (from bridge_device_profile.dart)
// ============================================================================

/// Physical key metadata from device profile.
class PhysicalKey {
  const PhysicalKey({required this.scanCode, required this.row, required this.col, this.alias});

  factory PhysicalKey.fromJson(Map<String, dynamic> json) => PhysicalKey(
    scanCode: (json['scan_code'] as num).toInt(),
    row: (json['row'] as num).toInt(),
    col: (json['col'] as num).toInt(),
    alias: json['alias']?.toString(),
  );

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

  factory ProfileSource.fromString(String source) => switch (source.toLowerCase()) {
    'discovered' => ProfileSource.discovered,
    'default' => ProfileSource.defaultSource,
    'migrated' => ProfileSource.migrated,
    _ => ProfileSource.defaultSource,
  };
}

/// Device profile containing layout and keymap information.
class DeviceProfile {
  const DeviceProfile({
    required this.schemaVersion,
    required this.vendorId,
    required this.productId,
    required this.discoveredAt,
    required this.rows,
    required this.colsPerRow,
    required this.keymap,
    required this.aliases,
    required this.source,
    this.name,
  });

  factory DeviceProfile.fromJson(Map<String, dynamic> json) {
    final keymapJson = json['keymap'] as Map<String, dynamic>? ?? {};
    final keymap = <int, PhysicalKey>{};
    for (final entry in keymapJson.entries) {
      final scanCode = int.parse(entry.key);
      keymap[scanCode] = PhysicalKey.fromJson(entry.value as Map<String, dynamic>);
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
    'keymap': keymap.map((key, value) => MapEntry(key.toString(), value.toJson())),
    'aliases': aliases,
    'source': source.label,
  };
}

/// Result of getting a device profile.
class DeviceProfileResult {
  const DeviceProfileResult({this.profile, this.errorMessage});

  factory DeviceProfileResult.success(DeviceProfile profile) =>
      DeviceProfileResult(profile: profile);

  factory DeviceProfileResult.error(String message) => DeviceProfileResult(errorMessage: message);

  factory DeviceProfileResult.parse(String raw) {
    final trimmed = raw.trim();
    if (trimmed.toLowerCase().startsWith('error:')) {
      return DeviceProfileResult.error(trimmed.substring('error:'.length).trim());
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

// ============================================================================
// Device Registry Models (from device_registry_ffi.dart)
// ============================================================================

/// Result wrapper for device registry operations.
class DeviceRegistryResult<T> {
  const DeviceRegistryResult({this.data, this.errorMessage});

  factory DeviceRegistryResult.success(T data) => DeviceRegistryResult(data: data);

  factory DeviceRegistryResult.error(String message) => DeviceRegistryResult(errorMessage: message);

  final T? data;
  final String? errorMessage;

  bool get hasError => errorMessage != null;
  bool get isSuccess => !hasError;
}

// ============================================================================
// Engine State Models (from bridge_engine.dart)
// ============================================================================

/// State stream payload containing a delta and optional full snapshot.
class BridgeStateUpdate {
  const BridgeStateUpdate({
    required this.delta,
    this.fullSnapshot,
    this.event,
    this.latencyUs,
    this.timing,
  });

  final BridgeStateDelta delta;
  final BridgeSnapshot? fullSnapshot;
  final String? event;
  final int? latencyUs;
  final BridgeTiming? timing;
}

/// Incremental delta describing changes to engine state.
class BridgeStateDelta {
  const BridgeStateDelta({
    required this.fromVersion,
    required this.toVersion,
    required this.changes,
  });

  factory BridgeStateDelta.fromJson(Map<String, dynamic> json) {
    final rawChanges = json['changes'] as List<dynamic>? ?? const [];
    return BridgeStateDelta(
      fromVersion: (json['from_version'] as num?)?.toInt() ?? 0,
      toVersion: (json['to_version'] as num?)?.toInt() ?? 0,
      changes: rawChanges.map(BridgeDeltaChange.fromJson).whereType<BridgeDeltaChange>().toList(),
    );
  }

  final int fromVersion;
  final int toVersion;
  final List<BridgeDeltaChange> changes;
}

/// Types of delta changes emitted by the core.
enum BridgeDeltaChangeType {
  keyPressed,
  keyReleased,
  allKeysReleased,
  layerActivated,
  layerDeactivated,
  layerStackChanged,
  modifierChanged,
  allModifiersCleared,
  pendingAdded,
  pendingResolved,
  allPendingCleared,
  versionChanged,
}

/// One change entry inside a delta.
class BridgeDeltaChange {
  const BridgeDeltaChange(this.type, {this.payload});

  final BridgeDeltaChangeType type;
  final Object? payload;

  static BridgeDeltaChange? fromJson(dynamic raw) {
    if (raw is String) {
      switch (raw) {
        case 'AllKeysReleased':
          return const BridgeDeltaChange(BridgeDeltaChangeType.allKeysReleased);
        case 'AllModifiersCleared':
          return const BridgeDeltaChange(BridgeDeltaChangeType.allModifiersCleared);
        case 'AllPendingCleared':
          return const BridgeDeltaChange(BridgeDeltaChangeType.allPendingCleared);
        default:
          return null;
      }
    }

    if (raw is Map<String, dynamic> && raw.length == 1) {
      final entry = raw.entries.first;
      final key = entry.key;
      final value = entry.value;

      switch (key) {
        case 'KeyPressed':
          return BridgeDeltaChange(BridgeDeltaChangeType.keyPressed, payload: value);
        case 'KeyReleased':
          return BridgeDeltaChange(BridgeDeltaChangeType.keyReleased, payload: value);
        case 'LayerActivated':
          return BridgeDeltaChange(BridgeDeltaChangeType.layerActivated, payload: value);
        case 'LayerDeactivated':
          return BridgeDeltaChange(BridgeDeltaChangeType.layerDeactivated, payload: value);
        case 'LayerStackChanged':
          return BridgeDeltaChange(BridgeDeltaChangeType.layerStackChanged, payload: value);
        case 'ModifierChanged':
          return BridgeDeltaChange(BridgeDeltaChangeType.modifierChanged, payload: value);
        case 'PendingAdded':
          return BridgeDeltaChange(BridgeDeltaChangeType.pendingAdded, payload: value);
        case 'PendingResolved':
          return BridgeDeltaChange(BridgeDeltaChangeType.pendingResolved, payload: value);
        case 'VersionChanged':
          return BridgeDeltaChange(BridgeDeltaChangeType.versionChanged, payload: value);
        default:
          return null;
      }
    }

    return null;
  }
}

/// Full snapshot of current engine state.
class BridgeSnapshot {
  const BridgeSnapshot({
    required this.version,
    required this.pressedKeys,
    required this.activeLayers,
    required this.baseLayer,
    required this.standardModifierBits,
    required this.virtualModifiers,
    required this.pendingCount,
    this.layouts = const [],
    this.sharedModifiers = const [],
  });

  factory BridgeSnapshot.fromJson(Map<String, dynamic> json) {
    final pressed =
        (json['pressed_keys'] as List<dynamic>?)
            ?.map((e) {
              if (e is Map<String, dynamic>) {
                final key = e['key']?.toString() ?? '';
                return key;
              }
              return e.toString();
            })
            .where((k) => k.isNotEmpty)
            .toList() ??
        const <String>[];

    var standardBits = 0;
    final standardRaw = json['standard_modifiers'];
    if (standardRaw is Map<String, dynamic>) {
      if (standardRaw['bits'] is num) {
        standardBits = (standardRaw['bits'] as num).toInt();
      } else if (standardRaw.values.isNotEmpty && standardRaw.values.first is num) {
        standardBits = (standardRaw.values.first as num).toInt();
      }
    }

    var layouts = const <BridgeLayoutSnapshot>[];
    var sharedModifiers = const <int>[];
    final compositor = json['layout_compositor'];
    if (compositor is Map<String, dynamic>) {
      final rawLayouts = compositor['layouts'] as List<dynamic>? ?? const [];
      layouts = rawLayouts
          .map(
            (entry) => entry is Map<String, dynamic> ? BridgeLayoutSnapshot.fromJson(entry) : null,
          )
          .whereType<BridgeLayoutSnapshot>()
          .toList();
      sharedModifiers = (compositor['shared_modifiers'] as List<dynamic>? ?? const [])
          .map((e) => (e as num?)?.toInt())
          .whereType<int>()
          .toList();
    }

    return BridgeSnapshot(
      version: (json['version'] as num?)?.toInt() ?? 0,
      pressedKeys: pressed,
      activeLayers:
          (json['active_layers'] as List<dynamic>?)
              ?.map((e) => (e as num?)?.toInt() ?? 0)
              .toList() ??
          const <int>[],
      baseLayer: (json['base_layer'] as num?)?.toInt() ?? 0,
      standardModifierBits: standardBits,
      virtualModifiers:
          (json['virtual_modifiers'] as List<dynamic>?)
              ?.map((e) => (e as num?)?.toInt() ?? 0)
              .toList() ??
          const <int>[],
      pendingCount: (json['pending_count'] as num?)?.toInt() ?? 0,
      layouts: layouts,
      sharedModifiers: sharedModifiers,
    );
  }

  final int version;
  final List<String> pressedKeys;
  final List<int> activeLayers;
  final int baseLayer;
  final int standardModifierBits;
  final List<int> virtualModifiers;
  final int pendingCount;
  final List<BridgeLayoutSnapshot> layouts;
  final List<int> sharedModifiers;
}

/// Snapshot of a single layout's state.
class BridgeLayoutSnapshot {
  const BridgeLayoutSnapshot({
    required this.id,
    required this.name,
    required this.priority,
    required this.enabled,
    this.activeLayers = const [],
    this.description,
    this.tags = const [],
    this.modifiers = const [],
  });

  factory BridgeLayoutSnapshot.fromJson(Map<String, dynamic> json) => BridgeLayoutSnapshot(
    id: json['id']?.toString() ?? '',
    name: json['name']?.toString() ?? '',
    priority: (json['priority'] as num?)?.toInt() ?? 0,
    enabled: json['enabled'] == true,
    activeLayers: (json['active_layers'] as List<dynamic>? ?? const [])
        .map((e) => (e as num?)?.toInt() ?? 0)
        .toList(),
    description: json['description']?.toString(),
    tags: (json['tags'] as List<dynamic>? ?? const [])
        .map((e) => e.toString())
        .where((e) => e.isNotEmpty)
        .toList(),
    modifiers: (json['modifiers'] as List<dynamic>? ?? const [])
        .map((e) => (e as num?)?.toInt())
        .whereType<int>()
        .toList(),
  );

  final String id;
  final String name;
  final int priority;
  final bool enabled;
  final List<int> activeLayers;
  final String? description;
  final List<String> tags;
  final List<int> modifiers;
}

/// Timing configuration for tap/hold decisions.
class BridgeTiming {
  const BridgeTiming({
    this.tapTimeoutMs,
    this.comboTimeoutMs,
    this.holdDelayMs,
    this.eagerTap,
    this.permissiveHold,
    this.retroTap,
  });

  final int? tapTimeoutMs;
  final int? comboTimeoutMs;
  final int? holdDelayMs;
  final bool? eagerTap;
  final bool? permissiveHold;
  final bool? retroTap;
}

/// Key registry entry.
class KeyRegistryEntry {
  const KeyRegistryEntry({required this.name, this.aliases = const [], this.evdev, this.vk});

  final String name;
  final List<String> aliases;
  final int? evdev;
  final int? vk;
}

/// Result of fetching the key registry.
class KeyRegistryResult {
  const KeyRegistryResult({required this.entries, this.error, this.usedFallback = false});

  factory KeyRegistryResult.fallback(String errorMessage) {
    return KeyRegistryResult(entries: const [], error: errorMessage, usedFallback: true);
  }

  factory KeyRegistryResult.parse(String raw) {
    final trimmed = raw.trim();
    if (trimmed.toLowerCase().startsWith('error:')) {
      return KeyRegistryResult.fallback(trimmed);
    }

    final payload = trimmed.toLowerCase().startsWith('ok:')
        ? trimmed.substring(trimmed.indexOf(':') + 1)
        : trimmed;

    try {
      final decoded = json.decode(payload);
      if (decoded is! List) {
        return KeyRegistryResult.fallback('error: invalid registry payload');
      }

      final entries = decoded
          .map((entry) {
            if (entry is! Map) {
              return const KeyRegistryEntry(name: '');
            }
            final name = entry['name']?.toString() ?? '';
            final aliases =
                (entry['aliases'] as List<dynamic>?)?.map((e) => e.toString()).toList() ??
                const <String>[];
            final evdev = (entry['evdev'] as num?)?.toInt();
            final vk = (entry['vk'] as num?)?.toInt();
            return KeyRegistryEntry(name: name, aliases: aliases, evdev: evdev, vk: vk);
          })
          .where((entry) => entry.name.isNotEmpty)
          .toList();

      if (entries.isEmpty) {
        return KeyRegistryResult.fallback('error: empty registry payload');
      }

      return KeyRegistryResult(entries: entries);
    } catch (e) {
      return KeyRegistryResult.fallback('error: $e');
    }
  }

  final List<KeyRegistryEntry> entries;
  final String? error;
  final bool usedFallback;

  bool get hasError => error != null;

  List<String> get names => entries.map((e) => e.name).toList();
}

/// Script validation error.
class ScriptValidationError {
  const ScriptValidationError({required this.message, this.line, this.column});

  final String message;
  final int? line;
  final int? column;
}

/// Result of script validation.
class ScriptValidationResult {
  const ScriptValidationResult({required this.valid, required this.errors, this.errorMessage});

  factory ScriptValidationResult.error(String message) =>
      ScriptValidationResult(valid: false, errors: const [], errorMessage: message);

  factory ScriptValidationResult.parse(String raw) {
    final trimmed = raw.trim();
    if (trimmed.toLowerCase().startsWith('error:')) {
      return ScriptValidationResult.error(trimmed.substring('error:'.length).trim());
    }

    final payload = trimmed.toLowerCase().startsWith('ok:')
        ? trimmed.substring(trimmed.indexOf(':') + 1)
        : trimmed;

    try {
      final decoded = json.decode(payload);
      if (decoded is! Map<String, dynamic>) {
        return ScriptValidationResult.error('invalid validation payload');
      }

      final valid = decoded['valid'] as bool? ?? false;
      final errorsList = decoded['errors'] as List<dynamic>? ?? [];
      final errors = errorsList.map((e) {
        if (e is! Map<String, dynamic>) {
          return const ScriptValidationError(message: 'unknown error');
        }
        return ScriptValidationError(
          line: (e['line'] as num?)?.toInt(),
          column: (e['column'] as num?)?.toInt(),
          message: e['message']?.toString() ?? 'unknown error',
        );
      }).toList();

      return ScriptValidationResult(valid: valid, errors: errors);
    } catch (e) {
      return ScriptValidationResult.error('$e');
    }
  }

  final bool valid;
  final List<ScriptValidationError> errors;
  final String? errorMessage;

  bool get hasError => !valid || errorMessage != null;
}
