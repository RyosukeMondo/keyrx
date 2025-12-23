//! E2E Test Harness for Virtual Keyboard Testing.
//!
//! This module provides infrastructure for running end-to-end tests using
//! virtual input devices (uinput) instead of requiring physical hardware.
//!
//! # Components
//!
//! - [`E2EError`]: Error types for E2E test operations
//! - [`E2EConfig`]: Test configuration with helper constructors
//! - [`E2EHarness`]: Complete test orchestration
//!
//! # Example
//!
//! ```ignore
//! use keyrx_daemon::tests::e2e_harness::{E2EConfig, E2EHarness};
//!
//! // Create a simple remap configuration
//! let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
//!
//! // Setup the test environment (starts daemon as subprocess)
//! let harness = E2EHarness::setup(config)?;
//! ```

#![cfg(all(target_os = "linux", feature = "linux"))]

use std::fmt;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use keyrx_compiler::serialize::serialize as serialize_config;
use keyrx_core::config::{
    BaseKeyMapping, Condition, ConditionItem, ConfigRoot, DeviceConfig, DeviceIdentifier, KeyCode,
    KeyMapping, Metadata, Version,
};
use keyrx_core::runtime::KeyEvent;
use keyrx_daemon::test_utils::{OutputCapture, VirtualDeviceError, VirtualKeyboard};

// ============================================================================
// TestTimeoutPhase - Phase tracking for timeout diagnostics
// ============================================================================

/// Represents the phase of an E2E test for timeout diagnostics.
///
/// When a test times out, this enum identifies exactly which phase was
/// executing, enabling precise diagnosis of where the test hung.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestTimeoutPhase {
    /// Test setup phase (creating virtual devices, starting daemon)
    Setup,
    /// Event injection phase (sending key events to virtual keyboard)
    Injection,
    /// Event capture phase (reading events from daemon output)
    Capture,
    /// Verification phase (comparing captured vs expected events)
    Verification,
    /// Test cleanup phase (stopping daemon, destroying devices)
    Teardown,
    /// User-defined test logic
    TestLogic,
}

impl fmt::Display for TestTimeoutPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestTimeoutPhase::Setup => write!(f, "setup"),
            TestTimeoutPhase::Injection => write!(f, "event injection"),
            TestTimeoutPhase::Capture => write!(f, "event capture"),
            TestTimeoutPhase::Verification => write!(f, "verification"),
            TestTimeoutPhase::Teardown => write!(f, "teardown"),
            TestTimeoutPhase::TestLogic => write!(f, "test logic"),
        }
    }
}

// ============================================================================
// E2EError - Error types for E2E test operations
// ============================================================================

/// Errors that can occur during E2E test operations.
///
/// This error type wraps [`VirtualDeviceError`] and adds E2E-specific error
/// variants for test setup, execution, and verification.
#[derive(Debug)]
pub enum E2EError {
    /// Error from virtual device operations (VirtualKeyboard, OutputCapture).
    VirtualDevice(VirtualDeviceError),

    /// Failed to create or serialize test configuration.
    ConfigError {
        /// Description of what went wrong
        message: String,
    },

    /// Failed to start daemon subprocess.
    DaemonStartError {
        /// Description of what went wrong
        message: String,
        /// Standard error output from daemon, if available
        stderr: Option<String>,
    },

    /// Daemon exited unexpectedly during test.
    DaemonCrashed {
        /// Exit code if available
        exit_code: Option<i32>,
        /// Standard error output from daemon, if available
        stderr: Option<String>,
    },

    /// Test verification failed - captured events don't match expected.
    VerificationFailed {
        /// Events that were captured during the test
        captured: Vec<KeyEvent>,
        /// Events that were expected
        expected: Vec<KeyEvent>,
        /// Detailed diff message
        diff: String,
    },

    /// Test timed out waiting for expected condition.
    Timeout {
        /// What operation timed out
        operation: String,
        /// How long we waited
        timeout_ms: u64,
    },

    /// Test exceeded its overall time limit.
    ///
    /// This error is returned when a test wrapped with [`with_timeout`] exceeds
    /// the maximum allowed duration. It includes diagnostic information about
    /// which phase was executing when the timeout occurred.
    TestTimeout {
        /// Which phase of the test timed out
        phase: TestTimeoutPhase,
        /// Total test timeout duration
        timeout: Duration,
        /// Time elapsed when timeout occurred
        elapsed: Duration,
        /// Diagnostic context about what was happening
        context: String,
    },

    /// I/O error during test operations.
    Io(std::io::Error),
}

impl std::error::Error for E2EError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            E2EError::VirtualDevice(e) => Some(e),
            E2EError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for E2EError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            E2EError::VirtualDevice(e) => write!(f, "virtual device error: {}", e),
            E2EError::ConfigError { message } => write!(f, "config error: {}", message),
            E2EError::DaemonStartError { message, stderr } => {
                write!(f, "daemon start error: {}", message)?;
                if let Some(stderr) = stderr {
                    write!(f, "\nstderr: {}", stderr)?;
                }
                Ok(())
            }
            E2EError::DaemonCrashed { exit_code, stderr } => {
                write!(f, "daemon crashed")?;
                if let Some(code) = exit_code {
                    write!(f, " with exit code {}", code)?;
                }
                if let Some(stderr) = stderr {
                    write!(f, "\nstderr: {}", stderr)?;
                }
                Ok(())
            }
            E2EError::VerificationFailed {
                captured,
                expected,
                diff,
            } => {
                writeln!(f, "verification failed:")?;
                writeln!(f, "  expected {} event(s): {:?}", expected.len(), expected)?;
                writeln!(f, "  captured {} event(s): {:?}", captured.len(), captured)?;
                write!(f, "\n{}", diff)
            }
            E2EError::Timeout {
                operation,
                timeout_ms,
            } => {
                write!(
                    f,
                    "timeout after {}ms waiting for {}",
                    timeout_ms, operation
                )
            }
            E2EError::TestTimeout {
                phase,
                timeout,
                elapsed,
                context,
            } => {
                writeln!(
                    f,
                    "TEST TIMEOUT: test exceeded {}s limit (elapsed: {:.2}s)",
                    timeout.as_secs(),
                    elapsed.as_secs_f64()
                )?;
                writeln!(f, "  Phase: {}", phase)?;
                if !context.is_empty() {
                    writeln!(f, "  Context: {}", context)?;
                }
                write!(f, "  Action: Check for hung daemon or slow I/O operations")
            }
            E2EError::Io(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl From<VirtualDeviceError> for E2EError {
    fn from(err: VirtualDeviceError) -> Self {
        E2EError::VirtualDevice(err)
    }
}

impl From<std::io::Error> for E2EError {
    fn from(err: std::io::Error) -> Self {
        E2EError::Io(err)
    }
}

// ============================================================================
// E2EConfig - Test configuration with helper constructors
// ============================================================================

/// Configuration for an E2E test scenario.
///
/// Provides helper constructors to easily create test configurations for
/// common remapping scenarios. The configuration includes:
///
/// - Device pattern for matching keyboards
/// - Key mappings to apply
///
/// # Example
///
/// ```ignore
/// // Simple A → B remapping
/// let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
///
/// // Navigation layer with modifier
/// let config = E2EConfig::with_modifier_layer(
///     KeyCode::CapsLock,
///     0,
///     vec![
///         (KeyCode::H, KeyCode::Left),
///         (KeyCode::J, KeyCode::Down),
///     ],
/// );
/// ```
#[derive(Debug, Clone)]
pub struct E2EConfig {
    /// Device pattern for matching (default: "*" for all devices)
    pub device_pattern: String,
    /// Key mappings to apply
    pub mappings: Vec<KeyMapping>,
}

impl E2EConfig {
    /// Creates a new E2EConfig with the given device pattern and mappings.
    #[allow(dead_code)]
    pub fn new(device_pattern: impl Into<String>, mappings: Vec<KeyMapping>) -> Self {
        Self {
            device_pattern: device_pattern.into(),
            mappings,
        }
    }

    /// Creates a configuration with a simple key remapping (A → B).
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = E2EConfig::simple_remap(KeyCode::CapsLock, KeyCode::Escape);
    /// ```
    pub fn simple_remap(from: KeyCode, to: KeyCode) -> Self {
        Self {
            device_pattern: "*".to_string(),
            mappings: vec![KeyMapping::simple(from, to)],
        }
    }

    /// Creates a configuration with multiple simple remappings.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = E2EConfig::simple_remaps(vec![
    ///     (KeyCode::A, KeyCode::B),
    ///     (KeyCode::CapsLock, KeyCode::Escape),
    /// ]);
    /// ```
    pub fn simple_remaps(remaps: Vec<(KeyCode, KeyCode)>) -> Self {
        let mappings = remaps
            .into_iter()
            .map(|(from, to)| KeyMapping::simple(from, to))
            .collect();

        Self {
            device_pattern: "*".to_string(),
            mappings,
        }
    }

    /// Creates a configuration with a custom modifier key.
    ///
    /// The modifier key will set internal state when held, but produces no
    /// output events.
    ///
    /// # Arguments
    ///
    /// * `from` - The key that activates the modifier
    /// * `modifier_id` - The modifier ID (0-254)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = E2EConfig::modifier(KeyCode::CapsLock, 0);
    /// ```
    pub fn modifier(from: KeyCode, modifier_id: u8) -> Self {
        Self {
            device_pattern: "*".to_string(),
            mappings: vec![KeyMapping::modifier(from, modifier_id)],
        }
    }

    /// Creates a configuration with a toggle lock key.
    ///
    /// The lock key toggles internal state on press (no output on release).
    ///
    /// # Arguments
    ///
    /// * `from` - The key that toggles the lock
    /// * `lock_id` - The lock ID (0-254)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = E2EConfig::lock(KeyCode::ScrollLock, 0);
    /// ```
    pub fn lock(from: KeyCode, lock_id: u8) -> Self {
        Self {
            device_pattern: "*".to_string(),
            mappings: vec![KeyMapping::lock(from, lock_id)],
        }
    }

    /// Creates a configuration with a conditional mapping.
    ///
    /// Maps `from` → `to` only when the specified modifier is active.
    ///
    /// # Arguments
    ///
    /// * `modifier_id` - The modifier that must be active
    /// * `from` - Source key for the mapping
    /// * `to` - Target key for the mapping
    ///
    /// # Example
    ///
    /// ```ignore
    /// // When modifier 0 is active, H → Left
    /// let config = E2EConfig::conditional(0, KeyCode::H, KeyCode::Left);
    /// ```
    pub fn conditional(modifier_id: u8, from: KeyCode, to: KeyCode) -> Self {
        Self {
            device_pattern: "*".to_string(),
            mappings: vec![KeyMapping::conditional(
                Condition::ModifierActive(modifier_id),
                vec![BaseKeyMapping::Simple { from, to }],
            )],
        }
    }

    /// Creates a configuration with a modifier key and conditional mappings.
    ///
    /// This is the common pattern for navigation layers (e.g., Vim-style HJKL).
    ///
    /// # Arguments
    ///
    /// * `modifier_key` - The key that activates the layer
    /// * `modifier_id` - The modifier ID for the layer
    /// * `layer_mappings` - List of (from, to) pairs active when layer is held
    ///
    /// # Example
    ///
    /// ```ignore
    /// // CapsLock activates layer, HJKL become arrow keys
    /// let config = E2EConfig::with_modifier_layer(
    ///     KeyCode::CapsLock,
    ///     0,
    ///     vec![
    ///         (KeyCode::H, KeyCode::Left),
    ///         (KeyCode::J, KeyCode::Down),
    ///         (KeyCode::K, KeyCode::Up),
    ///         (KeyCode::L, KeyCode::Right),
    ///     ],
    /// );
    /// ```
    pub fn with_modifier_layer(
        modifier_key: KeyCode,
        modifier_id: u8,
        layer_mappings: Vec<(KeyCode, KeyCode)>,
    ) -> Self {
        let mut mappings = vec![KeyMapping::modifier(modifier_key, modifier_id)];

        for (from, to) in layer_mappings {
            mappings.push(KeyMapping::conditional(
                Condition::AllActive(vec![ConditionItem::ModifierActive(modifier_id)]),
                vec![BaseKeyMapping::Simple { from, to }],
            ));
        }

        Self {
            device_pattern: "*".to_string(),
            mappings,
        }
    }

    /// Creates a configuration with a lock key and conditional mappings.
    ///
    /// Similar to modifier layer but uses toggle lock instead of momentary hold.
    ///
    /// # Arguments
    ///
    /// * `lock_key` - The key that toggles the layer
    /// * `lock_id` - The lock ID for the layer
    /// * `layer_mappings` - List of (from, to) pairs active when lock is on
    ///
    /// # Example
    ///
    /// ```ignore
    /// // ScrollLock toggles layer, number row becomes F-keys
    /// let config = E2EConfig::with_lock_layer(
    ///     KeyCode::ScrollLock,
    ///     0,
    ///     vec![
    ///         (KeyCode::Num1, KeyCode::F1),
    ///         (KeyCode::Num2, KeyCode::F2),
    ///     ],
    /// );
    /// ```
    pub fn with_lock_layer(
        lock_key: KeyCode,
        lock_id: u8,
        layer_mappings: Vec<(KeyCode, KeyCode)>,
    ) -> Self {
        let mut mappings = vec![KeyMapping::lock(lock_key, lock_id)];

        for (from, to) in layer_mappings {
            mappings.push(KeyMapping::conditional(
                Condition::LockActive(lock_id),
                vec![BaseKeyMapping::Simple { from, to }],
            ));
        }

        Self {
            device_pattern: "*".to_string(),
            mappings,
        }
    }

    /// Creates a configuration with a modified output mapping.
    ///
    /// When `from` is pressed, outputs `to` with specified physical modifiers.
    ///
    /// # Arguments
    ///
    /// * `from` - Source key
    /// * `to` - Target key
    /// * `shift` - Include Shift modifier
    /// * `ctrl` - Include Ctrl modifier
    /// * `alt` - Include Alt modifier
    /// * `win` - Include Win/Meta modifier
    ///
    /// # Example
    ///
    /// ```ignore
    /// // A → Shift+1 (outputs '!')
    /// let config = E2EConfig::modified_output(
    ///     KeyCode::A,
    ///     KeyCode::Num1,
    ///     true, false, false, false,
    /// );
    /// ```
    pub fn modified_output(
        from: KeyCode,
        to: KeyCode,
        shift: bool,
        ctrl: bool,
        alt: bool,
        win: bool,
    ) -> Self {
        Self {
            device_pattern: "*".to_string(),
            mappings: vec![KeyMapping::modified_output(from, to, shift, ctrl, alt, win)],
        }
    }

    /// Creates a configuration with a tap-hold mapping.
    ///
    /// When the key is tapped (quick press and release), it outputs `tap_key`.
    /// When held beyond `threshold_ms`, it activates `hold_modifier`.
    ///
    /// # Arguments
    ///
    /// * `from` - Source key (e.g., CapsLock)
    /// * `tap_key` - Key to output on tap (e.g., Escape)
    /// * `hold_modifier` - Modifier ID to activate on hold (0-254)
    /// * `threshold_ms` - Time in milliseconds to distinguish tap from hold
    ///
    /// # Example
    ///
    /// ```ignore
    /// // CapsLock: tap=Escape, hold=Ctrl (modifier 0), 200ms threshold
    /// let config = E2EConfig::tap_hold(
    ///     KeyCode::CapsLock,
    ///     KeyCode::Escape,
    ///     0,
    ///     200,
    /// );
    /// ```
    #[allow(dead_code)]
    pub fn tap_hold(from: KeyCode, tap_key: KeyCode, hold_modifier: u8, threshold_ms: u16) -> Self {
        Self {
            device_pattern: "*".to_string(),
            mappings: vec![KeyMapping::tap_hold(
                from,
                tap_key,
                hold_modifier,
                threshold_ms,
            )],
        }
    }

    /// Creates a configuration with a tap-hold mapping and conditional layer.
    ///
    /// Combines tap-hold with a layer of conditional mappings that activate
    /// when the hold modifier is active.
    ///
    /// # Arguments
    ///
    /// * `from` - Source key for tap-hold
    /// * `tap_key` - Key to output on tap
    /// * `hold_modifier` - Modifier ID to activate on hold
    /// * `threshold_ms` - Time in milliseconds to distinguish tap from hold
    /// * `layer_mappings` - List of (from, to) pairs active when modifier is held
    ///
    /// # Example
    ///
    /// ```ignore
    /// // CapsLock: tap=Escape, hold=navigation layer with HJKL arrows
    /// let config = E2EConfig::tap_hold_with_layer(
    ///     KeyCode::CapsLock,
    ///     KeyCode::Escape,
    ///     0,
    ///     200,
    ///     vec![
    ///         (KeyCode::H, KeyCode::Left),
    ///         (KeyCode::J, KeyCode::Down),
    ///         (KeyCode::K, KeyCode::Up),
    ///         (KeyCode::L, KeyCode::Right),
    ///     ],
    /// );
    /// ```
    #[allow(dead_code)]
    pub fn tap_hold_with_layer(
        from: KeyCode,
        tap_key: KeyCode,
        hold_modifier: u8,
        threshold_ms: u16,
        layer_mappings: Vec<(KeyCode, KeyCode)>,
    ) -> Self {
        let mut mappings = vec![KeyMapping::tap_hold(
            from,
            tap_key,
            hold_modifier,
            threshold_ms,
        )];

        for (layer_from, layer_to) in layer_mappings {
            mappings.push(KeyMapping::conditional(
                Condition::AllActive(vec![ConditionItem::ModifierActive(hold_modifier)]),
                vec![BaseKeyMapping::Simple {
                    from: layer_from,
                    to: layer_to,
                }],
            ));
        }

        Self {
            device_pattern: "*".to_string(),
            mappings,
        }
    }

    /// Adds additional mappings to this configuration.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B)
    ///     .with_mappings(vec![
    ///         KeyMapping::simple(KeyCode::C, KeyCode::D),
    ///     ]);
    /// ```
    pub fn with_mappings(mut self, mappings: Vec<KeyMapping>) -> Self {
        self.mappings.extend(mappings);
        self
    }

    /// Sets the device pattern for this configuration.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B)
    ///     .with_device_pattern("USB*");
    /// ```
    pub fn with_device_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.device_pattern = pattern.into();
        self
    }

    /// Converts this E2EConfig to a ConfigRoot for serialization.
    ///
    /// This creates a complete configuration with proper version and metadata.
    pub fn to_config_root(&self) -> ConfigRoot {
        ConfigRoot {
            version: Version::current(),
            devices: vec![DeviceConfig {
                identifier: DeviceIdentifier {
                    pattern: self.device_pattern.clone(),
                },
                mappings: self.mappings.clone(),
            }],
            metadata: Metadata {
                compilation_timestamp: 0,
                compiler_version: "e2e-test".to_string(),
                source_hash: "e2e-test".to_string(),
            },
        }
    }
}

