//! Session replay command.
//!
//! Replays a recorded `.krx` session file through the engine for
//! debugging and verification.

use crate::cli::{OutputFormat, OutputWriter};
use crate::engine::{
    replay::{ReplaySession, ReplayState},
    AdvancedEngine, OutputAction, SessionFile, TimingConfig,
};
use crate::scripting::{RemapRegistry, RhaiRuntime};
use crate::traits::{InputSource, ScriptRuntime};
use anyhow::{Context, Result};
use std::path::PathBuf;

/// Result of comparing replay outputs to recorded outputs.
#[derive(Debug, Default)]
pub struct VerificationResult {
    /// Total events processed.
    pub total_events: usize,
    /// Events with matching outputs.
    pub matched: usize,
    /// Events with mismatched outputs.
    pub mismatched: usize,
    /// Indices and details of mismatches.
    pub mismatches: Vec<MismatchDetail>,
}

/// Details of a single output mismatch.
#[derive(Debug)]
pub struct MismatchDetail {
    /// Event sequence number.
    pub seq: u64,
    /// Recorded output from the .krx file.
    pub recorded: Vec<OutputAction>,
    /// Actual output from the engine during replay.
    pub actual: Vec<OutputAction>,
}

impl VerificationResult {
    /// Check if all outputs matched.
    pub fn all_matched(&self) -> bool {
        self.mismatched == 0
    }
}

/// Replay a recorded session.
pub struct ReplayCommand {
    /// Path to the .krx session file.
    pub session_path: PathBuf,
    /// Whether to verify outputs match recorded outputs.
    pub verify: bool,
    /// Replay speed multiplier (0 = instant, 1 = realtime).
    pub speed: f64,
    /// Output writer.
    pub output: OutputWriter,
}

impl ReplayCommand {
    /// Create a new replay command.
    pub fn new(session_path: PathBuf, format: OutputFormat) -> Self {
        Self {
            session_path,
            verify: false,
            speed: 0.0, // Default to instant replay
            output: OutputWriter::new(format),
        }
    }

    /// Enable output verification against recorded outputs.
    pub fn with_verify(mut self, verify: bool) -> Self {
        self.verify = verify;
        self
    }

    /// Set replay speed multiplier.
    pub fn with_speed(mut self, speed: f64) -> Self {
        self.speed = speed;
        self
    }

    /// Run the replay command.
    pub async fn run(&self) -> Result<VerificationResult> {
        self.output
            .success(&format!("Loading session: {}", self.session_path.display()));

        let mut replay = ReplaySession::from_file(&self.session_path)
            .with_context(|| format!("Failed to load session: {}", self.session_path.display()))?;

        replay.set_speed(self.speed);

        let session = replay.session().clone();
        self.report_session_info(&session);

        // Load script if recorded
        let runtime = self.prepare_runtime(&session)?;
        let registry = runtime.registry().clone();
        let mut engine = self.build_engine(runtime, &registry, &session.timing_config);

        self.output.success(&format!(
            "Replaying {} events{}...",
            session.event_count(),
            if self.verify {
                " with verification"
            } else {
                ""
            }
        ));

        let result = self.run_replay(&mut replay, &mut engine, &session).await?;

        self.report_result(&result);

        Ok(result)
    }

    fn report_session_info(&self, session: &SessionFile) {
        self.output.success(&format!(
            "Session: {} events, {}ms duration, avg latency {}µs",
            session.event_count(),
            session.duration_us() / 1000,
            session.avg_latency_us()
        ));

        if let Some(ref script) = session.script_path {
            self.output.success(&format!("Script: {}", script));
        }
    }

    fn prepare_runtime(&self, session: &SessionFile) -> Result<RhaiRuntime> {
        let mut runtime = RhaiRuntime::new()?;

        if let Some(ref script_path) = session.script_path {
            // Check if the script still exists
            if std::path::Path::new(script_path).exists() {
                self.output
                    .success(&format!("Loading script: {}", script_path));
                runtime.load_file(script_path)?;
                runtime.run_script()?;

                if runtime.has_hook("on_init") {
                    runtime.call_hook("on_init")?;
                }
            } else {
                self.output.warning(&format!(
                    "Script not found: {}. Replaying without script.",
                    script_path
                ));
            }
        }

        Ok(runtime)
    }

    fn build_engine(
        &self,
        runtime: RhaiRuntime,
        registry: &RemapRegistry,
        timing_config: &TimingConfig,
    ) -> AdvancedEngine<RhaiRuntime> {
        use crate::engine::{LayerAction, RemapAction};

        let mut engine = AdvancedEngine::new(runtime, timing_config.clone());

        // Seed layer mappings from registry
        let mut layers = registry.layers().clone();
        if let Some(base_id) = layers.layer_id_by_name("base") {
            for (key, action) in registry.mappings() {
                if let Some(layer_action) = match action {
                    RemapAction::Remap(target) => Some(LayerAction::Remap(target)),
                    RemapAction::Block => Some(LayerAction::Block),
                    RemapAction::Pass => None,
                } {
                    layers.set_mapping_for_layer(base_id, key, layer_action);
                }
            }

            for (key, binding) in registry.tap_holds() {
                layers.set_mapping_for_layer(
                    base_id,
                    *key,
                    LayerAction::TapHold {
                        tap: binding.tap,
                        hold: binding.hold.clone(),
                    },
                );
            }
        }
        *engine.layers_mut() = layers;

        // Seed combos and modifiers
        for combo in registry.combos().all() {
            engine
                .combos_mut()
                .register(&combo.keys, combo.action.clone());
        }
        engine
            .modifiers_mut()
            .clone_from(&registry.modifier_state());

        engine
    }

