# Development Velocity Roadmap

## Overview

This document coordinates multiple specs focused on achieving **ultra-high-speed development iteration** for KeyRx. Each spec addresses a different layer of the development stack.

## Active Velocity Specs

### 1. Ultra-Speed Dev Experience (Flutter/Dart)
**Location**: `.spec-workflow/specs/ultra-speed-dev-experience/`

**Status**: 🟢 In Progress (Phase 1: 75% complete)

**Focus**: Flutter/Dart development tooling
- Pre-commit hooks for Dart
- Enhanced linting and analysis
- Mock FFI bridge for UI development
- Riverpod state management
- Widget/golden test infrastructure
- Structured logging

**Target Improvements**:
- UI iteration: 3-5s → <1s (hot reload)
- Add UI feature: 4-6h → <2h
- Test coverage: Unknown → 80%+

**Progress**:
- ✅ Phase 1.1: Enhanced Analysis Options (6/6 tasks)
- ✅ Phase 1.2: Pre-Commit Hooks for Dart (8/8 tasks)
- ✅ Phase 1.3: Coverage Tracking (6/6 tasks)
- ⏸️ Phase 1.4: Mock FFI Bridge (1/7 tasks)
- ⏳ Phase 2-7: Pending

**Next Action**: Complete Phase 1.4 (Mock FFI Bridge)

---

### 2. Rust-First AI-Native FFI
**Location**: `.spec-workflow/specs/rust-first-ai-native-ffi/`

**Status**: 🔵 Ready to Start