impl Default for E2EConfig {
    fn default() -> Self {
        Self {
            device_pattern: "*".to_string(),
            mappings: Vec::new(),
        }
    }
}

// ============================================================================
// TeardownResult - Result from explicit teardown
// ============================================================================

/// Result from an explicit teardown operation.
///
/// This struct provides detailed information about what happened during
/// teardown, which is useful for debugging test failures and verifying
/// cleanup behavior.
#[derive(Debug, Clone)]
pub struct TeardownResult {
    /// Whether SIGTERM was successfully sent to the daemon.
    pub sigterm_sent: bool,
    /// Whether SIGKILL was needed (daemon didn't respond to SIGTERM).
    pub sigkill_sent: bool,
    /// Whether the daemon shut down gracefully (responded to SIGTERM).
    pub graceful_shutdown: bool,
    /// The daemon's exit code, if available.
    pub exit_code: Option<i32>,
    /// Whether the config file was successfully removed.
    pub config_cleaned: bool,
    /// Any warnings that occurred during teardown.
    pub warnings: Vec<String>,
}

impl Default for TeardownResult {
    fn default() -> Self {
        Self {
            sigterm_sent: false,
            sigkill_sent: false,
            graceful_shutdown: false,
            exit_code: None,
            config_cleaned: false,
            warnings: Vec::new(),
        }
    }
}

impl TeardownResult {
    /// Returns true if teardown completed without any warnings.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.warnings.is_empty() && self.config_cleaned
    }

    /// Returns true if the daemon was forcefully killed.
    #[must_use]
    pub fn was_force_killed(&self) -> bool {
        self.sigkill_sent
    }
}

impl fmt::Display for TeardownResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Teardown Result:")?;
        writeln!(f, "  SIGTERM sent: {}", self.sigterm_sent)?;
        writeln!(f, "  SIGKILL sent: {}", self.sigkill_sent)?;
        writeln!(f, "  Graceful shutdown: {}", self.graceful_shutdown)?;
        writeln!(f, "  Exit code: {:?}", self.exit_code)?;
        writeln!(f, "  Config cleaned: {}", self.config_cleaned)?;
        if !self.warnings.is_empty() {
            writeln!(f, "  Warnings:")?;
            for warning in &self.warnings {
                writeln!(f, "    - {}", warning)?;
            }
        }
        Ok(())
    }
}

// ============================================================================
// E2EHarness - Complete E2E test orchestration
// ============================================================================

/// Default name for the daemon's virtual output device.
const DAEMON_OUTPUT_NAME: &str = "keyrx Virtual Keyboard";

/// Default timeout for waiting for daemon to be ready.
const DAEMON_STARTUP_TIMEOUT: Duration = Duration::from_secs(5);

/// Default timeout for waiting for output device to appear.
const OUTPUT_DEVICE_TIMEOUT: Duration = Duration::from_secs(5);

/// Orchestrates complete E2E test lifecycle.
///
/// This harness manages:
/// - Creation of a virtual input keyboard
/// - Generation and writing of test configuration
/// - Starting the daemon as a subprocess
/// - Finding and connecting to the daemon's output device
/// - Cleanup of all resources on drop
///
/// # Example
///
/// ```ignore
/// use keyrx_daemon::tests::e2e_harness::{E2EConfig, E2EHarness};
/// use keyrx_core::config::KeyCode;
///
/// // Create a simple A→B remapping test
/// let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
/// let harness = E2EHarness::setup(config)?;
///
/// // Harness is now ready for testing
/// // - Virtual keyboard created
/// // - Daemon running and grabbing the virtual keyboard
/// // - Output capture connected to daemon's output
/// ```
pub struct E2EHarness {
    /// Virtual keyboard for injecting test input events.
    virtual_input: VirtualKeyboard,
    /// Daemon subprocess handle.
    daemon_process: Option<Child>,
    /// Output capture for reading daemon's remapped events.
    output_capture: OutputCapture,
    /// Path to the temporary .krx config file.
    config_path: PathBuf,
    /// Captured stderr from daemon for diagnostics.
    /// This field will be populated and used by future teardown implementation.
    #[allow(dead_code)]
    daemon_stderr: Option<String>,
}

impl E2EHarness {
    /// Sets up a complete E2E test environment.
    ///
    /// This method performs the following steps:
    /// 1. Creates a VirtualKeyboard with a unique name
    /// 2. Generates a .krx config file targeting the virtual keyboard
    /// 3. Starts the daemon as a subprocess with the config
    /// 4. Waits for the daemon to grab the device and create its output
    /// 5. Finds and opens the daemon's output device
    ///
    /// # Arguments
    ///
    /// * `config` - Test configuration with mappings to apply
    ///
    /// # Returns
    ///
    /// An `E2EHarness` ready for test input/output operations.
    ///
    /// # Errors
    ///
    /// - [`E2EError::VirtualDevice`] if virtual keyboard creation fails
    /// - [`E2EError::ConfigError`] if config serialization fails
    /// - [`E2EError::DaemonStartError`] if daemon fails to start
    /// - [`E2EError::Timeout`] if daemon doesn't become ready in time
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = E2EConfig::simple_remap(KeyCode::CapsLock, KeyCode::Escape);
    /// let harness = E2EHarness::setup(config)?;
    /// ```
    pub fn setup(config: E2EConfig) -> Result<Self, E2EError> {
        Self::setup_with_timeout(config, DAEMON_STARTUP_TIMEOUT, OUTPUT_DEVICE_TIMEOUT)
    }

