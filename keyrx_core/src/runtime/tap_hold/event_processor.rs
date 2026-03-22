//! Tap-hold event processing.
//!
//! This module contains the [`TapHoldProcessor`] which manages the state machine
//! for all tap-hold keys and processes input events to determine tap vs hold outcomes.

use super::state_machine::TapHoldState;
use super::timeout_handler::{PendingKeyRegistry, DEFAULT_MAX_PENDING};
use super::types::TapHoldPhase;
use super::types::{TapHoldConfig, TapHoldOutput};
use crate::config::KeyCode;
use arrayvec::ArrayVec;

/// Logs a tap-hold event in debug builds.
#[cfg(debug_assertions)]
macro_rules! log_event {
    ($($arg:tt)*) => {
        log::trace!($($arg)*);
    };
}

/// No-op in release builds.
#[cfg(not(debug_assertions))]
macro_rules! log_event {
    ($($arg:tt)*) => {};
}

/// Maximum output events from a single tap-hold event processing.
///
/// Tap: 2 events (press + release of tap key)
/// Hold activation: 0 events (modifier state change only)
/// Hold deactivation: 0 events (modifier state change only)
pub const MAX_OUTPUT_EVENTS: usize = 4;

/// Tap-hold event processor.
///
/// Manages the state machine for all tap-hold keys and processes input events.
/// This processor handles the core logic of determining whether a key press
/// should result in a tap (quick release) or hold (sustained press) action.
///
/// # Type Parameters
///
/// * `N` - Maximum number of concurrent tap-hold keys (default: 32)
///
/// # Example
///
/// ```rust
/// use keyrx_core::runtime::tap_hold::{TapHoldProcessor, TapHoldConfig, TapHoldOutput, DEFAULT_MAX_PENDING};
/// use keyrx_core::config::KeyCode;
///
/// let mut processor: TapHoldProcessor<DEFAULT_MAX_PENDING> = TapHoldProcessor::new();
///
/// // Register a tap-hold configuration: CapsLock = tap:Escape, hold:modifier 0
/// let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
/// processor.register_tap_hold(KeyCode::CapsLock, config);
///
/// // Process a press event at time 0
/// let outputs = processor.process_press(KeyCode::CapsLock, 0);
/// assert!(outputs.is_empty()); // No immediate output, waiting for release or timeout
///
/// // Quick release at 100ms (under 200ms threshold) - this is a tap
/// let outputs = processor.process_release(KeyCode::CapsLock, 100_000);
/// assert_eq!(outputs.len(), 2); // Press and release of Escape
/// ```
#[derive(Debug, Clone)]
pub struct TapHoldProcessor<const N: usize = DEFAULT_MAX_PENDING> {
    /// Registry of pending tap-hold states
    pending: PendingKeyRegistry<N>,
    /// Registered tap-hold configurations (key -> config)
    /// Using ArrayVec for no_std compatibility
    configs: ArrayVec<(KeyCode, TapHoldConfig), N>,
}

impl<const N: usize> Default for TapHoldProcessor<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> TapHoldProcessor<N> {
    /// Creates a new tap-hold processor.
    pub const fn new() -> Self {
        Self {
            pending: PendingKeyRegistry::new(),
            configs: ArrayVec::new_const(),
        }
    }

    /// Registers a tap-hold configuration for a key.
    ///
    /// # Arguments
    ///
    /// * `key` - The physical key that triggers this tap-hold
    /// * `config` - Configuration for tap/hold behavior
    ///
    /// # Returns
    ///
    /// `true` if registration succeeded, `false` if at capacity or key already registered
    pub fn register_tap_hold(&mut self, key: KeyCode, config: TapHoldConfig) -> bool {
        // Check if already registered
        if self.get_config(key).is_some() {
            return false;
        }

        // Try to add
        self.configs.try_push((key, config)).is_ok()
    }

    /// Gets the tap-hold configuration for a key.
    pub fn get_config(&self, key: KeyCode) -> Option<&TapHoldConfig> {
        self.configs.iter().find(|(k, _)| *k == key).map(|(_, c)| c)
    }

    /// Checks if a key is a registered tap-hold key.
    pub fn is_tap_hold_key(&self, key: KeyCode) -> bool {
        self.get_config(key).is_some()
    }

    /// Checks if a key is currently in pending state.
    pub fn is_pending(&self, key: KeyCode) -> bool {
        self.pending
            .get(key)
            .map(|s| s.phase().is_pending())
            .unwrap_or(false)
    }

    /// Checks if a key is currently in hold state.
    pub fn is_hold(&self, key: KeyCode) -> bool {
        self.pending
            .get(key)
            .map(|s| s.phase().is_hold())
            .unwrap_or(false)
    }

    /// Returns the number of currently pending tap-hold keys.
    pub fn pending_count(&self) -> usize {
        self.pending.pending_keys().count()
    }

