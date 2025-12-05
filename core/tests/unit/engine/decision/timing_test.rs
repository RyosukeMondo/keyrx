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
