# Build Optimization (Rust + Flutter)

Goal: cut rebuild and feedback times below 5s warm-start for both Rust core and Flutter UI.

## Fast rebuild configuration (Rust)
- Incremental builds: enabled in `Cargo.toml` `[profile.dev]` (`incremental = true`, high `codegen-units` for parallelism).
- sccache: configured in `.cargo/config.toml` via `scripts/rustc-wrapper.sh` (prefers `sccache`, falls back to `rustc`), cache size 10G, idle timeout 30m. Install sccache (`cargo install sccache` or system package) and ensure it is on PATH.
- Dev profile dep tweaks: dependencies compiled with `opt-level = 2` but incremental off to reduce cache churn; crate code stays incremental.
- Optional: set `SCCACHE_DIR` to a fast disk (e.g., tmpfs) if available.

## Fast rebuild configuration (Flutter/Dart)
- Avoid `flutter clean` unless absolutely necessary; it invalidates cache and slows the loop.
- Run `just ui-setup` after pulling to refresh generated code without nuking caches.
- Use `just dev-mock` to iterate on UI without waiting on the Rust core.
- Keep `build_runner` outputs up to date: `cd ui && flutter pub run build_runner build --delete-conflicting-outputs` (already part of `ui-setup`).
- For performance tests and goldens, keep `CONCURRENCY` reasonable (2–4) to avoid CPU thrash that slows hot reload.

## Measuring rebuild time
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
- For Flutter, measure UI warm reload loops by timing `flutter test --concurrency 4` for the smallest suite you touch most often; keep it under ~10s.

## FFI module decoupling
- Global FFI event registry now lives in `core/src/ffi/events.rs` (`global_event_registry`).
- Engine/domain changes no longer force rebuilds of exports that only depend on event routing, reducing invalidation scope when iterating on engine logic.

## Troubleshooting
- `sccache: command not found`: install and confirm PATH; builds still work without it.
- Cache misses: run `sccache --show-stats` and verify `cache_hit_rate` improves after a few builds.
- Slow rebuilds: ensure `CARGO_INCREMENTAL=1` (default) and `RUSTC_WRAPPER` not overridden; avoid unnecessary `cargo clean`.
- Flutter rebuilds feel slow: drop test scope with `TEST_FILES=...`, keep goldens/benchmarks off the hot path unless needed, and avoid rebuilding generated files unless contracts changed.
