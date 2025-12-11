//! FFI exportable trait definitions.
//!
//! This module defines the [`FfiExportable`] trait which provides a contract for
//! domain modules that expose functionality through the FFI layer. Domain modules
//! implement this trait to integrate with the handle-based state management system
//! and participate in the FFI lifecycle.
//!
//! # Architecture
//!
//! The trait-based FFI architecture separates concerns into layers:
//!
//! 1. **Domain Layer**: Domain modules implement `FfiExportable` (e.g., `DiscoveryFfi`, `ValidationFfi`)
//! 2. **FFI Layer**: Generated C-ABI wrappers interact with domain implementations
//! 3. **Client Layer**: Flutter/Dart bindings call FFI functions via handles
//!
//! # Lifecycle
//!
//! An FFI domain follows this lifecycle:
//!
//! 1. Domain struct implements `FfiExportable` trait
//! 2. Flutter creates a context via FFI, receives a handle
//! 3. Flutter calls `init()` to initialize the domain within the context
//! 4. Domain methods are invoked via handle + method name
//! 5. Flutter calls `cleanup()` to release domain resources
//! 6. Context is disposed, handle becomes invalid
//!
//! # Example Implementation
//!
//! ```ignore
//! use keyrx_core::ffi::{FfiExportable, FfiContext, FfiError};
//!
//! struct DiscoveryFfi;
//!
//! impl FfiExportable for DiscoveryFfi {
//!     const DOMAIN: &'static str = "discovery";
//!
//!     fn init(ctx: &mut FfiContext) -> Result<(), FfiError> {
//!         // Initialize discovery-specific state
//!         struct DiscoveryState {
//!             active: bool,
//!             session: Option<DiscoverySession>,
//!         }
//!
//!         ctx.set_domain(Self::DOMAIN, DiscoveryState {
//!             active: false,
//!             session: None,
//!         });
//!
//!         Ok(())
//!     }
//!
//!     fn cleanup(ctx: &mut FfiContext) {
//!         // Clean up discovery resources
//!         ctx.remove_domain(Self::DOMAIN);
//!     }
//! }
//!
//! // Domain methods use #[ffi_export] macro (from procedural macro crate)
//! impl DiscoveryFfi {
//!     #[ffi_export]
//!     fn start_discovery(
//!         ctx: &mut FfiContext,
//!         device_id: &str,
//!         rows: u8,
//!         cols: u8,
//!     ) -> Result<StartResult, FfiError> {
//!         // Access domain state
//!         let mut state = ctx.get_domain_mut::<DiscoveryState>(Self::DOMAIN)
//!             .ok_or_else(|| FfiError::internal("discovery not initialized"))?;
//!
//!         // Business logic here
//!         Ok(StartResult { total_keys: rows as usize * cols as usize })
//!     }
//! }
//! ```
//!
//! # Design Patterns
//!
//! Following the pattern established by [`InputSource`](crate::traits::InputSource):
//!
//! - **Trait Constants**: Use `const DOMAIN` for compile-time domain identification
//! - **Clear Lifecycle**: Explicit `init()` and `cleanup()` methods
//! - **Comprehensive Docs**: Document lifecycle, thread safety, error handling
//! - **Thread Safety**: Domain state is wrapped in `Arc<RwLock<>>` by `FfiContext`
//! - **Error Handling**: Use [`FfiError`] for all failures

use crate::ffi::context::FfiContext;
use crate::ffi::error::FfiError;

