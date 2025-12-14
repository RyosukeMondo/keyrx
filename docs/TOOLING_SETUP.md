# FFI Development Tooling Setup

This guide installs the FFI-focused tools used across the project: cargo-watch, fast linkers (mold/zld), tokei metrics, and supporting configuration.

## Quickstart

```bash
# Run full dev setup (includes standard dev env + FFI tooling)
just setup

# Or run only the FFI-specific installer
./scripts/setup_ffi_tools.sh
```

The script is idempotent—safe to re-run. Use `--skip-linker` to avoid linker installs (e.g., managed environments).

## What Gets Installed
- `cargo-watch` for fast rebuild/watch cycles.
- `tokei` for code metrics and size dashboards.
- Fast linkers:
  - Linux: `mold`
  - macOS: `zld` (via Homebrew tap `michaeleisel/zld/zld`)
- `.cargo/config.toml` targets set to use `scripts/fast_linker.sh`, which prefers mold/zld via the system C compiler and falls back to the default compiler/linker.

## Platform Notes

### Linux
- Requirements: `rustup`, `cargo`, and either `apt`, `dnf`, or `pacman`.
- Installer attempts `sudo apt-get install mold` (or dnf/pacman). If your distro uses a different package name, install manually then rerun.
- Verify:
  ```bash
  mold --version
  cargo watch --version
  ```

### macOS
- Requirements: Homebrew.
- Installs `michaeleisel/zld/zld` and configures `.cargo/config.toml` to use it.
- Verify:
  ```bash
  zld --version
  cargo watch --version
  ```

### Windows
- cargo-based tools install normally.
- Fast linker install is not automated; install `lld` or another preferred linker manually and adjust `.cargo/config.toml` if needed. The wrapper `scripts/fast_linker.sh` falls back to the system `ld` to avoid breakage.

## Troubleshooting
- **Package manager not found:** Install the appropriate package manager (apt/dnf/pacman/brew) or install tools manually, then rerun.
- **Linker missing during build:** Ensure `mold` (Linux) or `zld` (macOS) is in `PATH`, or temporarily run `./scripts/setup_ffi_tools.sh --skip-linker`.
- **Permission errors:** The installer uses `sudo` for system packages. If unavailable, install dependencies manually with your environment’s mechanism.
- **Slow links even after install:** Confirm `scripts/fast_linker.sh` is executable and referenced in `.cargo/config.toml`. Clean old artifacts with `cargo clean` and rebuild.

## Verification Checklist
- `cargo watch --version` returns successfully.
- `tokei --version` returns successfully.
- `mold --version` (Linux) or `zld --version` (macOS) returns successfully.
- Rust builds log show the fast linker (mold/zld); expect noticeably faster link times (2–3x vs. system ld on typical dev workloads).
