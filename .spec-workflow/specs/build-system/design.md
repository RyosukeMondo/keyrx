# Design Document

## Overview

This design implements a comprehensive build system for KeyRx, including a task runner (just), CI/CD pipelines (GitHub Actions), and developer experience tooling. The system provides one-command setup, automated quality gates, cross-platform builds, and release automation.

## Steering Document Alignment

### Technical Standards (tech.md)

- **Build Profiles**: Implements dev, release, and profiling profiles as specified
- **Feature Flags**: Uses documented feature flags (linux, windows, tracing, profiling)
- **Cross-Platform**: Targets documented platforms (Linux x64, Windows x64, with ARM64 planned)
- **CI Pipeline**: Follows documented pipeline architecture (ci.yml, release.yml, maintenance.yml)
- **Developer Tools**: Installs documented tools (just, cargo-nextest, cargo-watch, etc.)

### Project Structure (structure.md)

```
keyrx/
├── justfile                    # Task runner (NEW)
├── cliff.toml                  # Changelog generator config (NEW)
├── .github/
│   └── workflows/
│       ├── ci.yml              # Enhanced CI (MODIFY)
│       ├── release.yml         # Release automation (NEW)
│       └── maintenance.yml     # Maintenance automation (NEW)
├── .vscode/
│   ├── settings.json           # IDE settings (NEW)
│   ├── extensions.json         # Recommended extensions (NEW)
│   └── launch.json             # Debug configurations (NEW)
├── .devcontainer/
│   └── devcontainer.json       # Dev container config (NEW)
├── scripts/
│   ├── install-hooks.sh        # Pre-commit hook installer (EXISTS)
│   └── release.sh              # Release helper script (NEW)
└── core/
    └── .config/
        └── nextest.toml        # Nextest CI profile (NEW)
```

## Code Reuse Analysis

### Existing Components to Leverage

- **scripts/install-hooks.sh**: Existing hook installer - will be enhanced with better error handling
- **.github/workflows/ci.yml**: Existing CI workflow - will be significantly enhanced
- **core/benches/latency.rs**: Existing benchmark - will be integrated into CI regression detection

### Integration Points

- **Cargo.toml**: Add release profile optimizations
- **pubspec.yaml**: Version synchronization for releases
- **pre-commit hook**: Already exists, will be formalized

## Architecture

### Build System Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           Developer Workstation                          │
├─────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐     │
│  │    justfile     │    │   Pre-commit    │    │   IDE Config    │     │
│  │   Task Runner   │    │     Hooks       │    │    (.vscode)    │     │
│  └────────┬────────┘    └────────┬────────┘    └─────────────────┘     │
│           │                      │                                       │
│           ▼                      ▼                                       │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                    Local Development Loop                        │   │
│  │  just setup → just dev → just check → git commit → pre-commit   │   │
│  └─────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    │ git push
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                           GitHub Actions                                 │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  on: push/PR                          on: tag v*                         │
│  ┌─────────────────────┐              ┌─────────────────────┐           │
│  │      ci.yml         │              │    release.yml      │           │
│  │  ┌───────────────┐  │              │  ┌───────────────┐  │           │
│  │  │ check (fast)  │  │              │  │ build-linux   │  │           │
│  │  │ fmt + clippy  │  │              │  │ build-windows │  │           │
│  │  └───────┬───────┘  │              │  └───────┬───────┘  │           │
│  │          ▼          │              │          ▼          │           │
│  │  ┌───────────────┐  │              │  ┌───────────────┐  │           │
│  │  │ test matrix   │  │              │  │   package     │  │           │
│  │  │ linux/windows │  │              │  │ tar.gz / zip  │  │           │
│  │  └───────┬───────┘  │              │  └───────┬───────┘  │           │
│  │          ▼          │              │          ▼          │           │
│  │  ┌───────────────┐  │              │  ┌───────────────┐  │           │
│  │  │ coverage      │  │              │  │   publish     │  │           │
│  │  │ security      │  │              │  │ GitHub Release│  │           │
│  │  │ benchmark     │  │              │  └───────────────┘  │           │
│  │  └───────────────┘  │              │                     │           │
│  └─────────────────────┘              └─────────────────────┘           │
│                                                                          │
│  on: schedule (weekly)                                                   │
│  ┌─────────────────────┐                                                │
│  │  maintenance.yml    │                                                │
│  │  deps update + PR   │                                                │
│  │  security audit     │                                                │
│  └─────────────────────┘                                                │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### Modular Design Principles