/// Trait for FFI-exportable domain modules.
///
/// Domain modules implement this trait to integrate with the FFI architecture.
/// Each domain is identified by a unique `DOMAIN` constant and manages its own
/// state within an [`FfiContext`].
///
/// # Domain State
///
/// Domain implementations typically define a companion state struct that is
/// stored in the `FfiContext` during initialization. The state struct should:
///
/// - Implement `Send + Sync` for thread safety
/// - Be `'static` to be stored in the context
/// - Encapsulate all domain-specific runtime state
///
/// # Thread Safety
///
/// The `FfiContext` provides thread-safe access to domain state through
/// `Arc<RwLock<>>` wrappers. Domain implementations must:
///
/// - Not use global mutable state
/// - Access state only through `FfiContext::get_domain()` / `get_domain_mut()`
/// - Be safe to call from multiple threads (FFI calls may come from any thread)
///
/// # Error Handling
///
/// - `init()` should return `FfiError` if initialization fails (e.g., invalid config)
/// - `cleanup()` should be infallible and make best-effort cleanup even if errors occur
/// - Domain methods should use `FfiResult<T>` for all operations
///
/// # Example
///
/// ```
/// # use keyrx_core::ffi::{FfiExportable, FfiContext, FfiError};
/// struct ValidationFfi;
///
/// impl FfiExportable for ValidationFfi {
///     const DOMAIN: &'static str = "validation";
///
///     fn init(ctx: &mut FfiContext) -> Result<(), FfiError> {
///         // Initialize domain state
///         struct ValidationState {
///             config: ValidationConfig,
///             results: Vec<ValidationResult>,
///         }
///
///         ctx.set_domain(Self::DOMAIN, ValidationState {
///             config: ValidationConfig::default(),
///             results: Vec::new(),
///         });
///
///         Ok(())
///     }
///
///     fn cleanup(ctx: &mut FfiContext) {
///         // Remove domain state
///         ctx.remove_domain(Self::DOMAIN);
///     }
/// }
/// ```
///
/// # Integration with Macros
///
/// The procedural macro `#[ffi_export]` (defined in `keyrx-ffi-macros` crate)
/// generates C-ABI wrapper functions for domain methods. The macro:
///
/// - Generates `extern "C"` function with handle parameter
/// - Performs null checks and UTF-8 validation on string parameters
/// - Wraps panics with `catch_unwind` and converts to `FfiError`
/// - Serializes results to JSON format (`"ok:{...}"` or `"error:{...}"`)
/// - Manages string ownership and provides cleanup functions
pub trait FfiExportable {
    /// Domain name for namespacing and identification.
    ///
    /// Must be a unique, lowercase identifier (e.g., "discovery", "validation", "engine").
    /// This constant is used to:
    /// - Store and retrieve domain state from `FfiContext`
    /// - Namespace FFI function names (e.g., `keyrx_discovery_start`)
    /// - Identify the domain in logs and error messages
    ///
    /// # Naming Convention
    ///
    /// - Use lowercase with underscores for multi-word domains
    /// - Keep it short and descriptive (e.g., "discovery", "validation")
    /// - Must be a valid Rust identifier component
    const DOMAIN: &'static str;

    /// Initialize domain state within the context.
    ///
    /// Called once when the domain is first accessed in a context. Should:
    /// - Create and store domain state via `ctx.set_domain()`
    /// - Perform any one-time setup (e.g., thread pool creation)
    /// - Return `Err` if initialization cannot proceed
    ///
    /// # Parameters
    ///
    /// * `ctx` - The FFI context to initialize domain state in
    ///
    /// # Returns
    ///
    /// - `Ok(())` - Domain successfully initialized
    /// - `Err(FfiError)` - Initialization failed (context remains valid)
    ///
    /// # Errors
    ///
    /// Return an error if:
    /// - Configuration is invalid
    /// - Required resources are unavailable
    /// - The domain is already initialized (if re-initialization is not supported)
    ///
    /// # Example
    ///
    /// ```
    /// # use keyrx_core::ffi::{FfiExportable, FfiContext, FfiError};
    /// # struct EngineFfi;
    /// # impl FfiExportable for EngineFfi {
    /// # const DOMAIN: &'static str = "engine";
    /// fn init(ctx: &mut FfiContext) -> Result<(), FfiError> {
    ///     if ctx.has_domain(Self::DOMAIN) {
    ///         return Err(FfiError::invalid_input("engine already initialized"));
    ///     }
    ///
    ///     struct EngineState {
    ///         running: bool,
    ///         config: EngineConfig,
    ///     }
    ///
    ///     ctx.set_domain(Self::DOMAIN, EngineState {
    ///         running: false,
    ///         config: EngineConfig::default(),
    ///     });
    ///
    ///     Ok(())
    /// }
    /// # fn cleanup(_ctx: &mut FfiContext) {}
    /// # }
    /// ```
    fn init(ctx: &mut FfiContext) -> Result<(), FfiError>;

