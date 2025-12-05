//! Unit tests for scripting::builtins module.

use keyrx_core::scripting::{
    normalize_layer_name, normalize_modifier_name, validate_timeout, ModifierPreview,
};

#[test]
fn modifier_preview_define_and_lookup() {
    let mut preview = ModifierPreview::new();
    let id = preview.define("hyper").unwrap();
    assert_eq!(id, 0);

    let same_id = preview.define("hyper").unwrap();
    assert_eq!(same_id, 0);

    let id2 = preview.define("meh").unwrap();
    assert_eq!(id2, 1);
}

#[test]
fn modifier_preview_activate_deactivate() {
    let mut preview = ModifierPreview::new();
    let id = preview.define("hyper").unwrap();

    assert!(!preview.is_active(id));
    preview.activate(id);
    assert!(preview.is_active(id));
    preview.deactivate(id);
    assert!(!preview.is_active(id));
}

#[test]
fn validate_timeout_ranges() {
    assert!(validate_timeout(0, "test", false).is_err());
    assert!(validate_timeout(1, "test", false).is_ok());
    assert!(validate_timeout(0, "test", true).is_ok());
    assert!(validate_timeout(5000, "test", false).is_ok());
    assert!(validate_timeout(5001, "test", false).is_err());
}

#[test]
fn normalize_layer_name_validation() {
    assert!(normalize_layer_name("", "test").is_err());
    assert!(normalize_layer_name("  ", "test").is_err());
    assert!(normalize_layer_name("layer:name", "test").is_err());
    assert_eq!(normalize_layer_name("nav", "test").unwrap(), "nav");
    assert_eq!(normalize_layer_name("  nav  ", "test").unwrap(), "nav");
}

#[test]
fn normalize_modifier_name_validation() {
    assert!(normalize_modifier_name("", "test").is_err());
    assert!(normalize_modifier_name("  ", "test").is_err());
    assert!(normalize_modifier_name("mod:name", "test").is_err());
    assert_eq!(normalize_modifier_name("hyper", "test").unwrap(), "hyper");
}
