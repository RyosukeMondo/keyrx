#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::field_reassign_with_default,
    clippy::useless_conversion,
    clippy::assertions_on_constants,
    clippy::manual_div_ceil,
    clippy::manual_strip,
    clippy::len_zero,
    clippy::redundant_closure,
    clippy::manual_range_contains,
    clippy::default_constructed_unit_structs,
    clippy::clone_on_copy,
    clippy::io_other_error,
    clippy::bool_assert_comparison,
    clippy::approx_constant,
    clippy::let_unit_value,
    clippy::while_let_on_iterator,
    clippy::await_holding_lock,
    clippy::unnecessary_cast,
    clippy::drop_non_drop,
    clippy::needless_range_loop,
    unused_imports,
    unused_variables,
    dead_code,
    unsafe_code,
    clippy::collapsible_if,
    clippy::bool_comparison,
    unexpected_cfgs
)]
//! Tests for keymap CLI command.

use keyrx_core::cli::commands::{KeymapAction, KeymapCommand, MapRequest};
use keyrx_core::cli::{Command, CommandContext, OutputFormat, Verbosity};
use keyrx_core::config::models::{ActionBinding, Keymap};
use keyrx_core::config::ConfigManager;
use tempfile::tempdir;

fn ctx(format: OutputFormat) -> CommandContext {
    CommandContext::new(format, Verbosity::Normal)
}

fn sample_keymap() -> Keymap {
    Keymap {
        id: "keymap-1".into(),
        name: "Test Keymap".into(),
        virtual_layout_id: "layout-1".into(),
        layers: vec![],
    }
}

#[test]
fn keymap_list_handles_empty_directory() {
    let temp = tempdir().unwrap();
    let mut cmd =
        KeymapCommand::new(OutputFormat::Json, KeymapAction::List).with_config_root(temp.path());

    let result = cmd.execute(&ctx(OutputFormat::Json));
    assert!(result.is_success(), "listing empty keymaps should succeed");
}

#[test]
fn keymap_map_sets_binding_and_show_reads_back() {
    let temp = tempdir().unwrap();
    let manager = ConfigManager::new(temp.path());
    let keymap = sample_keymap();
    manager.save_keymap(&keymap).expect("seed keymap");

    let mut map_cmd = KeymapCommand::new(
        OutputFormat::Human,
        KeymapAction::Map {
            request: MapRequest {
                keymap_id: keymap.id.clone(),
                layer: "base".into(),
                virtual_key: "K1".into(),
                action: Some("key:KEY_A".into()),
                clear: false,
            },
        },
    )
    .with_config_root(temp.path());
    let map_result = map_cmd.execute(&ctx(OutputFormat::Human));
    assert!(map_result.is_success(), "mapping key should succeed");

    let mut stored_maps = manager.load_keymaps().expect("load persisted keymap");
    let stored = stored_maps
        .remove(&keymap.id)
        .expect("stored keymap exists");
    let layer = stored
        .layers
        .iter()
        .find(|layer| layer.name == "base")
        .expect("layer created");
    assert_eq!(
        layer.bindings.get("K1"),
        Some(&ActionBinding::StandardKey("KEY_A".into()))
    );

    let mut show_cmd = KeymapCommand::new(
        OutputFormat::Json,
        KeymapAction::Show {
            id: keymap.id.clone(),
        },
    )
    .with_config_root(temp.path());
    let show_result = show_cmd.execute(&ctx(OutputFormat::Json));
    assert!(
        show_result.is_success(),
        "showing saved keymap should succeed"
    );
}