    /// Returns the number of currently active hold states.
    pub fn hold_count(&self) -> usize {
        self.pending.iter().filter(|s| s.phase().is_hold()).count()
    }

    /// Processes a key press event.
    ///
    /// If the key is a registered tap-hold key:
    /// - Transitions from Idle to Pending
    /// - Records the press timestamp
    /// - No output events are generated (waiting for release or timeout)
    ///
    /// # Arguments
    ///
    /// * `key` - The key that was pressed
    /// * `timestamp_us` - The press timestamp in microseconds
    ///
    /// # Returns
    ///
    /// Output events to be processed by the caller (usually empty for press).
    pub fn process_press(
        &mut self,
        key: KeyCode,
        timestamp_us: u64,
    ) -> ArrayVec<TapHoldOutput, MAX_OUTPUT_EVENTS> {
        let outputs = ArrayVec::new();

        // Check if this is a registered tap-hold key
        let Some(config) = self.get_config(key).copied() else {
            return outputs;
        };

        // Check if already in pending registry (shouldn't happen, but handle gracefully)
        if self.pending.contains(key) {
            log_event!(
                "tap-hold: {:?} press ignored - already in pending registry",
                key
            );
            return outputs;
        }

        log_event!(
            "tap-hold: {:?} press at {}us, threshold={}us, tap={:?}, hold_mod={}",
            key,
            timestamp_us,
            config.threshold_us(),
            config.tap_key(),
            config.hold_modifier()
        );

        // Create new pending state and add to registry
        let mut state = TapHoldState::new(key, config);
        state.transition_to_pending(timestamp_us);

        if !self.pending.add(state) {
            // Registry full - caller should handle this as passthrough
            // No outputs, key will be handled as normal key
            log_event!("tap-hold: {:?} registry full, treating as normal key", key);
        }

        outputs
    }

    /// Processes a key release event.
    ///
    /// Determines whether the release constitutes a tap or hold release:
    /// - If Pending and elapsed < threshold: TAP (emit tap key press + release)
    /// - If Pending and elapsed >= threshold: Delayed HOLD (activate modifier, then immediately deactivate)
    /// - If Hold: Deactivate the hold modifier
    ///
    /// # Arguments
    ///
    /// * `key` - The key that was released
    /// * `timestamp_us` - The release timestamp in microseconds
    ///
    /// # Returns
    ///
    /// Output events to be processed by the caller.
    pub fn process_release(
        &mut self,
        key: KeyCode,
        timestamp_us: u64,
    ) -> ArrayVec<TapHoldOutput, MAX_OUTPUT_EVENTS> {
        let mut outputs = ArrayVec::new();

        // Get the pending state
        let Some(state) = self.pending.get(key).copied() else {
            return outputs;
        };

        #[cfg(debug_assertions)]
        let elapsed = state.elapsed(timestamp_us);

        match state.phase() {
            TapHoldPhase::Idle => {
                // Shouldn't be in registry if Idle, but handle gracefully
                log_event!(
                    "tap-hold: {:?} release ignored - was in Idle phase (unexpected)",
                    key
                );
                self.pending.remove(key);
            }
            TapHoldPhase::Pending => {
                // Determine tap vs hold based on elapsed time
                if state.is_threshold_exceeded(timestamp_us) {
                    // Threshold exceeded during pending - this is a hold that wasn't detected
                    // Activate modifier briefly then deactivate (user held key but we didn't catch timeout)
                    log_event!(
                        "tap-hold: {:?} release after {}us (threshold {}us) -> late HOLD (missed timeout)",
                        key,
                        elapsed,
                        state.threshold_us()
                    );
                    let _ =
                        outputs.try_push(TapHoldOutput::activate_modifier(state.hold_modifier()));
                    let _ =
                        outputs.try_push(TapHoldOutput::deactivate_modifier(state.hold_modifier()));
                } else if state.config().is_tap_suppressed() {
                    // hold_only: tap does nothing
                    log_event!(
                        "tap-hold: {:?} release after {}us -> SUPPRESSED (hold_only)",
                        key,
                        elapsed
                    );
                } else {
                    // Quick release - this is a TAP
                    // Emit press and release of the tap key
                    log_event!(
                        "tap-hold: {:?} release after {}us (threshold {}us) -> TAP, emit {:?}",
                        key,
                        elapsed,
                        state.threshold_us(),
                        state.tap_key()
                    );
                    let _ =
                        outputs.try_push(TapHoldOutput::key_press(state.tap_key(), timestamp_us));
                    let _ =
                        outputs.try_push(TapHoldOutput::key_release(state.tap_key(), timestamp_us));
                }
                self.pending.remove(key);
            }
            TapHoldPhase::Hold => {
                // Release from hold state - deactivate the modifier
                log_event!(
                    "tap-hold: {:?} release from Hold after {}us, deactivating modifier {}",
                    key,
                    elapsed,
                    state.hold_modifier()
                );
                let _ = outputs.try_push(TapHoldOutput::deactivate_modifier(state.hold_modifier()));
                self.pending.remove(key);
            }
        }

        outputs
    }

