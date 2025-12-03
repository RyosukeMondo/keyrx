//! Fuzz testing with random key sequence generation.
//!
//! This module provides fuzz testing capabilities for the KeyRx engine,
//! generating random key sequences to test robustness and discover crashes.

use crate::drivers::keycodes::all_keycodes;
use crate::engine::{AdvancedEngine, InputEvent, KeyCode, TimingConfig};
use crate::mocks::MockRuntime;
use chrono::{DateTime, Utc};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Result of a fuzz testing run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuzzResult {
    /// Number of sequences tested.
    pub sequences_tested: usize,
    /// Duration in seconds.
    pub duration_secs: f64,
    /// Number of unique execution paths discovered.
    pub unique_paths: usize,
    /// Crash sequences found.
    pub crashes: Vec<CrashSequence>,
}

/// A crash-inducing sequence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashSequence {
    /// Timestamp when crash occurred.
    pub timestamp: String,
    /// Path to saved sequence file.
    pub file_path: String,
    /// Error message.
    pub error: String,
}

/// A single key event in a fuzz sequence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuzzEvent {
    /// The key code.
    pub key: KeyCode,
    /// Whether key is pressed (true) or released (false).
    pub pressed: bool,
    /// Timestamp in microseconds.
    pub timestamp_us: u64,
}

/// A complete fuzz sequence for replay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuzzSequence {
    /// Creation timestamp.
    pub created: DateTime<Utc>,
    /// List of key events.
    pub events: Vec<FuzzEvent>,
    /// Error that occurred (if any).
    pub error: Option<String>,
}

/// Configuration for fuzz testing.
#[derive(Debug, Clone)]
pub struct FuzzConfig {
    /// Minimum number of sequences to test.
    pub min_sequences: usize,
    /// Maximum duration for fuzz testing.
    pub max_duration: Duration,
    /// Events per sequence (min).
    pub min_events_per_sequence: usize,
    /// Events per sequence (max).
    pub max_events_per_sequence: usize,
    /// Probability of key press vs release (0.0-1.0).
    pub press_probability: f64,
    /// Directory to save crash sequences.
    pub crash_dir: PathBuf,
}

impl Default for FuzzConfig {
    fn default() -> Self {
        Self {
            min_sequences: 10_000,
            max_duration: Duration::from_secs(60),
            min_events_per_sequence: 5,
            max_events_per_sequence: 50,
            press_probability: 0.6,
            crash_dir: PathBuf::from("tests/crashes"),
        }
    }
}

/// Fuzz testing engine.
#[derive(Debug)]
pub struct FuzzEngine {
    config: FuzzConfig,
    available_keys: Vec<KeyCode>,
}

impl Default for FuzzEngine {
    fn default() -> Self {
        Self::new(FuzzConfig::default())
    }
}

impl FuzzEngine {
    /// Create a new fuzz engine with configuration.
    pub fn new(config: FuzzConfig) -> Self {
        Self {
            config,
            available_keys: all_keycodes(),
        }
    }

    /// Create a fuzz engine with a custom crash directory.
    pub fn with_crash_dir(crash_dir: impl Into<PathBuf>) -> Self {
        let config = FuzzConfig {
            crash_dir: crash_dir.into(),
            ..Default::default()
        };
        Self::new(config)
    }

    /// Run fuzz testing for specified duration or count.
    ///
    /// If `count` is provided, runs exactly that many sequences.
    /// Otherwise runs for `duration` or until min_sequences is reached.
    pub fn run(&self, duration: Duration, count: Option<u64>) -> FuzzResult {
        let target_count = count
            .map(|c| c as usize)
            .unwrap_or(self.config.min_sequences);
        let max_duration = duration.min(self.config.max_duration);

        let start = Instant::now();
        let mut rng = rand::rng();
        let mut sequences_tested = 0;
        let mut unique_paths = HashSet::new();
        let mut crashes = Vec::new();

        loop {
            // Stop conditions
            if sequences_tested >= target_count {
                break;
            }
            if count.is_none() && start.elapsed() >= max_duration {
                break;
            }

            // Generate a random sequence
            let sequence = self.generate_sequence(&mut rng);

            // Execute the sequence and check for crashes
            match self.execute_sequence(&sequence) {
                Ok(path_hash) => {
                    unique_paths.insert(path_hash);
                }
                Err(error) => {
                    // Save crash sequence
                    if let Some(crash) = self.save_crash_sequence(&sequence, &error) {
                        crashes.push(crash);
                    }
                }
            }

            sequences_tested += 1;
        }

        FuzzResult {
            sequences_tested,
            duration_secs: start.elapsed().as_secs_f64(),
            unique_paths: unique_paths.len(),
            crashes,
        }
    }

    /// Generate a random key sequence.
    fn generate_sequence<R: Rng>(&self, rng: &mut R) -> FuzzSequence {
        let event_count = rng.random_range(
            self.config.min_events_per_sequence..=self.config.max_events_per_sequence,
        );
        let mut events = Vec::with_capacity(event_count);
        let mut pressed_keys: HashSet<KeyCode> = HashSet::new();
        let mut timestamp_us = 0u64;

        for _ in 0..event_count {
            // Decide whether to press or release
            let should_press = if pressed_keys.is_empty() {
                true // Must press if nothing is held
            } else if pressed_keys.len() >= 6 {
                false // Release some keys if too many held
            } else {
                rng.random_bool(self.config.press_probability)
            };

            let (key, pressed) = if should_press {
                // Press a random key (prefer not already pressed)
                let key = self.available_keys[rng.random_range(0..self.available_keys.len())];
                pressed_keys.insert(key);
                (key, true)
            } else {
                // Release a pressed key
                let pressed_vec: Vec<_> = pressed_keys.iter().copied().collect();
                let key = pressed_vec[rng.random_range(0..pressed_vec.len())];
                pressed_keys.remove(&key);
                (key, false)
            };

            // Advance timestamp by 1-100ms
            timestamp_us += rng.random_range(1_000..100_000);

            events.push(FuzzEvent {
                key,
                pressed,
                timestamp_us,
            });
        }

        FuzzSequence {
            created: Utc::now(),
            events,
            error: None,
        }
    }

