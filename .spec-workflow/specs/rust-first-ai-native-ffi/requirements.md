# Rust-First AI-Native FFI Development - Requirements

## Overview

Transform the KeyRx FFI development workflow into an **AI-native, ultra-high-speed iteration system** following the principles from `docs/ultra-speed-rust-dart-win-linux-android.md`. This spec addresses the **62/100 alignment score** with the ultra-speed document by implementing missing AI-agent readiness features.

## Problem Statement

### Current State (Analysis Results)

**Strengths** ✅:
- Sophisticated contract-driven architecture with 10 domain contracts
- Modern Rust codegen tool (comprehensive pipeline)
- Flutter Rust Bridge v2 integration
- Excellent error handling and panic safety

**Critical Gaps** ❌ (Blocks AI-Native Development):
1. **NO `agent.md`** - AI agents lack project-specific directives
2. **NO automatic codegen workflow** - Manual `just gen-dart-bindings` required
3. **THREE-layer schema separation** - Rust API → JSON Contract → Dart (sync risk)
4. **Oversized modules** - 15,843-line files exceed LLM context windows
5. **NO implementation logging** - AI agents have no historical context
6. **Dual codegen tools** - Python (legacy) + Rust (modern) creates complexity
7. **NO watch/hot-regeneration** - Breaks developer flow

### Ultra-Speed Document Requirements

From `docs/ultra-speed-rust-dart-win-linux-android.md`:

> **契約駆動型アーキテクチャ：SSOTとFFIの進化**
> - Rustを唯一の真実とする (Rust as Single Source of Truth)
> - AIエージェント親和性の高いスキーマファースト開発
> - 契約の定義 → コード生成 → UIの実装 (Contract → Generate → UI)

> **AIコーディングエージェントの統合**
> - agent.md の設置 (Mandatory AI directive file)
> - コンテキストの分離 (Modules <500 lines for LLM context)
> - スキーマファースト (Schema-first prompting for AI)

> **超高速開発ワークフロー：CLIとAIの融合**
> - Just + Melos による完全自動化
> - just dev → Rust watch + Flutter hot reload
> - just codegen → 自動実行 (Auto-execute on change)

## Business Impact

### Current Development Speed Bottlenecks

**Scenario 1: Add new FFI function**
- Current process:
  1. Edit `core/src/ffi/exports_*.rs` (find right file among 15K lines)
  2. Update corresponding contract JSON manually
  3. Run `just gen-dart-bindings` manually
  4. Wait for compilation
  5. Check generated Dart, fix errors
  6. Repeat if missed sync
  - **Time**: 15-30 minutes per function

- Target process (AI-native):
  1. AI edits `api.rs` function signature (guided by `agent.md`)
  2. Contract auto-generated from `#[ffi_export]` annotation
  3. Dart bindings auto-regenerated (watch mode)
  4. Hot reload Flutter UI (<5s feedback)
  - **Time**: 2-5 minutes per function (5-10x faster)

**Scenario 2: Refactor large FFI module**
- Current: 15,843-line `exports_engine.rs` **exceeds** Claude's context window
- AI agent cannot understand full module scope
- Manual splitting required before AI can help
- **Productivity loss**: 80% (AI ineffective on oversized modules)

**Scenario 3: Debug FFI mismatch**
- Current: No contract validation in pre-commit
- Mismatch discovered at runtime (Dart crashes)
- Manual diff of JSON contract vs Rust code
- **Debug time**: 1-2 hours per issue

### Target Improvements

| Metric | Current | Target | Improvement |
|--------|---------|--------|-------------|
| Add FFI function | 15-30 min | 2-5 min | **5-10x faster** |
| Refactor FFI module | N/A (too large) | 30 min | **AI-enabled** |
| Contract sync errors | Runtime (crash) | Compile-time | **Zero runtime errors** |
| AI code generation accuracy | 30% (context overflow) | 85%+ | **3x more accurate** |
| Hot reload after FFI change | Never (full rebuild) | <10s | **Instant feedback** |

## Functional Requirements

### FR1: AI Agent Directive System

#### FR1.1: Create `agent.md` File
- **Location**: `.keyrx/agent.md` (project root)
- **Content**: AI-specific development rules, constraints, patterns
- **Format**: Markdown with clear sections:
  - Architecture Constraints (SSOT, no mutex in audio thread, etc.)
  - Code Style (FRB v2, anyhow errors, etc.)
  - Workflow (schema-first, contract → generate → UI)
  - Examples (good vs bad patterns)
  - Module boundaries
  - Testing requirements

#### FR1.2: Context Window Optimization
- **Max file size**: 500 lines (excluding comments, per ultra-speed doc)
- **Module naming**: Descriptive, single-responsibility
- **Import organization**: Clear dependency graph

#### FR1.3: Implementation Logging
- **Location**: `.spec-workflow/specs/ffi-implementation-log.md`
- **Format**: Timestamped entries with:
  - Task ID
  - FFI functions added/modified
  - Contracts changed
  - Files modified
  - Test coverage
  - AI agent notes (what worked, what failed)

