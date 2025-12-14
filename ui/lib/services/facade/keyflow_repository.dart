/// Repository that aggregates KeyFlow data from existing services.
library;

import 'dart:async';

import '../../ffi/bridge.dart';
import '../../models/device_state.dart';
import '../../models/hardware_profile.dart';
import '../../models/keymap.dart';
import '../../models/runtime_config.dart';
import '../../models/virtual_layout.dart';
import '../device_registry_service.dart';
import '../hardware_service.dart';
import '../keymap_service.dart';
import '../layout_service.dart';
import '../runtime_service.dart';
import '../keyflow_telemetry.dart';

/// Aggregated view of KeyFlow data for UI consumption.
class KeyFlowSnapshot {
  const KeyFlowSnapshot({
    required this.devices,
    required this.profiles,
    required this.layouts,
    required this.keymaps,
    required this.generatedAt,
    this.runtime,
    this.runtimeConfig,
    this.errors = const [],
  });

  final List<KeyFlowDevice> devices;
  final Map<String, HardwareProfile> profiles;
  final Map<String, VirtualLayout> layouts;
  final Map<String, Keymap> keymaps;
  final DateTime generatedAt;
  final RuntimeOverlay? runtime;
  final RuntimeConfig? runtimeConfig;
  final List<KeyFlowError> errors;

  bool get hasErrors => errors.isNotEmpty;
}

/// Per-device summary enriched with wiring stats.
class KeyFlowDevice {
  const KeyFlowDevice({
    required this.id,
    required this.displayName,
    required this.remapEnabled,
    this.profileId,
    this.layoutId,
    this.wiring = const {},
    this.gaps = 0,
    this.conflicts = 0,
    this.lastUpdated,
  });

  final String id;
  final String displayName;
  final bool remapEnabled;
  final String? profileId;
  final String? layoutId;
  final Map<int, String> wiring;
  final int gaps;
  final int conflicts;
  final String? lastUpdated;
}

/// Lightweight runtime overlay derived from bridge state events.
class RuntimeOverlay {
  const RuntimeOverlay({
    required this.activeLayers,
    required this.activeModifiers,
    required this.heldKeys,
    required this.pendingCount,
    required this.lastUpdate,
    this.lastEvent,
    this.latencyUs,
    this.version,
  });

  final List<int> activeLayers;
  final List<String> activeModifiers;
  final List<String> heldKeys;
  final int pendingCount;
  final DateTime lastUpdate;
  final String? lastEvent;
  final int? latencyUs;
  final int? version;

  RuntimeOverlay copyWith({
    List<int>? activeLayers,
    List<String>? activeModifiers,
    List<String>? heldKeys,
    int? pendingCount,
    DateTime? lastUpdate,
    String? lastEvent,
    int? latencyUs,
    int? version,
  }) {
    return RuntimeOverlay(
      activeLayers: activeLayers ?? this.activeLayers,
      activeModifiers: activeModifiers ?? this.activeModifiers,
      heldKeys: heldKeys ?? this.heldKeys,
      pendingCount: pendingCount ?? this.pendingCount,
      lastUpdate: lastUpdate ?? this.lastUpdate,
      lastEvent: lastEvent ?? this.lastEvent,
      latencyUs: latencyUs ?? this.latencyUs,
      version: version ?? this.version,
    );
  }

  static RuntimeOverlay initial() {
    return RuntimeOverlay(
      activeLayers: const [],
      activeModifiers: const [],
      heldKeys: const [],
      pendingCount: 0,
      lastUpdate: DateTime.fromMillisecondsSinceEpoch(0),
      version: null,
    );
  }
}

/// Structured error captured during aggregation.
class KeyFlowError {
  const KeyFlowError({required this.source, required this.message});

  final String source;
  final String message;
}

