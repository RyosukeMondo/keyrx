# Documentation Coverage Audit

**Date:** 2025-12-12
**Spec:** misc-improvements
**Task:** 1.4

## Executive Summary

The codebase has significant documentation gaps with **1,069 missing documentation items** across public APIs. The majority (669) are struct fields, but there are also 226 enum variants, 50 methods, 37 associated functions, 29 functions, 21 modules, 14 structs, 10 enums, and 2 traits without documentation.

Additionally, there are **28 documentation warnings** in `keyrx_core` related to broken links and HTML tag issues.

## Missing Documentation by Type

| Type | Count | Priority |
|------|-------|----------|
| Struct fields | 669 | Low (context usually clear) |
| Enum variants | 226 | Medium (error variants need docs) |
| Methods | 50 | High (API surface) |
| Associated functions | 37 | High (constructors, factories) |
| Functions | 29 | High (public API) |
| Modules | 21 | Critical (entry points) |
| Structs | 14 | Critical (core types) |
| Enums | 10 | Critical (core types) |
| Constants | 5 | Low |
| Associated constants | 4 | Low |
| Type aliases | 2 | Medium |
| Traits | 2 | Critical (interfaces) |
| **Total** | **1,069** | |

## Critical Path Analysis

### High Priority - Services (24 missing items)

| File | Count | Items |
|------|-------|-------|
| `core/src/services/device.rs` | 13 | `DeviceServiceError` enum, variants, `DeviceInfo` fields |
| `core/src/services/runtime.rs` | 7 | `RuntimeServiceError`, `RuntimeService`, `new()`, `with_defaults()` |
| `core/src/services/profile.rs` | 4 | `ProfileServiceError`, `ProfileService` |

**Priority: CRITICAL** - These are core service APIs that external consumers use.

### High Priority - FFI (156 missing items)

| File | Count | Items |
|------|-------|-------|
| `core/src/ffi/contract.rs` | 71 | FFI contracts, fields |
| `core/src/ffi/introspection.rs` | 33 | Introspection types |
| `core/src/ffi/events.rs` | 18 | Event types |
| `core/src/ffi/exports.rs` | 10 | Exported functions |
| Various domain modules | 24 | Domain-specific FFI |

**Priority: CRITICAL** - FFI is the external interface for bindings.

### High Priority - Engine (162 missing items)

| File | Count | Items |
|------|-------|-------|
| `core/src/engine/state/modifiers.rs` | 22 | Modifier state types |
| `core/src/engine/decision/pending.rs` | 22 | Pending decision queue |
| `core/src/engine/state/mutation.rs` | 19 | State mutation types |
| `core/src/engine/state/error.rs` | 18 | Error types |
| `core/src/engine/recording/format.rs` | 16 | Recording format |
| `core/src/engine/transitions/transition.rs` | 14 | Transition types |
| `core/src/engine/limits/enforcer.rs` | 14 | Limit enforcement |
| `core/src/engine/state/change.rs` | 13 | Change tracking |
| `core/src/engine/state/layers.rs` | 11 | Layer state |
| Other engine modules | 33 | Various types |

**Priority: HIGH** - Engine is core logic but mostly internal.

### Medium Priority - Config (40 missing items)

| File | Count | Items |
|------|-------|-------|
| `core/src/config/models.rs` | 40 | Configuration model fields |

**Priority: MEDIUM** - Config types with many fields.

### Medium Priority - CLI Commands (112 missing items)

| File | Count | Items |
|------|-------|-------|
| `core/src/cli/commands/hardware.rs` | 23 | Hardware command |
| `core/src/cli/commands/runtime.rs` | 22 | Runtime command |
| `core/src/cli/commands/devices.rs` | 17 | Devices command |
| `core/src/cli/commands/doctor.rs` | 11 | Doctor command |
| `core/src/cli/commands/keymap.rs` | 11 | Keymap command |
| `core/src/cli/commands/bench.rs` | 8 | Bench command |
| Various other commands | 20 | Other commands |

**Priority: MEDIUM** - CLI commands are user-facing but mostly internal.

### Other Notable Files

| File | Count | Priority |
|------|-------|----------|
| `core/src/scripting/builtins.rs` | 75 | Medium (scripting API) |
| `core/src/discovery/session.rs` | 51 | Medium (discovery types) |
| `core/src/errors/critical.rs` | 31 | High (error handling) |
| `core/src/scripting/sandbox/budget.rs` | 24 | Low (internal) |
| `core/src/hardware/cloud_sync.rs` | 19 | Low (internal) |
| `core/src/definitions/library.rs` | 17 | Medium (user types) |
| `core/src/definitions/types.rs` | 15 | Medium (user types) |
| `core/src/observability/otel/config.rs` | 16 | Low (config) |
| `core/src/metrics/grafana.rs` | 16 | Low (internal) |

