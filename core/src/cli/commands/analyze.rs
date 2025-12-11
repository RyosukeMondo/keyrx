//! Session analysis command.
//!
//! Analyzes a recorded `.krx` session file and generates timing diagrams
//! and statistics for debugging and performance analysis.

use crate::cli::{Command, CommandContext, CommandResult, ExitCode, OutputFormat, OutputWriter};
use crate::engine::{DecisionType, SessionFile};
use anyhow::Context;
use serde::Serialize;
use std::path::PathBuf;

/// Analysis result for a session.
#[derive(Debug, Serialize)]
pub struct AnalysisResult {
    /// Session file path.
    pub session_path: String,
    /// Total number of events.
    pub event_count: usize,
    /// Total session duration in microseconds.
    pub duration_us: u64,
    /// Average latency in microseconds.
    pub avg_latency_us: u64,
    /// Minimum latency in microseconds.
    pub min_latency_us: u64,
    /// Maximum latency in microseconds.
    pub max_latency_us: u64,
    /// Events by decision type.
    pub decision_breakdown: DecisionBreakdown,
}

/// Breakdown of events by decision type.
#[derive(Debug, Default, Serialize)]
pub struct DecisionBreakdown {
    pub pass_through: usize,
    pub remap: usize,
    pub block: usize,
    pub tap: usize,
    pub hold: usize,
    pub combo: usize,
    pub layer: usize,
    pub modifier: usize,
}

impl DecisionBreakdown {
    fn increment(&mut self, decision_type: DecisionType) {
        match decision_type {
            DecisionType::PassThrough => self.pass_through += 1,
            DecisionType::Remap => self.remap += 1,
            DecisionType::Block => self.block += 1,
            DecisionType::Tap => self.tap += 1,
            DecisionType::Hold => self.hold += 1,
            DecisionType::Combo => self.combo += 1,
            DecisionType::Layer => self.layer += 1,
            DecisionType::Modifier => self.modifier += 1,
        }
    }
}

/// Analyze a recorded session.
pub struct AnalyzeCommand {
    /// Path to the .krx session file.
    pub session_path: PathBuf,
    /// Whether to generate ASCII timing diagram.
    pub diagram: bool,
    /// Output writer.
    pub output: OutputWriter,
}

impl AnalyzeCommand {
    /// Create a new analyze command.
    pub fn new(session_path: PathBuf, format: OutputFormat) -> Self {
        Self {
            session_path,
            diagram: false,
            output: OutputWriter::new(format),
        }
    }

    /// Enable timing diagram output.
    pub fn with_diagram(mut self, diagram: bool) -> Self {
        self.diagram = diagram;
        self
    }

    /// Run the analyze command.
    pub fn run(&self) -> CommandResult<AnalysisResult> {
        self.output.success(&format!(
            "Analyzing session: {}",
            self.session_path.display()
        ));

        let content = match std::fs::read_to_string(&self.session_path)
            .with_context(|| format!("Failed to read session: {}", self.session_path.display()))
        {
            Ok(c) => c,
            Err(e) => return CommandResult::failure(ExitCode::GeneralError, format!("{:#}", e)),
        };

        let session = match SessionFile::from_json(&content)
            .with_context(|| format!("Failed to parse session: {}", self.session_path.display()))
        {
            Ok(s) => s,
            Err(e) => return CommandResult::failure(ExitCode::GeneralError, format!("{:#}", e)),
        };

        let result = self.analyze_session(&session);

        if self.diagram {
            self.print_timing_diagram(&session);
        }

        self.print_summary(&result);

        if matches!(
            self.output.format(),
            OutputFormat::Json | OutputFormat::Yaml
        ) {
            if let Err(e) = self.output.data(&result) {
                return CommandResult::failure(
                    ExitCode::GeneralError,
                    format!("Failed to output results: {}", e),
                );
            }
        }

        CommandResult::success(result)
    }

    fn analyze_session(&self, session: &SessionFile) -> AnalysisResult {
        let mut breakdown = DecisionBreakdown::default();
        let mut min_latency = u64::MAX;
        let mut max_latency = 0u64;

        for event in &session.events {
            breakdown.increment(event.decision_type);
            min_latency = min_latency.min(event.latency_us);
            max_latency = max_latency.max(event.latency_us);
        }

        // Handle empty session
        if session.events.is_empty() {
            min_latency = 0;
        }

        AnalysisResult {
            session_path: self.session_path.display().to_string(),
            event_count: session.event_count(),
            duration_us: session.duration_us(),
            avg_latency_us: session.avg_latency_us(),
            min_latency_us: min_latency,
            max_latency_us: max_latency,
            decision_breakdown: breakdown,
        }
    }

