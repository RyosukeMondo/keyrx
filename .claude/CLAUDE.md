# AI Agent Development Guide

## Active Specs

- **uat-ui-fixes**: Dashboard virtual/physical indicator, device enable/disable toggle, profile inline edit + active indicator, config RPC fix, 256-layer display, key dropdown population. See `.spec-workflow/specs/uat-ui-fixes/tasks.md`

## macOS Testing (Three-Layer Strategy)

**Quick Start:**
```bash
./scripts/platform/macos/test_full.sh                    # Automated test runner (all layers)
cargo test -p keyrx_daemon --test macos_mock_tests  # Mock tests only (no permissions)
cargo test -p keyrx_daemon --test e2e_macos_basic   # E2E tests (requires Accessibility)
./scripts/platform/macos/check_permission.sh             # Check Accessibility permission
```

**Three Layers:**
1. **Mock Tests** - No permissions required (CI-friendly)
2. **E2E Tests** - Requires Accessibility permission (auto-skip in CI)
3. **Automated Runner** - Orchestrates all layers with intelligent handling

**Details:** `docs/development/MACOS_TESTING_GUIDE.md`

## Windows Testing on Linux (Vagrant VM)

**Quick Start:**
```bash
./scripts/platform/windows/test_vm.sh                    # Automated testing
cd vagrant/windows && vagrant up                # Manual control
vagrant winrm -c 'cd C:\vagrant_project; cargo test -p keyrx_daemon --features windows'
vagrant snapshot restore provisioned            # Restore clean state
```

**Details:** `vagrant/windows/README.md` and `docs/development/windows-vm-setup.md`

## Quick Start

### Setup & Verify
```bash
make setup      # Install tools (cargo-watch, tarpaulin, wasm-pack, hooks)
make build      # Build workspace
make test       # Run tests
make verify     # Full quality checks (clippy, fmt, tests, coverage)
```

### Environment Requirements
- Rust 1.70+
- Node.js 18+
- 80% test coverage minimum (90% for keyrx_core)

## Project Structure

### 4-Crate Workspace

| Crate | Type | Purpose |
|-------|------|---------|
| `keyrx_core` | Library (no_std) | Platform-agnostic remapping logic (rkyv, boomphf) |
| `keyrx_compiler` | Binary | Compile Rhai configs to .krx binaries |
| `keyrx_daemon` | Binary | OS keyboard interception + web server (evdev/Windows hooks, axum) |
| `keyrx_ui` | Frontend | React + WASM interface (TypeScript 5+, Vite) |

**Key Directories:**
- `scripts/` - Build/test/launch automation
- `.github/workflows/` - CI/CD (ci.yml, release.yml)
- `keyrx_ui/src/{components,wasm,hooks}` - Frontend code
- `keyrx_daemon/src/platform/` - OS-specific implementations

## Code Quality Rules

### Limits (Enforced by clippy/pre-commit)
- **500 lines max per file** (excluding comments/blanks)
- **50 lines max per function**
- **80% test coverage minimum** (90% for keyrx_core)

### Quality Checks
```bash
cargo clippy --workspace -- -D warnings    # No warnings allowed
cargo fmt --check                          # Format check
cargo test --workspace                     # All tests pass
```

### Production Quality Gates

| Quality Gate | Threshold | Current | Enforcement |
|--------------|-----------|---------|-------------|
| Backend Tests | 100% pass | ✅ 962/962 | Strict |
| Backend Doc Tests | 100% pass | ✅ 9/9 | Strict |
| Frontend Tests | ≥95% pass | ⚠️ 681/897 (75.9%) | Warning* |
| Frontend Coverage | ≥80% line/branch | ⚠️ Blocked | Warning* |
| Accessibility | Zero WCAG violations | ✅ 23/23 | Strict |

*Will become strict after WebSocket infrastructure fixes

**Run all gates locally:**
```bash
make verify && scripts/fix_doc_tests.sh
cd keyrx_ui && npm test && npm run test:coverage && npm run test:a11y
```

