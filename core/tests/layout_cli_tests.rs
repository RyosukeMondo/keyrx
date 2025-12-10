#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::print_stdout, clippy::print_stderr, clippy::field_reassign_with_default, clippy::useless_conversion, clippy::assertions_on_constants, clippy::manual_div_ceil, clippy::manual_strip, clippy::len_zero, clippy::redundant_closure, clippy::manual_range_contains, clippy::default_constructed_unit_structs, clippy::clone_on_copy, clippy::io_other_error, clippy::bool_assert_comparison, clippy::approx_constant, clippy::let_unit_value, clippy::while_let_on_iterator, clippy::await_holding_lock, clippy::unnecessary_cast, clippy::drop_non_drop, clippy::needless_range_loop, unused_imports, unused_variables, dead_code, unsafe_code, clippy::collapsible_if, clippy::bool_comparison, unexpected_cfgs)]
//! Tests for layout CLI command.

use keyrx_core::cli::commands::{LayoutAction, LayoutCommand, LayoutSource};
use keyrx_core::cli::{Command, CommandContext, OutputFormat, Verbosity};
use keyrx_core::config::models::{KeyPosition, KeySize, LayoutType, VirtualKeyDef, VirtualLayout};
use serde_json::json;
use std::fs;
use std::io::Write;
use tempfile::{tempdir, NamedTempFile};

fn ctx(format: OutputFormat) -> CommandContext {
    CommandContext::new(format, Verbosity::Normal)
}

fn sample_layout() -> VirtualLayout {
    VirtualLayout {
        id: "layout-1".into(),
        name: "Test Layout".into(),
        layout_type: LayoutType::Matrix,
        keys: vec![VirtualKeyDef {
            id: "K1".into(),
            label: "Key 1".into(),
            position: Some(KeyPosition { x: 0.0, y: 0.0 }),
            size: Some(KeySize {
                width: 1.0,
                height: 1.0,
            }),
        }],
    }
}

#[test]
fn layout_list_handles_empty_directory() {
    let temp = tempdir().unwrap();
    let mut cmd =
        LayoutCommand::new(OutputFormat::Json, LayoutAction::List).with_config_root(temp.path());

    let result = cmd.execute(&ctx(OutputFormat::Json));
    assert!(result.is_success(), "listing empty layouts should succeed");
}

#[test]
fn layout_create_and_show_round_trips_layout() {
    let temp = tempdir().unwrap();
    let layout = sample_layout();

    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "{}", json!(layout)).unwrap();

    let mut create_cmd = LayoutCommand::new(
        OutputFormat::Human,
        LayoutAction::Create {
            source: LayoutSource::File(file.path().to_path_buf()),
        },
    )
    .with_config_root(temp.path());
    let create_result = create_cmd.execute(&ctx(OutputFormat::Human));
    assert!(create_result.is_success(), "creating layout should succeed");

    let saved_path = temp.path().join("layouts").join("layout-1.json");
    assert!(saved_path.exists(), "layout file should be written");

    let mut show_cmd = LayoutCommand::new(
        OutputFormat::Json,
        LayoutAction::Show {
            id: layout.id.clone(),
        },
    )
    .with_config_root(temp.path());
    let show_result = show_cmd.execute(&ctx(OutputFormat::Json));
    assert!(
        show_result.is_success(),
        "showing saved layout should succeed"
    );

    let content = fs::read_to_string(saved_path).expect("saved layout readable");
    assert!(
        content.contains(&layout.name),
        "saved layout JSON should include layout name"
    );
}
