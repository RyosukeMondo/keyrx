//! Built-in test scenarios and their canned event sequences.

use serde::{Deserialize, Serialize};

use super::types::{EventSequence, EventType, SimulatedEvent};

/// Built-in test scenarios
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BuiltinScenario {
    TapHoldUnderThreshold,
    TapHoldOverThreshold,
    PermissiveHold,
    CrossDeviceModifiers,
    MacroSequence,
}

impl BuiltinScenario {
    /// Get all available scenarios
    pub fn all() -> Vec<Self> {
        vec![
            Self::TapHoldUnderThreshold,
            Self::TapHoldOverThreshold,
            Self::PermissiveHold,
            Self::CrossDeviceModifiers,
            Self::MacroSequence,
        ]
    }

    /// Get scenario name as string
    pub fn name(&self) -> &'static str {
        match self {
            Self::TapHoldUnderThreshold => "tap-hold-under-threshold",
            Self::TapHoldOverThreshold => "tap-hold-over-threshold",
            Self::PermissiveHold => "permissive-hold",
            Self::CrossDeviceModifiers => "cross-device-modifiers",
            Self::MacroSequence => "macro-sequence",
        }
    }

    /// Generate event sequence for this scenario
    pub fn generate_events(&self) -> EventSequence {
        match self {
            Self::TapHoldUnderThreshold => EventSequence {
                events: vec![
                    SimulatedEvent {
                        device_id: None,
                        timestamp_us: 0,
                        key: "CapsLock".to_string(),
                        event_type: EventType::Press,
                    },
                    SimulatedEvent {
                        device_id: None,
                        timestamp_us: 50_000, // 50ms - under typical 200ms threshold
                        key: "CapsLock".to_string(),
                        event_type: EventType::Release,
                    },
                ],
                seed: 0,
            },
            Self::TapHoldOverThreshold => EventSequence {
                events: vec![
                    SimulatedEvent {
                        device_id: None,
                        timestamp_us: 0,
                        key: "CapsLock".to_string(),
                        event_type: EventType::Press,
                    },
                    SimulatedEvent {
                        device_id: None,
                        timestamp_us: 250_000, // 250ms - over threshold
                        key: "CapsLock".to_string(),
                        event_type: EventType::Release,
                    },
                ],
                seed: 0,
            },
            Self::PermissiveHold => EventSequence {
                events: vec![
                    SimulatedEvent {
                        device_id: None,
                        timestamp_us: 0,
                        key: "CapsLock".to_string(),
                        event_type: EventType::Press,
                    },
                    SimulatedEvent {
                        device_id: None,
                        timestamp_us: 100_000,
                        key: "A".to_string(),
                        event_type: EventType::Press,
                    },
                    SimulatedEvent {
                        device_id: None,
                        timestamp_us: 150_000,
                        key: "A".to_string(),
                        event_type: EventType::Release,
                    },
                    SimulatedEvent {
                        device_id: None,
                        timestamp_us: 300_000,
                        key: "CapsLock".to_string(),
                        event_type: EventType::Release,
                    },
                ],
                seed: 0,
            },
            Self::CrossDeviceModifiers => EventSequence {
                events: vec![
                    SimulatedEvent {
                        device_id: Some("device1".to_string()),
                        timestamp_us: 0,
                        key: "Shift".to_string(),
                        event_type: EventType::Press,
                    },
                    SimulatedEvent {
                        device_id: Some("device2".to_string()),
                        timestamp_us: 50_000,
                        key: "A".to_string(),
                        event_type: EventType::Press,
                    },
                    SimulatedEvent {
                        device_id: Some("device2".to_string()),
                        timestamp_us: 100_000,
                        key: "A".to_string(),
                        event_type: EventType::Release,
                    },
                    SimulatedEvent {
                        device_id: Some("device1".to_string()),
                        timestamp_us: 150_000,
                        key: "Shift".to_string(),
                        event_type: EventType::Release,
                    },
                ],
                seed: 0,
            },
            Self::MacroSequence => EventSequence {
                events: vec![
                    SimulatedEvent {
                        device_id: None,
                        timestamp_us: 0,
                        key: "F13".to_string(),
                        event_type: EventType::Press,
                    },
                    SimulatedEvent {
                        device_id: None,
                        timestamp_us: 50_000,
                        key: "F13".to_string(),
                        event_type: EventType::Release,
                    },
                ],
                seed: 0,
            },
        }
    }
}
