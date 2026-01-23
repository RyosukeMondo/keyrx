//! Simulation service providing business logic for event simulation operations.
//!
//! This service wraps [`SimulationEngine`] with a thread-safe service layer
//! for loading profiles, replaying events, and running test scenarios.
//!
//! # Examples
//!
//! ```no_run
//! use std::path::PathBuf;
//! use keyrx_daemon::services::SimulationService;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let service = SimulationService::new(PathBuf::from("./config"));
//!
//! // Load a profile
//! service.load_profile("default")?;
//!
//! // Run a built-in scenario
//! let result = service.run_scenario("tap-hold-under-threshold")?;
//! println!("Scenario passed: {}", result.passed);
//! # Ok(())
//! # }
//! ```

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use keyrx_core::config::KeyCode;
use keyrx_core::runtime::KeyEvent;
use tokio::sync::mpsc;

use crate::config::simulation_engine::{
    BuiltinScenario, EventSequence, EventType, OutputEvent, ScenarioResult, SimulationEngine,
    SimulationError,
};

/// Service for simulation operations.
///
/// Provides thread-safe access to simulation functionality via Mutex-wrapped
/// SimulationEngine. The engine is created when a profile is loaded.
///
/// # Thread Safety
///
/// SimulationService is `Send + Sync` and can be shared across threads.
/// Internal state is protected by a Mutex.
pub struct SimulationService {
    /// Configuration directory containing .krx profiles
    config_dir: PathBuf,
    /// Optional simulation engine (created when profile is loaded)
    engine: Mutex<Option<SimulationEngine>>,
    /// Event bus sender for routing simulated events to macro recorder
    event_tx: Option<mpsc::Sender<KeyEvent>>,
}

impl SimulationService {
    /// Creates a new SimulationService.
    ///
    /// # Arguments
    ///
    /// * `config_dir` - Path to configuration directory containing .krx profiles
    /// * `event_tx` - Optional event bus sender for routing simulated events
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::path::PathBuf;
    /// use keyrx_daemon::services::SimulationService;
    ///
    /// let service = SimulationService::new(PathBuf::from("./config"), None);
    /// ```
    pub fn new(config_dir: PathBuf, event_tx: Option<mpsc::Sender<KeyEvent>>) -> Self {
        log::debug!(
            "SimulationService initialized with config_dir: {:?}, event_tx: {}",
            config_dir,
            event_tx.is_some()
        );
        Self {
            config_dir,
            engine: Mutex::new(None),
            event_tx,
        }
    }

    /// Loads a profile by name and initializes the simulation engine.
    ///
    /// # Arguments
    ///
    /// * `profile_name` - Name of the profile (without .krx extension)
    ///
    /// # Returns
    ///
    /// Success if profile loaded, error if file not found or invalid.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::path::PathBuf;
    /// # use keyrx_daemon::services::SimulationService;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let service = SimulationService::new(PathBuf::from("./config"));
    /// service.load_profile("default")?;
    /// # Ok(())
    /// # }
    /// ```
    /// Helper method to convert OutputEvent to KeyEvent and send to event bus
    ///
    /// This is an async method that sends the event to the event bus channel.
    /// It converts the OutputEvent format to KeyEvent format with:
    /// - Lowercase event_type ("press", "release")
    /// - Current timestamp in microseconds
    /// - Key as string
    async fn send_to_event_bus(&self, output: &OutputEvent) -> Result<(), SimulationError> {
        if let Some(event_tx) = &self.event_tx {
            // Convert string key to KeyCode
            // For now, we'll parse common keys - this matches the key format from simulator
            let key_str = output.key.as_str();
            let keycode = match key_str {
                "A" => KeyCode::A,
                "B" => KeyCode::B,
                "C" => KeyCode::C,
                "D" => KeyCode::D,
                "E" => KeyCode::E,
                "F" => KeyCode::F,
                "G" => KeyCode::G,
                "H" => KeyCode::H,
                "I" => KeyCode::I,
                "J" => KeyCode::J,
                "K" => KeyCode::K,
                "L" => KeyCode::L,
                "M" => KeyCode::M,
                "N" => KeyCode::N,
                "O" => KeyCode::O,
                "P" => KeyCode::P,
                "Q" => KeyCode::Q,
                "R" => KeyCode::R,
                "S" => KeyCode::S,
                "T" => KeyCode::T,
                "U" => KeyCode::U,
                "V" => KeyCode::V,
                "W" => KeyCode::W,
                "X" => KeyCode::X,
                "Y" => KeyCode::Y,
                "Z" => KeyCode::Z,
                "CapsLock" => KeyCode::CapsLock,
                "Shift" => KeyCode::LShift,
                "Ctrl" => KeyCode::LCtrl,
                "Alt" => KeyCode::LAlt,
                "Space" => KeyCode::Space,
                "Enter" => KeyCode::Enter,
                _ => {
                    log::warn!("Unknown key '{}', defaulting to A", key_str);
                    KeyCode::A
                }
            };

            // Get current timestamp
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_micros() as u64;

            // Create KeyEvent based on event type
            let key_event = match output.event_type {
                EventType::Press => KeyEvent::press(keycode).with_timestamp(timestamp),
                EventType::Release => KeyEvent::release(keycode).with_timestamp(timestamp),
            };

            // Send to event bus
            if let Err(e) = event_tx.send(key_event).await {
                log::error!("Failed to send event to event bus: {}", e);
                return Err(SimulationError::IoError(std::io::Error::new(
                    std::io::ErrorKind::BrokenPipe,
                    format!("Event bus send failed: {}", e),
                )));
            }

            log::debug!("Sent event to bus: {:?} {}", output.event_type, output.key);
        }

        Ok(())
    }

