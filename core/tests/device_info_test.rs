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
//! DeviceInfo serialization and formatting tests.

use keyrx_core::drivers::DeviceInfo;
use std::path::PathBuf;

#[test]
fn device_info_creation() {
    let info = DeviceInfo::new(
        PathBuf::from("/dev/input/event0"),
        "Test Keyboard".to_string(),
        0x1234,
        0x5678,
        true,
    );

    assert_eq!(info.name(), "Test Keyboard");
    assert_eq!(info.path(), &PathBuf::from("/dev/input/event0"));
    assert_eq!(info.vendor_id(), 0x1234);
    assert_eq!(info.product_id(), 0x5678);
    assert!(info.is_keyboard());
}

#[test]
fn device_info_display_format() {
    let info = DeviceInfo::new(
        PathBuf::from("/dev/input/event5"),
        "My Keyboard".to_string(),
        0xABCD,
        0x1234,
        true,
    );

    let display = format!("{}", info);
    assert!(display.contains("My Keyboard"));
    assert!(display.contains("abcd:1234")); // hex format, lowercase
    assert!(display.contains("/dev/input/event5"));
}

#[test]
fn device_info_json_serialization() {
    let info = DeviceInfo::new(
        PathBuf::from("/dev/input/event0"),
        "USB Keyboard".to_string(),
        0x046D,
        0xC52B,
        true,
    );

    let json = serde_json::to_string(&info).expect("JSON serialization failed");
    assert!(json.contains("\"name\":\"USB Keyboard\""));
    assert!(json.contains("\"is_keyboard\":true"));
}
