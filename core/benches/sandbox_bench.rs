//! Sandbox overhead benchmarks.
//!
//! This benchmark suite verifies that sandbox security checks meet performance targets:
//! - Capability checks: < 10 nanoseconds (target overhead)
//! - Input validation: < 100 nanoseconds per check
//! - Resource tracking: < 50 nanoseconds per operation
//! - Combined overhead: < 200 nanoseconds total

// Allow unwrap/expect in benchmarks - panics are acceptable for setup code
#![allow(clippy::unwrap_used, clippy::expect_used)]

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use keyrx_core::drivers::keycodes::KeyCode;
use keyrx_core::scripting::sandbox::validation::InputValidator;
use keyrx_core::scripting::sandbox::validators::{
    KeyCodeValidator, LengthValidator, NonEmptyValidator, NonNegativeValidator, PatternValidator,
    PositiveValidator, RangeValidator,
};
use keyrx_core::scripting::sandbox::{
    build_function_registry, ResourceBudget, ResourceConfig, ScriptMode, ScriptSandbox,
};

// ===== Capability Check Benchmarks =====

/// Benchmark O(1) function capability lookup.
///
/// Target: < 10 nanoseconds per lookup
fn benchmark_capability_lookup(c: &mut Criterion) {
    let registry = build_function_registry();

    c.bench_function("capability_lookup_existing", |b| {
        b.iter(|| {
            let _ = registry.get(black_box("send_key"));
        })
    });
}

/// Benchmark capability check with mode validation.
///
/// Target: < 10 nanoseconds (includes lookup + comparison)
fn benchmark_capability_check(c: &mut Criterion) {
    let registry = build_function_registry();

    c.bench_function("capability_check_allowed", |b| {
        b.iter(|| {
            let _ = registry.is_allowed(black_box("send_key"), black_box(ScriptMode::Standard));
        })
    });
}

/// Benchmark capability check for denied function.
///
/// Target: < 10 nanoseconds (should be same as allowed case)
fn benchmark_capability_check_denied(c: &mut Criterion) {
    let registry = build_function_registry();

    c.bench_function("capability_check_denied", |b| {
        b.iter(|| {
            let _ = registry.is_allowed(black_box("clipboard_get"), black_box(ScriptMode::Safe));
        })
    });
}

/// Benchmark KeyCode-based function lookup.
///
/// Target: < 10 nanoseconds (HashMap lookup)
fn benchmark_keycode_lookup(c: &mut Criterion) {
    let registry = build_function_registry();

    c.bench_function("keycode_lookup", |b| {
        b.iter(|| {
            let _ = registry.for_keycode(black_box(KeyCode::A));
        })
    });
}

/// Benchmark capability registry creation and population.
///
/// This is not a hot path, but should complete quickly during initialization.
fn benchmark_registry_creation(c: &mut Criterion) {
    c.bench_function("registry_creation", |b| {
        b.iter(|| {
            black_box(build_function_registry());
        })
    });
}

/// Benchmark multiple capability checks in sequence.
///
/// Target: < 50 nanoseconds total (5 checks × 10ns each)
fn benchmark_multiple_capability_checks(c: &mut Criterion) {
    let registry = build_function_registry();

    c.bench_function("multiple_capability_checks", |b| {
        b.iter(|| {
            let _ = registry.is_allowed("send_key", ScriptMode::Standard);
            let _ = registry.is_allowed("delay", ScriptMode::Standard);
            let _ = registry.is_allowed("print", ScriptMode::Standard);
            let _ = registry.is_allowed("get_keycode", ScriptMode::Standard);
            let _ = registry.is_allowed("is_pressed", ScriptMode::Standard);
        })
    });
}

// ===== Input Validation Benchmarks =====

/// Benchmark range validation for integers.
///
/// Target: < 10 nanoseconds (simple comparison)
fn benchmark_range_validation_i64(c: &mut Criterion) {
    let validator = RangeValidator::new(0i64, 1000i64);

    c.bench_function("range_validation_i64_valid", |b| {
        b.iter(|| {
            let _ = validator.validate(black_box(&500));
        })
    });
}

/// Benchmark range validation failure.
///
/// Target: < 10 nanoseconds (comparison + error construction)
fn benchmark_range_validation_invalid(c: &mut Criterion) {
    let validator = RangeValidator::new(0i64, 1000i64);

    c.bench_function("range_validation_i64_invalid", |b| {
        b.iter(|| {
            let _ = validator.validate(black_box(&1500));
        })
    });
}

