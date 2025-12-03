//! Safety wrappers for Windows driver operations.
//!
//! This module contains safe abstractions over unsafe Windows API calls used by the
//! keyboard driver. Each wrapper encapsulates unsafe operations with documented invariants
//! and provides a safe public API.
//!
//! # Safety Architecture
//!
//! The Windows driver uses several low-level APIs that require unsafe code:
//! - **SetWindowsHookEx/UnhookWindowsHookEx**: Global keyboard hook management
//! - **GetAsyncKeyState**: Keyboard state queries
//! - **Thread-local storage**: Event routing from hook callback to main thread
//!
//! Each unsafe operation is isolated into a dedicated wrapper type that:
//! 1. Encapsulates invariants (e.g., hook handle validity)
//! 2. Provides RAII cleanup (e.g., unhook on Drop)
//! 3. Catches panics where applicable (e.g., hook callbacks)
//! 4. Documents safety requirements with SAFETY comments
//!
//! # Module Organization
//!
//! - `hook`: SafeHook wrapper for Windows keyboard hooks
//! - `callback`: HookCallback with panic catching
//! - `thread_local`: ThreadLocalState for event routing
//!
//! # Performance Considerations
//!
//! Safety wrappers are designed to have minimal overhead:
//! - No additional heap allocations in hot paths
//! - Inline-able small functions
//! - Zero-cost abstractions where possible
//! - Target: < 10μs overhead per operation
//!
//! # Error Handling
//!
//! All operations that can fail return `Result<T, DriverError>` with:
//! - Clear error messages
//! - Platform-specific error codes
//! - Suggested recovery actions
//! - Retryability information
//!
//! # Example Usage
//!
//! ```no_run
//! use keyrx_core::drivers::windows::safety::hook::SafeHook;
//! use keyrx_core::drivers::windows::safety::callback::HookCallback;
//!
//! // Create a panic-safe callback
//! let callback = HookCallback::new(|event| {
//!     // Process keyboard event
//!     HookAction::PassThrough
//! });
//!
//! // Install hook with automatic cleanup on drop
//! let hook = SafeHook::install(callback)?;
//!
//! // Hook is automatically uninstalled when `hook` is dropped
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

// Module declarations (components will be implemented in subsequent tasks)
// pub mod callback;
// pub mod hook;
// pub mod thread_local;
