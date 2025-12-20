/// Repository for runtime KeyFlow configuration with caching and streaming.
library;

import 'dart:async';
import 'dart:convert';
import 'dart:typed_data';

import 'package:core_logic/ffi/bridge.dart'
    if (dart.library.html) 'package:core_logic/ffi/bridge_stub.dart';
import 'package:core_logic/models/hardware_profile.dart';
import 'package:core_logic/models/runtime_config.dart';
import 'package:core_logic/services/config_result.dart';
import 'package:core_logic/utils/logger.dart';

/// Lightweight runtime API contract to simplify testing.
abstract class RuntimeApi {
  Object? get loadFailure;

  String getConfig();
  String addSlot(String deviceJson, String slotJson, int priority);
  String removeSlot(String deviceJson, String slotId);
}

class _BridgeRuntimeApi implements RuntimeApi {
  _BridgeRuntimeApi(this._bridge);

  final KeyrxBridge _bridge;

  dynamic get _bindings => _bridge.runtimeBindings;

  @override
  Object? get loadFailure => _bridge.loadFailure;

  @override
  String addSlot(String deviceJson, String slotJson, int priority) {
    final bindings = _bindings;
    if (bindings == null) throw StateError('Runtime bindings unavailable');
    return bindings.addSlot(deviceJson, slotJson, priority) as String;
  }

  @override
  String getConfig() {
    final bindings = _bindings;
    if (bindings == null) throw StateError('Runtime bindings unavailable');
    return bindings.getConfig() as String;
  }

  @override
  String removeSlot(String deviceJson, String slotId) {
    final bindings = _bindings;
    if (bindings == null) throw StateError('Runtime bindings unavailable');
    return bindings.removeSlot(deviceJson, slotId) as String;
  }
}

/// Data layer for runtime configuration via the FFI bridge.
///
/// Provides TTL caching, mutation helpers, structured logging, and a
/// broadcast stream for real-time updates.
class KeyFlowRepository {
  KeyFlowRepository({
    required RuntimeApi runtimeApi,
    Duration cacheTtl = const Duration(minutes: 5),
    StructuredLogger? logger,
    DateTime Function()? clock,
    void Function(EventType, void Function(Uint8List))? registerEventCallback,
    void Function(EventType)? unregisterEventCallback,
  }) : _runtimeApi = runtimeApi,
       _cacheTtl = cacheTtl,
       _clock = clock ?? DateTime.now,
       _logger = logger ?? getLogger('keyflow_repository'),
       _registerEventCallback = registerEventCallback,
       _unregisterEventCallback = unregisterEventCallback {
    _logger.info(
      'keyflow.repo.init',
      context: {'cache_ttl_ms': _cacheTtl.inMilliseconds},
    );
    _registerBridgeListeners();
  }

  factory KeyFlowRepository.forBridge(
    KeyrxBridge bridge, {
    Duration cacheTtl = const Duration(minutes: 5),
    StructuredLogger? logger,
    DateTime Function()? clock,
  }) => KeyFlowRepository(
    runtimeApi: _BridgeRuntimeApi(bridge),
    cacheTtl: cacheTtl,
    logger: logger,
    clock: clock,
    registerEventCallback: bridge.registerEventCallback,
    unregisterEventCallback: bridge.unregisterEventCallback,
  );

  final RuntimeApi _runtimeApi;
  final Duration _cacheTtl;
  final DateTime Function() _clock;
  final StructuredLogger _logger;
  final void Function(EventType, void Function(Uint8List))?
  _registerEventCallback;
  final void Function(EventType)? _unregisterEventCallback;

  RuntimeConfig? _cachedConfig;
  DateTime? _lastFetch;
  bool _disposed = false;
  bool _refreshingFromEvent = false;
  Future<ConfigOperationResult<RuntimeConfig>>? _inflightFetch;

  final _configController = StreamController<RuntimeConfig>.broadcast();

