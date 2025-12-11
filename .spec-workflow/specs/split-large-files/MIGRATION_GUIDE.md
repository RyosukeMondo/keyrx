# Migration Guide: Large File Splits

This guide helps contributors navigate the new module structure after the large file splitting refactoring.

## Overview

### What Changed and Why

The keyrx codebase underwent a significant refactoring to improve maintainability:

- **Problem**: 73 files exceeded the 500-line limit, with the largest at 1,893 lines
- **Solution**: Top 10 largest files were split into focused submodules
- **Impact**: ~10,000 lines reorganized into 30+ smaller, focused modules
- **Result**: Better code organization, faster incremental builds, easier code review

### Key Principle

**All public APIs remain unchanged.** Re-exports in `mod.rs` files ensure backward compatibility. Your existing `use` statements should continue to work.

---

## File Location Changes

### Quick Reference Table

| Original File | New Location | Submodules |
|--------------|--------------|------------|
| `scripting/bindings.rs` | `scripting/bindings/` | mod.rs, remapping.rs, tap_hold.rs, layers.rs, layouts.rs, modifiers.rs, timing.rs, row_col.rs, keyboard.rs |
| `engine/state/mod.rs` | `engine/state/` | mod.rs, key_state.rs, modifiers.rs, layers.rs, + existing submodules |
| `engine/transitions/log.rs` | `engine/transitions/log/` | mod.rs, entry.rs, ring_buffer.rs, stub.rs, tests.rs |
| `bin/keyrx.rs` | `bin/keyrx/` | main.rs, commands_core.rs, commands_config.rs, commands_test.rs |
| `scripting/docs/generators/html.rs` | `scripting/docs/generators/html/` | mod.rs, templates.rs, rendering.rs |
| `validation/engine.rs` | `validation/engine/` | mod.rs, context.rs, rhai_engine.rs, rules.rs, report.rs |
| `config/loader.rs` | `config/loader/` | mod.rs, parsing.rs, validation.rs |
| `registry/profile.rs` | `registry/profile/` | mod.rs, storage.rs, resolution.rs |
| `engine/advanced.rs` | `engine/advanced/` | mod.rs, processing.rs, combos.rs, tests.rs |
| `cli/commands/run.rs` | `cli/commands/run/` | mod.rs, setup.rs, execution.rs |

---

## Detailed Migration by Module

### 1. Scripting Bindings

**Before:**
```rust
use keyrx_core::scripting::bindings::register_all_functions;
use keyrx_core::scripting::bindings::register_remap;
```

**After (unchanged - re-exports work):**
```rust
use keyrx_core::scripting::bindings::register_all_functions;
use keyrx_core::scripting::bindings::register_remap;
```

**New structure:**
```
scripting/bindings/
├── mod.rs          # Re-exports, register_all_functions()
├── keyboard.rs     # Keyboard-related bindings
├── remapping.rs    # remap(), block(), pass() functions
├── tap_hold.rs     # tap_hold(), combo() functions
├── layers.rs       # layer_define(), layer_push/pop/toggle()
├── layouts.rs      # layout_define(), layout_enable()
├── modifiers.rs    # modifier functions, one_shot()
├── timing.rs       # Timeout configuration functions
└── row_col.rs      # Row-column API variants (*_rc functions)
```

**Finding specific functions:**
| Function | New Location |
|----------|--------------|
| `register_remap`, `register_block`, `register_pass` | `remapping.rs` |
| `register_tap_hold`, `register_combo` | `tap_hold.rs` |
| `register_layer_functions` | `layers.rs` |
| `register_layout_functions` | `layouts.rs` |
| `register_modifier_functions` | `modifiers.rs` |
| `register_timing_functions` | `timing.rs` |
| `*_rc` variants | `row_col.rs` |

---

### 2. Engine State

**Before:**
```rust
use keyrx_core::engine::state::EngineState;
use keyrx_core::engine::state::ModifierSet;
```

**After (unchanged):**
```rust
use keyrx_core::engine::state::EngineState;
use keyrx_core::engine::state::ModifierSet;
```

**New structure:**
```
engine/state/
├── mod.rs              # EngineState, ModifierSet, re-exports
├── key_state.rs        # Key state tracking
├── modifiers.rs        # Modifier state management
├── layers.rs           # Layer stack operations
├── mutation.rs         # State mutation operations
├── delta.rs            # State delta tracking
├── snapshot.rs         # State snapshots
├── persistence.rs      # State persistence
└── engine_state_tests.rs # Comprehensive tests
```

---

### 3. Transition Log

**Before:**
```rust
use keyrx_core::engine::transitions::log::TransitionLog;
use keyrx_core::engine::transitions::log::TransitionEntry;
```

**After (unchanged):**
```rust
use keyrx_core::engine::transitions::log::TransitionLog;
use keyrx_core::engine::transitions::log::TransitionEntry;
```

**New structure:**
```
engine/transitions/log/
├── mod.rs          # Re-exports, feature flag gating
├── entry.rs        # TransitionEntry struct
├── ring_buffer.rs  # TransitionLog implementation (feature-gated)
├── stub.rs         # No-op stub for disabled feature
└── tests.rs        # Test suite
```

---

### 4. Binary (keyrx CLI)

**Before:**
```
bin/keyrx.rs  # Single 1,382-line file
```

**After:**
```
bin/keyrx/
├── main.rs            # Entry point, CLI struct, main()
├── commands_core.rs   # run, simulate, check, discover
├── commands_config.rs # devices, hardware, layout, keymap, runtime
└── commands_test.rs   # test, replay, analyze, uat, regression, doctor, repl
```

