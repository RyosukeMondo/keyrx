#!/usr/bin/env bash
# Install FFI-focused tooling and fast linkers for Rust-first development.
# - Installs: cargo-watch, tokei (metrics), mold (Linux), zld (macOS)
# - Configures linker preference in .cargo/config.toml when available
# - Safe to re-run; skips already installed tools

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CONFIG_FILE="$REPO_ROOT/.cargo/config.toml"

log() { echo "[ffi-setup] $*"; }
warn() { echo "[ffi-setup][warn] $*" >&2; }

usage() {
    cat <<'EOF'
Usage: setup_ffi_tools.sh [options]
  --skip-linker   Skip fast-linker install/configuration
  --help          Show this help

Notes:
- Idempotent: safe to rerun; already-installed tools are skipped.
- Requires cargo for Rust tool installs. Linker installs rely on the host package manager.
EOF
}

OS_FAMILY="other"
case "$(uname -s)" in
    Linux*) OS_FAMILY="linux" ;;
    Darwin*) OS_FAMILY="darwin" ;;
    CYGWIN*|MINGW*|MSYS_NT*) OS_FAMILY="windows" ;;
esac

SKIP_LINKER=0
while [[ $# -gt 0 ]]; do
    case "$1" in
        --skip-linker) SKIP_LINKER=1 ;;
        --help) usage; exit 0 ;;
        *) warn "Unknown option: $1"; usage; exit 1 ;;
    esac
    shift
done

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
    local choco_pkg="${4:-$1}"

    if command -v "$name" >/dev/null 2>&1; then
        log "$name already installed"
        return 0
    fi

    if command -v apt-get >/dev/null 2>&1; then
        log "Installing $apt_pkg via apt-get..."
        sudo apt-get update -y >/dev/null
        sudo apt-get install -y "$apt_pkg" && return 0
    fi

    if command -v dnf >/dev/null 2>&1; then
        log "Installing $apt_pkg via dnf..."
        sudo dnf install -y "$apt_pkg" && return 0
    fi

    if command -v pacman >/dev/null 2>&1; then
        log "Installing $apt_pkg via pacman..."
        sudo pacman -Sy --noconfirm "$apt_pkg" && return 0
    fi

    if command -v brew >/dev/null 2>&1; then
        log "Installing $brew_pkg via Homebrew..."
        brew install "$brew_pkg" && return 0
    fi

    if command -v choco >/dev/null 2>&1; then
        log "Installing $choco_pkg via Chocolatey..."
        choco install -y "$choco_pkg" && return 0
    fi

    if command -v winget >/dev/null 2>&1; then
        log "Installing $name via winget..."
        winget install -e --id "$name" && return 0
    fi

    warn "No supported package manager found for $name"
    return 1
}

configure_linker() {
    local linker_bin="$1"
    local target_triple
    local linker_path="$linker_bin"

    if [[ "$linker_bin" == /* ]]; then
        linker_path="$linker_bin"
    elif [[ "$linker_bin" == */* ]]; then
        linker_path="$REPO_ROOT/$linker_bin"
    fi

    if [[ $SKIP_LINKER -eq 1 ]]; then
        log "Skipping linker configuration (--skip-linker)"
        return
    fi

    if ! command -v "$linker_path" >/dev/null 2>&1; then
        if [[ -x "$linker_path" ]]; then
            : # executable path provided; continue
        else
            warn "$linker_bin not found; skipping linker config"
            return
        fi
    fi

    if command -v rustc >/dev/null 2>&1; then
        target_triple="$(rustc -vV | awk '/host/ {print $2}')"
    else
        warn "rustc not found; cannot detect host triple for linker config"
        return
    fi

    mkdir -p "$(dirname "$CONFIG_FILE")"
    if [[ ! -f "$CONFIG_FILE" ]]; then
        log "Creating $CONFIG_FILE"
        cat <<EOF > "$CONFIG_FILE"
[build]
incremental = true

EOF
    fi

    if grep -q "\\[target.${target_triple//\./\\.}\\]" "$CONFIG_FILE"; then
        if grep -A2 "\\[target.${target_triple//\./\\.}\\]" "$CONFIG_FILE" | grep -q "linker *= *\"$linker_bin\""; then
            log "Linker already configured for $target_triple -> $linker_bin"
            return
        fi
        warn "Target $target_triple already has a linker configured; leaving unchanged"
        return
    fi

    {
        echo ""
        echo "# Prefer fast linker for local dev on $target_triple"
        echo "[target.$target_triple]"
        echo "linker = \"$linker_bin\""
    } >> "$CONFIG_FILE"
    log "Configured $linker_bin for target $target_triple in $CONFIG_FILE"
}

install_fast_linker() {
    case "$OS_FAMILY" in
        linux)
            ensure_package mold mold mold || warn "mold install skipped/failed"
            configure_linker "scripts/fast_linker.sh"
            ;;
        darwin)
            # Homebrew formula lives in custom tap.
            if ! command -v brew >/dev/null 2>&1; then
                warn "Homebrew not found; install brew then rerun for zld"
                return
            fi
            brew list michaeleisel/zld/zld >/dev/null 2>&1 || brew install michaeleisel/zld/zld
            configure_linker "scripts/fast_linker.sh"
            ;;
        windows)
            warn "Fast linker auto-install not supported on Windows; install lld/zld manually if desired"
            ;;
        *)
            warn "Unknown OS; skipping fast linker install"
            ;;
    esac
}

log "Setting up FFI tooling (OS: $OS_FAMILY) at $REPO_ROOT"

ensure_cargo_tool cargo-watch cargo-watch
ensure_cargo_tool tokei tokei

if [[ $SKIP_LINKER -eq 0 ]]; then
    install_fast_linker
else
    log "Linker installation skipped by flag"
fi

log "FFI tooling setup complete."