- **Single File Responsibility**: Each workflow handles one concern (CI, release, maintenance)
- **Component Isolation**: justfile recipes are independent and composable
- **Fail Fast**: Lint checks run before expensive test jobs
- **Caching**: All workflows use appropriate caching strategies

## Components and Interfaces

### Component 1: justfile (Task Runner)

**Purpose:** Provide unified, discoverable commands for all development workflows

**File:** `justfile`

**Interfaces (Recipes):**
```just
# Development
setup          # One-command environment setup
dev            # Watch mode for Rust (clippy + tests)
ui             # Run Flutter with hot reload

# Quality
check          # Run all checks (fmt + clippy + test)
fmt            # Format all code
clippy         # Run clippy lints
test           # Run all tests

# Build
build          # Build release for current platform
build-all      # Build for all platforms
build-linux    # Build Linux x64
build-windows  # Build Windows x64 (via cross)

# Release
bench          # Run latency benchmarks
release ver    # Prepare release with version
```

**Dependencies:** just, cargo, flutter, cross

### Component 2: CI Workflow (ci.yml)

**Purpose:** Automated quality gates on every push and PR

**File:** `.github/workflows/ci.yml`

**Jobs:**
```yaml
jobs:
  check:        # Fast: fmt + clippy (2 min)
  test:         # Matrix: ubuntu + windows (5 min)
  test-flutter: # Flutter analyze + test (3 min)
  coverage:     # cargo-llvm-cov → Codecov (4 min)
  security:     # cargo audit (1 min)
  benchmark:    # PR only: criterion regression (3 min)
```

**Dependencies:** dtolnay/rust-toolchain, Swatinem/rust-cache, subosito/flutter-action, codecov/codecov-action

### Component 3: Release Workflow (release.yml)

**Purpose:** Automated cross-platform builds and GitHub Release publishing

**File:** `.github/workflows/release.yml`

**Jobs:**
```yaml
jobs:
  build:        # Matrix: linux-x64, windows-x64
    steps:
      - Build Rust core
      - Build Flutter UI
      - Package artifacts (tar.gz/zip)
      - Upload artifacts

  publish:      # Depends on build
    steps:
      - Download all artifacts
      - Generate release notes
      - Create GitHub Release
```

**Dependencies:** softprops/action-gh-release

### Component 4: Maintenance Workflow (maintenance.yml)

**Purpose:** Automated dependency updates and security monitoring

**File:** `.github/workflows/maintenance.yml`

**Jobs:**
```yaml
jobs:
  update-dependencies:  # cargo update + create PR
  security-audit:       # cargo audit with notifications
```

**Dependencies:** peter-evans/create-pull-request, rustsec/audit-check

### Component 5: Pre-commit Hook

**Purpose:** Block commits that don't pass quality checks

**File:** `.git/hooks/pre-commit` (installed by `scripts/install-hooks.sh`)

**Interfaces:**
```bash
# Checks run in order (fail fast)
1. cargo fmt --check
2. cargo clippy -- -D warnings
3. cargo test --lib
```

**Exit Codes:** 0 = pass, 1 = fail

### Component 6: IDE Configuration

**Purpose:** Optimal VS Code setup for Rust + Flutter development

**Files:**
- `.vscode/settings.json` - rust-analyzer, formatOnSave, clippy
- `.vscode/extensions.json` - recommended extensions list
- `.vscode/launch.json` - debug configurations

### Component 7: Development Container

**Purpose:** Reproducible development environment

