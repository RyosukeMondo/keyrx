/// Unified facade interface for KeyRx operations.
///
/// This facade provides a simplified API surface over the 19+ underlying services,
/// making it easier to use common KeyRx operations without managing multiple
/// service dependencies. It aggregates state from multiple services and coordinates
/// multi-step operations atomically.
///
/// Example usage:
/// ```dart
/// final facade = KeyrxFacade.real(serviceRegistry);
///
/// // Start engine with script validation
/// final result = await facade.startEngine('/path/to/script.rhai');
/// result.when(
///   ok: (_) => print('Engine started'),
///   err: (error) => print('Failed: ${error.userMessage}'),
/// );
///
/// // Observe aggregated state
/// facade.stateStream.listen((state) {
///   if (state.engine == EngineStatus.running) {
///     print('Engine is running');
///   }
/// });
/// ```
library;

import '../../models/discovery_progress.dart';
import '../service_registry.dart';
import '../test_service.dart';
import '../../ffi/bridge.dart';
import 'facade_state.dart';
import 'result.dart';
import 'keyrx_facade_impl.dart';

/// Unified facade for all KeyRx operations.
///
/// Provides:
/// - Simplified API surface (single injection vs 7+ services)
/// - State aggregation (combined engine/device/script status)
/// - Operation coordination (multi-step operations handled atomically)
/// - Error translation (technical errors → user-friendly messages)
/// - Easy testing (mock single facade vs 7+ service mocks)
abstract class KeyrxFacade {
  // === Factories ===

  /// Create a real facade implementation wrapping the provided services.
  ///
  /// This is the standard factory for production use.
  factory KeyrxFacade.real(ServiceRegistry registry) {
    return KeyrxFacadeImpl(registry);
  }

  /// Create a mock facade for testing.
  ///
  /// Used in widget tests to avoid mocking 7+ individual services.
  factory KeyrxFacade.mock() {
    throw UnimplementedError('MockKeyrxFacade not yet implemented');
  }

  // === State ===

  /// Stream of aggregated state updates from all subsystems.
  ///
  /// Emits a new [FacadeState] whenever engine, device, validation, or
  /// discovery state changes. Updates are debounced by 100ms to avoid
  /// excessive emissions during rapid state changes.
  ///
  /// Subscribe to this stream to observe the entire application state
  /// through a single subscription.
  Stream<FacadeState> get stateStream;

  /// Current aggregated state snapshot.
  ///
  /// Returns the most recent [FacadeState] without subscribing to updates.
  /// Useful for checking state in event handlers or one-off queries.
  FacadeState get currentState;

  // === Engine Operations ===

  /// Start the engine with the specified script.
  ///
  /// This is a coordinated operation that:
  /// 1. Validates the script syntax
  /// 2. Loads the script into the engine
  /// 3. Initializes the engine
  /// 4. Updates state to reflect the running engine
  ///
  /// If any step fails, the operation is rolled back and an error returned.
  ///
  /// Returns:
  /// - `Result.ok(void)` if engine started successfully
  /// - `Result.err(error)` if validation, loading, or initialization failed
  Future<Result<void>> startEngine(String scriptPath);

  /// Stop the running engine.
  ///
  /// This coordinated operation:
  /// 1. Stops any active recording
  /// 2. Shuts down the engine
  /// 3. Cleans up resources
  /// 4. Updates state to reflect stopped engine
  ///
  /// Returns:
  /// - `Result.ok(void)` if engine stopped successfully
  /// - `Result.err(error)` if shutdown encountered issues
  Future<Result<void>> stopEngine();

  /// Get the current engine status.
  ///
  /// Returns the detailed status of the engine subsystem.
  ///
  /// Note: For most cases, prefer observing [stateStream] which includes
  /// engine status along with other subsystem states.
  Future<Result<EngineStatus>> getEngineStatus();

  // === Script Operations ===

  /// Validate a script's syntax and structure.
  ///
  /// Checks the script for syntax errors, type errors, and semantic issues.
  /// Returns detailed validation results with error messages and line numbers.
  ///
  /// Returns:
  /// - `Result.ok(ValidationResult)` with validation details
  /// - `Result.err(error)` if the validation process itself failed
  Future<Result<ScriptValidationResult>> validateScript(String scriptPath);

  /// Load script content from a file.
  ///
  /// Reads the script file and returns its content.
  ///
  /// Returns:
  /// - `Result.ok(content)` with the script text
  /// - `Result.err(error)` if file doesn't exist or can't be read
  Future<Result<String>> loadScriptContent(String path);

