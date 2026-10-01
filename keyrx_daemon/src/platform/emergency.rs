//! Configuration of the emergency stop (Linux): how a user who is locked out
//! by a bad config releases every keyboard and stops the daemon.
//!
//! Two independent ways, both detected on the raw (pre-remap) key stream:
//!
//! - the **chord**: hold a set of keys together (default Left Ctrl + Right
//!   Ctrl + Escape). Configurable, because that chord needs two hands;
//! - the **hold**: hold Escape alone for a few seconds (default 3 s) without
//!   touching any other key. This is the one-handed way. It was chosen over
//!   "press Escape N times" because mashing Escape is common (vim, closing
//!   dialogs) while holding it for seconds with nothing else pressed is not,
//!   and it needs no timing precision, which matters with a tremor.
//!
//! The chord cannot be disabled (there must always be a way out); the hold
//! can (`hold_ms = 0`).

use std::time::Duration;

use keyrx_core::config::KeyCode;
use keyrx_core::parser::validators::parse_physical_key;

/// The default chord: needs both hands, unlikely by accident.
pub const DEFAULT_CHORD: [KeyCode; 3] = [KeyCode::LCtrl, KeyCode::RCtrl, KeyCode::Escape];

/// The key of the one-handed hold.
pub const HOLD_KEY: KeyCode = KeyCode::Escape;

/// Default hold time of [`HOLD_KEY`].
pub const DEFAULT_HOLD: Duration = Duration::from_secs(3);

/// Shortest allowed hold: below this a normal long press would trigger it.
pub const MIN_HOLD: Duration = Duration::from_secs(1);

/// Longest allowed hold: beyond this nobody would wait it out locked out.
pub const MAX_HOLD: Duration = Duration::from_secs(30);

/// A chord needs at least two keys (one key is too easy to hit) and at most
/// this many (more cannot be held by one hand and gets unreliable on
/// keyboards with limited rollover).
pub const MIN_CHORD_KEYS: usize = 2;
pub const MAX_CHORD_KEYS: usize = 4;

/// The resolved emergency-stop settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmergencyConfig {
    /// Keys that must all be down together.
    pub chord: Vec<KeyCode>,
    /// How long [`HOLD_KEY`] must be held alone; `None` disables the hold.
    pub hold: Option<Duration>,
}

impl Default for EmergencyConfig {
    fn default() -> Self {
        Self {
            chord: DEFAULT_CHORD.to_vec(),
            hold: Some(DEFAULT_HOLD),
        }
    }
}

impl EmergencyConfig {
    /// Replaces the chord with `text`, e.g. `LCtrl+RCtrl+Escape`.
    ///
    /// # Errors
    ///
    /// A message naming the problem when a key is unknown, repeated, or the
    /// chord has too few or too many keys.
    pub fn set_chord(&mut self, text: &str) -> Result<(), String> {
        self.chord = parse_chord(text)?;
        Ok(())
    }

    /// Sets the hold time in milliseconds; `0` disables the hold.
    ///
    /// # Errors
    ///
    /// A message when the value is outside the allowed range.
    pub fn set_hold_ms(&mut self, ms: u64) -> Result<(), String> {
        self.hold = if ms == 0 {
            None
        } else {
            let hold = Duration::from_millis(ms);
            if !(MIN_HOLD..=MAX_HOLD).contains(&hold) {
                return Err(format!(
                    "emergency hold must be 0 (off) or {}..{} ms, got {ms}",
                    MIN_HOLD.as_millis(),
                    MAX_HOLD.as_millis()
                ));
            }
            Some(hold)
        };
        Ok(())
    }

    /// How a user reads the escape hatches. Printed at startup, by `doctor`
    /// and in the docs, so nobody has to look it up while locked out.
    #[must_use]
    pub fn describe(&self) -> String {
        let chord = self
            .chord
            .iter()
            .map(|k| key_label(*k))
            .collect::<Vec<_>>()
            .join(" + ");
        match self.hold {
            Some(hold) => format!(
                "hold {chord} together, or (one hand) hold {} alone for {} s with no \
                 other key pressed",
                key_label(HOLD_KEY),
                hold.as_secs_f64()
            ),
            None => format!("hold {chord} together"),
        }
    }
}

/// Parses `LCtrl+RCtrl+Escape` (key names as in the DSL; `VK_` optional).
///
/// # Errors
///
/// See [`EmergencyConfig::set_chord`].
pub fn parse_chord(text: &str) -> Result<Vec<KeyCode>, String> {
    let mut keys = Vec::new();
    for name in text.split('+').map(str::trim) {
        let key = parse_physical_key(name)
            .map_err(|_| format!("unknown key '{name}' in emergency chord '{text}'"))?;
        if keys.contains(&key) {
            return Err(format!(
                "key '{name}' is repeated in emergency chord '{text}'"
            ));
        }
        keys.push(key);
    }
    if !(MIN_CHORD_KEYS..=MAX_CHORD_KEYS).contains(&keys.len()) {
        return Err(format!(
            "emergency chord needs {MIN_CHORD_KEYS} to {MAX_CHORD_KEYS} keys joined by '+', got '{text}'"
        ));
    }
    Ok(keys)
}

/// A human label for `key` in the emergency text.
fn key_label(key: KeyCode) -> String {
    match key {
        KeyCode::LCtrl => "Left Ctrl".into(),
        KeyCode::RCtrl => "Right Ctrl".into(),
        KeyCode::LShift => "Left Shift".into(),
        KeyCode::RShift => "Right Shift".into(),
        KeyCode::LAlt => "Left Alt".into(),
        KeyCode::RAlt => "Right Alt".into(),
        KeyCode::LMeta => "Left Meta".into(),
        KeyCode::RMeta => "Right Meta".into(),
        other => format!("{other:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_the_documented_chord_and_a_three_second_hold() {
        let cfg = EmergencyConfig::default();
        assert_eq!(cfg.chord, DEFAULT_CHORD);
        assert_eq!(cfg.hold, Some(Duration::from_secs(3)));
        assert_eq!(
            cfg.describe(),
            "hold Left Ctrl + Right Ctrl + Escape together, or (one hand) hold Escape alone \
             for 3 s with no other key pressed"
        );
    }

    #[test]
    fn chord_is_parsed_with_or_without_vk_prefix() {
        assert_eq!(
            parse_chord("LShift + VK_RShift+Escape").unwrap(),
            vec![KeyCode::LShift, KeyCode::RShift, KeyCode::Escape]
        );
    }

    #[test]
    fn bad_chords_are_rejected_with_the_reason() {
        assert!(parse_chord("Escape").unwrap_err().contains("2 to 4"));
        assert!(parse_chord("A+B+C+D+E").unwrap_err().contains("2 to 4"));
        assert!(parse_chord("Escape+Escape")
            .unwrap_err()
            .contains("repeated"));
        assert!(parse_chord("Escape+Bogus").unwrap_err().contains("Bogus"));
        assert!(parse_chord("").is_err());
    }

    #[test]
    fn hold_is_off_at_zero_and_bounded_otherwise() {
        let mut cfg = EmergencyConfig::default();
        cfg.set_hold_ms(0).unwrap();
        assert_eq!(cfg.hold, None);
        assert!(cfg.describe().ends_with("together"));
        assert!(
            cfg.set_hold_ms(200).is_err(),
            "too short, would trigger by accident"
        );
        assert!(cfg.set_hold_ms(60_000).is_err());
        cfg.set_hold_ms(5_000).unwrap();
        assert_eq!(cfg.hold, Some(Duration::from_secs(5)));
    }
}
