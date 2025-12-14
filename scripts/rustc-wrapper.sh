#!/usr/bin/env bash
set -euo pipefail

# Cargo invokes the wrapper with the real rustc path as the first argument.
real_rustc="$1"
shift

# Prefer sccache when available; otherwise fall back to rustc so builds never fail.
if command -v sccache >/dev/null 2>&1; then
  exec sccache "$real_rustc" "$@"
fi

exec "$real_rustc" "$@"
