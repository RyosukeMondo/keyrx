//! Error quality and the accessibility functions (`one_shot`,
//! `tap_hold_timeout_only`), plus cross-cutting parser rules.

use super::*;

fn parse(script: &str) -> Result<keyrx_core::config::ConfigRoot, String> {
    Parser::new()
        .parse_string(script, &PathBuf::from("test.rhai"))
        .map_err(|e| format!("{e:?}"))
}

fn error_of(script: &str) -> String {
    parse(script).expect_err("script should not compile")
}

#[test]
fn duplicate_device_pattern_names_both_lines() {
    let err = error_of(
        "device_start(\"Kbd*\");\nmap(\"A\", \"VK_B\");\ndevice_end();\n\
         device_start(\"KBD*\");\nmap(\"C\", \"VK_D\");\ndevice_end();\n",
    );
    assert!(err.contains("Duplicate device_start"), "{err}");
    assert!(err.contains("line 1") && err.contains("line 4"), "{err}");
}

#[test]
fn distinct_device_patterns_are_fine() {
    let config = parse(
        "device_start(\"a\");\nmap(\"A\", \"VK_B\");\ndevice_end();\n\
         device_start(\"b\");\nmap(\"A\", \"VK_C\");\ndevice_end();\n",
    )
    .unwrap();
    assert_eq!(config.devices.len(), 2);
}

#[test]
fn unclosed_device_is_reported_at_its_opening_line() {
    let err = error_of("// c\ndevice_start(\"Kbd\");\nmap(\"A\", \"VK_B\");\n");
    assert!(err.contains("line: 2"), "{err}");
    assert!(
        err.contains("never closed") && err.contains("device_end"),
        "{err}"
    );
}

#[test]
fn tap_hold_with_three_arguments_says_what_it_needs() {
    let err =
        error_of("device_start(\"K\");\ntap_hold(\"A\", \"VK_B\", \"MD_00\");\ndevice_end();");
    assert!(err.contains("tap_hold needs 4 argument(s), got 3"), "{err}");
    assert!(
        err.contains("tap_hold(key, tap, hold, threshold_ms)"),
        "{err}"
    );
    assert!(!err.contains("ImmutableString"), "{err}");
}

#[test]
fn zero_and_out_of_range_thresholds_are_rejected() {
    for ms in ["0", "-5", "70000"] {
        let err = error_of(&format!(
            "device_start(\"K\");\ntap_hold(\"A\", \"VK_B\", \"VK_LCtrl\", {ms});\ndevice_end();"
        ));
        assert!(
            err.contains("threshold_ms must be between 1 and 65535"),
            "{ms}: {err}"
        );
    }
    let err = error_of("device_start(\"K\");\nhold_only(\"A\", \"MD_00\", 0);\ndevice_end();");
    assert!(err.contains("threshold_ms must be between"), "{err}");
}

#[test]
fn unrelated_unknown_key_gets_no_suggestions() {
    let err = error_of("device_start(\"K\");\nmap(\"Nope\", \"VK_B\");\ndevice_end();");
    assert!(err.contains("Unknown key name"), "{err}");
    assert!(!err.contains("Did you mean"), "{err}");
}

#[test]
fn syntax_errors_do_not_repeat_the_label() {
    let err = error_of("device_start(\"K\");\nmap(\"A\", \"VK_B\")\ndevice_end();");
    assert!(!err.contains("Syntax error: Syntax error"), "{err}");
    assert!(!err.contains("(line "), "{err}");
}

#[test]
fn one_shot_builds_a_mapping_with_and_without_timeout() {
    let config = parse(
        "device_start(\"K\");\none_shot(\"CapsLock\", \"VK_LShift\");\n\
         one_shot(\"Tab\", \"VK_LCtrl\", 3000);\ndevice_end();",
    )
    .unwrap();
    let kinds: Vec<_> = config.devices[0].mappings.iter().collect();
    assert_eq!(
        kinds[0],
        &KeyMapping::Base(BaseKeyMapping::OneShot {
            from: KeyCode::CapsLock,
            modifier: KeyCode::LShift,
            timeout_ms: 0
        })
    );
    assert_eq!(
        kinds[1],
        &KeyMapping::Base(BaseKeyMapping::OneShot {
            from: KeyCode::Tab,
            modifier: KeyCode::LCtrl,
            timeout_ms: 3000
        })
    );
}

#[test]
fn one_shot_rejects_non_modifiers() {
    for modifier in ["VK_A", "MD_00", "A"] {
        let err = error_of(&format!(
            "device_start(\"K\");\none_shot(\"CapsLock\", \"{modifier}\");\ndevice_end();"
        ));
        assert!(err.contains("one_shot modifier"), "{modifier}: {err}");
    }
}

#[test]
fn tap_hold_timeout_only_builds_its_own_mapping() {
    let config = parse(
        "device_start(\"K\");\ntap_hold_timeout_only(\"F\", \"VK_F\", \"VK_LCtrl\", 250);\ndevice_end();",
    )
    .unwrap();
    assert_eq!(
        config.devices[0].mappings[0],
        KeyMapping::Base(BaseKeyMapping::TapHoldKeyTimeoutOnly {
            from: KeyCode::F,
            tap: Some(KeyCode::F),
            hold: KeyCode::LCtrl,
            threshold_ms: 250
        })
    );
    let err = error_of(
        "device_start(\"K\");\ntap_hold_timeout_only(\"F\", \"VK_F\", \"MD_00\", 250);\ndevice_end();",
    );
    assert!(err.contains("real key"), "{err}");
}

#[test]
fn imported_files_can_use_every_dsl_function() {
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("lib.rhai"),
        "hold_only(\"Tab\", \"VK_LCtrl\");\nsequence(\"F1\", [\"VK_A\", \"VK_B\"]);\n\
         one_shot(\"CapsLock\", \"VK_LShift\");\n",
    )
    .unwrap();
    let main = dir.path().join("main.rhai");
    std::fs::write(
        &main,
        "device_start(\"K\");\nload(\"lib.rhai\");\ndevice_end();\n",
    )
    .unwrap();
    let config = Parser::new()
        .parse_script(&main)
        .expect("import should compile");
    assert_eq!(config.devices[0].mappings.len(), 3);
}