    /// Sets up E2E environment with custom timeouts.
    ///
    /// # Arguments
    ///
    /// * `config` - Test configuration
    /// * `daemon_timeout` - How long to wait for daemon to start
    /// * `output_timeout` - How long to wait for output device
    pub fn setup_with_timeout(
        config: E2EConfig,
        _daemon_timeout: Duration,
        output_timeout: Duration,
    ) -> Result<Self, E2EError> {
        // Step 1: Create virtual keyboard with unique name
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let input_name = format!("e2e-test-input-{}", timestamp);

        let virtual_input = VirtualKeyboard::create(&input_name)?;

        // Give the kernel a moment to register the device
        std::thread::sleep(Duration::from_millis(100));

        // Step 2: Generate .krx config file targeting the virtual keyboard
        // Modify the config to match our virtual keyboard's name
        let test_config = E2EConfig {
            device_pattern: virtual_input.name().to_string(),
            mappings: config.mappings,
        };

        let config_root = test_config.to_config_root();
        let config_bytes = serialize_config(&config_root).map_err(|e| E2EError::ConfigError {
            message: format!("failed to serialize config: {}", e),
        })?;

        // Write to temporary file
        let config_path = std::env::temp_dir().join(format!("keyrx-e2e-{}.krx", timestamp));
        let mut file = File::create(&config_path)?;
        file.write_all(&config_bytes)?;
        file.sync_all()?;

        // Step 3: Start daemon as subprocess
        let daemon_binary = Self::find_daemon_binary()?;

        let mut daemon_process = Command::new(&daemon_binary)
            .arg("run")
            .arg("--config")
            .arg(&config_path)
            .arg("--debug")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| E2EError::DaemonStartError {
                message: format!("failed to spawn daemon: {}", e),
                stderr: None,
            })?;

        // Step 4: Wait for daemon to start and grab our device
        // We do this by waiting for the output device to appear
        let start = Instant::now();

        // Give daemon a moment to initialize
        std::thread::sleep(Duration::from_millis(200));

        // Check if daemon is still running
        if let Some(status) = daemon_process.try_wait().map_err(E2EError::Io)? {
            // Daemon exited immediately - capture stderr for diagnostics
            let stderr = Self::read_child_stderr(&mut daemon_process);
            return Err(E2EError::DaemonCrashed {
                exit_code: status.code(),
                stderr,
            });
        }

        // Step 5: Find and open the daemon's output device
        let remaining_timeout = output_timeout.saturating_sub(start.elapsed());
        let mut output_capture = OutputCapture::find_by_name(DAEMON_OUTPUT_NAME, remaining_timeout)
            .map_err(|e| match e {
                VirtualDeviceError::NotFound { .. } | VirtualDeviceError::Timeout { .. } => {
                    // Daemon may have crashed - check and include stderr
                    let stderr = Self::read_child_stderr(&mut daemon_process);
                    if let Some(status) = daemon_process.try_wait().ok().flatten() {
                        E2EError::DaemonCrashed {
                            exit_code: status.code(),
                            stderr,
                        }
                    } else {
                        E2EError::Timeout {
                            operation: format!(
                                "waiting for output device '{}'",
                                DAEMON_OUTPUT_NAME
                            ),
                            timeout_ms: output_timeout.as_millis() as u64,
                        }
                    }
                }
                _ => E2EError::VirtualDevice(e),
            })?;

        // Drain any pending events from output capture
        let _ = output_capture.drain();

        Ok(Self {
            virtual_input,
            daemon_process: Some(daemon_process),
            output_capture,
            config_path,
            daemon_stderr: None,
        })
    }

    /// Returns a reference to the virtual input keyboard.
    #[must_use]
    pub fn virtual_input(&self) -> &VirtualKeyboard {
        &self.virtual_input
    }

    /// Returns a mutable reference to the virtual input keyboard.
    #[allow(dead_code)]
    pub fn virtual_input_mut(&mut self) -> &mut VirtualKeyboard {
        &mut self.virtual_input
    }

    /// Returns a reference to the output capture.
    #[allow(dead_code)]
    #[must_use]
    pub fn output_capture(&self) -> &OutputCapture {
        &self.output_capture
    }

    /// Returns a mutable reference to the output capture.
    #[allow(dead_code)]
    pub fn output_capture_mut(&mut self) -> &mut OutputCapture {
        &mut self.output_capture
    }

    /// Returns the config file path.
    #[must_use]
    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    /// Returns whether the daemon process is still running.
    #[must_use]
    pub fn is_daemon_running(&mut self) -> bool {
        if let Some(ref mut process) = self.daemon_process {
            matches!(process.try_wait(), Ok(None))
        } else {
            false
        }
    }

    // ========================================================================
    // Test Interaction Methods
    // ========================================================================

    /// Injects a sequence of key events into the virtual keyboard.
    ///
    /// This method delegates to [`VirtualKeyboard::inject_sequence`] and is the
    /// primary way to send test input through the daemon.
    ///
    /// # Arguments
    ///
    /// * `events` - Slice of key events to inject in order
    ///
    /// # Errors
    ///
    /// - [`E2EError::VirtualDevice`] if injection fails
    /// - [`E2EError::DaemonCrashed`] if the daemon has exited
    ///
    /// # Example
    ///
    /// ```ignore
    /// use keyrx_core::runtime::KeyEvent;
    /// use keyrx_core::config::KeyCode;
    ///
    /// let mut harness = E2EHarness::setup(config)?;
    ///
    /// // Inject a key tap (press + release)
    /// harness.inject(&[
    ///     KeyEvent::Press(KeyCode::A),
    ///     KeyEvent::Release(KeyCode::A),
    /// ])?;
    /// ```
    pub fn inject(&mut self, events: &[KeyEvent]) -> Result<(), E2EError> {
        // Check if daemon is still running before injection
        if !self.is_daemon_running() {
            let stderr = self
                .daemon_process
                .as_mut()
                .and_then(|p| Self::read_child_stderr(p));
            let exit_code = self
                .daemon_process
                .as_mut()
                .and_then(|p| p.try_wait().ok().flatten())
                .and_then(|s| s.code());
            return Err(E2EError::DaemonCrashed { exit_code, stderr });
        }

        self.virtual_input
            .inject_sequence(events, None)
            .map_err(E2EError::from)
    }

    /// Injects a sequence of key events with a delay between each.
    ///
    /// This is useful when you need to simulate realistic typing speed or
    /// when the daemon needs time to process events between injections.
    ///
    /// # Arguments
    ///
    /// * `events` - Slice of key events to inject in order
    /// * `delay` - Time to wait between each event
    ///
    /// # Example
    ///
    /// ```ignore
    /// use std::time::Duration;
    ///
    /// // Inject with 5ms delay between events
    /// harness.inject_with_delay(
    ///     &[KeyEvent::Press(KeyCode::A), KeyEvent::Release(KeyCode::A)],
    ///     Duration::from_millis(5),
    /// )?;
    /// ```
    pub fn inject_with_delay(
        &mut self,
        events: &[KeyEvent],
        delay: Duration,
    ) -> Result<(), E2EError> {
        // Check if daemon is still running before injection
        if !self.is_daemon_running() {
            let stderr = self
                .daemon_process
                .as_mut()
                .and_then(|p| Self::read_child_stderr(p));
            let exit_code = self
                .daemon_process
                .as_mut()
                .and_then(|p| p.try_wait().ok().flatten())
                .and_then(|s| s.code());
            return Err(E2EError::DaemonCrashed { exit_code, stderr });
        }

        self.virtual_input
            .inject_sequence(events, Some(delay))
            .map_err(E2EError::from)
    }

    /// Captures output events from the daemon with a timeout.
    ///
    /// This method delegates to [`OutputCapture::collect_events`] and collects
    /// all events that arrive within the specified timeout.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Time to wait for additional events after receiving each event.
    ///   The timeout resets after each event, so this is effectively the "idle timeout".
    ///
    /// # Returns
    ///
    /// A vector of captured events (may be empty if no events arrived within timeout).
    ///
    /// # Errors
    ///
    /// - [`E2EError::VirtualDevice`] if capture fails
    ///
    /// # Example
    ///
    /// ```ignore
    /// use std::time::Duration;
    ///
    /// // Capture events with 100ms idle timeout
    /// let events = harness.capture(Duration::from_millis(100))?;
    /// println!("Captured {} events", events.len());
    /// ```
    pub fn capture(&mut self, timeout: Duration) -> Result<Vec<KeyEvent>, E2EError> {
        self.output_capture
            .collect_events(timeout)
            .map_err(E2EError::from)
    }

    /// Captures a specific number of events with a timeout.
    ///
    /// This is useful when you know exactly how many events to expect.
    /// The method returns as soon as the expected count is reached or the
    /// timeout expires.
    ///
    /// # Arguments
    ///
    /// * `count` - Number of events to capture
    /// * `timeout` - Maximum total time to wait for all events
    ///
    /// # Returns
    ///
    /// A vector of captured events (may have fewer than `count` if timeout expires).
    ///
    /// # Errors
    ///
    /// - [`E2EError::VirtualDevice`] if capture fails
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Capture exactly 2 events (press + release)
    /// let events = harness.capture_n(2, Duration::from_millis(500))?;
    /// assert_eq!(events.len(), 2);
    /// ```
    pub fn capture_n(
        &mut self,
        count: usize,
        timeout: Duration,
    ) -> Result<Vec<KeyEvent>, E2EError> {
        let mut events = Vec::with_capacity(count);
        let start = Instant::now();

        while events.len() < count {
            let remaining = timeout.saturating_sub(start.elapsed());
            if remaining.is_zero() {
                break;
            }

            match self.output_capture.next_event(remaining)? {
                Some(event) => events.push(event),
                None => break, // Timeout
            }
        }

        Ok(events)
    }

    /// Drains any pending events from the output capture.
    ///
    /// This is useful before starting a test to ensure no stale events
    /// from previous operations affect the results.
    ///
    /// # Returns
    ///
    /// The number of events that were drained.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let drained = harness.drain()?;
    /// println!("Cleared {} stale events", drained);
    /// ```
    pub fn drain(&mut self) -> Result<usize, E2EError> {
        self.output_capture.drain().map_err(E2EError::from)
    }

    /// Injects events and captures the resulting output in one operation.
    ///
    /// This is the most common pattern for E2E testing. The method:
    /// 1. Drains any pending output events (to avoid stale data)
    /// 2. Injects the input events
    /// 3. Captures output events until the timeout expires
    ///
    /// # Arguments
    ///
    /// * `events` - Events to inject
    /// * `capture_timeout` - Time to wait for output events after injection
    ///
    /// # Returns
    ///
    /// A vector of captured output events.
    ///
    /// # Errors
    ///
    /// - [`E2EError::VirtualDevice`] if injection or capture fails
    /// - [`E2EError::DaemonCrashed`] if the daemon has exited
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Test A→B remapping
    /// let output = harness.inject_and_capture(
    ///     &[KeyEvent::Press(KeyCode::A), KeyEvent::Release(KeyCode::A)],
    ///     Duration::from_millis(100),
    /// )?;
    ///
    /// // Expect B events (if A→B remapping is configured)
    /// assert_eq!(output, vec![
    ///     KeyEvent::Press(KeyCode::B),
    ///     KeyEvent::Release(KeyCode::B),
    /// ]);
    /// ```
    pub fn inject_and_capture(
        &mut self,
        events: &[KeyEvent],
        capture_timeout: Duration,
    ) -> Result<Vec<KeyEvent>, E2EError> {
        // Drain any stale events before the test
        self.drain()?;

        // Inject the input events
        self.inject(events)?;

        // Small delay to allow events to propagate through the daemon
        std::thread::sleep(Duration::from_millis(10));

        // Capture the output
        self.capture(capture_timeout)
    }

    /// Injects events and captures a specific number of output events.
    ///
    /// Similar to [`inject_and_capture`](Self::inject_and_capture) but waits
    /// for exactly `expected_count` events instead of using an idle timeout.
    ///
    /// # Arguments
    ///
    /// * `events` - Events to inject
    /// * `expected_count` - Number of output events expected
    /// * `timeout` - Maximum time to wait for all events
    ///
    /// # Returns
    ///
    /// A vector of captured output events.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Inject 2 events, expect 2 output events
    /// let output = harness.inject_and_capture_n(
    ///     &[KeyEvent::Press(KeyCode::A), KeyEvent::Release(KeyCode::A)],
    ///     2,
    ///     Duration::from_millis(500),
    /// )?;
    /// ```
    pub fn inject_and_capture_n(
        &mut self,
        events: &[KeyEvent],
        expected_count: usize,
        timeout: Duration,
    ) -> Result<Vec<KeyEvent>, E2EError> {
        // Drain any stale events before the test
        self.drain()?;

        // Inject the input events
        self.inject(events)?;

        // Small delay to allow events to propagate through the daemon
        std::thread::sleep(Duration::from_millis(10));

        // Capture the expected number of output events
        self.capture_n(expected_count, timeout)
    }

    /// Verifies that captured events match expected events.
    ///
    /// This method compares the captured and expected events and returns
    /// a detailed error if they don't match.
    ///
    /// # Arguments
    ///
    /// * `captured` - Events that were actually captured
    /// * `expected` - Events that were expected
    ///
    /// # Returns
    ///
    /// `Ok(())` if events match, or [`E2EError::VerificationFailed`] with
    /// detailed diff information.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let captured = harness.inject_and_capture(
    ///     &[KeyEvent::Press(KeyCode::A), KeyEvent::Release(KeyCode::A)],
    ///     Duration::from_millis(100),
    /// )?;
    ///
    /// harness.verify(
    ///     &captured,
    ///     &[KeyEvent::Press(KeyCode::B), KeyEvent::Release(KeyCode::B)],
    /// )?;
    /// ```
    pub fn verify(&self, captured: &[KeyEvent], expected: &[KeyEvent]) -> Result<(), E2EError> {
        use keyrx_daemon::test_utils::compare_events;

        let result = compare_events(captured, expected);
        if result.passed {
            Ok(())
        } else {
            Err(E2EError::VerificationFailed {
                captured: captured.to_vec(),
                expected: expected.to_vec(),
                diff: result.format_diff(),
            })
        }
    }

    /// Injects events, captures output, and verifies against expected events.
    ///
    /// This is the most convenient method for E2E testing, combining
    /// injection, capture, and verification in one call.
    ///
    /// # Arguments
    ///
    /// * `input` - Events to inject
    /// * `expected` - Expected output events
    /// * `capture_timeout` - Time to wait for output events
    ///
    /// # Returns
    ///
    /// `Ok(())` if verification passes, or an error describing the failure.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Test that A is remapped to B
    /// harness.test_mapping(
    ///     &[KeyEvent::Press(KeyCode::A), KeyEvent::Release(KeyCode::A)],
    ///     &[KeyEvent::Press(KeyCode::B), KeyEvent::Release(KeyCode::B)],
    ///     Duration::from_millis(100),
    /// )?;
    /// ```
    pub fn test_mapping(
        &mut self,
        input: &[KeyEvent],
        expected: &[KeyEvent],
        capture_timeout: Duration,
    ) -> Result<(), E2EError> {
        let captured = self.inject_and_capture(input, capture_timeout)?;
        self.verify(&captured, expected)
    }

    // ========================================================================
    // Teardown and Cleanup
    // ========================================================================

    /// Gracefully tears down the E2E test environment.
    ///
    /// This method provides explicit, graceful cleanup of all test resources:
    /// 1. Sends SIGTERM to the daemon process
    /// 2. Waits for daemon to exit with a timeout
    /// 3. Sends SIGKILL if daemon doesn't respond to SIGTERM
    /// 4. Destroys the virtual keyboard
    /// 5. Removes the temporary config file
    ///
    /// Unlike the `Drop` implementation, this method:
    /// - Returns an error if cleanup fails
    /// - Provides diagnostic information about what happened
    /// - Consumes the harness, preventing further use
    ///
    /// # Returns
    ///
    /// - `Ok(TeardownResult)` with details about the cleanup
    /// - `Err(E2EError)` if critical cleanup fails
    ///
    /// # Example
    ///
    /// ```ignore
    /// let harness = E2EHarness::setup(config)?;
    /// // ... run tests ...
    ///
    /// let result = harness.teardown()?;
    /// println!("Daemon exited with code: {:?}", result.exit_code);
    /// ```
    pub fn teardown(self) -> Result<TeardownResult, E2EError> {
        self.teardown_with_timeout(Duration::from_secs(5))
    }

    /// Tears down with a custom timeout for daemon shutdown.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Maximum time to wait for daemon to exit gracefully
    pub fn teardown_with_timeout(mut self, timeout: Duration) -> Result<TeardownResult, E2EError> {
        let mut result = TeardownResult::default();

        // Step 1: Terminate daemon process
        if let Some(mut process) = self.daemon_process.take() {
            let pid = process.id();

            // Send SIGTERM for graceful shutdown
            #[cfg(unix)]
            {
                use nix::sys::signal::{kill, Signal};
                use nix::unistd::Pid;

                let nix_pid = Pid::from_raw(pid as i32);
                if let Err(e) = kill(nix_pid, Signal::SIGTERM) {
                    // Process may have already exited, which is fine
                    if e != nix::errno::Errno::ESRCH {
                        result.sigterm_sent = false;
                        result
                            .warnings
                            .push(format!("Failed to send SIGTERM: {}", e));
                    }
                } else {
                    result.sigterm_sent = true;
                }
            }

            // Wait for graceful shutdown with timeout
            let start = Instant::now();
            let poll_interval = Duration::from_millis(50);

            loop {
                match process.try_wait() {
                    Ok(Some(status)) => {
                        // Process has exited
                        result.exit_code = status.code();
                        result.graceful_shutdown = true;
                        break;
                    }
                    Ok(None) => {
                        // Still running, check timeout
                        if start.elapsed() >= timeout {
                            // Timeout - force kill
                            result.graceful_shutdown = false;

                            #[cfg(unix)]
                            {
                                use nix::sys::signal::{kill, Signal};
                                use nix::unistd::Pid;

                                let nix_pid = Pid::from_raw(pid as i32);
                                if let Err(e) = kill(nix_pid, Signal::SIGKILL) {
                                    if e != nix::errno::Errno::ESRCH {
                                        result
                                            .warnings
                                            .push(format!("Failed to send SIGKILL: {}", e));
                                    }
                                } else {
                                    result.sigkill_sent = true;
                                }
                            }

                            #[cfg(not(unix))]
                            {
                                let _ = process.kill();
                                result.sigkill_sent = true;
                            }

                            // Wait for forced termination
                            match process.wait() {
                                Ok(status) => result.exit_code = status.code(),
                                Err(e) => {
                                    result.warnings.push(format!(
                                        "Failed to wait for process after SIGKILL: {}",
                                        e
                                    ));
                                }
                            }
                            break;
                        }

                        std::thread::sleep(poll_interval);
                    }
                    Err(e) => {
                        result
                            .warnings
                            .push(format!("Error checking process status: {}", e));
                        break;
                    }
                }
            }

            // Try to read any remaining stderr for diagnostics
            self.daemon_stderr = Self::read_child_stderr(&mut process);
        }

        // Step 2: Virtual keyboard is automatically destroyed when self is dropped

        // Step 3: Remove config file
        if self.config_path.exists() {
            if let Err(e) = fs::remove_file(&self.config_path) {
                result
                    .warnings
                    .push(format!("Failed to remove config file: {}", e));
                result.config_cleaned = false;
            } else {
                result.config_cleaned = true;
            }
        } else {
            result.config_cleaned = true; // Already cleaned or never created
        }

        Ok(result)
    }

    /// Finds the daemon binary path.
    ///
    /// Looks in the following order:
    /// 1. `target/debug/keyrx_daemon` (debug build)
    /// 2. `target/release/keyrx_daemon` (release build)
    /// 3. Path from `KEYRX_DAEMON_PATH` environment variable
    fn find_daemon_binary() -> Result<PathBuf, E2EError> {
        // Check environment variable first
        if let Ok(path) = std::env::var("KEYRX_DAEMON_PATH") {
            let path = PathBuf::from(path);
            if path.exists() {
                return Ok(path);
            }
        }

        // Try workspace target directory
        // We navigate from the test crate to the workspace root
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
        let workspace_root = PathBuf::from(&manifest_dir)
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));

        // Try debug build first
        let debug_path = workspace_root.join("target/debug/keyrx_daemon");
        if debug_path.exists() {
            return Ok(debug_path);
        }

        // Try release build
        let release_path = workspace_root.join("target/release/keyrx_daemon");
        if release_path.exists() {
            return Ok(release_path);
        }

        Err(E2EError::ConfigError {
            message: format!(
                "Could not find keyrx_daemon binary. Tried:\n\
                 - {}\n\
                 - {}\n\
                 Set KEYRX_DAEMON_PATH environment variable to specify the path.",
                debug_path.display(),
                release_path.display()
            ),
        })
    }

    /// Reads stderr from the daemon process if available.
    fn read_child_stderr(child: &mut Child) -> Option<String> {
        use std::io::Read;
        child.stderr.as_mut().and_then(|stderr| {
            let mut buf = String::new();
            stderr.read_to_string(&mut buf).ok()?;
            if buf.is_empty() {
                None
            } else {
                Some(buf)
            }
        })
    }
}