/// Facade that composes existing services into KeyFlow-ready DTOs.
class KeyFlowRepository {
  KeyFlowRepository({
    required DeviceRegistryService deviceRegistryService,
    required HardwareService hardwareService,
    required LayoutService layoutService,
    required KeymapService keymapService,
    required RuntimeService runtimeService,
    Stream<BridgeStateUpdate>? runtimeUpdates,
    KeyFlowTelemetry? telemetry,
  }) : _deviceRegistryService = deviceRegistryService,
       _hardwareService = hardwareService,
       _layoutService = layoutService,
       _keymapService = keymapService,
       _runtimeService = runtimeService,
       _runtimeUpdates = runtimeUpdates,
       _telemetry = telemetry {
    _wireRuntimeStream();
  }

  final DeviceRegistryService _deviceRegistryService;
  final HardwareService _hardwareService;
  final LayoutService _layoutService;
  final KeymapService _keymapService;
  final RuntimeService _runtimeService;
  final Stream<BridgeStateUpdate>? _runtimeUpdates;
  final KeyFlowTelemetry? _telemetry;

  final StreamController<RuntimeOverlay> _runtimeController =
      StreamController<RuntimeOverlay>.broadcast();
  StreamSubscription<BridgeStateUpdate>? _runtimeSub;
  RuntimeOverlay _runtimeOverlay = RuntimeOverlay.initial();

  Map<String, HardwareProfile> _cachedProfiles = const {};
  Map<String, VirtualLayout> _cachedLayouts = const {};
  Map<String, Keymap> _cachedKeymaps = const {};
  RuntimeConfig? _lastRuntimeConfig;

  /// Aggregate all KeyFlow data with partial failure isolation.
  Future<KeyFlowSnapshot> loadAll() async {
    final sw = Stopwatch()..start();
    var success = true;
    KeyFlowSnapshot? snapshot;
    final errors = <KeyFlowError>[];

    try {
      final devices = await _safeDevices(errors);
      await _refreshProfiles(errors);
      await _refreshLayouts(errors);
      await _refreshKeymaps(errors);
      await _refreshRuntimeConfig(errors);

      snapshot = _composeSnapshot(devices, errors: errors);
      return snapshot;
    } catch (_) {
      success = false;
      rethrow;
    } finally {
      _telemetry?.recordFetch(
        duration: sw.elapsed,
        success: success && (snapshot?.errors.isEmpty ?? true),
      );
      if (errors.isNotEmpty && snapshot != null) {
        _maybeRecordParseErrors(snapshot!, errors);
      }
    }
  }

  /// Refresh devices only (fast path) and reuse cached config data.
  Future<KeyFlowSnapshot> refreshDevices() async {
    final sw = Stopwatch()..start();
    var success = true;
    KeyFlowSnapshot? snapshot;
    if (_cachedProfiles.isEmpty ||
        _cachedLayouts.isEmpty ||
        _cachedKeymaps.isEmpty) {
      return loadAll();
    }

    final errors = <KeyFlowError>[];
    List<DeviceState> devices = const [];
    try {
      devices = await _deviceRegistryService.refresh();
    } catch (e) {
      errors.add(_error('devices', 'refresh failed: $e'));
    }

    if (devices.isEmpty) {
      // Fallback to cached get if refresh failed.
      devices = await _safeDevices(errors);
    }

    try {
      snapshot = _composeSnapshot(devices, errors: errors);
      return snapshot;
    } catch (_) {
      success = false;
      rethrow;
    } finally {
      _telemetry?.recordFetch(
        duration: sw.elapsed,
        success: success && (snapshot?.errors.isEmpty ?? true),
      );
      if (errors.isNotEmpty && snapshot != null) {
        _maybeRecordParseErrors(snapshot!, errors);
      }
    }
  }

  /// Stream runtime overlays without blocking UI consumers.
  Stream<RuntimeOverlay> runtimeStream() {
    return _runtimeController.stream;
  }

  /// Dispose internal subscriptions.
  Future<void> dispose() async {
    await _runtimeSub?.cancel();
    await _runtimeController.close();
  }

  Future<List<DeviceState>> _safeDevices(List<KeyFlowError> errors) async {
    try {
      return await _deviceRegistryService.getDevices();
    } catch (e) {
      errors.add(_error('devices', 'list failed: $e'));
      return const [];
    }
  }

