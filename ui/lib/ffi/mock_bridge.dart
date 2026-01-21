/// Mock implementation of the KeyRx FFI bridge.
///
/// Provides a Dart-only bridge for development and testing when the native
/// library is unavailable or when `MOCK_FFI` is enabled via --dart-define.
///
/// This is a PARTIAL mock implementation that stubs all required methods.
/// Only frequently-used methods return realistic data. Most methods return
/// success indicators or empty data to allow UI development without FFI.
library;

import 'dart:async';
import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter/foundation.dart';

import '../models/device_identity.dart';
import '../models/device_state.dart';
import '../models/layout_type.dart';
import '../models/profile.dart';
import '../models/validation.dart';
import '../pages/migration_prompt_page.dart';
import 'bindings.dart';
import 'bridge.dart';
import 'generated_bindings.dart';
import 'bridge_config.dart';
import 'bridge_device_profile.dart';
import 'bridge_discovery.dart';
import 'bridge_engine.dart';
import 'bridge_migration.dart';
import 'bridge_session.dart';
import 'bridge_session_types.dart';
import 'bridge_testing.dart';
import 'bridge_testing_types.dart';
import 'bridge_validation.dart';
import 'device_registry_ffi.dart';
import 'event_types.dart';
import 'profile_registry_ffi.dart';

/// Mock implementation providing realistic data for UI development.
///
/// Usage:
/// ```dart
/// // In provider:
/// const useMock = bool.fromEnvironment('MOCK_FFI');
/// final bridge = useMock ? MockKeyrxBridge() : KeyrxBridge.open();
///
/// // Run app:
/// flutter run --dart-define=MOCK_FFI=true
/// ```
class MockKeyrxBridge implements KeyrxBridge {
  MockKeyrxBridge({
    this.ffiLatency = const Duration(milliseconds: 10),
  });

  /// Simulated FFI call latency for testing.
  final Duration ffiLatency;

  bool _initialized = false;
  bool _bypassActive = false;
  final Map<EventType, void Function(Uint8List)> _eventHandlers = {};
  final StreamController<BridgeStateUpdate> _stateController =
      StreamController<BridgeStateUpdate>.broadcast();

  final List<DeviceState> _mockDevices = [
    DeviceState(
      identity: const DeviceIdentity(
        vendorId: 0x1234,
        productId: 0x5678,
        serialNumber: 'MOCK-KB-001',
      ),
      deviceEnabled: true,
      remapEnabled: true,
      connectedAt: DateTime.now().toIso8601String(),
      updatedAt: DateTime.now().toIso8601String(),
    ),
    DeviceState(
      identity: const DeviceIdentity(
        vendorId: 0xABCD,
        productId: 0xEF01,
        serialNumber: 'MOCK-KB-002',
        userLabel: 'My Gaming KB',
      ),
      deviceEnabled: true,
      remapEnabled: false,
      profileId: 'gaming',
      connectedAt: DateTime.now().subtract(const Duration(hours: 2)).toIso8601String(),
      updatedAt: DateTime.now().subtract(const Duration(hours: 2)).toIso8601String(),
    ),
  ];

  // ========== Core Interface ==========

  @override
  KeyrxBindings? get bindings => null;

  @override
  ConfigBindings? get configBindings => null;

  @override
  RuntimeBindings? get runtimeBindings => null;

  @override
  bool get initialized => _initialized;

  @override
  set initialized(bool value) => _initialized = value;

  @override
  Object? get loadFailure => null;

  @override
  StreamController<BridgeStateUpdate>? get stateController => _stateController;

  // ========== BridgeCoreMixin ==========

  @override
  bool initialize() {
    _initialized = true;
    return true;
  }

  @override
  void setConfigRoot(String path) {
    // Mock: no-op
  }

  @override
  String get version => 'mock-0.1.0';

  @override
  bool get isInitialized => _initialized;

  // ========== BridgeEngineMixin ==========

