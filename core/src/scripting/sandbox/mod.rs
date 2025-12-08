//! Script sandbox for secure Rhai execution.
//!
//! This module provides a hardened sandbox for executing Rhai scripts with:
//! - Capability-based function access control
//! - Resource limits (CPU, memory, recursion)
//! - Input validation
//! - O(1) function lookup

pub mod budget;
pub mod capability;
pub mod function_capabilities;
pub mod registry;
pub mod validation;
pub mod validators;

pub use budget::{ResourceBudget, ResourceConfig, ResourceExhausted, ResourceUsage};
pub use capability::{ScriptCapability, ScriptMode};
pub use function_capabilities::build_function_registry;
pub use registry::{CapabilityRegistry, FunctionCapability};
pub use validation::{InputValidator, ValidationError, ValidationResult};

use rhai::Engine;
use std::sync::Arc;
use thiserror::Error;

/// Unified sandbox for secure script execution.
///
/// Combines capability checks, input validation, and resource tracking into
/// a single interface for script security enforcement.
///
/// # Architecture
///
/// The sandbox operates in layers:
/// 1. **Capability checks**: Function calls are validated against the current mode
/// 2. **Input validation**: Function parameters are validated before execution
/// 3. **Resource tracking**: CPU, memory, and recursion limits are enforced
///
/// # Examples
///
/// ```
/// use keyrx_core::scripting::sandbox::{
///     ScriptSandbox, ScriptMode, ResourceConfig, build_function_registry
/// };
///
/// let registry = build_function_registry();
/// let config = ResourceConfig::default();
/// let sandbox = ScriptSandbox::new(registry, config, ScriptMode::Standard);
///
/// // Check if function is allowed
/// assert!(sandbox.check_function_allowed("send_key").is_ok());
/// assert!(sandbox.check_function_allowed("clipboard_get").is_err());
/// ```
pub struct ScriptSandbox {
    /// Registry of function capabilities
    registry: Arc<CapabilityRegistry>,
    /// Resource budget for current execution
    budget: Arc<ResourceBudget>,
    /// Current execution mode
    mode: ScriptMode,
}

impl ScriptSandbox {
    /// Create a new sandbox with the given configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// use keyrx_core::scripting::sandbox::{
    ///     ScriptSandbox, ScriptMode, ResourceConfig, CapabilityRegistry
    /// };
    ///
    /// let registry = CapabilityRegistry::new();
    /// let config = ResourceConfig::default();
    /// let sandbox = ScriptSandbox::new(registry, config, ScriptMode::Standard);
    /// ```
    pub fn new(registry: CapabilityRegistry, config: ResourceConfig, mode: ScriptMode) -> Self {
        Self {
            registry: Arc::new(registry),
            budget: Arc::new(ResourceBudget::new(config)),
            mode,
        }
    }

    /// Get the current execution mode.
    pub fn mode(&self) -> ScriptMode {
        self.mode
    }

    /// Set the execution mode.
    ///
    /// # Examples
    ///
    /// ```
    /// use keyrx_core::scripting::sandbox::{ScriptSandbox, ScriptMode, ResourceConfig, CapabilityRegistry};
    ///
    /// let mut sandbox = ScriptSandbox::new(
    ///     CapabilityRegistry::new(),
    ///     ResourceConfig::default(),
    ///     ScriptMode::Standard
    /// );
    ///
    /// sandbox.set_mode(ScriptMode::Safe);
    /// assert_eq!(sandbox.mode(), ScriptMode::Safe);
    /// ```
    pub fn set_mode(&mut self, mode: ScriptMode) {
        self.mode = mode;
    }

    /// Get reference to the capability registry.
    pub fn registry(&self) -> &CapabilityRegistry {
        &self.registry
    }

    /// Get reference to the resource budget.
    pub fn budget(&self) -> &ResourceBudget {
        &self.budget
    }