**Focus**: Rust FFI workflow optimization for AI agents
- AI agent directive system (agent.md)
- Automatic codegen workflow (watch mode)
- Contract derivation from Rust (#[ffi_export])
- Module size refactoring (15K→500 line files)
- Fast linker integration (mold/zld)
- Implementation logging for AI context

**Target Improvements**:
- Add FFI function: 15-30 min → 2-5 min (5-10x faster)
- Hot reload after Rust change: Never → <10s
- AI code accuracy: 30% → 85%+
- Module comprehension: 2+ hours → 15 min

**Progress**:
- ⏳ All phases pending (not started)

**Next Action**: Start Phase 1 (AI Readiness Foundation)

---

### 3. KeyFlow Implementation
**Location**: `.spec-workflow/specs/keyflow-implementation/`

**Status**: 🟡 Blocked (awaits foundations)

**Focus**: Visual configuration flow editor
- Interactive node-based UI
- Real-time device updates
- FFI integration for runtime config
- Drag-drop slot management

**Dependencies**:
- **Requires**: Ultra-Speed Dev Experience (Phase 1-2) for Flutter tooling
- **Requires**: Rust-First AI-Native FFI (Phase 1-3) for efficient FFI development
- **Benefits**: Fast iteration on both UI and FFI layers

**Progress**:
- ⏳ Awaiting foundations

**Next Action**: Wait for dependencies

---

## Coordination Strategy

### Execution Order

**Recommended Approach**: Parallel development on separate tracks

```
Track 1 (Flutter/Dart):          Track 2 (Rust/FFI):
┌──────────────────────┐         ┌──────────────────────┐
│ Ultra-Speed Dev Exp  │         │ Rust-First AI-Native │
│ Phase 1: Foundation  │  ←─┐    │ Phase 1: AI Readiness│
│ Phase 2: Testing     │    │    │ Phase 2: Workflow    │
│ Phase 3: FFI Optim   │    └─→  │ Phase 3: Contracts   │
│ Phase 4: Observ      │         │ Phase 4: Refactoring │
└──────────┬───────────┘         └──────────┬───────────┘
           └─────────────┬─────────────────┘
                         ▼
                  ┌──────────────┐
                  │   KeyFlow    │
                  │ Implementation│
                  └──────────────┘
```

**Alternative Approach**: Sequential (if solo developer)

```
Phase 1: Complete Ultra-Speed Dev Experience (Phases 1-4)
    ↓
Phase 2: Complete Rust-First AI-Native FFI (Phases 1-4)
    ↓
Phase 3: Implement KeyFlow (benefits from both foundations)
```

### Cross-References

#### Ultra-Speed Dev Experience → Rust-First AI-Native FFI
- Phase 1.4 (Mock FFI Bridge) enables Phase 2.1 (Watch Mode testing)
- Phase 2.4 (Structured Logging) complements Phase 5.1 (Tracing)
- Phase 3.1 (FFI Watch Mode) **duplicated** → Use Rust-First version

#### Rust-First AI-Native FFI → Ultra-Speed Dev Experience
- Phase 1.1 (agent.md) guides Phase 5.1 (design.md creation)
- Phase 2.2 (Integrated Dev Workflow) merges with Phase 3 (FFI Optimization)
- Phase 5.1 (Tracing) integrates with Phase 4 (Observability)

### Duplication Resolution

**Identified Overlaps**:
1. **FFI Watch Mode**:
   - Ultra-Speed Phase 3.1: FFI watch mode automation
   - Rust-First Phase 2.1: Watch mode implementation
   - **Resolution**: Use Rust-First version (more comprehensive)

2. **Observability/Logging**:
   - Ultra-Speed Phase 2.4: Structured logging (Dart)
   - Rust-First Phase 5.1: Tracing infrastructure (Rust)
   - **Resolution**: Both needed (different layers), coordinate format

3. **Pre-commit Hooks**:
   - Ultra-Speed Phase 1.2: Dart pre-commit
   - Rust-First Phase 3.3: Contract validation in pre-commit
   - **Resolution**: Single `.githooks/pre-commit` with both checks

**Action**: Remove duplicate tasks, add cross-references

---

## Combined Success Metrics

| Aspect | Baseline | After Ultra-Speed | After Rust-First | Combined |
|--------|----------|-------------------|------------------|----------|
| **UI Iteration** | 3-5s | <1s | N/A | <1s |
| **FFI Iteration** | 60s+ rebuild | N/A | <10s | <10s |
| **Add UI Feature** | 4-6h | <2h | N/A | <2h |
| **Add FFI Function** | 15-30 min | N/A | 2-5 min | 2-5 min |
| **Test Coverage (Dart)** | Unknown | 80%+ | N/A | 80%+ |
| **Test Coverage (Rust)** | ~80% | N/A | 90%+ | 90%+ |
| **AI Effectiveness** | 30% | 60% (Flutter) | 85% (Rust) | **85%+ overall** |

**Combined Impact**: **10-15x faster development velocity**

---

## Task Count Summary

| Spec | Total Tasks | Phases | Est. Effort |
|------|-------------|--------|-------------|
| Ultra-Speed Dev Experience | ~280 tasks | 7 phases | 6-8 weeks |
| Rust-First AI-Native FFI | ~140 tasks | 7 phases | 4-5 weeks |
| KeyFlow Implementation | ~150 tasks | 9 phases | 4-5 weeks |
| **Combined** | **~570 tasks** | - | **14-18 weeks** |

**With Parallel Development**: 8-10 weeks (2 tracks)

---

## Current Status Dashboard

### Ultra-Speed Dev Experience
```
Phase 1: Foundation              ████████████████░░░░ 75%
  ├─ 1.1 Enhanced Analysis       ████████████████████ 100%
  ├─ 1.2 Pre-Commit Hooks        ████████████████████ 100%
  ├─ 1.3 Coverage Tracking       ████████████████████ 100%
  ├─ 1.4 Mock FFI Bridge         ███░░░░░░░░░░░░░░░░░  14%
  └─ 1.5 Justfile Recipes        ░░░░░░░░░░░░░░░░░░░░   0%

Phase 2: Testing                 ░░░░░░░░░░░░░░░░░░░░   0%
Phase 3: FFI Optimization        ░░░░░░░░░░░░░░░░░░░░   0%
Phase 4: Observability           ░░░░░░░░░░░░░░░░░░░░   0%
Phase 5: Documentation           ░░░░░░░░░░░░░░░░░░░░   0%
Phase 6: CI/CD                   ░░░░░░░░░░░░░░░░░░░░   0%
Phase 7: Developer Experience    ░░░░░░░░░░░░░░░░░░░░   0%

Overall Progress: ██░░░░░░░░░░░░░░░░░░ 11% (30/280 tasks)
```

### Rust-First AI-Native FFI
```
Phase 1: AI Readiness            ░░░░░░░░░░░░░░░░░░░░   0%
Phase 2: Workflow Automation     ░░░░░░░░░░░░░░░░░░░░   0%
Phase 3: Contract Automation     ░░░░░░░░░░░░░░░░░░░░   0%
Phase 4: Module Refactoring      ░░░░░░░░░░░░░░░░░░░░   0%
Phase 5: Observability           ░░░░░░░░░░░░░░░░░░░░   0%
Phase 6: Documentation           ░░░░░░░░░░░░░░░░░░░░   0%
Phase 7: Performance             ░░░░░░░░░░░░░░░░░░░░   0%

Overall Progress: ░░░░░░░░░░░░░░░░░░░░   0% (0/140 tasks)
```

---

## Recommendations

### For Solo Developer
1. **Complete Ultra-Speed Phase 1** (1-2 days remaining)
2. **Start Rust-First Phase 1** (2-3 days)
3. **Alternate phases** to avoid context fatigue
4. **Checkpoint every 10 tasks** (review progress)

### For Small Team (2-3 devs)
1. **Dev 1**: Ultra-Speed Dev Experience (Flutter focus)
2. **Dev 2**: Rust-First AI-Native FFI (Rust focus)
3. **Dev 3**: KeyFlow Implementation (awaits foundations)
4. **Weekly sync**: Coordinate overlaps and blockers

### For AI-Assisted Development
1. **Use separate AI agents** for each spec (avoid context mixing)
2. **Agent 1**: "You are working on ultra-speed-dev-experience, focus on Flutter/Dart"
3. **Agent 2**: "You are working on rust-first-ai-native-ffi, focus on Rust/FFI"
4. **Provide agent.md** to each for context

---

## Next Immediate Actions

### This Week
- [ ] Complete ultra-speed-dev-experience Phase 1.4 (Mock FFI Bridge)
- [ ] Start rust-first-ai-native-ffi Phase 1.1 (Create agent.md)
- [ ] Remove duplicate tasks between specs
- [ ] Add cross-references in requirements.md

### Next Week
- [ ] Complete ultra-speed-dev-experience Phase 1 (Foundation)
- [ ] Complete rust-first-ai-native-ffi Phase 1 (AI Readiness)
- [ ] Begin Phase 2 on both tracks

### Month 1 Goal
- [ ] Ultra-Speed: Phases 1-3 complete (Foundation + Testing + FFI Optimization)
- [ ] Rust-First: Phases 1-2 complete (AI Readiness + Workflow Automation)
- [ ] Ready to start KeyFlow implementation

---

## Conclusion

**Recommendation**: **Keep specs separate, use this roadmap for coordination.**

**Rationale**:
1. ✅ Each spec is manageable for AI agents (~150-200 tasks)
2. ✅ Different technical domains can be developed in parallel
3. ✅ Follows established KeyRx pattern (many small specs)
4. ✅ Avoids AI context overflow (your concern)
5. ✅ Clear ownership and dependencies

**Trade-off**: Requires coordination document (this file) to track relationships.

**Alternative**: If specs truly must be merged, split by phase instead (7 small specs instead of 2 medium ones).