    pub fn load_profile(&self, profile_name: &str) -> Result<(), SimulationError> {
        log::debug!("Loading profile: {}", profile_name);

        let krx_path = self
            .config_dir
            .join("profiles")
            .join(format!("{}.krx", profile_name));
        let engine = SimulationEngine::new(&krx_path)?;

        let mut guard = self.engine.lock().unwrap();
        *guard = Some(engine);

        log::info!("Profile '{}' loaded successfully", profile_name);
        Ok(())
    }

    /// Replays a sequence of events through the simulation engine.
    ///
    /// # Arguments
    ///
    /// * `sequence` - Event sequence to replay
    ///
    /// # Returns
    ///
    /// Vector of output events generated by the simulation.
    ///
    /// # Errors
    ///
    /// Returns error if no profile is loaded or replay fails.
    pub async fn replay(
        &self,
        sequence: &EventSequence,
    ) -> Result<Vec<OutputEvent>, SimulationError> {
        log::debug!(
            "Replaying event sequence with {} events",
            sequence.events.len()
        );

        let outputs = {
            let mut guard = self.engine.lock().unwrap();
            let engine = guard
                .as_mut()
                .ok_or_else(|| SimulationError::LoadError("No profile loaded".to_string()))?;

            engine.replay(sequence)?
        };

        // Send each output event to the event bus
        for output in &outputs {
            self.send_to_event_bus(output).await?;
        }

        Ok(outputs)
    }

    /// Runs a built-in test scenario by name.
    ///
    /// # Arguments
    ///
    /// * `scenario_name` - Name of the scenario (e.g., "tap-hold-under-threshold")
    ///
    /// # Returns
    ///
    /// Result containing input events, output events, and pass/fail status.
    ///
    /// # Errors
    ///
    /// Returns error if no profile is loaded or scenario not found.
    pub fn run_scenario(&self, scenario_name: &str) -> Result<ScenarioResult, SimulationError> {
        log::debug!("Running scenario: {}", scenario_name);

        // Parse scenario name
        let scenario = match scenario_name {
            "tap-hold-under-threshold" => BuiltinScenario::TapHoldUnderThreshold,
            "tap-hold-over-threshold" => BuiltinScenario::TapHoldOverThreshold,
            "permissive-hold" => BuiltinScenario::PermissiveHold,
            "cross-device-modifiers" => BuiltinScenario::CrossDeviceModifiers,
            "macro-sequence" => BuiltinScenario::MacroSequence,
            _ => return Err(SimulationError::ScenarioNotFound(scenario_name.to_string())),
        };

        let mut guard = self.engine.lock().unwrap();
        let engine = guard
            .as_mut()
            .ok_or_else(|| SimulationError::LoadError("No profile loaded".to_string()))?;

        engine.run_scenario(scenario)
    }

    /// Runs all built-in test scenarios.
    ///
    /// # Returns
    ///
    /// Vector of results for all scenarios with pass/fail status.
    ///
    /// # Errors
    ///
    /// Returns error if no profile is loaded.
    pub fn run_all_scenarios(&self) -> Result<Vec<ScenarioResult>, SimulationError> {
        log::debug!("Running all scenarios");

        let mut guard = self.engine.lock().unwrap();
        let engine = guard
            .as_mut()
            .ok_or_else(|| SimulationError::LoadError("No profile loaded".to_string()))?;

        engine.run_all_scenarios()
    }

