//! Emergency stop detection: ungrab every keyboard and stop the daemon even
//! if a bad config makes the remapped keyboard otherwise unusable.
//!
//! Detection runs on the raw evdev stream, before any remapping, so it fires
//! on the physical keys regardless of what the live config does with them.
//! What triggers it is an [`EmergencyConfig`]: a chord (default Left Ctrl +
//! Right Ctrl + Escape) and the one-handed alternative of holding Escape alone
//! for a few seconds (see `platform::emergency` for why that one).

use std::collections::HashSet;
use std::time::Instant;

use keyrx_core::config::KeyCode;
use keyrx_core::runtime::event::KeyEvent;

use crate::platform::emergency::{EmergencyConfig, HOLD_KEY};

/// Tracks the physical keys that are down (across every managed device - the
/// chord works no matter which physical keyboard each key comes from) and
/// reports when the chord completes or the hold elapses.
pub struct EmergencyDetector {
    config: EmergencyConfig,
    /// Every physical key currently down.
    down: HashSet<KeyCode>,
    /// When [`HOLD_KEY`] went down with no other key down; cleared by any
    /// other key press or by its release.
    hold_since: Option<Instant>,
}

impl EmergencyDetector {
    pub fn new(config: EmergencyConfig) -> Self {
        Self {
            config,
            down: HashSet::new(),
            hold_since: None,
        }
    }

    /// Feeds one raw input event seen at `now`. Returns `true` exactly once
    /// per completed chord - releasing and re-holding a key while the rest
    /// stay down does not re-fire, only fully forming the chord again does.
    pub fn observe(&mut self, event: &KeyEvent, now: Instant) -> bool {
        let key = event.keycode();
        if !event.is_press() {
            self.down.remove(&key);
            if key == HOLD_KEY {
                self.hold_since = None;
            }
            return false;
        }
        let was_complete = self.chord_complete();
        let others_down = self.down.iter().any(|k| *k != key);
        self.down.insert(key);
        if key == HOLD_KEY && !others_down {
            self.hold_since = Some(now);
        } else {
            // Any other key means this is normal typing, not an emergency.
            self.hold_since = None;
        }
        !was_complete && self.chord_complete()
    }

    /// Returns `true` once the one-handed hold has lasted long enough. Call
    /// regularly (the event loop does, about every 10 ms); no key events
    /// arrive while a key is simply held, so time must be checked separately.
    pub fn poll(&mut self, now: Instant) -> bool {
        let (Some(since), Some(hold)) = (self.hold_since, self.config.hold) else {
            return false;
        };
        if now.saturating_duration_since(since) >= hold {
            self.hold_since = None;
            return true;
        }
        false
    }