    /// Check if a function is allowed in the current mode.
    ///
    /// Returns `Ok(())` if allowed, or an error if not.
    ///
    /// # Examples
    ///
    /// ```
    /// use keyrx_core::scripting::sandbox::{
    ///     ScriptSandbox, ScriptMode, ResourceConfig,
    ///     CapabilityRegistry, FunctionCapability, ScriptCapability
    /// };
    ///
    /// let mut registry = CapabilityRegistry::new();
    /// registry.register(FunctionCapability::new(
    ///     "send_key",
    ///     ScriptCapability::Standard,
    ///     "Send a key event"
    /// ));
    ///
    /// let sandbox = ScriptSandbox::new(registry, ResourceConfig::default(), ScriptMode::Standard);
    ///
    /// assert!(sandbox.check_function_allowed("send_key").is_ok());
    /// ```
    pub fn check_function_allowed(&self, function_name: &str) -> Result<(), SandboxError> {
        let cap =
            self.registry
                .get(function_name)
                .ok_or_else(|| SandboxError::UnknownFunction {
                    name: function_name.to_string(),
                })?;

        if !cap.is_allowed_in(self.mode) {
            return Err(SandboxError::FunctionNotAllowed {
                name: function_name.to_string(),
                required: cap.capability,
                current_mode: self.mode,
            });
        }

        Ok(())
    }

    /// Configure a Rhai engine with sandbox limits.
    ///
    /// This sets engine-level limits that complement sandbox checks:
    /// - Maximum operations (instruction count)
    /// - Maximum call stack depth (recursion limit)
    /// - Expression depth limits
    ///
    /// # Examples
    ///
    /// ```
    /// use keyrx_core::scripting::sandbox::{ScriptSandbox, ScriptMode, ResourceConfig, CapabilityRegistry};
    /// use rhai::Engine;
    ///
    /// let sandbox = ScriptSandbox::new(
    ///     CapabilityRegistry::new(),
    ///     ResourceConfig::default(),
    ///     ScriptMode::Standard
    /// );
    ///
    /// let mut engine = Engine::new();
    /// sandbox.configure_engine(&mut engine);
    /// ```
    pub fn configure_engine(&self, engine: &mut Engine) {
        let usage = self.budget.usage();

        // Set operation limit based on budget
        engine.set_max_operations(usage.max_instructions);

        // Set call stack depth based on recursion limit
        engine.set_max_call_levels(usage.max_recursion as usize);

        // Set expression depth to prevent deeply nested expressions
        let max_expr_depth = 64;
        engine.set_max_expr_depths(max_expr_depth, max_expr_depth);

        // Set module limit
        engine.set_max_modules(16);

        // Set function limit
        engine.set_max_functions(256);
    }

    /// Get current resource usage statistics.
    ///
    /// # Examples
    ///
    /// ```
    /// use keyrx_core::scripting::sandbox::{ScriptSandbox, ScriptMode, ResourceConfig, CapabilityRegistry};
    ///
    /// let sandbox = ScriptSandbox::new(
    ///     CapabilityRegistry::new(),
    ///     ResourceConfig::default(),
    ///     ScriptMode::Standard
    /// );
    ///
    /// let usage = sandbox.resource_usage();
    /// println!("Instructions used: {}/{}", usage.instructions, usage.max_instructions);
    /// ```
    pub fn resource_usage(&self) -> ResourceUsage {
        self.budget.usage()
    }

    /// Check all resource limits.
    ///
    /// This is a convenience method that checks all limits at once.
    pub fn check_resources(&self) -> Result<(), ResourceExhausted> {
        self.budget.check_instructions()?;
        self.budget.check_timeout()?;
        Ok(())
    }

    /// Reset resource usage for a new execution.
    pub fn reset_resources(&self) {
        self.budget.reset();
    }
}

/// Sandbox errors.
#[derive(Debug, Error)]
pub enum SandboxError {
    /// Function is not registered in the capability registry.
    #[error("Unknown function: {name}")]
    UnknownFunction { name: String },

