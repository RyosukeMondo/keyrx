//! CLI behaviour a first-time user hits: profile discovery, honest exit
//! codes, help text that matches reality, paths that honour
//! `KEYRX_CONFIG_DIR`, and the emergency stop being visible.
//!
//! Every command runs against a scratch config dir and a scratch
//! `XDG_RUNTIME_DIR`, so it can never reach a real daemon's IPC socket.

use assert_cmd::Command;
use keyrx_daemon::config::{ProfileManager, ProfileTemplate};
use predicates::prelude::*;
use serde_json::Value;
use tempfile::TempDir;

struct Sandbox {
    config: TempDir,
    runtime: TempDir,
}

impl Sandbox {
    fn new() -> Self {
        Self {
            config: TempDir::new().unwrap(),
            runtime: TempDir::new().unwrap(),
        }
    }

    #[allow(deprecated)]
    fn cmd(&self) -> Command {
        let mut cmd = Command::cargo_bin("keyrx_daemon").unwrap();
        cmd.env("KEYRX_CONFIG_DIR", self.config.path())
            .env("XDG_RUNTIME_DIR", self.runtime.path());
        cmd
    }
}

// --- profile discovery: the directory is the single source of truth -------

#[test]
fn a_profile_created_by_another_process_is_visible_and_activatable() {
    let dir = TempDir::new().unwrap();
    // "The daemon": a manager built before the profile exists.
    let daemon_side = ProfileManager::new(dir.path().to_path_buf()).unwrap();
    assert!(daemon_side.list().is_empty());

    // "The CLI": another manager creates a profile on disk.
    ProfileManager::new(dir.path().to_path_buf())
        .unwrap()
        .create("late", ProfileTemplate::CapslockEscape)
        .unwrap();

    assert!(
        daemon_side.list().iter().any(|p| p.name == "late"),
        "list must see profiles created after startup"
    );
    assert!(daemon_side.get("late").is_some());
    let result = daemon_side.activate("late").expect("activate must find it");
    assert!(result.success, "{:?}", result.error);
}

#[test]
fn a_profile_deleted_by_another_process_disappears() {
    let dir = TempDir::new().unwrap();
    let a = ProfileManager::new(dir.path().to_path_buf()).unwrap();
    a.create("gone", ProfileTemplate::Blank).unwrap();
    std::fs::remove_file(dir.path().join("profiles/gone.rhai")).unwrap();
    assert!(a.get("gone").is_none());
    assert!(a.list().is_empty());
}

// --- templates ------------------------------------------------------------

#[test]
fn every_template_compiles() {
    let dir = TempDir::new().unwrap();
    for template in ProfileTemplate::ALL {
        let src = dir.path().join(format!("{}.rhai", template.name()));
        std::fs::write(&src, template.source()).unwrap();
        keyrx_compiler::compile_file(&src, &dir.path().join("out.krx"))
            .unwrap_or_else(|e| panic!("template '{}' does not compile: {e}", template.name()));
    }
}

#[test]
fn template_names_round_trip_and_reject_unknown() {
    for template in ProfileTemplate::ALL {
        assert_eq!(
            ProfileTemplate::from_name(template.name()).unwrap().name(),
            template.name()
        );
    }
    assert!(ProfileTemplate::from_name("Vim-Navigation").is_ok());
    let err = ProfileTemplate::from_name("qmk-layers").unwrap_err();
    assert!(
        err.contains("qmk-layers") && err.contains("vim_navigation"),
        "{err}"
    );
}

#[test]
fn profiles_create_help_lists_the_real_templates() {
    let sandbox = Sandbox::new();
    sandbox
        .cmd()
        .args(["profiles", "create", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("vim_navigation"))
        .stdout(predicate::str::contains("qmk").not());
}

#[test]
fn profiles_create_prints_the_real_path_not_a_hardcoded_home() {
    let sandbox = Sandbox::new();
    let expected = sandbox.config.path().join("profiles/mine.rhai");
    sandbox
        .cmd()
        .args(["profiles", "create", "mine"])
        .assert()
        .success()
        .stdout(predicate::str::contains(expected.display().to_string()))
        .stdout(predicate::str::contains("~/.config/keyrx").not());

    let json = sandbox
        .cmd()
        .args(["profiles", "create", "mine2", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&json).unwrap();
    assert!(json["rhai_path"]
        .as_str()
        .unwrap()
        .starts_with(&sandbox.config.path().display().to_string()));
}

// --- honest activation ----------------------------------------------------

#[test]
fn activating_a_missing_profile_fails_once_and_exits_non_zero() {
    let sandbox = Sandbox::new();
    let output = sandbox
        .cmd()
        .args(["profiles", "activate", "nope"])
        .assert()
        .failure()
        .get_output()
        .clone();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        stderr.matches("not found").count(),
        1,
        "the failure must be printed exactly once:\n{stderr}"
    );
}

#[test]
fn a_profile_that_does_not_compile_reports_its_error_once() {
    let sandbox = Sandbox::new();
    sandbox
        .cmd()
        .args(["profiles", "create", "broken"])
        .assert()
        .success();
    std::fs::write(
        sandbox.config.path().join("profiles/broken.rhai"),
        "device_start(\"*\"\n  map(",
    )
    .unwrap();
    let output = sandbox
        .cmd()
        .args(["profiles", "activate", "broken"])
        .assert()
        .failure()
        .get_output()
        .clone();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("broken.rhai"), "names the file:\n{stderr}");
    assert_eq!(
        stderr.matches("Activation failed").count()
            + stderr.matches("Command 'activate' failed").count(),
        1,
        "one failure report, not two:\n{stderr}"
    );
}