  @override
  bool loadScript(String path) {
    _emitStateUpdate('script_loaded', {'path': path});
    return true;
  }

  @override
  Future<String?> eval(String command) async {
    await Future<void>.delayed(ffiLatency);
    return '{"status": "ok", "result": null}';
  }

  @override
  KeyRegistryResult listKeys() => const KeyRegistryResult(
      entries: [
        KeyRegistryEntry(name: 'A', evdev: 0x1e, vk: 0x41, aliases: ['a']),
        KeyRegistryEntry(name: 'Escape', evdev: 0x01, vk: 0x1b, aliases: ['esc']),
        KeyRegistryEntry(name: 'CapsLock', evdev: 0x3a, vk: 0x14, aliases: ['caps']),
      ],
    );

  @override
  bool isBypassActive() => _bypassActive;

  @override
  void setBypass(bool active) {
    _bypassActive = active;
    _emitStateUpdate('bypass_changed', {'active': active});
  }

  @override
  ScriptValidationResult checkScript(String path) => const ScriptValidationResult(valid: true, errors: []);

  @override
  Stream<BridgeStateUpdate> get stateStream => _stateController.stream;

  // ========== BridgeSessionMixin ==========

  @override
  RecordingStartResult startRecording(String path) => RecordingStartResult(
      success: true,
      outputPath: '$path/mock-session-${DateTime.now().millisecondsSinceEpoch}.json',
    );

  @override
  RecordingStopResult stopRecording() => const RecordingStopResult(
      success: true,
      durationMs: 10000.0,
      eventCount: 42,
    );

  @override
  SessionListResult listSessions(String dirPath) => SessionListResult(sessions: [
      SessionInfo(
        path: '$dirPath/session-001.json',
        name: 'session-001',
        created: DateTime.now().subtract(const Duration(days: 1)).toIso8601String(),
        eventCount: 150,
        durationMs: 300000.0,
      ),
    ]);

  @override
  SessionAnalysisResult analyzeSession(String path) => SessionAnalysisResult(
      analysis: SessionAnalysis(
        sessionPath: path,
        eventCount: 150,
        durationMs: 300000.0,
        avgLatencyUs: 150,
        minLatencyUs: 50,
        maxLatencyUs: 500,
        decisionBreakdown: const DecisionBreakdown(
          passThrough: 20,
          remap: 100,
          block: 10,
          tap: 60,
          hold: 30,
          combo: 15,
          layer: 5,
          modifier: 10,
        ),
      ),
    );

  @override
  ReplayResult replaySession(String path, {bool verify = false}) => const ReplayResult(
      totalEvents: 150,
      matched: 150,
      mismatched: 0,
      success: true,
      mismatches: [],
    );

  // ========== BridgeDeviceProfileMixin ==========

  @override
  DeviceProfileResult getDeviceProfile(int vendorId, int productId) {
    if (vendorId == 0x1234 && productId == 0x5678) {
      return DeviceProfileResult.success(
        DeviceProfile(
          schemaVersion: 1,
          vendorId: vendorId,
          productId: productId,
          name: 'Mock Keyboard',
          discoveredAt: DateTime.now(),
          rows: 6,
          colsPerRow: const [15, 15, 15, 15, 15, 15],
          keymap: const {},
          aliases: const {},
          source: ProfileSource.defaultSource,
        ),
      );
    }
    return DeviceProfileResult.error('Profile not found');
  }

  @override
  bool hasDeviceProfile(int vendorId, int productId) => vendorId == 0x1234 && productId == 0x5678;

  @override
  bool saveDeviceProfile(String profileJson) => true;

  // ========== BridgeDiscoveryMixin ==========

  @override
  int selectDevice(String path) => 1;

  // ========== BridgeTestingMixin ==========

  @override
  TestDiscoveryResult discoverTests(String path) => TestDiscoveryResult(tests: [
      DiscoveredTest(name: 'mock_test_1', file: '$path/test1.keyrx'),
    ]);