impl Drop for E2EHarness {
    /// Ensures cleanup even on panic.
    ///
    /// This method performs best-effort cleanup that never panics:
    /// 1. Terminates the daemon process (SIGTERM, then SIGKILL if needed)
    /// 2. Removes the temporary config file
    /// 3. Virtual keyboard is dropped automatically
    ///
    /// For explicit cleanup with error reporting, use [`teardown`](Self::teardown).
    fn drop(&mut self) {
        // Terminate daemon process
        if let Some(mut process) = self.daemon_process.take() {
            let pid = process.id();

            // Try graceful shutdown first with SIGTERM
            #[cfg(unix)]
            {
                use nix::sys::signal::{kill, Signal};
                use nix::unistd::Pid;

                let nix_pid = Pid::from_raw(pid as i32);
                // Ignore errors - process may have already exited
                let _ = kill(nix_pid, Signal::SIGTERM);
            }

            // Wait briefly for graceful shutdown (short timeout in Drop)
            let start = Instant::now();
            let drop_timeout = Duration::from_millis(500);
            let poll_interval = Duration::from_millis(25);

            loop {
                match process.try_wait() {
                    Ok(Some(_)) => break, // Process exited
                    Ok(None) => {
                        if start.elapsed() >= drop_timeout {
                            // Timeout - force kill
                            #[cfg(unix)]
                            {
                                use nix::sys::signal::{kill, Signal};
                                use nix::unistd::Pid;

                                let nix_pid = Pid::from_raw(pid as i32);
                                let _ = kill(nix_pid, Signal::SIGKILL);
                            }

                            #[cfg(not(unix))]
                            {
                                let _ = process.kill();
                            }

                            // Wait for forced termination
                            let _ = process.wait();
                            break;
                        }
                        std::thread::sleep(poll_interval);
                    }
                    Err(_) => break, // Error checking status, give up
                }
            }
        }

        // Clean up config file (ignore errors in Drop)
        let _ = fs::remove_file(&self.config_path);

        // Note: Virtual keyboard (self.virtual_input) is automatically dropped,
        // which destroys the uinput device via its own Drop implementation.
    }
}

// ============================================================================
// TestEvents - Helper for concise test event creation
// ============================================================================

/// Helper struct for creating test events concisely.
///
/// `TestEvents` provides associated functions for creating common event patterns
/// used in E2E tests. All methods return `Vec<KeyEvent>` for compatibility with
/// [`E2EHarness::inject`].
///
/// # Example
///
/// ```ignore
/// use keyrx_daemon::tests::e2e_harness::TestEvents;
/// use keyrx_core::config::KeyCode;
///
/// // Single key tap
/// let events = TestEvents::tap(KeyCode::A);
/// assert_eq!(events.len(), 2);
///
/// // Multiple taps
/// let events = TestEvents::taps(&[KeyCode::A, KeyCode::B, KeyCode::C]);
/// assert_eq!(events.len(), 6);
///
/// // Type a word
/// let events = TestEvents::type_keys(&[KeyCode::H, KeyCode::E, KeyCode::L, KeyCode::L, KeyCode::O]);
/// ```
pub struct TestEvents;

impl TestEvents {
    /// Creates a Press event for a single key.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let events = TestEvents::press(KeyCode::A);
    /// assert_eq!(events, vec![KeyEvent::Press(KeyCode::A)]);
    /// ```
    pub fn press(key: KeyCode) -> Vec<KeyEvent> {
        vec![KeyEvent::Press(key)]
    }

    /// Creates a Release event for a single key.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let events = TestEvents::release(KeyCode::A);
    /// assert_eq!(events, vec![KeyEvent::Release(KeyCode::A)]);
    /// ```
    pub fn release(key: KeyCode) -> Vec<KeyEvent> {
        vec![KeyEvent::Release(key)]
    }

    /// Creates a complete key tap (Press + Release).
    ///
    /// This is the most common pattern for testing key remapping.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let events = TestEvents::tap(KeyCode::A);
    /// assert_eq!(events, vec![
    ///     KeyEvent::Press(KeyCode::A),
    ///     KeyEvent::Release(KeyCode::A),
    /// ]);
    /// ```
    pub fn tap(key: KeyCode) -> Vec<KeyEvent> {
        vec![KeyEvent::Press(key), KeyEvent::Release(key)]
    }