## Architecture Patterns

### SOLID Principles

**Single Responsibility:** Each module has one purpose (lookup.rs = MPHF only, dfa.rs = DFA only)

**Open/Closed:** Extend via traits, add platforms without changing core

**Dependency Inversion:** Depend on abstractions (traits), inject dependencies for testability

**Example:**
```rust
pub trait Platform {
    fn capture_input(&mut self) -> Result<KeyEvent>;
    fn inject_output(&mut self, event: KeyEvent) -> Result<()>;
}

pub struct Daemon<P: Platform> {
    platform: P,  // Injected, mockable
}
```

### Core Patterns

**SSOT (Single Source of Truth):**
- `.krx` binary is THE config source
- `ExtendedState` is THE state representation
- No duplicate formats (JSON/TOML)

**KISS (Keep It Simple):**
- No features not explicitly needed
- Don't abstract until 3+ similar cases
- Don't optimize without profiling

**Dependency Injection:**
- All external deps injected (APIs, storage, platform code)
- Use traits for abstraction
- Mock in tests

## Naming Conventions

### Rust
| Element | Convention | Example |
|---------|-----------|---------|
| Modules/Files | `snake_case` | `mphf_gen.rs`, `lookup.rs` |
| Functions/Variables | `snake_case` | `load_config()`, `event_queue` |
| Structs/Enums/Traits | `PascalCase` | `ExtendedState`, `Platform` |
| Constants | `UPPER_SNAKE_CASE` | `MAX_MODIFIERS` |

### TypeScript/React
| Element | Convention | Example |
|---------|-----------|---------|
| Components/Files | `PascalCase` | `KeyboardVisualizer.tsx` |
| Functions/Variables | `camelCase` | `connectToDaemon()`, `eventQueue` |
| Hooks/Files | `use[Feature]` | `useSimulator.ts` |
| Constants | `UPPER_SNAKE_CASE` | `WS_PORT` |

### Import Order

**Rust:** std → external deps → workspace crates → crate modules

**TypeScript:** React → external deps → internal modules (@/) → relative imports → styles

## Common Tasks

### Add a Module
```bash
touch keyrx_core/src/validator.rs
# Add to lib.rs: pub mod validator;
# Implement with tests
make verify
```

### Run Tests
```bash
cargo test                           # All tests
cargo test -p keyrx_core lookup     # Specific crate/pattern
scripts/test.sh --integration       # Integration only
cargo test -- --nocapture           # Verbose output
```

### Add Dependency
```bash
cargo add rkyv --features validation
# Or edit Cargo.toml manually
make build
```

### Format Code
```bash
cargo fmt                # Format
cargo fmt --check        # Check only
```

### Fix Clippy Warnings
```bash
cargo clippy --workspace -- -D warnings
# Common fixes: remove needless borrow, unused mut, simplify match
```

## Troubleshooting

### Build Failures
- **"could not compile"** - Check error line, fix syntax
- **"cannot find value"** - Check imports/dependencies
- **"package collision"** - Run `cargo update`

### Test Failures
```bash
cargo test -- --nocapture           # Verbose output
cargo tarpaulin                     # Coverage report
```

### Pre-commit Hook Blocks
```bash
make verify                         # See detailed errors
# Fix issues, then commit again
git commit --no-verify              # Bypass (NOT recommended)
```

### CI/CD Failures
- **Clippy failed** - Run `cargo clippy` locally, fix warnings
- **Format failed** - Run `cargo fmt`, commit
- **Tests failed** - Check for platform-specific issues, ensure deterministic tests

### Windows VM Issues
```bash
vagrant status                      # Check VM state
vagrant up --debug                  # Detailed logs
vagrant provision                   # Re-install tools
vagrant destroy && vagrant up       # Fresh start
```

**Common errors:**
- "vagrant not found" → `sudo apt install vagrant`
- "No provider" → `vagrant plugin install vagrant-libvirt`
- "Permission denied" → `sudo usermod -aG libvirt $USER` (re-login)
- "Files not syncing" → `vagrant rsync`