    async fn run_replay(
        &self,
        replay: &mut ReplaySession,
        engine: &mut AdvancedEngine<RhaiRuntime>,
        session: &SessionFile,
    ) -> Result<VerificationResult> {
        let mut result = VerificationResult {
            total_events: session.event_count(),
            ..Default::default()
        };

        replay.start().await?;

        let mut event_idx = 0usize;
        let mut last_timestamp = 0u64;

        while replay.state() == ReplayState::Playing {
            let events = replay.poll_events().await?;

            for input_event in events {
                // Process tick for pending actions
                if input_event.timestamp_us > last_timestamp {
                    let _tick_outputs = engine.tick(input_event.timestamp_us);
                    last_timestamp = input_event.timestamp_us;
                }

                // Process the event
                let outputs = engine.process_event(input_event);

                // Verify if requested
                if self.verify && event_idx < session.events.len() {
                    let recorded = &session.events[event_idx];

                    if outputs != recorded.output {
                        result.mismatched += 1;
                        result.mismatches.push(MismatchDetail {
                            seq: recorded.seq,
                            recorded: recorded.output.clone(),
                            actual: outputs.clone(),
                        });
                    } else {
                        result.matched += 1;
                    }
                }

                event_idx += 1;
            }

            // Small delay to prevent busy loop (only for realtime mode)
            if self.speed > 0.0 {
                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
            }
        }

        replay.stop().await?;

        Ok(result)
    }

    fn report_result(&self, result: &VerificationResult) {
        if self.verify {
            if result.all_matched() {
                self.output.success(&format!(
                    "✓ All {} events matched recorded outputs",
                    result.total_events
                ));
            } else {
                self.output.error(&format!(
                    "✗ {}/{} events mismatched",
                    result.mismatched, result.total_events
                ));

                // Show first few mismatches
                for (i, mismatch) in result.mismatches.iter().take(5).enumerate() {
                    self.output.warning(&format!(
                        "  Mismatch #{}: seq={}, recorded={:?}, actual={:?}",
                        i + 1,
                        mismatch.seq,
                        mismatch.recorded,
                        mismatch.actual
                    ));
                }

                if result.mismatches.len() > 5 {
                    self.output.warning(&format!(
                        "  ... and {} more mismatches",
                        result.mismatches.len() - 5
                    ));
                }
            }
        } else {
            self.output
                .success(&format!("Replayed {} events", result.total_events));
        }
    }
}

/// Exit codes for the replay command (re-exported from config).
pub mod exit_codes {
    pub use crate::config::exit_codes::{ERROR, SUCCESS, VERIFICATION_FAILED};
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{
        DecisionType, EngineState, EventRecordBuilder, KeyCode, LayerStack, ModifierState,
    };
    use tempfile::TempDir;

    fn make_test_session() -> SessionFile {
        let initial_state = EngineState {
            pressed_keys: vec![],
            modifiers: ModifierState::default(),
            layers: LayerStack::new(),
            pending: vec![],
            timing: TimingConfig::default(),
            safe_mode: false,
        };

        let mut session = SessionFile::new(
            "2024-01-15T10:30:00Z".to_string(),
            None,
            TimingConfig::default(),
            initial_state,
        );

        // Add pass-through events
        for i in 0..3 {
            let input = crate::engine::InputEvent::key_down(KeyCode::A, (i * 10_000) as u64);
            session.add_event(
                EventRecordBuilder::new()
                    .seq(i)
                    .timestamp_us((i * 10_000) as u64)
                    .input(input)
                    .output(vec![OutputAction::KeyDown(KeyCode::A)])
                    .decision_type(DecisionType::PassThrough)
                    .active_layers(vec![0])
                    .modifiers_state(ModifierState::default())
                    .latency_us(50)
                    .build(),
            );
        }

        session
    }

    fn write_session_file(dir: &TempDir, session: &SessionFile) -> PathBuf {
        let path = dir.path().join("test_session.krx");
        let content = session.to_json().expect("serialize session");
        std::fs::write(&path, content).expect("write session file");
        path
    }

    #[tokio::test]
    async fn replay_command_loads_session() {
        let dir = TempDir::new().expect("create temp dir");
        let session = make_test_session();
        let path = write_session_file(&dir, &session);

        let cmd = ReplayCommand::new(path, OutputFormat::Human);
        let result = cmd.run().await.expect("replay should succeed");

        assert_eq!(result.total_events, 3);
    }

    #[tokio::test]
    async fn replay_with_verify_all_match() {
        let dir = TempDir::new().expect("create temp dir");
        let session = make_test_session();
        let path = write_session_file(&dir, &session);

        let cmd = ReplayCommand::new(path, OutputFormat::Human).with_verify(true);
        let result = cmd.run().await.expect("replay should succeed");

        assert!(result.all_matched());
        assert_eq!(result.matched, 3);
        assert_eq!(result.mismatched, 0);
    }

    #[tokio::test]
    async fn replay_fails_on_missing_file() {
        let cmd = ReplayCommand::new(
            PathBuf::from("/nonexistent/session.krx"),
            OutputFormat::Human,
        );
        let result = cmd.run().await;

        assert!(result.is_err());
    }

    #[test]
    fn verification_result_all_matched() {
        let mut result = VerificationResult::default();
        result.total_events = 10;
        result.matched = 10;
        result.mismatched = 0;

        assert!(result.all_matched());
    }

    #[test]
    fn verification_result_with_mismatches() {
        let mut result = VerificationResult::default();
        result.total_events = 10;
        result.matched = 8;
        result.mismatched = 2;

        assert!(!result.all_matched());
    }
}