### FR2: Automatic Codegen Workflow

#### FR2.1: Watch Mode for FFI Changes
- **Trigger**: Any Rust file change in `core/src/ffi/` or `core/src/api.rs`
- **Action**: Auto-run code generation
- **Feedback**: Terminal notification + file change count
- **Debounce**: 500ms to avoid excessive regeneration

#### FR2.2: Integrated Hot Reload
- **Workflow**:
  1. Developer edits `api.rs`
  2. cargo-watch detects change
  3. Recompile Rust (with fast linker: mold/zld)
  4. Regenerate Dart bindings
  5. Flutter hot reload (if only Dart changed)
  6. OR full restart (if Rust ABI changed)
- **Target**: <10s from edit to UI update

#### FR2.3: Smart Incremental Generation
- **Track**: SHA256 of contract JSON + Rust source
- **Skip**: Regeneration if no actual changes (avoid churn)
- **Notify**: "Skipped (no changes)" vs "Generated (N files updated)"

### FR3: Contract-First Enforcement

#### FR3.1: Derive Contracts from Rust Code
- **Current**: Manual JSON editing
- **Target**: `#[ffi_export]` macro generates contract
- **Benefits**: Single source of truth, no sync issues

**Example**:
```rust
#[ffi_export(domain = "engine", version = "1.0")]
pub fn start_engine(script_path: String) -> Result<EngineHandle> {
    // Implementation
}

// Auto-generates contract JSON:
// {
//   "function": "start_engine",
//   "domain": "engine",
//   "params": [{"name": "script_path", "type": "String"}],
//   "return": {"type": "Result<EngineHandle>"}
// }
```

#### FR3.2: Contract Validation in Pre-Commit
- **Hook**: `.githooks/pre-commit` includes contract validation
- **Check**: All `#[ffi_export]` functions have matching generated contracts
- **Fail**: If manual JSON edits don't match Rust signatures

#### FR3.3: Breaking Change Detection
- **Tool**: `just diff-ffi-contracts` compares contracts against previous version
- **Output**: List of breaking changes (removed functions, changed signatures)
- **CI**: Block PR if breaking changes without version bump

### FR4: Module Size Refactoring

#### FR4.1: Split Oversized Modules
- **Target**: All FFI modules ≤500 lines
- **Current violations**:
  - `exports_engine.rs`: 15,843 lines → Split into 32 modules
  - `exports_compat.rs`: 15,754 lines → Split into 31 modules

**Proposed structure**:
```
core/src/ffi/domains/
├── engine/
│   ├── lifecycle.rs         (start, stop, pause)
│   ├── state_queries.rs     (get_state, get_metrics)
│   ├── script_management.rs (load, validate, reload)
│   └── mod.rs               (re-exports)
├── compat/
│   ├── legacy_v1.rs
│   ├── legacy_v2.rs
│   └── mod.rs
└── mod.rs
```

#### FR4.2: Naming Conventions
- **Pattern**: `{domain}/{feature_group}.rs`
- **Max exports per file**: 10 functions
- **Documentation**: Each module has comprehensive module-level docs

### FR5: Codegen Consolidation

#### FR5.1: Remove Python Codegen
- **Action**: Delete `scripts/generate_dart_bindings.py`
- **Reason**: Duplication, complexity, maintenance burden
- **Migration**: All users use Rust tool only

#### FR5.2: Unified Codegen CLI
- **Tool**: `core/tools/generate_dart_bindings`
- **Flags**:
  - `--watch`: Watch mode (auto-regenerate)
  - `--check`: Verification only (CI)
  - `--domain <name>`: Generate specific domain only
  - `--verbose`: Detailed logging

### FR6: Observability Integration

#### FR6.1: FFI Call Tracing
- **Instrumentation**: `#[instrument]` on all FFI functions
- **Output**: Dart DevTools timeline integration
- **Metrics**: Call count, latency, errors per function

#### FR6.2: Contract Metrics Dashboard
- **Metrics**:
  - Total FFI functions
  - Functions by domain
  - Average function complexity
  - Breaking changes over time
- **Format**: JSON report + HTML dashboard

#### FR6.3: Implementation Log Analysis
- **Tool**: `just analyze-ffi-history`
- **Output**:
  - Velocity (FFI functions added per week)
  - Churn (functions modified frequently)
  - Coverage (tested vs untested functions)

## Non-Functional Requirements

### NFR1: Performance Targets

| Operation | Target | Current | Improvement Needed |
|-----------|--------|---------|-------------------|
| Code generation (full) | <5s | ~8s | 1.6x faster |
| Code generation (incremental) | <1s | N/A (not implemented) | New feature |
| Hot reload after Rust change | <10s | N/A (full rebuild only) | New capability |
| Contract validation | <500ms | N/A (manual) | Automated |

### NFR2: AI Agent Compatibility

