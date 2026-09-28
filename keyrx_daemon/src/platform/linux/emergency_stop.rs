//! Emergency escape chord: hold Left Ctrl + Right Ctrl + Escape together to
//! ungrab every keyboard and stop the daemon, even if a bad config makes the
//! remapped keyboard otherwise unusable.
//!
//! Detection runs on the raw evdev stream, before any remapping, so it fires
//! on the physical keys regardless of what the live config does with them.
//! This chord was picked because it needs both Ctrl keys plus Escape held at
//! once - unlikely to happen by accident, but typeable with the config
//! broken (see `docs/user-guide/linux-setup.md`).

use std::collections::HashSet;

use keyrx_core::config::KeyCode;
use keyrx_core::runtime::event::KeyEvent;

/// The chord: hold all three simultaneously.
pub const CHORD: [KeyCode; 3] = [KeyCode::LCtrl, KeyCode::RCtrl, KeyCode::Escape];

/// Tracks which of the chord's keys are currently down (across every managed
/// device - the chord works no matter which physical keyboard each key comes
/// from) and fires once when the last one is pressed.
#[derive(Default)]
pub struct EmergencyChord {
    held: HashSet<KeyCode>,
}

impl EmergencyChord {
    pub fn new() -> Self {
        Self::default()
    }

    /// Feeds one raw input event. Returns `true` exactly once per completed
    /// chord - releasing and re-holding a key while the rest stay down does
    /// not re-fire, only fully forming the chord again does.
    pub fn observe(&mut self, event: &KeyEvent) -> bool {
        let key = event.keycode();
        if !CHORD.contains(&key) {
            return false;
        }
        if event.is_press() {
            let was_complete = self.is_complete();
            self.held.insert(key);
            !was_complete && self.is_complete()
        } else {
            self.held.remove(&key);
            false
        }
    }

    fn is_complete(&self) -> bool {
        CHORD.iter().all(|k| self.held.contains(k))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(key: KeyCode) -> KeyEvent {
        KeyEvent::press(key)
    }
    fn release(key: KeyCode) -> KeyEvent {
        KeyEvent::release(key)
    }

    #[test]
    fn fires_only_once_all_three_are_down() {
        let mut chord = EmergencyChord::new();
        assert!(!chord.observe(&press(KeyCode::LCtrl)));
        assert!(!chord.observe(&press(KeyCode::RCtrl)));
        assert!(chord.observe(&press(KeyCode::Escape)));
    }

    #[test]
    fn order_does_not_matter() {
        let mut chord = EmergencyChord::new();
        assert!(!chord.observe(&press(KeyCode::Escape)));
        assert!(!chord.observe(&press(KeyCode::RCtrl)));
        assert!(chord.observe(&press(KeyCode::LCtrl)));
    }

    #[test]
    fn unrelated_keys_are_ignored() {
        let mut chord = EmergencyChord::new();
        assert!(!chord.observe(&press(KeyCode::A)));
        assert!(!chord.observe(&press(KeyCode::LCtrl)));
        assert!(!chord.observe(&press(KeyCode::RCtrl)));
        assert!(!chord.observe(&press(KeyCode::B)));
        assert!(chord.observe(&press(KeyCode::Escape)));
    }

    #[test]
    fn releasing_one_key_requires_the_full_chord_again() {
        let mut chord = EmergencyChord::new();
        assert!(!chord.observe(&press(KeyCode::LCtrl)));
        assert!(!chord.observe(&press(KeyCode::RCtrl)));
        assert!(chord.observe(&press(KeyCode::Escape)));

        // Already fired once; still fully held, must not fire again.
        assert!(!chord.observe(&press(KeyCode::Escape)));

        chord.observe(&release(KeyCode::Escape));
        assert!(!chord.observe(&press(KeyCode::LCtrl))); // still down, no-op
        assert!(chord.observe(&press(KeyCode::Escape))); // fires again
    }

    #[test]
    fn two_of_three_never_fires() {
        let mut chord = EmergencyChord::new();
        assert!(!chord.observe(&press(KeyCode::LCtrl)));
        assert!(!chord.observe(&press(KeyCode::RCtrl)));
        assert!(!chord.observe(&release(KeyCode::LCtrl)));
        assert!(!chord.observe(&press(KeyCode::Escape)));
    }
}
