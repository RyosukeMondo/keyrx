# Contributing to KeyRX2

Thank you for your interest in contributing to KeyRX2! This document provides guidelines for contributing to the project.

## Development Setup

See the [AI Agent Development Guide](/.claude/CLAUDE.md) for comprehensive development setup instructions.

## Pre-Commit Hooks

This project uses [Husky](https://typicode.github.io/husky/) and [lint-staged](https://github.com/okonet/lint-staged) to run automated quality checks before each commit.

### What Gets Checked

When you commit changes, the following checks run automatically:

**For Rust files (*.rs):**
- `cargo fmt --check --` - Verifies code formatting
- `cargo clippy --all-targets -- -D warnings` - Runs linter (treats warnings as errors)

**For TypeScript/JavaScript files (keyrx_ui/**/*.{ts,tsx,js,jsx}):**
- `prettier --check` - Verifies code formatting
- `eslint` - Runs linter

### If Checks Fail

If any check fails, your commit will be blocked. You'll see an error message showing what failed.

**To fix the issues:**

1. For Rust formatting errors: Run `cargo fmt` to auto-format
2. For Rust clippy warnings: Fix the warnings shown in the error message
3. For TypeScript formatting: Run `cd keyrx_ui && prettier --write .`
4. For ESLint errors: Run `cd keyrx_ui && eslint --fix .`

Then stage your changes and try committing again.

### Bypassing Hooks (Not Recommended)

In rare cases where you need to bypass the pre-commit hooks (e.g., work-in-progress commits), you can use:

```bash
git commit --no-verify -m "WIP: your message"
```

**⚠️ Warning:** Only use `--no-verify` for legitimate reasons:
- Work-in-progress commits in a feature branch
- Emergency hotfixes that will be cleaned up immediately
- Commits that intentionally add failing tests as placeholders

**Never** use `--no-verify` to:
- Avoid fixing legitimate code quality issues
- Commit code that doesn't meet project standards
- Bypass CI checks (the CI will still run and fail)

### Why We Use Pre-Commit Hooks

Pre-commit hooks provide fast feedback during development:
- Catch formatting and linting issues before they enter the codebase
- Reduce CI build failures
- Maintain consistent code quality across all contributors
- Save time by catching issues locally instead of in CI

### Manual Quality Checks

You can also run all quality checks manually at any time:

```bash
# Full verification (runs all checks)
make verify

# Or using the script directly
./scripts/verify.sh

# Individual checks
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
cd keyrx_ui && npm test
```

## Code Quality Standards

- **Rust**: Code must pass `cargo fmt` and `cargo clippy` with zero warnings
- **TypeScript**: Code must pass Prettier formatting and ESLint with zero errors
- **Test Coverage**: Minimum 80% overall, 90% for `keyrx_core`
- **File Size**: Maximum 500 lines per file (excluding comments/blanks)
- **Function Size**: Maximum 50 lines per function

See [CLAUDE.md](/.claude/CLAUDE.md) for complete development guidelines.

## Commit Message Format

We follow the Conventional Commits specification:

```
<type>(<scope>): <subject>

<body>

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Adding or updating tests
- `refactor`: Code refactoring
- `chore`: Maintenance tasks

## Questions?

If you have questions about contributing, please:
- Check the [AI Agent Development Guide](/.claude/CLAUDE.md)
- Open an issue on GitHub
- Review existing documentation in the `docs/` directory