    /// Clean up domain state and release resources.
    ///
    /// Called when the context is disposed or the domain is explicitly removed.
    /// Should:
    /// - Stop any background tasks or threads
    /// - Release system resources (file handles, network connections)
    /// - Remove domain state via `ctx.remove_domain()`
    ///
    /// # Parameters
    ///
    /// * `ctx` - The FFI context to clean up domain state from
    ///
    /// # Panics
    ///
    /// Should not panic. Make best-effort cleanup even if errors occur.
    /// Log errors for debugging but don't propagate them.
    ///
    /// # Example
    ///
    /// ```
    /// # use keyrx_core::ffi::{FfiExportable, FfiContext, FfiError};
    /// # struct DiscoveryFfi;
    /// # impl FfiExportable for DiscoveryFfi {
    /// # const DOMAIN: &'static str = "discovery";
    /// # fn init(_ctx: &mut FfiContext) -> Result<(), FfiError> { Ok(()) }
    /// fn cleanup(ctx: &mut FfiContext) {
    ///     // Stop background discovery session if active
    ///     if let Some(mut state) = ctx.get_domain_mut::<DiscoveryState>(Self::DOMAIN) {
    ///         if let Some(session) = state.downcast_mut::<DiscoveryState>()
    ///             .and_then(|s| s.session.take()) {
    ///             // Cancel the session
    ///             session.cancel();
    ///         }
    ///     }
    ///
    ///     // Remove domain state
    ///     ctx.remove_domain(Self::DOMAIN);
    /// }
    /// # }
    /// ```
    fn cleanup(ctx: &mut FfiContext);
}

/// Marker trait for domain state types.
///
/// This trait is automatically implemented for all types that are `Send + Sync + 'static`.
/// It serves as documentation and type constraint for domain state stored in `FfiContext`.
///
/// # Purpose
///
/// - Document which types are intended to be stored as domain state
/// - Provide a compile-time guarantee of thread safety
/// - Enable trait bounds in generic functions working with domain state
///
/// # Example
///
/// ```
/// # use keyrx_core::ffi::FfiDomain;
/// struct DiscoveryState {
///     active: bool,
///     session: Option<DiscoverySession>,
/// }
///
/// // Automatically implements FfiDomain if Send + Sync
/// fn assert_is_domain_state<T: FfiDomain>() {}
/// assert_is_domain_state::<DiscoveryState>();
/// ```
pub trait FfiDomain: Send + Sync + 'static {}

