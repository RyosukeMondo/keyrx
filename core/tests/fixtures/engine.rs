#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::print_stdout, clippy::print_stderr, clippy::field_reassign_with_default, clippy::useless_conversion, clippy::assertions_on_constants, clippy::manual_div_ceil, clippy::manual_strip, clippy::len_zero, clippy::redundant_closure, clippy::manual_range_contains, clippy::default_constructed_unit_structs, clippy::clone_on_copy, clippy::io_other_error, clippy::bool_assert_comparison, clippy::approx_constant, clippy::let_unit_value, clippy::while_let_on_iterator, clippy::await_holding_lock, clippy::unnecessary_cast, clippy::drop_non_drop, clippy::needless_range_loop, unused_imports, unused_variables, dead_code, unsafe_code, clippy::collapsible_if, clippy::bool_comparison, unexpected_cfgs)]
//! TestEngine fixture for simplified engine testing with mock dependencies.
//!
//! This fixture provides a convenient wrapper around the Engine with pre-configured
//! mock implementations of InputSource, ScriptRuntime, and StateStore. It simplifies
//! common test patterns and reduces boilerplate when testing engine behavior.
//!
//! ## Features
//!
//! - Pre-configured mock dependencies (MockInput, MockRuntime, MockState)
//! - Fluent builder API for configuring scripts and remappings
//! - Helper methods for common test operations
//! - Automatic cleanup (mocks are dropped when TestEngine drops)
//!
//! ## Usage
//!
//! ```rust
//! use fixtures::engine::TestEngine;
//! use keyrx_core::engine::{KeyCode, InputEvent, OutputAction};
//!
//! // Create engine with a simple remap
//! let mut engine = TestEngine::new()
//!     .with_remap(KeyCode::A, KeyCode::B);
//!
//! // Process an event
//! let event = InputEvent::key_down(KeyCode::A, 0);
//! let output = engine.process(&event);
//! assert_eq!(output, OutputAction::KeyDown(KeyCode::B));
//! ```

use keyrx_core::engine::{Engine, InputEvent, KeyCode, OutputAction};
use keyrx_core::mocks::{MockInput, MockRuntime, MockState};
use keyrx_core::metrics;

/// Wrapper for testing the Engine with mock dependencies.
///
/// TestEngine provides a simplified API for creating and configuring an Engine
/// instance with MockInput, MockRuntime, and MockState. It reduces test setup
/// boilerplate and provides a fluent interface for common configuration patterns.
pub struct TestEngine {
    engine: Engine<MockInput, MockRuntime, MockState>,
}

impl TestEngine {
    /// Create a new TestEngine with default mock dependencies.
    ///
    /// The engine starts with:
    /// - MockInput (no pre-configured events)
    /// - MockRuntime (empty registry, all hooks succeed)
    /// - MockState (empty state)
    /// - Noop metrics collector
    ///
    /// # Example
    ///
    /// ```rust
    /// let engine = TestEngine::new();
    /// ```
    pub fn new() -> Self {
        let input = MockInput::new();
        let runtime = MockRuntime::new();
        let state = MockState::new();
        let metrics = metrics::default_noop_collector();

        let engine = Engine::new(input, runtime, state, metrics);

        Self { engine }
    }

    /// Configure the engine with a custom MockRuntime.
    ///
    /// This allows you to pre-configure complex runtime behavior before
    /// creating the engine.
    ///
    /// # Example
    ///
    /// ```rust
    /// let runtime = MockRuntime::new()
    ///     .with_remap(KeyCode::A, KeyCode::B)
    ///     .with_hook("on_init");
    ///
    /// let engine = TestEngine::with_runtime(runtime);
    /// ```
    pub fn with_runtime(runtime: MockRuntime) -> Self {
        let input = MockInput::new();
        let state = MockState::new();
        let metrics = metrics::default_noop_collector();

        let engine = Engine::new(input, runtime, state, metrics);

        Self { engine }
    }

    /// Add a key remapping to the engine's runtime.
    ///
    /// This is a convenience method that configures the underlying MockRuntime
    /// to remap `from` to `to`.
    ///
    /// # Example
    ///
    /// ```rust
    /// let engine = TestEngine::new()
    ///     .with_remap(KeyCode::CapsLock, KeyCode::Escape);
    /// ```
    pub fn with_remap(mut self, from: KeyCode, to: KeyCode) -> Self {
        self.engine.script_mut().registry_mut().remap(from, to);
        self
    }