**File:** `.devcontainer/devcontainer.json`

**Features:**
- Rust stable toolchain
- just task runner
- Auto-runs `just setup` on create

### Component 8: Nextest Configuration

**Purpose:** Fast test runner with CI-specific profile

**File:** `core/.config/nextest.toml`

**Profiles:**
```toml
[profile.default]
retries = 0

[profile.ci]
retries = 2
fail-fast = false
```

### Component 9: Changelog Generator (cliff.toml)

**Purpose:** Generate changelog from conventional commits

**File:** `cliff.toml`

**Commit Parsing:**
- `feat` → Added
- `fix` → Fixed
- `perf` → Performance
- `refactor` → Changed

## Data Models

### Nextest Config (TOML)

```toml
[profile.default]
retries = 0
slow-timeout = { period = "60s" }

[profile.ci]
retries = 2
fail-fast = false
```

### Devcontainer Config (JSON)

```json
{
  "name": "KeyRx Development",
  "image": "mcr.microsoft.com/devcontainers/rust:1-bookworm",
  "features": {
    "ghcr.io/devcontainers/features/rust:1": {},
    "ghcr.io/aspect-build/aspect-workflows-images/features/just:1": {}
  },
  "postCreateCommand": "just setup"
}
```

### VS Code Settings (JSON)

```json
{
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.check.command": "clippy",
  "editor.formatOnSave": true
}
```

### Cliff Config (TOML)

```toml
[changelog]
header = "# Changelog\n"
body = "{% for group, commits in commits %}..."

[git]
conventional_commits = true
commit_parsers = [
  { message = "^feat", group = "Added" },
  { message = "^fix", group = "Fixed" }
]
```

## Error Handling

### Error Scenarios

1. **Pre-commit hook fails**
   - **Handling:** Display specific failure (fmt/clippy/test) with fix command
   - **User Impact:** Commit blocked, user runs suggested fix command

2. **CI workflow fails**
   - **Handling:** Job-level failure with logs, GitHub status check blocks merge
   - **User Impact:** PR cannot be merged until fixed

3. **Release build fails**
   - **Handling:** Workflow fails, no GitHub Release created
   - **User Impact:** Maintainer investigates logs, re-triggers after fix

4. **Benchmark regression detected**
   - **Handling:** CI warning annotation (not failure by default)
   - **User Impact:** PR author reviews benchmark results

5. **Security vulnerability found**
   - **Handling:** `cargo audit` fails CI, weekly audit sends notification
   - **User Impact:** Team addresses vulnerability before merge

6. **Setup fails (missing tool)**
   - **Handling:** Clear error message with installation command
   - **User Impact:** User runs suggested installation command

## Testing Strategy

### Unit Testing

Not applicable - build system is tested via integration.

### Integration Testing

1. **Pre-commit hook test:**
   ```bash
   # Test that hook blocks bad commits
   echo "bad code" >> test.rs
   git commit -m "test" # Should fail
   ```

2. **justfile recipe test:**
   ```bash
   just check  # Should pass on clean repo
   ```

3. **CI workflow test:**
   - Push to feature branch
   - Verify all jobs pass
   - Verify caching works (second run faster)

### End-to-End Testing

1. **Full release flow:**
   ```bash
   git tag v0.0.1-test
   git push origin v0.0.1-test
   # Verify GitHub Release created with artifacts
   git push --delete origin v0.0.1-test
   ```

2. **New contributor setup:**
   ```bash
   git clone ...
   just setup
   just check  # Should pass
   ```

## Implementation Sequence

1. **justfile** - Foundation for all other tasks
2. **nextest.toml** - CI test profile
3. **ci.yml enhancement** - Full CI pipeline
4. **Pre-commit hook update** - Better error messages
5. **VS Code config** - IDE setup
6. **devcontainer.json** - Container environment
7. **cliff.toml** - Changelog generation
8. **release.yml** - Release automation
9. **maintenance.yml** - Maintenance automation
10. **release.sh** - Release helper script
