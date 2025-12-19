/// Barrel file for data models.
///
/// Re-exports all model types for convenient importing.
/// Note: Many models are defined alongside their services for cohesion.
library;

export 'action_binding.dart';
export 'config_ids.dart';
// Revolutionary mapping models
export 'device_config.dart';
export 'device_identity.dart';
export 'device_state.dart';
export 'flow/flow_graph.dart';
// Generated FFI models
export 'generated_models.dart';
export 'hardware_profile.dart';
// Layout models
export 'keyboard_layout.dart';
export 'keymap.dart';
export 'layout_type.dart';
export 'profile.dart';
export 'runtime_config.dart';
// Validation models
export 'validation.dart';
export 'virtual_layout.dart';
export 'virtual_layout_type.dart';

// Service models - re-exported for convenience
// These are defined in their respective service files:
// - KeyboardDevice, KeyboardDeviceInfo → device_service.dart
// - TestCase, TestCaseResult → test_service.dart
// - SimulationResult, KeyMapping → simulation_service.dart
// - SessionRecord, SessionAnalysisData, SessionReplayData → session_service.dart
// - BenchmarkData → benchmark_service.dart
// - DiagnosticCheckData, DiagnosticReport → doctor_service.dart
