//! Latency benchmark for the KeyRx engine.

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use keyrx_core::engine::{
    AdvancedEngine, ComboRegistry, HoldAction, InputEvent, KeyCode, Layer, LayerAction, LayerStack,
    TimingConfig,
};
use keyrx_core::mocks::MockRuntime;

fn benchmark_event_creation(c: &mut Criterion) {
    c.bench_function("create_input_event", |b| {
        b.iter(|| {
            let event = InputEvent::key_down(black_box(KeyCode::A), black_box(1000));
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

fn benchmark_process_event_with_pending(c: &mut Criterion) {
    c.bench_function("process_event_tap_hold_pending", |b| {
        b.iter_batched(
            || {
                let mut engine = AdvancedEngine::new(MockRuntime::new(), TimingConfig::default());
                let tap_hold = LayerAction::TapHold {
                    tap: KeyCode::B,
                    hold: HoldAction::Key(KeyCode::C),
                };
                debug_assert!(engine
                    .layers_mut()
                    .set_mapping_for_layer(0, KeyCode::A, tap_hold));

                let down = InputEvent::with_metadata(
                    KeyCode::A,
                    true,
                    1_000,
                    Some("bench".to_string()),
                    false,
                    false,
                    0,
                );
                let up = InputEvent::with_metadata(
                    KeyCode::A,
                    false,
                    1_050,
                    Some("bench".to_string()),
                    false,
                    false,
                    0,
                );
                (engine, down, up)
            },
            |(mut engine, down, up)| {
                black_box(engine.process_event(down.clone()));
                black_box(engine.process_event(up.clone()));
            },
            BatchSize::SmallInput,
        )
    });
}

fn build_layer_stack_for_lookup() -> LayerStack {
    let mut stack = LayerStack::new();

    for layer_id in 1..=5 {
        let mut layer = Layer::with_id(layer_id, format!("layer-{layer_id}"));
        layer.transparent = layer_id % 2 == 1;
        if layer_id == 5 {
            layer.set_mapping(KeyCode::F1, LayerAction::Remap(KeyCode::F2));
        }
        stack.define_layer(layer);
        debug_assert!(stack.push(layer_id));
    }

    stack
}

fn benchmark_layer_lookup_many_layers(c: &mut Criterion) {
    let stack = build_layer_stack_for_lookup();
    c.bench_function("layer_lookup_with_6_layers", |b| {
        b.iter(|| black_box(stack.lookup(KeyCode::F1)))
    });
}

fn build_combo_registry() -> ComboRegistry {
    let mut registry = ComboRegistry::new();
    let combos: &[(&[KeyCode], LayerAction)] = &[
        (
            &[KeyCode::A, KeyCode::S],
            LayerAction::Remap(KeyCode::Escape),
        ),
        (
            &[KeyCode::Q, KeyCode::W, KeyCode::E],
            LayerAction::LayerPush(1),
        ),
        (
            &[KeyCode::J, KeyCode::K, KeyCode::L],
            LayerAction::LayerToggle(2),
        ),
        (&[KeyCode::Z, KeyCode::X], LayerAction::ModifierActivate(1)),
        (
            &[KeyCode::C, KeyCode::V],
            LayerAction::ModifierDeactivate(1),
        ),
        (&[KeyCode::B, KeyCode::N], LayerAction::ModifierOneShot(2)),
        (
            &[KeyCode::U, KeyCode::I, KeyCode::O],
            LayerAction::TapHold {
                tap: KeyCode::Enter,
                hold: HoldAction::Key(KeyCode::Tab),
            },
        ),
        (
            &[KeyCode::Key1, KeyCode::Key2, KeyCode::Key3],
            LayerAction::Block,
        ),
        (
            &[KeyCode::Space, KeyCode::LeftShift],
            LayerAction::Remap(KeyCode::Tab),
        ),
        (
            &[KeyCode::LeftCtrl, KeyCode::C],
            LayerAction::Remap(KeyCode::C),
        ),
        (
            &[KeyCode::LeftCtrl, KeyCode::V],
            LayerAction::Remap(KeyCode::V),
        ),
        (
            &[KeyCode::LeftCtrl, KeyCode::X],
            LayerAction::Remap(KeyCode::X),
        ),
    ];

    for (keys, action) in combos {
        debug_assert!(registry.register(keys, action.clone()));
    }

    registry
}

fn benchmark_combo_matching_ten_plus(c: &mut Criterion) {
    let registry = build_combo_registry();
    let two_key = [KeyCode::Space, KeyCode::LeftShift];
    let three_key = [KeyCode::Key3, KeyCode::Key1, KeyCode::Key2];

    c.bench_function("combo_matching_with_registry", |b| {
        b.iter(|| {
            black_box(registry.find(black_box(&two_key)));
            black_box(registry.find(black_box(&three_key)));
        })
    });
}

criterion_group!(
    benches,
    benchmark_event_creation,
    benchmark_modifier_set,
    benchmark_process_event_with_pending,
    benchmark_layer_lookup_many_layers,
    benchmark_combo_matching_ten_plus
);
criterion_main!(benches);
