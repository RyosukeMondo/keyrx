#![allow(unsafe_code)]
//! FFI marshaling benchmarks.
//!
//! Compares different marshaling strategies (JSON vs C struct) and measures
//! the overhead of converting between Rust and C representations.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use keyrx_core::ffi::marshal::impls::array::free_ffi_array;
use keyrx_core::ffi::marshal::impls::json::{free_ffi_json, JsonWrapper};
use keyrx_core::ffi::marshal::impls::string::free_ffi_string;
use keyrx_core::ffi::marshal::traits::FfiMarshaler;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Test Data Structures
// ============================================================================

/// Simple struct for C struct marshaling benchmark.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct SimpleDevice {
    vendor_id: u16,
    product_id: u16,
    name: String,
}

/// Complex nested struct for JSON marshaling benchmark.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ComplexConfig {
    devices: Vec<SimpleDevice>,
    settings: HashMap<String, i32>,
    layers: Vec<LayerConfig>,
    enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct LayerConfig {
    id: u32,
    name: String,
    mappings: HashMap<String, String>,
    transparent: bool,
}

// ============================================================================
// Benchmark: Primitive Types
// ============================================================================

fn benchmark_primitives(c: &mut Criterion) {
    let mut group = c.benchmark_group("primitives");

    // u32 marshaling (zero-copy)
    group.bench_function("u32_to_c", |b| {
        let value = 42u32;
        b.iter(|| {
            let c_repr = black_box(&value).to_c().unwrap();
            black_box(c_repr)
        })
    });

    group.bench_function("u32_from_c", |b| {
        let c_value = 42u32;
        b.iter(|| {
            let value = u32::from_c(black_box(c_value)).unwrap();
            black_box(value)
        })
    });

    group.bench_function("u32_roundtrip", |b| {
        let value = 42u32;
        b.iter(|| {
            let c_repr = black_box(&value).to_c().unwrap();
            let restored = u32::from_c(black_box(c_repr)).unwrap();
            black_box(restored)
        })
    });

    // u64 marshaling
    group.bench_function("u64_roundtrip", |b| {
        let value = 12345678901234u64;
        b.iter(|| {
            let c_repr = black_box(&value).to_c().unwrap();
            let restored = u64::from_c(black_box(c_repr)).unwrap();
            black_box(restored)
        })
    });

    // bool marshaling
    group.bench_function("bool_roundtrip", |b| {
        let value = true;
        b.iter(|| {
            let c_repr = black_box(&value).to_c().unwrap();
            let restored = bool::from_c(black_box(c_repr)).unwrap();
            black_box(restored)
        })
    });

    // f64 marshaling
    group.bench_function("f64_roundtrip", |b| {
        let value = 3.14159265359f64;
        b.iter(|| {
            let c_repr = black_box(&value).to_c().unwrap();
            let restored = f64::from_c(black_box(c_repr)).unwrap();
            black_box(restored)
        })
    });

    group.finish();
}

// ============================================================================
// Benchmark: String Marshaling
// ============================================================================

fn benchmark_strings(c: &mut Criterion) {
    let mut group = c.benchmark_group("strings");

    // Small string (< 64 bytes)
    let small_str = "Hello, World!";
    group.throughput(Throughput::Bytes(small_str.len() as u64));
    group.bench_function("small_string_to_c", |b| {
        let s = String::from(small_str);
        b.iter(|| {
            let c_str = black_box(&s).to_c().unwrap();
            unsafe {
                free_ffi_string(c_str);
            }
        })
    });

    // Medium string (~256 bytes)
    let medium_str = "a".repeat(256);
    group.throughput(Throughput::Bytes(medium_str.len() as u64));
    group.bench_function("medium_string_to_c", |b| {
        let s = medium_str.clone();
        b.iter(|| {
            let c_str = black_box(&s).to_c().unwrap();
            unsafe {
                free_ffi_string(c_str);
            }
        })
    });

    // Large string (~4KB)
    let large_str = "a".repeat(4096);
    group.throughput(Throughput::Bytes(large_str.len() as u64));
    group.bench_function("large_string_to_c", |b| {
        let s = large_str.clone();
        b.iter(|| {
            let c_str = black_box(&s).to_c().unwrap();
            unsafe {
                free_ffi_string(c_str);
            }
        })
    });

    // String roundtrip
    group.bench_function("string_roundtrip", |b| {
        let s = String::from("test string");
        b.iter(|| {
            let c_str = black_box(&s).to_c().unwrap();
            let restored = String::from_c(black_box(c_str)).unwrap();
            unsafe {
                free_ffi_string(c_str);
            }
            black_box(restored)
        })
    });

    // Unicode string
    let unicode_str = "Hello 世界 🦀".to_string();
    group.bench_function("unicode_string_roundtrip", |b| {
        let s = unicode_str.clone();
        b.iter(|| {
            let c_str = black_box(&s).to_c().unwrap();
            let restored = String::from_c(black_box(c_str)).unwrap();
            unsafe {
                free_ffi_string(c_str);
            }
            black_box(restored)
        })
    });

    group.finish();
}

