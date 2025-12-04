//! Fallback engine for graceful degradation.
//!
//! Provides a minimal passthrough engine that activates when the main engine
//! encounters critical failures. Ensures keyboard remains functional even
//! during catastrophic errors.

use crate::engine::{InputEvent, OutputAction};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use tracing::{error, info, warn};

/// Reason why the fallback engine was activated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FallbackReason {
    /// Main engine panicked during event processing.
    Panic(String),
    /// Circuit breaker opened due to repeated failures.
    CircuitBreakerOpen,
    /// Critical error in driver or core component.
    CriticalError(String),
    /// Manual activation for testing or maintenance.
    Manual,
}

impl std::fmt::Display for FallbackReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Panic(msg) => write!(f, "Panic: {}", msg),
            Self::CircuitBreakerOpen => write!(f, "Circuit breaker open"),
            Self::CriticalError(msg) => write!(f, "Critical error: {}", msg),
            Self::Manual => write!(f, "Manual activation"),
        }
    }
}

/// Minimal fallback engine that passes all input through unchanged.
///
/// The FallbackEngine provides a zero-dependency safety net that ensures
/// keyboard input continues working even when the main engine fails. It
/// implements a simple passthrough policy with no remapping, no state
/// tracking, and no external dependencies.
///
/// # Design Principles
///
/// 1. **Minimal Dependencies**: No script runtime, state store, or metrics
/// 2. **Always Works**: Cannot fail - all inputs pass through unchanged
/// 3. **Observable**: Tracks activation reason for debugging
/// 4. **Thread-Safe**: Can be shared across threads
///
/// # Usage
///
/// ```ignore
/// let fallback = FallbackEngine::new();
///
/// // Activate when main engine fails
/// fallback.activate(FallbackReason::Panic("main engine crashed".into()));
///
/// // Process events (always returns PassThrough)
/// let event = InputEvent::key_down(KeyCode::A, 1000);
/// let action = fallback.process_event(&event);
/// assert_eq!(action, OutputAction::PassThrough);
///
/// // Deactivate when main engine recovers
/// fallback.deactivate();
/// ```
pub struct FallbackEngine {
    /// Whether fallback mode is currently active.
    active: Arc<AtomicBool>,
    /// Reason for current or last activation.
    reason: Arc<RwLock<Option<FallbackReason>>>,
    /// Count of events processed in fallback mode.
    event_count: Arc<std::sync::atomic::AtomicU64>,
}

