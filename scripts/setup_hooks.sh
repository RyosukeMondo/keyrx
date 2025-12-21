#!/bin/bash
# Setup Git hooks for automated quality checks
# Installs pre-commit hook that runs verify.sh before allowing commits

set -euo pipefail

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Check if we're in a git repository
if [[ ! -d "$PROJECT_ROOT/.git" ]]; then
    echo "Error: Not a git repository. Cannot install git hooks." >&2
    echo "Please run this script from within a git repository." >&2
    exit 1
fi

echo "Installing git hooks..."

# Create pre-commit hook
PRE_COMMIT_HOOK="$PROJECT_ROOT/.git/hooks/pre-commit"

cat > "$PRE_COMMIT_HOOK" << 'EOF'
#!/bin/bash
# Pre-commit hook: Run verification before allowing commit
# This ensures all code meets quality standards before being committed

set -euo pipefail

echo "Running pre-commit verification..."
echo "This may take a moment. Please wait..."
echo ""

# Run verification script
if ! scripts/verify.sh --quiet; then
    echo ""
    echo "==================================================================="
    echo "PRE-COMMIT VERIFICATION FAILED"
    echo "==================================================================="
    echo ""
    echo "Your commit has been blocked because the code does not meet"
    echo "quality standards. Please fix the issues above and try again."
    echo ""
    echo "To see detailed output, run:"
    echo "  scripts/verify.sh"
    echo ""
    echo "To skip this hook (NOT recommended), use:"
    echo "  git commit --no-verify"
    echo "==================================================================="
    exit 1
fi

echo ""
echo "✓ Pre-commit verification passed!"
echo ""
exit 0
EOF

# Make the hook executable
chmod +x "$PRE_COMMIT_HOOK"

echo "✓ Pre-commit hook installed successfully at: $PRE_COMMIT_HOOK"
echo ""
echo "The hook will run the following checks before each commit:"
echo "  - Build verification (cargo build)"
echo "  - Linting (cargo clippy)"
echo "  - Formatting (cargo fmt --check)"
echo "  - Tests (cargo test)"
echo "  - Code coverage (cargo tarpaulin, 80% minimum)"
echo ""
echo "To bypass the hook (not recommended), use: git commit --no-verify"
echo ""
echo "Setup complete!"