// ============================================================================
// Benchmark: Array Marshaling
// ============================================================================

fn benchmark_arrays(c: &mut Criterion) {
    let mut group = c.benchmark_group("arrays");

    // Small array (10 elements)
    let small_array: Vec<u32> = (0..10).collect();
    group.throughput(Throughput::Elements(small_array.len() as u64));
    group.bench_function("small_u32_array_to_c", |b| {
        let arr = small_array.clone();
        b.iter(|| {
            let c_arr = black_box(&arr).to_c().unwrap();
            unsafe {
                free_ffi_array(c_arr);
            }
        })
    });

    // Medium array (100 elements)
    let medium_array: Vec<u32> = (0..100).collect();
    group.throughput(Throughput::Elements(medium_array.len() as u64));
    group.bench_function("medium_u32_array_to_c", |b| {
        let arr = medium_array.clone();
        b.iter(|| {
            let c_arr = black_box(&arr).to_c().unwrap();
            unsafe {
                free_ffi_array(c_arr);
            }
        })
    });

    // Large array (1000 elements)
    let large_array: Vec<u32> = (0..1000).collect();
    group.throughput(Throughput::Elements(large_array.len() as u64));
    group.bench_function("large_u32_array_to_c", |b| {
        let arr = large_array.clone();
        b.iter(|| {
            let c_arr = black_box(&arr).to_c().unwrap();
            unsafe {
                free_ffi_array(c_arr);
            }
        })
    });

    // Array roundtrip
    let test_array: Vec<u32> = vec![1, 2, 3, 4, 5];
    group.bench_function("u32_array_roundtrip", |b| {
        let arr = test_array.clone();
        b.iter(|| {
            let c_arr = black_box(&arr).to_c().unwrap();
            let restored = Vec::<u32>::from_c(black_box(c_arr)).unwrap();
            unsafe {
                free_ffi_array(c_arr);
            }
            black_box(restored)
        })
    });

    // String array (heap allocations per element)
    let string_array = vec![
        "hello".to_string(),
        "world".to_string(),
        "test".to_string(),
        "data".to_string(),
    ];
    group.bench_function("string_array_to_c", |b| {
        let arr = string_array.clone();
        b.iter(|| {
            let c_arr = black_box(&arr).to_c().unwrap();
            unsafe {
                // Free each string, then the array
                let slice = c_arr.as_slice();
                for ffi_str in slice {
                    free_ffi_string(*ffi_str);
                }
                free_ffi_array(c_arr);
            }
        })
    });

    group.finish();
}

// ============================================================================
// Benchmark: JSON Marshaling
// ============================================================================

fn benchmark_json_marshaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("json");

    // Simple struct
    let simple_device = SimpleDevice {
        vendor_id: 0x1234,
        product_id: 0x5678,
        name: "Test Keyboard".to_string(),
    };

    group.bench_function("simple_struct_to_c", |b| {
        let device = simple_device.clone();
        b.iter(|| {
            let wrapper = JsonWrapper::new(black_box(device.clone()));
            let c_json = wrapper.to_c().unwrap();
            unsafe {
                free_ffi_json(c_json);
            }
        })
    });

    group.bench_function("simple_struct_roundtrip", |b| {
        let device = simple_device.clone();
        b.iter(|| {
            let wrapper = JsonWrapper::new(black_box(device.clone()));
            let c_json = wrapper.to_c().unwrap();
            let restored = JsonWrapper::<SimpleDevice>::from_c(black_box(c_json)).unwrap();
            unsafe {
                free_ffi_json(c_json);
            }
            black_box(restored)
        })
    });

    // Complex nested struct
    let complex_config = create_complex_config(5, 10);

    group.bench_function("complex_struct_to_c", |b| {
        let config = complex_config.clone();
        b.iter(|| {
            let wrapper = JsonWrapper::new(black_box(config.clone()));
            let c_json = wrapper.to_c().unwrap();
            unsafe {
                free_ffi_json(c_json);
            }
        })
    });

    group.bench_function("complex_struct_roundtrip", |b| {
        let config = complex_config.clone();
        b.iter(|| {
            let wrapper = JsonWrapper::new(black_box(config.clone()));
            let c_json = wrapper.to_c().unwrap();
            let restored = JsonWrapper::<ComplexConfig>::from_c(black_box(c_json)).unwrap();
            unsafe {
                free_ffi_json(c_json);
            }
            black_box(restored)
        })
    });

    // Very large config (tests scalability)
    let very_large_config = create_complex_config(20, 50);
    group.bench_function("very_large_struct_to_c", |b| {
        let config = very_large_config.clone();
        b.iter(|| {
            let wrapper = JsonWrapper::new(black_box(config.clone()));
            let c_json = wrapper.to_c().unwrap();
            unsafe {
                free_ffi_json(c_json);
            }
        })
    });

    group.finish();
}

// ============================================================================
// Benchmark: JSON vs C Struct Comparison
// ============================================================================

