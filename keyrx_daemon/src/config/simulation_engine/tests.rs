use super::*;
use keyrx_compiler::serialize::serialize;
use keyrx_core::config::{
    BaseKeyMapping, Condition, ConfigRoot, DeviceConfig, DeviceIdentifier, KeyCode, KeyMapping,
    Metadata, Version,
};
use std::io::Write;
use tempfile::NamedTempFile;

/// Writes a real, valid `.krx` file (magic bytes, hash, rkyv structure) so
/// `SimulationEngine::new` - which now loads and validates it like the
/// daemon does - can load it. A literal byte string is no longer a valid
/// fixture: the old engine ignored `krx_data` entirely (that was the bug),
/// the new one actually uses it.
fn write_krx(devices: Vec<DeviceConfig>) -> NamedTempFile {
    let config = ConfigRoot {
        version: Version::current(),
        devices,
        metadata: Metadata {
            compilation_timestamp: 0,
            compiler_version: "test".to_string(),
            source_hash: "test".to_string(),
        },
    };
    let bytes = serialize(&config).expect("serialize test config");
    let mut file = NamedTempFile::new().expect("create temp krx file");
    file.write_all(&bytes).expect("write temp krx file");
    file
}

/// A config with everything the engine tests below exercise: CapsLock as a
/// tap-hold (tap -> Escape, hold -> MD_00), and A remapped to Left only
/// while MD_00 is active (so a hold's effect is observable, not just its
/// absence of a direct KeyEvent).
fn tap_hold_krx() -> NamedTempFile {
    write_krx(vec![DeviceConfig {
        identifier: DeviceIdentifier {
            pattern: "*".to_string(),
        },
        mappings: vec![
            KeyMapping::tap_hold(KeyCode::CapsLock, KeyCode::Escape, 0, 200),
            KeyMapping::conditional(
                Condition::ModifierActive(0),
                vec![BaseKeyMapping::Simple {
                    from: KeyCode::A,
                    to: KeyCode::Left,
                }],
            ),
        ],
    }])
}

fn passthrough_krx() -> NamedTempFile {
    write_krx(vec![DeviceConfig {
        identifier: DeviceIdentifier {
            pattern: "*".to_string(),
        },
        mappings: vec![],
    }])
}

#[test]
fn test_simulation_engine_new_rejects_a_krx_that_fails_validation() {
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(b"not a real krx file").unwrap();
    let result = SimulationEngine::new(file.path());
    assert!(matches!(result, Err(SimulationError::LoadError(_))));
}

#[test]
fn test_simulation_engine_new_loads_a_real_config() {
    let krx_file = tap_hold_krx();
    assert!(SimulationEngine::new(krx_file.path()).is_ok());
}

#[test]
fn test_parse_event_dsl() {
    let dsl = "press:A,wait:50,release:A";
    let sequence = SimulationEngine::parse_event_dsl(dsl, 0).unwrap();

    assert_eq!(sequence.events.len(), 2);
    assert_eq!(sequence.events[0].key, "A");
    assert_eq!(sequence.events[0].event_type, EventType::Press);
    assert_eq!(sequence.events[0].timestamp_us, 0);

    assert_eq!(sequence.events[1].key, "A");
    assert_eq!(sequence.events[1].event_type, EventType::Release);
    assert_eq!(sequence.events[1].timestamp_us, 50_000);
}

#[test]
fn test_parse_event_dsl_invalid() {
    let result = SimulationEngine::parse_event_dsl("invalid", 0);
    assert!(result.is_err());
}

#[test]
fn test_replay_deterministic() {
    let krx_file = tap_hold_krx();
    let mut engine = SimulationEngine::new(krx_file.path()).unwrap();

    let sequence = EventSequence {
        events: vec![
            SimulatedEvent {
                device_id: None,
                timestamp_us: 0,
                key: "A".to_string(),
                event_type: EventType::Press,
            },
            SimulatedEvent {
                device_id: None,
                timestamp_us: 50_000,
                key: "A".to_string(),
                event_type: EventType::Release,
            },
        ],
        seed: 42,
    };

    let result1 = engine.replay(&sequence).unwrap();
    let result2 = engine.replay(&sequence).unwrap();

    assert_eq!(result1, result2);
}

