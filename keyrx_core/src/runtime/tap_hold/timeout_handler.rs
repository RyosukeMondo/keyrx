//! Timeout handling for tap-hold keys.
//!
//! This module provides the registry for tracking pending tap-hold keys and
//! checking for timeouts when the hold threshold is exceeded.

use super::state_machine::TapHoldState;
use crate::config::KeyCode;
use arrayvec::ArrayVec;

/// Default maximum concurrent tap-hold keys.
pub const DEFAULT_MAX_PENDING: usize = 32;

/// Result of a timeout check operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeoutResult {
    /// The key that timed out
    pub key: KeyCode,
    /// The hold modifier to activate
    pub hold_modifier: u8,
    /// The real key to press instead of a modifier, for key-hold tap-holds
    pub hold_key: Option<KeyCode>,
}

/// Registry for tracking multiple concurrent tap-hold keys.
///
/// Uses a fixed-size `ArrayVec` to avoid heap allocation while supporting
/// multiple simultaneous tap-hold key presses. Keys in `Pending` state are
/// tracked here for timeout checking.
///
/// # Capacity
///
/// The registry has a compile-time maximum capacity specified by the const
/// generic `N`. When full, new entries cannot be added. The default is 32
/// concurrent tap-hold keys, which is more than enough for typical usage.
///
/// # Performance
///
/// - Add: O(1) amortized
/// - Remove: O(n) linear scan
/// - Get/GetMut: O(n) linear scan
/// - Iteration: O(n) cache-friendly
///
/// # Example
///
/// ```rust
/// use keyrx_core::runtime::tap_hold::{PendingKeyRegistry, TapHoldState, TapHoldConfig, DEFAULT_MAX_PENDING};
/// use keyrx_core::config::KeyCode;
///
/// let mut registry: PendingKeyRegistry<DEFAULT_MAX_PENDING> = PendingKeyRegistry::new();
///
/// // Add a pending tap-hold state
/// let config = TapHoldConfig::new(KeyCode::Escape, 0, 200_000);
/// let mut state = TapHoldState::new(KeyCode::CapsLock, config);
/// state.transition_to_pending(1000);
///
/// assert!(registry.add(state));
/// assert_eq!(registry.len(), 1);
///
/// // Look up by key
/// assert!(registry.get(KeyCode::CapsLock).is_some());
///
/// // Remove when done
/// assert!(registry.remove(KeyCode::CapsLock));
/// assert!(registry.is_empty());
/// ```
#[derive(Debug, Clone)]
pub struct PendingKeyRegistry<const N: usize = DEFAULT_MAX_PENDING> {
    /// Storage for pending tap-hold states
    entries: ArrayVec<TapHoldState, N>,
}

impl<const N: usize> Default for PendingKeyRegistry<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> PendingKeyRegistry<N> {
    /// Creates a new empty registry.
    pub const fn new() -> Self {
        Self {
            entries: ArrayVec::new_const(),
        }
    }

    /// Returns the number of pending tap-hold keys.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns true if the registry is at capacity.
    pub fn is_full(&self) -> bool {
        self.entries.is_full()
    }

    /// Returns the maximum capacity of the registry.
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Adds a tap-hold state to the registry.
    ///
    /// # Returns
    ///
    /// - `true` if the state was added
    /// - `false` if the registry is full or the key already exists
    ///
    /// # Note
    ///
    /// If a state for the same key already exists, the add fails.
    /// Use `remove` first to replace an existing entry.
    pub fn add(&mut self, state: TapHoldState) -> bool {
        // Check if key already exists
        if self.contains(state.key()) {
            return false;
        }

        // Try to add (fails if full)
        self.entries.try_push(state).is_ok()
    }

    /// Removes a tap-hold state by key.
    ///
    /// # Returns
    ///
    /// - `true` if the state was found and removed
    /// - `false` if no state for this key exists
    pub fn remove(&mut self, key: KeyCode) -> bool {
        if let Some(idx) = self.find_index(key) {
            self.entries.swap_remove(idx);
            true
        } else {
            false
        }
    }

    /// Checks if a key is in the registry.
    pub fn contains(&self, key: KeyCode) -> bool {
        self.entries.iter().any(|s| s.key() == key)
    }