  Future<void> _refreshProfiles(List<KeyFlowError> errors) async {
    try {
      final result = await _hardwareService.listProfiles();
      if (result.hasError || result.data == null) {
        errors.add(_error('hardware', result.errorMessage ?? 'unknown error'));
        _recordParseError(
          'profiles',
          result.errorMessage,
          payloadBytes: _estimateProfilesBytes(result.data),
        );
        return;
      }
      _cachedProfiles = {
        for (final profile in result.data!) profile.id: profile,
      };
    } catch (e) {
      errors.add(_error('hardware', 'list failed: $e'));
      _recordParseError(
        'profiles',
        e,
        payloadBytes: _estimateProfilesBytes(_cachedProfiles.values),
      );
    }
  }

  Future<void> _refreshLayouts(List<KeyFlowError> errors) async {
    try {
      final result = await _layoutService.listLayouts();
      if (result.hasError || result.data == null) {
        errors.add(_error('layouts', result.errorMessage ?? 'unknown error'));
        _recordParseError(
          'layouts',
          result.errorMessage,
          payloadBytes: _estimateLayoutBytes(result.data),
        );
        return;
      }
      _cachedLayouts = {for (final layout in result.data!) layout.id: layout};
    } catch (e) {
      errors.add(_error('layouts', 'list failed: $e'));
      _recordParseError(
        'layouts',
        e,
        payloadBytes: _estimateLayoutBytes(_cachedLayouts.values),
      );
    }
  }

  Future<void> _refreshKeymaps(List<KeyFlowError> errors) async {
    try {
      final result = await _keymapService.listKeymaps();
      if (result.hasError || result.data == null) {
        errors.add(_error('keymaps', result.errorMessage ?? 'unknown error'));
        _recordParseError(
          'keymaps',
          result.errorMessage,
          payloadBytes: _estimateKeymapBytes(result.data),
        );
        return;
      }
      _cachedKeymaps = {for (final keymap in result.data!) keymap.id: keymap};
    } catch (e) {
      errors.add(_error('keymaps', 'list failed: $e'));
      _recordParseError(
        'keymaps',
        e,
        payloadBytes: _estimateKeymapBytes(_cachedKeymaps.values),
      );
    }
  }

  Future<void> _refreshRuntimeConfig(List<KeyFlowError> errors) async {
    try {
      final result = await _runtimeService.getConfig();
      if (result.hasError) {
        errors.add(_error('runtime', result.errorMessage ?? 'unknown error'));
        _recordParseError(
          'runtime',
          result.errorMessage,
          payloadBytes: _estimateRuntimeBytes(result.data),
        );
        return;
      }
      _lastRuntimeConfig = result.data;
    } catch (e) {
      errors.add(_error('runtime', 'config failed: $e'));
      _recordParseError(
        'runtime',
        e,
        payloadBytes: _estimateRuntimeBytes(_lastRuntimeConfig),
      );
    }
  }

  KeyFlowSnapshot _composeSnapshot(
    List<DeviceState> devices, {
    List<KeyFlowError> errors = const [],
  }) {
    final mappedDevices = devices
        .map(
          (device) => _toKeyFlowDevice(
            device,
            profiles: _cachedProfiles,
            layouts: _cachedLayouts,
          ),
        )
        .toList(growable: false);

    return KeyFlowSnapshot(
      devices: mappedDevices,
      profiles: _cachedProfiles,
      layouts: _cachedLayouts,
      keymaps: _cachedKeymaps,
      runtime: _runtimeOverlay,
      runtimeConfig: _lastRuntimeConfig,
      generatedAt: DateTime.now(),
      errors: errors,
    );
  }

  KeyFlowDevice _toKeyFlowDevice(
    DeviceState device, {
    required Map<String, HardwareProfile> profiles,
    required Map<String, VirtualLayout> layouts,
  }) {
    final profileId = device.profileId;
    final profile = profileId != null ? profiles[profileId] : null;
    final layoutId = profile?.virtualLayoutId;
    final layout = layoutId != null ? layouts[layoutId] : null;
    final wiring = profile?.wiring ?? const {};

    final conflicts = _countConflicts(wiring);
    final gaps = _countGaps(layout, wiring);

    return KeyFlowDevice(
      id: device.identity.toKey(),
      displayName: device.identity.displayName,
      remapEnabled: device.remapEnabled,
      profileId: profileId,
      layoutId: layoutId,
      wiring: Map<int, String>.from(wiring),
      gaps: gaps,
      conflicts: conflicts,
      lastUpdated: device.updatedAt,
    );
  }

