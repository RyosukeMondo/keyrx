# Rust Build Optimization (Keyrx Core)

Goal: cut Rust rebuild time from 30–60s to <5s with warm cache.

## Fast Rebuild Configuration
- Incremental builds: enabled in `Cargo.toml` `[profile.dev]` (`incremental = true`, high `codegen-units` for parallelism).
- sccache: configured in `.cargo/config.toml` via `scripts/rustc-wrapper.sh` (prefers `sccache`, falls back to `rustc`), cache size 10G, idle timeout 30m. Install sccache (`cargo install sccache` or system package) and ensure it is on PATH.
- Dev profile dep tweaks: dependencies compiled with `opt-level = 2` but incremental off to reduce cache churn; crate code stays incremental.
- Optional: set `SCCACHE_DIR` to a fast disk (e.g., tmpfs) if available.

## Measuring Rebuild Time
Use `hyperfine` for consistent timing:
```bash
hyperfine --warmup 1 \
  'cargo clean -p keyrx_core && cargo build -p keyrx_core' \
  'cargo build -p keyrx_core'
```
- First command captures cold cache; second measures warm incremental rebuild (target <5s).
- If sccache is installed, compare with and without cache:
```bash
SCCACHE_BUCKET=local-cache sccache --start-server
hyperfine --warmup 1 'cargo build -p keyrx_core'
sccache --show-stats
```

## FFI Module Decoupling
- Global FFI event registry now lives in `core/src/ffi/events.rs` (`global_event_registry`).
- Engine/domain changes no longer force rebuilds of exports that only depend on event routing, reducing invalidation scope when iterating on engine logic.

## Troubleshooting
- `sccache: command not found`: install and confirm PATH; builds still work without it.
- Cache misses: run `sccache --show-stats` and verify `cache_hit_rate` improves after a few builds.
- Slow rebuilds: ensure `CARGO_INCREMENTAL=1` (default) and `RUSTC_WRAPPER` not overridden; avoid unnecessary `cargo clean`.