    fn print_timing_diagram(&self, session: &SessionFile) {
        // Print header
        println!();
        println!(
            "┌{:─<6}┬{:─<15}┬{:─<15}┬{:─<15}┬{:─<10}┬{:─<12}┐",
            "", "", "", "", "", ""
        );
        println!(
            "│{:^6}│{:^15}│{:^15}│{:^15}│{:^10}│{:^12}│",
            "Seq", "Input", "Decision", "Output", "Latency", "Timestamp"
        );
        println!(
            "├{:─<6}┼{:─<15}┼{:─<15}┼{:─<15}┼{:─<10}┼{:─<12}┤",
            "", "", "", "", "", ""
        );

        // Print events
        for event in &session.events {
            let input = format_input(&event.input);
            let decision = format_decision(event.decision_type);
            let output_str = format_output(&event.output);
            let latency = format!("{}µs", event.latency_us);
            let timestamp = format!("{}ms", event.timestamp_us / 1000);

            println!(
                "│{:>5} │{:<14} │{:<14} │{:<14} │{:>9} │{:>11} │",
                event.seq, input, decision, output_str, latency, timestamp
            );
        }

        // Print footer
        println!(
            "└{:─<6}┴{:─<15}┴{:─<15}┴{:─<15}┴{:─<10}┴{:─<12}┘",
            "", "", "", "", "", ""
        );
        println!();
    }

    fn print_summary(&self, result: &AnalysisResult) {
        self.output.success(&format!(
            "Events: {}, Duration: {}ms, Avg latency: {}µs",
            result.event_count,
            result.duration_us / 1000,
            result.avg_latency_us
        ));

        self.output.success(&format!(
            "Latency range: {}µs - {}µs",
            result.min_latency_us, result.max_latency_us
        ));

        let bd = &result.decision_breakdown;
        let breakdown_parts: Vec<String> = [
            ("pass", bd.pass_through),
            ("remap", bd.remap),
            ("block", bd.block),
            ("tap", bd.tap),
            ("hold", bd.hold),
            ("combo", bd.combo),
            ("layer", bd.layer),
            ("mod", bd.modifier),
        ]
        .iter()
        .filter(|(_, count)| *count > 0)
        .map(|(name, count)| format!("{}: {}", name, count))
        .collect();

        if !breakdown_parts.is_empty() {
            self.output
                .success(&format!("Decisions: {}", breakdown_parts.join(", ")));
        }
    }
}

fn format_input(input: &crate::engine::InputEvent) -> String {
    let key_str = format!("{:?}", input.key);
    let action = if input.pressed { "↓" } else { "↑" };
    format!("{}{}", key_str, action)
}

fn format_decision(decision: DecisionType) -> String {
    match decision {
        DecisionType::PassThrough => "PassThrough".to_string(),
        DecisionType::Remap => "Remap".to_string(),
        DecisionType::Block => "Block".to_string(),
        DecisionType::Tap => "Tap".to_string(),
        DecisionType::Hold => "Hold".to_string(),
        DecisionType::Combo => "Combo".to_string(),
        DecisionType::Layer => "Layer".to_string(),
        DecisionType::Modifier => "Modifier".to_string(),
    }
}

fn format_output(output: &[crate::engine::OutputAction]) -> String {
    if output.is_empty() {
        return "-".to_string();
    }

    let formatted: Vec<String> = output
        .iter()
        .take(2) // Limit to first 2 for display
        .map(|action| match action {
            crate::engine::OutputAction::KeyDown(k) => format!("{:?}↓", k),
            crate::engine::OutputAction::KeyUp(k) => format!("{:?}↑", k),
            crate::engine::OutputAction::KeyTap(k) => format!("{:?}⇅", k),
            crate::engine::OutputAction::Block => "Block".to_string(),
            crate::engine::OutputAction::PassThrough => "Pass".to_string(),
        })
        .collect();

    let result = formatted.join(",");
    if output.len() > 2 {
        format!("{}...", result)
    } else {
        result
    }
}

impl Command for AnalyzeCommand {
    fn name(&self) -> &str {
        "analyze"
    }

