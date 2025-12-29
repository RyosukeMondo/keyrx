//! Test CLI commands.
//!
//! This module implements the `keyrx test` command for autonomous testing
//! using built-in scenarios. Provides pass/fail reporting for configuration
//! validation.

use crate::config::simulation_engine::{BuiltinScenario, ScenarioResult, SimulationEngine};
use clap::Args;
use serde::Serialize;
use std::path::PathBuf;

/// Test subcommands.
#[derive(Args)]
pub struct TestArgs {
    /// Profile name to test (defaults to current active profile).
    #[arg(long)]
    pub profile: Option<String>,

    /// Scenario name to run (or "all" for all scenarios).
    #[arg(long, default_value = "all")]
    pub scenario: String,

    /// Output as JSON.
    #[arg(long)]
    pub json: bool,
}

/// JSON output structure for test results.
#[derive(Serialize)]
struct TestOutput {
    success: bool,
    profile: String,
    total: usize,
    passed: usize,
    failed: usize,
    results: Vec<ScenarioResult>,
}

/// Execute the test command.
pub fn execute(args: TestArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Determine KRX file path
    let krx_path = resolve_krx_path(args.profile.as_deref())?;
    let profile_name = args
        .profile
        .clone()
        .unwrap_or_else(|| "default".to_string());

    // Create simulation engine
    let mut engine = SimulationEngine::new(&krx_path)?;

    // Run scenarios
    let results = if args.scenario == "all" {
        engine.run_all_scenarios()?
    } else {
        // Parse scenario name
        let scenario = parse_scenario_name(&args.scenario)?;
        vec![engine.run_scenario(scenario)?]
    };

    // Calculate pass/fail counts
    let total = results.len();
    let passed = results.iter().filter(|r| r.passed).count();
    let failed = total - passed;
    let success = failed == 0;

    // Output results
    if args.json {
        print_json_output(&profile_name, total, passed, failed, success, &results)?;
    } else {
        print_human_output(&profile_name, total, passed, failed, &results);
    }

    // Return error if any tests failed (main.rs will call std::process::exit(1))
    if !success {
        return Err(format!("{} of {} tests failed", failed, total).into());
    }

    Ok(())
}

/// Parse scenario name string into BuiltinScenario enum.
fn parse_scenario_name(name: &str) -> Result<BuiltinScenario, Box<dyn std::error::Error>> {
    match name {
        "tap-hold-under-threshold" => Ok(BuiltinScenario::TapHoldUnderThreshold),
        "tap-hold-over-threshold" => Ok(BuiltinScenario::TapHoldOverThreshold),
        "permissive-hold" => Ok(BuiltinScenario::PermissiveHold),
        "cross-device-modifiers" => Ok(BuiltinScenario::CrossDeviceModifiers),
        "macro-sequence" => Ok(BuiltinScenario::MacroSequence),
        _ => Err(format!(
            "Unknown scenario: '{}'. Available scenarios: tap-hold-under-threshold, \
             tap-hold-over-threshold, permissive-hold, cross-device-modifiers, macro-sequence",
            name
        )
        .into()),
    }
}

/// Resolve KRX file path from profile name or use active profile.
fn resolve_krx_path(profile: Option<&str>) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let config_dir = get_config_dir()?;

    let profile_name = if let Some(name) = profile {
        name.to_string()
    } else {
        "default".to_string()
    };

    let krx_path = config_dir
        .join("profiles")
        .join(format!("{}.krx", profile_name));

    if !krx_path.exists() {
        return Err(format!("Profile not found: {}", profile_name).into());
    }

    Ok(krx_path)
}

/// Get config directory path.
fn get_config_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| "Could not determine home directory")?;

    Ok(PathBuf::from(home).join(".config").join("keyrx"))
}

/// Print JSON output.
fn print_json_output(
    profile: &str,
    total: usize,
    passed: usize,
    failed: usize,
    success: bool,
    results: &[ScenarioResult],
) -> Result<(), Box<dyn std::error::Error>> {
    let output = TestOutput {
        success,
        profile: profile.to_string(),
        total,
        passed,
        failed,
        results: results.to_vec(),
    };

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

/// Print human-readable output.
fn print_human_output(
    profile: &str,
    total: usize,
    passed: usize,
    failed: usize,
    results: &[ScenarioResult],
) {
    println!("Testing profile: {}", profile);
    println!();

    for result in results {
        let status = if result.passed {
            "✓ PASS"
        } else {
            "✗ FAIL"
        };
        println!("  {} {}", status, result.scenario);

        if !result.passed {
            if let Some(error) = &result.error {
                println!("    Error: {}", error);
            }
            println!("    Input events: {}", result.input.len());
            println!("    Output events: {}", result.output.len());
        }
    }

    println!();
    println!(
        "Results: {} total, {} passed, {} failed",
        total, passed, failed
    );

    if failed == 0 {
        println!("All tests passed!");
    } else {
        println!("Some tests failed.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[allow(dead_code)]
    fn create_test_profile(dir: &TempDir, name: &str) -> PathBuf {
        let profiles_dir = dir.path().join("profiles");
        std::fs::create_dir_all(&profiles_dir).unwrap();

        let krx_path = profiles_dir.join(format!("{}.krx", name));
        let mut file = std::fs::File::create(&krx_path).unwrap();
        file.write_all(b"test krx data").unwrap();

        krx_path
    }

    #[test]
    fn test_parse_scenario_name_valid() {
        assert!(matches!(
            parse_scenario_name("tap-hold-under-threshold"),
            Ok(BuiltinScenario::TapHoldUnderThreshold)
        ));
        assert!(matches!(
            parse_scenario_name("tap-hold-over-threshold"),
            Ok(BuiltinScenario::TapHoldOverThreshold)
        ));
        assert!(matches!(
            parse_scenario_name("permissive-hold"),
            Ok(BuiltinScenario::PermissiveHold)
        ));
        assert!(matches!(
            parse_scenario_name("cross-device-modifiers"),
            Ok(BuiltinScenario::CrossDeviceModifiers)
        ));
        assert!(matches!(
            parse_scenario_name("macro-sequence"),
            Ok(BuiltinScenario::MacroSequence)
        ));
    }

    #[test]
    fn test_parse_scenario_name_invalid() {
        let result = parse_scenario_name("invalid-scenario");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown scenario"));
    }

    // Note: These unit tests modify global environment and should be run serially
    // Remove these tests and rely on integration tests in tests/cli_test_test.rs instead

    #[test]
    fn test_json_output_format() {
        let results = vec![ScenarioResult {
            scenario: "test-scenario".to_string(),
            passed: true,
            input: vec![],
            output: vec![],
            error: None,
        }];

        let result = print_json_output("test", 1, 1, 0, true, &results);
        assert!(result.is_ok());
    }
}
