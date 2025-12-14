#!/usr/bin/env bash
# Automated developer environment setup for KeyRx.
# Installs common tooling (rustfmt, clippy, cargo-nextest, watchexec, sccache, lcov),
# installs project dependencies, runs initial builds, and installs git hooks.
# Usage: ./scripts/setup_dev_env.sh [--skip-build] [--skip-hooks] [--skip-flutter] [--skip-rust]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

SKIP_BUILD=0
SKIP_HOOKS=0
SKIP_FLUTTER=0
SKIP_RUST=0

log() { echo "[setup] $*"; }
warn() { echo "[setup][warn] $*" >&2; }

usage() {
    cat <<'EOF'
Usage: setup_dev_env.sh [options]
  --skip-build     Skip initial Rust/Flutter build steps
  --skip-hooks     Skip installing git hooks
  --skip-flutter   Skip Flutter dependency/build steps
  --skip-rust      Skip Rust dependency/build steps
  --help           Show this help
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --skip-build) SKIP_BUILD=1 ;;
        --skip-hooks) SKIP_HOOKS=1 ;;
        --skip-flutter) SKIP_FLUTTER=1 ;;
        --skip-rust) SKIP_RUST=1 ;;
        --help) usage; exit 0 ;;
        *) warn "Unknown option: $1"; usage; exit 1 ;;
    esac
    shift
done

os_family() {
    case "$(uname -s)" in
        Linux*) echo "linux" ;;
        Darwin*) echo "darwin" ;;
        *) echo "other" ;;
    esac
}

ensure_rust_component() {
    local component="$1"
    if ! command -v rustup >/dev/null 2>&1; then
        warn "rustup not found; skipping rust component install for $component"
        return
    fi
    if rustup component list --installed | grep -q "^${component}[[:space:]]"; then
        log "rust component ${component} already installed"
    else
        log "Installing rust component ${component}..."
        rustup component add "$component"
    fi
}

ensure_cargo_tool() {
    local bin_name="$1"
    local crate_name="${2:-$1}"
    if command -v "$bin_name" >/dev/null 2>&1; then
        log "$bin_name already installed"
        return
    fi
    if ! command -v cargo >/dev/null 2>&1; then
        warn "cargo not found; cannot install $crate_name"
        return
    fi
    log "Installing $crate_name via cargo..."
    cargo install "$crate_name" --locked || warn "Failed to install $crate_name (continuing)"
}

ensure_package() {
    local name="$1"
    local apt_pkg="${2:-$1}"
    local brew_pkg="${3:-$1}"
    if command -v "$name" >/dev/null 2>&1; then
        log "$name already installed"
        return 0
    fi

    if command -v apt-get >/dev/null 2>&1; then
        log "Installing $apt_pkg via apt-get..."
        sudo apt-get update -y >/dev/null
        sudo apt-get install -y "$apt_pkg"
        return 0
    elif command -v brew >/dev/null 2>&1; then
        log "Installing $brew_pkg via Homebrew..."
        brew install "$brew_pkg"
        return 0
    fi

    warn "No supported package manager found for $name"
    return 1
}

log "Starting KeyRx developer environment setup..."
log "Repository root: $REPO_ROOT"

if [[ $SKIP_RUST -eq 0 ]]; then
    ensure_rust_component rustfmt
    ensure_rust_component clippy
    ensure_cargo_tool cargo-nextest cargo-nextest
    ensure_cargo_tool cargo-watch cargo-watch
    ensure_cargo_tool watchexec-cli watchexec-cli
    ensure_cargo_tool sccache sccache
fi

# Install coverage tooling and lcov.
"$SCRIPT_DIR/install-coverage-tools.sh"

# Ensure lcov exists (install-coverage-tools handles install when missing).
if ! command -v lcov >/dev/null 2>&1; then
    ensure_package lcov lcov lcov || warn "lcov not installed; coverage HTML reports may fail"
fi

if [[ $SKIP_RUST -eq 0 ]]; then
    if command -v cargo >/dev/null 2>&1; then
        log "Fetching Rust dependencies..."
        (cd "$REPO_ROOT/core" && cargo fetch)
    else
        warn "cargo not found; skipping Rust dependency install"
    fi
fi

if [[ $SKIP_FLUTTER -eq 0 ]]; then
    if command -v flutter >/dev/null 2>&1; then
        log "Fetching Flutter dependencies..."
        (cd "$REPO_ROOT/ui" && flutter pub get)
        log "Running build_runner to prime generated code..."
        (cd "$REPO_ROOT/ui" && flutter pub run build_runner build --delete-conflicting-outputs)
    else
        warn "flutter not found; skipping Flutter dependency install"
    fi
fi

if [[ $SKIP_HOOKS -eq 0 ]]; then
    if [[ -x "$SCRIPT_DIR/install-hooks.sh" ]]; then
        log "Installing git hooks..."
        "$SCRIPT_DIR/install-hooks.sh"
    else
        warn "install-hooks.sh not found or not executable; skipping hook install"
    fi
fi

if [[ $SKIP_BUILD -eq 0 ]]; then
    if [[ $SKIP_RUST -eq 0 && -d "$REPO_ROOT/core" && -x "$(command -v cargo || true)" ]]; then
        log "Running initial Rust build..."
        if command -v sccache >/dev/null 2>&1; then
            export RUSTC_WRAPPER="${RUSTC_WRAPPER:-sccache}"
        fi
        (cd "$REPO_ROOT/core" && cargo build --workspace)
    else
        warn "Skipping Rust build (missing cargo or SKIP_RUST enabled)"
    fi

    if [[ $SKIP_FLUTTER -eq 0 && -d "$REPO_ROOT/ui" && -x "$(command -v flutter || true)" ]]; then
        log "Running initial Flutter test build (compiles code)..."
        (cd "$REPO_ROOT/ui" && flutter test --concurrency "${CONCURRENCY:-4}")
    else
        warn "Skipping Flutter build (missing flutter or SKIP_FLUTTER enabled)"
    fi
else
    log "Skipping initial build steps (--skip-build provided)"
fi

log "Developer environment setup complete."