  /// Stream of runtime configs; emits on cache updates and FFI-driven changes.
  Stream<RuntimeConfig> watchRuntimeConfig() => _configController.stream;

  /// Dispose stream resources and unregister callbacks.
  Future<void> dispose() async {
    _disposed = true;
    _unregisterEventCallback?.call(EventType.deviceConnected);
    _unregisterEventCallback?.call(EventType.deviceDisconnected);
    _unregisterEventCallback?.call(EventType.deviceUpdated);
    _unregisterEventCallback?.call(EventType.profileUpdated);
    await _configController.close();
  }

  /// Get runtime config using a 5 minute TTL cache unless [forceRefresh].
  Future<ConfigOperationResult<RuntimeConfig>> getRuntimeConfig({
    bool forceRefresh = false,
  }) async {
    if (!forceRefresh && _cachedConfig != null && _isCacheFresh) {
      _logger.debug('keyflow.repo.cache_hit', context: {'age_ms': _cacheAgeMs});
      return ConfigOperationResult.success(_cachedConfig);
    }

    if (_inflightFetch != null) {
      _logger.debug('keyflow.repo.join_inflight');
      return _inflightFetch!;
    }

    _inflightFetch = _fetchAndCache('get_runtime_config');
    final result = await _inflightFetch!;
    _inflightFetch = null;
    return result;
  }

  /// Add a slot; if no priority provided, use highest+1 for the device.
  Future<ConfigOperationResult<RuntimeConfig>> addSlot(
    DeviceInstanceId device,
    ProfileSlot slot, {
    int? priorityOverride,
  }) async {
    final priority =
        priorityOverride ?? await _computeNextPriority(device, slot.priority);
    final slotWithPriority = slot.copyWith(priority: priority);

    return _performMutation(
      operation: 'add_slot',
      invoke: () => _runtimeApi.addSlot(
        jsonEncode(device.toJson()),
        jsonEncode(slotWithPriority.toJson()),
        priority,
      ),
    );
  }

  /// Update an existing slot (upsert semantics via addSlot FFI call).
  Future<ConfigOperationResult<RuntimeConfig>> updateSlot(
    DeviceInstanceId device,
    ProfileSlot slot,
  ) => _performMutation(
    operation: 'update_slot',
    invoke: () => _runtimeApi.addSlot(
      jsonEncode(device.toJson()),
      jsonEncode(slot.toJson()),
      slot.priority,
    ),
  );

  /// Remove a slot for a device.
  Future<ConfigOperationResult<RuntimeConfig>> removeSlot(
    DeviceInstanceId device,
    String slotId,
  ) => _performMutation(
    operation: 'remove_slot',
    invoke: () => _runtimeApi.removeSlot(jsonEncode(device.toJson()), slotId),
  );

  Future<ConfigOperationResult<RuntimeConfig>> _fetchAndCache(
    String operation,
  ) async {
    final result = await _callRuntime(operation, _runtimeApi.getConfig);
    if (result.isSuccess && result.data != null) {
      _cacheConfig(result.data!, reason: operation);
    }
    return result;
  }

  Future<ConfigOperationResult<RuntimeConfig>> _performMutation({
    required String operation,
    required String Function() invoke,
  }) async {
    final result = await _callRuntime(operation, invoke);
    if (result.isSuccess && result.data != null) {
      _cacheConfig(result.data!, reason: operation);
    } else {
      _invalidateCache(reason: '${operation}_error');
    }
    return result;
  }

  Future<ConfigOperationResult<RuntimeConfig>> _callRuntime(
    String operation,
    String Function() invoke,
  ) async {
    if (_runtimeApi.loadFailure != null) {
      final message = 'Engine unavailable: ${_runtimeApi.loadFailure}';
      _logger.error(
        'keyflow.repo.bridge_unavailable',
        message: message,
        context: {'operation': operation},
      );
      return ConfigOperationResult.error(message);
    }

    try {
      final raw = invoke();
      final decoded = _decodeRuntimeConfig(raw);
      if (decoded.isSuccess) {
        _logger.info(
          'keyflow.repo.$operation.success',
          context: {'cached': _cachedConfig != null},
        );
      } else {
        _logger.warn(
          'keyflow.repo.$operation.error',
          message: decoded.errorMessage,
        );
      }
      return decoded;
    } on Exception catch (e, st) {
      _logger.error(
        'keyflow.repo.$operation.exception',
        message: '$e',
        error: e,
        stackTrace: st,
      );
      return ConfigOperationResult.error('$e');
    }
  }