    /// Configure a key to be blocked.
    ///
    /// # Example
    ///
    /// ```rust
    /// let engine = TestEngine::new()
    ///     .with_block(KeyCode::CapsLock);
    /// ```
    pub fn with_block(mut self, key: KeyCode) -> Self {
        self.engine.script_mut().registry_mut().block(key);
        self
    }

    /// Configure multiple remappings at once.
    ///
    /// # Example
    ///
    /// ```rust
    /// let engine = TestEngine::new()
    ///     .with_remaps(&[
    ///         (KeyCode::A, KeyCode::B),
    ///         (KeyCode::CapsLock, KeyCode::Escape),
    ///     ]);
    /// ```
    pub fn with_remaps(mut self, remaps: &[(KeyCode, KeyCode)]) -> Self {
        for (from, to) in remaps {
            self.engine.script_mut().registry_mut().remap(*from, *to);
        }
        self
    }

    /// Configure multiple keys to be blocked.
    ///
    /// # Example
    ///
    /// ```rust
    /// let engine = TestEngine::new()
    ///     .with_blocks(&[KeyCode::CapsLock, KeyCode::NumLock]);
    /// ```
    pub fn with_blocks(mut self, keys: &[KeyCode]) -> Self {
        for key in keys {
            self.engine.script_mut().registry_mut().block(*key);
        }
        self
    }

    /// Process a single input event through the engine.
    ///
    /// This is the main test interface - it takes an InputEvent and returns
    /// the resulting OutputAction.
    ///
    /// # Example
    ///
    /// ```rust
    /// let engine = TestEngine::new()
    ///     .with_remap(KeyCode::A, KeyCode::B);
    ///
    /// let event = InputEvent::key_down(KeyCode::A, 0);
    /// let output = engine.process(&event);
    /// assert_eq!(output, OutputAction::KeyDown(KeyCode::B));
    /// ```
    pub fn process(&self, event: &InputEvent) -> OutputAction {
        self.engine.process_event(event)
    }

    /// Process a key-down event for the given key.
    ///
    /// Convenience method that creates an InputEvent and processes it.
    ///
    /// # Example
    ///
    /// ```rust
    /// let engine = TestEngine::new()
    ///     .with_remap(KeyCode::A, KeyCode::B);
    ///
    /// let output = engine.process_key_down(KeyCode::A);
    /// assert_eq!(output, OutputAction::KeyDown(KeyCode::B));
    /// ```
    pub fn process_key_down(&self, key: KeyCode) -> OutputAction {
        let event = InputEvent::key_down(key, 0);
        self.process(&event)
    }

    /// Process a key-up event for the given key.
    ///
    /// Convenience method that creates an InputEvent and processes it.
    ///
    /// # Example
    ///
    /// ```rust
    /// let engine = TestEngine::new()
    ///     .with_remap(KeyCode::A, KeyCode::B);
    ///
    /// let output = engine.process_key_up(KeyCode::A);
    /// assert_eq!(output, OutputAction::KeyUp(KeyCode::B));
    /// ```
    pub fn process_key_up(&self, key: KeyCode) -> OutputAction {
        let event = InputEvent::key_up(key, 0);
        self.process(&event)
    }

    /// Get a reference to the underlying Engine.
    ///
    /// Use this for accessing engine methods not wrapped by TestEngine.
    pub fn engine(&self) -> &Engine<MockInput, MockRuntime, MockState> {
        &self.engine
    }

    /// Get a mutable reference to the underlying Engine.
    ///
    /// Use this for accessing mutable engine methods not wrapped by TestEngine.
    pub fn engine_mut(&mut self) -> &mut Engine<MockInput, MockRuntime, MockState> {
        &mut self.engine
    }

    /// Get a reference to the MockRuntime.
    ///
    /// Useful for verifying runtime behavior or checking call history.
    pub fn runtime(&self) -> &MockRuntime {
        self.engine.script()
    }

    /// Get a mutable reference to the MockRuntime.
    ///
    /// Useful for modifying runtime configuration after engine creation.
    pub fn runtime_mut(&mut self) -> &mut MockRuntime {
        self.engine.script_mut()
    }

