# Contributing to KeyRx

Guidelines for developing and contributing to KeyRx.

## Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- [Flutter](https://docs.flutter.dev/get-started/install) (for UI development)
- [just](https://github.com/casey/just#installation) (task runner)

## Quick Setup

```bash
just setup
```

This installs all toolchain components, development tools (cargo-nextest, cargo-watch), dependencies, and git hooks.

## Development Commands

Run `just` to see all available commands:

| Command | Description |
|---------|-------------|
| `just setup` | Install tools, dependencies, and git hooks |
| `just dev` | Run core with auto-reload (cargo watch) |
| `just ui` | Run Flutter UI in development mode |
| `just check` | Run all quality checks (fmt, clippy, test) |
| `just fmt` | Format Rust code |
| `just clippy` | Run clippy linter |
| `just test` | Run tests with nextest |
| `just bench` | Run benchmarks |
| `just clean` | Clean build artifacts |

## Building

### Current Platform

```bash
just build
```

Creates an optimized release binary at `core/target/release/keyrx`.

### Cross-Platform

```bash
just build-all      # Linux and Windows x86_64
just build-linux    # Linux x86_64
just build-windows  # Windows x86_64 (requires cross)
```

Windows builds require [cross](https://github.com/cross-rs/cross).

## Code Quality

### Pre-commit Hooks

Git hooks are automatically installed with `just setup`. The pre-commit hook runs:

1. `cargo fmt --check` - Code formatting
2. `cargo clippy -- -D warnings` - Linting
3. `cargo test --lib` - Unit tests

Commits are blocked if any check fails. Run `just fmt` to fix formatting issues.

### Quality Checks

Before submitting changes, run the full quality suite:

```bash
just check
```

## Testing

### Test Levels

| Level | Command | Purpose |
|-------|---------|---------|
| Unit | `cargo test --lib` | Small logic pieces |
| Integration | `cargo test --test '*'` | Full module interactions |
| UAT | `keyrx uat` | User acceptance tests |
| Regression | `keyrx regression` | Golden session verification |
| Performance | `keyrx bench` | Latency benchmarks |

### User Acceptance Tests (UAT)

Run the UAT test suite:

```bash
keyrx uat
```

Filter tests by category or priority:

```bash
keyrx uat --category core --priority P0
keyrx uat --category layers
```

Apply a quality gate:

```bash
keyrx uat --gate default  # 95% pass rate required
keyrx uat --gate alpha    # Relaxed (80% pass rate)
keyrx uat --gate ga       # Strictest (100% pass rate)
```

### Golden Sessions (Regression Testing)

Record a golden session:

```bash
keyrx golden record my_session --script path/to/script.rhai
```

Verify a golden session:

```bash
keyrx golden verify my_session
```

Run all regression tests:

```bash
keyrx regression
```

### CI Check

Run the complete CI test suite:

```bash
keyrx ci-check                  # Run all tests
keyrx ci-check --gate beta      # With quality gate enforcement
keyrx ci-check --skip-perf      # Skip performance tests
keyrx ci-check --json           # JSON output for CI parsing
```

### Quality Gates

Quality gates are defined in `.keyrx/quality-gates.toml`:

| Gate | Pass Rate | Max P0 | Max P1 | Max Latency | Min Coverage |
|------|-----------|--------|--------|-------------|--------------|
| alpha | 80% | 0 | 5 | 2000us | 60% |
| beta | 90% | 0 | 2 | 1000us | 75% |
| default | 95% | 0 | 2 | 1000us | 80% |
| rc | 98% | 0 | 0 | 500us | 85% |
| ga | 100% | 0 | 0 | 500us | 90% |

### Writing UAT Tests

Create Rhai test files in `tests/uat/`:

```javascript
// @category: core
// @priority: P0
// @requirement: REQ-001
// @latency: 1000
fn uat_basic_mapping() {
    let result = 1 + 1;
    if result != 2 {
        throw "Basic math failed";
    }
}
```

Metadata tags:
- `@category`: Test category for filtering (e.g., core, layers, performance)
- `@priority`: P0 (critical), P1 (high), P2 (normal)
- `@requirement`: Traceability to requirements
- `@latency`: Maximum allowed execution time in microseconds

## Creating a Release

```bash
just release 1.0.0
```

This updates version numbers, generates the changelog, and creates a git tag.

## Project Structure

```
keyrx/
├── core/           # Rust backend (engine, CLI, drivers)
├── ui/             # Flutter frontend (GUI)
├── docs/           # Architecture documentation
├── scripts/        # Example Rhai scripts
│   ├── std/        # Standard library
│   └── examples/   # Usage examples
└── tests/          # Integration and UAT tests
```

## Architecture

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for detailed architecture documentation including:
- Technology stack rationale
- Core design principles
- Data model
- Quality assurance approach