    /// Creates multiple Press events for a sequence of keys.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let events = TestEvents::presses(&[KeyCode::LShift, KeyCode::A]);
    /// assert_eq!(events, vec![
    ///     KeyEvent::Press(KeyCode::LShift),
    ///     KeyEvent::Press(KeyCode::A),
    /// ]);
    /// ```
    pub fn presses(keys: &[KeyCode]) -> Vec<KeyEvent> {
        keys.iter().map(|&k| KeyEvent::Press(k)).collect()
    }

    /// Creates multiple Release events for a sequence of keys.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let events = TestEvents::releases(&[KeyCode::A, KeyCode::LShift]);
    /// assert_eq!(events, vec![
    ///     KeyEvent::Release(KeyCode::A),
    ///     KeyEvent::Release(KeyCode::LShift),
    /// ]);
    /// ```
    pub fn releases(keys: &[KeyCode]) -> Vec<KeyEvent> {
        keys.iter().map(|&k| KeyEvent::Release(k)).collect()
    }

    /// Creates multiple key taps in sequence.
    ///
    /// Each key is pressed and released before the next key.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let events = TestEvents::taps(&[KeyCode::A, KeyCode::B]);
    /// assert_eq!(events, vec![
    ///     KeyEvent::Press(KeyCode::A),
    ///     KeyEvent::Release(KeyCode::A),
    ///     KeyEvent::Press(KeyCode::B),
    ///     KeyEvent::Release(KeyCode::B),
    /// ]);
    /// ```
    pub fn taps(keys: &[KeyCode]) -> Vec<KeyEvent> {
        keys.iter()
            .flat_map(|&k| vec![KeyEvent::Press(k), KeyEvent::Release(k)])
            .collect()
    }

    /// Creates events for typing a sequence of keys.
    ///
    /// This is an alias for [`taps`](Self::taps) with a more intuitive name
    /// for simulating keyboard typing.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Type "hello"
    /// let events = TestEvents::type_keys(&[
    ///     KeyCode::H, KeyCode::E, KeyCode::L, KeyCode::L, KeyCode::O
    /// ]);
    /// ```
    pub fn type_keys(keys: &[KeyCode]) -> Vec<KeyEvent> {
        Self::taps(keys)
    }

    /// Creates events for a modified key press (e.g., Shift+A).
    ///
    /// The modifier is pressed first, then the key is tapped, then the
    /// modifier is released. This produces the correct event sequence for
    /// modified key combinations.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Shift+A (for typing uppercase 'A')
    /// let events = TestEvents::modified(KeyCode::LShift, KeyCode::A);
    /// assert_eq!(events, vec![
    ///     KeyEvent::Press(KeyCode::LShift),
    ///     KeyEvent::Press(KeyCode::A),
    ///     KeyEvent::Release(KeyCode::A),
    ///     KeyEvent::Release(KeyCode::LShift),
    /// ]);
    /// ```
    pub fn modified(modifier: KeyCode, key: KeyCode) -> Vec<KeyEvent> {
        vec![
            KeyEvent::Press(modifier),
            KeyEvent::Press(key),
            KeyEvent::Release(key),
            KeyEvent::Release(modifier),
        ]
    }

    /// Creates events for a key press with multiple modifiers.
    ///
    /// Modifiers are pressed in order, then the key is tapped, then modifiers
    /// are released in reverse order.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Ctrl+Shift+C
    /// let events = TestEvents::with_modifiers(
    ///     &[KeyCode::LCtrl, KeyCode::LShift],
    ///     KeyCode::C,
    /// );
    /// assert_eq!(events, vec![
    ///     KeyEvent::Press(KeyCode::LCtrl),
    ///     KeyEvent::Press(KeyCode::LShift),
    ///     KeyEvent::Press(KeyCode::C),
    ///     KeyEvent::Release(KeyCode::C),
    ///     KeyEvent::Release(KeyCode::LShift),
    ///     KeyEvent::Release(KeyCode::LCtrl),
    /// ]);
    /// ```
    pub fn with_modifiers(modifiers: &[KeyCode], key: KeyCode) -> Vec<KeyEvent> {
        let mut events = Vec::with_capacity(modifiers.len() * 2 + 2);

        // Press modifiers in order
        for &modifier in modifiers {
            events.push(KeyEvent::Press(modifier));
        }

        // Tap the key
        events.push(KeyEvent::Press(key));
        events.push(KeyEvent::Release(key));

        // Release modifiers in reverse order
        for &modifier in modifiers.iter().rev() {
            events.push(KeyEvent::Release(modifier));
        }

        events
    }

    /// Creates events for holding a modifier while typing multiple keys.
    ///
    /// The modifier is held down while all keys are tapped in sequence,
    /// then the modifier is released.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Shift held while typing "ABC"
    /// let events = TestEvents::hold_while_typing(
    ///     KeyCode::LShift,
    ///     &[KeyCode::A, KeyCode::B, KeyCode::C],
    /// );
    /// // Produces: Press(LShift), Press(A), Release(A), Press(B), Release(B), ...
    /// ```
    pub fn hold_while_typing(modifier: KeyCode, keys: &[KeyCode]) -> Vec<KeyEvent> {
        let mut events = Vec::with_capacity(keys.len() * 2 + 2);

        // Press modifier
        events.push(KeyEvent::Press(modifier));

        // Tap each key
        for &key in keys {
            events.push(KeyEvent::Press(key));
            events.push(KeyEvent::Release(key));
        }

        // Release modifier
        events.push(KeyEvent::Release(modifier));

        events
    }

    /// Creates events from raw event data for custom patterns.
    ///
    /// This is useful when you need a specific event sequence that doesn't
    /// fit the other helper methods.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Custom pattern: press A, press B, release A, release B
    /// let events = TestEvents::from_events(&[
    ///     KeyEvent::Press(KeyCode::A),
    ///     KeyEvent::Press(KeyCode::B),
    ///     KeyEvent::Release(KeyCode::A),
    ///     KeyEvent::Release(KeyCode::B),
    /// ]);
    /// ```
    pub fn from_events(events: &[KeyEvent]) -> Vec<KeyEvent> {
        events.to_vec()
    }

    /// Creates an empty event sequence.
    ///
    /// Useful for testing scenarios where no events are expected.
    pub fn empty() -> Vec<KeyEvent> {
        Vec::new()
    }
}

// ============================================================================
// Test Timeout Handling
// ============================================================================

/// Default timeout for E2E tests (30 seconds).
///
/// This is generous to allow for slow CI environments while still catching
/// genuinely hung tests. Tests can override this with custom timeouts.
pub const DEFAULT_TEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Result type for timeout-wrapped test functions.
pub type TimeoutResult<T> = Result<T, E2EError>;

/// Runs a test function with a timeout, ensuring cleanup on timeout or panic.
///
/// This wrapper function provides:
/// - A configurable timeout for the entire test
/// - Automatic cleanup of the harness on timeout
/// - Phase tracking for diagnostic messages
/// - Proper error reporting with context
///
/// # Arguments
///
/// * `timeout` - Maximum duration allowed for the test
/// * `test_fn` - The test function to run. Receives the harness and phase setter.
///
/// # Type Parameters
///
/// * `F` - The test function type
/// * `T` - The return type of the test function
///
/// # Returns
///
/// The result of the test function, or a [`E2EError::TestTimeout`] if the
/// test exceeds the timeout.
///
/// # Example
///
/// ```ignore
/// use keyrx_daemon::tests::e2e_harness::{with_timeout, E2EConfig, DEFAULT_TEST_TIMEOUT};
///
/// #[test]
/// fn test_with_timeout_wrapper() {
///     let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
///
///     let result = with_timeout(DEFAULT_TEST_TIMEOUT, config, |harness, set_phase| {
///         set_phase(TestTimeoutPhase::TestLogic);
///
///         // Your test logic here
///         let captured = harness.inject_and_capture(
///             &TestEvents::tap(KeyCode::A),
///             Duration::from_millis(100),
///         )?;
///
///         harness.verify(&captured, &TestEvents::tap(KeyCode::B))?;
///         Ok(())
///     });
///
///     result.expect("Test should pass");
/// }
/// ```
///
/// # Notes
///
/// The test function runs in a separate thread to enable timeout detection.
/// If the test panics, the panic will be propagated to the calling thread
/// after cleanup is performed.
pub fn with_timeout<F, T>(timeout: Duration, config: E2EConfig, test_fn: F) -> TimeoutResult<T>
where
    F: FnOnce(&mut E2EHarness, &dyn Fn(TestTimeoutPhase)) -> Result<T, E2EError> + Send + 'static,
    T: Send + 'static,
{
    let start = Instant::now();

    // Shared phase tracking (using atomic operations via channel)
    let (phase_tx, phase_rx) = mpsc::channel::<TestTimeoutPhase>();

    // Spawn the test in a separate thread
    let handle = thread::spawn(move || {
        // Phase setter function
        let set_phase = |phase: TestTimeoutPhase| {
            let _ = phase_tx.send(phase);
        };

        // Setup phase
        set_phase(TestTimeoutPhase::Setup);
        let harness_result = E2EHarness::setup(config);

        match harness_result {
            Ok(mut harness) => {
                // Run the test function
                let test_result = test_fn(&mut harness, &set_phase);

                // Teardown phase (cleanup happens in Drop if we don't call teardown explicitly)
                set_phase(TestTimeoutPhase::Teardown);

                // Return both the result and harness for cleanup
                (Some(harness), test_result)
            }
            Err(e) => (None, Err(e)),
        }
    });

    // Wait for the test with timeout, polling for phase updates
    let mut last_phase = TestTimeoutPhase::Setup;
    let poll_interval = Duration::from_millis(100);

    loop {
        // Check for phase updates (non-blocking)
        while let Ok(phase) = phase_rx.try_recv() {
            last_phase = phase;
        }

        // Check if thread is done
        if handle.is_finished() {
            break;
        }

        // Check timeout
        let elapsed = start.elapsed();
        if elapsed >= timeout {
            // Timeout occurred - the thread will continue running but we return an error
            // The harness Drop implementation will handle cleanup when the thread eventually
            // finishes or is terminated
            return Err(E2EError::TestTimeout {
                phase: last_phase,
                timeout,
                elapsed,
                context: format!(
                    "Test hung during {} phase. The daemon process will be cleaned up.",
                    last_phase
                ),
            });
        }

        thread::sleep(poll_interval);
    }

    // Thread finished - get the result
    match handle.join() {
        Ok((harness_opt, result)) => {
            // Explicit teardown if harness exists
            if let Some(harness) = harness_opt {
                // Attempt graceful teardown, ignore errors (cleanup is best-effort)
                let _ = harness.teardown();
            }
            result
        }
        Err(panic_payload) => {
            // Thread panicked - re-panic with the original payload
            std::panic::resume_unwind(panic_payload)
        }
    }
}

/// Runs a test function with the default timeout.
///
/// This is a convenience wrapper around [`with_timeout`] using
/// [`DEFAULT_TEST_TIMEOUT`] (30 seconds).
///
/// # Example
///
/// ```ignore
/// use keyrx_daemon::tests::e2e_harness::{with_default_timeout, E2EConfig};
///
/// #[test]
/// fn test_simple_remap() {
///     let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
///
///     with_default_timeout(config, |harness, _| {
///         harness.test_mapping(
///             &TestEvents::tap(KeyCode::A),
///             &TestEvents::tap(KeyCode::B),
///             Duration::from_millis(100),
///         )
///     }).expect("Test should pass");
/// }
/// ```
pub fn with_default_timeout<F, T>(config: E2EConfig, test_fn: F) -> TimeoutResult<T>
where
    F: FnOnce(&mut E2EHarness, &dyn Fn(TestTimeoutPhase)) -> Result<T, E2EError> + Send + 'static,
    T: Send + 'static,
{
    with_timeout(DEFAULT_TEST_TIMEOUT, config, test_fn)
}