  int _countConflicts(Map<int, String> wiring) {
    final counts = <String, int>{};
    for (final entry in wiring.entries) {
      counts.update(entry.value, (value) => value + 1, ifAbsent: () => 1);
    }
    var conflictTotal = 0;
    for (final value in counts.values) {
      if (value > 1) {
        conflictTotal += value - 1;
      }
    }
    return conflictTotal;
  }

  int _countGaps(VirtualLayout? layout, Map<int, String> wiring) {
    if (layout == null || layout.keys.isEmpty) {
      return 0;
    }
    final mappedVirtualKeys = wiring.values.toSet();
    final expectedKeys = layout.keys.map((key) => key.id).toSet();
    return expectedKeys.difference(mappedVirtualKeys).length;
  }

  KeyFlowError _error(String source, String message) {
    return KeyFlowError(source: source, message: message);
  }

  void _recordParseError(String source, Object? error, {int? payloadBytes}) {
    final message = error?.toString().toLowerCase();
    if (message == null) return;
    if (!message.contains('parse') && !message.contains('format')) {
      return;
    }
    _telemetry?.recordParseError(source: source, payloadBytes: payloadBytes);
  }

  void _maybeRecordParseErrors(
    KeyFlowSnapshot snapshot,
    List<KeyFlowError> errors,
  ) {
    for (final error in errors) {
      _recordParseError(
        error.source,
        error.message,
        payloadBytes: _estimateSnapshotBytes(snapshot),
      );
    }
  }

  int _estimateSnapshotBytes(KeyFlowSnapshot snapshot) {
    final deviceBytes = snapshot.devices.length * 128;
    final profileBytes = snapshot.profiles.length * 256;
    final layoutBytes = _estimateLayoutBytes(snapshot.layouts.values);
    final keymapBytes = snapshot.keymaps.length * 192;
    final runtimeBytes = _estimateRuntimeBytes(snapshot.runtimeConfig);
    return deviceBytes +
        profileBytes +
        layoutBytes +
        keymapBytes +
        runtimeBytes;
  }

  int _estimateLayoutBytes(Iterable<VirtualLayout>? layouts) {
    if (layouts == null) return 0;
    var keyCount = 0;
    for (final layout in layouts) {
      keyCount += layout.keys.length;
    }
    // Roughly 32 bytes per key for sizing; avoids PII.
    return keyCount * 32;
  }

  int _estimateProfilesBytes(Iterable<HardwareProfile>? profiles) {
    if (profiles == null) return 0;
    return profiles.length * 128;
  }

  int _estimateKeymapBytes(Iterable<Keymap>? keymaps) {
    if (keymaps == null) return 0;
    return keymaps.length * 96;
  }

  int _estimateRuntimeBytes(RuntimeConfig? config) {
    if (config == null) return 0;
    var slotCount = 0;
    for (final device in config.devices) {
      slotCount += device.slots.length;
    }
    return (config.devices.length * 64) + (slotCount * 32);
  }

  void _wireRuntimeStream() {
    final updates = _runtimeUpdates;
    if (updates == null) {
      return;
    }

    _runtimeSub = updates.listen(
      _handleRuntimeUpdate,
      onError: (_) {
        // Ignore runtime stream errors; UI can show stale overlay.
      },
    );
  }

  void _handleRuntimeUpdate(BridgeStateUpdate update) {
    final now = DateTime.now();

    var overlay = _runtimeOverlay;
    if (update.fullSnapshot != null) {
      overlay = _applyFullSnapshot(update.fullSnapshot!, now);
    } else {
      overlay = _applyDelta(update.delta, now);
    }

    overlay = overlay.copyWith(
      lastEvent: update.event ?? overlay.lastEvent,
      latencyUs: update.latencyUs ?? overlay.latencyUs,
      version: update.delta.toVersion == 0
          ? overlay.version
          : update.delta.toVersion,
    );

    _runtimeOverlay = overlay;
    if (!_runtimeController.isClosed) {
      _runtimeController.add(_runtimeOverlay);
    }
  }

