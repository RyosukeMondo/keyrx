/// FFI function names and JSON response key constants.
///
/// Use these constants for consistent FFI-related strings
/// throughout the application.
library;

/// FFI function names used in native library lookups.
///
/// These constants match the exported symbols from the Rust core library.
abstract class FfiFunctions {
  // Core functions (always available)

  /// Initialize the KeyRx engine.
  static const init = 'keyrx_init';

  /// Get the library version string.
  static const version = 'keyrx_version';

  /// Load a Rhai script file.
  static const loadScript = 'keyrx_load_script';

  /// Free a string allocated by the native library.
  static const freeString = 'keyrx_free_string';

  // Engine control functions

  /// Evaluate a REPL command.
  static const eval = 'keyrx_eval';

  /// List available key names from the registry.
  static const listKeys = 'keyrx_list_keys';

  /// Check if bypass mode is active.
  static const isBypassActive = 'keyrx_is_bypass_active';

  /// Set bypass mode state.
  static const setBypass = 'keyrx_set_bypass';

  // Device functions

  /// List available input devices.
  static const listDevices = 'keyrx_list_devices';

  /// Select an input device by path.
  static const selectDevice = 'keyrx_select_device';

  // Script functions

  /// Check a script file for errors.
  static const checkScript = 'keyrx_check_script';

  /// Validate script content.
  static const validateScript = 'keyrx_validate_script';

  /// Validate script with options.
  static const validateScriptWithOptions = 'keyrx_validate_script_with_options';

  /// Get key name suggestions for autocomplete.
  static const suggestKeys = 'keyrx_suggest_keys';

  /// Get all valid key names.
  static const allKeyNames = 'keyrx_all_key_names';

  // Testing functions

  /// Discover test functions in a script.
  static const discoverTests = 'keyrx_discover_tests';

  /// Run tests in a script.
  static const runTests = 'keyrx_run_tests';

  /// Simulate key sequences.
  static const simulate = 'keyrx_simulate';

  /// Run performance benchmark.
  static const runBenchmark = 'keyrx_run_benchmark';

  /// Run system diagnostics.
  static const runDoctor = 'keyrx_run_doctor';

  // Session functions

  /// List session files in a directory.
  static const listSessions = 'keyrx_list_sessions';

  /// Analyze a session file.
  static const analyzeSession = 'keyrx_analyze_session';

  /// Replay a session file.
  static const replaySession = 'keyrx_replay_session';

  // Recording functions

  /// Start recording key events.
  static const startRecording = 'keyrx_start_recording';

  /// Stop recording and save session.
  static const stopRecording = 'keyrx_stop_recording';

  // Discovery functions

  /// Start keyboard discovery wizard.
  static const startDiscovery = 'keyrx_start_discovery';

  // Callback registration

  /// Register state change callback.
  static const onState = 'keyrx_on_state';
}

/// JSON response keys used in FFI responses.
///
/// These keys match the JSON structure returned by the Rust core library.
abstract class JsonKeys {
  // Common response keys

  /// Success indicator.
  static const success = 'success';

  /// Error message.
  static const error = 'error';

  /// Generic path field.
  static const path = 'path';

  /// Generic name field.
  static const name = 'name';

  // Key registry keys

  /// Key aliases list.
  static const aliases = 'aliases';

  /// Linux evdev code.
  static const evdev = 'evdev';

  /// Windows virtual key code.
  static const vk = 'vk';

  // Session keys

  /// Session creation timestamp.
  static const created = 'created';

  /// Event count in session.
  static const eventCount = 'eventCount';

  /// Duration in milliseconds.
  static const durationMs = 'durationMs';

  /// Output path for recordings.
  static const outputPath = 'outputPath';

  // Session analysis keys

  /// Session file path.
  static const sessionPath = 'sessionPath';

  /// Average latency in microseconds.
  static const avgLatencyUs = 'avgLatencyUs';

  /// Minimum latency in microseconds.
  static const minLatencyUs = 'minLatencyUs';

  /// Maximum latency in microseconds.
  static const maxLatencyUs = 'maxLatencyUs';

  /// Decision breakdown object.
  static const decisionBreakdown = 'decisionBreakdown';

