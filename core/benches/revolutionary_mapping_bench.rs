//! Benchmarks for revolutionary mapping pipeline components.
//!
//! Verifies latency targets:
//! - Device resolution: <50μs
//! - Profile lookup (cached): <100μs
//! - Coordinate translation (cached): <20μs
//! - Full pipeline: <1ms

// Allow unwrap/expect in benchmarks - panics are acceptable for setup code
#![allow(clippy::unwrap_used, clippy::expect_used)]

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use keyrx_core::definitions::library::DeviceDefinitionLibrary;
use keyrx_core::drivers::keycodes::KeyCode;
use keyrx_core::engine::{CoordinateTranslator, DeviceResolver, InputEvent, ProfileResolver};
use keyrx_core::identity::DeviceIdentity;
use keyrx_core::registry::device::DeviceRegistry;
use keyrx_core::registry::profile::{KeyAction, LayoutType, PhysicalPosition, Profile};
use keyrx_core::registry::{ProfileRegistry, ProfileRegistryStorage};
use std::sync::Arc;
use tempfile::TempDir;
use tokio::runtime::Runtime;

/// Create a test device definition
fn create_test_definition() -> (TempDir, Arc<DeviceDefinitionLibrary>) {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.toml");

    let toml = r#"
name = "Test Device"
vendor_id = 0x1234
product_id = 0x5678
manufacturer = "Test"

[layout]
layout_type = "matrix"
rows = 5
cols = 5

[matrix_map]
"#;

    let mut content = toml.to_string();
    for i in 1..=25 {
        let row = (i - 1) / 5;
        let col = (i - 1) % 5;
        content.push_str(&format!("\"{i}\" = {{ row = {row}, col = {col} }}\n"));
    }

    content.push_str(
        r#"
[visual]
key_width = 80
key_height = 80
key_spacing = 4
"#,
    );

    std::fs::write(&file_path, content).unwrap();

    let mut library = DeviceDefinitionLibrary::new();
    library.load_from_directory(temp_dir.path()).unwrap();

    (temp_dir, Arc::new(library))
}

/// Create a test profile
fn create_test_profile() -> Profile {
    let mut profile = Profile::new("Benchmark Profile", LayoutType::Matrix);

    // Add mappings for all 25 positions
    for row in 0..5 {
        for col in 0..5 {
            profile.set_action(
                PhysicalPosition::new(row, col),
                KeyAction::key(KeyCode::A), // Simple mapping
            );
        }
    }

    profile
}

/// Create test event with identity
fn create_event(scancode: u16) -> InputEvent {
    InputEvent::with_full_identity(
        KeyCode::Unknown(0),
        true,
        1000,
        Some("/dev/input/event0".to_string()),
        false,
        false,
        scancode,
        Some("TEST123".to_string()),
        Some(0x1234),
        Some(0x5678),
    )
}

fn bench_device_resolution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    // Setup
    let (registry, _rx) = DeviceRegistry::new();
    let resolver = DeviceResolver::new(registry.clone());

    let identity = DeviceIdentity::new(0x1234, 0x5678, "TEST123".to_string());

    rt.block_on(async {
        registry.register_device(identity.clone()).await;
        registry.set_remap_enabled(&identity, true).await.unwrap();
    });

    let event = create_event(1);

    c.bench_function("device_resolution", |b| {
        b.iter(|| {
            let result = rt.block_on(resolver.resolve(black_box(&event)));
            black_box(result)
        })
    });
}

