#!/usr/bin/env bash
#
# Release helper script for KeyRx
# Automates version bump, changelog generation, and tag creation
# Usage: ./scripts/release.sh <version>
# Example: ./scripts/release.sh 1.0.0

set -euo pipefail

VERSION="${1:-}"

# Validate version argument
if [[ -z "$VERSION" ]]; then
    echo "Error: Version argument required" >&2
    echo "Usage: $0 <version>" >&2
    echo "Example: $0 1.0.0" >&2
    exit 1
fi

# Validate semver format (major.minor.patch with optional prerelease)
if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$ ]]; then
    echo "Error: Invalid semver format '$VERSION'" >&2
    echo "Expected format: major.minor.patch (e.g., 1.0.0 or 1.0.0-beta.1)" >&2
    exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CARGO_TOML="$REPO_ROOT/core/Cargo.toml"
PUBSPEC_YAML="$REPO_ROOT/ui/pubspec.yaml"

echo "Releasing version $VERSION..."

# Update version in core/Cargo.toml
echo "Updating core/Cargo.toml..."
sed -i "s/^version = \"[^\"]*\"/version = \"$VERSION\"/" "$CARGO_TOML"

# Update version in ui/pubspec.yaml (preserve build number)
echo "Updating ui/pubspec.yaml..."
sed -i "s/^version: [0-9.]*+/version: $VERSION+/" "$PUBSPEC_YAML"

# Generate changelog using git-cliff
echo "Generating CHANGELOG.md..."
if command -v git-cliff &> /dev/null; then
    git-cliff --tag "v$VERSION" -o "$REPO_ROOT/CHANGELOG.md"
else
    echo "Warning: git-cliff not found, skipping changelog generation" >&2
fi

# Stage changes
echo "Staging changes..."
git add "$CARGO_TOML" "$PUBSPEC_YAML"
[[ -f "$REPO_ROOT/CHANGELOG.md" ]] && git add "$REPO_ROOT/CHANGELOG.md"

# Create commit
echo "Creating release commit..."
git commit -m "chore(release): v$VERSION"

# Create annotated tag
echo "Creating tag v$VERSION..."
git tag -a "v$VERSION" -m "Release v$VERSION"

echo ""
echo "Release v$VERSION prepared successfully!"
echo "Review the changes, then push with:"
echo "  git push origin main --tags"