#[test]
fn activating_without_a_daemon_succeeds_and_says_it_applies_on_next_start() {
    let sandbox = Sandbox::new();
    sandbox
        .cmd()
        .args(["profiles", "create", "ok", "--template", "capslock_escape"])
        .assert()
        .success();
    sandbox
        .cmd()
        .args(["profiles", "activate", "ok"])
        .assert()
        .success()
        .stdout(predicate::str::contains("applies on next start"));
}

// --- validate -------------------------------------------------------------

#[cfg(target_os = "linux")]
#[test]
fn validate_accepts_a_rhai_source_instead_of_reporting_bad_magic_bytes() {
    let sandbox = Sandbox::new();
    let source = sandbox.config.path().join("mine.rhai");
    std::fs::write(&source, ProfileTemplate::CapslockEscape.source()).unwrap();
    sandbox
        .cmd()
        .args(["validate", "--config"])
        .arg(&source)
        .assert()
        .stderr(predicate::str::contains("magic").not())
        .stdout(predicate::str::contains("compiling"));
}

#[cfg(target_os = "linux")]
#[test]
fn validate_reports_a_rhai_compile_error_with_its_location() {
    let sandbox = Sandbox::new();
    let source = sandbox.config.path().join("bad.rhai");
    std::fs::write(&source, "device_start(\"*\"\n  map(").unwrap();
    sandbox
        .cmd()
        .args(["validate", "--config"])
        .arg(&source)
        .assert()
        .failure()
        .stderr(predicate::str::contains("bad.rhai"))
        .stderr(predicate::str::contains("magic").not());
}

// --- devices --------------------------------------------------------------

#[test]
fn devices_list_says_when_no_daemon_is_running_and_never_mentions_a_nonexistent_command() {
    let sandbox = Sandbox::new();
    sandbox
        .cmd()
        .args(["devices", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("daemon is not running"))
        .stdout(predicate::str::contains("keyrx daemon run").not());

    let json = sandbox
        .cmd()
        .args(["devices", "list", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: Value = serde_json::from_slice(&json).unwrap();
    assert_eq!(json["daemon_running"], false);
    assert!(json["captured"].as_array().unwrap().is_empty());
}

// --- emergency stop and logging flags -------------------------------------

#[cfg(target_os = "linux")]
#[test]
fn the_emergency_chord_is_in_run_help_and_doctor() {
    let chord = "Left Ctrl + Right Ctrl + Escape";
    let sandbox = Sandbox::new();
    let help = sandbox.cmd().args(["run", "--help"]).output().unwrap();
    // clap re-wraps long help; compare without whitespace differences.
    let flat = |bytes: &[u8]| {
        String::from_utf8_lossy(bytes)
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    };
    assert!(
        flat(&help.stdout).contains(chord),
        "run --help: {}",
        flat(&help.stdout)
    );

    let doctor = sandbox.cmd().arg("doctor").output().unwrap();
    assert!(
        flat(&doctor.stdout).contains(chord),
        "doctor: {}",
        flat(&doctor.stdout)
    );
    // The one-handed alternative is announced next to the chord.
    assert!(flat(&help.stdout).contains("hold Escape alone"));
    assert!(flat(&doctor.stdout).contains("hold Escape alone for 3 s"));
}

#[test]
fn run_help_documents_the_key_logging_and_watch_flags() {
    let sandbox = Sandbox::new();
    sandbox
        .cmd()
        .args(["run", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--log-keys"))
        .stdout(predicate::str::contains("--no-watch"));
}