/// A simpler timeout wrapper that doesn't require phase tracking.
///
/// This is useful for tests that don't need fine-grained phase information.
/// The phase will be set to [`TestTimeoutPhase::TestLogic`] for the duration
/// of the test function.
///
/// # Example
///
/// ```ignore
/// use keyrx_daemon::tests::e2e_harness::{run_test_with_timeout, E2EConfig};
///
/// #[test]
/// fn test_simple() {
///     let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
///
///     run_test_with_timeout(Duration::from_secs(10), config, |harness| {
///         harness.test_mapping(
///             &TestEvents::tap(KeyCode::A),
///             &TestEvents::tap(KeyCode::B),
///             Duration::from_millis(100),
///         )
///     }).expect("Test should pass");
/// }
/// ```
pub fn run_test_with_timeout<F, T>(
    timeout: Duration,
    config: E2EConfig,
    test_fn: F,
) -> TimeoutResult<T>
where
    F: FnOnce(&mut E2EHarness) -> Result<T, E2EError> + Send + 'static,
    T: Send + 'static,
{
    with_timeout(timeout, config, |harness, set_phase| {
        set_phase(TestTimeoutPhase::TestLogic);
        test_fn(harness)
    })
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------------
    // E2EError Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_e2e_error_display_virtual_device() {
        let vd_err = VirtualDeviceError::uinput_permission_denied();
        let err = E2EError::from(vd_err);
        let msg = err.to_string();
        assert!(msg.contains("virtual device error"));
        assert!(msg.contains("permission denied"));
    }

    #[test]
    fn test_e2e_error_display_config_error() {
        let err = E2EError::ConfigError {
            message: "invalid mapping".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("config error"));
        assert!(msg.contains("invalid mapping"));
    }

    #[test]
    fn test_e2e_error_display_daemon_start() {
        let err = E2EError::DaemonStartError {
            message: "failed to spawn".to_string(),
            stderr: Some("permission denied".to_string()),
        };
        let msg = err.to_string();
        assert!(msg.contains("daemon start error"));
        assert!(msg.contains("failed to spawn"));
        assert!(msg.contains("permission denied"));
    }

    #[test]
    fn test_e2e_error_display_daemon_crashed() {
        let err = E2EError::DaemonCrashed {
            exit_code: Some(1),
            stderr: Some("segfault".to_string()),
        };
        let msg = err.to_string();
        assert!(msg.contains("daemon crashed"));
        assert!(msg.contains("exit code 1"));
        assert!(msg.contains("segfault"));
    }

    #[test]
    fn test_e2e_error_display_verification_failed() {
        let err = E2EError::VerificationFailed {
            captured: vec![KeyEvent::Press(KeyCode::A)],
            expected: vec![KeyEvent::Press(KeyCode::B)],
            diff: "line 1: expected B, got A".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("verification failed"));
        assert!(msg.contains("expected 1 event"));
        assert!(msg.contains("captured 1 event"));
    }

    #[test]
    fn test_e2e_error_display_timeout() {
        let err = E2EError::Timeout {
            operation: "event capture".to_string(),
            timeout_ms: 5000,
        };
        let msg = err.to_string();
        assert!(msg.contains("timeout"));
        assert!(msg.contains("5000ms"));
        assert!(msg.contains("event capture"));
    }

    #[test]
    fn test_e2e_error_display_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = E2EError::from(io_err);
        let msg = err.to_string();
        assert!(msg.contains("I/O error"));
        assert!(msg.contains("file not found"));
    }

    // ------------------------------------------------------------------------
    // E2EConfig Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_e2e_config_simple_remap() {
        let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
        assert_eq!(config.device_pattern, "*");
        assert_eq!(config.mappings.len(), 1);

        match &config.mappings[0] {
            KeyMapping::Base(BaseKeyMapping::Simple { from, to }) => {
                assert_eq!(*from, KeyCode::A);
                assert_eq!(*to, KeyCode::B);
            }
            _ => panic!("Expected Simple mapping"),
        }
    }

    #[test]
    fn test_e2e_config_simple_remaps() {
        let config =
            E2EConfig::simple_remaps(vec![(KeyCode::A, KeyCode::B), (KeyCode::C, KeyCode::D)]);
        assert_eq!(config.mappings.len(), 2);
    }

    #[test]
    fn test_e2e_config_modifier() {
        let config = E2EConfig::modifier(KeyCode::CapsLock, 0);
        assert_eq!(config.mappings.len(), 1);

        match &config.mappings[0] {
            KeyMapping::Base(BaseKeyMapping::Modifier { from, modifier_id }) => {
                assert_eq!(*from, KeyCode::CapsLock);
                assert_eq!(*modifier_id, 0);
            }
            _ => panic!("Expected Modifier mapping"),
        }
    }

    #[test]
    fn test_e2e_config_lock() {
        let config = E2EConfig::lock(KeyCode::ScrollLock, 1);
        assert_eq!(config.mappings.len(), 1);

        match &config.mappings[0] {
            KeyMapping::Base(BaseKeyMapping::Lock { from, lock_id }) => {
                assert_eq!(*from, KeyCode::ScrollLock);
                assert_eq!(*lock_id, 1);
            }
            _ => panic!("Expected Lock mapping"),
        }
    }

    #[test]
    fn test_e2e_config_conditional() {
        let config = E2EConfig::conditional(0, KeyCode::H, KeyCode::Left);
        assert_eq!(config.mappings.len(), 1);

        match &config.mappings[0] {
            KeyMapping::Conditional {
                condition,
                mappings,
            } => {
                assert!(matches!(condition, Condition::ModifierActive(0)));
                assert_eq!(mappings.len(), 1);
                match &mappings[0] {
                    BaseKeyMapping::Simple { from, to } => {
                        assert_eq!(*from, KeyCode::H);
                        assert_eq!(*to, KeyCode::Left);
                    }
                    _ => panic!("Expected Simple base mapping"),
                }
            }
            _ => panic!("Expected Conditional mapping"),
        }
    }

    #[test]
    fn test_e2e_config_with_modifier_layer() {
        let config = E2EConfig::with_modifier_layer(
            KeyCode::CapsLock,
            0,
            vec![(KeyCode::H, KeyCode::Left), (KeyCode::J, KeyCode::Down)],
        );

        // Should have: 1 modifier + 2 conditional mappings
        assert_eq!(config.mappings.len(), 3);

        // First mapping should be the modifier
        match &config.mappings[0] {
            KeyMapping::Base(BaseKeyMapping::Modifier { from, modifier_id }) => {
                assert_eq!(*from, KeyCode::CapsLock);
                assert_eq!(*modifier_id, 0);
            }
            _ => panic!("Expected Modifier as first mapping"),
        }

        // Second and third should be conditional
        assert!(matches!(
            &config.mappings[1],
            KeyMapping::Conditional { .. }
        ));
        assert!(matches!(
            &config.mappings[2],
            KeyMapping::Conditional { .. }
        ));
    }

    #[test]
    fn test_e2e_config_with_lock_layer() {
        let config = E2EConfig::with_lock_layer(
            KeyCode::ScrollLock,
            0,
            vec![(KeyCode::Num1, KeyCode::F1), (KeyCode::Num2, KeyCode::F2)],
        );

        // Should have: 1 lock + 2 conditional mappings
        assert_eq!(config.mappings.len(), 3);

        // First mapping should be the lock
        match &config.mappings[0] {
            KeyMapping::Base(BaseKeyMapping::Lock { from, lock_id }) => {
                assert_eq!(*from, KeyCode::ScrollLock);
                assert_eq!(*lock_id, 0);
            }
            _ => panic!("Expected Lock as first mapping"),
        }
    }

    #[test]
    fn test_e2e_config_modified_output() {
        let config = E2EConfig::modified_output(
            KeyCode::A,
            KeyCode::Num1,
            true,  // shift
            false, // ctrl
            false, // alt
            false, // win
        );

        assert_eq!(config.mappings.len(), 1);

        match &config.mappings[0] {
            KeyMapping::Base(BaseKeyMapping::ModifiedOutput {
                from,
                to,
                shift,
                ctrl,
                alt,
                win,
            }) => {
                assert_eq!(*from, KeyCode::A);
                assert_eq!(*to, KeyCode::Num1);
                assert!(*shift);
                assert!(!*ctrl);
                assert!(!*alt);
                assert!(!*win);
            }
            _ => panic!("Expected ModifiedOutput mapping"),
        }
    }

    #[test]
    fn test_e2e_config_with_mappings() {
        let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B)
            .with_mappings(vec![KeyMapping::simple(KeyCode::C, KeyCode::D)]);

        assert_eq!(config.mappings.len(), 2);
    }

    #[test]
    fn test_e2e_config_with_device_pattern() {
        let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B).with_device_pattern("USB*");

        assert_eq!(config.device_pattern, "USB*");
    }

    #[test]
    fn test_e2e_config_to_config_root() {
        let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
        let root = config.to_config_root();

        assert_eq!(root.version, Version::current());
        assert_eq!(root.devices.len(), 1);
        assert_eq!(root.devices[0].identifier.pattern, "*");
        assert_eq!(root.devices[0].mappings.len(), 1);
        assert_eq!(root.metadata.compiler_version, "e2e-test");
    }

    #[test]
    fn test_e2e_config_default() {
        let config = E2EConfig::default();
        assert_eq!(config.device_pattern, "*");
        assert!(config.mappings.is_empty());
    }

    // ------------------------------------------------------------------------
    // E2EHarness Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_daemon_output_name_constant() {
        // Verify the constant matches what the daemon creates
        assert_eq!(DAEMON_OUTPUT_NAME, "keyrx Virtual Keyboard");
    }

    #[test]
    fn test_daemon_startup_timeout_is_reasonable() {
        // Timeout should be between 1 and 30 seconds
        assert!(DAEMON_STARTUP_TIMEOUT >= Duration::from_secs(1));
        assert!(DAEMON_STARTUP_TIMEOUT <= Duration::from_secs(30));
    }

    #[test]
    fn test_output_device_timeout_is_reasonable() {
        // Timeout should be between 1 and 30 seconds
        assert!(OUTPUT_DEVICE_TIMEOUT >= Duration::from_secs(1));
        assert!(OUTPUT_DEVICE_TIMEOUT <= Duration::from_secs(30));
    }

    #[test]
    fn test_find_daemon_binary_uses_environment_variable() {
        // Set up a fake path via environment variable
        // This won't find the binary but tests the lookup logic
        std::env::set_var("KEYRX_DAEMON_PATH", "/nonexistent/path/keyrx_daemon");

        let result = E2EHarness::find_daemon_binary();

        // Clean up
        std::env::remove_var("KEYRX_DAEMON_PATH");

        // Should fail because the path doesn't exist
        // But it should try the environment variable first
        // Since it doesn't exist, it falls through to trying workspace paths
        // The test verifies the function exists and returns a Result
        match result {
            Ok(path) => {
                // If we found a binary, it should exist
                assert!(path.exists());
            }
            Err(E2EError::ConfigError { message }) => {
                // Expected when no binary is available
                assert!(message.contains("Could not find keyrx_daemon binary"));
            }
            Err(e) => {
                panic!("Unexpected error type: {:?}", e);
            }
        }
    }

    /// Test that E2EHarness::setup creates all components correctly
    #[test]
    fn test_e2e_harness_setup() {
        keyrx_daemon::skip_if_no_uinput!();
        let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
        let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

        // Verify components are initialized
        assert!(!harness.virtual_input().name().is_empty());
        assert!(harness
            .virtual_input()
            .name()
            .starts_with("e2e-test-input-"));
        assert!(harness.config_path().exists());
        assert!(harness.is_daemon_running());

        // Harness will be cleaned up on drop
    }

    /// Test that E2EHarness cleanup works on drop
    #[test]
    fn test_e2e_harness_cleanup() {
        keyrx_daemon::skip_if_no_uinput!();
        let config_path;

        {
            let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
            let harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

            config_path = harness.config_path().clone();
            assert!(config_path.exists(), "Config file should exist during test");

            // Harness is dropped here
        }

        // After drop, config file should be cleaned up
        assert!(
            !config_path.exists(),
            "Config file should be removed after drop"
        );
    }

    /// Test that E2EHarness fails gracefully when daemon binary is missing
    #[test]
    fn test_e2e_harness_setup_fails_without_binary() {
        // Set environment to a nonexistent path to ensure binary isn't found
        let original_path = std::env::var("KEYRX_DAEMON_PATH").ok();
        std::env::set_var("KEYRX_DAEMON_PATH", "/definitely/nonexistent/keyrx_daemon");

        let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);

        // Setup should fail either with VirtualDevice error (no uinput access)
        // or ConfigError (no daemon binary)
        let result = E2EHarness::setup(config);

        // Restore environment
        match original_path {
            Some(path) => std::env::set_var("KEYRX_DAEMON_PATH", path),
            None => std::env::remove_var("KEYRX_DAEMON_PATH"),
        }

        // Should be an error
        assert!(result.is_err());
    }

    // ------------------------------------------------------------------------
    // E2EHarness Inject/Capture/Verify Method Tests
    // ------------------------------------------------------------------------

    /// Test that inject, capture, and verify methods can be called on harness
    #[test]
    fn test_e2e_harness_inject_capture_verify() {
        keyrx_daemon::skip_if_no_uinput!();
        let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
        let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

        // Inject A key press and release
        let input = vec![KeyEvent::Press(KeyCode::A), KeyEvent::Release(KeyCode::A)];

        let captured = harness
            .inject_and_capture(&input, Duration::from_millis(500))
            .expect("Failed to inject and capture");

        // Expect B events due to A→B remapping
        let expected = vec![KeyEvent::Press(KeyCode::B), KeyEvent::Release(KeyCode::B)];

        harness
            .verify(&captured, &expected)
            .expect("Verification should pass for A→B remap");
    }

    /// Test the test_mapping convenience method
    #[test]
    fn test_e2e_harness_test_mapping() {
        keyrx_daemon::skip_if_no_uinput!();
        let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
        let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

        // Test the mapping in one call
        harness
            .test_mapping(
                &[KeyEvent::Press(KeyCode::A), KeyEvent::Release(KeyCode::A)],
                &[KeyEvent::Press(KeyCode::B), KeyEvent::Release(KeyCode::B)],
                Duration::from_millis(500),
            )
            .expect("Test mapping should pass");
    }

    /// Test inject_with_delay method
    #[test]
    fn test_e2e_harness_inject_with_delay() {
        keyrx_daemon::skip_if_no_uinput!();
        let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
        let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

        // Drain any pending events
        harness.drain().expect("Failed to drain events");

        // Inject with delay
        let start = Instant::now();
        harness
            .inject_with_delay(
                &[
                    KeyEvent::Press(KeyCode::A),
                    KeyEvent::Release(KeyCode::A),
                    KeyEvent::Press(KeyCode::A),
                    KeyEvent::Release(KeyCode::A),
                ],
                Duration::from_millis(10),
            )
            .expect("Failed to inject with delay");
        let elapsed = start.elapsed();

        // Should have taken at least 30ms (3 delays of 10ms each)
        // Allow some tolerance for scheduling
        assert!(
            elapsed >= Duration::from_millis(20),
            "Inject with delay should have taken at least 20ms, took {:?}",
            elapsed
        );
    }

    /// Test capture_n method
    #[test]
    fn test_e2e_harness_capture_n() {
        keyrx_daemon::skip_if_no_uinput!();
        let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
        let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

        // Drain any pending events
        harness.drain().expect("Failed to drain events");

        // Inject 4 events (2 key taps)
        harness
            .inject(&[
                KeyEvent::Press(KeyCode::A),
                KeyEvent::Release(KeyCode::A),
                KeyEvent::Press(KeyCode::A),
                KeyEvent::Release(KeyCode::A),
            ])
            .expect("Failed to inject events");

        // Capture exactly 4 events
        let captured = harness
            .capture_n(4, Duration::from_millis(500))
            .expect("Failed to capture_n events");

        assert_eq!(captured.len(), 4, "Should have captured 4 events");
    }

    /// Test inject_and_capture_n method
    #[test]
    fn test_e2e_harness_inject_and_capture_n() {
        keyrx_daemon::skip_if_no_uinput!();
        let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
        let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

        // Inject and capture exactly 2 events
        let captured = harness
            .inject_and_capture_n(
                &[KeyEvent::Press(KeyCode::A), KeyEvent::Release(KeyCode::A)],
                2,
                Duration::from_millis(500),
            )
            .expect("Failed to inject and capture_n");

        assert_eq!(captured.len(), 2, "Should have captured 2 events");
        assert_eq!(captured[0], KeyEvent::Press(KeyCode::B));
        assert_eq!(captured[1], KeyEvent::Release(KeyCode::B));
    }

    /// Test drain method
    #[test]
    fn test_e2e_harness_drain() {
        keyrx_daemon::skip_if_no_uinput!();
        let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
        let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

        // Inject some events
        harness
            .inject(&[KeyEvent::Press(KeyCode::A), KeyEvent::Release(KeyCode::A)])
            .expect("Failed to inject events");

        // Wait a bit for events to propagate
        std::thread::sleep(Duration::from_millis(50));

        // Drain should clear them
        let drained = harness.drain().expect("Failed to drain events");
        assert!(drained > 0, "Should have drained some events");

        // Now capture should return empty
        let captured = harness
            .capture(Duration::from_millis(50))
            .expect("Failed to capture events");
        assert!(captured.is_empty(), "Should have no events after drain");
    }

    /// Test verify returns error on mismatch
    #[test]
    fn test_e2e_harness_verify_mismatch() {
        keyrx_daemon::skip_if_no_uinput!();
        let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
        let harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

        // Verify with mismatched events
        let captured = vec![KeyEvent::Press(KeyCode::B), KeyEvent::Release(KeyCode::B)];
        let expected = vec![KeyEvent::Press(KeyCode::C), KeyEvent::Release(KeyCode::C)];

        let result = harness.verify(&captured, &expected);
        assert!(result.is_err(), "Should fail verification");

        match result {
            Err(E2EError::VerificationFailed { diff, .. }) => {
                assert!(diff.contains("FAILED"), "Diff should show failure");
            }
            _ => panic!("Expected VerificationFailed error"),
        }
    }

    /// Test passthrough behavior for unmapped keys
    #[test]
    fn test_e2e_harness_passthrough() {
        keyrx_daemon::skip_if_no_uinput!();
        let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
        let mut harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

        // Inject C key (not mapped) - should pass through unchanged
        let captured = harness
            .inject_and_capture(
                &[KeyEvent::Press(KeyCode::C), KeyEvent::Release(KeyCode::C)],
                Duration::from_millis(500),
            )
            .expect("Failed to inject and capture");

        // Should get C events (passthrough)
        harness
            .verify(
                &captured,
                &[KeyEvent::Press(KeyCode::C), KeyEvent::Release(KeyCode::C)],
            )
            .expect("Unmapped key should pass through unchanged");
    }

    // ------------------------------------------------------------------------
    // TeardownResult Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_teardown_result_default() {
        let result = TeardownResult::default();
        assert!(!result.sigterm_sent);
        assert!(!result.sigkill_sent);
        assert!(!result.graceful_shutdown);
        assert!(result.exit_code.is_none());
        assert!(!result.config_cleaned);
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_teardown_result_is_clean() {
        // Clean result has no warnings and config cleaned
        let mut result = TeardownResult::default();
        result.config_cleaned = true;
        assert!(result.is_clean());

        // Add warning - no longer clean
        result.warnings.push("some warning".to_string());
        assert!(!result.is_clean());

        // Remove warning but config not cleaned - not clean
        result.warnings.clear();
        result.config_cleaned = false;
        assert!(!result.is_clean());
    }

    #[test]
    fn test_teardown_result_was_force_killed() {
        let mut result = TeardownResult::default();
        assert!(!result.was_force_killed());

        result.sigkill_sent = true;
        assert!(result.was_force_killed());
    }

    #[test]
    fn test_teardown_result_display() {
        let mut result = TeardownResult {
            sigterm_sent: true,
            sigkill_sent: false,
            graceful_shutdown: true,
            exit_code: Some(0),
            config_cleaned: true,
            warnings: vec![],
        };

        let display = result.to_string();
        assert!(display.contains("SIGTERM sent: true"));
        assert!(display.contains("SIGKILL sent: false"));
        assert!(display.contains("Graceful shutdown: true"));
        assert!(display.contains("Exit code: Some(0)"));
        assert!(display.contains("Config cleaned: true"));

        // Add warnings and check they appear
        result.warnings.push("test warning".to_string());
        let display = result.to_string();
        assert!(display.contains("Warnings:"));
        assert!(display.contains("test warning"));
    }

    #[test]
    fn test_teardown_result_graceful_vs_forced() {
        // Simulate graceful shutdown
        let graceful = TeardownResult {
            sigterm_sent: true,
            sigkill_sent: false,
            graceful_shutdown: true,
            exit_code: Some(0),
            config_cleaned: true,
            warnings: vec![],
        };
        assert!(graceful.graceful_shutdown);
        assert!(!graceful.was_force_killed());

        // Simulate forced shutdown
        let forced = TeardownResult {
            sigterm_sent: true,
            sigkill_sent: true,
            graceful_shutdown: false,
            exit_code: Some(137), // SIGKILL exit code
            config_cleaned: true,
            warnings: vec![],
        };
        assert!(!forced.graceful_shutdown);
        assert!(forced.was_force_killed());
    }

    /// Test that explicit teardown works correctly
    #[test]
    fn test_e2e_harness_explicit_teardown() {
        keyrx_daemon::skip_if_no_uinput!();
        let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
        let harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

        let config_path = harness.config_path().clone();
        assert!(
            config_path.exists(),
            "Config file should exist before teardown"
        );

        let result = harness.teardown().expect("Teardown should succeed");

        // Verify teardown results
        assert!(result.sigterm_sent, "SIGTERM should have been sent");
        assert!(
            result.config_cleaned,
            "Config file should have been cleaned"
        );
        assert!(
            !config_path.exists(),
            "Config file should not exist after teardown"
        );

        // Print result for manual verification
        println!("{}", result);
    }

    /// Test teardown with custom timeout
    #[test]
    fn test_e2e_harness_teardown_with_timeout() {
        keyrx_daemon::skip_if_no_uinput!();
        let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
        let harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

        // Use a short timeout - daemon should still shut down gracefully
        let result = harness
            .teardown_with_timeout(Duration::from_secs(2))
            .expect("Teardown should succeed");

        assert!(result.sigterm_sent);
        // If graceful shutdown worked, SIGKILL should not have been needed
        if result.graceful_shutdown {
            assert!(!result.sigkill_sent);
        }
    }

    /// Test that Drop cleans up properly when harness is simply dropped
    #[test]
    fn test_e2e_harness_drop_cleanup() {
        keyrx_daemon::skip_if_no_uinput!();
        let config_path;

        {
            let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
            let harness = E2EHarness::setup(config).expect("Failed to setup E2E harness");

            config_path = harness.config_path().clone();
            assert!(config_path.exists(), "Config file should exist during test");

            // Harness is dropped here
        }

        // Small delay to ensure Drop has completed
        std::thread::sleep(Duration::from_millis(100));

        // After drop, config file should be cleaned up
        assert!(
            !config_path.exists(),
            "Config file should be removed after drop"
        );
    }

    // ------------------------------------------------------------------------
    // TestEvents Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_test_events_press() {
        let events = TestEvents::press(KeyCode::A);
        assert_eq!(events, vec![KeyEvent::Press(KeyCode::A)]);
    }

    #[test]
    fn test_test_events_release() {
        let events = TestEvents::release(KeyCode::A);
        assert_eq!(events, vec![KeyEvent::Release(KeyCode::A)]);
    }

    #[test]
    fn test_test_events_tap() {
        let events = TestEvents::tap(KeyCode::A);
        assert_eq!(
            events,
            vec![KeyEvent::Press(KeyCode::A), KeyEvent::Release(KeyCode::A),]
        );
    }

    #[test]
    fn test_test_events_presses() {
        let events = TestEvents::presses(&[KeyCode::LShift, KeyCode::A]);
        assert_eq!(
            events,
            vec![
                KeyEvent::Press(KeyCode::LShift),
                KeyEvent::Press(KeyCode::A),
            ]
        );
    }

    #[test]
    fn test_test_events_releases() {
        let events = TestEvents::releases(&[KeyCode::A, KeyCode::LShift]);
        assert_eq!(
            events,
            vec![
                KeyEvent::Release(KeyCode::A),
                KeyEvent::Release(KeyCode::LShift),
            ]
        );
    }

    #[test]
    fn test_test_events_taps() {
        let events = TestEvents::taps(&[KeyCode::A, KeyCode::B]);
        assert_eq!(
            events,
            vec![
                KeyEvent::Press(KeyCode::A),
                KeyEvent::Release(KeyCode::A),
                KeyEvent::Press(KeyCode::B),
                KeyEvent::Release(KeyCode::B),
            ]
        );
    }

    #[test]
    fn test_test_events_type_keys() {
        // type_keys is an alias for taps
        let events =
            TestEvents::type_keys(&[KeyCode::H, KeyCode::E, KeyCode::L, KeyCode::L, KeyCode::O]);
        assert_eq!(events.len(), 10); // 5 keys * 2 events each

        // Verify first key
        assert_eq!(events[0], KeyEvent::Press(KeyCode::H));
        assert_eq!(events[1], KeyEvent::Release(KeyCode::H));

        // Verify last key
        assert_eq!(events[8], KeyEvent::Press(KeyCode::O));
        assert_eq!(events[9], KeyEvent::Release(KeyCode::O));
    }

    #[test]
    fn test_test_events_modified() {
        let events = TestEvents::modified(KeyCode::LShift, KeyCode::A);
        assert_eq!(
            events,
            vec![
                KeyEvent::Press(KeyCode::LShift),
                KeyEvent::Press(KeyCode::A),
                KeyEvent::Release(KeyCode::A),
                KeyEvent::Release(KeyCode::LShift),
            ]
        );
    }

    #[test]
    fn test_test_events_with_modifiers_single() {
        let events = TestEvents::with_modifiers(&[KeyCode::LCtrl], KeyCode::C);
        assert_eq!(
            events,
            vec![
                KeyEvent::Press(KeyCode::LCtrl),
                KeyEvent::Press(KeyCode::C),
                KeyEvent::Release(KeyCode::C),
                KeyEvent::Release(KeyCode::LCtrl),
            ]
        );
    }

    #[test]
    fn test_test_events_with_modifiers_multiple() {
        let events = TestEvents::with_modifiers(&[KeyCode::LCtrl, KeyCode::LShift], KeyCode::C);
        assert_eq!(
            events,
            vec![
                KeyEvent::Press(KeyCode::LCtrl),
                KeyEvent::Press(KeyCode::LShift),
                KeyEvent::Press(KeyCode::C),
                KeyEvent::Release(KeyCode::C),
                KeyEvent::Release(KeyCode::LShift),
                KeyEvent::Release(KeyCode::LCtrl),
            ]
        );
    }

    #[test]
    fn test_test_events_with_modifiers_empty() {
        // Edge case: no modifiers is just a tap
        let events = TestEvents::with_modifiers(&[], KeyCode::A);
        assert_eq!(
            events,
            vec![KeyEvent::Press(KeyCode::A), KeyEvent::Release(KeyCode::A),]
        );
    }

    #[test]
    fn test_test_events_hold_while_typing() {
        let events =
            TestEvents::hold_while_typing(KeyCode::LShift, &[KeyCode::A, KeyCode::B, KeyCode::C]);
        assert_eq!(
            events,
            vec![
                KeyEvent::Press(KeyCode::LShift),
                KeyEvent::Press(KeyCode::A),
                KeyEvent::Release(KeyCode::A),
                KeyEvent::Press(KeyCode::B),
                KeyEvent::Release(KeyCode::B),
                KeyEvent::Press(KeyCode::C),
                KeyEvent::Release(KeyCode::C),
                KeyEvent::Release(KeyCode::LShift),
            ]
        );
    }

    #[test]
    fn test_test_events_hold_while_typing_empty() {
        // Edge case: no keys is just press/release modifier
        let events = TestEvents::hold_while_typing(KeyCode::LShift, &[]);
        assert_eq!(
            events,
            vec![
                KeyEvent::Press(KeyCode::LShift),
                KeyEvent::Release(KeyCode::LShift),
            ]
        );
    }

    #[test]
    fn test_test_events_from_events() {
        let input = vec![
            KeyEvent::Press(KeyCode::A),
            KeyEvent::Press(KeyCode::B),
            KeyEvent::Release(KeyCode::A),
            KeyEvent::Release(KeyCode::B),
        ];
        let events = TestEvents::from_events(&input);
        assert_eq!(events, input);
    }

    #[test]
    fn test_test_events_empty() {
        let events = TestEvents::empty();
        assert!(events.is_empty());
    }

    #[test]
    fn test_test_events_presses_empty() {
        let events = TestEvents::presses(&[]);
        assert!(events.is_empty());
    }

    #[test]
    fn test_test_events_releases_empty() {
        let events = TestEvents::releases(&[]);
        assert!(events.is_empty());
    }

    #[test]
    fn test_test_events_taps_empty() {
        let events = TestEvents::taps(&[]);
        assert!(events.is_empty());
    }

    #[test]
    fn test_test_events_type_keys_empty() {
        let events = TestEvents::type_keys(&[]);
        assert!(events.is_empty());
    }

    /// Test a complex vim-style navigation pattern using TestEvents
    #[test]
    fn test_test_events_vim_navigation_pattern() {
        // Simulate: hold CapsLock, press HJKL for arrow navigation
        let events = TestEvents::hold_while_typing(
            KeyCode::CapsLock,
            &[KeyCode::H, KeyCode::J, KeyCode::K, KeyCode::L],
        );

        // 1 press + 4*(press+release) + 1 release = 10 events
        assert_eq!(events.len(), 10);

        // First event is CapsLock press
        assert_eq!(events[0], KeyEvent::Press(KeyCode::CapsLock));

        // Last event is CapsLock release
        assert_eq!(events[9], KeyEvent::Release(KeyCode::CapsLock));
    }

    // ------------------------------------------------------------------------
    // TestTimeoutPhase Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_timeout_phase_display() {
        assert_eq!(TestTimeoutPhase::Setup.to_string(), "setup");
        assert_eq!(TestTimeoutPhase::Injection.to_string(), "event injection");
        assert_eq!(TestTimeoutPhase::Capture.to_string(), "event capture");
        assert_eq!(TestTimeoutPhase::Verification.to_string(), "verification");
        assert_eq!(TestTimeoutPhase::Teardown.to_string(), "teardown");
        assert_eq!(TestTimeoutPhase::TestLogic.to_string(), "test logic");
    }

    #[test]
    fn test_timeout_phase_equality() {
        assert_eq!(TestTimeoutPhase::Setup, TestTimeoutPhase::Setup);
        assert_ne!(TestTimeoutPhase::Setup, TestTimeoutPhase::Teardown);
    }

    #[test]
    fn test_timeout_phase_clone() {
        let phase = TestTimeoutPhase::Capture;
        let cloned = phase;
        assert_eq!(phase, cloned);
    }

    // ------------------------------------------------------------------------
    // E2EError::TestTimeout Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_e2e_error_display_test_timeout() {
        let err = E2EError::TestTimeout {
            phase: TestTimeoutPhase::Capture,
            timeout: Duration::from_secs(30),
            elapsed: Duration::from_secs_f64(30.5),
            context: "Waiting for daemon output".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("TEST TIMEOUT"));
        assert!(msg.contains("30s limit"));
        assert!(msg.contains("30.50s"));
        assert!(msg.contains("event capture"));
        assert!(msg.contains("Waiting for daemon output"));
        assert!(msg.contains("Check for hung daemon"));
    }

    #[test]
    fn test_e2e_error_display_test_timeout_empty_context() {
        let err = E2EError::TestTimeout {
            phase: TestTimeoutPhase::Setup,
            timeout: Duration::from_secs(10),
            elapsed: Duration::from_secs(10),
            context: String::new(),
        };
        let msg = err.to_string();
        assert!(msg.contains("TEST TIMEOUT"));
        assert!(msg.contains("setup"));
        // Empty context should not add "Context:" line
        assert!(!msg.contains("Context:"));
    }

    // ------------------------------------------------------------------------
    // Timeout Constants Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_default_test_timeout_is_reasonable() {
        // Should be between 10 and 120 seconds
        assert!(DEFAULT_TEST_TIMEOUT >= Duration::from_secs(10));
        assert!(DEFAULT_TEST_TIMEOUT <= Duration::from_secs(120));
        // Default is 30 seconds
        assert_eq!(DEFAULT_TEST_TIMEOUT, Duration::from_secs(30));
    }

    // ------------------------------------------------------------------------
    // Timeout Wrapper Integration Tests
    // ------------------------------------------------------------------------

    /// Test that with_timeout correctly handles setup failures
    /// Note: This test doesn't require uinput as it tests the timeout wrapper behavior
    /// when setup fails (which it will without uinput access)
    #[test]
    fn test_with_timeout_handles_setup_failure() {
        let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);

        // This will fail during setup because we don't have uinput access
        let result = with_timeout(Duration::from_secs(5), config, |_harness, _set_phase| {
            // This should never be called because setup fails
            Ok(())
        });

        // Should fail with VirtualDevice or ConfigError (not TestTimeout)
        assert!(result.is_err());
        match result {
            Err(E2EError::VirtualDevice(_)) | Err(E2EError::ConfigError { .. }) => {
                // Expected errors when uinput is not available
            }
            Err(E2EError::DaemonStartError { .. }) | Err(E2EError::DaemonCrashed { .. }) => {
                // Also acceptable if we got past device creation
            }
            Err(E2EError::TestTimeout { .. }) => {
                panic!("Should not be a timeout error for quick setup failure");
            }
            Err(e) => {
                // Other errors are also acceptable
                println!("Got expected error: {}", e);
            }
            Ok(_) => {
                // If it succeeded, that's fine too (means we have uinput access)
                println!("Setup succeeded unexpectedly - running in privileged environment");
            }
        }
    }

    /// Test that the phase setter is callable
    #[test]
    fn test_phase_setter_is_callable() {
        // We can't easily test the actual phase tracking without a real harness,
        // but we can verify the function signature works
        let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);

        let result: TimeoutResult<()> =
            with_timeout(Duration::from_millis(500), config, |_harness, set_phase| {
                // Test that we can call set_phase with different phases
                set_phase(TestTimeoutPhase::Injection);
                set_phase(TestTimeoutPhase::Capture);
                set_phase(TestTimeoutPhase::Verification);
                // Return quickly since we expect setup to fail without uinput
                Err(E2EError::Timeout {
                    operation: "test".to_string(),
                    timeout_ms: 0,
                })
            });

        // We expect an error (either from setup or our explicit error)
        assert!(result.is_err());
    }

    /// Test run_test_with_timeout convenience function
    #[test]
    fn test_run_test_with_timeout_convenience() {
        let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);

        let result: TimeoutResult<()> =
            run_test_with_timeout(Duration::from_millis(500), config, |_harness| {
                // Return error since we expect setup to fail
                Err(E2EError::Timeout {
                    operation: "test".to_string(),
                    timeout_ms: 0,
                })
            });

        // We expect an error
        assert!(result.is_err());
    }

    /// Test that with_default_timeout uses the correct default
    #[test]
    fn test_with_default_timeout_uses_default() {
        let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);

        // This should use DEFAULT_TEST_TIMEOUT (30s)
        // We just verify it compiles and runs without immediate timeout
        let result: TimeoutResult<()> = with_default_timeout(config, |_harness, _set_phase| {
            Err(E2EError::Timeout {
                operation: "test".to_string(),
                timeout_ms: 0,
            })
        });

        // Should fail with setup error or our explicit error, not timeout
        assert!(result.is_err());
        if let Err(E2EError::TestTimeout { .. }) = result {
            panic!("Should not timeout with 30s limit for quick failure");
        }
    }

    /// Test that the timeout wrapper runs on E2E tests when available
    #[test]
    fn test_timeout_wrapper_with_real_harness() {
        keyrx_daemon::skip_if_no_uinput!();
        let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);

        let result = with_timeout(Duration::from_secs(30), config, |harness, set_phase| {
            // Set phase to show we're in test logic
            set_phase(TestTimeoutPhase::TestLogic);

            // Test a simple remap
            set_phase(TestTimeoutPhase::Injection);
            harness.inject(&TestEvents::tap(KeyCode::A))?;

            set_phase(TestTimeoutPhase::Capture);
            let captured = harness.capture(Duration::from_millis(500))?;

            set_phase(TestTimeoutPhase::Verification);
            harness.verify(&captured, &TestEvents::tap(KeyCode::B))?;

            Ok(())
        });

        result.expect("Test should pass with timeout wrapper");
    }

    /// Test that the timeout wrapper correctly reports timeout phase
    /// This test uses a mock scenario since we can't easily trigger a real hang
    #[test]
    fn test_timeout_error_has_phase_info() {
        // Create an error with specific phase info
        let err = E2EError::TestTimeout {
            phase: TestTimeoutPhase::Capture,
            timeout: Duration::from_secs(30),
            elapsed: Duration::from_secs(30),
            context: "Waiting for key events from daemon".to_string(),
        };

        // Verify the error contains all relevant diagnostic info
        let msg = err.to_string();
        assert!(
            msg.contains("event capture"),
            "Should mention capture phase"
        );
        assert!(msg.contains("30s"), "Should mention timeout duration");
        assert!(msg.contains("key events"), "Should include context");
    }
}
