//! Simulation CLI commands.
//!
//! This module implements the `keyrx simulate` command for deterministic
//! event replay testing. Supports inline event DSL, event files, and
//! seed-based determinism.

use crate::config::simulation_engine::{
    EventSequence, OutputEvent, SimulatedEvent, SimulationEngine,
};
use clap::Args;
use serde::Serialize;
use std::path::PathBuf;

/// Simulation subcommands.
#[derive(Args, Debug)]
pub struct SimulateArgs {
    /// Profile name to simulate (defaults to current active profile).
    #[arg(long)]
    profile: Option<String>,

    /// Inline event DSL (e.g., "press:A,wait:50,release:A").
    #[arg(long, conflicts_with = "events_file")]
    events: Option<String>,

    /// Event file path (JSON format).
    #[arg(long, conflicts_with = "events")]
    events_file: Option<PathBuf>,

    /// Seed for deterministic behavior.
    #[arg(long, default_value = "0")]
    seed: u64,

    /// Output as JSON.
    #[arg(long)]
    json: bool,
}

/// JSON output structure for simulation.
#[derive(Serialize)]
struct SimulationOutput {
    success: bool,
    input: Vec<SimulatedEvent>,
    output: Vec<OutputEvent>,
    seed: u64,
    error: Option<String>,
}

/// Execute the simulate command.
pub fn execute(args: SimulateArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Determine KRX file path
    let krx_path = resolve_krx_path(args.profile.as_deref())?;

    // Create simulation engine
    let mut engine = SimulationEngine::new(&krx_path)?;

    // Load event sequence
    let sequence = if let Some(events_file) = args.events_file {
        SimulationEngine::load_events_from_file(&events_file)?
    } else if let Some(events_dsl) = args.events {
        SimulationEngine::parse_event_dsl(&events_dsl, args.seed)?
    } else {
        return Err("Either --events or --events-file must be specified".into());
    };

    // Run simulation
    let result = engine.replay(&sequence);

    // Output results
    match result {
        Ok(output) => {
            if args.json {
                print_json_output(&sequence, &output, sequence.seed, None)?;
            } else {
                print_human_output(&sequence, &output, sequence.seed);
            }
            Ok(())
        }
        Err(e) => {
            if args.json {
                print_json_output(&sequence, &[], sequence.seed, Some(e.to_string()))?;
            } else {
                eprintln!("Error: {}", e);
            }
            std::process::exit(1);
        }
    }
}

/// Resolve KRX file path from profile name or use active profile.
fn resolve_krx_path(profile: Option<&str>) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let config_dir = get_config_dir()?;

    let profile_name = if let Some(name) = profile {
        name.to_string()
    } else {
        // Read active profile from config
        // For now, use "default" as fallback
        "default".to_string()
    };

    let krx_path = config_dir
        .join("profiles")
        .join(format!("{}.krx", profile_name));

    if !krx_path.exists() {
        return Err(format!(
            "Profile '{}' not found ({})",
            profile_name,
            krx_path.display()
        )
        .into());
    }

    Ok(krx_path)
}

/// Get configuration directory.
fn get_config_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    crate::cli::config_dir::get_config_dir()
}

/// Print human-readable output.
fn print_human_output(sequence: &EventSequence, output: &[OutputEvent], seed: u64) {
    println!("Simulation Results (seed: {})", seed);
    println!();
    println!("Input Events ({}):", sequence.events.len());
    for event in &sequence.events {
        let device = event.device_id.as_deref().unwrap_or("default");
        println!(
            "  [{:>8} us] {:?} {} (device: {})",
            event.timestamp_us, event.event_type, event.key, device
        );
    }

    println!();
    println!("Output Events ({}):", output.len());
    for event in output {
        println!(
            "  [{:>8} us] {:?} {}",
            event.timestamp_us, event.event_type, event.key
        );
    }
}

/// Print JSON output.
fn print_json_output(
    sequence: &EventSequence,
    output: &[OutputEvent],
    seed: u64,
    error: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let output_data = SimulationOutput {
        success: error.is_none(),
        input: sequence.events.clone(),
        output: output.to_vec(),
        seed,
        error,
    };

    println!("{}", serde_json::to_string_pretty(&output_data)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use tempfile::TempDir;

    fn create_test_environment() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory for test");
        let config_dir = temp_dir.path().to_path_buf();
        let profiles_dir = config_dir.join("profiles");
        std::fs::create_dir_all(&profiles_dir)
            .expect("Failed to create profiles directory for test");

        let krx_path = profiles_dir.join("default.krx");
        std::fs::write(&krx_path, b"test krx data").expect("Failed to write test KRX file");

        (temp_dir, config_dir)
    }

    #[test]
    #[serial]
    fn test_resolve_krx_path_with_profile() {
        let (_temp_dir, config_dir) = create_test_environment();
        let old = std::env::var("KEYRX_CONFIG_DIR").ok();
        std::env::set_var("KEYRX_CONFIG_DIR", &config_dir);

        let result = resolve_krx_path(Some("default"));
        assert!(result.is_ok());
        assert!(result
            .expect("resolve_krx_path should succeed")
            .ends_with("default.krx"));

        match old {
            Some(v) => std::env::set_var("KEYRX_CONFIG_DIR", v),
            None => std::env::remove_var("KEYRX_CONFIG_DIR"),
        }
    }

    #[test]
    #[serial]
    fn test_resolve_krx_path_not_found() {
        let (_temp_dir, config_dir) = create_test_environment();
        let old = std::env::var("KEYRX_CONFIG_DIR").ok();
        std::env::set_var("KEYRX_CONFIG_DIR", &config_dir);

        let result = resolve_krx_path(Some("nonexistent"));
        assert!(result.is_err());

        match old {
            Some(v) => std::env::set_var("KEYRX_CONFIG_DIR", v),
            None => std::env::remove_var("KEYRX_CONFIG_DIR"),
        }
    }

    #[test]
    #[serial]
    fn test_get_config_dir_from_env() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory for test");
        let old = std::env::var("KEYRX_CONFIG_DIR").ok();
        std::env::set_var("KEYRX_CONFIG_DIR", temp_dir.path());

        let result = get_config_dir().expect("get_config_dir should succeed");
        assert_eq!(result, temp_dir.path());

        match old {
            Some(v) => std::env::set_var("KEYRX_CONFIG_DIR", v),
            None => std::env::remove_var("KEYRX_CONFIG_DIR"),
        }
    }

    #[test]
    fn test_simulation_output_json() {
        let events = vec![SimulatedEvent {
            device_id: None,
            timestamp_us: 0,
            key: "A".to_string(),
            event_type: crate::config::simulation_engine::EventType::Press,
        }];

        let output = vec![OutputEvent {
            key: "A".to_string(),
            event_type: crate::config::simulation_engine::EventType::Press,
            timestamp_us: 0,
        }];

        let sequence = EventSequence { events, seed: 42 };

        let result = print_json_output(&sequence, &output, 42, None);
        assert!(result.is_ok());
    }
}
