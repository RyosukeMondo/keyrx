//! Parallel FFI tests to verify isolated state management.
//!
//! These tests validate that multiple FFI contexts can run concurrently
//! without state interference, ensuring thread-safe operation and
//! proper isolation of domain state.
//!
//! Run with cargo nextest for true parallel execution:
//! ```bash
//! cargo nextest run --package keyrx_core parallel_tests
//! ```

#![allow(unsafe_code)] // Test callbacks require unsafe

use crate::ffi::context::{context_registry, FfiContext, FfiContextRegistry};
use crate::ffi::error::FfiError;
use crate::ffi::events::{EventRegistry, EventType};
use crate::ffi::traits::FfiExportable;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// Test domain state
#[derive(Debug, Clone)]
struct TestDomainState {
    counter: u32,
    name: String,
}

// Mock domain for testing
struct TestDomain;

impl FfiExportable for TestDomain {
    const DOMAIN: &'static str = "test_domain";

    fn init(ctx: &mut FfiContext) -> Result<(), FfiError> {
        if ctx.has_domain(Self::DOMAIN) {
            return Err(FfiError::invalid_input("domain already initialized"));
        }
        ctx.set_domain(
            Self::DOMAIN,
            TestDomainState {
                counter: 0,
                name: "initial".to_string(),
            },
        );
        Ok(())
    }

    fn cleanup(ctx: &mut FfiContext) {
        ctx.remove_domain(Self::DOMAIN);
    }
}

#[test]
fn test_parallel_context_creation() {
    // Create multiple contexts in parallel
    let handles: Vec<_> = (0..10)
        .map(|i| {
            thread::spawn(move || {
                let ctx = FfiContext::new();
                let handle = ctx.handle();
                // Each context should have a unique handle
                assert!(handle > 0);
                (i, handle)
            })
        })
        .collect();

    let mut all_handles = Vec::new();
    for handle in handles {
        let (_, h) = handle.join().unwrap();
        all_handles.push(h);
    }

    // All handles should be unique
    for i in 0..all_handles.len() {
        for j in (i + 1)..all_handles.len() {
            assert_ne!(all_handles[i], all_handles[j], "Handles should be unique");
        }
    }
}

#[test]
fn test_parallel_domain_state_isolation() {
    // Create multiple contexts and verify state isolation
    let registry = Arc::new(FfiContextRegistry::new());
    let handles: Vec<_> = (0..5)
        .map(|i| {
            let mut ctx = FfiContext::new();
            TestDomain::init(&mut ctx).unwrap();

            // Set unique state for each context
            {
                let mut guard = ctx
                    .get_domain_mut::<TestDomainState>("test_domain")
                    .unwrap();
                let state = guard.downcast_mut::<TestDomainState>().unwrap();
                state.counter = i * 10;
                state.name = format!("context_{}", i);
            }

            let handle = registry.register(ctx);
            (i, handle)
        })
        .collect();

    // Verify each context has its own isolated state (parallel reads)
    let verification_handles: Vec<_> = handles
        .iter()
        .map(|(expected_id, handle)| {
            let handle = *handle;
            let expected_id = *expected_id;
            let reg_clone = Arc::clone(&registry);

            thread::spawn(move || {
                let ctx_arc = reg_clone.get(handle).unwrap();
                let ctx = ctx_arc.read().unwrap();

                let guard = ctx.get_domain::<TestDomainState>("test_domain").unwrap();
                let state = guard.downcast_ref::<TestDomainState>().unwrap();

                assert_eq!(state.counter, expected_id * 10);
                assert_eq!(state.name, format!("context_{}", expected_id));
            })
        })
        .collect();

    for h in verification_handles {
        h.join().unwrap();
    }

    // Clean up
    for (_, handle) in handles {
        registry.unregister(handle);
    }
}

