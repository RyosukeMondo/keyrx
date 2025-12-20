#![allow(unsafe_code)]
//! FFI Tooling Performance Benchmarks
//!
//! Comprehensive benchmarks for the AI-native FFI workflow:
//! - Codegen performance (full run and incremental)
//! - Contract validation speed
//! - Watch mode overhead
//! - FFI call latency
//!
//! Performance Targets:
//! - Codegen full run: <5s
//! - Codegen incremental: <1s
//! - Watch mode overhead: <100ms
//! - FFI call latency: <1ms per function
//! - Contract validation: <500ms

// Allow unwrap/expect in benchmarks - panics are acceptable for setup code
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::print_stderr
)]

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

// ============================================================================
// Helper Functions
// ============================================================================

/// Get the project root directory
fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Get the codegen tool path
fn codegen_tool_path() -> PathBuf {
    let root = project_root();
    root.join("core/tools/generate_dart_bindings")
}

/// Check if codegen tool is built
fn ensure_codegen_tool_built() -> bool {
    let tool_path = codegen_tool_path();
    let target_path = tool_path.join("target/release/generate_dart_bindings");

    if !target_path.exists() {
        eprintln!("⚠ Codegen tool not built. Building...");
        let status = Command::new("cargo")
            .current_dir(&tool_path)
            .args(["build", "--release"])
            .status();

        match status {
            Ok(s) if s.success() => true,
            _ => {
                eprintln!("⚠ Failed to build codegen tool. Skipping codegen benchmarks.");
                false
            }
        }
    } else {
        true
    }
}

/// Get FFI source directory
fn ffi_source_dir() -> PathBuf {
    project_root().join("core/src/ffi")
}

/// Get contract output directory
fn contracts_dir() -> PathBuf {
    project_root().join("core/src/ffi/contracts")
}

// ============================================================================
// Benchmark: Codegen Full Run
// ============================================================================

fn benchmark_codegen_full_run(c: &mut Criterion) {
    if !ensure_codegen_tool_built() {
        eprintln!("Skipping codegen_full_run benchmark (tool not available)");
        return;
    }

    let mut group = c.benchmark_group("codegen");
    group.sample_size(10); // Reduce sample size for expensive operations
    group.measurement_time(Duration::from_secs(60)); // Allow more time

    let tool_path = codegen_tool_path().join("target/release/generate_dart_bindings");
    let root = project_root();

    group.bench_function("full_run", |b| {
        b.iter(|| {
            let output = Command::new(&tool_path)
                .current_dir(&root)
                .args(["--force"]) // Force full regeneration
                .output()
                .expect("Failed to run codegen tool");

            black_box(output.status.success())
        });
    });

    group.finish();
}

// ============================================================================
// Benchmark: Codegen Incremental
// ============================================================================

fn benchmark_codegen_incremental(c: &mut Criterion) {
    if !ensure_codegen_tool_built() {
        eprintln!("Skipping codegen_incremental benchmark (tool not available)");
        return;
    }

    let mut group = c.benchmark_group("codegen");
    group.sample_size(20);
    group.measurement_time(Duration::from_secs(30));

    let tool_path = codegen_tool_path().join("target/release/generate_dart_bindings");
    let root = project_root();

    // First run to populate cache
    let _ = Command::new(&tool_path)
        .current_dir(&root)
        .output()
        .expect("Failed to run initial codegen");

    group.bench_function("incremental_no_changes", |b| {
        b.iter(|| {
            let output = Command::new(&tool_path)
                .current_dir(&root)
                .output()
                .expect("Failed to run codegen tool");

            black_box(output.status.success())
        });
    });

    group.finish();
}

// ============================================================================
// Benchmark: Contract Validation
// ============================================================================

fn benchmark_contract_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("contract_validation");

    let validate_script = project_root().join("scripts/validate_ffi_contracts.sh");

    if !validate_script.exists() {
        eprintln!("Skipping contract_validation benchmark (script not found)");
        return;
    }

    group.sample_size(20);
    group.measurement_time(Duration::from_secs(30));

    group.bench_function("validate_all_contracts", |b| {
        b.iter(|| {
            let output = Command::new(&validate_script)
                .output()
                .expect("Failed to run validation script");

            black_box(output.status.success())
        });
    });

    group.finish();
}

// ============================================================================
// Benchmark: File Scanning Overhead
// ============================================================================

