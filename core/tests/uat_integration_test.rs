#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::print_stdout, clippy::print_stderr, clippy::field_reassign_with_default, clippy::useless_conversion, clippy::assertions_on_constants, clippy::manual_div_ceil, clippy::manual_strip, clippy::len_zero, clippy::redundant_closure, clippy::manual_range_contains, clippy::default_constructed_unit_structs, clippy::clone_on_copy, clippy::io_other_error, clippy::bool_assert_comparison, clippy::approx_constant, clippy::let_unit_value, clippy::while_let_on_iterator, clippy::await_holding_lock, clippy::unnecessary_cast, clippy::drop_non_drop, clippy::needless_range_loop, unused_imports, unused_variables, dead_code, unsafe_code, clippy::collapsible_if, clippy::bool_comparison, unexpected_cfgs)]
//! Integration tests for UAT system.
//!
//! Tests the complete UAT workflow including test discovery,
//! execution, quality gate evaluation, and golden session verification.

use keyrx_core::uat::{
    GoldenSessionManager, QualityGate, QualityGateEnforcer, UatFilter, UatRunner,
};
use std::fs;
use tempfile::TempDir;

/// Test that the full UAT workflow executes correctly.
#[test]
fn uat_full_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.rhai");

    // Create a test file with metadata
    let script = r#"
// @category: core
// @priority: P0
// @requirement: 1.1
fn uat_basic_math() {
    let x = 1 + 1;
    if x != 2 {
        throw "Math is broken";
    }
}

// @category: layers
// @priority: P1
fn uat_string_ops() {
    let s = "hello";
    if s.len() != 5 {
        throw "String length wrong";
    }
}
"#;
    fs::write(&test_file, script).unwrap();

    // 1. Discover tests
    let runner = UatRunner::with_test_dir(temp_dir.path());
    let tests = runner.discover();
    assert_eq!(tests.len(), 2, "Should discover 2 tests");

    // 2. Run tests
    let results = runner.run(&UatFilter::default());
    assert_eq!(results.total, 2);
    assert_eq!(results.passed, 2);
    assert_eq!(results.failed, 0);

    // 3. Evaluate against quality gate
    let enforcer = QualityGateEnforcer::new();
    let gate = QualityGate::default();
    let gate_result = enforcer.evaluate(&gate, &results);
    assert!(gate_result.passed, "Gate should pass with 100% pass rate");
}

/// Test category filtering in integration.
#[test]
fn uat_category_filtering() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.rhai");

    let script = r#"
// @category: core
fn uat_core_test() { }

// @category: layers
fn uat_layer_test() { }

// @category: performance
fn uat_perf_test() { }
"#;
    fs::write(&test_file, script).unwrap();

    let runner = UatRunner::with_test_dir(temp_dir.path());

    // Filter by core category
    let filter = UatFilter {
        categories: vec!["core".to_string()],
        ..Default::default()
    };
    let results = runner.run(&filter);
    assert_eq!(results.total, 1, "Should only run core tests");

    // Filter by multiple categories
    let filter = UatFilter {
        categories: vec!["core".to_string(), "layers".to_string()],
        ..Default::default()
    };
    let results = runner.run(&filter);
    assert_eq!(results.total, 2, "Should run core and layers tests");
}

/// Test priority filtering in integration.
#[test]
fn uat_priority_filtering() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.rhai");

    let script = r#"
// @priority: P0
fn uat_critical() { }

// @priority: P1
fn uat_high() { }

// @priority: P2
fn uat_normal() { }
"#;
    fs::write(&test_file, script).unwrap();

    let runner = UatRunner::with_test_dir(temp_dir.path());

    // Filter by P0 only
    let filter = UatFilter {
        priorities: vec![keyrx_core::uat::Priority::P0],
        ..Default::default()
    };
    let results = runner.run(&filter);
    assert_eq!(results.total, 1, "Should only run P0 tests");

    // Filter by P0 and P1
    let filter = UatFilter {
        priorities: vec![keyrx_core::uat::Priority::P0, keyrx_core::uat::Priority::P1],
        ..Default::default()
    };
    let results = runner.run(&filter);
    assert_eq!(results.total, 2, "Should run P0 and P1 tests");
}

/// Test quality gate evaluation with failures.
#[test]
fn uat_gate_evaluation_with_failures() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.rhai");

    // Create tests that will fail
    let script = r#"
// @priority: P0
fn uat_failing_p0() {
    throw "P0 test fails";
}

// @priority: P1
fn uat_failing_p1() {
    throw "P1 test fails";
}

fn uat_passing() {
    let x = 1;
}
"#;
    fs::write(&test_file, script).unwrap();

    let runner = UatRunner::with_test_dir(temp_dir.path());
    let results = runner.run(&UatFilter::default());

    assert_eq!(results.total, 3);
    assert_eq!(results.passed, 1);
    assert_eq!(results.failed, 2);

    // Default gate should fail (0 P0 allowed)
    let enforcer = QualityGateEnforcer::new();
    let gate = QualityGate::default();
    let gate_result = enforcer.evaluate(&gate, &results);

    assert!(!gate_result.passed);
    assert!(
        gate_result
            .violations
            .iter()
            .any(|v| v.criterion == "p0_open"),
        "Should have P0 violation"
    );
}