    /// Parses event DSL and replays the resulting sequence.
    ///
    /// DSL format: "press:A,wait:50,release:A"
    /// - press:KEY - Press key
    /// - release:KEY - Release key
    /// - wait:MS - Wait milliseconds
    ///
    /// # Arguments
    ///
    /// * `dsl` - Event DSL string
    /// * `seed` - Seed for deterministic behavior
    ///
    /// # Returns
    ///
    /// Vector of output events generated by the simulation.
    ///
    /// # Errors
    ///
    /// Returns error if no profile is loaded, DSL is invalid, or replay fails.
    pub async fn replay_dsl(
        &self,
        dsl: &str,
        seed: u64,
    ) -> Result<Vec<OutputEvent>, SimulationError> {
        log::debug!("Replaying DSL: {} (seed: {})", dsl, seed);

        let sequence = SimulationEngine::parse_event_dsl(dsl, seed)?;
        self.replay(&sequence).await
    }

    /// Resets the simulation state by clearing the loaded engine.
    ///
    /// After reset, a profile must be loaded again before running simulations.
    pub fn reset(&self) {
        log::debug!("Resetting simulation state");

        let mut guard = self.engine.lock().unwrap();
        *guard = None;

        log::info!("Simulation state reset");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_profile(dir: &TempDir, name: &str) -> PathBuf {
        let krx_path = dir.path().join(format!("{}.krx", name));
        let mut file = std::fs::File::create(&krx_path).unwrap();
        file.write_all(b"test krx data").unwrap();
        krx_path
    }

    #[test]
    fn test_new() {
        let dir = TempDir::new().unwrap();
        let service = SimulationService::new(dir.path().to_path_buf(), None);

        // Should have no engine initially
        assert!(service.engine.lock().unwrap().is_none());
    }

    #[test]
    fn test_load_profile() {
        let dir = TempDir::new().unwrap();
        create_test_profile(&dir, "test");

        let service = SimulationService::new(dir.path().to_path_buf(), None);
        let result = service.load_profile("test");

        assert!(result.is_ok());
        assert!(service.engine.lock().unwrap().is_some());
    }

    #[test]
    fn test_load_profile_not_found() {
        let dir = TempDir::new().unwrap();
        let service = SimulationService::new(dir.path().to_path_buf(), None);

        let result = service.load_profile("nonexistent");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_replay_without_profile() {
        let dir = TempDir::new().unwrap();
        let service = SimulationService::new(dir.path().to_path_buf(), None);

        let sequence = EventSequence {
            events: vec![],
            seed: 0,
        };

        let result = service.replay(&sequence).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_run_scenario() {
        let dir = TempDir::new().unwrap();
        create_test_profile(&dir, "test");

        let service = SimulationService::new(dir.path().to_path_buf(), None);
        service.load_profile("test").unwrap();

        let result = service.run_scenario("tap-hold-under-threshold");
        assert!(result.is_ok());

        let scenario_result = result.unwrap();
        assert_eq!(scenario_result.scenario, "tap-hold-under-threshold");
        assert!(scenario_result.passed);
    }

    #[test]
    fn test_run_scenario_unknown() {
        let dir = TempDir::new().unwrap();
        create_test_profile(&dir, "test");

        let service = SimulationService::new(dir.path().to_path_buf(), None);
        service.load_profile("test").unwrap();

        let result = service.run_scenario("unknown-scenario");
        assert!(result.is_err());
    }

    #[test]
    fn test_run_all_scenarios() {
        let dir = TempDir::new().unwrap();
        create_test_profile(&dir, "test");

        let service = SimulationService::new(dir.path().to_path_buf(), None);
        service.load_profile("test").unwrap();

        let results = service.run_all_scenarios().unwrap();
        assert_eq!(results.len(), BuiltinScenario::all().len());
        assert!(results.iter().all(|r| r.passed));
    }

    #[tokio::test]
    async fn test_replay_dsl() {
        let dir = TempDir::new().unwrap();
        create_test_profile(&dir, "test");

        let service = SimulationService::new(dir.path().to_path_buf(), None);
        service.load_profile("test").unwrap();

        let result = service.replay_dsl("press:A,wait:50,release:A", 42).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.len(), 2); // Press and release
    }

    #[test]
    fn test_reset() {
        let dir = TempDir::new().unwrap();
        create_test_profile(&dir, "test");

        let service = SimulationService::new(dir.path().to_path_buf(), None);
        service.load_profile("test").unwrap();

        assert!(service.engine.lock().unwrap().is_some());

        service.reset();
        assert!(service.engine.lock().unwrap().is_none());
    }
}
