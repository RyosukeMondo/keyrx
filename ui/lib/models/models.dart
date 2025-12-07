/// Barrel file for data models.
///
/// Re-exports all model types for convenient importing.
/// Note: Many models are defined alongside their services for cohesion.
library;

// Layout models
export 'keyboard_layout.dart';

// Validation models
export 'validation.dart';

// Revolutionary mapping models
export 'device_identity.dart';
export 'device_state.dart';
export 'layout_type.dart';
export 'profile.dart';
export 'config_ids.dart';
export 'virtual_layout_type.dart';
export 'virtual_layout.dart';
export 'hardware_profile.dart';
export 'action_binding.dart';
export 'keymap.dart';
export 'runtime_config.dart';

// Service models - re-exported for convenience
// These are defined in their respective service files:
// - KeyboardDevice, KeyboardDeviceInfo → device_service.dart
// - TestCase, TestCaseResult → test_service.dart
// - SimulationResult, KeyMapping → simulation_service.dart
// - SessionRecord, SessionAnalysisData, SessionReplayData → session_service.dart
// - BenchmarkData → benchmark_service.dart
// - DiagnosticCheckData, DiagnosticReport → doctor_service.dart