impl FallbackEngine {
    /// Create a new fallback engine in inactive state.
    pub fn new() -> Self {
        Self {
            active: Arc::new(AtomicBool::new(false)),
            reason: Arc::new(RwLock::new(None)),
            event_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    /// Activate fallback mode with the given reason.
    ///
    /// Logs a warning and records the activation reason. All subsequent
    /// events will be passed through unchanged until deactivation.
    pub fn activate(&self, reason: FallbackReason) {
        if self.active.swap(true, Ordering::SeqCst) {
            warn!(
                service = "keyrx",
                event = "fallback_already_active",
                component = "fallback_engine",
                reason = %reason,
                "Fallback engine already active, updating reason"
            );
        } else {
            error!(
                service = "keyrx",
                event = "fallback_activated",
                component = "fallback_engine",
                reason = %reason,
                "Fallback engine activated - entering passthrough mode"
            );
        }

        if let Ok(mut guard) = self.reason.write() {
            *guard = Some(reason);
        }
    }

    /// Deactivate fallback mode and return to normal operation.
    ///
    /// Logs the number of events processed in fallback mode.
    pub fn deactivate(&self) {
        if !self.active.swap(false, Ordering::SeqCst) {
            warn!(
                service = "keyrx",
                event = "fallback_not_active",
                component = "fallback_engine",
                "Attempted to deactivate inactive fallback engine"
            );
            return;
        }

        let count = self.event_count.swap(0, Ordering::SeqCst);
        let reason = self.reason.read().ok().and_then(|r| r.clone());

        info!(
            service = "keyrx",
            event = "fallback_deactivated",
            component = "fallback_engine",
            events_processed = count,
            reason = ?reason,
            "Fallback engine deactivated - returning to normal operation"
        );

        if let Ok(mut guard) = self.reason.write() {
            *guard = None;
        }
    }

    /// Check if fallback mode is currently active.
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }

    /// Get the current activation reason, if active.
    pub fn reason(&self) -> Option<FallbackReason> {
        self.reason.read().ok().and_then(|r| r.clone())
    }

    /// Get the number of events processed since activation.
    pub fn event_count(&self) -> u64 {
        self.event_count.load(Ordering::SeqCst)
    }

    /// Process an input event in fallback mode.
    ///
    /// Always returns `OutputAction::PassThrough` to ensure input
    /// continues working. Increments the event counter.
    ///
    /// # Panics
    ///
    /// Never panics - guaranteed to return successfully.
    pub fn process_event(&self, event: &InputEvent) -> OutputAction {
        if self.is_active() {
            self.event_count.fetch_add(1, Ordering::SeqCst);
        }

        // Log first few events in fallback mode for debugging
        let count = self.event_count.load(Ordering::SeqCst);
        if count <= 3 {
            info!(
                service = "keyrx",
                event = "fallback_passthrough",
                component = "fallback_engine",
                key = ?event.key,
                pressed = event.pressed,
                count = count,
                "Processing event in fallback mode"
            );
        }

        OutputAction::PassThrough
    }

    /// Reset the event counter without deactivating.
    ///
    /// Useful for testing or when transitioning between failure states.
    pub fn reset_counter(&self) {
        self.event_count.store(0, Ordering::SeqCst);
    }
}

impl Default for FallbackEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for FallbackEngine {
    fn clone(&self) -> Self {
        Self {
            active: Arc::clone(&self.active),
            reason: Arc::clone(&self.reason),
            event_count: Arc::clone(&self.event_count),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::KeyCode;

    #[test]
    fn new_engine_is_inactive() {
        let engine = FallbackEngine::new();
        assert!(!engine.is_active());
        assert_eq!(engine.reason(), None);
        assert_eq!(engine.event_count(), 0);
    }

    #[test]
    fn activate_sets_reason() {
        let engine = FallbackEngine::new();
        let reason = FallbackReason::Panic("test panic".into());

        engine.activate(reason.clone());

        assert!(engine.is_active());
        assert_eq!(engine.reason(), Some(reason));
    }

    #[test]
    fn deactivate_clears_state() {
        let engine = FallbackEngine::new();
        engine.activate(FallbackReason::Manual);

        // Process some events
        let event = InputEvent::key_down(KeyCode::A, 1000);
        engine.process_event(&event);
        engine.process_event(&event);

        assert!(engine.event_count() > 0);

        engine.deactivate();

        assert!(!engine.is_active());
        assert_eq!(engine.reason(), None);
        assert_eq!(engine.event_count(), 0);
    }

    #[test]
    fn process_event_always_passes_through() {
        let engine = FallbackEngine::new();
        engine.activate(FallbackReason::Manual);

        let events = vec![
            InputEvent::key_down(KeyCode::A, 1000),
            InputEvent::key_up(KeyCode::A, 1100),
            InputEvent::key_down(KeyCode::Escape, 1200),
        ];

        for event in events {
            let action = engine.process_event(&event);
            assert_eq!(action, OutputAction::PassThrough);
        }
    }

    #[test]
    fn process_event_increments_counter() {
        let engine = FallbackEngine::new();
        engine.activate(FallbackReason::Manual);

        assert_eq!(engine.event_count(), 0);

        let event = InputEvent::key_down(KeyCode::A, 1000);
        engine.process_event(&event);
        assert_eq!(engine.event_count(), 1);

        engine.process_event(&event);
        assert_eq!(engine.event_count(), 2);
    }

    #[test]
    fn process_event_when_inactive_returns_passthrough() {
        let engine = FallbackEngine::new();
        // Not activated

        let event = InputEvent::key_down(KeyCode::A, 1000);
        let action = engine.process_event(&event);

        assert_eq!(action, OutputAction::PassThrough);
        // Counter should not increment when inactive
        assert_eq!(engine.event_count(), 0);
    }

    #[test]
    fn clone_shares_state() {
        let engine1 = FallbackEngine::new();
        engine1.activate(FallbackReason::Manual);

        let engine2 = engine1.clone();

        assert!(engine2.is_active());
        assert_eq!(engine2.reason(), engine1.reason());

        let event = InputEvent::key_down(KeyCode::A, 1000);
        engine2.process_event(&event);

        // Counter should be shared
        assert_eq!(engine1.event_count(), 1);
        assert_eq!(engine2.event_count(), 1);
    }

    #[test]
    fn multiple_activations_update_reason() {
        let engine = FallbackEngine::new();

        engine.activate(FallbackReason::Manual);
        assert_eq!(engine.reason(), Some(FallbackReason::Manual));

        let panic_reason = FallbackReason::Panic("error".into());
        engine.activate(panic_reason.clone());
        assert_eq!(engine.reason(), Some(panic_reason));
    }

    #[test]
    fn reset_counter_preserves_active_state() {
        let engine = FallbackEngine::new();
        engine.activate(FallbackReason::Manual);

        let event = InputEvent::key_down(KeyCode::A, 1000);
        engine.process_event(&event);
        engine.process_event(&event);

        assert_eq!(engine.event_count(), 2);

        engine.reset_counter();

        assert!(engine.is_active());
        assert_eq!(engine.event_count(), 0);
    }

    #[test]
    fn fallback_reason_display() {
        assert_eq!(
            FallbackReason::Panic("test".into()).to_string(),
            "Panic: test"
        );
        assert_eq!(
            FallbackReason::CircuitBreakerOpen.to_string(),
            "Circuit breaker open"
        );
        assert_eq!(
            FallbackReason::CriticalError("err".into()).to_string(),
            "Critical error: err"
        );
        assert_eq!(FallbackReason::Manual.to_string(), "Manual activation");
    }
}
