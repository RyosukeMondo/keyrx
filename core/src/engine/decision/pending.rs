use crate::config::MICROS_PER_MS;
use crate::engine::{HoldAction, InputEvent, KeyCode, LayerAction, TimingConfig};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

/// A pending timing decision waiting for resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingDecision {
    TapHold(TapHoldDecision),
    Combo(ComboDecision),
}

/// Serializable view of a pending decision for inspection/telemetry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PendingDecisionState {
    TapHold {
        key: KeyCode,
        pressed_at: u64,
        deadline: u64,
        tap_action: KeyCode,
        hold_action: HoldAction,
        interrupted: bool,
        eager_tap: bool,
        hold_emitted: bool,
    },
    Combo {
        keys: Vec<KeyCode>,
        started_at: u64,
        deadline: u64,
        matched: Vec<KeyCode>,
        action: LayerAction,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TapHoldDecision {
    key: KeyCode,
    pressed_at: u64,
    deadline: u64,
    tap_action: KeyCode,
    hold_action: HoldAction,
    interrupted: bool,
    eager_tap: bool,
    hold_emitted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComboDecision {
    keys: SmallVec<[KeyCode; 4]>,
    started_at: u64,
    deadline: u64,
    action: LayerAction,
    matched: SmallVec<[KeyCode; 4]>,
}

/// Resolution of a pending decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecisionResolution {
    /// Decision resolved as tap.
    Tap { key: KeyCode, was_eager: bool },
    /// Decision resolved as hold.
    Hold {
        key: KeyCode,
        action: HoldAction,
        /// True if an eager tap was emitted earlier.
        from_eager: bool,
    },
    /// Decision resolved but event should be blocked without output.
    Consume(KeyCode),
    /// Combo matched and triggered.
    ComboTriggered(LayerAction),
    /// Combo timed out, original keys should pass through.
    ComboTimeout(SmallVec<[KeyCode; 4]>),
}

/// Queue of pending decisions (tap-hold and combos).
#[derive(Debug, Default)]
pub struct DecisionQueue {
    pending: Vec<PendingDecision>,
    config: TimingConfig,
}

impl DecisionQueue {
    /// Maximum number of pending decisions to track.
    pub const MAX_PENDING: usize = 32;

    pub fn new(config: TimingConfig) -> Self {
        Self {
            pending: Vec::new(),
            config,
        }
    }

    /// Add a new tap-hold pending decision. Returns an eager tap resolution if configured.
    pub fn add_tap_hold(
        &mut self,
        key: KeyCode,
        pressed_at: u64,
        tap_action: KeyCode,
        hold_action: HoldAction,
    ) -> (bool, Option<DecisionResolution>) {
        if self.pending.len() >= Self::MAX_PENDING {
            return (false, None);
        }

        let tap_timeout_us = (self.config.tap_timeout_ms as u64).saturating_mul(MICROS_PER_MS);
        let hold_delay_us = (self.config.hold_delay_ms as u64).saturating_mul(MICROS_PER_MS);
        let deadline = pressed_at
            .saturating_add(tap_timeout_us)
            .saturating_add(hold_delay_us);

        let eager_tap = self.config.eager_tap;
        let decision = TapHoldDecision {
            key,
            pressed_at,
            deadline,
            tap_action,
            hold_action,
            interrupted: false,
            eager_tap,
            hold_emitted: false,
        };

        let eager_resolution = if eager_tap {
            Some(DecisionResolution::Tap {
                key: tap_action,
                was_eager: true,
            })
        } else {
            None
        };

        self.pending.push(PendingDecision::TapHold(decision));

        (true, eager_resolution)
    }

    /// Add a new combo pending decision. Returns false if queue is full or keys are invalid.
    pub fn add_combo(&mut self, keys: &[KeyCode], started_at: u64, action: LayerAction) -> bool {
        if self.pending.len() >= Self::MAX_PENDING {
            return false;
        }
        if keys.len() < 2 || keys.len() > 4 {
            return false;
        }

        let mut unique_keys: SmallVec<[KeyCode; 4]> = SmallVec::new();
        for key in keys.iter().copied() {
            if !unique_keys.contains(&key) {
                unique_keys.push(key);
            }
        }

        if unique_keys.len() < 2 {
            return false;
        }

        let deadline = started_at
            .saturating_add((self.config.combo_timeout_ms as u64).saturating_mul(MICROS_PER_MS));

        self.pending.push(PendingDecision::Combo(ComboDecision {
            keys: unique_keys.clone(),
            started_at,
            deadline,
            action,
            matched: SmallVec::new(),
        }));

        true
    }

    /// Check for resolution based on a new event.
    pub fn check_event(&mut self, event: &InputEvent) -> Vec<DecisionResolution> {
        let mut resolutions = Vec::new();
        let mut remaining = Vec::with_capacity(self.pending.len());

        for decision in self.pending.drain(..) {
            match decision {
                PendingDecision::TapHold(tap_hold) => {
                    // Resolve on release of the tracked key.
                    if event.key == tap_hold.key && !event.pressed {
                        let interrupted = self.config.permissive_hold && tap_hold.interrupted;
                        let timed_out = event.timestamp_us >= tap_hold.deadline;

                        if tap_hold.hold_emitted {
                            if self.config.retro_tap {
                                resolutions.push(DecisionResolution::Tap {
                                    key: tap_hold.tap_action,
                                    was_eager: false,
                                });
                            }
                            resolutions.push(DecisionResolution::Consume(tap_hold.key));
                        } else if interrupted || timed_out {
                            resolutions.push(DecisionResolution::Hold {
                                key: tap_hold.key,
                                action: tap_hold.hold_action.clone(),
                                from_eager: tap_hold.eager_tap,
                            });
                            if self.config.retro_tap {
                                resolutions.push(DecisionResolution::Tap {
                                    key: tap_hold.tap_action,
                                    was_eager: false,
                                });
                            }
                        } else {
                            resolutions.push(DecisionResolution::Tap {
                                key: tap_hold.tap_action,
                                was_eager: false,
                            });
                        }
                        continue;
                    }

                    remaining.push(PendingDecision::TapHold(tap_hold));
                }
                PendingDecision::Combo(mut combo) => {
                    if event.pressed
                        && combo.keys.contains(&event.key)
                        && !combo.matched.contains(&event.key)
                    {
                        combo.matched.push(event.key);
                        if combo.matched.len() == combo.keys.len() {
                            resolutions.push(DecisionResolution::ComboTriggered(combo.action));
                            continue;
                        }
                    }

                    remaining.push(PendingDecision::Combo(combo));
                }
            }
        }

        self.pending = remaining;
        resolutions
    }

    /// Check for timeout-based resolutions (tap-hold deadlines and combo windows).
    pub fn check_timeouts(&mut self, now_us: u64) -> Vec<DecisionResolution> {
        let mut resolutions = Vec::new();
        let mut remaining = Vec::with_capacity(self.pending.len());

        for decision in self.pending.drain(..) {
            match decision {
                PendingDecision::TapHold(mut tap_hold) => {
                    if tap_hold.hold_emitted {
                        remaining.push(PendingDecision::TapHold(tap_hold));
                        continue;
                    }

                    if now_us >= tap_hold.deadline {
                        resolutions.push(DecisionResolution::Hold {
                            key: tap_hold.key,
                            action: tap_hold.hold_action.clone(),
                            from_eager: tap_hold.eager_tap,
                        });
                        if self.config.retro_tap {
                            tap_hold.hold_emitted = true;
                            remaining.push(PendingDecision::TapHold(tap_hold));
                        }
                    } else {
                        remaining.push(PendingDecision::TapHold(tap_hold));
                    }
                }
                PendingDecision::Combo(combo) => {
                    if now_us >= combo.deadline {
                        let keys = if combo.matched.is_empty() {
                            combo.keys
                        } else {
                            combo.matched
                        };
                        resolutions.push(DecisionResolution::ComboTimeout(keys));
                    } else {
                        remaining.push(PendingDecision::Combo(combo));
                    }
                }
            }
        }

        self.pending = remaining;
        resolutions
    }

    /// Mark tap-hold decisions as interrupted (for permissive_hold).
    pub fn mark_interrupted(&mut self, by_key: KeyCode) {
        for decision in &mut self.pending {
            if let PendingDecision::TapHold(tap_hold) = decision {
                if tap_hold.key != by_key {
                    tap_hold.interrupted = true;
                }
            }
        }
    }

    /// Inspect pending decisions for debugging/telemetry.
    pub fn pending(&self) -> &[PendingDecision] {
        &self.pending
    }

    /// Snapshot pending decisions into a serializable form.
    pub fn snapshot(&self) -> Vec<PendingDecisionState> {
        self.pending.iter().map(PendingDecision::to_state).collect()
    }

    /// Clear all pending decisions.
    pub fn clear(&mut self) {
        self.pending.clear();
    }
}

impl PendingDecision {
    fn to_state(&self) -> PendingDecisionState {
        match self {
            PendingDecision::TapHold(decision) => PendingDecisionState::TapHold {
                key: decision.key,
                pressed_at: decision.pressed_at,
                deadline: decision.deadline,
                tap_action: decision.tap_action,
                hold_action: decision.hold_action.clone(),
                interrupted: decision.interrupted,
                eager_tap: decision.eager_tap,
                hold_emitted: decision.hold_emitted,
            },
            PendingDecision::Combo(combo) => PendingDecisionState::Combo {
                keys: combo.keys.iter().copied().collect(),
                started_at: combo.started_at,
                deadline: combo.deadline,
                matched: combo.matched.iter().copied().collect(),
                action: combo.action.clone(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smallvec::smallvec;

    #[test]
    fn tap_resolves_on_release_before_deadline() {
        let mut queue = DecisionQueue::new(TimingConfig::default());
        let hold_action = HoldAction::Key(KeyCode::C);
        let (added, eager) = queue.add_tap_hold(KeyCode::A, 0, KeyCode::B, hold_action.clone());
        assert!(added);
        assert!(eager.is_none());

        let event = InputEvent::key_up(KeyCode::A, 50_000); // 50ms
        let resolutions = queue.check_event(&event);

        assert_eq!(
            resolutions,
            vec![DecisionResolution::Tap {
                key: KeyCode::B,
                was_eager: false
            }]
        );
        assert!(queue.pending().is_empty());
    }

    #[test]
    fn hold_resolves_on_timeout() {
        let mut queue = DecisionQueue::new(TimingConfig::default());
        let hold_action = HoldAction::Key(KeyCode::C);
        let (added, eager) = queue.add_tap_hold(KeyCode::A, 0, KeyCode::B, hold_action.clone());
        assert!(added);
        assert!(eager.is_none());

        let resolutions = queue.check_timeouts(250_000); // 250ms

        assert_eq!(
            resolutions,
            vec![DecisionResolution::Hold {
                key: KeyCode::A,
                action: hold_action,
                from_eager: false
            }]
        );
        assert!(queue.pending().is_empty());
    }

    #[test]
    fn permissive_hold_on_interruption_before_deadline() {
        let mut queue = DecisionQueue::new(TimingConfig::default());
        let hold_action = HoldAction::Key(KeyCode::C);
        let (added, eager) = queue.add_tap_hold(KeyCode::A, 0, KeyCode::B, hold_action.clone());
        assert!(added);
        assert!(eager.is_none());

        queue.mark_interrupted(KeyCode::Z);
        let resolutions = queue.check_event(&InputEvent::key_up(KeyCode::A, 50_000));

        assert_eq!(
            resolutions,
            vec![DecisionResolution::Hold {
                key: KeyCode::A,
                action: hold_action,
                from_eager: false
            }]
        );
        assert!(queue.pending().is_empty());
    }

    #[test]
    fn eager_tap_emits_immediately_and_upgrades_to_hold() {
        let mut config = TimingConfig::default();
        config.eager_tap = true;
        let mut queue = DecisionQueue::new(config);
        let hold_action = HoldAction::Key(KeyCode::LeftShift);

        let (added, eager) = queue.add_tap_hold(KeyCode::A, 0, KeyCode::B, hold_action.clone());
        assert!(added);
        assert_eq!(
            eager,
            Some(DecisionResolution::Tap {
                key: KeyCode::B,
                was_eager: true
            })
        );

        let resolutions = queue.check_timeouts(250_000);
        assert_eq!(
            resolutions,
            vec![DecisionResolution::Hold {
                key: KeyCode::A,
                action: hold_action,
                from_eager: true
            }]
        );
        assert!(queue.pending().is_empty());
    }

    #[test]
    fn retro_tap_emits_tap_after_hold_on_release() {
        let mut config = TimingConfig::default();
        config.retro_tap = true;
        let mut queue = DecisionQueue::new(config);
        let hold_action = HoldAction::Modifier(1);

        let (added, eager) = queue.add_tap_hold(KeyCode::A, 0, KeyCode::B, hold_action.clone());
        assert!(added);
        assert!(eager.is_none());
        let hold_resolution = queue.check_timeouts(250_000);

        assert_eq!(
            hold_resolution,
            vec![DecisionResolution::Hold {
                key: KeyCode::A,
                action: hold_action,
                from_eager: false
            }]
        );
        assert_eq!(queue.pending().len(), 1);

        let tap_resolution = queue.check_event(&InputEvent::key_up(KeyCode::A, 300_000));
        assert_eq!(
            tap_resolution,
            vec![
                DecisionResolution::Tap {
                    key: KeyCode::B,
                    was_eager: false
                },
                DecisionResolution::Consume(KeyCode::A)
            ]
        );
        assert!(queue.pending().is_empty());
    }

    #[test]
    fn combo_triggers_when_all_keys_pressed_any_order() {
        let mut queue = DecisionQueue::new(TimingConfig::default());
        assert!(queue.add_combo(
            &[KeyCode::A, KeyCode::B],
            0,
            LayerAction::Remap(KeyCode::Escape)
        ));

        let first = queue.check_event(&InputEvent::key_down(KeyCode::B, 10_000));
        assert!(first.is_empty());

        let second = queue.check_event(&InputEvent::key_down(KeyCode::A, 20_000));
        assert_eq!(
            second,
            vec![DecisionResolution::ComboTriggered(LayerAction::Remap(
                KeyCode::Escape
            ))]
        );
        assert!(queue.pending().is_empty());
    }

    #[test]
    fn combo_timeout_returns_matched_keys() {
        let mut queue = DecisionQueue::new(TimingConfig::default());
        assert!(queue.add_combo(
            &[KeyCode::A, KeyCode::B],
            0,
            LayerAction::Remap(KeyCode::Escape)
        ));

        // Only one key pressed before timeout.
        let _ = queue.check_event(&InputEvent::key_down(KeyCode::A, 10_000));
        let resolutions = queue.check_timeouts(100_000); // > combo_timeout_ms (50ms)

        assert_eq!(
            resolutions,
            vec![DecisionResolution::ComboTimeout(smallvec![KeyCode::A])]
        );
        assert!(queue.pending().is_empty());
    }

    #[test]
    fn capacity_limits_prevent_overflow() {
        let mut queue = DecisionQueue::new(TimingConfig::default());
        for _ in 0..DecisionQueue::MAX_PENDING {
            let _ = queue.add_tap_hold(KeyCode::A, 0, KeyCode::B, HoldAction::Key(KeyCode::C));
        }

        assert_eq!(
            queue.add_tap_hold(KeyCode::Z, 0, KeyCode::Z, HoldAction::Key(KeyCode::Z)),
            (false, None)
        );
        assert!(!queue.add_combo(
            &[KeyCode::A, KeyCode::B],
            0,
            LayerAction::Remap(KeyCode::Escape)
        ));
    }

    #[test]
    fn snapshot_includes_tap_hold_fields() {
        let mut queue = DecisionQueue::new(TimingConfig::default());
        let (added, _) = queue.add_tap_hold(
            KeyCode::CapsLock,
            10,
            KeyCode::Escape,
            HoldAction::Modifier(2),
        );
        assert!(added);

        let snapshot = queue.snapshot();
        assert_eq!(snapshot.len(), 1);
        let (key, pressed_at, tap_action, hold_action, interrupted, eager_tap, hold_emitted) =
            match &snapshot[0] {
                PendingDecisionState::TapHold {
                    key,
                    pressed_at,
                    tap_action,
                    hold_action,
                    interrupted,
                    eager_tap,
                    hold_emitted,
                    ..
                } => (
                    *key,
                    *pressed_at,
                    *tap_action,
                    hold_action.clone(),
                    *interrupted,
                    *eager_tap,
                    *hold_emitted,
                ),
                other => {
                    unreachable!("expected TapHold snapshot, got {:?}", other)
                }
            };
        assert_eq!(key, KeyCode::CapsLock);
        assert_eq!(pressed_at, 10);
        assert_eq!(tap_action, KeyCode::Escape);
        assert_eq!(hold_action, HoldAction::Modifier(2));
        assert!(!interrupted);
        assert_eq!(eager_tap, TimingConfig::default().eager_tap);
        assert!(!hold_emitted);
        // Ensure serializable
        serde_json::to_string(&snapshot[0]).expect("serializes");
    }

    #[test]
    fn snapshot_includes_combo_state() {
        let mut queue = DecisionQueue::new(TimingConfig::default());
        assert!(queue.add_combo(&[KeyCode::A, KeyCode::B, KeyCode::C], 5, LayerAction::Block));
        let _ = queue.check_event(&InputEvent::key_down(KeyCode::C, 7));

        let snapshot = queue.snapshot();
        assert_eq!(snapshot.len(), 1);
        let (keys, started_at, matched, action) = match &snapshot[0] {
            PendingDecisionState::Combo {
                keys,
                started_at,
                matched,
                action,
                ..
            } => (keys.clone(), *started_at, matched.clone(), action.clone()),
            other => {
                unreachable!("expected Combo snapshot, got {:?}", other)
            }
        };
        assert!(keys.contains(&KeyCode::A));
        assert!(matched.contains(&KeyCode::C));
        assert_eq!(started_at, 5);
        assert_eq!(action, LayerAction::Block);
        serde_json::to_string(&snapshot[0]).expect("serializes");
    }
}
