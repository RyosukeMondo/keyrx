//! Test utilities for tap-hold functionality.
//!
//! This module provides shared test helpers and re-exports all test modules.

extern crate alloc;

pub(crate) use alloc::vec;
pub(crate) use alloc::vec::Vec;

// Re-export everything from parent module for tests
pub(crate) use super::*;
pub(crate) use crate::config::KeyCode;

// Shared test helper functions
pub(crate) fn make_pending_state(
    key: KeyCode,
    tap: KeyCode,
    modifier: u8,
    press_time: u64,
) -> TapHoldState {
    let config = TapHoldConfig::from_ms(tap, modifier, 200);
    let mut state = TapHoldState::new(key, config);
    state.transition_to_pending(press_time);
    state
}

mod processor_test;
mod registry_test;
mod scenarios_part1;
mod scenarios_part2;
mod state_test;
mod types_test;
