#!/usr/bin/env bash
#
# Install git hooks from .githooks/ to .git/hooks/
# Usage: ./scripts/install-hooks.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
GITHOOKS_DIR="$REPO_ROOT/.githooks"
GIT_HOOKS_DIR="$REPO_ROOT/.git/hooks"

# Check if .githooks directory exists
if [[ ! -d "$GITHOOKS_DIR" ]]; then
    echo "Error: .githooks directory not found at $GITHOOKS_DIR" >&2
    exit 1
fi

# Check if .git directory exists
if [[ ! -d "$REPO_ROOT/.git" ]]; then
    echo "Error: Not a git repository (no .git directory found)" >&2
    exit 1
fi

# Create .git/hooks if it doesn't exist
if [[ ! -d "$GIT_HOOKS_DIR" ]]; then
    mkdir -p "$GIT_HOOKS_DIR"
fi

# Track installed hooks
installed_hooks=()

# Copy all hooks from .githooks to .git/hooks
for hook in "$GITHOOKS_DIR"/*; do
    if [[ -f "$hook" ]]; then
        hook_name="$(basename "$hook")"
        dest="$GIT_HOOKS_DIR/$hook_name"

        cp "$hook" "$dest"
        chmod +x "$dest"
        installed_hooks+=("$hook_name")
    fi
done

# Print results
if [[ ${#installed_hooks[@]} -eq 0 ]]; then
    echo "No hooks found in .githooks/"
else
    echo "Hooks installed: ${installed_hooks[*]}"
fi
