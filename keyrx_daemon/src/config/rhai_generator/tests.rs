use super::*;

const PROFILE: &str = concat!(
    "// my profile\n",
    "device_start(\"*\");\n",
    "\n",
    "    // caps is escape\n",
    "    map(  \"VK_CapsLock\",\"VK_Escape\"  );\n",
    "\tmap(\"A\", \"VK_B\");\n",
    "\n",
    "when_start(\"MD_00\");\n",
    "  map(\"H\", \"VK_Left\");\n",
    "when_end();\n",
    "\n",
    "device_end();\n",
    "// the end\n"
);

fn remap(output: &str) -> KeyAction {
    KeyAction::SimpleRemap {
        output: output.to_string(),
    }
}

fn line_diff(before: &str, after: &str) -> (Vec<String>, Vec<String>) {
    let b: Vec<&str> = before.lines().collect();
    let a: Vec<&str> = after.lines().collect();
    let removed = b
        .iter()
        .filter(|l| !a.contains(l))
        .map(|l| l.to_string())
        .collect();
    let added = a
        .iter()
        .filter(|l| !b.contains(l))
        .map(|l| l.to_string())
        .collect();
    (removed, added)
}

#[test]
fn an_unmodified_profile_round_trips_byte_for_byte() {
    for text in [
        PROFILE,
        &PROFILE.replace('\n', "\r\n"),
        &PROFILE[..PROFILE.len() - 1],
    ] {
        assert_eq!(RhaiGenerator::parse(text).unwrap().to_string(), text);
    }
}

#[test]
fn setting_a_key_spelled_differently_replaces_it_instead_of_duplicating() {
    let mut gen = RhaiGenerator::parse(PROFILE).unwrap();
    // The file says VK_CapsLock; the user types the manual's bare spelling.
    gen.set_key_mapping("base", "CapsLock", remap("VK_LCtrl"))
        .unwrap();
    let out = gen.to_string();
    assert_eq!(out.matches("CapsLock").count(), 1, "no duplicate:\n{out}");
    let (removed, added) = line_diff(PROFILE, &out);
    assert_eq!(removed, vec!["    map(  \"VK_CapsLock\",\"VK_Escape\"  );"]);
    assert_eq!(
        added,
        vec!["    map(\"CapsLock\", \"VK_LCtrl\");"],
        "indent kept"
    );
}

#[test]
fn an_edit_changes_only_its_own_line_and_leaves_odd_formatting_alone() {
    let mut gen = RhaiGenerator::parse(PROFILE).unwrap();
    gen.set_key_mapping("base", "VK_A", remap("VK_C")).unwrap();
    let (removed, added) = line_diff(PROFILE, &gen.to_string());
    assert_eq!(removed, vec!["\tmap(\"A\", \"VK_B\");"]);
    assert_eq!(added, vec!["\tmap(\"A\", \"VK_C\");"], "tab indent kept");
    assert!(gen
        .to_string()
        .contains("map(  \"VK_CapsLock\",\"VK_Escape\"  );"));
}

#[test]
fn a_new_base_mapping_goes_after_the_last_base_mapping_not_at_the_end() {
    let mut gen = RhaiGenerator::parse(PROFILE).unwrap();
    gen.set_key_mapping("base", "F13", remap("VK_F14")).unwrap();
    let out = gen.to_string();
    let lines: Vec<&str> = out.lines().collect();
    let new = lines.iter().position(|l| l.contains("F13")).unwrap();
    assert!(
        lines[new - 1].contains("map(\"A\""),
        "after the last base mapping"
    );
    assert_eq!(
        lines[new + 1],
        "",
        "blank line before the layer is untouched"
    );
    assert_eq!(lines[new], "\tmap(\"F13\", \"VK_F14\");");
}

#[test]
fn layer_mappings_are_scoped_to_their_layer() {
    let mut gen = RhaiGenerator::parse(PROFILE).unwrap();
    gen.set_key_mapping("MD_00", "H", remap("VK_Home")).unwrap();
    gen.set_key_mapping("MD_00", "J", remap("VK_Down")).unwrap();
    gen.set_key_mapping("base", "H", remap("VK_X")).unwrap();
    let out = gen.to_string();
    assert!(out.contains("when_start(\"MD_00\");\n  map(\"H\", \"VK_Home\");\n  map(\"J\", \"VK_Down\");\nwhen_end();"));
    assert!(
        out.contains("map(\"H\", \"VK_X\")"),
        "base H is separate from layer H"
    );
    assert_eq!(
        gen.get_layer_mappings("MD_00").unwrap(),
        vec!["map(\"H\", \"VK_Home\");", "map(\"J\", \"VK_Down\");"]
    );
    assert!(gen.set_key_mapping("MD_99", "A", remap("VK_B")).is_err());
}

#[test]
fn input_keys_are_bare_names_and_vk_prefix_is_tolerated() {
    let mut gen = RhaiGenerator::parse(PROFILE).unwrap();
    gen.set_key_mapping("base", "F13", remap("F14")).unwrap();
    gen.set_key_mapping("base", "VK_F15", remap("VK_F16"))
        .unwrap();
    let out = gen.to_string();
    assert!(
        out.contains("map(\"F13\", \"VK_F14\")"),
        "output gets its VK_ prefix"
    );
    assert!(
        out.contains("map(\"F15\", \"VK_F16\")"),
        "input prefix is dropped"
    );
    assert!(matches!(
        gen.set_key_mapping("base", "NotAKey", remap("VK_A")),
        Err(GeneratorError::InvalidKeyName(_))
    ));
    assert!(gen.set_key_mapping("base", "A", remap("NotAKey")).is_err());
}

