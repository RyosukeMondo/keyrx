//! Modifier/lock bits shared across every device a [`super::DeviceState`]
//! belongs to.
//!
//! # Cross-device state sharing
//!
//! The DSL manual documents that custom modifiers and locks (`MD_XX`,
//! `LK_XX`) are ONE state shared by every input device: holding a layer key
//! on device A makes `when("MD_00")` mappings apply on device B too (e.g. a
//! foot pedal used as a layer key for a separate keyboard). Tap-hold pending
//! state and press→release output tracking, in contrast, MUST stay
//! per-device: a release on device B must never release device A's tracked
//! outputs, and a tap-hold key pending on device A must not be affected by
//! keystrokes on device B.
//!
//! [`SharedModifierState`] holds only the modifier/lock bits. Every
//! [`super::DeviceState`] created via [`super::DeviceState::new_sharing`]
//! points at the SAME `Arc<spin::Mutex<SharedModifierState>>`, so
//! `set_modifier`/`is_modifier_active`/`toggle_lock`/`is_lock_active` read
//! and write one shared instance while tap-hold and pressed-key tracking
//! remain private to each `DeviceState`. `DeviceState::new()` still creates
//! its own private (unshared) instance, so every other caller (tests,
//! `device_manager`, `processor`) is unaffected.
//!
//! An `Arc<Mutex<_>>` rather than the cheaper `Rc<RefCell<_>>` because the
//! daemon moves its whole `Remapper` onto a dedicated platform thread
//! (`std::thread::spawn`), which requires `Send`; `spin::Mutex` needs no OS
//! thread support so this also works unchanged on `wasm32-unknown-unknown`.
//! In practice the engine still runs on one thread at a time, so contention
//! never happens - this is a `Send` requirement, not real concurrency.

extern crate alloc;

use alloc::sync::Arc;

use bitvec::prelude::*;
use spin::Mutex;

use crate::config::{MAX_LOCK_ID, MAX_MODIFIER_ID, MODIFIER_COUNT};

/// Handle to a [`SharedModifierState`], cheap to clone (reference count
/// only).
pub type SharedState = Arc<Mutex<SharedModifierState>>;

/// The 255-bit modifier and lock vectors, shared by reference across the
/// devices routed through one [`crate::runtime::remapper::Remapper`].
pub struct SharedModifierState {
    modifiers: BitVec<u8, Lsb0>,
    locks: BitVec<u8, Lsb0>,
}

impl SharedModifierState {
    /// A fresh, all-clear shared state.
    pub fn new() -> Self {
        Self {
            modifiers: bitvec![u8, Lsb0; 0; MODIFIER_COUNT],
            locks: bitvec![u8, Lsb0; 0; MODIFIER_COUNT],
        }
    }

    /// Wraps a fresh [`SharedModifierState`] in a new handle.
    pub fn new_handle() -> SharedState {
        Arc::new(Mutex::new(Self::new()))
    }

    #[inline]
    fn valid(id: u8) -> bool {
        id as u16 <= MAX_MODIFIER_ID
    }

    pub fn set_modifier(&mut self, id: u8) -> bool {
        if !Self::valid(id) {
            return false;
        }
        self.modifiers.set(id as usize, true);
        true
    }

    pub fn clear_modifier(&mut self, id: u8) -> bool {
        if !Self::valid(id) {
            return false;
        }
        self.modifiers.set(id as usize, false);
        true
    }

    pub fn is_modifier_active(&self, id: u8) -> bool {
        Self::valid(id) && self.modifiers[id as usize]
    }

    pub fn toggle_lock(&mut self, id: u8) -> bool {
        if !Self::valid(id) {
            return false;
        }
        let current = self.locks[id as usize];
        self.locks.set(id as usize, !current);
        true
    }

    pub fn is_lock_active(&self, id: u8) -> bool {
        Self::valid(id) && self.locks[id as usize]
    }

    /// Highest modifier/lock id ever valid (`MAX_MODIFIER_ID` ==
    /// `MAX_LOCK_ID`, see [`crate::config::constants`]).
    pub const fn max_id() -> u8 {
        MAX_LOCK_ID as u8
    }
}

impl Default for SharedModifierState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modifiers_and_locks_are_independent_bit_spaces() {
        let mut s = SharedModifierState::new();
        assert!(s.set_modifier(3));
        assert!(s.is_modifier_active(3));
        assert!(!s.is_lock_active(3));

        assert!(s.toggle_lock(3));
        assert!(s.is_lock_active(3));

        assert!(s.clear_modifier(3));
        assert!(!s.is_modifier_active(3));
        assert!(s.is_lock_active(3));
    }

    #[test]
    fn out_of_range_ids_are_rejected() {
        let mut s = SharedModifierState::new();
        assert!(!s.set_modifier(255));
        assert!(!s.toggle_lock(255));
        assert!(!s.is_modifier_active(255));
        assert!(!s.is_lock_active(255));
    }

    #[test]
    fn two_handles_to_the_same_instance_see_each_others_writes() {
        let handle = SharedModifierState::new_handle();
        let other = Arc::clone(&handle);

        handle.lock().set_modifier(5);
        assert!(other.lock().is_modifier_active(5));
    }
}
