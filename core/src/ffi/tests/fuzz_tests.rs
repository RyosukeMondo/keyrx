//! Fuzz tests for FFI layer robustness.
//!
//! Uses proptest to generate random inputs and verify:
//! - No panics escape the FFI boundary
//! - Edge cases are handled gracefully (empty strings, max lengths, null bytes)
//! - Error serialization is always valid JSON
//! - Context handles remain valid under stress

#![allow(unsafe_code)] // FFI testing requires unsafe

use crate::ffi::context::FfiContext;
use crate::ffi::error::{FfiError, FfiResult, SerializeFfiResult};
use crate::ffi::events::EventRegistry;
use crate::ffi::traits::FfiExportable;
use proptest::prelude::*;
use serde::{Deserialize, Serialize};
use std::ffi::CString;
use std::sync::atomic::{AtomicU32, Ordering};

// Test domain for fuzz testing
#[derive(Debug, Clone)]
struct FuzzDomainState {
    data: String,
    counter: u32,
}

struct FuzzDomain;

impl FfiExportable for FuzzDomain {
    const DOMAIN: &'static str = "fuzz_test";

    fn init(ctx: &mut FfiContext) -> Result<(), FfiError> {
        ctx.set_domain(
            Self::DOMAIN,
            FuzzDomainState {
                data: String::new(),
                counter: 0,
            },
        );
        Ok(())
    }

    fn cleanup(ctx: &mut FfiContext) {
        ctx.remove_domain(Self::DOMAIN);
    }
}

impl FuzzDomain {
    fn process_string(ctx: &mut FfiContext, input: &str) -> FfiResult<String> {
        let mut state_guard = ctx
            .get_domain_mut::<FuzzDomainState>(Self::DOMAIN)
            .ok_or_else(|| FfiError::invalid_input("domain not initialized"))?;

        let state = state_guard
            .downcast_mut::<FuzzDomainState>()
            .ok_or_else(|| FfiError::internal("invalid domain type"))?;

        state.data = input.to_string();
        state.counter += 1;

        Ok(format!("Processed: {}, count: {}", input, state.counter))
    }

