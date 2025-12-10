#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::print_stdout, clippy::print_stderr, clippy::field_reassign_with_default, clippy::useless_conversion, clippy::assertions_on_constants, clippy::manual_div_ceil, clippy::manual_strip, clippy::len_zero, clippy::redundant_closure, clippy::manual_range_contains, clippy::default_constructed_unit_structs, clippy::clone_on_copy, clippy::io_other_error, clippy::bool_assert_comparison, clippy::approx_constant, clippy::let_unit_value, clippy::while_let_on_iterator, clippy::await_holding_lock, clippy::unnecessary_cast, clippy::drop_non_drop, clippy::needless_range_loop, unused_imports, unused_variables, dead_code, unsafe_code, clippy::collapsible_if, clippy::bool_comparison, unexpected_cfgs)]
//! Unit tests for TimingConfig.

use keyrx_core::engine::TimingConfig;

#[test]
fn defaults_match_spec() {
    let config = TimingConfig::default();
    assert_eq!(config.tap_timeout_ms, 200);
    assert_eq!(config.combo_timeout_ms, 50);
    assert_eq!(config.hold_delay_ms, 0);
    assert!(!config.eager_tap);
    assert!(config.permissive_hold);
    assert!(!config.retro_tap);
}

#[test]
fn builder_overrides_fields() {
    let config = TimingConfig::builder()
        .tap_timeout_ms(150)
        .combo_timeout_ms(30)
        .hold_delay_ms(10)
        .eager_tap(true)
        .permissive_hold(false)
        .retro_tap(true)
        .build();

    assert_eq!(
        config,
        TimingConfig {
            tap_timeout_ms: 150,
            combo_timeout_ms: 30,
            hold_delay_ms: 10,
            eager_tap: true,
            permissive_hold: false,
            retro_tap: true,
        }
    );
}

#[test]
fn serde_roundtrip_preserves_values() {
    let config = TimingConfig::builder()
        .tap_timeout_ms(175)
        .combo_timeout_ms(60)
        .hold_delay_ms(5)
        .eager_tap(true)
        .permissive_hold(true)
        .retro_tap(true)
        .build();

    let json = serde_json::to_string(&config).expect("serialize");
    let decoded: TimingConfig = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(decoded, config);
}