/// Blanket implementation for all types that meet the requirements.
impl<T: Send + Sync + 'static> FfiDomain for T {}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestDomainState {
        value: i32,
        active: bool,
    }

    struct TestFfi;

    impl FfiExportable for TestFfi {
        const DOMAIN: &'static str = "test";

        fn init(ctx: &mut FfiContext) -> Result<(), FfiError> {
            if ctx.has_domain(Self::DOMAIN) {
                return Err(FfiError::invalid_input("test domain already initialized"));
            }

            ctx.set_domain(
                Self::DOMAIN,
                TestDomainState {
                    value: 0,
                    active: false,
                },
            );

            Ok(())
        }

        fn cleanup(ctx: &mut FfiContext) {
            ctx.remove_domain(Self::DOMAIN);
        }
    }

    #[test]
    fn test_ffi_exportable_init() {
        let mut ctx = FfiContext::new();
        let result = TestFfi::init(&mut ctx);
        assert!(result.is_ok());
        assert!(ctx.has_domain(TestFfi::DOMAIN));
    }

    #[test]
    fn test_ffi_exportable_double_init_fails() {
        let mut ctx = FfiContext::new();
        TestFfi::init(&mut ctx).unwrap();

        // Second init should fail
        let result = TestFfi::init(&mut ctx);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "INVALID_INPUT");
    }

    #[test]
    fn test_ffi_exportable_cleanup() {
        let mut ctx = FfiContext::new();
        TestFfi::init(&mut ctx).unwrap();
        assert!(ctx.has_domain(TestFfi::DOMAIN));

        TestFfi::cleanup(&mut ctx);
        assert!(!ctx.has_domain(TestFfi::DOMAIN));
    }

    #[test]
    fn test_domain_state_access() {
        let mut ctx = FfiContext::new();
        TestFfi::init(&mut ctx).unwrap();

        // Access domain state
        let state_guard = ctx.get_domain::<TestDomainState>(TestFfi::DOMAIN);
        assert!(state_guard.is_some());

        // Verify initial values
        let guard = state_guard.unwrap();
        let state = guard.downcast_ref::<TestDomainState>().unwrap();
        assert_eq!(state.value, 0);
        assert!(!state.active);
    }

    #[test]
    fn test_domain_state_mutation() {
        let mut ctx = FfiContext::new();
        TestFfi::init(&mut ctx).unwrap();

        // Modify state
        {
            let mut state_guard = ctx
                .get_domain_mut::<TestDomainState>(TestFfi::DOMAIN)
                .unwrap();
            let state = state_guard.downcast_mut::<TestDomainState>().unwrap();
            state.value = 42;
            state.active = true;
        }

        // Verify mutation
        let state_guard = ctx.get_domain::<TestDomainState>(TestFfi::DOMAIN).unwrap();
        let state = state_guard.downcast_ref::<TestDomainState>().unwrap();
        assert_eq!(state.value, 42);
        assert!(state.active);
    }

    #[test]
    fn test_ffi_domain_marker_trait() {
        // Verify that our test state implements FfiDomain
        fn assert_ffi_domain<T: FfiDomain>() {}
        assert_ffi_domain::<TestDomainState>();

        // Verify that common types implement FfiDomain
        assert_ffi_domain::<String>();
        assert_ffi_domain::<Vec<i32>>();
        assert_ffi_domain::<std::collections::HashMap<String, i32>>();
    }

    #[test]
    fn test_multiple_domains_in_context() {
        struct AnotherTestFfi;
        #[allow(dead_code)]
        struct AnotherState {
            name: String,
        }

        impl FfiExportable for AnotherTestFfi {
            const DOMAIN: &'static str = "another_test";

            fn init(ctx: &mut FfiContext) -> Result<(), FfiError> {
                ctx.set_domain(
                    Self::DOMAIN,
                    AnotherState {
                        name: "test".to_string(),
                    },
                );
                Ok(())
            }

            fn cleanup(ctx: &mut FfiContext) {
                ctx.remove_domain(Self::DOMAIN);
            }
        }

        let mut ctx = FfiContext::new();
        TestFfi::init(&mut ctx).unwrap();
        AnotherTestFfi::init(&mut ctx).unwrap();

        // Both domains should exist
        assert!(ctx.has_domain(TestFfi::DOMAIN));
        assert!(ctx.has_domain(AnotherTestFfi::DOMAIN));

        // Clean up one domain
        TestFfi::cleanup(&mut ctx);
        assert!(!ctx.has_domain(TestFfi::DOMAIN));
        assert!(ctx.has_domain(AnotherTestFfi::DOMAIN));
    }
}