#[test]
fn test_parallel_domain_mutations() {
    // Test that concurrent mutations to different contexts don't interfere
    let registry = Arc::new(FfiContextRegistry::new());
    let counter = Arc::new(AtomicU32::new(0));

    let contexts: Vec<_> = (0..10)
        .map(|_| {
            let mut ctx = FfiContext::new();
            TestDomain::init(&mut ctx).unwrap();
            registry.register(ctx)
        })
        .collect();

    // Spawn threads that concurrently mutate their own contexts
    let mutation_handles: Vec<_> = contexts
        .iter()
        .enumerate()
        .map(|(idx, handle)| {
            let handle = *handle;
            let counter_clone = Arc::clone(&counter);
            let reg = Arc::clone(&registry);

            thread::spawn(move || {
                for i in 0..100 {
                    let ctx_arc = reg.get(handle).unwrap();
                    let ctx = ctx_arc.read().unwrap();

                    // Mutate domain state
                    let mut guard = ctx
                        .get_domain_mut::<TestDomainState>("test_domain")
                        .unwrap();
                    let state = guard.downcast_mut::<TestDomainState>().unwrap();
                    state.counter += 1;
                    state.name = format!("ctx_{}_{}", idx, i);

                    counter_clone.fetch_add(1, Ordering::SeqCst);

                    // Small delay to increase chance of race conditions
                    thread::sleep(Duration::from_micros(1));
                }
            })
        })
        .collect();

    // Wait for all mutations to complete
    for h in mutation_handles {
        h.join().unwrap();
    }

    // Verify total number of mutations
    assert_eq!(counter.load(Ordering::SeqCst), 1000);

    // Verify each context has expected final state
    for (idx, handle) in contexts.iter().enumerate() {
        let ctx_arc = registry.get(*handle).unwrap();
        let ctx = ctx_arc.read().unwrap();

        let guard = ctx.get_domain::<TestDomainState>("test_domain").unwrap();
        let state = guard.downcast_ref::<TestDomainState>().unwrap();

        assert_eq!(state.counter, 100);
        assert_eq!(state.name, format!("ctx_{}_{}", idx, 99));
    }

    // Clean up
    for handle in contexts {
        registry.unregister(handle);
    }
}

#[test]
fn test_parallel_registry_operations() {
    // Test concurrent registry operations
    let registered = Arc::new(Mutex::new(Vec::new()));

    // Register many contexts in parallel
    let registration_handles: Vec<_> = (0..20)
        .map(|_| {
            let reg_clone = Arc::clone(&registered);
            thread::spawn(move || {
                let ctx = FfiContext::new();
                let handle = ctx.handle();
                context_registry().register(ctx);

                reg_clone.lock().unwrap().push(handle);
                handle
            })
        })
        .collect();

    let mut all_handles = Vec::new();
    for h in registration_handles {
        all_handles.push(h.join().unwrap());
    }

    // Verify all contexts are registered
    assert!(context_registry().count() >= 20);

    // Concurrently access registered contexts
    let access_handles: Vec<_> = all_handles
        .iter()
        .map(|handle| {
            let handle = *handle;
            thread::spawn(move || {
                let ctx = context_registry().get(handle);
                assert!(ctx.is_some());
            })
        })
        .collect();

    for h in access_handles {
        h.join().unwrap();
    }

    // Concurrently unregister contexts
    let unregister_handles: Vec<_> = all_handles
        .iter()
        .map(|handle| {
            let handle = *handle;
            thread::spawn(move || {
                let result = context_registry().unregister(handle);
                assert!(result, "Should successfully unregister");
            })
        })
        .collect();

    for h in unregister_handles {
        h.join().unwrap();
    }

    // Verify all contexts are unregistered
    // Note: We can't assert count == 0 because other tests may be running in parallel
    // and using the global registry. Instead, verify none of our handles exist.
    for handle in &all_handles {
        assert!(context_registry().get(*handle).is_none());
    }
}