    /// Get a reference to the MockState.
    ///
    /// Useful for verifying state changes during tests.
    pub fn state(&self) -> &MockState {
        self.engine.state()
    }

    /// Get a mutable reference to the MockState.
    ///
    /// Useful for setting up initial state before tests.
    pub fn state_mut(&mut self) -> &mut MockState {
        self.engine.state_mut()
    }

    /// Check if the engine is currently running.
    pub fn is_running(&self) -> bool {
        self.engine.is_running()
    }
}

impl Default for TestEngine {
    fn default() -> Self {
        Self::new()
    }
}

// Drop implementation ensures proper cleanup of mock resources
impl Drop for TestEngine {
    fn drop(&mut self) {
        // Mocks automatically clean up when dropped
        // This is here as a documentation point and for future extensibility
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_engine() {
        let engine = TestEngine::new();
        assert!(!engine.is_running());
    }

    #[test]
    fn test_with_remap() {
        let engine = TestEngine::new().with_remap(KeyCode::A, KeyCode::B);

        let output = engine.process_key_down(KeyCode::A);
        assert_eq!(output, OutputAction::KeyDown(KeyCode::B));

        let output = engine.process_key_up(KeyCode::A);
        assert_eq!(output, OutputAction::KeyUp(KeyCode::B));
    }

    #[test]
    fn test_with_block() {
        let engine = TestEngine::new().with_block(KeyCode::CapsLock);

        let output = engine.process_key_down(KeyCode::CapsLock);
        assert_eq!(output, OutputAction::Block);

        let output = engine.process_key_up(KeyCode::CapsLock);
        assert_eq!(output, OutputAction::Block);
    }

    #[test]
    fn test_with_remaps() {
        let engine = TestEngine::new().with_remaps(&[
            (KeyCode::A, KeyCode::B),
            (KeyCode::CapsLock, KeyCode::Escape),
        ]);

        assert_eq!(
            engine.process_key_down(KeyCode::A),
            OutputAction::KeyDown(KeyCode::B)
        );
        assert_eq!(
            engine.process_key_down(KeyCode::CapsLock),
            OutputAction::KeyDown(KeyCode::Escape)
        );
    }

    #[test]
    fn test_with_blocks() {
        let engine = TestEngine::new().with_blocks(&[KeyCode::CapsLock, KeyCode::NumLock]);

        assert_eq!(
            engine.process_key_down(KeyCode::CapsLock),
            OutputAction::Block
        );
        assert_eq!(
            engine.process_key_down(KeyCode::NumLock),
            OutputAction::Block
        );
    }

    #[test]
    fn test_process_passthrough() {
        let engine = TestEngine::new();

        // Unconfigured key should pass through
        let output = engine.process_key_down(KeyCode::A);
        assert_eq!(output, OutputAction::PassThrough);
    }

    #[test]
    fn test_process_with_event() {
        let engine = TestEngine::new().with_remap(KeyCode::A, KeyCode::B);

        let event = InputEvent::key_down(KeyCode::A, 12345);
        let output = engine.process(&event);
        assert_eq!(output, OutputAction::KeyDown(KeyCode::B));
    }

    #[test]
    fn test_runtime_access() {
        let mut engine = TestEngine::new();

        // Configure via mutable runtime access
        engine.runtime_mut().registry_mut().remap(KeyCode::A, KeyCode::B);

        let output = engine.process_key_down(KeyCode::A);
        assert_eq!(output, OutputAction::KeyDown(KeyCode::B));
    }

    #[test]
    fn test_state_access() {
        let mut engine = TestEngine::new();

        // Verify we can access state
        let state = engine.state();
        assert!(state.get_changes().is_empty());

        // Verify we can mutate state
        let _state_mut = engine.state_mut();
    }

    #[test]
    fn test_with_custom_runtime() {
        let runtime = MockRuntime::new()
            .with_remap(KeyCode::A, KeyCode::B)
            .with_block(KeyCode::CapsLock);

        let engine = TestEngine::with_runtime(runtime);

        assert_eq!(
            engine.process_key_down(KeyCode::A),
            OutputAction::KeyDown(KeyCode::B)
        );
        assert_eq!(
            engine.process_key_down(KeyCode::CapsLock),
            OutputAction::Block
        );
    }

    #[test]
    fn test_default_implementation() {
        let engine = TestEngine::default();
        assert!(!engine.is_running());
    }
}