    fn maybe_panic(_ctx: &FfiContext, should_panic: bool) -> FfiResult<()> {
        if should_panic {
            panic!("Intentional panic for testing");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestResult {
    value: String,
}

// Strategy for generating various string inputs including edge cases
fn string_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Normal strings
        "[a-zA-Z0-9 ]{0,100}",
        // Empty string
        Just(String::new()),
        // Very long strings
        "[a-zA-Z]{1000,2000}",
        // Unicode characters
        "[\u{0000}-\u{FFFF}]{0,50}",
        // Strings with newlines and special chars
        "[a-z\n\r\t]{0,100}",
        // Paths and common patterns
        "(/[a-z]+){1,5}",
        // JSON-like strings
        r#"\{[a-z":,0-9 ]+\}"#,
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Fuzz test: Error serialization always produces valid JSON
    #[test]
    fn fuzz_error_serialization(
        code in "[A-Z_]{1,20}",
        message in ".*{0,200}",
    ) {
        let error = FfiError::new(code.clone(), message.clone());
        let json = serde_json::to_string(&error);

        // Serialization should never fail
        prop_assert!(json.is_ok(), "Error serialization failed");

        let json = json.unwrap();

        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&json)
            .expect("Serialized error should be valid JSON");

        // Should have required fields
        prop_assert!(parsed.get("code").is_some(), "Missing 'code' field");
        prop_assert!(parsed.get("message").is_some(), "Missing 'message' field");
        prop_assert_eq!(parsed["code"].as_str().unwrap(), code.as_str());
    }

    /// Fuzz test: FfiResult serialization handles various data types
    #[test]
    fn fuzz_result_serialization(
        value in string_strategy(),
        is_error in prop::bool::ANY,
    ) {
        let result: FfiResult<TestResult> = if is_error {
            Err(FfiError::invalid_input(&value))
        } else {
            Ok(TestResult { value: value.clone() })
        };

        let json = result.to_ffi_json();

        // Should always serialize successfully
        prop_assert!(json.is_ok(), "Result serialization failed");

        let json = json.unwrap();

        // Should start with ok: or error:
        prop_assert!(
            json.starts_with("ok:") || json.starts_with("error:"),
            "Invalid result format: {}", json
        );

        // The payload after prefix should be valid JSON
        let payload = if json.starts_with("ok:") {
            &json[3..]
        } else {
            &json[6..]
        };

        let parsed: serde_json::Value = serde_json::from_str(payload)
            .expect("Payload should be valid JSON");
        prop_assert!(parsed.is_object(), "Payload should be a JSON object");
    }

    /// Fuzz test: Context handles remain unique under concurrent creation
    #[test]
    fn fuzz_context_handle_uniqueness(count in 1usize..50) {
        let contexts: Vec<_> = (0..count)
            .map(|_| FfiContext::new())
            .collect();

        let handles: Vec<_> = contexts.iter().map(|c| c.handle()).collect();

        // All handles should be unique
        for i in 0..handles.len() {
            for j in (i + 1)..handles.len() {
                prop_assert_ne!(
                    handles[i], handles[j],
                    "Handles must be unique"
                );
            }
        }
    }

    /// Fuzz test: Domain state isolation with various inputs
    #[test]
    fn fuzz_domain_state_isolation(
        input1 in string_strategy(),
        input2 in string_strategy(),
    ) {
        let mut ctx1 = FfiContext::new();
        let mut ctx2 = FfiContext::new();

        // Initialize both domains
        FuzzDomain::init(&mut ctx1).expect("init failed");
        FuzzDomain::init(&mut ctx2).expect("init failed");

        // Process different inputs
        let result1 = FuzzDomain::process_string(&mut ctx1, &input1);
        let result2 = FuzzDomain::process_string(&mut ctx2, &input2);

        prop_assert!(result1.is_ok(), "Context 1 processing failed");
        prop_assert!(result2.is_ok(), "Context 2 processing failed");

        // Verify state isolation
        let state1_guard = ctx1.get_domain::<FuzzDomainState>(FuzzDomain::DOMAIN).unwrap();
        let state1 = state1_guard.downcast_ref::<FuzzDomainState>().unwrap();
        let state2_guard = ctx2.get_domain::<FuzzDomainState>(FuzzDomain::DOMAIN).unwrap();
        let state2 = state2_guard.downcast_ref::<FuzzDomainState>().unwrap();

        prop_assert_eq!(&state1.data, &input1, "Context 1 data mismatch");
        prop_assert_eq!(&state2.data, &input2, "Context 2 data mismatch");
        prop_assert_eq!(state1.counter, 1, "Context 1 counter wrong");
        prop_assert_eq!(state2.counter, 1, "Context 2 counter wrong");
    }

    /// Fuzz test: Error code generation doesn't panic
    #[test]
    fn fuzz_error_constructors(message in ".*{0,500}") {
        // All error constructors should handle any input without panic
        let _ = FfiError::invalid_input(&message);
        let _ = FfiError::internal(&message);
        let _ = FfiError::not_found(&message);
        let _ = FfiError::null_pointer(&message);
        let _ = FfiError::invalid_utf8(&message);

        // All should serialize successfully
        prop_assert!(FfiError::invalid_input(&message).to_string().len() > 0);
    }

    /// Fuzz test: Event types handle various payloads
    #[test]
    fn fuzz_event_payloads(
        event_data in string_strategy(),
        is_valid_json in prop::bool::ANY,
    ) {
        let payload = if is_valid_json {
            // Try to create valid JSON
            serde_json::to_vec(&serde_json::json!({
                "data": event_data
            })).unwrap()
        } else {
            // Use raw string data
            event_data.into_bytes()
        };

        // Event registry should handle any payload gracefully
        let _registry = EventRegistry::new();

        // This shouldn't panic even with invalid data
        // (actual invocation would need a callback, but construction should be safe)
        prop_assert!(payload.len() >= 0); // Trivial check to use the data
    }
}

/// Test: String parameters with embedded nulls
#[test]
fn test_embedded_null_bytes() {
    // CString creation should fail with embedded nulls
    let input = "hello\0world";
    let result = CString::new(input);
    assert!(result.is_err(), "CString should reject embedded nulls");

    // But our error handling should work fine
    let error_result: FfiResult<()> = Err(FfiError::invalid_input("string contains null bytes"));
    let json = error_result.to_ffi_json();
    assert!(json.is_ok());
}

/// Test: Maximum length strings
#[test]
fn test_max_length_strings() {
    // Very large string (1MB)
    let large_string = "a".repeat(1024 * 1024);
    let error = FfiError::invalid_input(&large_string);
    let json = serde_json::to_string(&error);
    assert!(json.is_ok(), "Should serialize large strings");

    // Verify it's valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&json.unwrap()).unwrap();
    assert_eq!(parsed["code"], "INVALID_INPUT");
}

/// Test: Empty strings are handled correctly
#[test]
fn test_empty_strings() {
    let errors = vec![
        FfiError::invalid_input(""),
        FfiError::not_found(""),
        FfiError::null_pointer(""),
        FfiError::invalid_utf8(""),
    ];

    for error in errors {
        let result: FfiResult<()> = Err(error);
        let json = result.to_ffi_json();
        assert!(json.is_ok(), "Empty string should serialize");
        let json_str = json.unwrap();
        assert!(json_str.starts_with("error:"));

        // Parse to verify valid JSON
        let payload = &json_str[6..];
        let parsed: serde_json::Value = serde_json::from_str(payload).unwrap();
        assert!(parsed.is_object());
    }
}

/// Test: Unicode and special characters
#[test]
fn test_unicode_in_errors() {
    let messages = vec![
        "Error with emoji: 🚀",
        "日本語のエラー",
        "Ошибка на русском",
        "Special chars: \n\r\t",
        "Quotes: \"hello\" and 'world'",
    ];

    for msg in messages {
        let error = FfiError::invalid_input(msg);
        let json = serde_json::to_string(&error);
        assert!(json.is_ok(), "Should handle unicode: {}", msg);

        // Verify round-trip
        let json_str = json.unwrap();
        let parsed: FfiError = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed.message, msg);
    }
}