**Finding command handlers:**
| Command | Location |
|---------|----------|
| `run`, `simulate`, `check`, `discover` | `commands_core.rs` |
| `devices`, `hardware`, `layout`, `keymap`, `runtime` | `commands_config.rs` |
| `test`, `replay`, `analyze`, `uat`, `regression`, `doctor`, `repl` | `commands_test.rs` |

---

### 5. HTML Documentation Generator

**Before:**
```rust
use keyrx_core::scripting::docs::generators::html::generate_html;
```

**After (unchanged):**
```rust
use keyrx_core::scripting::docs::generators::html::generate_html;
```

**New structure:**
```
scripting/docs/generators/html/
├── mod.rs        # generate_html() entry point
├── templates.rs  # HTML/CSS templates
└── rendering.rs  # Content rendering logic
```

---

### 6. Validation Engine

**Before:**
```rust
use keyrx_core::validation::engine::ValidationEngine;
use keyrx_core::validation::engine::ScriptContext;
```

**After (unchanged):**
```rust
use keyrx_core::validation::engine::ValidationEngine;
use keyrx_core::validation::engine::ScriptContext;
```

**New structure:**
```
validation/engine/
├── mod.rs          # ValidationEngine, re-exports
├── context.rs      # ScriptContext, LocatedOp, ParsedScript
├── rhai_engine.rs  # Rhai engine setup
├── rules.rs        # Validation rule implementations
└── report.rs       # Error reporting, formatting
```

---

### 7. Config Loader

**Before:**
```rust
use keyrx_core::config::loader::load_config;
use keyrx_core::config::loader::Config;
```

**After (unchanged):**
```rust
use keyrx_core::config::loader::load_config;
use keyrx_core::config::loader::Config;
```

**New structure:**
```
config/loader/
├── mod.rs         # Main API, Config struct
├── parsing.rs     # TOML/YAML parsing
└── validation.rs  # Config validation
```

---

### 8. Profile Registry

**Before:**
```rust
use keyrx_core::registry::profile::ProfileRegistry;
use keyrx_core::registry::profile::Profile;
```

**After (unchanged):**
```rust
use keyrx_core::registry::profile::ProfileRegistry;
use keyrx_core::registry::profile::Profile;
```

**New structure:**
```
registry/profile/
├── mod.rs         # ProfileRegistry, Profile, re-exports
├── storage.rs     # Persistence, file I/O
└── resolution.rs  # Profile resolution logic
```

---

### 9. Advanced Engine

**Before:**
```rust
use keyrx_core::engine::advanced::AdvancedEngine;
```

**After (unchanged):**
```rust
use keyrx_core::engine::advanced::AdvancedEngine;
```

**New structure:**
```
engine/advanced/
├── mod.rs         # AdvancedEngine struct, core API
├── processing.rs  # Event processing logic
├── combos.rs      # Combo detection and handling
└── tests.rs       # Test suite
```

---

### 10. Run Command

**Before:**
```rust
use keyrx_core::cli::commands::run::RunCommand;
```

**After (unchanged):**
```rust
use keyrx_core::cli::commands::run::RunCommand;
```

**New structure:**
```
cli/commands/run/
├── mod.rs        # RunCommand struct, public API
├── setup.rs      # Engine/config initialization
└── execution.rs  # Main execution loop
```

---

## Finding Relocated Code

### Using grep/ripgrep

```bash
# Find where a function moved to
rg "fn your_function_name" core/src/

# Find a struct definition
rg "struct YourStruct" core/src/

# Find all files in a split module
fd . core/src/scripting/bindings/
```

### Using IDE Features

Most IDEs with Rust support (VS Code + rust-analyzer, IntelliJ IDEA) can:
- Jump to definition (F12 or Ctrl+Click)
- Find all references
- Navigate via symbols

Re-exports mean IDE navigation still works through the original paths.

---

## Frequently Asked Questions

### Q: Do I need to update my imports?

**A: No.** All public APIs are re-exported from `mod.rs` files. Your existing imports should continue to work without changes.

### Q: Where do I add new functions?

**A:** Add new functions to the appropriate submodule based on functionality:
- Key remapping → `remapping.rs`
- Layer operations → `layers.rs`
- Timing configuration → `timing.rs`
- etc.

Then add a re-export in `mod.rs` if the function should be publicly accessible.

### Q: How do I know which submodule to edit?

**A:** Each submodule has a focused responsibility:
1. Read the module-level documentation in each file
2. Look at existing functions in the file for context
3. When in doubt, check the tables in this guide

### Q: What if I need to add cross-cutting functionality?

**A:** If functionality spans multiple submodules:
1. Add it to the most appropriate submodule
2. Use `pub(crate)` for internal helpers
3. Re-export only the public API from `mod.rs`

### Q: Are there any breaking changes?

**A: No.** The refactoring maintains 100% API compatibility. All public types, functions, and traits remain accessible through the same paths.

### Q: Why were these specific files chosen?

**A:** The top 10 largest files were selected based on:
1. Line count (all exceeded 900 lines, target is <500)
2. Clear domain boundaries for splitting
3. Impact on build times
4. Code review complexity

### Q: Will there be more splits?

**A:** Possibly. 63 files still exceed 500 lines. Future work may address files in the 500-800 line range if needed.

---

## Summary Statistics

| Metric | Before | After |
|--------|--------|-------|
| Files split | 10 | 30+ submodules |
| Largest file | 1,893 lines | <500 lines |
| Top 10 total lines | ~11,500 | Distributed |
| API compatibility | - | 100% |
| Tests passing | - | All |

---

## Getting Help

- **Code questions**: Check the module-level documentation in each `mod.rs`
- **Build issues**: Run `cargo build` and check error messages for path hints
- **Finding code**: Use `rg` (ripgrep) or IDE search
- **Architecture questions**: See `split_plans.md` for design rationale
