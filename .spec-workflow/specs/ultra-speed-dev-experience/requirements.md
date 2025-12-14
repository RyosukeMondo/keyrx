# Ultra-Speed Dev Experience - Requirements

## Overview

This spec defines improvements to achieve ultra-high-speed development iteration for KeyRx, with focus on Flutter/Dart development velocity while maintaining the existing Rust quality standards.

## Current State Analysis

### Strengths
- ✅ Excellent Rust tooling (cargo, clippy, pre-commit hooks)
- ✅ Comprehensive CI/CD pipeline
- ✅ FFI contract verification
- ✅ Strong quality gates for Rust (80% coverage, <1ms latency)
- ✅ Well-documented architecture (tech.md, structure.md, product.md)

### Critical Bottlenecks

#### 1. FFI Development Cycle (HIGHEST IMPACT)
**Problem**: Rust changes require full app restart
- ❌ No hot reload for FFI changes
- ❌ Manual FFI binding regeneration
- ❌ 30-60s rebuild cycle for Rust changes
- ❌ No debug workflow for FFI boundary issues

**Impact**: 80% of development time involves FFI layer

#### 2. Testing Velocity
**Problem**: Slow feedback loops for Flutter code
- ❌ Dart tests NOT in pre-commit (only Rust)
- ❌ No coverage tracking for Flutter (only Rust 80% target)
- ❌ Missing widget test scaffolding/patterns
- ❌ No golden test infrastructure
- ❌ Manual test running (no watch mode workflow)

**Impact**: Tests discovered late, harder to fix

#### 3. State Management Inconsistency
**Problem**: No standardized pattern slows feature development
- ❌ Mix of ChangeNotifier, Provider, RxDart streams
- ❌ No documented pattern for new features
- ❌ Manual provider wiring is error-prone
- ❌ No code generation for boilerplate

**Impact**: Every new feature requires architectural decisions

#### 4. Missing Design System
**Problem**: Slower UI decisions, inconsistent styling
- ❌ No design.md (unlike other steering docs)
- ❌ No component library documentation
- ❌ No color palette, typography, spacing system
- ❌ No Flutter widget patterns documented

**Impact**: UI work requires more decision-making overhead

#### 5. Code Quality Automation Gaps
**Problem**: Manual code quality checks
- ❌ `analysis_options.yaml` is minimal (just flutter_lints)
- ❌ No import ordering/organization rules
- ❌ No automated code generation workflow
- ❌ No pre-commit for Dart formatting/linting

**Impact**: Inconsistent code style, slower PR reviews

#### 6. Debugging & Observability
**Problem**: Hard to debug issues crossing FFI boundary
- ❌ No Flutter DevTools integration guide
- ❌ No structured logging on Dart side (only Rust)
- ❌ No performance profiling workflow
- ❌ No FFI call tracing

**Impact**: Debugging takes 2-3x longer

## Requirements

### Functional Requirements

#### FR1: Fast FFI Development Cycle
- **FR1.1**: Hot reload for Rust changes (or <5s rebuild)
- **FR1.2**: Automatic FFI binding regeneration on Rust changes
- **FR1.3**: FFI call tracing/debugging tools
- **FR1.4**: Mock FFI bridge for UI development without Rust

#### FR2: Comprehensive Testing Infrastructure
- **FR2.1**: Dart tests in pre-commit hooks (like Rust)
- **FR2.2**: Coverage tracking with 80% minimum (matching Rust)
- **FR2.3**: Widget test scaffolding and patterns
- **FR2.4**: Golden test infrastructure
- **FR2.5**: Watch mode for continuous testing during dev

#### FR3: Standardized State Management
- **FR3.1**: Document official state management pattern
- **FR3.2**: Code generation for state boilerplate
- **FR3.3**: Provider setup automation
- **FR3.4**: Stream management patterns

#### FR4: Complete Design System
- **FR4.1**: Design system documentation (design.md)
- **FR4.2**: Component library with examples
- **FR4.3**: Color palette, typography, spacing tokens
- **FR4.4**: Flutter widget patterns and conventions
- **FR4.5**: Dark mode support guidelines

#### FR5: Enhanced Code Quality
- **FR5.1**: Comprehensive `analysis_options.yaml` with strict rules
- **FR5.2**: Import ordering and organization rules
- **FR5.3**: Automated code generation in build workflow
- **FR5.4**: Pre-commit hooks for Dart (fmt, analyze, test)
- **FR5.5**: File size metrics (500 lines max like Rust)
- **FR5.6**: Function size metrics (50 lines max like Rust)

#### FR6: Advanced Debugging & Observability
- **FR6.1**: Flutter DevTools integration guide
- **FR6.2**: Structured logging on Dart side (matching Rust JSON logs)
- **FR6.3**: Performance profiling workflow
- **FR6.4**: FFI boundary tracing and visualization
- **FR6.5**: Error tracking across FFI boundary

### Non-Functional Requirements

#### NFR1: Performance Targets
- **NFR1.1**: Hot reload <1s (Flutter standard)
- **NFR1.2**: Rust rebuild <5s (with caching)
- **NFR1.3**: Test suite <30s (for pre-commit)
- **NFR1.4**: Full build <2min (CI pipeline)
- **NFR1.5**: App startup <500ms to interactive

