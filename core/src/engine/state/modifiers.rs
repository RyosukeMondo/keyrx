use serde::{Deserialize, Serialize};

use crate::config::MAX_MODIFIER_ID;
use crate::traits::ModifierProvider;

/// Identifies a modifier, either a standard OS modifier or a virtual one.
///
/// Modifiers can be either:
/// - Standard OS modifiers (Shift, Control, Alt, Meta)
/// - Virtual modifiers (user-defined, identified by a u8 ID)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Modifier {
    /// A standard OS modifier key.
    Standard(StandardModifier),
    /// A virtual modifier identified by its unique ID (0-255).
    Virtual(u8),
}

/// Standard OS modifiers tracked with a compact bitmask.
///
/// These are the four common modifier keys found on most keyboards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StandardModifier {
    /// The Shift key.
    Shift,
    /// The Control key.
    Control,
    /// The Alt key (Option on macOS).
    Alt,
    /// The Meta key (Windows key or Command on macOS).
    Meta,
}

/// Bitset for the four standard modifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct StandardModifiers {
    bits: u8,
}

impl StandardModifiers {
    const SHIFT: u8 = 1 << 0;
    const CONTROL: u8 = 1 << 1;
    const ALT: u8 = 1 << 2;
    const META: u8 = 1 << 3;

    /// Activates the specified modifier.
    pub fn activate(&mut self, modifier: StandardModifier) {
        self.bits |= Self::mask(modifier);
    }

    /// Deactivates the specified modifier.
    pub fn deactivate(&mut self, modifier: StandardModifier) {
        self.bits &= !Self::mask(modifier);
    }

    /// Returns `true` if the specified modifier is currently active.
    #[inline]
    pub fn is_active(&self, modifier: StandardModifier) -> bool {
        self.bits & Self::mask(modifier) != 0
    }

    /// Clears all active modifiers.
    pub fn clear(&mut self) {
        self.bits = 0;
    }

    #[inline]
    fn mask(modifier: StandardModifier) -> u8 {
        match modifier {
            StandardModifier::Shift => Self::SHIFT,
            StandardModifier::Control => Self::CONTROL,
            StandardModifier::Alt => Self::ALT,
            StandardModifier::Meta => Self::META,
        }
    }
}

/// Fixed-size bitmap for 256 virtual modifiers.
///
/// Virtual modifiers are user-defined modifiers stored as a compact bitmap.
/// Each modifier is identified by a u8 ID (0-255).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct VirtualModifiers {
    bits: [u64; 4],
}

impl VirtualModifiers {
    /// Maximum virtual modifier ID (re-exported from config for convenience).
    pub const MAX_ID: u8 = MAX_MODIFIER_ID;

    /// Activates the virtual modifier with the given ID.
    pub fn activate(&mut self, id: u8) {
        let (idx, mask) = Self::index_mask(id);
        self.bits[idx] |= mask;
    }

    /// Deactivates the virtual modifier with the given ID.
    pub fn deactivate(&mut self, id: u8) {
        let (idx, mask) = Self::index_mask(id);
        self.bits[idx] &= !mask;
    }

    /// Returns `true` if the virtual modifier with the given ID is active.
    #[inline]
    pub fn is_active(&self, id: u8) -> bool {
        let (idx, mask) = Self::index_mask(id);
        self.bits[idx] & mask != 0
    }

    /// Clears all active virtual modifiers.
    pub fn clear(&mut self) {
        self.bits = [0; 4];
    }

    #[inline]
    fn index_mask(id: u8) -> (usize, u64) {
        let idx = (id / 64) as usize;
        let shift = id % 64;
        (idx, 1u64 << shift)
    }
}

/// Tracks one-shot (sticky) modifiers that apply to the next event only.
///
/// One-shot modifiers are "armed" and then consumed on the next key event,
/// providing temporary modifier behavior without holding down the key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct OneShotState {
    standard: StandardModifiers,
    virtual_mods: VirtualModifiers,
}

impl OneShotState {
    /// Arms a one-shot modifier so it will be active for the next event.
    pub fn arm(&mut self, modifier: Modifier) {
        match modifier {
            Modifier::Standard(m) => self.standard.activate(m),
            Modifier::Virtual(id) => self.virtual_mods.activate(id),
        }
    }

    /// Consumes the one-shot flag. Returns true if a flag was set.
    pub fn consume(&mut self, modifier: Modifier) -> bool {
        match modifier {
            Modifier::Standard(m) => {
                let was_set = self.standard.is_active(m);
                self.standard.deactivate(m);
                was_set
            }
            Modifier::Virtual(id) => {
                let was_set = self.virtual_mods.is_active(id);
                self.virtual_mods.deactivate(id);
                was_set
            }
        }
    }

    /// Returns `true` if the specified modifier is armed as a one-shot.
    #[inline]
    pub fn is_armed(&self, modifier: Modifier) -> bool {
        match modifier {
            Modifier::Standard(m) => self.standard.is_active(m),
            Modifier::Virtual(id) => self.virtual_mods.is_active(id),
        }
    }

    /// Clears all armed one-shot modifiers.
    pub fn clear(&mut self) {
        self.standard.clear();
        self.virtual_mods.clear();
    }
}

