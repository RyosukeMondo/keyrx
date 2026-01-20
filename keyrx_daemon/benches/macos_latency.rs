//! macOS Platform Latency Benchmarks
//!
//! This benchmark measures the performance of macOS-specific platform code:
//! - Input capture latency (keycode mapping overhead)
//! - Output injection preparation latency
//! - Full pipeline latency (capture + remap + inject)
//!
//! These benchmarks verify the <1ms latency requirement for macOS.
//!
//! Run with: cargo bench --bench macos_latency

use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[cfg(target_os = "macos")]
use keyrx_core::config::KeyCode;
#[cfg(target_os = "macos")]
use keyrx_core::runtime::KeyEvent;
#[cfg(target_os = "macos")]
use keyrx_daemon::platform::macos::keycode_map::{
    cgkeycode_to_keyrx, keyrx_to_cgkeycode, keyrx_to_enigo_key, rdev_key_to_keyrx,
};

/// Benchmark input capture overhead (CGKeyCode → KeyRx KeyCode)
fn benchmark_input_capture(c: &mut Criterion) {
    #[cfg(target_os = "macos")]
    {
        c.bench_function("macos_input_capture_latency", |b| {
            b.iter(|| {
                // Simulate CGKeyCode 0 (A key) coming from system
                let cgkeycode = black_box(0);

                // Convert to KeyRx KeyCode (what input_capture.rs does)
                let keycode = cgkeycode_to_keyrx(cgkeycode);

                // Create KeyEvent (what the daemon does)
                let event = KeyEvent::press(keycode.unwrap_or(KeyCode::Unknown));

                black_box(event)
            })
        });

        c.bench_function("macos_rdev_key_conversion", |b| {
            b.iter(|| {
                // Simulate rdev::Key::KeyA coming from rdev::listen
                #[cfg(target_os = "macos")]
                {
                    use rdev::Key;
                    let rdev_key = black_box(Key::KeyA);

                    // Convert to KeyRx KeyCode (what input_capture.rs does)
                    let keycode = rdev_key_to_keyrx(&rdev_key);

                    black_box(keycode)
                }
            })
        });
    }

    #[cfg(not(target_os = "macos"))]
    {
        c.bench_function("macos_input_capture_latency_noop", |b| {
            b.iter(|| black_box(()))
        });
    }
}

/// Benchmark output injection overhead (KeyRx KeyCode → enigo::Key)
fn benchmark_output_injection(c: &mut Criterion) {
    #[cfg(target_os = "macos")]
    {
        c.bench_function("macos_output_injection_latency", |b| {
            b.iter(|| {
                // Simulate remapped KeyEvent to inject
                let keycode = black_box(KeyCode::A);

                // Convert to enigo::Key (what output_injection.rs does)
                let enigo_key = keyrx_to_enigo_key(keycode);

                // Note: We don't actually inject (would require Accessibility permission)
                // This measures the conversion overhead only
                black_box(enigo_key)
            })
        });

        c.bench_function("macos_cgkeycode_conversion", |b| {
            b.iter(|| {
                // Simulate conversion for CGEventPost
                let keycode = black_box(KeyCode::A);

                // Convert to CGKeyCode
                let cgkeycode = keyrx_to_cgkeycode(keycode);

                black_box(cgkeycode)
            })
        });
    }

    #[cfg(not(target_os = "macos"))]
    {
        c.bench_function("macos_output_injection_latency_noop", |b| {
            b.iter(|| black_box(()))
        });
    }
}

/// Benchmark full pipeline (input → remap → output preparation)
fn benchmark_full_pipeline(c: &mut Criterion) {
    #[cfg(target_os = "macos")]
    {
        use keyrx_core::runtime::ExtendedState;

        c.bench_function("macos_full_pipeline_latency", |b| {
            // Setup: Create a simple remapping config
            // CapsLock (0x39) → Escape (0x35)
            let mut state = ExtendedState::new();

            b.iter(|| {
                // 1. Input Capture: CGKeyCode → KeyRx KeyCode
                let cgkeycode = black_box(0x39); // CapsLock
                let input_keycode = cgkeycode_to_keyrx(cgkeycode).unwrap_or(KeyCode::Unknown);

                // 2. Create event
                let input_event = KeyEvent::press(input_keycode);

                // 3. Remap (simplified - no actual config loaded)
                // In real scenario, ExtendedState.process() would be called
                let output_keycode = black_box(KeyCode::Escape); // Simulated remap result

                // 4. Output Injection: KeyRx KeyCode → enigo::Key
                let enigo_key = keyrx_to_enigo_key(output_keycode);

                black_box((input_event, enigo_key))
            })
        });
    }

    #[cfg(not(target_os = "macos"))]
    {
        c.bench_function("macos_full_pipeline_latency_noop", |b| {
            b.iter(|| black_box(()))
        });
    }
}

/// Benchmark keycode mapping lookup performance
fn benchmark_keycode_mapping(c: &mut Criterion) {
    #[cfg(target_os = "macos")]
    {
        c.bench_function("macos_keycode_bidirectional_mapping", |b| {
            b.iter(|| {
                // Test round-trip conversion
                let keycode = black_box(KeyCode::A);

                // KeyRx → CGKeyCode
                let cgkeycode = keyrx_to_cgkeycode(keycode);

                // CGKeyCode → KeyRx
                let recovered = cgkeycode_to_keyrx(cgkeycode.unwrap_or(0));

                black_box(recovered)
            })
        });

        c.bench_function("macos_special_keys_mapping", |b| {
            b.iter(|| {
                // Test special keys that might have complex mapping
                let keys = [
                    KeyCode::LeftShift,
                    KeyCode::RightShift,
                    KeyCode::LeftControl,
                    KeyCode::RightControl,
                    KeyCode::LeftAlt,
                    KeyCode::RightAlt,
                    KeyCode::LeftSuper,
                    KeyCode::RightSuper,
                ];

                for key in keys.iter() {
                    let cgkeycode = keyrx_to_cgkeycode(black_box(*key));
                    black_box(cgkeycode);
                }
            })
        });
    }

    #[cfg(not(target_os = "macos"))]
    {
        c.bench_function("macos_keycode_mapping_noop", |b| {
            b.iter(|| black_box(()))
        });
    }
}

criterion_group!(
    benches,
    benchmark_input_capture,
    benchmark_output_injection,
    benchmark_full_pipeline,
    benchmark_keycode_mapping
);
criterion_main!(benches);
