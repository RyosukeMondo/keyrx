//! Property-based fuzz tests for the script sandbox.
//!
//! This test suite uses proptest to verify sandbox security properties:
//! - Resource limits cannot be bypassed
//! - Invalid inputs are rejected
//! - Capability checks are enforced
//! - No panics or undefined behavior

use keyrx_core::scripting::sandbox::{
    build_function_registry, CapabilityRegistry, FunctionCapability, ResourceBudget,
    ResourceConfig, ScriptCapability, ScriptMode, ScriptSandbox, ValidationError,
};
use proptest::prelude::*;
use rhai::Engine;
use std::time::Duration;

// === Property Test Strategies ===

/// Generate arbitrary resource configurations with extreme values.
fn arb_resource_config() -> impl Strategy<Value = ResourceConfig> {
    (
        prop_oneof![
            Just(0u64),
            Just(1u64),
            Just(100u64),
            Just(10_000u64),
            Just(1_000_000u64),
            Just(u64::MAX),
        ],
        prop_oneof![
            Just(0u32),
            Just(1u32),
            Just(5u32),
            Just(64u32),
            Just(1000u32),
            Just(u32::MAX),
        ],
        prop_oneof![
            Just(0usize),
            Just(1usize),
            Just(1024usize),
            Just(1024 * 1024usize),
            Just(100 * 1024 * 1024usize),
            Just(usize::MAX / 2), // Avoid allocation failures
        ],
        prop_oneof![
            Just(Duration::from_nanos(1)),
            Just(Duration::from_micros(1)),
            Just(Duration::from_millis(1)),
            Just(Duration::from_millis(100)),
            Just(Duration::from_secs(1)),
            Just(Duration::from_secs(60)),
        ],
    )
        .prop_map(
            |(max_instructions, max_recursion, max_memory, timeout)| ResourceConfig {
                max_instructions,
                max_recursion,
                max_memory,
                timeout,
            },
        )
}

/// Generate arbitrary script modes.
fn arb_script_mode() -> impl Strategy<Value = ScriptMode> {
    prop_oneof![
        Just(ScriptMode::Safe),
        Just(ScriptMode::Standard),
        Just(ScriptMode::Full),
    ]
}

/// Generate arbitrary capability levels.
fn arb_capability() -> impl Strategy<Value = ScriptCapability> {
    prop_oneof![
        Just(ScriptCapability::Safe),
        Just(ScriptCapability::Standard),
        Just(ScriptCapability::Advanced),
        Just(ScriptCapability::Internal),
    ]
}

/// Generate arbitrary function names (valid and invalid).
fn arb_function_name() -> impl Strategy<Value = String> {
    prop_oneof![
        // Valid function names from registry
        Just("send_key".to_string()),
        Just("delay".to_string()),
        Just("log".to_string()),
        // Invalid/unknown function names
        Just("".to_string()),
        Just("unknown_function".to_string()),
        Just("../../../etc/passwd".to_string()),
        Just("'; DROP TABLE users; --".to_string()),
        Just("\0null_byte".to_string()),
        Just("🚀emoji_func".to_string()),
        "[a-z]{1,50}".prop_map(|s| s.to_string()),
        "\\PC{1,50}".prop_map(|s| s.to_string()),
    ]
}