#[test]
fn a_failed_edit_leaves_the_text_unchanged() {
    let mut gen = RhaiGenerator::parse(PROFILE).unwrap();
    let _ = gen.set_key_mapping("base", "A", remap("NotAKey"));
    assert_eq!(gen.to_string(), PROFILE);
}

#[test]
fn delete_removes_every_spelling_of_the_key() {
    let mut gen = RhaiGenerator::parse(PROFILE).unwrap();
    gen.delete_key_mapping("base", "VK_CapsLock").unwrap();
    gen.delete_key_mapping("base", "CapsLock").unwrap();
    assert!(!gen.to_string().contains("CapsLock"));
    assert_eq!(gen.to_string().lines().count(), PROFILE.lines().count() - 1);
}

#[test]
fn find_mapping_matches_either_spelling() {
    let gen = RhaiGenerator::parse(PROFILE).unwrap();
    assert!(gen.find_mapping("base", "CapsLock").unwrap().is_some());
    assert!(gen.find_mapping("base", "Z").unwrap().is_none());
    assert!(gen.find_mapping("MD_00", "H").unwrap().is_some());
}

#[test]
fn tap_hold_and_macro_lines_are_generated() {
    let mut gen = RhaiGenerator::parse("device_start(\"*\");\ndevice_end();\n").unwrap();
    gen.set_key_mapping(
        "base",
        "Space",
        KeyAction::TapHold {
            tap: "Space".to_string(),
            hold: "MD_00".to_string(),
            threshold_ms: 200,
        },
    )
    .unwrap();
    gen.set_key_mapping(
        "base",
        "F1",
        KeyAction::Macro {
            sequence: vec![
                MacroStep::Press("A".into()),
                MacroStep::Wait(10),
                MacroStep::Release("A".into()),
            ],
        },
    )
    .unwrap();
    let out = gen.to_string();
    assert!(
        out.contains("  tap_hold(\"Space\", \"VK_Space\", \"MD_00\", 200);"),
        "{out}"
    );
    assert!(
        out.contains("  macro(\"F1\", [press(\"VK_A\"), wait(10), release(\"VK_A\")]);"),
        "{out}"
    );
}

#[test]
fn a_tap_hold_line_is_replaced_by_a_simple_remap_of_the_same_key() {
    let src = "device_start(\"*\");\n  tap_hold(\"VK_Space\", \"VK_Space\", \"MD_00\", 200);\ndevice_end();\n";
    let mut gen = RhaiGenerator::parse(src).unwrap();
    gen.set_key_mapping("base", "Space", remap("VK_Enter"))
        .unwrap();
    assert_eq!(
        gen.to_string(),
        "device_start(\"*\");\n  map(\"Space\", \"VK_Enter\");\ndevice_end();\n"
    );
}

#[test]
fn layers_can_be_added_renamed_and_deleted_without_touching_other_lines() {
    let mut gen = RhaiGenerator::parse(PROFILE).unwrap();
    gen.add_layer("MD_01", "Nav", LayerMode::Single).unwrap();
    assert_eq!(
        gen.list_layers(),
        vec![("MD_00".to_string(), 1), ("MD_01".to_string(), 0)]
    );
    assert!(matches!(
        gen.add_layer("MD_01", "x", LayerMode::Single),
        Err(GeneratorError::LayerExists(_))
    ));
    gen.rename_layer("MD_01", "MD_02").unwrap();
    gen.delete_layer("MD_02").unwrap();
    assert_eq!(gen.to_string(), PROFILE, "add + rename + delete is a no-op");
    gen.delete_layer("MD_00").unwrap();
    assert!(!gen.to_string().contains("MD_00"));
    assert!(gen.add_layer("Nav", "x", LayerMode::Single).is_err());
}

#[test]
fn only_the_first_device_block_is_edited() {
    let src = "device_start(\"a*\");\n  map(\"A\", \"VK_B\");\ndevice_end();\n\ndevice_start(\"b*\");\n  map(\"A\", \"VK_C\");\ndevice_end();\n";
    let mut gen = RhaiGenerator::parse(src).unwrap();
    gen.set_key_mapping("base", "A", remap("VK_D")).unwrap();
    assert_eq!(gen.to_string(), src.replacen("VK_B", "VK_D", 1));
}

#[test]
fn structure_errors_are_reported() {
    assert!(matches!(
        RhaiGenerator::parse("map(\"A\", \"VK_B\");"),
        Err(GeneratorError::DeviceNotFound)
    ));
    assert!(matches!(
        RhaiGenerator::parse("device_start(\"*\");\nwhen_start(\"MD_00\");\ndevice_end();"),
        Err(GeneratorError::UnclosedWhenBlock(_))
    ));
}

#[test]
fn key_identity_ignores_the_vk_prefix() {
    assert!(same_key("CapsLock", "VK_CapsLock"));
    assert!(!same_key("CapsLock", "VK_Escape"));
    assert!(same_key("MD_00", "MD_00"));
    assert!(!same_key("MD_00", "MD_01"));
}

#[test]
fn layer_ids_must_be_md_ids() {
    assert!(validate_layer_id("MD_00").is_ok());
    assert!(validate_layer_id("VK_A").is_err());
    assert!(validate_layer_id("Invalid").is_err());
}