    /// Function is not allowed in current execution mode.
    #[error(
        "Function '{name}' requires {required:?} capability but current mode is {current_mode:?}"
    )]
    FunctionNotAllowed {
        name: String,
        required: ScriptCapability,
        current_mode: ScriptMode,
    },

    /// Input validation failed.
    #[error("Validation failed: {0}")]
    ValidationFailed(#[from] ValidationError),

    /// Resource limit exceeded.
    #[error("Resource limit exceeded: {0}")]
    ResourceExhausted(#[from] ResourceExhausted),
}

impl Default for ScriptSandbox {
    fn default() -> Self {
        Self::new(
            build_function_registry(),
            ResourceConfig::default(),
            ScriptMode::default(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_registry() -> CapabilityRegistry {
        let mut registry = CapabilityRegistry::new();
        registry.register(FunctionCapability::new(
            "safe_func",
            ScriptCapability::Safe,
            "A safe function",
        ));
        registry.register(FunctionCapability::new(
            "std_func",
            ScriptCapability::Standard,
            "A standard function",
        ));
        registry.register(FunctionCapability::new(
            "adv_func",
            ScriptCapability::Advanced,
            "An advanced function",
        ));
        registry.register(FunctionCapability::new(
            "internal_func",
            ScriptCapability::Internal,
            "An internal function",
        ));
        registry
    }

    #[test]
    fn test_new_sandbox() {
        let registry = create_test_registry();
        let config = ResourceConfig::default();
        let sandbox = ScriptSandbox::new(registry, config, ScriptMode::Standard);

        assert_eq!(sandbox.mode(), ScriptMode::Standard);
        assert_eq!(sandbox.registry().len(), 4);
    }

    #[test]
    fn test_set_mode() {
        let mut sandbox = ScriptSandbox::new(
            create_test_registry(),
            ResourceConfig::default(),
            ScriptMode::Standard,
        );

        assert_eq!(sandbox.mode(), ScriptMode::Standard);

        sandbox.set_mode(ScriptMode::Safe);
        assert_eq!(sandbox.mode(), ScriptMode::Safe);

        sandbox.set_mode(ScriptMode::Full);
        assert_eq!(sandbox.mode(), ScriptMode::Full);
    }

    #[test]
    fn test_check_function_allowed_safe_mode() {
        let sandbox = ScriptSandbox::new(
            create_test_registry(),
            ResourceConfig::default(),
            ScriptMode::Safe,
        );

        assert!(sandbox.check_function_allowed("safe_func").is_ok());
        assert!(sandbox.check_function_allowed("std_func").is_err());
        assert!(sandbox.check_function_allowed("adv_func").is_err());
        assert!(sandbox.check_function_allowed("internal_func").is_err());
    }

    #[test]
    fn test_check_function_allowed_standard_mode() {
        let sandbox = ScriptSandbox::new(
            create_test_registry(),
            ResourceConfig::default(),
            ScriptMode::Standard,
        );

        assert!(sandbox.check_function_allowed("safe_func").is_ok());
        assert!(sandbox.check_function_allowed("std_func").is_ok());
        assert!(sandbox.check_function_allowed("adv_func").is_err());
        assert!(sandbox.check_function_allowed("internal_func").is_err());
    }

    #[test]
    fn test_check_function_allowed_full_mode() {
        let sandbox = ScriptSandbox::new(
            create_test_registry(),
            ResourceConfig::default(),
            ScriptMode::Full,
        );

        assert!(sandbox.check_function_allowed("safe_func").is_ok());
        assert!(sandbox.check_function_allowed("std_func").is_ok());
        assert!(sandbox.check_function_allowed("adv_func").is_ok());
        assert!(sandbox.check_function_allowed("internal_func").is_err());
    }

    #[test]
    fn test_check_function_allowed_unknown() {
        let sandbox = ScriptSandbox::new(
            create_test_registry(),
            ResourceConfig::default(),
            ScriptMode::Full,
        );

        let result = sandbox.check_function_allowed("unknown_func");
        assert!(result.is_err());
        match result.unwrap_err() {
            SandboxError::UnknownFunction { name } => assert_eq!(name, "unknown_func"),
            _ => panic!("Expected UnknownFunction error"),
        }
    }

    #[test]
    fn test_check_function_not_allowed_error() {
        let sandbox = ScriptSandbox::new(
            create_test_registry(),
            ResourceConfig::default(),
            ScriptMode::Safe,
        );

        let result = sandbox.check_function_allowed("std_func");
        assert!(result.is_err());
        match result.unwrap_err() {
            SandboxError::FunctionNotAllowed {
                name,
                required,
                current_mode,
            } => {
                assert_eq!(name, "std_func");
                assert_eq!(required, ScriptCapability::Standard);
                assert_eq!(current_mode, ScriptMode::Safe);
            }
            _ => panic!("Expected FunctionNotAllowed error"),
        }
    }

    #[test]
    fn test_configure_engine() {
        let config = ResourceConfig {
            max_instructions: 50_000,
            max_recursion: 32,
            max_memory: 512 * 1024,
            timeout: std::time::Duration::from_millis(50),
        };

        let sandbox = ScriptSandbox::new(create_test_registry(), config, ScriptMode::Standard);

        let mut engine = Engine::new();
        sandbox.configure_engine(&mut engine);

        // Engine is configured - verify it runs
        assert!(engine.eval::<i64>("40 + 2").is_ok());
    }

    #[test]
    fn test_resource_usage() {
        let sandbox = ScriptSandbox::new(
            create_test_registry(),
            ResourceConfig::default(),
            ScriptMode::Standard,
        );

        let usage = sandbox.resource_usage();
        assert_eq!(usage.instructions, 0);
        assert_eq!(usage.max_instructions, 100_000);
        assert_eq!(usage.recursion_depth, 0);
        assert_eq!(usage.max_recursion, 64);
    }

    #[test]
    fn test_check_resources() {
        let sandbox = ScriptSandbox::new(
            create_test_registry(),
            ResourceConfig::default(),
            ScriptMode::Standard,
        );

        assert!(sandbox.check_resources().is_ok());
    }

    #[test]
    fn test_check_resources_timeout() {
        let config = ResourceConfig {
            timeout: std::time::Duration::from_nanos(1),
            ..Default::default()
        };

        let sandbox = ScriptSandbox::new(create_test_registry(), config, ScriptMode::Standard);

        std::thread::sleep(std::time::Duration::from_millis(1));
        assert!(sandbox.check_resources().is_err());
    }

    #[test]
    fn test_default_sandbox() {
        let sandbox = ScriptSandbox::default();
        assert_eq!(sandbox.mode(), ScriptMode::Standard);
        assert!(!sandbox.registry().is_empty());
    }

    #[test]
    fn test_registry_access() {
        let sandbox = ScriptSandbox::new(
            create_test_registry(),
            ResourceConfig::default(),
            ScriptMode::Standard,
        );

        let registry = sandbox.registry();
        assert_eq!(registry.len(), 4);
        assert!(registry.get("safe_func").is_some());
    }

    #[test]
    fn test_budget_access() {
        let sandbox = ScriptSandbox::new(
            create_test_registry(),
            ResourceConfig::default(),
            ScriptMode::Standard,
        );

        let budget = sandbox.budget();
        assert!(budget.check_instructions().is_ok());
    }

    #[test]
    fn test_mode_switching_affects_checks() {
        let mut sandbox = ScriptSandbox::new(
            create_test_registry(),
            ResourceConfig::default(),
            ScriptMode::Full,
        );

        assert!(sandbox.check_function_allowed("adv_func").is_ok());

        sandbox.set_mode(ScriptMode::Standard);
        assert!(sandbox.check_function_allowed("adv_func").is_err());

        sandbox.set_mode(ScriptMode::Safe);
        assert!(sandbox.check_function_allowed("std_func").is_err());
    }

    #[test]
    fn test_sandbox_error_display() {
        let err = SandboxError::UnknownFunction {
            name: "test".to_string(),
        };
        assert!(err.to_string().contains("test"));

        let err = SandboxError::FunctionNotAllowed {
            name: "func".to_string(),
            required: ScriptCapability::Advanced,
            current_mode: ScriptMode::Safe,
        };
        assert!(err.to_string().contains("func"));
        assert!(err.to_string().contains("Advanced"));
    }

    #[test]
    fn test_engine_limits_prevent_infinite_loop() {
        let config = ResourceConfig {
            max_instructions: 1000,
            ..Default::default()
        };

        let sandbox = ScriptSandbox::new(create_test_registry(), config, ScriptMode::Standard);

        let mut engine = Engine::new();
        sandbox.configure_engine(&mut engine);

        let result = engine.eval::<()>("loop {}");
        assert!(result.is_err());
    }

    #[test]
    fn test_engine_limits_prevent_deep_recursion() {
        let config = ResourceConfig {
            max_recursion: 5,
            ..Default::default()
        };

        let sandbox = ScriptSandbox::new(create_test_registry(), config, ScriptMode::Standard);

        let mut engine = Engine::new();
        sandbox.configure_engine(&mut engine);

        let script = r#"
            fn recurse(n) {
                if n > 0 {
                    recurse(n - 1);
                }
            }
            recurse(100);
        "#;

        let result = engine.eval::<()>(script);
        assert!(result.is_err());
    }
}