fn bench_profile_resolution_cold(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("profile_resolution_cold", |b| {
        b.iter_batched(
            || {
                // Setup for each iteration (fresh cache)
                let temp = TempDir::new().unwrap();
                let registry = Arc::new(ProfileRegistry::with_directory(temp.path().to_path_buf()));
                let resolver = ProfileResolver::new(registry.clone());

                let profile = create_test_profile();
                let profile_id = profile.id.clone();

                rt.block_on(async {
                    registry.save_profile(&profile).await.unwrap();
                    // Clear cache to simulate cold load
                    resolver.invalidate_all().await;
                });

                (resolver, profile_id, temp)
            },
            |(resolver, profile_id, _temp)| {
                rt.block_on(async {
                    let result = resolver.resolve(black_box(&profile_id)).await;
                    black_box(result)
                })
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

fn bench_profile_resolution_warm(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    // Setup once (cache will be warm)
    let temp = TempDir::new().unwrap();
    let registry = Arc::new(ProfileRegistry::with_directory(temp.path().to_path_buf()));
    let resolver = ProfileResolver::new(registry.clone());

    let profile = create_test_profile();
    let profile_id = profile.id.clone();

    rt.block_on(async {
        registry.save_profile(&profile).await.unwrap();
        // Warm up cache
        let _ = resolver.resolve(&profile_id).await.unwrap();
    });

    c.bench_function("profile_resolution_warm", |b| {
        b.iter(|| {
            let result = rt.block_on(resolver.resolve(black_box(&profile_id)));
            black_box(result)
        })
    });
}

fn bench_coordinate_translation_cold(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("coordinate_translation_cold", |b| {
        b.iter_batched(
            || {
                // Setup for each iteration (fresh cache)
                let (_temp, library) = create_test_definition();
                let translator = CoordinateTranslator::new(library);
                let device = DeviceIdentity::new(0x1234, 0x5678, "TEST123".to_string());

                (translator, device)
            },
            |(translator, device)| {
                rt.block_on(async {
                    let result = translator.translate(black_box(&device), black_box(1)).await;
                    black_box(result)
                })
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

fn bench_coordinate_translation_warm(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    // Setup once (cache will be warm)
    let (_temp, library) = create_test_definition();
    let translator = CoordinateTranslator::new(library);
    let device = DeviceIdentity::new(0x1234, 0x5678, "TEST123".to_string());

    // Warm up cache
    rt.block_on(async {
        let _ = translator.translate(&device, 1).await.unwrap();
    });

    c.bench_function("coordinate_translation_warm", |b| {
        b.iter(|| {
            let result = rt.block_on(translator.translate(black_box(&device), black_box(1)));
            black_box(result)
        })
    });
}

fn bench_coordinate_translation_various_scancodes(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let (_temp, library) = create_test_definition();
    let translator = CoordinateTranslator::new(library);
    let device = DeviceIdentity::new(0x1234, 0x5678, "TEST123".to_string());

    // Warm up cache
    rt.block_on(async {
        let _ = translator.translate(&device, 1).await.unwrap();
    });

    let mut group = c.benchmark_group("coordinate_translation_scancodes");

    for scancode in [1, 5, 10, 15, 20, 25] {
        group.bench_with_input(
            BenchmarkId::from_parameter(scancode),
            &scancode,
            |b, &scancode| {
                b.iter(|| {
                    let result =
                        rt.block_on(translator.translate(black_box(&device), black_box(scancode)));
                    black_box(result)
                })
            },
        );
    }

    group.finish();
}

fn bench_full_pipeline_warm(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    // Setup all components
    let (registry, _rx) = DeviceRegistry::new();
    let temp = TempDir::new().unwrap();
    let profile_registry = Arc::new(ProfileRegistry::with_directory(temp.path().to_path_buf()));
    let (_temp_defs, library) = create_test_definition();

    let device_resolver = DeviceResolver::new(registry.clone());
    let profile_resolver = ProfileResolver::new(profile_registry.clone());
    let coord_translator = CoordinateTranslator::new(library);

    let identity = DeviceIdentity::new(0x1234, 0x5678, "TEST123".to_string());
    let profile = create_test_profile();
    let profile_id = profile.id.clone();

    rt.block_on(async {
        registry.register_device(identity.clone()).await;
        registry.set_remap_enabled(&identity, true).await.unwrap();
        profile_registry.save_profile(&profile).await.unwrap();
        registry
            .assign_profile(&identity, profile_id.clone())
            .await
            .unwrap();

        // Warm up all caches
        let event = create_event(1);
        let _ = device_resolver.resolve(&event).await.unwrap();
        let _ = profile_resolver.resolve(&profile_id).await.unwrap();
        let _ = coord_translator.translate(&identity, 1).await.unwrap();
    });

    c.bench_function("full_pipeline_warm", |b| {
        b.iter(|| {
            rt.block_on(async {
                let event = create_event(black_box(1));

                // Step 1: Device resolution
                let device_state = device_resolver
                    .resolve(black_box(&event))
                    .await
                    .unwrap()
                    .unwrap();

                // Step 2: Profile resolution
                let profile = profile_resolver
                    .resolve(black_box(&device_state.profile_id.unwrap()))
                    .await
                    .unwrap();

                // Step 3: Coordinate translation
                let physical_pos = coord_translator
                    .translate(
                        black_box(&device_state.identity),
                        black_box(event.scan_code),
                    )
                    .await
                    .unwrap();

                // Step 4: Action lookup
                let action = profile.get_action(black_box(&physical_pos)).cloned();

                black_box(action)
            })
        })
    });
}

fn bench_full_pipeline_multiple_events(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    // Setup
    let (registry, _rx) = DeviceRegistry::new();
    let temp = TempDir::new().unwrap();
    let profile_registry = Arc::new(ProfileRegistry::with_directory(temp.path().to_path_buf()));
    let (_temp_defs, library) = create_test_definition();

    let device_resolver = DeviceResolver::new(registry.clone());
    let profile_resolver = ProfileResolver::new(profile_registry.clone());
    let coord_translator = CoordinateTranslator::new(library);

    let identity = DeviceIdentity::new(0x1234, 0x5678, "TEST123".to_string());
    let profile = create_test_profile();
    let profile_id = profile.id.clone();

    rt.block_on(async {
        registry.register_device(identity.clone()).await;
        registry.set_remap_enabled(&identity, true).await.unwrap();
        profile_registry.save_profile(&profile).await.unwrap();
        registry
            .assign_profile(&identity, profile_id.clone())
            .await
            .unwrap();

        // Warm up caches
        let event = create_event(1);
        let _ = device_resolver.resolve(&event).await.unwrap();
        let _ = profile_resolver.resolve(&profile_id).await.unwrap();
        let _ = coord_translator.translate(&identity, 1).await.unwrap();
    });

    let mut group = c.benchmark_group("full_pipeline_multiple_events");

    for event_count in [1, 10, 50, 100] {
        group.bench_with_input(
            BenchmarkId::from_parameter(event_count),
            &event_count,
            |b, &event_count| {
                b.iter(|| {
                    rt.block_on(async {
                        for i in 0..event_count {
                            let scancode = ((i % 25) + 1) as u16;
                            let event = create_event(black_box(scancode));

                            let device_state = device_resolver
                                .resolve(black_box(&event))
                                .await
                                .unwrap()
                                .unwrap();

                            let profile = profile_resolver
                                .resolve(black_box(&device_state.profile_id.unwrap()))
                                .await
                                .unwrap();

                            let physical_pos = coord_translator
                                .translate(
                                    black_box(&device_state.identity),
                                    black_box(event.scan_code),
                                )
                                .await
                                .unwrap();

                            let action = profile.get_action(black_box(&physical_pos)).cloned();
                            black_box(action);
                        }
                    })
                })
            },
        );
    }

    group.finish();
}

fn bench_concurrent_device_lookups(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let (registry, _rx) = DeviceRegistry::new();
    let resolver = DeviceResolver::new(registry.clone());

    let identity = DeviceIdentity::new(0x1234, 0x5678, "TEST123".to_string());

    rt.block_on(async {
        registry.register_device(identity.clone()).await;
        registry.set_remap_enabled(&identity, true).await.unwrap();
    });

    let mut group = c.benchmark_group("concurrent_device_lookups");

    for concurrency in [1, 5, 10, 20] {
        group.bench_with_input(
            BenchmarkId::from_parameter(concurrency),
            &concurrency,
            |b, &concurrency| {
                b.iter(|| {
                    rt.block_on(async {
                        let mut handles = vec![];

                        for _ in 0..concurrency {
                            let resolver = resolver.clone();
                            let event = create_event(1);
                            let handle =
                                tokio::spawn(
                                    async move { resolver.resolve(black_box(&event)).await },
                                );
                            handles.push(handle);
                        }

                        for handle in handles {
                            let _ = handle.await;
                        }
                    })
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_device_resolution,
    bench_profile_resolution_cold,
    bench_profile_resolution_warm,
    bench_coordinate_translation_cold,
    bench_coordinate_translation_warm,
    bench_coordinate_translation_various_scancodes,
    bench_full_pipeline_warm,
    bench_full_pipeline_multiple_events,
    bench_concurrent_device_lookups,
);

criterion_main!(benches);