/// Test that tests execute and report failures.
#[test]
fn uat_reports_failures_correctly() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.rhai");

    let script = r#"
fn uat_test_a() { }

fn uat_test_b() {
    throw "This fails";
}

fn uat_test_c() { }

fn uat_test_d() { }
"#;
    fs::write(&test_file, script).unwrap();

    let runner = UatRunner::with_test_dir(temp_dir.path());

    // All tests should run
    let filter = UatFilter::default();
    let results = runner.run(&filter);
    assert_eq!(results.total, 4);
    assert_eq!(results.passed, 3, "3 tests should pass");
    assert_eq!(results.failed, 1, "1 test should fail");

    // Verify the failure details
    let failed_result = results.results.iter().find(|r| !r.passed).unwrap();
    assert_eq!(failed_result.test.name, "uat_test_b");
    assert!(failed_result.error.is_some());
}

/// Test golden session record and verify workflow.
#[test]
fn golden_session_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let script_file = temp_dir.path().join("test_script.rhai");

    // Create a simple script
    let script = r#"
fn on_init() {
    // Simple init
}
"#;
    fs::write(&script_file, script).unwrap();

    // Create manager with custom directory
    let golden_dir = temp_dir.path().join("golden");
    fs::create_dir(&golden_dir).unwrap();

    let manager = GoldenSessionManager::with_dir(&golden_dir);

    // Record session - pass script path as string
    let script_path_str = script_file.to_string_lossy();
    let record_result = manager.record("test_session", &script_path_str);
    assert!(record_result.is_ok(), "Recording should succeed");

    // Verify session exists
    let sessions = manager.list_sessions().unwrap();
    assert!(sessions.contains(&"test_session".to_string()));

    // Verify session
    let verify_result = manager.verify("test_session");
    assert!(verify_result.is_ok(), "Verification should succeed");
    assert!(verify_result.unwrap().passed, "Verification should pass");
}

/// Test requirements traceability.
#[test]
fn uat_requirements_traceability() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.rhai");

    let script = r#"
// @requirement: REQ-001, REQ-002
fn uat_feature_a() { }

// @requirement: REQ-003
fn uat_feature_b() { }

fn uat_no_req() { }
"#;
    fs::write(&test_file, script).unwrap();

    let runner = UatRunner::with_test_dir(temp_dir.path());
    let tests = runner.discover();

    let feature_a = tests.iter().find(|t| t.name == "uat_feature_a").unwrap();
    assert_eq!(feature_a.requirements, vec!["REQ-001", "REQ-002"]);

    let feature_b = tests.iter().find(|t| t.name == "uat_feature_b").unwrap();
    assert_eq!(feature_b.requirements, vec!["REQ-003"]);

    let no_req = tests.iter().find(|t| t.name == "uat_no_req").unwrap();
    assert!(no_req.requirements.is_empty());
}

/// Test latency threshold metadata.
#[test]
fn uat_latency_threshold() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.rhai");

    let script = r#"
// @latency: 1000
fn uat_with_latency() { }

fn uat_no_latency() { }
"#;
    fs::write(&test_file, script).unwrap();

    let runner = UatRunner::with_test_dir(temp_dir.path());
    let tests = runner.discover();

    let with_latency = tests.iter().find(|t| t.name == "uat_with_latency").unwrap();
    assert_eq!(with_latency.latency_threshold, Some(1000));

    let no_latency = tests.iter().find(|t| t.name == "uat_no_latency").unwrap();
    assert_eq!(no_latency.latency_threshold, None);
}

/// Test multiple test files across directories.
#[test]
fn uat_multiple_files_and_directories() {
    let temp_dir = TempDir::new().unwrap();

    // Create multiple files in different directories
    let core_dir = temp_dir.path().join("core");
    fs::create_dir(&core_dir).unwrap();
    fs::write(core_dir.join("basic.rhai"), "fn uat_core_basic() { }").unwrap();
    fs::write(core_dir.join("advanced.rhai"), "fn uat_core_advanced() { }").unwrap();

    let layers_dir = temp_dir.path().join("layers");
    fs::create_dir(&layers_dir).unwrap();
    fs::write(layers_dir.join("switch.rhai"), "fn uat_layer_switch() { }").unwrap();

    let runner = UatRunner::with_test_dir(temp_dir.path());
    let tests = runner.discover();

    assert_eq!(tests.len(), 3, "Should find all 3 tests across directories");

    // Verify all tests are found
    let names: Vec<_> = tests.iter().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"uat_core_basic"));
    assert!(names.contains(&"uat_core_advanced"));
    assert!(names.contains(&"uat_layer_switch"));
}