/// Benchmark non-negative validation.
///
/// Target: < 5 nanoseconds (single comparison)
fn benchmark_non_negative_validation(c: &mut Criterion) {
    let validator = NonNegativeValidator;

    c.bench_function("non_negative_validation", |b| {
        b.iter(|| {
            let _ = validator.validate(black_box(&42i64));
        })
    });
}

/// Benchmark positive validation.
///
/// Target: < 5 nanoseconds (single comparison)
fn benchmark_positive_validation(c: &mut Criterion) {
    let validator = PositiveValidator;

    c.bench_function("positive_validation", |b| {
        b.iter(|| {
            let _ = validator.validate(black_box(&42i64));
        })
    });
}

/// Benchmark string length validation.
///
/// Target: < 10 nanoseconds (length check + comparison)
fn benchmark_string_length_validation(c: &mut Criterion) {
    let validator = LengthValidator::new(1, 100);
    let test_str = "Hello, World!";

    c.bench_function("string_length_validation", |b| {
        b.iter(|| {
            let _ = validator.validate(black_box(test_str));
        })
    });
}

/// Benchmark non-empty string validation.
///
/// Target: < 5 nanoseconds (is_empty check)
fn benchmark_non_empty_validation(c: &mut Criterion) {
    let validator = NonEmptyValidator;
    let test_str = "Hello";

    c.bench_function("non_empty_validation", |b| {
        b.iter(|| {
            let _ = validator.validate(black_box(test_str));
        })
    });
}

/// Benchmark pattern (regex) validation.
///
/// Target: < 50 nanoseconds (regex match is more expensive)
fn benchmark_pattern_validation(c: &mut Criterion) {
    let validator = PatternValidator::new(r"^[a-zA-Z0-9_]+$").unwrap();
    let test_str = "valid_identifier_123";

    c.bench_function("pattern_validation", |b| {
        b.iter(|| {
            let _ = validator.validate(black_box(test_str));
        })
    });
}

/// Benchmark KeyCode validation.
///
/// Target: < 50 nanoseconds (string parsing)
fn benchmark_keycode_validation(c: &mut Criterion) {
    let validator = KeyCodeValidator;

    c.bench_function("keycode_validation", |b| {
        b.iter(|| {
            let _ = validator.validate(black_box("A"));
        })
    });
}

/// Benchmark multiple validators chained together.
///
/// Target: < 50 nanoseconds (3 validations)
fn benchmark_chained_validation(c: &mut Criterion) {
    let range = RangeValidator::new(0i64, 1000i64);
    let positive = PositiveValidator;

    c.bench_function("chained_validation", |b| {
        b.iter(|| {
            let value = black_box(&42i64);
            let _ = range.validate(value).and_then(|_| positive.validate(value));
        })
    });
}

/// Benchmark validation with string allocation on error.
///
/// This tests the cost of error path (validation failure with error message).
fn benchmark_validation_error_path(c: &mut Criterion) {
    let validator = RangeValidator::new(0i64, 100i64);

    c.bench_function("validation_error_path", |b| {
        b.iter(|| {
            // This will fail and create an error with formatted strings
            let result = validator.validate(black_box(&500));
            let _ = black_box(result);
        })
    });
}

// ===== Resource Tracking Benchmarks =====

/// Benchmark instruction counting.
///
/// Target: < 10 nanoseconds (atomic fetch_add)
fn benchmark_instruction_increment(c: &mut Criterion) {
    let budget = ResourceBudget::new(ResourceConfig::default());

    c.bench_function("instruction_increment", |b| {
        b.iter(|| {
            let _ = budget.increment_instructions(black_box(1));
        })
    });
}

/// Benchmark instruction limit check.
///
/// Target: < 5 nanoseconds (atomic load + comparison)
fn benchmark_instruction_check(c: &mut Criterion) {
    let budget = ResourceBudget::new(ResourceConfig::default());

    c.bench_function("instruction_check", |b| {
        b.iter(|| {
            let _ = budget.check_instructions();
        })
    });
}

/// Benchmark recursion guard creation and drop.
///
/// Target: < 20 nanoseconds (atomic increment + RAII decrement)
fn benchmark_recursion_guard(c: &mut Criterion) {
    let budget = ResourceBudget::new(ResourceConfig::default());

    c.bench_function("recursion_guard", |b| {
        b.iter(|| {
            let _guard = budget.enter_recursion().unwrap();
            // Guard drops here, decrementing depth
        })
    });
}