/// Combines standard, virtual, and one-shot modifiers into a single state.
///
/// This is the main modifier tracking structure that handles all types of modifiers
/// and their interactions, including one-shot behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ModifierState {
    standard: StandardModifiers,
    virtual_mods: VirtualModifiers,
    one_shot: OneShotState,
}

impl ModifierState {
    /// Creates a new empty modifier state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Activates the specified modifier and clears any pending one-shot for it.
    pub fn activate(&mut self, modifier: Modifier) {
        match modifier {
            Modifier::Standard(m) => self.standard.activate(m),
            Modifier::Virtual(id) => self.virtual_mods.activate(id),
        }
        // A real activation supersedes a pending one-shot.
        self.one_shot.consume(modifier);
    }

    /// Deactivates the specified modifier and clears any pending one-shot for it.
    pub fn deactivate(&mut self, modifier: Modifier) {
        match modifier {
            Modifier::Standard(m) => self.standard.deactivate(m),
            Modifier::Virtual(id) => self.virtual_mods.deactivate(id),
        }
        // Clear any pending one-shot for the same modifier.
        self.one_shot.consume(modifier);
    }

    /// Returns `true` if the modifier is active (either held or one-shot armed).
    #[inline]
    pub fn is_active(&self, modifier: Modifier) -> bool {
        if self.one_shot.is_armed(modifier) {
            return true;
        }

        match modifier {
            Modifier::Standard(m) => self.standard.is_active(m),
            Modifier::Virtual(id) => self.virtual_mods.is_active(id),
        }
    }

    /// Arms the modifier as a one-shot (active for the next event only).
    pub fn arm_one_shot(&mut self, modifier: Modifier) {
        self.one_shot.arm(modifier);
    }

    /// Consumes a one-shot modifier. Returns `true` if one was cleared.
    pub fn consume_one_shot(&mut self, modifier: Modifier) -> bool {
        self.one_shot.consume(modifier)
    }

    /// Clears all modifier state including one-shots.
    pub fn clear(&mut self) {
        self.standard.clear();
        self.virtual_mods.clear();
        self.one_shot.clear();
    }

    /// Returns the current standard modifiers state.
    pub fn standard(&self) -> StandardModifiers {
        self.standard
    }

    /// Returns the current virtual modifiers state.
    pub fn virtual_mods(&self) -> VirtualModifiers {
        self.virtual_mods
    }
}

impl ModifierProvider for ModifierState {
    fn is_active(&self, modifier: Modifier) -> bool {
        ModifierState::is_active(self, modifier)
    }

    fn activate(&mut self, modifier: Modifier) {
        ModifierState::activate(self, modifier);
    }

    fn deactivate(&mut self, modifier: Modifier) {
        ModifierState::deactivate(self, modifier);
    }

    fn arm_one_shot(&mut self, modifier: Modifier) {
        ModifierState::arm_one_shot(self, modifier);
    }

    fn clear(&mut self) {
        ModifierState::clear(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtual_modifiers_bit_packing() {
        let mut virtuals = VirtualModifiers::default();
        virtuals.activate(0);
        virtuals.activate(63);
        virtuals.activate(64);
        virtuals.activate(128);
        virtuals.activate(255);

        assert!(virtuals.is_active(0));
        assert!(virtuals.is_active(63));
        assert!(virtuals.is_active(64));
        assert!(virtuals.is_active(128));
        assert!(virtuals.is_active(255));

        virtuals.deactivate(63);
        assert!(!virtuals.is_active(63));
    }

    #[test]
    fn standard_modifiers_track_flags() {
        let mut standard = StandardModifiers::default();
        standard.activate(StandardModifier::Shift);
        standard.activate(StandardModifier::Alt);

        assert!(standard.is_active(StandardModifier::Shift));
        assert!(standard.is_active(StandardModifier::Alt));
        assert!(!standard.is_active(StandardModifier::Control));

        standard.deactivate(StandardModifier::Shift);
        assert!(!standard.is_active(StandardModifier::Shift));
    }

    #[test]
    fn modifier_state_combines_standard_and_virtual() {
        let mut state = ModifierState::new();
        state.activate(Modifier::Standard(StandardModifier::Control));
        state.activate(Modifier::Virtual(10));

        assert!(state.is_active(Modifier::Standard(StandardModifier::Control)));
        assert!(state.is_active(Modifier::Virtual(10)));

        state.deactivate(Modifier::Virtual(10));
        assert!(!state.is_active(Modifier::Virtual(10)));
    }

    #[test]
    fn one_shot_applies_once_for_virtual_mod() {
        let mut state = ModifierState::new();
        state.arm_one_shot(Modifier::Virtual(5));

        assert!(state.is_active(Modifier::Virtual(5)));
        assert!(state.consume_one_shot(Modifier::Virtual(5)));
        assert!(!state.is_active(Modifier::Virtual(5)));
        assert!(!state.consume_one_shot(Modifier::Virtual(5)));
    }

    #[test]
    fn one_shot_consumption_preserves_active_flag() {
        let mut state = ModifierState::new();
        state.activate(Modifier::Standard(StandardModifier::Meta));
        state.arm_one_shot(Modifier::Standard(StandardModifier::Meta));

        assert!(state.consume_one_shot(Modifier::Standard(StandardModifier::Meta)));
        assert!(state.is_active(Modifier::Standard(StandardModifier::Meta)));
    }
}
