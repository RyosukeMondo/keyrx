//! Recording configuration and management for engine sessions.

use crate::cli::OutputWriter;
use crate::engine::{
    infer_decision_type, AdvancedEngine, EventRecordBuilder, EventRecorder, InputEvent,
    OutputAction, TimingConfig,
};
use crate::scripting::RhaiRuntime;
use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::time::Instant;

/// Manages session recording for engine runs.
pub struct RecordingManager<'a> {
    record_path: Option<PathBuf>,
    output: &'a OutputWriter,
}

impl<'a> RecordingManager<'a> {
    /// Create a new recording manager.
    pub fn new(record_path: Option<PathBuf>, output: &'a OutputWriter) -> Self {
        Self {
            record_path,
            output,
        }
    }

    /// Create an EventRecorder if a record path is specified.
    pub fn create_recorder(
        &self,
        engine: &AdvancedEngine<RhaiRuntime>,
        script_path: Option<String>,
        timing_config: TimingConfig,
    ) -> Result<Option<EventRecorder>> {
        match &self.record_path {
            Some(path) => {
                self.output
                    .success(&format!("Recording session to: {}", path.display()));
                let initial_state = engine.snapshot();
                EventRecorder::new(path, script_path, timing_config, initial_state)
                    .map(Some)
                    .map_err(|e| anyhow!("Failed to create recorder: {}", e))
            }
            None => Ok(None),
        }
    }

    /// Finish recording and save to file.
    pub fn finish_recording(&self, recorder: Option<EventRecorder>) -> Result<()> {
        if let Some(rec) = recorder {
            let count = rec.event_count();
            match rec.finish() {
                Ok(session) => {
                    self.output.success(&format!(
                        "Session saved: {} events, avg latency {}µs",
                        count,
                        session.avg_latency_us()
                    ));
                }
                Err(e) => {
                    self.output.error(&format!("Failed to save session: {}", e));
                }
            }
        }
        Ok(())
    }
}

/// Context for recording individual events during processing.
pub struct RecordingContext<'a> {
    recorder: &'a mut Option<EventRecorder>,
    seq: &'a mut u64,
}

impl<'a> RecordingContext<'a> {
    /// Create a new recording context.
    pub fn new(recorder: &'a mut Option<EventRecorder>, seq: &'a mut u64) -> Self {
        Self { recorder, seq }
    }

    /// Record a single event with its output and timing.
    pub fn record_event(
        &mut self,
        event: &InputEvent,
        outputs: &[OutputAction],
        engine: &AdvancedEngine<RhaiRuntime>,
        process_start: Instant,
    ) {
        if let Some(ref mut rec) = self.recorder {
            let latency_us = process_start.elapsed().as_micros() as u64;
            let snapshot = engine.snapshot();
            let active_layers: Vec<u32> = snapshot.layers.active_layer_ids();
            rec.record_event(
                EventRecordBuilder::new()
                    .seq(*self.seq)
                    .timestamp_us(event.timestamp_us)
                    .input(event.clone())
                    .output(outputs.to_vec())
                    .decision_type(infer_decision_type(event, outputs))
                    .active_layers(active_layers)
                    .modifiers_state(snapshot.modifiers)
                    .latency_us(latency_us)
                    .build(),
            );
            *self.seq += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::OutputFormat;

    #[test]
    fn create_recorder_returns_none_when_no_path() {
        let output = OutputWriter::new(OutputFormat::Human);
        let manager = RecordingManager::new(None, &output);
        let runtime = RhaiRuntime::new().unwrap();
        let registry = runtime.registry().clone();
        let engine = AdvancedEngine::new(runtime, registry.timing_config().clone());

        let result = manager
            .create_recorder(&engine, None, Default::default())
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn finish_recording_handles_none() {
        let output = OutputWriter::new(OutputFormat::Human);
        let manager = RecordingManager::new(None, &output);
        // Should not panic
        manager.finish_recording(None).unwrap();
    }
}