fn benchmark_json_vs_c_struct(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_vs_c_struct");

    // Create test data with varying sizes
    for size in [1, 5, 10, 25, 50] {
        let devices: Vec<SimpleDevice> = (0..size)
            .map(|i| SimpleDevice {
                vendor_id: (0x1000 + i) as u16,
                product_id: (0x2000 + i) as u16,
                name: format!("Device {}", i),
            })
            .collect();

        // JSON marshaling
        group.bench_with_input(BenchmarkId::new("json", size), &devices, |b, devices| {
            b.iter(|| {
                let wrapper = JsonWrapper::new(black_box(devices.clone()));
                let c_json = wrapper.to_c().unwrap();
                let restored = JsonWrapper::<Vec<SimpleDevice>>::from_c(black_box(c_json)).unwrap();
                unsafe {
                    free_ffi_json(c_json);
                }
                black_box(restored)
            })
        });

        // For comparison: raw serde_json serialization overhead
        group.bench_with_input(
            BenchmarkId::new("serde_json_only", size),
            &devices,
            |b, devices| {
                b.iter(|| {
                    let json_str = serde_json::to_string(black_box(&devices)).unwrap();
                    let restored: Vec<SimpleDevice> = serde_json::from_str(&json_str).unwrap();
                    black_box(restored)
                })
            },
        );
    }

    group.finish();
}

// ============================================================================
// Benchmark: Estimated Size Calculation
// ============================================================================

fn benchmark_estimated_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("estimated_size");

    // Primitives
    group.bench_function("u32_estimated_size", |b| {
        let value = 42u32;
        b.iter(|| {
            let size = black_box(&value).estimated_size();
            black_box(size)
        })
    });

    // String
    group.bench_function("string_estimated_size", |b| {
        let s = String::from("test string");
        b.iter(|| {
            let size = black_box(&s).estimated_size();
            black_box(size)
        })
    });

    // Array
    group.bench_function("array_estimated_size", |b| {
        let arr: Vec<u32> = (0..100).collect();
        b.iter(|| {
            let size = black_box(&arr).estimated_size();
            black_box(size)
        })
    });

    // JSON wrapper
    group.bench_function("json_wrapper_estimated_size", |b| {
        let device = SimpleDevice {
            vendor_id: 0x1234,
            product_id: 0x5678,
            name: "Test".to_string(),
        };
        let wrapper = JsonWrapper::new(device);
        b.iter(|| {
            let size = black_box(&wrapper).estimated_size();
            black_box(size)
        })
    });

    group.finish();
}

// ============================================================================
// Benchmark: Memory Allocation Patterns
// ============================================================================

fn benchmark_allocation_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocation_patterns");

    // Single large allocation vs many small allocations
    group.bench_function("single_large_array", |b| {
        b.iter(|| {
            let large_array: Vec<u32> = (0..1000).collect();
            let c_arr = black_box(&large_array).to_c().unwrap();
            unsafe {
                free_ffi_array(c_arr);
            }
        })
    });

    group.bench_function("many_small_strings", |b| {
        b.iter(|| {
            let strings: Vec<String> = (0..100).map(|i| format!("str_{}", i)).collect();
            let c_arr = black_box(&strings).to_c().unwrap();
            unsafe {
                let slice = c_arr.as_slice();
                for ffi_str in slice {
                    free_ffi_string(*ffi_str);
                }
                free_ffi_array(c_arr);
            }
        })
    });

    group.finish();
}

// ============================================================================
// Helper Functions
// ============================================================================

fn create_complex_config(num_layers: usize, mappings_per_layer: usize) -> ComplexConfig {
    let devices = vec![
        SimpleDevice {
            vendor_id: 0x1234,
            product_id: 0x5678,
            name: "Keyboard 1".to_string(),
        },
        SimpleDevice {
            vendor_id: 0xABCD,
            product_id: 0xEF01,
            name: "Keyboard 2".to_string(),
        },
    ];

    let mut settings = HashMap::new();
    settings.insert("timeout".to_string(), 200);
    settings.insert("combo_timeout".to_string(), 50);
    settings.insert("tap_hold_threshold".to_string(), 150);

    let layers: Vec<LayerConfig> = (0..num_layers)
        .map(|i| {
            let mut mappings = HashMap::new();
            for j in 0..mappings_per_layer {
                mappings.insert(format!("key_{}", j), format!("action_{}", j));
            }
            LayerConfig {
                id: i as u32,
                name: format!("Layer {}", i),
                mappings,
                transparent: i % 2 == 0,
            }
        })
        .collect();

    ComplexConfig {
        devices,
        settings,
        layers,
        enabled: true,
    }
}

// ============================================================================
// Criterion Configuration
// ============================================================================

criterion_group!(
    benches,
    benchmark_primitives,
    benchmark_strings,
    benchmark_arrays,
    benchmark_json_marshaling,
    benchmark_json_vs_c_struct,
    benchmark_estimated_size,
    benchmark_allocation_patterns
);

criterion_main!(benches);