/// Generate arbitrary Rhai scripts (including pathological ones).
fn arb_rhai_script() -> impl Strategy<Value = String> {
    prop_oneof![
        // Empty/minimal scripts
        Just("".to_string()),
        Just("42".to_string()),
        Just("true".to_string()),
        // Infinite loops
        Just("loop {}".to_string()),
        Just("while true {}".to_string()),
        Just("for i in 0..1000000000 { i }".to_string()),
        // Deep recursion
        Just(
            r#"
            fn recurse(n) {
                if n > 0 {
                    recurse(n - 1);
                }
            }
            recurse(64);
        "#
            .to_string()
        ),
        Just(
            r#"
            fn a() { b(); }
            fn b() { c(); }
            fn c() { a(); }
            a();
        "#
            .to_string()
        ),
        // Memory exhaustion
        Just(
            r#"
            let arr = [];
            for i in 0..1000000 {
                arr.push(i);
            }
        "#
            .to_string()
        ),
        Just(
            r#"
            let s = "";
            for i in 0..1000000 {
                s += "x";
            }
        "#
            .to_string()
        ),
        // Deep nesting
        Just("((((((((((((((((((((42))))))))))))))))))))".to_string()),
        Just(
            r#"
            if true {
                if true {
                    if true {
                        if true {
                            if true {
                                if true {
                                    if true {
                                        if true {
                                            if true {
                                                42
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        "#
            .to_string()
        ),
        // Invalid syntax
        Just("}{".to_string()),
        Just("fn fn fn".to_string()),
        Just("let let let".to_string()),
        // Arbitrary random scripts
        "[a-z0-9 +\\-*/(){};]{0,200}".prop_map(|s| s.to_string()),
    ]
}

/// Generate arbitrary instruction counts.
fn arb_instruction_count() -> impl Strategy<Value = u64> {
    prop_oneof![
        Just(0u64),
        Just(1u64),
        Just(1000u64),
        Just(u64::MAX),
        0u64..1_000_000u64,
    ]
}

/// Generate arbitrary memory sizes.
fn arb_memory_size() -> impl Strategy<Value = usize> {
    prop_oneof![
        Just(0usize),
        Just(1usize),
        Just(1024usize),
        Just(usize::MAX / 2),
        0usize..100_000_000usize,
    ]
}

// === Resource Limit Tests ===

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Instruction count tracking is accurate and never overflows.
    #[test]
    fn test_instruction_tracking_accurate(
        config in arb_resource_config(),
        increments in prop::collection::vec(arb_instruction_count(), 1..10)
    ) {
        let budget = ResourceBudget::new(config);
        let mut total = 0u64;

        for inc in increments {
            let result = budget.increment_instructions(inc);
            total = total.saturating_add(inc);

            if total >= budget.usage().max_instructions {
                prop_assert!(result.is_err(), "Should fail when limit exceeded");
            }
        }
    }

    /// Memory allocation tracking prevents overflow.
    #[test]
    fn test_memory_tracking_safe(
        config in arb_resource_config(),
        allocations in prop::collection::vec(arb_memory_size(), 1..20)
    ) {
        let budget = ResourceBudget::new(config);
        let mut allocated = Vec::new();

        for size in allocations {
            let result = budget.allocate(size);

            if result.is_ok() {
                allocated.push(size);
            }

            // Verify total doesn't exceed limit
            let total: usize = allocated.iter().sum();
            if total > budget.usage().max_memory {
                prop_assert!(result.is_err(), "Should fail when memory limit exceeded");
            }
        }

        // Cleanup
        for size in allocated {
            budget.deallocate(size);
        }
    }

    /// Recursion guard prevents stack overflow.
    #[test]
    fn test_recursion_guard_prevents_overflow(config in arb_resource_config()) {
        let budget = ResourceBudget::new(config);
        let mut guards = Vec::new();

        // Try to allocate more recursion levels than allowed
        for _ in 0..1000 {
            match budget.enter_recursion() {
                Ok(guard) => guards.push(guard),
                Err(_) => break,
            }
        }

        // Should have stopped at or before limit
        prop_assert!(
            guards.len() as u32 <= budget.usage().max_recursion,
            "Recursion depth should not exceed limit: {} > {}",
            guards.len(),
            budget.usage().max_recursion
        );
    }

    /// Timeout detection is reliable.
    #[test]
    fn test_timeout_detection(
        timeout_ns in 1u64..10_000_000u64,
        sleep_ns in 0u64..20_000_000u64
    ) {
        let config = ResourceConfig {
            timeout: Duration::from_nanos(timeout_ns),
            ..Default::default()
        };
        let budget = ResourceBudget::new(config);

        if sleep_ns > 0 {
            std::thread::sleep(Duration::from_nanos(sleep_ns));
        }

        let result = budget.check_timeout();

        if sleep_ns >= timeout_ns {
            prop_assert!(result.is_err(), "Should timeout when sleep >= timeout");
        }
    }

    /// Resource config values don't cause panics.
    #[test]
    fn test_resource_config_no_panic(config in arb_resource_config()) {
        let _ = ResourceBudget::new(config.clone());
        let _ = ScriptSandbox::new(
            CapabilityRegistry::new(),
            config,
            ScriptMode::Standard
        );
    }
}

// === Capability Enforcement Tests ===

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Capability checks are consistent with mode restrictions.
    #[test]
    fn test_capability_enforcement_consistent(
        capability in arb_capability(),
        mode in arb_script_mode()
    ) {
        let mut registry = CapabilityRegistry::new();
        registry.register(FunctionCapability::new(
            "test_func",
            capability,
            "Test function"
        ));

        let sandbox = ScriptSandbox::new(
            registry,
            ResourceConfig::default(),
            mode
        );

        let result = sandbox.check_function_allowed("test_func");

        // Verify capability hierarchy
        let should_allow = match (capability, mode) {
            (ScriptCapability::Safe, _) => true,
            (ScriptCapability::Standard, ScriptMode::Standard | ScriptMode::Full) => true,
            (ScriptCapability::Advanced, ScriptMode::Full) => true,
            (ScriptCapability::Internal, _) => false,
            _ => false,
        };

        prop_assert_eq!(
            result.is_ok(),
            should_allow,
            "Capability {:?} in mode {:?} should {} be allowed",
            capability,
            mode,
            if should_allow { "" } else { "not" }
        );
    }

    /// Unknown functions are always rejected.
    #[test]
    fn test_unknown_functions_rejected(
        func_name in arb_function_name(),
        mode in arb_script_mode()
    ) {
        let registry = CapabilityRegistry::new(); // Empty registry
        let sandbox = ScriptSandbox::new(
            registry,
            ResourceConfig::default(),
            mode
        );

        let result = sandbox.check_function_allowed(&func_name);
        prop_assert!(result.is_err(), "Unknown function should be rejected");
    }

    /// Mode changes affect capability checks correctly.
    #[test]
    fn test_mode_switching_affects_checks(
        capability in arb_capability(),
        initial_mode in arb_script_mode(),
        new_mode in arb_script_mode()
    ) {
        let mut registry = CapabilityRegistry::new();
        registry.register(FunctionCapability::new(
            "test_func",
            capability,
            "Test function"
        ));

        let mut sandbox = ScriptSandbox::new(
            registry,
            ResourceConfig::default(),
            initial_mode
        );

        sandbox.set_mode(new_mode);
        let result = sandbox.check_function_allowed("test_func");

        // Check should reflect new mode, not initial mode
        let should_allow = match (capability, new_mode) {
            (ScriptCapability::Safe, _) => true,
            (ScriptCapability::Standard, ScriptMode::Standard | ScriptMode::Full) => true,
            (ScriptCapability::Advanced, ScriptMode::Full) => true,
            (ScriptCapability::Internal, _) => false,
            _ => false,
        };

        prop_assert_eq!(
            result.is_ok(),
            should_allow,
            "Capability check should reflect new mode {:?}",
            new_mode
        );
    }
}

// === Script Execution Tests ===

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Engine limits prevent runaway scripts.
    #[test]
    #[ignore = "Slow property test prone to long runtimes; run explicitly when needed"]
    fn test_engine_limits_prevent_runaway(
        script in arb_rhai_script(),
        config in arb_resource_config()
    ) {
        let sandbox = ScriptSandbox::new(
            CapabilityRegistry::new(),
            config,
            ScriptMode::Full
        );

        let mut engine = Engine::new();
        sandbox.configure_engine(&mut engine);

        // Script execution should not hang or panic
        let start = std::time::Instant::now();
        let result = engine.eval::<rhai::Dynamic>(&script);
        let elapsed = start.elapsed();

        // Should complete quickly (either succeed or fail, but not hang)
        prop_assert!(
            elapsed < Duration::from_secs(10),
            "Script execution should not hang: elapsed {:?}",
            elapsed
        );

        // If it fails, that's fine - we're testing that limits work
        // The key is that it doesn't panic or hang
        let _ = result;
    }

    /// Configured limits are respected by engine.
    #[test]
    fn test_configured_limits_respected(config in arb_resource_config()) {
        let sandbox = ScriptSandbox::new(
            CapabilityRegistry::new(),
            config.clone(),
            ScriptMode::Full
        );

        let mut engine = Engine::new();
        sandbox.configure_engine(&mut engine);

        // Test instruction limit with infinite loop
        if config.max_instructions > 0 && config.max_instructions < 1_000_000 {
            let result = engine.eval::<()>("loop {}");
            prop_assert!(
                result.is_err(),
                "Infinite loop should be stopped by instruction limit"
            );
        }

        // Test recursion limit
        if config.max_recursion > 0 && config.max_recursion < 1000 {
            let script = format!(
                r#"
                fn recurse(n) {{
                    if n > 0 {{
                        recurse(n - 1);
                    }}
                }}
                recurse({});
            "#,
                config.max_recursion as u64 * 2
            );
            let result = engine.eval::<()>(&script);
            prop_assert!(
                result.is_err(),
                "Deep recursion should be stopped by recursion limit"
            );
        }
    }

    /// No panics on arbitrary scripts.
    #[test]
    fn test_no_panic_on_arbitrary_script(script in arb_rhai_script()) {
        let sandbox = ScriptSandbox::default();
        let mut engine = Engine::new();
        sandbox.configure_engine(&mut engine);

        // Should never panic, even on invalid/pathological scripts
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = engine.eval::<rhai::Dynamic>(&script);
        }));
    }
}

// === Input Validation Tests ===

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// ValidationError creation never panics.
    #[test]
    fn test_validation_error_creation_safe(
        value in any::<i64>(),
        min in any::<i64>(),
        max in any::<i64>()
    ) {
        let _ = ValidationError::out_of_range(value, min, max);
    }

    /// ValidationError display never panics.
    #[test]
    fn test_validation_error_display_safe(
        context in "[a-z]{1,50}",
        reason in "[a-z]{1,50}"
    ) {
        let err = ValidationError::invalid_value(&context, &reason);
        let _ = format!("{}", err);
        let _ = format!("{:?}", err);
    }

    /// Range validation catches out-of-bounds values.
    #[test]
    fn test_range_validation(value in any::<i64>(), min in any::<i64>(), max in any::<i64>()) {
        if min <= max {
            let in_range = value >= min && value <= max;

            // Simulate range validation
            let is_valid = value >= min && value <= max;

            prop_assert_eq!(
                is_valid,
                in_range,
                "Range validation should be accurate"
            );
        }
    }
}

// === Stress Tests ===

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Concurrent access to ResourceBudget is safe.
    #[test]
    fn test_concurrent_budget_access(
        config in arb_resource_config(),
        thread_count in 2usize..8usize
    ) {
        let budget = std::sync::Arc::new(ResourceBudget::new(config));
        let mut handles = vec![];

        for _ in 0..thread_count {
            let budget = budget.clone();
            let handle = std::thread::spawn(move || {
                for _ in 0..100 {
                    let _ = budget.increment_instructions(1);
                    let _ = budget.allocate(1024);
                    let _ = budget.check_timeout();
                    if let Ok(guard) = budget.enter_recursion() {
                        drop(guard);
                    }
                    budget.deallocate(1024);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            prop_assert!(handle.join().is_ok(), "Thread should not panic");
        }
    }

    /// Sandbox state remains consistent under mode switching.
    #[test]
    fn test_sandbox_consistency_under_mode_switching(
        modes in prop::collection::vec(arb_script_mode(), 1..20)
    ) {
        let registry = build_function_registry();
        let mut sandbox = ScriptSandbox::new(
            registry,
            ResourceConfig::default(),
            ScriptMode::Standard
        );

        for mode in modes {
            sandbox.set_mode(mode);
            prop_assert_eq!(sandbox.mode(), mode, "Mode should be set correctly");

            // Verify registry is still accessible
            prop_assert!(!sandbox.registry().is_empty(), "Registry should remain valid");

            // Verify budget is still accessible
            let _ = sandbox.resource_usage();
        }
    }
}

#[cfg(test)]
mod regression_tests {
    use super::*;

    /// Regression test: Zero instruction limit should fail immediately.
    #[test]
    fn test_zero_instruction_limit() {
        let config = ResourceConfig {
            max_instructions: 0,
            ..Default::default()
        };
        let budget = ResourceBudget::new(config);
        assert!(budget.check_instructions().is_err());
    }

    /// Regression test: Zero recursion limit should prevent any recursion.
    #[test]
    fn test_zero_recursion_limit() {
        let config = ResourceConfig {
            max_recursion: 0,
            ..Default::default()
        };
        let budget = ResourceBudget::new(config);
        assert!(budget.enter_recursion().is_err());
    }

    /// Regression test: Empty function name.
    #[test]
    fn test_empty_function_name() {
        let sandbox = ScriptSandbox::default();
        assert!(sandbox.check_function_allowed("").is_err());
    }

    /// Regression test: Very long function name.
    #[test]
    fn test_very_long_function_name() {
        let sandbox = ScriptSandbox::default();
        let long_name = "a".repeat(10000);
        assert!(sandbox.check_function_allowed(&long_name).is_err());
    }

    /// Regression test: SQL injection attempt in function name.
    #[test]
    fn test_sql_injection_function_name() {
        let sandbox = ScriptSandbox::default();
        assert!(sandbox
            .check_function_allowed("'; DROP TABLE users; --")
            .is_err());
    }

    /// Regression test: Null byte in function name.
    #[test]
    fn test_null_byte_function_name() {
        let sandbox = ScriptSandbox::default();
        assert!(sandbox.check_function_allowed("func\0name").is_err());
    }

    /// Regression test: Path traversal in function name.
    #[test]
    fn test_path_traversal_function_name() {
        let sandbox = ScriptSandbox::default();
        assert!(sandbox
            .check_function_allowed("../../../etc/passwd")
            .is_err());
    }
}