/// Benchmark memory allocation tracking.
///
/// Target: < 15 nanoseconds (atomic fetch_add + check)
fn benchmark_memory_allocation(c: &mut Criterion) {
    let budget = ResourceBudget::new(ResourceConfig::default());

    c.bench_function("memory_allocation", |b| {
        b.iter(|| {
            let _ = budget.allocate(black_box(1024));
        })
    });
}

/// Benchmark timeout check.
///
/// Target: < 20 nanoseconds (Instant comparison)
fn benchmark_timeout_check(c: &mut Criterion) {
    let budget = ResourceBudget::new(ResourceConfig::default());

    c.bench_function("timeout_check", |b| {
        b.iter(|| {
            let _ = budget.check_timeout();
        })
    });
}

/// Benchmark resource usage snapshot.
///
/// Target: < 50 nanoseconds (4 atomic loads)
fn benchmark_resource_usage_snapshot(c: &mut Criterion) {
    let budget = ResourceBudget::new(ResourceConfig::default());

    // Pre-populate with some usage
    let _ = budget.increment_instructions(1000);
    let _ = budget.allocate(4096);

    c.bench_function("resource_usage_snapshot", |b| {
        b.iter(|| {
            black_box(budget.usage());
        })
    });
}

/// Benchmark all resource checks combined.
///
/// Target: < 50 nanoseconds (multiple atomic operations)
fn benchmark_all_resource_checks(c: &mut Criterion) {
    let budget = ResourceBudget::new(ResourceConfig::default());

    c.bench_function("all_resource_checks", |b| {
        b.iter(|| {
            let _ = budget.check_instructions();
            let _ = budget.check_timeout();
        })
    });
}

// ===== Sandbox Integration Benchmarks =====

/// Benchmark full sandbox check for allowed function.
///
/// Target: < 20 nanoseconds (capability check + mode validation)
fn benchmark_sandbox_check_allowed(c: &mut Criterion) {
    let sandbox = ScriptSandbox::default();

    c.bench_function("sandbox_check_allowed", |b| {
        b.iter(|| {
            let _ = sandbox.check_function_allowed(black_box("send_key"));
        })
    });
}

/// Benchmark full sandbox check for denied function.
///
/// Target: < 20 nanoseconds (should fail fast after capability check)
fn benchmark_sandbox_check_denied(c: &mut Criterion) {
    let mut sandbox = ScriptSandbox::default();
    sandbox.set_mode(ScriptMode::Safe);

    c.bench_function("sandbox_check_denied", |b| {
        b.iter(|| {
            let _ = sandbox.check_function_allowed(black_box("clipboard_get"));
        })
    });
}

/// Benchmark sandbox mode switching.
///
/// Target: < 5 nanoseconds (simple assignment)
fn benchmark_mode_switching(c: &mut Criterion) {
    c.bench_function("mode_switching", |b| {
        b.iter_batched(
            ScriptSandbox::default,
            |mut sandbox| {
                sandbox.set_mode(black_box(ScriptMode::Safe));
                sandbox.set_mode(black_box(ScriptMode::Full));
                sandbox.set_mode(black_box(ScriptMode::Standard));
            },
            BatchSize::SmallInput,
        )
    });
}

/// Benchmark sandbox creation with full registry.
///
/// This is an initialization cost, not a hot path operation.
fn benchmark_sandbox_creation(c: &mut Criterion) {
    c.bench_function("sandbox_creation", |b| {
        b.iter(|| {
            black_box(ScriptSandbox::default());
        })
    });
}

/// Benchmark Rhai engine configuration with sandbox limits.
///
/// This is an initialization cost, not a hot path operation.
fn benchmark_engine_configuration(c: &mut Criterion) {
    let sandbox = ScriptSandbox::default();

    c.bench_function("engine_configuration", |b| {
        b.iter(|| {
            let mut engine = rhai::Engine::new();
            sandbox.configure_engine(&mut engine);
            black_box(engine);
        })
    });
}

// ===== Concurrent Access Benchmarks =====

/// Benchmark concurrent capability checks from multiple threads.
///
/// Target: Should scale linearly with minimal contention (HashMap is read-heavy)
fn benchmark_concurrent_capability_checks(c: &mut Criterion) {
    use std::sync::Arc;
    use std::thread;

    let sandbox = Arc::new(ScriptSandbox::default());

    c.bench_function("concurrent_capability_checks", |b| {
        b.iter(|| {
            let threads: Vec<_> = (0..4)
                .map(|_| {
                    let s = sandbox.clone();
                    thread::spawn(move || {
                        for _ in 0..25 {
                            let _ = s.check_function_allowed("send_key");
                        }
                    })
                })
                .collect();

            for t in threads {
                t.join().unwrap();
            }
        })
    });
}