    /// Checks for timeout-triggered hold activations.
    ///
    /// This should be called periodically (e.g., every 10ms or on each event)
    /// to detect when pending keys have exceeded their hold threshold.
    ///
    /// # Arguments
    ///
    /// * `current_time` - Current timestamp in microseconds
    ///
    /// # Returns
    ///
    /// Output events for keys that transitioned to hold state.
    pub fn check_timeouts(
        &mut self,
        current_time: u64,
    ) -> ArrayVec<TapHoldOutput, MAX_OUTPUT_EVENTS> {
        let mut outputs = ArrayVec::new();

        // Use the registry's check_timeouts which transitions states internally
        let timeouts = self.pending.check_timeouts(current_time);

        for timeout in timeouts {
            log_event!(
                "tap-hold: {:?} timeout at {}us -> HOLD, activating modifier {}",
                timeout.key,
                current_time,
                timeout.hold_modifier
            );
            let _ = outputs.try_push(TapHoldOutput::activate_modifier(timeout.hold_modifier));
        }

        outputs
    }

    /// Processes an "other key" press event for Permissive Hold.
    ///
    /// When any key that is NOT a currently pending tap-hold key is pressed,
    /// this method should be called BEFORE processing that key. It will:
    /// 1. Check if any tap-hold keys are in Pending state
    /// 2. Immediately transition them to Hold state
    /// 3. Return modifier activation events
    ///
    /// This implements "Permissive Hold" behavior: if you press CapsLock (tap-hold)
    /// and then press 'A' before the hold threshold, CapsLock immediately becomes
    /// Ctrl so that 'A' is processed as Ctrl+A.
    ///
    /// # Arguments
    ///
    /// * `key` - The key that was pressed (the interrupting key)
    ///
    /// # Returns
    ///
    /// Output events for modifiers that were activated. The caller should apply
    /// these BEFORE processing the interrupting key.
    ///
    /// # Example
    ///
    /// ```rust
    /// use keyrx_core::runtime::tap_hold::{TapHoldProcessor, TapHoldConfig, TapHoldOutput, DEFAULT_MAX_PENDING};
    /// use keyrx_core::config::KeyCode;
    ///
    /// let mut processor: TapHoldProcessor<DEFAULT_MAX_PENDING> = TapHoldProcessor::new();
    ///
    /// // Register CapsLock as tap=Escape, hold=Ctrl (modifier 0)
    /// let config = TapHoldConfig::from_ms(KeyCode::Escape, 0, 200);
    /// processor.register_tap_hold(KeyCode::CapsLock, config);
    ///
    /// // Press CapsLock at t=0 (enters Pending state)
    /// processor.process_press(KeyCode::CapsLock, 0);
    ///
    /// // User types 'A' at t=50ms (before 200ms threshold)
    /// // This should trigger permissive hold
    /// let outputs = processor.process_other_key_press(KeyCode::A);
    ///
    /// // CapsLock's modifier should be activated
    /// assert_eq!(outputs.len(), 1);
    /// match outputs[0] {
    ///     TapHoldOutput::ActivateModifier { modifier_id } => assert_eq!(modifier_id, 0),
    ///     _ => panic!("Expected modifier activation"),
    /// }
    ///
    /// // Now CapsLock is in Hold state
    /// assert!(processor.is_hold(KeyCode::CapsLock));
    /// ```
    pub fn process_other_key_press(
        &mut self,
        key: KeyCode,
    ) -> ArrayVec<TapHoldOutput, MAX_OUTPUT_EVENTS> {
        let mut outputs = ArrayVec::new();

        // If the pressed key is itself a pending tap-hold key, don't trigger permissive hold
        // (This handles the case where user presses another tap-hold key)
        if self.pending.contains(key) {
            return outputs;
        }

        // Trigger permissive hold for all pending keys
        let results = self.pending.trigger_permissive_hold();

        for result in results {
            log_event!(
                "tap-hold: {:?} permissive hold triggered by {:?}, activating modifier {}",
                result.key,
                key,
                result.hold_modifier
            );
            let _ = outputs.try_push(TapHoldOutput::activate_modifier(result.hold_modifier));
        }

        outputs
    }

    /// Checks if there are any keys in pending state.
    ///
    /// Useful for determining whether permissive hold logic should be invoked.
    pub fn has_pending_keys(&self) -> bool {
        self.pending_count() > 0
    }

    /// Clears all pending states.
    ///
    /// This is useful for error recovery or when reloading configuration.
    /// Note: This does NOT emit deactivation events. Caller is responsible
    /// for handling any active modifiers.
    pub fn clear(&mut self) {
        self.pending.clear();
    }

    /// Clears all configurations and pending states.
    pub fn reset(&mut self) {
        self.pending.clear();
        self.configs.clear();
    }
}
