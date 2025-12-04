//! Common session state for recording and replay.
//!
//! This module provides a shared [`SessionState`] type that encapsulates
//! common state management logic used by both recording and replay sessions.

use std::time::Instant;

/// Session status tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStatus {
    /// Session not started.
    Idle,
    /// Session actively running.
    Active,
    /// Session paused (for step-by-step control).
    Paused,
    /// Session completed.
    Completed,
}

/// Common session state for recording and replay operations.
///
/// This type encapsulates shared functionality between recording
/// and replay sessions, reducing code duplication and providing
/// consistent state management.
#[derive(Debug)]
pub struct SessionState {
    /// Current session status.
    status: SessionStatus,
    /// Timestamp when session started (when status became Active).
    start_time: Option<Instant>,
}

impl Default for SessionState {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionState {
    /// Create a new idle session state.
    pub fn new() -> Self {
        Self {
            status: SessionStatus::Idle,
            start_time: None,
        }
    }

    /// Start the session (transition to Active).
    pub fn start(&mut self) {
        if self.status == SessionStatus::Idle {
            self.status = SessionStatus::Active;
            self.start_time = Some(Instant::now());
        }
    }

    /// Pause the session.
    pub fn pause(&mut self) {
        if self.status == SessionStatus::Active {
            self.status = SessionStatus::Paused;
        }
    }

    /// Resume the session from paused state.
    pub fn resume(&mut self) {
        if self.status == SessionStatus::Paused {
            self.status = SessionStatus::Active;
        }
    }

    /// Mark the session as completed.
    pub fn complete(&mut self) {
        self.status = SessionStatus::Completed;
    }

    /// Stop the session and reset to idle state.
    pub fn stop(&mut self) {
        self.status = SessionStatus::Idle;
        self.start_time = None;
    }

    /// Check if the session is active.
    pub fn is_active(&self) -> bool {
        self.status == SessionStatus::Active
    }

    /// Check if the session is idle.
    pub fn is_idle(&self) -> bool {
        self.status == SessionStatus::Idle
    }

    /// Check if the session is paused.
    pub fn is_paused(&self) -> bool {
        self.status == SessionStatus::Paused
    }

    /// Check if the session is completed.
    pub fn is_completed(&self) -> bool {
        self.status == SessionStatus::Completed
    }

    /// Get the current status.
    pub fn status(&self) -> SessionStatus {
        self.status
    }

    /// Get the start time if session has been started.
    pub fn start_time(&self) -> Option<Instant> {
        self.start_time
    }

    /// Get elapsed time since session start in microseconds.
    pub fn elapsed_us(&self) -> u64 {
        self.start_time
            .map(|t| t.elapsed().as_micros() as u64)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_session_is_idle() {
        let state = SessionState::new();
        assert_eq!(state.status(), SessionStatus::Idle);
        assert!(state.is_idle());
        assert!(!state.is_active());
        assert!(state.start_time().is_none());
    }

    #[test]
    fn start_transitions_to_active() {
        let mut state = SessionState::new();
        state.start();
        assert_eq!(state.status(), SessionStatus::Active);
        assert!(state.is_active());
        assert!(!state.is_idle());
        assert!(state.start_time().is_some());
    }

    #[test]
    fn pause_and_resume() {
        let mut state = SessionState::new();
        state.start();
        state.pause();
        assert_eq!(state.status(), SessionStatus::Paused);
        assert!(state.is_paused());
        assert!(!state.is_active());

        state.resume();
        assert_eq!(state.status(), SessionStatus::Active);
        assert!(state.is_active());
        assert!(!state.is_paused());
    }

    #[test]
    fn complete_marks_completed() {
        let mut state = SessionState::new();
        state.start();
        state.complete();
        assert_eq!(state.status(), SessionStatus::Completed);
        assert!(state.is_completed());
        assert!(!state.is_active());
    }

    #[test]
    fn stop_resets_to_idle() {
        let mut state = SessionState::new();
        state.start();
        assert!(state.start_time().is_some());

        state.stop();
        assert_eq!(state.status(), SessionStatus::Idle);
        assert!(state.is_idle());
        assert!(state.start_time().is_none());
    }

    #[test]
    fn elapsed_us_returns_zero_when_not_started() {
        let state = SessionState::new();
        assert_eq!(state.elapsed_us(), 0);
    }

    #[test]
    fn elapsed_us_returns_time_after_start() {
        let mut state = SessionState::new();
        state.start();
        std::thread::sleep(std::time::Duration::from_micros(100));
        assert!(state.elapsed_us() >= 100);
    }

    #[test]
    fn pause_from_idle_has_no_effect() {
        let mut state = SessionState::new();
        state.pause();
        assert_eq!(state.status(), SessionStatus::Idle);
    }

    #[test]
    fn resume_from_idle_has_no_effect() {
        let mut state = SessionState::new();
        state.resume();
        assert_eq!(state.status(), SessionStatus::Idle);
    }

    #[test]
    fn start_multiple_times_is_idempotent() {
        let mut state = SessionState::new();
        state.start();
        let first_start = state.start_time();

        state.start();
        let second_start = state.start_time();

        // Second start should not change the start time
        assert_eq!(first_start, second_start);
    }
}