    fn chord_complete(&self) -> bool {
        self.config.chord.iter().all(|k| self.down.contains(k))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn press(key: KeyCode) -> KeyEvent {
        KeyEvent::press(key)
    }
    fn release(key: KeyCode) -> KeyEvent {
        KeyEvent::release(key)
    }
    fn detector() -> EmergencyDetector {
        EmergencyDetector::new(EmergencyConfig::default())
    }
    fn at(ms: u64, base: Instant) -> Instant {
        base + Duration::from_millis(ms)
    }

    #[test]
    fn fires_only_once_all_three_are_down() {
        let (mut d, t) = (detector(), Instant::now());
        assert!(!d.observe(&press(KeyCode::LCtrl), t));
        assert!(!d.observe(&press(KeyCode::RCtrl), t));
        assert!(d.observe(&press(KeyCode::Escape), t));
    }

    #[test]
    fn order_does_not_matter() {
        let (mut d, t) = (detector(), Instant::now());
        assert!(!d.observe(&press(KeyCode::Escape), t));
        assert!(!d.observe(&press(KeyCode::RCtrl), t));
        assert!(d.observe(&press(KeyCode::LCtrl), t));
    }

    #[test]
    fn unrelated_keys_are_ignored() {
        let (mut d, t) = (detector(), Instant::now());
        assert!(!d.observe(&press(KeyCode::A), t));
        assert!(!d.observe(&press(KeyCode::LCtrl), t));
        assert!(!d.observe(&press(KeyCode::RCtrl), t));
        assert!(!d.observe(&press(KeyCode::B), t));
        assert!(d.observe(&press(KeyCode::Escape), t));
    }

    #[test]
    fn releasing_one_key_requires_the_full_chord_again() {
        let (mut d, t) = (detector(), Instant::now());
        d.observe(&press(KeyCode::LCtrl), t);
        d.observe(&press(KeyCode::RCtrl), t);
        assert!(d.observe(&press(KeyCode::Escape), t));
        assert!(
            !d.observe(&press(KeyCode::Escape), t),
            "no re-fire while held"
        );
        d.observe(&release(KeyCode::Escape), t);
        assert!(d.observe(&press(KeyCode::Escape), t), "fires again");
    }

    #[test]
    fn two_of_three_never_fires() {
        let (mut d, t) = (detector(), Instant::now());
        d.observe(&press(KeyCode::LCtrl), t);
        d.observe(&press(KeyCode::RCtrl), t);
        d.observe(&release(KeyCode::LCtrl), t);
        assert!(!d.observe(&press(KeyCode::Escape), t));
    }

    #[test]
    fn a_configured_chord_replaces_the_default() {
        let mut cfg = EmergencyConfig::default();
        cfg.set_chord("LShift+RShift").unwrap();
        let (mut d, t) = (EmergencyDetector::new(cfg), Instant::now());
        d.observe(&press(KeyCode::LCtrl), t);
        d.observe(&press(KeyCode::RCtrl), t);
        assert!(!d.observe(&press(KeyCode::Escape), t), "old chord is dead");
        d.observe(&press(KeyCode::LShift), t);
        assert!(d.observe(&press(KeyCode::RShift), t));
    }

    #[test]
    fn holding_escape_alone_for_three_seconds_fires_once() {
        let (mut d, t) = (detector(), Instant::now());
        d.observe(&press(KeyCode::Escape), t);
        assert!(!d.poll(at(2_999, t)));
        assert!(d.poll(at(3_000, t)));
        assert!(!d.poll(at(3_100, t)), "fires once");
    }

    #[test]
    fn releasing_escape_early_cancels_the_hold() {
        let (mut d, t) = (detector(), Instant::now());
        d.observe(&press(KeyCode::Escape), t);
        d.observe(&release(KeyCode::Escape), at(1_000, t));
        assert!(!d.poll(at(5_000, t)));
    }

    #[test]
    fn any_other_key_cancels_the_hold() {
        let (mut d, t) = (detector(), Instant::now());
        d.observe(&press(KeyCode::Escape), t);
        d.observe(&press(KeyCode::A), at(500, t));
        d.observe(&release(KeyCode::A), at(600, t));
        assert!(
            !d.poll(at(5_000, t)),
            "typing during the hold means no emergency"
        );
    }

    #[test]
    fn escape_pressed_while_another_key_is_held_never_arms_the_hold() {
        let (mut d, t) = (detector(), Instant::now());
        d.observe(&press(KeyCode::LShift), t);
        d.observe(&press(KeyCode::Escape), at(10, t));
        assert!(!d.poll(at(10_000, t)));
    }

    #[test]
    fn mashing_escape_is_not_a_hold() {
        let (mut d, t) = (detector(), Instant::now());
        for i in 0..20 {
            d.observe(&press(KeyCode::Escape), at(i * 200, t));
            d.observe(&release(KeyCode::Escape), at(i * 200 + 100, t));
            assert!(!d.poll(at(i * 200 + 150, t)));
        }
    }

    #[test]
    fn the_hold_can_be_disabled_and_resized() {
        let mut cfg = EmergencyConfig::default();
        cfg.set_hold_ms(0).unwrap();
        let (mut d, t) = (EmergencyDetector::new(cfg.clone()), Instant::now());
        d.observe(&press(KeyCode::Escape), t);
        assert!(!d.poll(at(60_000, t)));

        cfg.set_hold_ms(1_500).unwrap();
        let mut d = EmergencyDetector::new(cfg);
        d.observe(&press(KeyCode::Escape), t);
        assert!(!d.poll(at(1_499, t)));
        assert!(d.poll(at(1_500, t)));
    }
}
