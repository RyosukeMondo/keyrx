//! Virtual E2E Tests for keyrx daemon.
//!
//! These tests use virtual input devices (uinput) to test the complete
//! keyboard remapping pipeline without requiring physical hardware.
//!
//! # Running These Tests
//!
//! These tests require:
//! - Linux with uinput module loaded (`sudo modprobe uinput`)
//! - Read/write access to `/dev/uinput` (add user to 'uinput' group)
//! - Read access to `/dev/input/event*` (add user to 'input' group)
//! - The keyrx_daemon binary built
//!
//! Run with:
//! ```bash
//! cargo test -p keyrx_daemon --features linux --test virtual_e2e_test
//! ```
//!
//! Tests automatically skip with a message if uinput/input access is not available.

#![cfg(any(
    all(target_os = "linux", feature = "linux"),
    all(target_os = "windows", feature = "windows")
))]

pub mod advanced_output;
pub mod advanced_sequences;
pub mod basic;
pub mod complex;
pub mod layers;
pub mod passthrough;