#### NFR2: Developer Experience
- **NFR2.1**: Single command setup for new developers
- **NFR2.2**: Clear error messages with actionable fixes
- **NFR2.3**: Comprehensive documentation with examples
- **NFR2.4**: IDE integration (VSCode, Android Studio)

#### NFR3: Quality Metrics (Match Rust Standards)
- **NFR3.1**: 80% test coverage minimum (90% critical paths)
- **NFR3.2**: Zero lint warnings in CI
- **NFR3.3**: Zero type errors
- **NFR3.4**: Max 500 lines/file (excluding comments)
- **NFR3.5**: Max 50 lines/function

#### NFR4: Maintainability
- **NFR4.1**: All tooling configuration in version control
- **NFR4.2**: Self-documenting code with clear patterns
- **NFR4.3**: Automated dependency updates
- **NFR4.4**: Migration guides for breaking changes

## Success Criteria

### Phase 1: Foundation (Week 1)
- ✅ Pre-commit hooks for Dart (fmt, analyze, test)
- ✅ Enhanced `analysis_options.yaml` with strict rules
- ✅ Coverage tracking configured (80% target)
- ✅ Design system documented (design.md)
- ✅ State management pattern documented

### Phase 2: Testing Infrastructure (Week 2)
- ✅ Widget test scaffolding and examples
- ✅ Golden test infrastructure
- ✅ Watch mode workflow documented
- ✅ Mock FFI bridge for UI development
- ✅ Coverage reports in CI

### Phase 3: FFI Optimization (Week 3)
- ✅ FFI binding auto-regeneration on save
- ✅ FFI call tracing tools
- ✅ Rust rebuild caching (<5s rebuilds)
- ✅ FFI debugging guide

### Phase 4: Observability (Week 4)
- ✅ Structured logging on Dart side
- ✅ Flutter DevTools integration
- ✅ Performance profiling workflow
- ✅ Error tracking dashboard

## Out of Scope

### Won't Do in v1
- ❌ Full Rust hot reload (technically complex, marginal value)
- ❌ Custom state management library (use existing: Riverpod/Provider)
- ❌ Custom component framework (use Flutter widgets)
- ❌ E2E test framework (focus on unit + widget tests first)
- ❌ Visual regression testing service (golden tests sufficient)
- ❌ Custom build system (leverage existing cargo + flutter tools)

### Future Considerations
- Code generation for FFI bindings (beyond flutter_rust_bridge)
- Advanced profiling (flame graphs, memory snapshots)
- Distributed tracing across Rust + Dart
- Load testing infrastructure

## Dependencies

### Required Tools/Libraries
- **Existing**: Flutter 3.24.0, Dart 3.9.2, flutter_rust_bridge v2
- **New**:
  - `riverpod` or `provider` (state management)
  - `mockito` or `mocktail` (mocking)
  - `golden_toolkit` (golden tests)
  - `logger` or `flogger` (structured logging)
  - `very_good_analysis` or custom lint rules

### Documentation Updates
- Create: `.spec-workflow/steering/design.md`
- Update: `.spec-workflow/steering/tech.md` (add Flutter sections)
- Update: `.spec-workflow/steering/structure.md` (match actual Dart layout)

### Infrastructure Changes
- Add: `.githooks/pre-commit` (Dart checks)
- Update: `.github/workflows/ci.yml` (add Flutter coverage)
- Add: `analysis_options.yaml` (comprehensive rules)
- Add: `justfile` recipes for Dart workflows

## Acceptance Criteria

This spec is complete when:

1. **Development Velocity**:
   - ✅ Typical feature implementation is 2-3x faster
   - ✅ FFI changes have <10s feedback loop (down from 60s)
   - ✅ UI-only changes have <1s feedback loop (hot reload)

2. **Quality Maintained**:
   - ✅ Dart code meets same standards as Rust (80% coverage, lint-clean)
   - ✅ Pre-commit prevents broken code from entering repo
   - ✅ CI enforces all quality gates

3. **Documentation Complete**:
   - ✅ design.md covers all UI patterns
   - ✅ tech.md has comprehensive Flutter/Dart section
   - ✅ structure.md matches actual implementation
   - ✅ Debugging guide for FFI boundary

4. **Developer Onboarding**:
   - ✅ New developer can run `just setup` and start contributing
   - ✅ Common workflows documented with examples
   - ✅ Error messages are actionable

5. **Measurable Improvement**:
   - ✅ Time to implement new feature: <2 hours (down from 4-6)
   - ✅ Time to debug FFI issue: <30 min (down from 2 hours)
   - ✅ Time to add new UI screen: <1 hour (down from 3)

## References

- AUTONOMOUS-DEV-GUIDE.md - Autonomous development patterns
- .spec-workflow/steering/tech.md - Current technical standards
- .spec-workflow/steering/product.md - Product vision
- CLAUDE.md - Code quality KPIs (500 lines/file, 50 lines/fn, 80% coverage)