    fn execute(&mut self, _ctx: &CommandContext) -> CommandResult<()> {
        // Run and discard the result value since Command trait returns ()
        self.run().map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{
        EventRecordBuilder, InputEvent, KeyCode, ModifierState, OutputAction, TimingConfig,
    };
    use tempfile::TempDir;

    fn make_test_session() -> SessionFile {
        // Create engine state with new API and convert to snapshot
        let engine = crate::engine::AdvancedEngine::new(
            crate::scripting::RhaiRuntime::new().unwrap(),
            TimingConfig::default(),
        );
        let initial_state = engine.snapshot();

        let mut session = SessionFile::new(
            "2024-01-15T10:30:00Z".to_string(),
            None,
            TimingConfig::default(),
            initial_state,
        );

        // Add various events with different decision types
        let events = [
            (KeyCode::A, DecisionType::PassThrough, 50),
            (KeyCode::CapsLock, DecisionType::Remap, 75),
            (KeyCode::B, DecisionType::Tap, 100),
        ];

        for (i, (key, decision, latency)) in events.iter().enumerate() {
            let input = InputEvent::key_down(*key, (i * 10_000) as u64);
            session.add_event(
                EventRecordBuilder::new()
                    .seq(i as u64)
                    .timestamp_us((i * 10_000) as u64)
                    .input(input)
                    .output(vec![OutputAction::KeyDown(*key)])
                    .decision_type(*decision)
                    .active_layers(vec![0])
                    .modifiers_state(ModifierState::default())
                    .latency_us(*latency)
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

    #[test]
    fn analyze_command_loads_session() {
        let dir = TempDir::new().expect("create temp dir");
        let session = make_test_session();
        let path = write_session_file(&dir, &session);

        let cmd = AnalyzeCommand::new(path, OutputFormat::Human);
        let cmd_result = cmd.run();
        assert!(cmd_result.is_success());
        let result = cmd_result.value().expect("analyze should succeed");

        assert_eq!(result.event_count, 3);
        assert_eq!(result.duration_us, 20_000);
    }

    #[test]
    fn analyze_computes_latency_stats() {
        let dir = TempDir::new().expect("create temp dir");
        let session = make_test_session();
        let path = write_session_file(&dir, &session);

        let cmd = AnalyzeCommand::new(path, OutputFormat::Human);
        let cmd_result = cmd.run();
        assert!(cmd_result.is_success());
        let result = cmd_result.value().expect("analyze should succeed");

        assert_eq!(result.min_latency_us, 50);
        assert_eq!(result.max_latency_us, 100);
        assert_eq!(result.avg_latency_us, 75); // (50+75+100)/3 = 75
    }

    #[test]
    fn analyze_counts_decision_types() {
        let dir = TempDir::new().expect("create temp dir");
        let session = make_test_session();
        let path = write_session_file(&dir, &session);

        let cmd = AnalyzeCommand::new(path, OutputFormat::Human);
        let cmd_result = cmd.run();
        assert!(cmd_result.is_success());
        let result = cmd_result.value().expect("analyze should succeed");

        assert_eq!(result.decision_breakdown.pass_through, 1);
        assert_eq!(result.decision_breakdown.remap, 1);
        assert_eq!(result.decision_breakdown.tap, 1);
        assert_eq!(result.decision_breakdown.block, 0);
    }

    #[test]
    fn analyze_fails_on_missing_file() {
        let cmd = AnalyzeCommand::new(
            PathBuf::from("/nonexistent/session.krx"),
            OutputFormat::Human,
        );
        let result = cmd.run();

        assert!(result.is_failure());
    }

    #[test]
    fn analyze_with_diagram_flag() {
        let dir = TempDir::new().expect("create temp dir");
        let session = make_test_session();
        let path = write_session_file(&dir, &session);

        let cmd = AnalyzeCommand::new(path, OutputFormat::Human).with_diagram(true);
        let cmd_result = cmd.run();
        assert!(cmd_result.is_success());
        let result = cmd_result.value().expect("analyze should succeed");

        // Diagram is printed to stdout, verify command succeeds
        assert_eq!(result.event_count, 3);
    }

    #[test]
    fn format_input_shows_key_and_direction() {
        let input_down = InputEvent::key_down(KeyCode::A, 0);
        let input_up = InputEvent::key_up(KeyCode::A, 0);

        assert!(format_input(&input_down).contains("A"));
        assert!(format_input(&input_down).contains("↓"));
        assert!(format_input(&input_up).contains("↑"));
    }

    #[test]
    fn format_output_handles_multiple_actions() {
        let single = vec![OutputAction::KeyDown(KeyCode::B)];
        assert!(format_output(&single).contains("B"));

        let empty: Vec<OutputAction> = vec![];
        assert_eq!(format_output(&empty), "-");

        let many = vec![
            OutputAction::KeyDown(KeyCode::A),
            OutputAction::KeyDown(KeyCode::B),
            OutputAction::KeyDown(KeyCode::C),
        ];
        assert!(format_output(&many).contains("..."));
    }

    #[test]
    fn decision_breakdown_increments_correctly() {
        let mut breakdown = DecisionBreakdown::default();

        breakdown.increment(DecisionType::PassThrough);
        breakdown.increment(DecisionType::PassThrough);
        breakdown.increment(DecisionType::Remap);

        assert_eq!(breakdown.pass_through, 2);
        assert_eq!(breakdown.remap, 1);
        assert_eq!(breakdown.block, 0);
    }

    #[test]
    fn empty_session_handles_gracefully() {
        let dir = TempDir::new().expect("create temp dir");

        // Create engine state with new API and convert to snapshot
        let engine = crate::engine::AdvancedEngine::new(
            crate::scripting::RhaiRuntime::new().unwrap(),
            TimingConfig::default(),
        );
        let initial_state = engine.snapshot();

        let session = SessionFile::new(
            "2024-01-15T10:30:00Z".to_string(),
            None,
            TimingConfig::default(),
            initial_state,
        );

        let path = write_session_file(&dir, &session);

        let cmd = AnalyzeCommand::new(path, OutputFormat::Human);
        let cmd_result = cmd.run();
        assert!(cmd_result.is_success());
        let result = cmd_result.value().expect("analyze should succeed");

        assert_eq!(result.event_count, 0);
        assert_eq!(result.min_latency_us, 0);
        assert_eq!(result.max_latency_us, 0);
    }
}
