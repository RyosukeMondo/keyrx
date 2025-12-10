#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::print_stdout, clippy::print_stderr, clippy::field_reassign_with_default, clippy::useless_conversion, clippy::assertions_on_constants, clippy::manual_div_ceil, clippy::manual_strip, clippy::len_zero, clippy::redundant_closure, clippy::manual_range_contains, clippy::default_constructed_unit_structs, clippy::clone_on_copy, clippy::io_other_error, clippy::bool_assert_comparison, clippy::approx_constant, clippy::let_unit_value, clippy::while_let_on_iterator, clippy::await_holding_lock, clippy::unnecessary_cast, clippy::drop_non_drop, clippy::needless_range_loop, unused_imports, unused_variables, dead_code, unsafe_code, clippy::collapsible_if, clippy::bool_comparison, unexpected_cfgs)]
//! Unit tests for scripting::cache module.

use keyrx_core::scripting::cache::{cache_key, serialize_entry, ScriptCache};
use keyrx_core::scripting::cache::{CACHE_FILE_EXTENSION, CACHE_FORMAT_VERSION, CORE_VERSION};
use rhai::{Engine, AST};
use std::fs;
use std::time::Instant;
use tempfile::tempdir;

fn make_cache(max_size_bytes: usize) -> (ScriptCache, tempfile::TempDir) {
    let dir = tempdir().expect("tempdir");
    let cache = ScriptCache::with_limits(dir.path().to_path_buf(), max_size_bytes, 8);
    (cache, dir)
}

fn compile_ast_with_time(script: &str) -> (AST, u64) {
    let start = Instant::now();
    let ast = Engine::new().compile(script).expect("compile");
    let micros = start.elapsed().as_micros() as u64;
    (ast, micros)
}

fn compile_ast(script: &str) -> AST {
    compile_ast_with_time(script).0
}

fn serialized_len(script: &str, ast: &AST) -> usize {
    serialize_entry(&cache_key(script), script, ast, Some(0))
        .expect("serialize")
        .len()
}

#[test]
fn cache_hit_returns_ast() {
    let (cache, _dir) = make_cache(1024);
    let script = "let a = 1; a + 1;";
    let (ast, compile_micros) = compile_ast_with_time(script);

    cache.put(script, &ast, Some(compile_micros));

    assert!(cache.get(script).is_some());

    let stats = cache.stats();
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 0);
    assert_eq!(stats.entries, 1);
}

#[test]
fn cache_tracks_startup_savings() {
    let (cache, _dir) = make_cache(2048);
    let script = "let a = 1 + 2;";
    let (ast, _compile_micros) = compile_ast_with_time(script);

    cache.put(script, &ast, Some(50_000));
    assert!(cache.get(script).is_some());

    let stats = cache.stats();
    assert_eq!(stats.hits, 1);
    assert!(stats.startup_micros_saved > 0);
}

#[test]
fn cache_miss_records_stat() {
    let (cache, _dir) = make_cache(1024);
    assert!(cache.get("not cached").is_none());

    let stats = cache.stats();
    assert_eq!(stats.hits, 0);
    assert_eq!(stats.misses, 1);
}

#[test]
fn lru_eviction_respects_byte_budget() {
    let script_a = "let a = 11;";
    let script_b = "let b = 22;";
    let script_c = "let c = 33;";

    let (ast_a, micros_a) = compile_ast_with_time(script_a);
    let (ast_b, micros_b) = compile_ast_with_time(script_b);
    let (ast_c, micros_c) = compile_ast_with_time(script_c);

    let size_a = serialized_len(script_a, &ast_a);
    let size_b = serialized_len(script_b, &ast_b);
    let budget = size_a + size_b;
    let (cache, _dir) = make_cache(budget);

    cache.put(script_a, &ast_a, Some(micros_a));
    cache.put(script_b, &ast_b, Some(micros_b));
    cache.put(script_c, &ast_c, Some(micros_c));

    // Verify eviction happened by checking stats
    let stats = cache.stats();
    assert!(stats.evictions >= 1);
    assert!(stats.entries <= 2);
}