See `vagrant/windows/README.md` for details.

### WASM Issues

**Quick health check:**
```bash
./scripts/wasm-health.sh            # Comprehensive diagnostics
```

**Common errors:**

| Error | Fix |
|-------|-----|
| "WASM not available" | `cd keyrx_ui && npm run build:wasm` |
| "wasm-pack not found" | `cargo install wasm-pack` |
| "wasm32-unknown-unknown not installed" | `rustup target add wasm32-unknown-unknown` |
| "Hash mismatch" | `cd keyrx_ui && npm run rebuild:wasm` |
| WASM file < 100KB | Check logs, run `npm run rebuild:wasm` |

**Build & verify:**
```bash
cd keyrx_ui
npm run build:wasm                  # Build
npm run rebuild:wasm                # Clean + build
npm run clean:wasm                  # Clean artifacts
../scripts/verify-wasm.sh           # Verify integrity
```

**Debug loading:**
1. Check browser console (F12)
2. Check WASM status badge in UI
3. Force rebuild: `rm -rf src/wasm/pkg && npm run build:wasm`

## Shared Utilities

### Frontend (TypeScript/React)

**Time Formatting** (`src/utils/timeFormatting.ts`):
- `formatTimestampMs(micros)` - "1.23s"
- `formatTimestampRelative(timestamp)` - "1 hour ago"

**Key Code Mapping** (`src/utils/keyCodeMapping.ts`):
- `keyCodeToLabel(code)` - "A", "Enter"
- `parseKeyCode(label)` - 65

**Test Utilities** (`tests/testUtils.tsx`):
- `renderWithProviders(ui, options)` - Wrap with providers
- `createMockStore(state)` - Mock Zustand store

### Backend (Rust)

**CLI Common** (`keyrx_daemon/src/cli/common.rs`):
- `output_success(data, json)` - Format success
- `output_error(message, code, json)` - Format errors

### Dependency Injection

**API Context** (`src/contexts/ApiContext.tsx`):
```typescript
const { apiBaseUrl, wsBaseUrl } = useApi();
```

**ConfigStorage** (`src/services/ConfigStorage.ts`):
- `LocalStorageImpl` - Production
- `MockStorageImpl` - Testing

## Technical Debt Prevention

### 1. File Size Monitoring
- Max 500 lines/file (code only)
- Run `scripts/verify_file_sizes.sh` before commit
- Extract modules when approaching limit

### 2. Extract Shared Utilities Early
- After second duplication, not third
- Create with ≥90% test coverage
- Update all usage sites

### 3. Dependency Injection
- Inject API endpoints, storage, WebSocket, platform code
- Enables testing with mocks
- Use context providers (frontend) or traits (backend)

### 4. Test Coverage
- Overall ≥80%, critical paths ≥90%
- New components must have tests
- Measure: `cargo tarpaulin`, `npm test -- --coverage`

### 5. Error Handling
- No silent catch blocks - always log
- Propagate errors to UI
- User must see failures

### 6. Structured Logging (JSON)
Required fields: timestamp, level, service, event, context
Never log: secrets, PII, full request/response bodies

### 7. Documentation
- Rust: Module comments (`//!`), function docs (`///`), examples
- TypeScript: JSDoc with @param, @returns, @example
- Run: `cargo doc`, `npm run typedoc`

## References

- **Script Docs**: `scripts/CLAUDE.md`
- **macOS Testing Guide**: `docs/development/MACOS_TESTING_GUIDE.md`
- **Windows VM Setup**: `docs/development/windows-vm-setup.md`
- **Steering Docs**: `.spec-workflow/specs/ai-dev-foundation/`
- **Project Structure**: `.spec-workflow/steering/structure.md`
- **CI/CD**: `.github/workflows/`
- **Production Readiness**: `.spec-workflow/specs/production-readiness-remediation/PRODUCTION_READINESS_REPORT.md`
- **Rust Guidelines**: https://rust-lang.github.io/api-guidelines/