    /// Execute a sequence against the engine, returning a path hash or error.
    fn execute_sequence(&self, sequence: &FuzzSequence) -> Result<u64, String> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let runtime = MockRuntime::default();
        let mut engine = AdvancedEngine::new(runtime, TimingConfig::default());
        let mut path_hasher = DefaultHasher::new();
        let mut tick_time = 0u64;

        for event in &sequence.events {
            // Create input event
            let input = if event.pressed {
                InputEvent::key_down(event.key, event.timestamp_us)
            } else {
                InputEvent::key_up(event.key, event.timestamp_us)
            };

            // Process event (panics are caught by caller if using catch_unwind)
            let outputs = engine.process_event(input);

            // Hash the outputs for path tracking
            for output in &outputs {
                format!("{:?}", output).hash(&mut path_hasher);
            }

            // Periodically tick the engine
            if event.timestamp_us > tick_time + 10_000 {
                tick_time = event.timestamp_us;
                let tick_outputs = engine.tick(tick_time);
                for output in &tick_outputs {
                    format!("{:?}", output).hash(&mut path_hasher);
                }
            }
        }

        Ok(path_hasher.finish())
    }

    /// Save a crash sequence to disk.
    fn save_crash_sequence(&self, sequence: &FuzzSequence, error: &str) -> Option<CrashSequence> {
        // Ensure crash directory exists
        if let Err(e) = fs::create_dir_all(&self.config.crash_dir) {
            eprintln!("Failed to create crash directory: {}", e);
            return None;
        }

        let timestamp = Utc::now().format("%Y-%m-%dT%H-%M-%S").to_string();
        let filename = format!("{}.krx", timestamp);
        let file_path = self.config.crash_dir.join(&filename);

        let mut crash_sequence = sequence.clone();
        crash_sequence.error = Some(error.to_string());

        match serde_json::to_string_pretty(&crash_sequence) {
            Ok(json) => {
                if let Err(e) = fs::write(&file_path, json) {
                    eprintln!("Failed to write crash sequence: {}", e);
                    return None;
                }
            }
            Err(e) => {
                eprintln!("Failed to serialize crash sequence: {}", e);
                return None;
            }
        }

        Some(CrashSequence {
            timestamp,
            file_path: file_path.to_string_lossy().to_string(),
            error: error.to_string(),
        })
    }

    /// Replay a crash sequence from file.
    pub fn replay_crash(&self, crash_file: &Path) -> Result<(), String> {
        let content = fs::read_to_string(crash_file)
            .map_err(|e| format!("Failed to read crash file: {}", e))?;

        let sequence: FuzzSequence = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse crash sequence: {}", e))?;

        match self.execute_sequence(&sequence) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzz_engine_default() {
        let engine = FuzzEngine::default();
        assert!(!engine.available_keys.is_empty());
        assert_eq!(engine.config.min_sequences, 10_000);
    }

    #[test]
    fn test_generate_sequence() {
        let engine = FuzzEngine::default();
        let mut rng = rand::rng();

        let sequence = engine.generate_sequence(&mut rng);
        assert!(!sequence.events.is_empty());
        assert!(sequence.events.len() >= engine.config.min_events_per_sequence);
        assert!(sequence.events.len() <= engine.config.max_events_per_sequence);
    }

    #[test]
    fn test_execute_sequence_no_crash() {
        let engine = FuzzEngine::default();
        let sequence = FuzzSequence {
            created: Utc::now(),
            events: vec![
                FuzzEvent {
                    key: KeyCode::A,
                    pressed: true,
                    timestamp_us: 0,
                },
                FuzzEvent {
                    key: KeyCode::A,
                    pressed: false,
                    timestamp_us: 50_000,
                },
            ],
            error: None,
        };

        let result = engine.execute_sequence(&sequence);
        assert!(result.is_ok());
    }

    #[test]
    fn test_fuzz_run_small() {
        let mut config = FuzzConfig::default();
        config.min_sequences = 5;
        config.max_duration = Duration::from_secs(30);
        config.min_events_per_sequence = 2;
        config.max_events_per_sequence = 5;

        let engine = FuzzEngine::new(config);
        let result = engine.run(Duration::from_secs(30), Some(5));

        assert!(result.sequences_tested >= 5);
        assert!(result.unique_paths > 0);
    }

    #[test]
    fn test_sequence_timestamps_increase() {
        let engine = FuzzEngine::default();
        let mut rng = rand::rng();

        let sequence = engine.generate_sequence(&mut rng);
        let mut prev_ts = 0u64;

        for event in &sequence.events {
            assert!(
                event.timestamp_us > prev_ts,
                "Timestamps should strictly increase"
            );
            prev_ts = event.timestamp_us;
        }
    }

    #[test]
    fn test_sequence_serialization() {
        let sequence = FuzzSequence {
            created: Utc::now(),
            events: vec![FuzzEvent {
                key: KeyCode::A,
                pressed: true,
                timestamp_us: 0,
            }],
            error: None,
        };

        let json = serde_json::to_string(&sequence).expect("serialization should work");
        let parsed: FuzzSequence =
            serde_json::from_str(&json).expect("deserialization should work");

        assert_eq!(parsed.events.len(), 1);
        assert_eq!(parsed.events[0].key, KeyCode::A);
    }
}
