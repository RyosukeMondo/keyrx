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
//! Unit tests for scripting::row_col_resolver module.

use chrono::Utc;
use keyrx_core::discovery::types::{DeviceProfile, PhysicalKey, ProfileSource};
use keyrx_core::scripting::{ResolverError, RowColResolver};
use std::collections::HashMap;
use std::sync::Arc;

fn create_test_profile() -> DeviceProfile {
    let mut keymap = HashMap::new();

    // Add some test keys (matching typical QWERTY layout)
    // scan_code 1 = Escape at r0_c0
    keymap.insert(1, PhysicalKey::new(1, 0, 0));

    // scan_code 30 = A at r3_c1 (home row, 2nd position)
    keymap.insert(30, PhysicalKey::new(30, 3, 1));

    // scan_code 31 = S at r3_c2
    keymap.insert(31, PhysicalKey::new(31, 3, 2));

    // scan_code 58 = CapsLock at r3_c0
    keymap.insert(58, PhysicalKey::new(58, 3, 0));

    DeviceProfile {
        schema_version: 1,
        vendor_id: 0x1234,
        product_id: 0x5678,
        name: Some("Test Keyboard".to_string()),
        discovered_at: Utc::now(),
        rows: 6,
        cols_per_row: vec![17, 15, 14, 13, 14, 12],
        keymap,
        aliases: HashMap::new(),
        source: ProfileSource::Discovered,
    }
}

#[test]
fn resolver_without_profile_fails() {
    let resolver = RowColResolver::without_profile();
    let result = resolver.resolve(3, 1);

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ResolverError::NoProfileLoaded
    ));
}

#[test]
fn resolver_finds_valid_position() {
    let profile = create_test_profile();
    let resolver = RowColResolver::new(Some(Arc::new(profile)));

    // r3_c1 should resolve to scan_code 30 → KeyCode::A
    let result = resolver.resolve(3, 1);
    assert!(result.is_ok());

    #[cfg(target_os = "linux")]
    {
        use keyrx_core::engine::KeyCode;
        assert_eq!(result.unwrap(), KeyCode::A);
    }
}

#[test]
fn resolver_fails_on_invalid_position() {
    let profile = create_test_profile();
    let resolver = RowColResolver::new(Some(Arc::new(profile)));

    // r99_c99 doesn't exist in our test profile
    let result = resolver.resolve(99, 99);
    assert!(result.is_err());

    match result.unwrap_err() {
        ResolverError::PositionNotFound { row, col, .. } => {
            assert_eq!(row, 99);
            assert_eq!(col, 99);
        }
        _ => panic!("Expected PositionNotFound error"),
    }
}

#[test]
fn resolver_handles_multiple_positions() {
    let profile = create_test_profile();
    let resolver = RowColResolver::new(Some(Arc::new(profile)));

    // Test multiple valid positions
    assert!(resolver.resolve(0, 0).is_ok()); // Escape
    assert!(resolver.resolve(3, 0).is_ok()); // CapsLock
    assert!(resolver.resolve(3, 1).is_ok()); // A
    assert!(resolver.resolve(3, 2).is_ok()); // S
}

#[test]
fn resolver_has_profile_check() {
    let profile = create_test_profile();
    let resolver_with = RowColResolver::new(Some(Arc::new(profile)));
    let resolver_without = RowColResolver::without_profile();

    assert!(resolver_with.has_profile());
    assert!(!resolver_without.has_profile());
}

#[test]
fn resolver_device_name() {
    let profile = create_test_profile();
    let resolver = RowColResolver::new(Some(Arc::new(profile)));

    assert_eq!(resolver.device_name(), Some("Test Keyboard".to_string()));
}
