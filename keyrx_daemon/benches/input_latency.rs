// This benchmark tests DeviceMap performance which is Windows-specific.
// On non-Windows platforms, this compiles but does nothing.

use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[cfg(target_os = "windows")]
use crossbeam_channel::unbounded;
#[cfg(target_os = "windows")]
use keyrx_core::runtime::KeyEvent;
#[cfg(target_os = "windows")]
use keyrx_daemon::platform::windows::device_map::DeviceMap;

fn benchmark_input_processing(c: &mut Criterion) {
    #[cfg(target_os = "windows")]
    {
        // Setup
        let device_map = DeviceMap::new();
        let (tx, rx) = unbounded();

        c.bench_function("input_processing_overhead", |b| {
            b.iter(|| {
                // Simulate 1 event processing

                // 1. Scancode to Keycode (simulated)
                let keycode = keyrx_core::config::KeyCode::A;

                // 2. Event creation
                let event = KeyEvent::press(keycode);

                // 3. Simulated Device Lookup (using the real map, empty)
                // Even failed lookup represents logic execution time
                let _info = device_map.get(black_box(12345 as *mut std::ffi::c_void));

                // 4. Send to channel
                let _ = tx.send(black_box(event));

                // Drain channel to prevent unbound growth affecting mem
                let _ = rx.try_recv();
            })
        });
    }

    #[cfg(not(target_os = "windows"))]
    {
        // On non-Windows platforms, create a no-op benchmark
        c.bench_function("input_processing_overhead_noop", |b| {
            b.iter(|| {
                // No-op benchmark for non-Windows platforms
                black_box(());
            })
        });
    }
}

criterion_group!(benches, benchmark_input_processing);
criterion_main!(benches);