#### NFR2.1: LLM Context Window Limits
- **Constraint**: Claude 3.5 Sonnet has ~200K token context
- **Target**: Any single FFI module fits within 10K tokens (~500 lines)
- **Verification**: Automated check in pre-commit

#### NFR2.2: Prompt Engineering Support
- **Requirement**: `agent.md` must provide:
  - Clear architectural constraints
  - Step-by-step workflow guidance
  - Examples of correct patterns
  - Anti-patterns to avoid

### NFR3: Developer Experience

#### NFR3.1: Zero Manual Sync
- **Requirement**: Developers never manually edit contract JSON
- **Enforcement**: Contract files marked as "generated, do not edit"
- **Tooling**: Contracts regenerated from Rust source only

#### NFR3.2: Fast Feedback Loops
- **Edit → Feedback time**: <10s for most changes
- **Error clarity**: Compilation errors point to exact Rust line
- **Hot reload**: Works for Dart-only changes

## Success Criteria

### Phase 1: AI Readiness (Week 1)
- ✅ `agent.md` created with comprehensive directives
- ✅ Implementation logging system established
- ✅ Module size analysis complete (identify all >500 line files)
- ✅ Watch mode prototype working (auto-regenerate on change)

### Phase 2: Workflow Automation (Week 2)
- ✅ `just dev` starts Rust watch + Flutter hot reload
- ✅ `just codegen --watch` implemented
- ✅ Fast linker (mold/zld) integrated
- ✅ Incremental generation (skip unchanged contracts)

### Phase 3: Contract Automation (Week 3)
- ✅ `#[ffi_export]` macro generates contracts
- ✅ Contract validation in pre-commit
- ✅ Breaking change detection tool
- ✅ Python codegen removed

### Phase 4: Module Refactoring (Week 3-4)
- ✅ `exports_engine.rs` split into ≤500 line modules
- ✅ `exports_compat.rs` split into ≤500 line modules
- ✅ All FFI modules meet size constraints
- ✅ Module naming conventions applied

### Phase 5: Observability (Week 4)
- ✅ FFI tracing integrated with Dart DevTools
- ✅ Contract metrics dashboard
- ✅ Implementation log analysis tools
- ✅ Performance regression tests

## Out of Scope

### Won't Do in v1
- ❌ Full contract versioning system (use Git history)
- ❌ GUI for contract editing (CLI only)
- ❌ Automated migration of Dart code (manual)
- ❌ Contract format beyond JSON (keep current)
- ❌ Multi-language FFI (Dart only, no Swift/Kotlin)

### Future Considerations
- Contract schema evolution (v2, v3 support)
- Visual FFI dependency graph
- Automated performance regression detection
- Contract-based mock generation
- FFI fuzzing from contracts

## Dependencies

### Required Tools
- ✅ `flutter_rust_bridge: ^2.0` (already installed)
- ✅ Rust stable (already configured)
- 🆕 `cargo-watch` (for watch mode)
- 🆕 `mold` (Linux) or `zld` (macOS) - fast linkers
- 🆕 `tokei` (for code metrics)

### Documentation Updates
- Update: `docs/ffi-architecture.md` - Add AI-native workflow section
- Create: `.keyrx/agent.md` - AI agent directives
- Create: `docs/FFI_DEVELOPMENT.md` - Developer guide for new workflow

### Infrastructure Changes
- Update: `.githooks/pre-commit` - Add contract validation
- Update: `justfile` - Add watch mode recipes
- Create: `.spec-workflow/specs/ffi-implementation-log.md`
- Update: `core/tools/generate_dart_bindings` - Add derive macro

## Acceptance Criteria

This spec is complete when:

1. **AI Agent Effectiveness**:
   - ✅ AI agent can add new FFI function end-to-end without human intervention
   - ✅ Agent uses `agent.md` directives correctly
   - ✅ All FFI modules fit in LLM context window (≤500 lines)

2. **Development Velocity**:
   - ✅ Add FFI function: ≤5 minutes (down from 15-30 min)
   - ✅ Hot reload after Rust change: <10s
   - ✅ Zero manual contract editing

3. **Code Quality**:
   - ✅ Zero contract sync errors in CI
   - ✅ Pre-commit validates all contracts
   - ✅ Breaking changes detected automatically

4. **Observability**:
   - ✅ FFI calls visible in Dart DevTools timeline
   - ✅ Implementation log tracks all FFI changes
   - ✅ Metrics dashboard shows FFI health

5. **Documentation**:
   - ✅ `agent.md` comprehensive and tested with actual AI
   - ✅ Updated architecture docs
   - ✅ Migration guide for developers

## References

- `docs/ultra-speed-rust-dart-win-linux-android.md` - Source requirements
- `docs/ffi-architecture.md` - Current FFI documentation
- `docs/ffi-contract-schema.md` - Contract specification
- `.spec-workflow/steering/tech.md` - Technical standards
- Exploration report: Detailed gap analysis (62/100 score)