/// This is the bug `simulate`/`test` shipped with: the old engine hardcoded
/// "CapsLock" tap/hold and ignored the loaded config entirely. Here A is not
/// hardcoded at all - it is Left only because the loaded config's real
/// tap-hold mapping activated MD_00.
#[test]
fn test_replay_uses_the_loaded_config_not_a_hardcoded_rule() {
    let krx_file = tap_hold_krx();
    let mut engine = SimulationEngine::new(krx_file.path()).unwrap();

    // A quick tap of CapsLock produces Escape (from the loaded config).
    let tap = EventSequence {
        events: vec![
            SimulatedEvent {
                device_id: None,
                timestamp_us: 0,
                key: "CapsLock".to_string(),
                event_type: EventType::Press,
            },
            SimulatedEvent {
                device_id: None,
                timestamp_us: 50_000,
                key: "CapsLock".to_string(),
                event_type: EventType::Release,
            },
        ],
        seed: 0,
    };
    let tap_output = engine.replay(&tap).unwrap();
    assert_eq!(tap_output.len(), 2);
    assert_eq!(tap_output[0].key, "Escape");
    assert_eq!(tap_output[0].event_type, EventType::Press);

    // Holding CapsLock past its 200ms threshold, then pressing A, remaps A
    // to Left (per the loaded config's `when(MD_00)` mapping) - proof the
    // hold actually activated the config's own modifier.
    let hold_then_a = EventSequence {
        events: vec![
            SimulatedEvent {
                device_id: None,
                timestamp_us: 0,
                key: "CapsLock".to_string(),
                event_type: EventType::Press,
            },
            SimulatedEvent {
                device_id: None,
                timestamp_us: 250_000,
                key: "A".to_string(),
                event_type: EventType::Press,
            },
            SimulatedEvent {
                device_id: None,
                timestamp_us: 260_000,
                key: "A".to_string(),
                event_type: EventType::Release,
            },
        ],
        seed: 0,
    };
    let hold_output = engine.replay(&hold_then_a).unwrap();
    let a_press = hold_output
        .iter()
        .find(|e| e.event_type == EventType::Press)
        .expect("A should have produced a press output");
    assert_eq!(a_press.key, "Left");
}

#[test]
fn test_event_sequence_validation() {
    let krx_file = tap_hold_krx();
    let mut engine = SimulationEngine::new(krx_file.path()).unwrap();

    // Create sequence with too many events
    let events: Vec<SimulatedEvent> = (0..MAX_EVENT_COUNT + 1)
        .map(|i| SimulatedEvent {
            device_id: None,
            timestamp_us: i as u64,
            key: "A".to_string(),
            event_type: EventType::Press,
        })
        .collect();

    let sequence = EventSequence { events, seed: 0 };
    let result = engine.replay(&sequence);

    assert!(matches!(result, Err(SimulationError::TooManyEvents(_))));
}

#[test]
fn test_replay_rejects_an_unknown_key_name() {
    let krx_file = passthrough_krx();
    let mut engine = SimulationEngine::new(krx_file.path()).unwrap();
    let sequence = EventSequence {
        events: vec![SimulatedEvent {
            device_id: None,
            timestamp_us: 0,
            key: "NotAKey".to_string(),
            event_type: EventType::Press,
        }],
        seed: 0,
    };
    assert!(matches!(
        engine.replay(&sequence),
        Err(SimulationError::InvalidEventFile(_))
    ));
}

/// The built-in scenarios are replayed against whatever profile is loaded,
/// so there is no config-specific expected output to assert against - only
/// an invariant that holds for ANY config: nothing is left stuck (pressed
/// but never released) at the end of the run.
#[test]
fn test_all_scenarios_pass_the_no_stuck_keys_invariant() {
    let krx_file = passthrough_krx();
    let mut engine = SimulationEngine::new(krx_file.path()).unwrap();

    let results = engine.run_all_scenarios().unwrap();

    assert_eq!(results.len(), BuiltinScenario::all().len());
    for result in &results {
        assert!(result.passed, "{}: {:?}", result.scenario, result.error);
    }
}

#[test]
fn test_run_scenario_reports_the_real_engine_output() {
    let krx_file = tap_hold_krx();
    let mut engine = SimulationEngine::new(krx_file.path()).unwrap();

    let result = engine
        .run_scenario(BuiltinScenario::TapHoldUnderThreshold)
        .unwrap();

    assert!(result.passed);
    assert_eq!(result.scenario, "tap-hold-under-threshold");
    // The scenario taps CapsLock quickly: the loaded config's real tap-hold
    // mapping resolves that to Escape.
    assert_eq!(result.output[0].key, "Escape");
}