    /// Gets an immutable reference to a tap-hold state by key.
    pub fn get(&self, key: KeyCode) -> Option<&TapHoldState> {
        self.entries.iter().find(|s| s.key() == key)
    }

    /// Gets a mutable reference to a tap-hold state by key.
    pub fn get_mut(&mut self, key: KeyCode) -> Option<&mut TapHoldState> {
        self.entries.iter_mut().find(|s| s.key() == key)
    }

    /// Returns an iterator over all pending states.
    pub fn iter(&self) -> impl Iterator<Item = &TapHoldState> {
        self.entries.iter()
    }

    /// Returns a mutable iterator over all pending states.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut TapHoldState> {
        self.entries.iter_mut()
    }

    /// Clears all entries from the registry.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Checks for timeouts and returns keys that exceeded their thresholds.
    ///
    /// This method checks all pending keys against the current time and:
    /// 1. Returns information about keys that exceeded their threshold
    /// 2. Transitions those keys to Hold state
    ///
    /// # Arguments
    ///
    /// * `current_time` - Current timestamp in microseconds
    ///
    /// # Returns
    ///
    /// An `ArrayVec` of `TimeoutResult` for keys that timed out.
    /// Maximum size matches the registry capacity.
    ///
    /// # Example
    ///
    /// ```rust
    /// use keyrx_core::runtime::tap_hold::{PendingKeyRegistry, TapHoldState, TapHoldConfig, DEFAULT_MAX_PENDING};
    /// use keyrx_core::config::KeyCode;
    ///
    /// let mut registry: PendingKeyRegistry<DEFAULT_MAX_PENDING> = PendingKeyRegistry::new();
    ///
    /// // Add a pending state
    /// let config = TapHoldConfig::new(KeyCode::Escape, 0, 200_000); // 200ms threshold
    /// let mut state = TapHoldState::new(KeyCode::CapsLock, config);
    /// state.transition_to_pending(0); // pressed at time 0
    /// registry.add(state);
    ///
    /// // Before threshold - no timeouts
    /// let timeouts = registry.check_timeouts(100_000); // 100ms
    /// assert!(timeouts.is_empty());
    ///
    /// // After threshold - CapsLock times out
    /// let timeouts = registry.check_timeouts(300_000); // 300ms
    /// assert_eq!(timeouts.len(), 1);
    /// assert_eq!(timeouts[0].key, KeyCode::CapsLock);
    /// assert_eq!(timeouts[0].hold_modifier, 0);
    /// ```
    pub fn check_timeouts(&mut self, current_time: u64) -> ArrayVec<TimeoutResult, N> {
        let mut results = ArrayVec::new();

        for state in self.entries.iter_mut() {
            // Only check Pending states
            if !state.phase().is_pending() {
                continue;
            }

            // Check if threshold exceeded
            if state.is_threshold_exceeded(current_time) {
                // Transition to Hold
                state.transition_to_hold();

                // Record the timeout
                // This won't fail since results capacity matches entries capacity
                let _ = results.try_push(TimeoutResult {
                    key: state.key(),
                    hold_modifier: state.hold_modifier(),
                    hold_key: state.hold_key(),
                });
            }
        }

        results
    }

    /// Finds all keys in Pending state.
    ///
    /// Useful for checking if any keys might need permissive hold activation.
    pub fn pending_keys(&self) -> impl Iterator<Item = &TapHoldState> {
        self.entries.iter().filter(|s| s.phase().is_pending())
    }

    /// Triggers permissive hold for all pending keys.
    ///
    /// When another key is pressed while tap-hold keys are pending,
    /// this method transitions all pending keys to Hold state immediately.
    ///
    /// # Returns
    ///
    /// Information about all keys that were transitioned to Hold.
    pub fn trigger_permissive_hold(&mut self) -> ArrayVec<TimeoutResult, N> {
        let mut results = ArrayVec::new();

        for state in self.entries.iter_mut() {
            if state.phase().is_pending() {
                state.transition_to_hold();

                let _ = results.try_push(TimeoutResult {
                    key: state.key(),
                    hold_modifier: state.hold_modifier(),
                    hold_key: state.hold_key(),
                });
            }
        }

        results
    }

    // --- Private helpers ---

    /// Finds the index of a state by key.
    fn find_index(&self, key: KeyCode) -> Option<usize> {
        self.entries.iter().position(|s| s.key() == key)
    }
}