  @override
  TestRunResult runTests(String path, {String? filter}) => const TestRunResult(
      total: 6,
      passed: 5,
      failed: 0,
      durationMs: 120.0,
      results: [],
    );

  @override
  SimulationResult simulate(
    List<KeyInput> keys, {
    String? scriptPath,
    bool comboMode = false,
  }) => SimulationResult(
      mappings: keys.map((k) => SimulationMapping(
        input: k.toString(),
        output: k.toString(),
        decision: 'PassThrough',
      )).toList(),
      activeLayers: const ['0'],
      pending: const [],
    );

  @override
  BenchmarkResult runBenchmark(int iterations, {String? scriptPath}) => BenchmarkResult(
      minNs: 100000,
      maxNs: 500000,
      meanNs: 150000,
      p99Ns: 250000,
      iterations: iterations,
      hasWarning: false,
    );

  @override
  DoctorResult runDoctor() => const DoctorResult(
      checks: [
        DiagnosticCheck(name: 'FFI Bridge', status: 'passed', details: 'Mock OK'),
      ],
      passed: 1,
      failed: 0,
      warned: 0,
    );

  // ========== BridgeValidationMixin ==========

  @override
  ValidationResult validateScript(String script, [ValidationOptions? options]) {
    if (script.trim().isEmpty) {
      return const ValidationResult(
        isValid: false,
        errors: [ValidationError(code: 'E001', message: 'Empty script')],
        warnings: [],
      );
    }
    return const ValidationResult(isValid: true, errors: [], warnings: []);
  }

  @override
  List<String> suggestKeys(String partial) {
    final all = ['A', 'B', 'C', 'Escape', 'Enter', 'Space', 'CapsLock'];
    return all.where((k) => k.toLowerCase().startsWith(partial.toLowerCase())).toList();
  }

  @override
  List<String> allKeyNames() => ['A', 'B', 'C', 'Escape', 'Enter', 'Space', 'CapsLock'];

  // ========== BridgeConfigMixin ==========

  @override
  ConfigPathResult getConfigRoot() => ConfigPathResult.success('/tmp/keyrx-mock-config');

  // ========== BridgeMigrationMixin ==========

  @override
  Future<bool> checkMigrationNeeded(String oldProfilesDir) async {
    await Future<void>.delayed(ffiLatency);
    return false; // No migration needed in mock
  }

  @override
  Future<MigrationReport> runMigration(
    String oldProfilesDir,
    String newProfilesDir, {
    bool createBackup = true,
  }) async {
    await Future<void>.delayed(ffiLatency);
    return MigrationReport(
      totalCount: 0,
      migratedCount: 0,
      failedCount: 0,
      backupPath: createBackup ? '/tmp/backup' : null,
    );
  }

  // ========== DeviceRegistryFFIMixin ==========

  @override
  DeviceRegistryResult<List<DeviceState>> listRegisteredDevices() => DeviceRegistryResult.success(List<DeviceState>.from(_mockDevices));

  @override
  DeviceRegistryResult<void> setDeviceEnabled(String deviceKey, bool enabled) {
    final device = _mockDevices.firstWhere(
      (d) => d.identity.toKey() == deviceKey,
      orElse: () => _mockDevices.first,
    );
    final index = _mockDevices.indexOf(device);
    // When disabling device, also disable remap
    _mockDevices[index] = device.copyWith(
      deviceEnabled: enabled,
      remapEnabled: enabled ? device.remapEnabled : false,
    );
    return DeviceRegistryResult.success(null);
  }

  @override
  DeviceRegistryResult<void> setRemapEnabled(String deviceKey, bool enabled) {
    final device = _mockDevices.firstWhere(
      (d) => d.identity.toKey() == deviceKey,
      orElse: () => _mockDevices.first,
    );
    final index = _mockDevices.indexOf(device);
    _mockDevices[index] = device.copyWith(remapEnabled: enabled);
    return DeviceRegistryResult.success(null);
  }

