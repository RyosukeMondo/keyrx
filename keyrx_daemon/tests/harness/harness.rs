//! E2E Test Harness - Complete test orchestration.
//!
//! This module contains the [`E2EHarness`] struct that orchestrates
//! end-to-end tests, including lifecycle management and event handling.

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant, SystemTime};

use keyrx_compiler::serialize::serialize as serialize_config;
use keyrx_daemon::test_utils::{OutputCapture, VirtualDeviceError, VirtualKeyboard};
use keyrx_core::runtime::KeyEvent;

use super::assertions::TeardownResult;
use super::config::E2EConfig;
use super::error::E2EError;

/// Default name for the daemon's virtual output device.
#[cfg(target_os = "linux")]
const DAEMON_OUTPUT_NAME: &str = "keyrx Virtual Keyboard";
#[cfg(target_os = "windows")]
const DAEMON_OUTPUT_NAME: &str = "Windows Global Keyboard Hook";

/// Default timeout for waiting for daemon to be ready.
const DAEMON_STARTUP_TIMEOUT: Duration = Duration::from_secs(5);

/// Default timeout for waiting for output device to appear.
const OUTPUT_DEVICE_TIMEOUT: Duration = Duration::from_secs(5);

/// Orchestrates complete E2E test lifecycle.
pub struct E2EHarness {
    virtual_input: VirtualKeyboard,
    daemon_process: Option<Child>,
    output_capture: OutputCapture,
    config_path: PathBuf,
    #[allow(dead_code)]
    daemon_stderr: Option<String>,
}

