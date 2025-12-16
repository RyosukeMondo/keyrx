#!/usr/bin/env bash
# Wrapper linker that prefers fast linkers when available (mold on Linux, zld on macOS)
# Falls back to system ld to avoid breaking environments where fast linkers are missing.

set -euo pipefail

OS="$(uname -s)"
CC_BIN="${CC:-cc}"

if [[ "$OS" == Darwin* ]]; then
    if command -v zld >/dev/null 2>&1; then
        exec "$CC_BIN" -fuse-ld="$(command -v zld)" "$@"
    fi
elif [[ "$OS" == Linux* ]]; then
    if command -v mold >/dev/null 2>&1; then
        exec "$CC_BIN" -fuse-ld=mold "$@"
    fi
    if command -v ld.lld >/dev/null 2>&1; then
        exec "$CC_BIN" -fuse-ld=lld "$@"
    fi
fi

exec "$CC_BIN" "$@"