## Documentation Warnings (28 total)

### Broken Links (5)
- `unresolved link to 'stub'`
- `unresolved link to 'FfiMarshaler'`
- `unresolved link to 'FfiStreamMarshaler::get_chunk'`
- `unresolved link to 'FfiStreamMarshaler::from_chunks'`
- `unresolved link to 'InputValidator'`

### Private Item Links (12)
- `config` links to private: `timing`, `paths`, `limits`, `loader`
- `profile` links to private: `storage`, `resolution`
- `log` links to private: `entry`, `ring_buffer`
- `html` links to private: `templates`, `rendering`
- `engine` links to private: `context`, `rhai_engine`
- `PendingQueueBounds` links to private: `MAX_PENDING_DECISIONS`

### HTML Tag Issues (11)
- Unclosed `<HashMap>` (2)
- Unclosed `<Profile>` (2)
- Unclosed `<T>` (3)
- Unclosed `<Utf8>` (3)
- Unclosed `<code>`, `<text>`, `<layer>` (1 each)

### Other (2)
- Redundant explicit link targets (2)
- `array` is both module and primitive type (1)

## Prioritized Documentation Recommendations

### Critical Priority (Must Fix)

1. **Services module** (24 items)
   - Add module-level docs to `services/mod.rs`
   - Document `DeviceServiceError`, `ProfileServiceError`, `RuntimeServiceError`
   - Document `ProfileService`, `RuntimeService` structs
   - Document public constructors (`new()`, `with_defaults()`)

2. **FFI contracts** (71 items in `ffi/contract.rs`)
   - Document all contract types used by external bindings
   - Critical for Dart/Flutter integration

3. **Core error types** (31 items in `errors/critical.rs`)
   - Document error enum variants for user troubleshooting

4. **Module-level documentation** (21 modules)
   - Add `//!` doc comments to all public modules
   - Focus on: `services`, `ffi`, `engine`, `config`, `discovery`

### High Priority (Should Fix)

5. **Engine state types** (~100 items)
   - `state/modifiers.rs`, `state/mutation.rs`, `state/error.rs`
   - Document key decision-making types

6. **FFI introspection** (33 items)
   - Types used for runtime introspection

7. **Fix documentation warnings** (28 warnings)
   - Replace broken links with valid references
   - Escape HTML-like type parameters (`<T>` → `T`)
   - Remove references to private items

### Medium Priority (Nice to Have)

8. **CLI command structs** (~100 items)
   - Document command parameters
   - Add usage examples

9. **Config models** (40 items)
   - Document configuration fields
   - Add validation constraints

10. **Scripting builtins** (75 items)
    - Document scripting API for users

### Low Priority (Consider Later)

11. **Internal struct fields** (~500 items)
    - Most field names are self-explanatory
    - Document only complex fields

12. **Test-related types**
    - Skip if only used in tests

## Recommendations for Phase 5 Tasks

### Task 5.2 - Critical Public APIs
Focus on:
1. All items in `services/` directory
2. `ffi/contract.rs` - FFI contracts
3. `errors/critical.rs` - Error types
4. Module-level docs for all 21 undocumented modules

Estimated items: ~150

### Task 5.3 - Remaining Public APIs
Focus on:
1. Engine state types
2. CLI commands
3. Config models
4. Fix all 28 documentation warnings

Estimated items: ~450 (prioritize, may skip trivial fields)

## Metrics Summary

| Metric | Value |
|--------|-------|
| Total missing doc items | 1,069 |
| Critical path items (services/ffi/engine) | 342 |
| Documentation warnings | 28 |
| Undocumented modules | 21 |
| Undocumented public functions/methods | 116 |
| Undocumented types (struct/enum/trait) | 26 |

## Conclusion

Documentation coverage is poor, with over 1,000 missing items. However, most missing items (669) are struct fields which are often self-explanatory.

**Key priorities:**
1. Module-level documentation (21 modules) - highest impact
2. Services API documentation (24 items) - critical path
3. FFI contracts (71 items) - external interface
4. Fix broken links and warnings (28) - clean builds

A realistic target is **95%+ coverage** excluding trivial struct fields, achieved by:
- Documenting all 21 modules
- Documenting all 26 undocumented types
- Documenting all 116 public functions/methods
- Documenting important error variants (~100)
- Fixing all 28 warnings
