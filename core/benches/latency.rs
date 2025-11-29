//! Latency benchmark for the KeyRx engine.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use keyrx_core::engine::{InputEvent, KeyCode};

fn benchmark_event_creation(c: &mut Criterion) {
    c.bench_function("create_input_event", |b| {
        b.iter(|| {
            let event = InputEvent::key_down(black_box(KeyCode(0x41)), black_box(1000));
            black_box(event)
        })
    });
}

fn benchmark_modifier_set(c: &mut Criterion) {
    use keyrx_core::engine::ModifierSet;

    c.bench_function("modifier_set_operations", |b| {
        b.iter(|| {
            let mut mods = ModifierSet::new();
            mods.add(black_box(1));
            mods.add(black_box(2));
            mods.contains(black_box(1));
            mods.contains_all(black_box(&[1, 2]));
            mods.remove(black_box(1));
            black_box(mods)
        })
    });
}

criterion_group!(benches, benchmark_event_creation, benchmark_modifier_set);
criterion_main!(benches);