/// Benchmark concurrent resource tracking from multiple threads.
///
/// Target: Should scale well with atomics (lock-free)
fn benchmark_concurrent_resource_tracking(c: &mut Criterion) {
    use std::sync::Arc;
    use std::thread;

    let budget = Arc::new(ResourceBudget::new(ResourceConfig::default()));

    c.bench_function("concurrent_resource_tracking", |b| {
        b.iter(|| {
            let threads: Vec<_> = (0..4)
                .map(|_| {
                    let b = budget.clone();
                    thread::spawn(move || {
                        for _ in 0..25 {
                            let _ = b.increment_instructions(1);
                            let _ = b.check_instructions();
                        }
                    })
                })
                .collect();

            for t in threads {
                t.join().unwrap();
            }
        })
    });
}

// ===== Realistic Workload Benchmarks =====

/// Benchmark realistic script execution overhead.
///
/// Simulates checking 10 function calls with capability and resource checks.
/// Target: < 300 nanoseconds total (10 functions × 30ns overhead each)
fn benchmark_realistic_script_overhead(c: &mut Criterion) {
    let sandbox = ScriptSandbox::default();

    c.bench_function("realistic_script_overhead", |b| {
        b.iter(|| {
            // Simulate 10 function calls in a script
            for func in &[
                "send_key",
                "delay",
                "print",
                "get_keycode",
                "is_pressed",
                "send_text",
                "get_timestamp",
                "send_key_combo",
                "delay",
                "print",
            ] {
                let _ = sandbox.check_function_allowed(func);
                let _ = sandbox.budget().increment_instructions(10);
                let _ = sandbox.budget().check_instructions();
            }
        })
    });
}

/// Benchmark worst-case scenario: denied function with full checks.
///
/// Target: Should fail fast, < 30 nanoseconds
fn benchmark_worst_case_denied(c: &mut Criterion) {
    let mut sandbox = ScriptSandbox::default();
    sandbox.set_mode(ScriptMode::Safe);

    c.bench_function("worst_case_denied", |b| {
        b.iter(|| {
            // Try to call 10 advanced/internal functions in Safe mode
            for func in &[
                "clipboard_get",
                "clipboard_set",
                "execute_command",
                "read_file",
                "write_file",
                "system_info",
                "network_request",
                "open_url",
                "show_notification",
                "play_sound",
            ] {
                let _ = sandbox.check_function_allowed(func);
            }
        })
    });
}

/// Benchmark comparison: baseline vs sandbox overhead.
///
/// This helps quantify the actual overhead added by sandbox checks.
fn benchmark_baseline_function_call(c: &mut Criterion) {
    c.bench_function("baseline_function_call", |b| {
        b.iter(|| {
            // Simulate 10 no-op function calls without any checks
            for _ in 0..10 {
                black_box(());
            }
        })
    });
}

criterion_group!(
    capability_benches,
    benchmark_capability_lookup,
    benchmark_capability_check,
    benchmark_capability_check_denied,
    benchmark_keycode_lookup,
    benchmark_registry_creation,
    benchmark_multiple_capability_checks,
);

criterion_group!(
    validation_benches,
    benchmark_range_validation_i64,
    benchmark_range_validation_invalid,
    benchmark_non_negative_validation,
    benchmark_positive_validation,
    benchmark_string_length_validation,
    benchmark_non_empty_validation,
    benchmark_pattern_validation,
    benchmark_keycode_validation,
    benchmark_chained_validation,
    benchmark_validation_error_path,
);

criterion_group!(
    resource_benches,
    benchmark_instruction_increment,
    benchmark_instruction_check,
    benchmark_recursion_guard,
    benchmark_memory_allocation,
    benchmark_timeout_check,
    benchmark_resource_usage_snapshot,
    benchmark_all_resource_checks,
);

criterion_group!(
    sandbox_benches,
    benchmark_sandbox_check_allowed,
    benchmark_sandbox_check_denied,
    benchmark_mode_switching,
    benchmark_sandbox_creation,
    benchmark_engine_configuration,
);

criterion_group!(
    concurrent_benches,
    benchmark_concurrent_capability_checks,
    benchmark_concurrent_resource_tracking,
);

criterion_group!(
    realistic_benches,
    benchmark_realistic_script_overhead,
    benchmark_worst_case_denied,
    benchmark_baseline_function_call,
);

criterion_main!(
    capability_benches,
    validation_benches,
    resource_benches,
    sandbox_benches,
    concurrent_benches,
    realistic_benches,
);