#[allow(dead_code)]
impl E2EHarness {
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
    TestTimeout {
        /// Which phase the test was in when it timed out.
        phase: TestTimeoutPhase,
        /// The timeout limit that was exceeded.
        timeout: Duration,
        /// Total time elapsed since test start.
        elapsed: Duration,
        /// Additional diagnostic context.
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
                let context_msg = if context.is_empty() {
                    String::new()
                } else {
                    format!("\nContext: {}", context)
                };
                write!(
                    f,
                    "E2E TEST TIMEOUT: Phase {} took {:.2}s ({}s limit).{}\nCheck for hung daemon processes and verify timeout configuration.",
                    phase,
                    elapsed.as_secs_f64(),
                    timeout.as_secs(),
                    context_msg
                )
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

#[allow(dead_code)]
impl E2EConfig {
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
#[derive(Debug, Clone, Default)]
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

#[allow(dead_code)]
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
#[cfg(target_os = "linux")]
const DAEMON_OUTPUT_NAME: &str = "keyrx Virtual Keyboard";
#[cfg(target_os = "windows")]
const DAEMON_OUTPUT_NAME: &str = "Windows Global Keyboard Hook";

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

#[allow(dead_code)]
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
        let device_pattern = if cfg!(target_os = "linux") {
            virtual_input.name().to_string()
        } else {
            "*".to_string()
        };

        let test_config = E2EConfig {
            device_pattern,
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
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| E2EError::DaemonStartError {
                message: format!("failed to spawn daemon: {}", e),
                stderr: None,
            })?;

        // Step 4: Wait for daemon to start and grab our device
        // We do this by waiting for the output device to appear
        let start = Instant::now();

        // Give daemon a moment to initialize and install its hook
        std::thread::sleep(Duration::from_millis(500));

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
                .and_then(Self::read_child_stderr);
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
                .and_then(Self::read_child_stderr);
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

    /// Replays a recording from a file.
    ///
    /// Reads a JSON recording file (generated by `keyrx_daemon record`) and
    /// injects the events into the virtual keyboard, preserving relative timing.
    /// Captures all output events generated during the replay.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the .json recording file
    ///
    /// # Returns
    ///
    /// A vector of captured output events.
    #[allow(dead_code)]
    pub fn replay_recording(&mut self, path: &std::path::Path) -> Result<Vec<KeyEvent>, E2EError> {
        use serde::Deserialize;
        use std::io::BufReader;

        #[derive(Deserialize)]
        struct Metadata {
            #[allow(dead_code)]
            version: String,
            #[allow(dead_code)]
            timestamp: String,
            #[allow(dead_code)]
            device_name: String,
        }

        #[derive(Deserialize)]
        struct Recording {
            #[allow(dead_code)]
            metadata: Metadata,
            events: Vec<keyrx_core::runtime::KeyEvent>,
        }

        let file = File::open(path).map_err(E2EError::Io)?;
        let reader = BufReader::new(file);
        let recording: Recording =
            serde_json::from_reader(reader).map_err(|e| E2EError::ConfigError {
                message: format!("Failed to parse recording: {}", e),
            })?;

        // Drain any pending events
        self.drain()?;

        // Calculate initial offset to normalize times
        let start_time = if let Some(first) = recording.events.first() {
            first.timestamp_us()
        } else {
            return Ok(Vec::new());
        };

        let mut last_processed_time = start_time;

        for event in recording.events {
            // Calculate delay from the previous event
            let delay_us = event.timestamp_us().saturating_sub(last_processed_time);

            if delay_us > 0 {
                // Sleep to simulate realistic timing
                std::thread::sleep(Duration::from_micros(delay_us));
            }

            last_processed_time = event.timestamp_us();

            // Inject the event
            // Note: The timestamp in the injected event is ignored by uinput/kernel,
            // which assigns a new timestamp when the event is received.
            self.inject(&[event])?;
        }

        // Wait a bit for final processing
        std::thread::sleep(Duration::from_millis(100));

        // Capture everything that was generated
        // We use a relatively long timeout to ensure we catch everything buffered
        self.capture(Duration::from_millis(500))
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
            let _pid = process.id();

            #[cfg(target_os = "linux")]
            {
                use nix::sys::signal::{kill, Signal};
                use nix::unistd::Pid;

                let nix_pid = Pid::from_raw(_pid as i32);
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
            #[cfg(target_os = "windows")]
            {
                // No real SIGTERM on Windows, we just kill for now
                let _ = process.kill();
                result.sigterm_sent = true;
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

                            #[cfg(target_os = "linux")]
                            {
                                use nix::sys::signal::{kill, Signal};
                                use nix::unistd::Pid;

                                let nix_pid = Pid::from_raw(_pid as i32);
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

                            #[cfg(target_os = "windows")]
                            {
                                let _ = process.kill();
                                result.sigkill_sent = true;
                            }

                            #[cfg(all(not(unix), not(windows)))]
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

        // Print Captured Logs BEFORE returning result, if any
        if let Some(logs) = self.daemon_stderr.as_ref() {
            if !logs.is_empty() {
                println!("--- Daemon Stderr ---");
                println!("{}", logs);
                println!("---------------------");
            }
        }

        Ok(result)
    }

    fn find_daemon_binary() -> Result<PathBuf, E2EError> {
        let binary_name = if cfg!(windows) {
            "keyrx_daemon.exe"
        } else {
            "keyrx_daemon"
        };

        // Check environment variable first
        if let Ok(path) = std::env::var("KEYRX_DAEMON_PATH") {
            let path = PathBuf::from(path);
            if path.exists() {
                return Ok(path);
            }
            // Fall through if environment path doesn't exist to allow tests to verify lookup logic
        }

        // Try workspace target directory
        // We navigate from the test crate to the workspace root
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
        let workspace_root = PathBuf::from(&manifest_dir)
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));

        // Try debug build first
        let debug_path = workspace_root.join(format!("target/debug/{}", binary_name));
        if debug_path.exists() {
            return Ok(debug_path);
        }

        // Try release build
        let release_path = workspace_root.join(format!("target/release/{}", binary_name));
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

    #[allow(dead_code)]
    fn read_daemon_logs(&mut self) -> String {
        if let Some(stderr) = self.daemon_stderr.as_ref() {
            stderr.clone()
        } else if let Some(child) = self.daemon_process.as_mut() {
            Self::read_child_stderr(child).unwrap_or_default()
        } else {
            String::new()
        }
    }
}

impl Drop for E2EHarness {
