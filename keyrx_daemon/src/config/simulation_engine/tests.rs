use super::*;
use std::io::Write;
use tempfile::NamedTempFile;

fn create_test_krx() -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(b"test krx data").unwrap();
    file
}

#[test]
fn test_virtual_clock() {
    let mut clock = VirtualClock::new(42);
    assert_eq!(clock.now_us(), 0);

    clock.advance(1000);
    assert_eq!(clock.now_us(), 1000);

    clock.advance(500);
    assert_eq!(clock.now_us(), 1500);

    clock.reset();
    assert_eq!(clock.now_us(), 0);
}

#[test]
fn test_simulation_engine_new() {
    let krx_file = create_test_krx();
    let engine = SimulationEngine::new(krx_file.path()).unwrap();
    assert_eq!(engine.krx_data, b"test krx data");
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
    let krx_file = create_test_krx();
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

#[test]
fn test_builtin_scenario_tap_hold_under_threshold() {
    let krx_file = create_test_krx();
    let mut engine = SimulationEngine::new(krx_file.path()).unwrap();

    let result = engine
        .run_scenario(BuiltinScenario::TapHoldUnderThreshold)
        .unwrap();

    assert!(result.passed);
    assert_eq!(result.scenario, "tap-hold-under-threshold");
    assert_eq!(result.output.len(), 2); // Press and release
    assert_eq!(result.output[0].key, "Escape"); // Tap action (press)
    assert_eq!(result.output[0].event_type, EventType::Press);
    assert_eq!(result.output[1].key, "Escape"); // Tap action (release)
    assert_eq!(result.output[1].event_type, EventType::Release);
}

#[test]
fn test_builtin_scenario_tap_hold_over_threshold() {
    let krx_file = create_test_krx();
    let mut engine = SimulationEngine::new(krx_file.path()).unwrap();

    let result = engine
        .run_scenario(BuiltinScenario::TapHoldOverThreshold)
        .unwrap();

    assert!(result.passed);
    assert_eq!(result.output.len(), 2); // Press and release
    assert_eq!(result.output[0].key, "Control"); // Hold action (press)
    assert_eq!(result.output[0].event_type, EventType::Press);
    assert_eq!(result.output[1].key, "Control"); // Hold action (release)
    assert_eq!(result.output[1].event_type, EventType::Release);
}

#[test]
fn test_event_sequence_validation() {
    let krx_file = create_test_krx();
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
fn test_all_scenarios() {
    let krx_file = create_test_krx();
    let mut engine = SimulationEngine::new(krx_file.path()).unwrap();

    let results = engine.run_all_scenarios().unwrap();

    assert_eq!(results.len(), BuiltinScenario::all().len());
    assert!(results.iter().all(|r| r.passed));
}
