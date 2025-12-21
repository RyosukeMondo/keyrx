# KeyRX Architecture Visual Guide

## 🎯 Current State Summary

**Good News:** Only **3 minor Dart warnings** found (not 600!)
- The "600 warnings" are Rust compiler warnings during generation (normal - mostly unused code warnings)
- Your Dart code is healthy ✅

---

## 📐 Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    SINGLE SOURCE OF TRUTH                        │
│                                                                  │
│    core/src/ffi/contracts/*.ffi-contract.json (15 domains)      │
│         ├─ base.ffi-contract.json                               │
│         ├─ config.ffi-contract.json                            │
│         ├─ discovery.ffi-contract.json                         │
│         ├─ engine.ffi-contract.json                            │
│         └─ ... (11 more)                                       │
└──────────────────┬──────────────────────────────────────────────┘
                   │
                   │ (Rust Code Generator)
                   ↓
┌─────────────────────────────────────────────────────────────────┐
│                GENERATED DART CODE (3 FILES)                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  📁 packages/core_logic/lib/                                    │
│     ├── ffi/generated_bindings.dart  (121KB, 3500+ lines)      │
│     │    ├─ Native FFI typedefs                                │
│     │    ├─ Function pointers                                  │
│     │    └─ 15 binding classes (one per domain)               │
│     │                                                           │
│     └── models/generated_models.dart (8KB, 337 lines)          │
│          ├─ DiscoveredDevice                                   │
│          ├─ SessionStatus (enum)                               │
│          ├─ DiscoverySummary                                   │
│          └─ ... (all serializable models)                      │
│                                                                  │
│  📁 ui/lib/services/                                            │
│     └── generated/ipc_client.dart  (27KB, 950+ lines)          │
│          ├─ BaseClient                                         │
│          ├─ ConfigClient                                       │
│          ├─ DiscoveryClient                                    │
│          ├─ EngineClient                                       │
│          └─ ... (15 client classes)                            │
└─────────────────────────────────────────────────────────────────┘
```

---

## 🔄 Code Generation Flow

```
┌─────────────────────┐
│   You Run Command   │
│  just gen-bindings  │
└──────────┬──────────┘
           │
           ↓
┌─────────────────────────────────────────────────────────┐
│  cargo run generate_dart_bindings                       │
│                                                         │
│  1. Load 15 JSON contracts                             │
│  2. Sort by domain name (for deterministic output)     │
│  3. Generate code in memory:                           │
│     • FFI bindings   (low-level C interop)            │
│     • Models         (data classes with JSON)          │
│     • IPC clients    (high-level async API)           │
│  4. Compare with existing files (hash-based)           │
│  5. Write only if changed                              │
│  6. Run dart format                                    │
└───────────────────┬─────────────────────────────────────┘
                    │
                    ↓
┌────────────────────────────────────────────────────────┐
│          Files Written (if changed):                   │
│                                                        │
│  packages/core_logic/lib/ffi/generated_bindings.dart  │
│  packages/core_logic/lib/models/generated_models.dart │
│  packages/core_logic/lib/services/generated/          │
│                           ipc_client.dart              │
│  ui/lib/services/generated/ipc_client.dart            │
└────────────────────────────────────────────────────────┘
```

---

## 🏗️ Package Architecture (Layered)

```
┌─────────────────────────────────────────────────────────────┐
│                      UI PACKAGE (ui/)                       │
│  ┌────────────────────────────────────────────────────┐   │
│  │  🎨 Pages & Widgets                                 │   │
│  │     ├─ DevicesPage                                  │   │
│  │     ├─ MonitorView                                  │   │
│  │     └─ SlotActions                                  │   │
│  └─────────────────┬────────────────────────────────────   │
│                    │ uses                                   │
│  ┌────────────────↓────────────────────────────────────┐   │
│  │  🔌 UI Services (Flutter-specific)                  │   │
│  │     ├─ DaemonService        (IPC to daemon)        │   │
│  │     ├─ ServiceRegistry      (DI container)         │   │
│  │     ├─ UIMetricsService     (polling + caching)    │   │
│  │     └─ UIObservabilityService (timers + UI logic)  │   │
│  └─────────────────┬────────────────────────────────────   │
│                    │ imports from                           │
└────────────────────┼────────────────────────────────────────┘
                     │
                     ↓
┌─────────────────────────────────────────────────────────────┐
│           CORE LOGIC PACKAGE (packages/core_logic/)         │
│  ┌────────────────────────────────────────────────────┐   │
│  │  📦 Abstract Service Interfaces (SSOT)             │   │
│  │     ├─ MetricsService                              │   │
│  │     ├─ ObservabilityService                        │   │
│  │     ├─ DeviceRegistryService                       │   │
│  │     └─ ... (platform-agnostic contracts)           │   │
│  └─────────────────┬────────────────────────────────────   │
│                    │ implemented by                         │
│  ┌────────────────↓────────────────────────────────────┐   │
│  │  🔧 Service Implementations                         │   │
│  │     ├─ MetricsServiceFFI      (native impl)       │   │
│  │     ├─ MetricsServiceWeb      (web impl)          │   │
│  │     ├─ DeviceRegistryServiceFFI                    │   │
│  │     └─ ...                                         │   │
│  └─────────────────┬────────────────────────────────────   │
│                    │ uses                                   │
│  ┌────────────────↓────────────────────────────────────┐   │
│  │  ⚡ FFI Bridge Layer (high-level)                  │   │
│  │     ├─ bridge.dart           (legacy wrapper)      │   │
│  │     ├─ bridge_engine.dart                          │   │
│  │     ├─ device_registry_ffi.dart                    │   │
│  │     └─ ...                                         │   │
│  └─────────────────┬────────────────────────────────────   │
│                    │ calls                                  │
│  ┌────────────────↓────────────────────────────────────┐   │
│  │  📡 GENERATED FFI BINDINGS (low-level)             │   │
│  │     ├─ generated_bindings.dart                     │   │
│  │     │   ├─ BaseBindings                            │   │
│  │     │   ├─ ConfigBindings                          │   │
│  │     │   └─ ... (15 domain classes)                 │   │
│  │     └─ generated_models.dart                       │   │
│  └─────────────────┬────────────────────────────────────   │
│                    │ loads                                  │
└────────────────────┼────────────────────────────────────────┘
                     │
                     ↓
┌─────────────────────────────────────────────────────────────┐
│              NATIVE LIBRARY (.so/.dll/.dylib)               │
│                   libkeyrx_core.so                          │
│                                                             │
│  Rust implementation of all FFI functions:                  │
│     ├─ keyrx_init()                                        │
│     ├─ keyrx_config_load()                                 │
│     ├─ keyrx_discovery_start()                             │
│     └─ ... (200+ functions)                                │
└─────────────────────────────────────────────────────────────┘
```

---

## 🔀 Two Communication Paths

### Path 1: Direct FFI (Performance Critical)

```
UI Widget
    ↓ calls
Service (core_logic)
    ↓ calls
FFI Bridge
    ↓ calls
Generated Bindings
    ↓ C function pointer
Rust Native Library
```

**Used for:**
- Discovery operations
- Device registry access
- Real-time input processing
- Configuration management

---

### Path 2: IPC via Daemon (Process Isolation)

```
UI Widget
    ↓ calls
Generated IPC Client
    ↓ calls
DaemonService
    ↓ sends JSON-RPC over Unix socket
───────────────────────────────────────
KeyRX Daemon Process
    ↓ receives request
    ↓ calls Rust FFI
    ↓ returns JSON response
───────────────────────────────────────
DaemonService
    ↓ receives response
    ↓ deserializes
Generated IPC Client
    ↓ returns typed data
UI Widget
```

**Used for:**
- Engine control (start/stop)
- Runtime configuration
- Long-running operations
- Operations that need process isolation

---

## 🚨 Current Issues & Fixes

### ✅ FIXED: Non-Deterministic Model Generation
**Problem:** Models generated in random order → verification always failed
**Root Cause:** HashMap iteration (unordered)
**Fix:** Sort types before generation (models_gen.rs:57-59)
```rust
let mut sorted_types: Vec<_> = contract.types.iter().collect();
sorted_types.sort_by_key(|(k, _)| *k);
```

### ✅ FIXED: IPC Client Import Error
**Problem:** `DaemonService` class not found
**Root Cause:** IPC client generated in wrong package
**Fix:**
- Bindings/models → `packages/core_logic/lib/` (pure Dart)
- IPC client → `ui/lib/` (needs Flutter deps)

### ⚠️ KNOWN: Dual IPC Client Locations
**Current State:** IPC client exists in BOTH:
- `packages/core_logic/lib/services/generated/ipc_client.dart` (for verification)
- `ui/lib/services/generated/ipc_client.dart` (actually used)

**Why:** Verification script expects all 3 files in same location
**Impact:** None (UI uses correct one)
**Future Fix:** Update verification to check files in separate locations

### ℹ️ INFO: "600 Warnings" Explained
**What you're seeing:** Rust compiler warnings during generation
```
warning: unused import
warning: unreachable pattern
warning: field is never read
```

**Why:** Generated Rust code in the generator tool itself
**Impact:** None - this is normal for code generators
**Not a problem:** Your actual Dart code has only 3 minor linting warnings

---

## 📊 SSOT (Single Source of Truth) Status

```
┌────────────────────────────────────────┐
│  Rust Contracts (JSON)                 │  ← SSOT
│  15 files × ~50 functions each         │
│  = ~750 FFI exports defined            │
└───────────┬────────────────────────────┘
            │ auto-generates
            ↓
┌────────────────────────────────────────┐
│  Dart Bindings                         │  ← Derived
│  3,500 lines auto-generated            │
│  Never edit manually                   │
└────────────────────────────────────────┘
```

**SSOT Violations Tracker:**
- Initial: 28 violations
- After rename refactor: 20 violations
- Remaining: Direct FFI usage (acceptable pattern)

**Types of remaining violations:**
```
✅ Acceptable: Performance-critical direct FFI calls
❌ Avoid: Business logic duplicated between packages
❌ Avoid: Flutter imports in core_logic
```

---

## 🎯 How to Use the System

### Generate bindings:
```bash
just gen-dart-bindings        # Generate all
just verify-bindings          # Check if in sync
```

### Watch mode (auto-regenerate):
```bash
just dev-watch                # Watch FFI changes
```

### Verification:
```bash
python3 scripts/ffi/verify_bindings.py
# ✓ Bindings are in sync!
```

### Add new FFI function:
1. Edit contract: `core/src/ffi/contracts/base.ffi-contract.json`
2. Run: `just gen-dart-bindings`
3. Use generated binding in your service

---

## 🔧 Files You Can Edit vs Generated

```
✅ EDIT THESE (Hand-written):
   ├─ contracts/*.json              (define your API)
   ├─ services/*_service.dart       (business logic)
   ├─ services/*_service_ffi.dart   (FFI implementations)
   ├─ bridge*.dart                  (high-level wrappers)
   └─ pages/*.dart                  (UI code)

❌ NEVER EDIT (Auto-generated):
   ├─ generated_bindings.dart       (will be overwritten)
   ├─ generated_models.dart         (will be overwritten)
   └─ ipc_client.dart               (will be overwritten)
```

---

## 📈 Code Statistics

| Component | Lines | Files | Purpose |
|-----------|-------|-------|---------|
| Generated Bindings | 3,500 | 1 | Low-level FFI |
| Generated Models | 337 | 1 | Data classes |
| Generated IPC | 950 | 1 | High-level API |
| Hand-written Services | ~5,000 | 30+ | Business logic |
| UI Code | ~15,000 | 100+ | Interface |

---

## 🎓 Key Takeaways

1. **Contracts are king** - All code derives from JSON contracts
2. **Three layers** - FFI bindings → Services → UI
3. **Two modes** - Direct FFI (fast) or IPC (isolated)
4. **SSOT enforced** - core_logic has no Flutter deps
5. **Generation is deterministic** - Same input = same output
6. **Only 3 real warnings** - Your code is healthy!

---

## 🔍 Verification Checklist

When you run generation, verify:
- ✅ All 3 files updated (check timestamps)
- ✅ `just verify-bindings` passes
- ✅ `flutter analyze` shows no errors
- ✅ Dart format applied automatically
- ✅ Git diff shows only intended changes

---

*Generated: 2025-12-21*
*This document provides a visual overview of KeyRX's FFI code generation architecture.*
