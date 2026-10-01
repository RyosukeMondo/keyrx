use criterion::{black_box, criterion_group, criterion_main, Criterion};
use keyrx_daemon::config::profile_manager::{ProfileManager, ProfileTemplate};
use tempfile::TempDir;

/// Benchmark profile activation (hot-reload) excluding compilation time.
/// Target: <100ms for hot-reload
fn benchmark_profile_activation(c: &mut Criterion) {
    // Setup: Create temporary config directory and profile manager
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_dir = temp_dir.path().to_path_buf();

    // Create profiles directory
    let profiles_dir = config_dir.join("profiles");
    std::fs::create_dir_all(&profiles_dir).expect("Failed to create profiles dir");

    // Initialize ProfileManager
    let manager = ProfileManager::new(config_dir.clone()).expect("Failed to create ProfileManager");

    // Create a test profile
    manager
        .create("benchmark_profile", ProfileTemplate::Blank)
        .expect("Failed to create benchmark profile");

    // Pre-compile the profile to ensure .krx exists
    // This is done outside the benchmark since we're measuring hot-reload, not compilation
    let _ = manager.activate("benchmark_profile");

    c.bench_function("profile_activation_hot_reload", |b| {
        b.iter(|| {
            // Measure only the hot-reload portion (excluding compilation)
            // Since .krx already exists, this benchmarks the load + swap operation
            let result = manager.activate(black_box("benchmark_profile"));

            // Ensure the operation succeeded
            assert!(result.is_ok(), "Profile activation failed");

            if let Ok(activation_result) = result {
                // The reload_time_ms is what we care about (<100ms target)
                // Note: This doesn't include compilation since .krx is cached
                let _ = black_box(activation_result);
            }
        });
    });
}

/// Benchmark profile creation from template.
/// This measures file I/O and template generation overhead.
fn benchmark_profile_creation(c: &mut Criterion) {
    c.bench_function("profile_creation_blank_template", |b| {
        b.iter_batched(
            || {
                // Setup: Fresh temp directory for each iteration
                let temp_dir = TempDir::new().expect("Failed to create temp dir");
                let config_dir = temp_dir.path().to_path_buf();
                let profiles_dir = config_dir.join("profiles");
                std::fs::create_dir_all(&profiles_dir).expect("Failed to create profiles dir");

                let manager =
                    ProfileManager::new(config_dir).expect("Failed to create ProfileManager");

                (manager, temp_dir)
            },
            |(manager, _temp_dir)| {
                // Benchmark: Create profile from blank template
                let result =
                    manager.create(black_box("test_profile"), black_box(ProfileTemplate::Blank));

                assert!(result.is_ok(), "Profile creation failed");
                let _ = black_box(result);
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

/// Benchmark profile listing operation.
/// This measures filesystem scanning performance.
fn benchmark_profile_listing(c: &mut Criterion) {
    // Setup: Create temp directory with multiple profiles
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_dir = temp_dir.path().to_path_buf();
    let profiles_dir = config_dir.join("profiles");
    std::fs::create_dir_all(&profiles_dir).expect("Failed to create profiles dir");

    let manager = ProfileManager::new(config_dir).expect("Failed to create ProfileManager");

    // Create 10 test profiles
    for i in 0..10 {
        manager
            .create(&format!("profile_{}", i), ProfileTemplate::Blank)
            .expect("Failed to create profile");
    }

    c.bench_function("profile_list_10_profiles", |b| {
        b.iter(|| {
            let profiles = manager.list();
            black_box(profiles);
        });
    });
}

criterion_group!(
    benches,
    benchmark_profile_activation,
    benchmark_profile_creation,
    benchmark_profile_listing
);
criterion_main!(benches);
