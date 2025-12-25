use criterion::{black_box, criterion_group, criterion_main, Criterion};
use crossbeam_channel::unbounded;
use keyrx_core::runtime::KeyEvent;
use keyrx_daemon::platform::windows::device_map::DeviceMap; // Need to ensure this is accessible

// Note: To benchmark internal functions we might need to expose them or integrate benchmark inside the crate.
// For now, we simulate the workload of process_raw_keyboard:
// 1. Scancode conversion (trivial)
// 2. Device ID lookup (DeviceMap)
// 3. Channel send

fn benchmark_input_processing(c: &mut Criterion) {
    // Setup
    let device_map = DeviceMap::new();
    // Pre-populate device map with a "device"
    // We can't insert cheaply because it uses internal functionality, but we can rely on empty map lookup speed
    // or try to add a dummy handle if we can mock the API calls (impossible without traits).
    // Let's benchmark the "worst case" (cache miss / empty) or "best case" (empty).

    // Actually, we can use the `DeviceMap` if we can insert. `add_device` calls Windows APIs.
    // So we are stuck benchmarking empty map lookup unless we mock.

    // Alternative: Benchmark the channel overhead + scancode conversion which are the other parts.

    // Let's assume we are benchmarking the "infrastructure overhead".

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

criterion_group!(benches, benchmark_input_processing);
criterion_main!(benches);