  RuntimeOverlay _applyFullSnapshot(
    BridgeSnapshot snapshot,
    DateTime timestamp,
  ) {
    final activeLayers = snapshot.activeLayers.isNotEmpty
        ? snapshot.activeLayers
        : [snapshot.baseLayer];

    final standardMods = _standardModifierLabels(snapshot.standardModifierBits);
    final virtualMods = snapshot.virtualModifiers.map((id) => 'M$id');

    return RuntimeOverlay(
      activeLayers: activeLayers,
      activeModifiers: [...standardMods, ...virtualMods],
      heldKeys: snapshot.pressedKeys,
      pendingCount: snapshot.pendingCount,
      lastUpdate: timestamp,
      latencyUs: null,
      version: snapshot.version,
    );
  }

  RuntimeOverlay _applyDelta(BridgeStateDelta delta, DateTime timestamp) {
    final layers = _runtimeOverlay.activeLayers.toList();
    final modifiers = _runtimeOverlay.activeModifiers.toSet();
    final heldKeys = _runtimeOverlay.heldKeys.toSet();
    var pending = _runtimeOverlay.pendingCount;

    for (final change in delta.changes) {
      switch (change.type) {
        case BridgeDeltaChangeType.keyPressed:
          final key = change.payload?.toString();
          if (key != null && key.isNotEmpty) {
            heldKeys.add(key);
          }
          break;
        case BridgeDeltaChangeType.keyReleased:
          final key = change.payload?.toString();
          if (key != null && key.isNotEmpty) {
            heldKeys.remove(key);
          }
          break;
        case BridgeDeltaChangeType.allKeysReleased:
          heldKeys.clear();
          break;
        case BridgeDeltaChangeType.layerActivated:
          final id = change.payload as int?;
          if (id != null && !layers.contains(id)) {
            layers.add(id);
          }
          break;
        case BridgeDeltaChangeType.layerDeactivated:
          layers.remove(change.payload as int?);
          break;
        case BridgeDeltaChangeType.layerStackChanged:
          final payload = change.payload;
          if (payload is List<dynamic>) {
            layers
              ..clear()
              ..addAll(
                payload.map((e) => (e as num?)?.toInt()).whereType<int>(),
              );
          }
          break;
        case BridgeDeltaChangeType.modifierChanged:
          final payload = change.payload;
          if (payload is Map<String, dynamic>) {
            final id = payload['id'];
            final active = payload['active'] == true;
            final label = 'M$id';
            if (active) {
              modifiers.add(label);
            } else {
              modifiers.remove(label);
            }
          }
          break;
        case BridgeDeltaChangeType.allModifiersCleared:
          modifiers.clear();
          break;
        case BridgeDeltaChangeType.pendingAdded:
          final id = change.payload as int?;
          if (id != null) {
            pending += 1;
          }
          break;
        case BridgeDeltaChangeType.pendingResolved:
          final id = change.payload as int?;
          if (id != null && pending > 0) {
            pending -= 1;
          }
          break;
        case BridgeDeltaChangeType.allPendingCleared:
          pending = 0;
          break;
        case BridgeDeltaChangeType.versionChanged:
          // Version handled at call site.
          break;
      }
    }

    return _runtimeOverlay.copyWith(
      activeLayers: layers,
      activeModifiers: modifiers.toList(growable: false),
      heldKeys: heldKeys.toList(growable: false),
      pendingCount: pending,
      lastUpdate: timestamp,
    );
  }

  Iterable<String> _standardModifierLabels(int bits) {
    final labels = <String>[];
    if (bits & 1 != 0) labels.add('Shift');
    if (bits & 2 != 0) labels.add('Control');
    if (bits & 4 != 0) labels.add('Alt');
    if (bits & 8 != 0) labels.add('Meta');
    return labels;
  }
}