  ConfigOperationResult<RuntimeConfig> _decodeRuntimeConfig(String raw) {
    final parsed = parseConfigFfiResult<RuntimeConfig>(raw, (json) {
      if (json is Map<String, dynamic>) {
        return RuntimeConfig.fromJson(json);
      }
      throw const FormatException('RuntimeConfig expects a JSON object');
    });
    if (parsed.isSuccess && parsed.data != null) {
      return parsed;
    }

    try {
      var payload = raw.trim();
      if (payload.startsWith('ok:')) {
        payload = payload.substring(3).trim();
      }
      final map = jsonDecode(payload) as Map<String, dynamic>;
      final config = RuntimeConfig.fromJson(map);
      return ConfigOperationResult.success(config);
    } on Object catch (_) {
      // Fall through to the original parse error.
    }

    return ConfigOperationResult.error(
      parsed.errorMessage ?? 'Failed to parse runtime config',
    );
  }

  Future<int> _computeNextPriority(
    DeviceInstanceId device,
    int fallbackPriority,
  ) async {
    final cached = _cachedConfig;
    if (cached == null) {
      final fetched = await getRuntimeConfig();
      if (fetched.data == null) {
        return fallbackPriority;
      }
      return _computeNextPriorityForConfig(
        fetched.data!,
        device,
        fallbackPriority,
      );
    }
    return _computeNextPriorityForConfig(cached, device, fallbackPriority);
  }

  int _computeNextPriorityForConfig(
    RuntimeConfig config,
    DeviceInstanceId device,
    int fallbackPriority,
  ) {
    final slots = config.devices
        .firstWhere(
          (d) => d.device == device,
          orElse: () => DeviceSlots(device: device),
        )
        .slots;
    if (slots.isEmpty) {
      return fallbackPriority;
    }
    final highest = slots
        .map((s) => s.priority)
        .fold<int>(-1, (a, b) => a > b ? a : b);
    return highest + 1;
  }

  void _cacheConfig(RuntimeConfig config, {required String reason}) {
    _cachedConfig = config;
    _lastFetch = _clock();
    _logger.debug(
      'keyflow.repo.cache_set',
      context: {'reason': reason, 'ts': _lastFetch!.toIso8601String()},
    );
    if (!_configController.isClosed) {
      _configController.add(config);
    }
  }

  void _invalidateCache({required String reason}) {
    _logger.debug('keyflow.repo.cache_clear', context: {'reason': reason});
    _cachedConfig = null;
    _lastFetch = null;
  }

  bool get _isCacheFresh {
    final last = _lastFetch;
    if (last == null) return false;
    return _clock().difference(last) < _cacheTtl;
  }

  int get _cacheAgeMs {
    final last = _lastFetch;
    if (last == null) return _cacheTtl.inMilliseconds;
    return _clock().difference(last).inMilliseconds;
  }

  void _registerBridgeListeners() {
    if (_registerEventCallback == null) {
      return;
    }
    for (final type in const [
      EventType.deviceConnected,
      EventType.deviceDisconnected,
      EventType.deviceUpdated,
      EventType.profileUpdated,
    ]) {
      _registerEventCallback(type, _handleBridgeEvent);
    }
  }

  void _handleBridgeEvent(Uint8List _) {
    if (_disposed || _refreshingFromEvent) return;
    _refreshingFromEvent = true;
    getRuntimeConfig(forceRefresh: true).whenComplete(() {
      _refreshingFromEvent = false;
    });
  }
}