  // Decision breakdown keys

  /// Pass-through decisions count.
  static const passThrough = 'passThrough';

  /// Remap decisions count.
  static const remap = 'remap';

  /// Block decisions count.
  static const block = 'block';

  /// Tap decisions count.
  static const tap = 'tap';

  /// Hold decisions count.
  static const hold = 'hold';

  /// Combo decisions count.
  static const combo = 'combo';

  /// Layer change decisions count.
  static const layer = 'layer';

  /// Modifier decisions count.
  static const modifier = 'modifier';

  // Replay keys

  /// Total events count.
  static const totalEvents = 'totalEvents';

  /// Matched events count.
  static const matched = 'matched';

  /// Mismatched events count.
  static const mismatched = 'mismatched';

  /// Mismatches list.
  static const mismatches = 'mismatches';

  /// Sequence number.
  static const seq = 'seq';

  /// Recorded value.
  static const recorded = 'recorded';

  /// Actual value.
  static const actual = 'actual';

  // Test keys

  /// Test results list.
  static const results = 'results';

  /// Total tests count.
  static const total = 'total';

  /// Passed tests count.
  static const passed = 'passed';

  /// Failed tests count.
  static const failed = 'failed';

  /// Test file path.
  static const file = 'file';

  /// Source line number.
  static const line = 'line';

  // Benchmark keys

  /// Minimum time in nanoseconds.
  static const minNs = 'minNs';

  /// Maximum time in nanoseconds.
  static const maxNs = 'maxNs';

  /// Mean time in nanoseconds.
  static const meanNs = 'meanNs';

  /// 99th percentile in nanoseconds.
  static const p99Ns = 'p99Ns';

  /// Iteration count.
  static const iterations = 'iterations';

  /// Warning flag.
  static const hasWarning = 'hasWarning';

  /// Warning message.
  static const warning = 'warning';

  // Doctor keys

  /// Diagnostic checks list.
  static const checks = 'checks';

  /// Check status.
  static const status = 'status';

  /// Check details.
  static const details = 'details';

  /// Remediation steps.
  static const remediation = 'remediation';

  /// Warned checks count.
  static const warned = 'warned';

  // Simulation keys

  /// Key mappings list.
  static const mappings = 'mappings';

  /// Input key.
  static const input = 'input';

  /// Output key.
  static const output = 'output';

  /// Decision type.
  static const decision = 'decision';

  /// Active layers list.
  static const activeLayers = 'activeLayers';

  /// Pending decisions list.
  static const pending = 'pending';

  // Validation keys

  /// Valid flag.
  static const valid = 'valid';

  /// Errors list.
  static const errors = 'errors';

  /// Column number.
  static const column = 'column';

  /// Error/warning message.
  static const message = 'message';

  // State snapshot keys

  /// Active layers list.
  static const layers = 'layers';

  /// Active modifiers list.
  static const modifiers = 'modifiers';

  /// Held keys list.
  static const held = 'held';

  /// Last event string.
  static const event = 'event';

  /// Latency in microseconds.
  static const latencyUs = 'latency_us';

  /// Timing configuration object.
  static const timing = 'timing';

  // Timing config keys

  /// Tap timeout in milliseconds.
  static const tapTimeoutMs = 'tap_timeout_ms';

  /// Combo timeout in milliseconds.
  static const comboTimeoutMs = 'combo_timeout_ms';

  /// Hold delay in milliseconds.
  static const holdDelayMs = 'hold_delay_ms';

  /// Eager tap flag.
  static const eagerTap = 'eager_tap';

  /// Permissive hold flag.
  static const permissiveHold = 'permissive_hold';

  /// Retro tap flag.
  static const retroTap = 'retro_tap';

  // Key input keys

  /// Key code.
  static const code = 'code';

  /// Hold duration in milliseconds.
  static const holdMs = 'holdMs';
}

/// Response prefixes used in FFI string responses.
///
/// The Rust core library returns strings prefixed with "ok:" or "error:"
/// to indicate success or failure.
abstract class ResponsePrefixes {
  /// Success prefix.
  static const ok = 'ok:';

  /// Error prefix.
  static const error = 'error:';
}
