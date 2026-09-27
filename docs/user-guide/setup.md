# Development Environment Setup

Quick guide for setting up the KeyRx development environment for autonomous AI coding agents.

## TL;DR - Quick Setup

```bash
# Comprehensive setup (recommended, requires sudo)
make setup

# OR quick setup (no sudo, Cargo tools only)
make setup-quick
```

## What Gets Installed

### System Dependencies (requires sudo)
- **build-essential**: C/C++ compiler for native dependencies
- **pkg-config**: Package configuration tool
- **libssl-dev**: OpenSSL development files
- **libevdev-dev**: evdev development files (Linux keyboard input)
- **libudev-dev**: udev development files
- **bats**: Bash Automated Testing System
- **jq**: JSON processor for parsing script output

### Rust Toolchain Components
- **nightly toolchain**: For benchmarks (`cargo bench`)
- **rustfmt**: Code formatter (quality gates)
- **clippy**: Linter (quality gates)

### Cargo Development Tools
- **cargo-watch**: Continuous build on file changes
- **cargo-tarpaulin**: Code coverage analysis (80% minimum required)
- **cargo-fuzz**: Fuzzing support for security testing
- **wasm-pack**: WASM compilation for browser simulator

### Git Hooks
- **pre-commit**: Runs `verify.sh --quiet` before every commit
  - Blocks commits that fail quality checks
  - Enforces: clippy, rustfmt, tests, coverage

## Prerequisites

Before running setup, ensure you have:

1. **Rust 1.70+** installed from https://rustup.rs
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Git** installed
   ```bash
   sudo apt-get install git
   ```

3. **Ubuntu 22.04+** (or compatible Debian-based distro)

## Setup Options

### Option 1: Comprehensive Setup (Recommended)

Installs everything including system dependencies:

```bash
make setup
```

**Requires**: sudo password for apt-get

**What it does**:
1. Checks prerequisites (Rust, Git, repository)
2. Installs system dependencies with apt-get
3. Installs Rust nightly toolchain
4. Installs Cargo development tools
5. Sets up Git pre-commit hooks
6. Verifies all installations
7. Creates `.dev-tools.md` summary
8. Tests basic workflow

**Time**: 5-10 minutes (depending on internet speed)

### Option 2: Quick Setup (No Sudo)

Installs only Cargo tools and Git hooks:

```bash
make setup-quick
```

**Requires**: No sudo

**What it does**:
1. Installs Cargo tools (cargo-watch, cargo-tarpaulin, cargo-fuzz, wasm-pack)
2. Sets up Git pre-commit hooks

**Missing**: System dependencies (BATS, jq, libevdev-dev, etc.)

**Use case**: When you don't have sudo access or want minimal setup

**Time**: 3-5 minutes

### Option 3: Manual Setup

If you prefer manual control:

```bash
# Run the comprehensive setup script directly
./scripts/setup_dev_environment.sh

# Or install individual tools
cargo install cargo-watch
cargo install cargo-tarpaulin
cargo install cargo-fuzz
cargo install wasm-pack
cargo install typeshare-cli

# Install system dependencies
sudo apt-get install build-essential pkg-config libssl-dev libevdev-dev libudev-dev \
    libgtk-3-dev libxdo-dev libayatana-appindicator3-dev bats jq

# Setup hooks
./scripts/setup_hooks.sh
```

## Verification

After setup, verify everything works:

```bash
# Check installed tools
cargo watch --version
cargo tarpaulin --version
cargo fuzz --version
wasm-pack --version
bats --version
jq --version

# Check Git hooks
ls -la .git/hooks/pre-commit

# Test basic workflow
make build
make test
```

## Troubleshooting

### Issue: "cargo-tarpaulin fails to compile"

**Solution**: Install OpenSSL development files
```bash
sudo apt-get install libssl-dev pkg-config
cargo install cargo-tarpaulin --force
```

### Issue: "BATS not found"

**Solution**: Install BATS manually
```bash
sudo apt-get install bats
```

### Issue: "Pre-commit hook not running"

**Solution**: Reinstall hooks
```bash
./scripts/setup_hooks.sh
chmod +x .git/hooks/pre-commit
```

### Issue: "Permission denied when running scripts"

**Solution**: Make scripts executable
```bash
chmod +x scripts/*.sh
```

### Issue: "cargo-fuzz requires nightly"

**Solution**: Install nightly toolchain
```bash
rustup toolchain install nightly
```

## Post-Setup Workflow

Once setup is complete:

1. **Build workspace**:
   ```bash
   make build
   ```

2. **Run quality checks**:
   ```bash
   make verify
   ```

3. **Run tests**:
   ```bash
   make test
   ```

4. **Start development**:
   ```bash
   # Build with watch mode
   scripts/build.sh --watch
   ```

## For AI Coding Agents

All tools are designed for autonomous AI development:

- **CLI-first design**: Every operation has a command-line interface
- **Structured output**: Scripts support `--json` flag for machine-readable output
- **Status markers**: `=== accomplished ===`, `=== failed ===`, `=== warning ===`
- **Deterministic**: Same inputs always produce same outputs
- **Exit codes**: 0 = success, 1 = failure, 2 = warning
- **Pre-commit hooks**: Automatic quality enforcement
- **Comprehensive logging**: All operations logged with timestamps

See `.claude/CLAUDE.md` for complete AI agent development guide.

## Next Steps

After setup:

1. Read `.dev-tools.md` for tool usage details
2. Read `.claude/CLAUDE.md` for AI agent guidance
3. Run `make verify` to ensure everything works
4. Start coding!

## Support

If you encounter issues not covered here:

1. Check `.dev-tools.md` for tool-specific troubleshooting
2. Run `make verify` to diagnose issues
3. Check script logs in `scripts/logs/`
4. Verify Rust version: `rustc --version` (need 1.70+)