fn benchmark_file_scanning(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_scanning");

    let ffi_dir = ffi_source_dir();

    if !ffi_dir.exists() {
        eprintln!("Skipping file_scanning benchmark (FFI dir not found)");
        return;
    }

    group.bench_function("scan_ffi_modules", |b| {
        b.iter(|| {
            let mut count = 0;
            if let Ok(entries) = fs::read_dir(&ffi_dir) {
                for entry in entries.flatten() {
                    if let Ok(file_type) = entry.file_type() {
                        if file_type.is_file() {
                            let path = entry.path();
                            if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                                count += 1;
                            }
                        }
                    }
                }
            }
            black_box(count)
        });
    });

    // Recursive scanning
    group.bench_function("scan_ffi_modules_recursive", |b| {
        b.iter(|| {
            use walkdir::WalkDir;
            let count = WalkDir::new(&ffi_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("rs"))
                .count();
            black_box(count)
        });
    });

    group.finish();
}

// ============================================================================
// Benchmark: Contract Parsing
// ============================================================================

fn benchmark_contract_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("contract_parsing");

    let contracts_dir = contracts_dir();

    // Find sample contract files
    let sample_contracts: Vec<PathBuf> = if contracts_dir.exists() {
        fs::read_dir(&contracts_dir)
            .ok()
            .into_iter()
            .flat_map(|entries| entries.filter_map(|e| e.ok()))
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("json"))
            .map(|e| e.path())
            .take(5)
            .collect()
    } else {
        vec![]
    };

    if sample_contracts.is_empty() {
        eprintln!("Skipping contract_parsing benchmark (no contracts found)");
        return;
    }

    for contract_path in sample_contracts {
        let name = contract_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        group.bench_with_input(
            BenchmarkId::new("parse_contract", name),
            &contract_path,
            |b, path| {
                b.iter(|| {
                    if let Ok(content) = fs::read_to_string(path) {
                        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&content);
                        black_box(parsed.is_ok())
                    } else {
                        black_box(false)
                    }
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Benchmark: Module Size Analysis
// ============================================================================

fn benchmark_module_size_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("module_analysis");

    let analyze_script = project_root().join("scripts/analyze_module_sizes.sh");

    if !analyze_script.exists() {
        eprintln!("Skipping module_size_analysis benchmark (script not found)");
        return;
    }

    group.sample_size(10);
    group.measurement_time(Duration::from_secs(30));

    group.bench_function("analyze_all_modules", |b| {
        b.iter(|| {
            let output = Command::new(&analyze_script)
                .output()
                .expect("Failed to run analysis script");

            black_box(output.status.success())
        });
    });

    group.finish();
}

// ============================================================================
// Benchmark: FFI Function Call Overhead (Simulated)
// ============================================================================

fn benchmark_ffi_call_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("ffi_call_overhead");

    // Simulate FFI call overhead with tracing
    use tracing::info_span;

    group.bench_function("no_tracing", |b| {
        b.iter(|| {
            // Simulate a simple FFI function
            let result = simple_ffi_operation(42);
            black_box(result)
        });
    });

    group.bench_function("with_tracing", |b| {
        b.iter(|| {
            let _span = info_span!("ffi_call", domain = "test", function = "simple").entered();
            let result = simple_ffi_operation(42);
            black_box(result)
        });
    });

    group.bench_function("with_detailed_tracing", |b| {
        b.iter(|| {
            let _span = info_span!(
                "ffi_call",
                domain = "test",
                function = "simple",
                param = 42,
                correlation_id = "test-123"
            )
            .entered();
            let result = simple_ffi_operation(42);
            black_box(result)
        });
    });

    group.finish();
}

// Simulated FFI operation
#[inline(never)]
fn simple_ffi_operation(value: i32) -> i32 {
    value * 2
}

// ============================================================================
// Benchmark: Breaking Change Detection
// ============================================================================

fn benchmark_breaking_change_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("breaking_change_detection");

    let detect_script = project_root().join("scripts/detect_ffi_breaking_changes.sh");

    if !detect_script.exists() {
        eprintln!("Skipping breaking_change_detection benchmark (script not found)");
        return;
    }

    group.sample_size(10);
    group.measurement_time(Duration::from_secs(30));

    group.bench_function("detect_breaking_changes", |b| {
        b.iter(|| {
            let output = Command::new(&detect_script)
                .output()
                .expect("Failed to run detection script");

            black_box(output.status.success())
        });
    });

    group.finish();
}

// ============================================================================
// Criterion Configuration
// ============================================================================

criterion_group!(
    codegen_benches,
    benchmark_codegen_full_run,
    benchmark_codegen_incremental,
);

criterion_group!(
    validation_benches,
    benchmark_contract_validation,
    benchmark_breaking_change_detection,
);

criterion_group!(
    analysis_benches,
    benchmark_file_scanning,
    benchmark_contract_parsing,
    benchmark_module_size_analysis,
);

criterion_group!(overhead_benches, benchmark_ffi_call_overhead,);

criterion_main!(
    codegen_benches,
    validation_benches,
    analysis_benches,
    overhead_benches
);