#[test]
fn test_parallel_init_and_cleanup() {
    // Test parallel initialization and cleanup of domains
    let handles: Vec<_> = (0..10)
        .map(|i| {
            thread::spawn(move || {
                let mut ctx = FfiContext::new();

                // Initialize domain
                TestDomain::init(&mut ctx).unwrap();
                assert!(ctx.has_domain("test_domain"));

                // Modify state
                {
                    let mut guard = ctx
                        .get_domain_mut::<TestDomainState>("test_domain")
                        .unwrap();
                    let state = guard.downcast_mut::<TestDomainState>().unwrap();
                    state.counter = i;
                }

                // Clean up
                TestDomain::cleanup(&mut ctx);
                assert!(!ctx.has_domain("test_domain"));

                // Re-initialize should work
                TestDomain::init(&mut ctx).unwrap();
                assert!(ctx.has_domain("test_domain"));

                // State should be reset
                let guard = ctx.get_domain::<TestDomainState>("test_domain").unwrap();
                let state = guard.downcast_ref::<TestDomainState>().unwrap();
                assert_eq!(state.counter, 0);
                assert_eq!(state.name, "initial");
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }
}

#[test]
fn test_parallel_event_registry() {
    // Test concurrent event registration and invocation
    let registry = EventRegistry::new();
    let call_count = Arc::new(AtomicU32::new(0));

    // Create test callbacks
    unsafe extern "C" fn test_callback_1(_data: *const u8, _len: usize) {}
    unsafe extern "C" fn test_callback_2(_data: *const u8, _len: usize) {}
    unsafe extern "C" fn test_callback_3(_data: *const u8, _len: usize) {}

    // Register multiple callbacks in parallel
    let registration_handles: Vec<_> = (0..5)
        .map(|i| {
            let reg_clone = registry.clone();
            thread::spawn(move || {
                // Register different event types
                let (event_type, callback) = match i % 3 {
                    0 => (
                        EventType::DiscoveryProgress,
                        test_callback_1 as unsafe extern "C" fn(*const u8, usize),
                    ),
                    1 => (
                        EventType::ValidationProgress,
                        test_callback_2 as unsafe extern "C" fn(*const u8, usize),
                    ),
                    _ => (
                        EventType::EngineState,
                        test_callback_3 as unsafe extern "C" fn(*const u8, usize),
                    ),
                };

                reg_clone.register(event_type, Some(callback));
            })
        })
        .collect();

    for h in registration_handles {
        h.join().unwrap();
    }

    // Verify registrations
    assert!(registry.is_registered(EventType::DiscoveryProgress));
    assert!(registry.is_registered(EventType::ValidationProgress));
    assert!(registry.is_registered(EventType::EngineState));

    // Invoke callbacks in parallel
    let invocation_handles: Vec<_> = (0..5)
        .map(|i| {
            let reg_clone = registry.clone();
            let count_clone = Arc::clone(&call_count);
            thread::spawn(move || {
                let event_type = match i % 3 {
                    0 => EventType::DiscoveryProgress,
                    1 => EventType::ValidationProgress,
                    _ => EventType::EngineState,
                };

                let data = serde_json::json!({"test": "data"});
                reg_clone.invoke(event_type, &data);
                count_clone.fetch_add(1, Ordering::SeqCst);

                // Small delay
                thread::sleep(Duration::from_micros(10));
            })
        })
        .collect();

    for h in invocation_handles {
        h.join().unwrap();
    }

    // Verify invocations completed
    let total_calls = call_count.load(Ordering::SeqCst);
    assert_eq!(total_calls, 5, "All invocations should complete");
}

#[test]
fn test_no_cross_context_interference() {
    // Comprehensive test: multiple contexts, multiple domains, concurrent operations
    let registry = Arc::new(FfiContextRegistry::new());

    // Create contexts with unique IDs
    let context_ids: Vec<_> = (0..5)
        .map(|i| {
            let mut ctx = FfiContext::new();
            TestDomain::init(&mut ctx).unwrap();

            // Set unique identifier in state
            {
                let mut guard = ctx
                    .get_domain_mut::<TestDomainState>("test_domain")
                    .unwrap();
                let state = guard.downcast_mut::<TestDomainState>().unwrap();
                state.counter = i * 1000; // Unique base value
                state.name = format!("context_{}", i);
            }

            let handle = registry.register(ctx);
            (i, handle)
        })
        .collect();

    // Spawn threads that repeatedly read and verify their context's state
    let verification_handles: Vec<_> = context_ids
        .iter()
        .map(|(id, handle)| {
            let id = *id;
            let handle = *handle;
            let reg = Arc::clone(&registry);

            thread::spawn(move || {
                for iteration in 0..50 {
                    let ctx_arc = reg.get(handle).unwrap();
                    let ctx = ctx_arc.read().unwrap();

                    // Read state
                    let guard = ctx.get_domain::<TestDomainState>("test_domain").unwrap();
                    let state = guard.downcast_ref::<TestDomainState>().unwrap();

                    // Verify state hasn't been corrupted by other contexts
                    assert_eq!(
                        state.counter,
                        id * 1000,
                        "Context {} state corrupted at iteration {}",
                        id,
                        iteration
                    );
                    assert_eq!(
                        state.name,
                        format!("context_{}", id),
                        "Context {} name corrupted at iteration {}",
                        id,
                        iteration
                    );

                    // Small delay to allow interleaving
                    thread::sleep(Duration::from_micros(1));
                }
            })
        })
        .collect();

    // Wait for all verifications
    for h in verification_handles {
        h.join().unwrap();
    }

    // Clean up
    for (_, handle) in context_ids {
        registry.unregister(handle);
    }
}

#[test]
fn test_parallel_context_cleanup() {
    // Test that cleanup of one context doesn't affect others
    let registry = Arc::new(FfiContextRegistry::new());

    let contexts: Vec<_> = (0..10)
        .map(|i| {
            let mut ctx = FfiContext::new();
            TestDomain::init(&mut ctx).unwrap();

            {
                let mut guard = ctx
                    .get_domain_mut::<TestDomainState>("test_domain")
                    .unwrap();
                let state = guard.downcast_mut::<TestDomainState>().unwrap();
                state.counter = i;
            }

            registry.register(ctx)
        })
        .collect();

    // Clean up half the contexts in parallel
    let cleanup_handles: Vec<_> = contexts[0..5]
        .iter()
        .map(|handle| {
            let handle = *handle;
            let reg = Arc::clone(&registry);
            thread::spawn(move || {
                let ctx_arc = reg.get(handle).unwrap();
                let mut ctx = ctx_arc.write().unwrap();
                TestDomain::cleanup(&mut ctx);
            })
        })
        .collect();

    for h in cleanup_handles {
        h.join().unwrap();
    }

    // Verify remaining contexts still have their state
    let verification_handles: Vec<_> = contexts[5..]
        .iter()
        .enumerate()
        .map(|(idx, handle)| {
            let handle = *handle;
            let expected_counter = 5 + idx as u32;
            let reg = Arc::clone(&registry);

            thread::spawn(move || {
                let ctx_arc = reg.get(handle).unwrap();
                let ctx = ctx_arc.read().unwrap();

                let guard = ctx.get_domain::<TestDomainState>("test_domain").unwrap();
                let state = guard.downcast_ref::<TestDomainState>().unwrap();
                assert_eq!(state.counter, expected_counter);
            })
        })
        .collect();

    for h in verification_handles {
        h.join().unwrap();
    }

    // Clean up all
    for handle in contexts {
        registry.unregister(handle);
    }
}

#[test]
fn test_stress_parallel_operations() {
    // Stress test with many concurrent operations
    let operations = Arc::new(AtomicU32::new(0));

    let handles: Vec<_> = (0..20)
        .map(|i| {
            let ops_clone = Arc::clone(&operations);
            thread::spawn(move || {
                // Create context
                let mut ctx = FfiContext::new();
                TestDomain::init(&mut ctx).unwrap();

                // Perform many state updates
                for j in 0..100 {
                    let mut guard = ctx
                        .get_domain_mut::<TestDomainState>("test_domain")
                        .unwrap();
                    let state = guard.downcast_mut::<TestDomainState>().unwrap();
                    state.counter = j;
                    state.name = format!("thread_{}_iter_{}", i, j);
                    ops_clone.fetch_add(1, Ordering::SeqCst);
                }

                // Verify final state
                {
                    let guard = ctx.get_domain::<TestDomainState>("test_domain").unwrap();
                    let state = guard.downcast_ref::<TestDomainState>().unwrap();
                    assert_eq!(state.counter, 99);
                }

                TestDomain::cleanup(&mut ctx);
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    // Verify all operations completed
    assert_eq!(operations.load(Ordering::SeqCst), 2000);
}