  /// Save script content to a file.
  ///
  /// Writes the script content to the specified path, creating parent
  /// directories if needed.
  ///
  /// Returns:
  /// - `Result.ok(void)` if saved successfully
  /// - `Result.err(error)` if file operation failed
  Future<Result<void>> saveScript(String path, String content);

  // === Device Operations ===

  /// List all available keyboard devices.
  ///
  /// Scans the system for HID devices and returns keyboard devices
  /// that KeyRx can use.
  ///
  /// Returns:
  /// - `Result.ok(devices)` with the list of available keyboards
  /// - `Result.err(error)` if device enumeration failed
  Future<Result<List<KeyboardDevice>>> listDevices();

  /// Select a device for the engine to use.
  ///
  /// Configures the engine to read input from the specified device.
  ///
  /// Returns:
  /// - `Result.ok(void)` if device selected successfully
  /// - `Result.err(error)` if device not found or selection failed
  Future<Result<void>> selectDevice(String devicePath);

  // === Testing Operations ===

  /// Discover test functions in a Rhai script.
  ///
  /// Scans the script for test function definitions and returns metadata
  /// about each discovered test.
  ///
  /// Returns:
  /// - `Result.ok(discovery)` with list of discovered tests
  /// - `Result.err(error)` if script invalid or discovery failed
  Future<Result<TestDiscoveryServiceResult>> discoverTests(String scriptPath);

  /// Run tests in a Rhai script.
  ///
  /// Executes test functions found in the script and returns results.
  ///
  /// [scriptPath] - Path to the script containing tests
  /// [filter] - Optional pattern to filter which tests to run
  ///
  /// Returns:
  /// - `Result.ok(results)` with test execution results
  /// - `Result.err(error)` if test execution failed
  Future<Result<TestRunServiceResult>> runTests(
    String scriptPath, {
    String? filter,
  });

  /// Cancel currently running tests.
  ///
  /// Stops test execution and cleans up resources. Safe to call even
  /// if no tests are running.
  ///
  /// Returns:
  /// - `Result.ok(void)` if cancelled successfully or no tests running
  /// - `Result.err(error)` if cancellation encountered issues
  Future<Result<void>> cancelTests();

  // === Lifecycle ===

  /// Dispose of facade resources and clean up subscriptions.
  ///
  /// Must be called when the facade is no longer needed to prevent
  /// resource leaks. Stops all subscriptions and releases held resources.
  Future<void> dispose();

  // === Advanced Access ===

  /// Direct access to underlying services for advanced use cases.
  ///
  /// This is an escape hatch for operations not exposed through the facade.
  /// Most code should use the facade methods instead of accessing services
  /// directly.
  ///
  /// Example:
  /// ```dart
  /// // Rare case: need access to a service-specific method
  /// final keyRegistry = await facade.services.engineService.fetchKeyRegistry();
  /// ```
  ServiceRegistry get services;
}

/// Result of script validation.
///
/// Contains validation status and any errors or warnings found.
class ScriptValidationResult {
  const ScriptValidationResult({
    required this.isValid,
    required this.errors,
    required this.warnings,
    this.suggestions,
  });

  /// Whether the script is syntactically valid and can be loaded.
  final bool isValid;

  /// List of validation errors that prevent script execution.
  final List<ValidationIssue> errors;

  /// List of validation warnings (script is valid but has potential issues).
  final List<ValidationIssue> warnings;

  /// Optional suggestions for improving the script.
  final List<String>? suggestions;

  /// Check if validation found any issues (errors or warnings).
  bool get hasIssues => errors.isNotEmpty || warnings.isNotEmpty;

  /// Total number of issues found.
  int get issueCount => errors.length + warnings.length;
}

/// A single validation issue (error or warning).
class ValidationIssue {
  const ValidationIssue({
    required this.message,
    this.line,
    this.column,
    this.severity = IssueSeverity.error,
  });

  /// The issue description.
  final String message;

  /// Line number where the issue occurs (1-based).
  final int? line;

  /// Column number where the issue occurs (1-based).
  final int? column;

  /// Severity level of this issue.
  final IssueSeverity severity;

  /// Format the issue for display.
  String format() {
    final location = (line != null && column != null)
        ? 'Line $line, Col $column: '
        : (line != null)
        ? 'Line $line: '
        : '';
    final severityStr = severity == IssueSeverity.error ? 'Error' : 'Warning';
    return '$severityStr: $location$message';
  }
}

/// Severity level of a validation issue.
enum IssueSeverity {
  /// Critical error that prevents script execution.
  error,

  /// Warning about potential issues but script can still execute.
  warning,
}