#[test]
fn cache_updates_recency_on_get() {
    let script_a = "let a = 100;";
    let script_b = "let b = 200;";
    let script_c = "let c = 300;";

    let (ast_a, micros_a) = compile_ast_with_time(script_a);
    let (ast_b, micros_b) = compile_ast_with_time(script_b);
    let (ast_c, micros_c) = compile_ast_with_time(script_c);
    let large_script = "let really_big_value = 1234567890;";
    let (large_ast, large_micros) = compile_ast_with_time(large_script);

    let budget = serialized_len(script_a, &ast_a)
        + serialized_len(script_b, &ast_b)
        + serialized_len(large_script, &large_ast) / 2;

    let (cache, _dir) = make_cache(budget);

    cache.put(script_a, &ast_a, Some(micros_a));
    cache.put(script_b, &ast_b, Some(micros_b));
    cache.put(script_c, &ast_c, Some(micros_c));

    // Access A to make it most recent
    assert!(cache.get(script_a).is_some());

    // Force eviction by exceeding byte budget with a larger script
    cache.put(large_script, &large_ast, Some(large_micros));

    // Verify that A is still cached (it was accessed more recently)
    assert!(cache.get(script_a).is_some());

    // Verify eviction happened
    let stats = cache.stats();
    assert!(stats.evictions >= 1);
    assert!(stats.entries <= 2);
}

#[test]
fn cache_clear_resets_state() {
    let (cache, dir) = make_cache(256);
    let script = "let x = 42;";
    let (ast, compile_micros) = compile_ast_with_time(script);
    cache.put(script, &ast, Some(compile_micros));
    assert!(cache.get(script).is_some());

    cache.clear();
    assert!(cache.get(script).is_none());

    let stats = cache.stats();
    assert_eq!(stats.entries, 0);
    assert_eq!(stats.hits, 0);
    assert_eq!(stats.misses, 1); // miss after clear
    assert!(!dir
        .path()
        .join(format!("{}.{}", cache_key(script), CACHE_FILE_EXTENSION))
        .exists());
}

#[test]
fn cache_key_is_content_addressable() {
    let script_a1 = "let x = 1 + 2;";
    let script_a2 = "let x = 1 + 2;";
    let script_b = "let x = 3 + 4;";

    let hash_a1 = cache_key(script_a1);
    let hash_a2 = cache_key(script_a2);
    let hash_b = cache_key(script_b);

    assert_eq!(hash_a1, hash_a2);
    assert_ne!(hash_a1, hash_b);
}

#[test]
fn cache_persists_ast_to_disk() {
    let dir = tempdir().expect("tempdir");
    let script = "let load_me = 10 + 5;";
    let (ast, compile_micros) = compile_ast_with_time(script);

    let cache = ScriptCache::with_limits(dir.path().to_path_buf(), 4096, 8);
    cache.put(script, &ast, Some(compile_micros));

    let reloaded = ScriptCache::with_limits(dir.path().to_path_buf(), 4096, 8);
    let retrieved = reloaded.get(script);
    assert!(retrieved.is_some());

    let stats = reloaded.stats();
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 0);
}

#[test]
fn incompatible_cache_version_is_ignored() {
    use keyrx_core::scripting::cache::PersistedAst;

    let dir = tempdir().expect("tempdir");
    let script = "let stale = 99;";

    let entry = PersistedAst {
        format_version: CACHE_FORMAT_VERSION + 1,
        core_version: CORE_VERSION.to_string(),
        hash: cache_key(script),
        script: script.to_string(),
        compile_micros: None,
    };

    let encoded = serde_json::to_vec(&entry).expect("encode");
    let path = dir
        .path()
        .join(format!("{}.{}", cache_key(script), CACHE_FILE_EXTENSION));
    fs::create_dir_all(dir.path()).expect("mkdirs");
    fs::write(&path, encoded).expect("write");

    let cache = ScriptCache::with_limits(dir.path().to_path_buf(), 4096, 8);
    assert!(cache.get(script).is_none());
    assert_eq!(cache.stats().misses, 1);
    assert!(!path.exists());
}
