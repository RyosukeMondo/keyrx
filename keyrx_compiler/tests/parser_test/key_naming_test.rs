//! The key-naming rule: inputs are bare names (VK_ tolerated), outputs need
//! VK_, MD_/LK_ are outputs only.

use super::*;

fn parse(body: &str) -> Result<keyrx_core::config::ConfigRoot, String> {
    let script = format!("device_start(\"*\");\n{body}\ndevice_end();\n");
    Parser::new()
        .parse_string(&script, &PathBuf::from("test.rhai"))
        .map_err(|e| e.to_string())
}

#[test]
fn bare_and_prefixed_inputs_mean_the_same_key() {
    let bare = parse(r#"map("CapsLock", "VK_Escape");"#).expect("bare input");
    let prefixed = parse(r#"map("VK_CapsLock", "VK_Escape");"#).expect("prefixed input");
    assert_eq!(bare.devices[0].mappings, prefixed.devices[0].mappings);
}

#[test]
fn output_without_vk_prefix_is_rejected_with_a_hint() {
    let err = parse(r#"map("A", "B");"#).expect_err("bare output");
    assert!(err.contains("VK_B"), "{err}");
}

#[test]
fn custom_modifier_as_input_is_rejected_with_a_hint() {
    for input in ["MD_00", "LK_01"] {
        let err = parse(&format!(r#"map("{input}", "VK_A");"#)).expect_err("MD/LK input");
        assert!(
            err.contains("can only be an output") && err.contains(input),
            "{err}"
        );
    }
}

#[test]
fn every_mapping_function_applies_the_same_input_rule() {
    for call in [
        r#"tap_hold("MD_00", "VK_A", "MD_01", 200);"#,
        r#"hold_only("MD_00", "MD_01");"#,
        r#"sequence("MD_00", ["VK_A"]);"#,
        r#"map("MD_00", with_shift("VK_A"));"#,
    ] {
        let err = parse(call).expect_err(call);
        assert!(err.contains("can only be an output"), "{call}: {err}");
    }
}

#[test]
fn sequence_produces_the_keys_in_order() {
    let config = parse(r#"sequence("F21", ["VK_H", "VK_I"]);"#).expect("sequence");
    assert_eq!(
        config.devices[0].mappings,
        vec![KeyMapping::Base(BaseKeyMapping::Sequence {
            from: KeyCode::F21,
            keys: vec![KeyCode::H, KeyCode::I],
        })]
    );
}

#[test]
fn duplicate_across_mapping_kinds_is_an_error() {
    // A sequence after a map on the same trigger key was silently dead.
    let err = parse("map(\"F21\", \"VK_F22\");\nsequence(\"F21\", [\"VK_A\", \"VK_B\"]);")
        .expect_err("duplicate trigger key");
    assert!(err.contains("Duplicate mapping for key F21"), "{err}");
    assert!(err.contains("line 2") && err.contains("line 3"), "{err}");
}