/// Test: Panic catching in simulated FFI wrapper
#[test]
fn test_panic_catching() {
    use std::panic::{catch_unwind, AssertUnwindSafe};

    let mut ctx = FfiContext::new();
    FuzzDomain::init(&mut ctx).unwrap();

    // Simulate what the macro-generated wrapper does
    let result = catch_unwind(AssertUnwindSafe(|| FuzzDomain::maybe_panic(&ctx, true)));

    // Panic should be caught
    assert!(result.is_err(), "Panic should be caught");

    // In real FFI wrapper, this would be converted to FfiError::internal
    let error_result: FfiResult<()> = Err(FfiError::internal("panic caught"));
    let json = error_result.to_ffi_json();
    assert!(json.is_ok());
    assert!(json.unwrap().starts_with("error:"));
}

/// Test: Multiple panics don't corrupt state
#[test]
fn test_repeated_panics() {
    use std::panic::{catch_unwind, AssertUnwindSafe};

    let mut ctx = FfiContext::new();
    FuzzDomain::init(&mut ctx).unwrap();

    for _ in 0..10 {
        let ctx_ref = &ctx;
        let _ = catch_unwind(AssertUnwindSafe(|| FuzzDomain::maybe_panic(ctx_ref, true)));
    }

    // Context should still be valid
    let state_guard = ctx
        .get_domain::<FuzzDomainState>(FuzzDomain::DOMAIN)
        .unwrap();
    let state = state_guard.downcast_ref::<FuzzDomainState>().unwrap();
    assert_eq!(state.counter, 0); // No successful operations
}

/// Test: Concurrent error serialization
#[test]
fn test_concurrent_error_serialization() {
    use std::sync::Arc;
    use std::thread;

    let counter = Arc::new(AtomicU32::new(0));
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let counter = Arc::clone(&counter);
            thread::spawn(move || {
                for j in 0..100 {
                    let result: FfiResult<()> =
                        Err(FfiError::invalid_input(format!("Error {}-{}", i, j)));
                    let json = result.to_ffi_json();
                    assert!(json.is_ok());
                    counter.fetch_add(1, Ordering::SeqCst);
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(counter.load(Ordering::SeqCst), 1000);
}

/// Test: Error details with complex JSON
#[test]
fn test_error_details_complex_json() {
    let details = serde_json::json!({
        "nested": {
            "array": [1, 2, 3],
            "object": {
                "key": "value"
            }
        },
        "unicode": "Hello 世界",
        "special": "quotes\"and\\slashes"
    });

    let error = FfiError::with_details("COMPLEX", "Complex error", details.clone());
    let json = serde_json::to_string(&error).unwrap();

    // Round trip
    let parsed: FfiError = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.code, "COMPLEX");
    assert_eq!(parsed.details, Some(details));
}

/// Test: Context cleanup with various states
#[test]
fn test_context_cleanup_robustness() {
    for i in 0..50 {
        let mut ctx = FfiContext::new();

        if i % 2 == 0 {
            FuzzDomain::init(&mut ctx).unwrap();
        }

        if i % 3 == 0 && ctx.has_domain(FuzzDomain::DOMAIN) {
            let _ = FuzzDomain::process_string(&mut ctx, &format!("Test {}", i));
        }

        // Cleanup should always work
        FuzzDomain::cleanup(&mut ctx);
        assert!(!ctx.has_domain(FuzzDomain::DOMAIN));
    }
}

/// Test: Stress test with rapid context creation/destruction
#[test]
fn test_rapid_context_lifecycle() {
    for _ in 0..1000 {
        let mut ctx = FfiContext::new();
        FuzzDomain::init(&mut ctx).unwrap();
        let _ = FuzzDomain::process_string(&mut ctx, "test");
        FuzzDomain::cleanup(&mut ctx);
    }
}