  @override
  DeviceRegistryResult<void> assignProfile(String deviceKey, String profileId) {
    final device = _mockDevices.firstWhere(
      (d) => d.identity.toKey() == deviceKey,
      orElse: () => _mockDevices.first,
    );
    final index = _mockDevices.indexOf(device);
    _mockDevices[index] = device.copyWith(profileId: profileId);
    return DeviceRegistryResult.success(null);
  }

  @override
  DeviceRegistryResult<void> setUserLabel(String deviceKey, String? label) {
    final device = _mockDevices.firstWhere(
      (d) => d.identity.toKey() == deviceKey,
      orElse: () => _mockDevices.first,
    );
    final index = _mockDevices.indexOf(device);
    final updatedIdentity = device.identity.copyWith(userLabel: label);
    _mockDevices[index] = device.copyWith(identity: updatedIdentity);
    return DeviceRegistryResult.success(null);
  }

  // ========== ProfileRegistryFFIMixin ==========

  @override
  ProfileRegistryResult<List<String>> listProfiles() => ProfileRegistryResult.success(['default', 'gaming', 'productivity']);

  @override
  ProfileRegistryResult<Profile> getProfile(String profileId) => ProfileRegistryResult.success(
      Profile(
        id: profileId,
        name: profileId.toUpperCase(),
        layoutType: LayoutType.standard,
        createdAt: DateTime.now().toIso8601String(),
        updatedAt: DateTime.now().toIso8601String(),
      ),
    );

  @override
  ProfileRegistryResult<void> saveProfile(Profile profile) => ProfileRegistryResult.success(null);

  @override
  ProfileRegistryResult<void> deleteProfile(String profileId) => ProfileRegistryResult.success(null);

  @override
  ProfileRegistryResult<List<Profile>> findCompatibleProfiles(
    LayoutType layoutType,
  ) => ProfileRegistryResult.success([
      Profile(
        id: 'default',
        name: 'Default',
        layoutType: layoutType,
        createdAt: DateTime.now().toIso8601String(),
        updatedAt: DateTime.now().toIso8601String(),
      ),
    ]);

  // ========== KeyrxBridge Methods ==========

  @override
  Future<void> dispose() async {
    await _stateController.close();
    _eventHandlers.clear();
  }

  @override
  bool startEngineLoop() => true;

  @override
  bool stopEngineLoop() => true;

  @override
  void shutdown() {
    _initialized = false;
  }

  @override
  bool registerEventCallback(
    EventType eventType,
    void Function(Uint8List) handler,
  ) {
    _eventHandlers[eventType] = handler;
    return true;
  }

  @override
  bool unregisterEventCallback(EventType eventType) => _eventHandlers.remove(eventType) != null;

  @override
  bool isEventCallbackRegistered(EventType eventType) => _eventHandlers.containsKey(eventType);

  @override
  void requestFullStateResubscribe() {
    _emitStateUpdate('full_resubscribe', {});
  }

  // ========== Helper Methods ==========

  void _emitStateUpdate(String event, Map<String, dynamic> payload) {
    if (_stateController.isClosed) return;

    final delta = BridgeStateDelta(
      fromVersion: 0,
      toVersion: 1,
      changes: [
        BridgeDeltaChange(
          BridgeDeltaChangeType.versionChanged,
          payload: payload,
        ),
      ],
    );

    _stateController.add(BridgeStateUpdate(
      delta: delta,
      event: event,
      latencyUs: ffiLatency.inMicroseconds,
    ));
  }

  /// Simulate event callback trigger (for testing).
  @visibleForTesting
  void simulateEvent(EventType type, Map<String, dynamic> data) {
    final handler = _eventHandlers[type];
    if (handler != null) {
      final json = jsonEncode(data);
      final bytes = Uint8List.fromList(utf8.encode(json));
      handler(bytes);
    }
  }
}
