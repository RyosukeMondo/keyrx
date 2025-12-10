#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::print_stdout, clippy::print_stderr, clippy::field_reassign_with_default, clippy::useless_conversion, clippy::assertions_on_constants, clippy::manual_div_ceil, clippy::manual_strip, clippy::len_zero, clippy::redundant_closure, clippy::manual_range_contains, clippy::default_constructed_unit_structs, clippy::clone_on_copy, clippy::io_other_error, clippy::bool_assert_comparison, clippy::approx_constant, clippy::let_unit_value, clippy::while_let_on_iterator, clippy::await_holding_lock, clippy::unnecessary_cast, clippy::drop_non_drop, clippy::needless_range_loop, unused_imports, unused_variables, dead_code, unsafe_code, clippy::collapsible_if, clippy::bool_comparison, unexpected_cfgs))]
//! KeyRx Core - The Ultimate Input Remapping Engine
//!
//!
//! This crate provides the core functionality for KeyRx:
//! - Async event loop for input processing
//! - Rhai scripting integration
//! - Layer and modifier state management
//! - OS-specific input drivers
//! - FFI exports for Flutter integration

// KeyrxError is intentionally large to include all necessary context
#![allow(clippy::result_large_err)]

pub mod api;
pub mod services;

/// Global allocator that enables allocation tracking when profiling is active.
#[global_allocator]
static GLOBAL_ALLOCATOR: profiling::TrackingAllocator = profiling::TrackingAllocator;

// Re-export procedural macros
pub use keyrx_ffi_macros::ffi_export;

pub mod cli;
pub mod config;
pub mod definitions;
pub mod discovery;
pub mod drivers;
pub mod engine;
pub mod error;
pub mod errors;
pub mod ffi;
pub mod hardware;
pub mod identity;
pub mod metrics;
pub mod migration;
pub mod mocks;
pub mod observability;
pub mod profiling;
pub mod registry;
pub mod safety;
pub mod scripting;
pub mod traits;
pub mod uat;
pub mod validation;

// Re-export commonly used types
pub use discovery::{
    default_schema_version, device_profiles_dir, DeviceEvent, DeviceEventReceiver,
    DeviceEventSender, DeviceId, DeviceProfile, DeviceRegistry, DeviceState, DeviceWatchError,
    DeviceWatcher, DiscoveryProgress, DiscoveryReason, DiscoverySession, DiscoverySummary,
    DuplicateWarning, ExpectedPosition, PhysicalKey, ProfileSource, RegistryEntry, RegistryStatus,
    SessionError, SessionStatus, SessionUpdate, WatcherResult, SCHEMA_VERSION,
};
pub use engine::{
    CoordinationAction, Engine, HotplugAction, HotplugSession, InputEvent, KeyCode, Layer,
    ModifierSet, MultiDeviceCoordinator, OutputAction,
};
pub use error::KeyRxError;
pub use ffi::runtime::{
    clear_revolutionary_runtime, set_revolutionary_runtime, with_revolutionary_runtime,
    RevolutionaryRuntime, RevolutionaryRuntimeGuard,
};
pub use identity::DeviceIdentity;
pub use mocks::{MockInput, MockRuntime, MockState};
pub use scripting::RhaiRuntime;
pub use traits::{InputSource, ScriptRuntime, StateStore};
