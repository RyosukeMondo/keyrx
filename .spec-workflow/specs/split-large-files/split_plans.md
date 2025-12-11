# Split Plans for Large Files

This document contains analysis and split recommendations for the top 10 largest files in the keyrx core codebase.

## Summary

| File | Lines | Recommended Split | Priority |
|------|-------|-------------------|----------|
| scripting/bindings.rs | 1,893 | 5 modules | High |
| engine/state/mod.rs | 1,570 | Already well-organized | Low |
| engine/transitions/log.rs | 1,403 | 3 modules | Medium |
| bin/keyrx.rs | 1,382 | 4 modules | High |
| scripting/docs/generators/html.rs | 1,069 | 3 modules | Medium |
| validation/engine.rs | 968 | 3 modules | Medium |
| config/loader.rs | 949 | 4 modules | Medium |
| registry/profile.rs | 918 | 3 modules | Medium |
| engine/advanced.rs | 906 | 3 modules | Medium |
| cli/commands/run.rs | 899 | 3 modules | Medium |

---

## 1. scripting/bindings.rs (1,893 lines)

### Current Structure
Rhai function bindings organized into registration groups:
- Debug functions (~20 lines)
- Remap functions (~60 lines)
- Block/Pass functions (~80 lines)
- Tap-hold functions (~180 lines)
- Combo functions (~60 lines)
- Layout functions (~200 lines)
- Layer functions (~330 lines)
- Modifier functions (~280 lines)
- Timing functions (~250 lines)
- Row-Column API functions (~400 lines)

### Recommended Split

**Module: `scripting/bindings/`**

1. **`mod.rs`** (~100 lines)
   - `register_all_functions()` orchestrator
   - Re-exports from submodules

2. **`remapping.rs`** (~300 lines)
   - `register_remap()`, `register_block()`, `register_pass()`
   - `remap_impl()`, `block_impl()`, `pass_impl()`

3. **`tap_hold.rs`** (~250 lines)
   - `register_tap_hold()`, `register_tap_hold_mod()`
   - `tap_hold_impl()`, `tap_hold_mod_impl()`
   - `register_combo()`, `combo_impl()`

4. **`layers.rs`** (~400 lines)
   - `register_layer_functions()` orchestrator
   - `layer_define_impl()`, `layer_map_impl()`
   - `layer_push_impl()`, `layer_pop_impl()`, `layer_toggle_impl()`
   - `is_layer_active_impl()`
   - Layout functions: `layout_define_impl()`, `layout_enable_impl()`, etc.
   - Helper functions: `normalize_layer_name()`, `normalize_layout_id()`, etc.

5. **`modifiers.rs`** (~350 lines)
   - `register_modifier_functions()` orchestrator
   - `define_modifier_impl()`, `modifier_on_impl()`, `modifier_off_impl()`
   - `one_shot_impl()`, `is_modifier_active_impl()`
   - Timing functions: `set_tap_timeout_impl()`, `set_combo_timeout_impl()`, etc.

6. **`row_col.rs`** (~450 lines)
   - Row-column API functions
   - `remap_rc_impl()`, `tap_hold_rc_impl()`, `block_rc_impl()`
   - `combo_rc_impl()`, `layer_map_rc_impl()`

### Priority: **High**
- File is nearly 4x the 500-line target
- Clear functional boundaries already exist with `register_*` groups

---

## 2. engine/state/mod.rs (1,570 lines)

### Current Structure
Already well-organized with submodules:
- Module declarations and re-exports (~45 lines)
- `ModifierSet` struct (~50 lines)
- `EngineState` struct and impl (~650 lines)
- Tests (~800 lines)

### Recommended Split

The file is already well-organized with most components in separate submodules. The main content is:
- `ModifierSet` - Small utility type
- `EngineState` - Unified state container with query/mutation methods
- Tests - Comprehensive test suite

**Option A: Extract tests**
Move tests to `engine/state/tests.rs` (~800 lines), leaving ~770 lines.

**Option B: Leave as-is**
The state module is a cohesive unit. The `EngineState` struct needs all its methods together for clarity. Tests could be extracted but this is low priority.

### Priority: **Low**
- Tests account for ~50% of the file
- Core struct needs cohesion
- Good candidate for test extraction only

---

## 3. engine/transitions/log.rs (1,403 lines)

### Current Structure
- `TransitionEntry` struct and impl (~210 lines)
- Tests for TransitionEntry (~260 lines)
- `TransitionLog` with feature flag (~240 lines)
- Stub implementation for disabled feature (~140 lines)
- Feature tests (~70 lines)
- Transition log tests (~440 lines)

### Recommended Split

**Module: `engine/transitions/log/`**

1. **`mod.rs`** (~50 lines)
   - Re-exports from submodules
   - Feature flag gating

2. **`entry.rs`** (~250 lines)
   - `TransitionEntry` struct and impl
   - `state_diff_summary()` method

3. **`ring_buffer.rs`** (~350 lines)
   - `TransitionLog` struct and impl (feature-gated)
   - Search methods: `search_by_category()`, `search_by_name()`, etc.
   - Export methods: `export_json()`, `export_json_pretty()`, etc.
   - `statistics()` method

4. **`stub.rs`** (~150 lines)
   - Zero-overhead stub implementation for disabled feature
   - All no-op methods

5. **`tests.rs`** (~600 lines)
   - All test modules combined

### Priority: **Medium**
- Tests are ~60% of the file
- Clear separation between entry and log types
- Feature flag complexity suggests modular approach

---

## 4. bin/keyrx.rs (1,382 lines)

### Current Structure
CLI main binary with:
- Imports and global CLI struct (~100 lines)
- `Commands` enum with all subcommands (~340 lines)
- Hardware/Layout/Keymap/Runtime/Device subcommand enums (~290 lines)
- Golden subcommands (~80 lines)
- Helper functions (~60 lines)
- `main()` function (~70 lines)
- `run_command()` function (~470 lines)
- Tests (~40 lines)

### Recommended Split

**Module: `cli/commands/` (already exists, extend it)**

1. **`bin/keyrx.rs`** (~300 lines)
   - Global CLI struct
   - `main()` function
   - `run_command()` dispatch (can be simplified with match delegation)

2. **`cli/args/mod.rs`** (new) (~150 lines)
   - Re-exports
   - Helper functions: `parse_format()`, `parse_hex_or_decimal_u16()`

3. **`cli/args/commands.rs`** (new) (~350 lines)
   - `Commands` enum definition
   - All subcommand argument structs

4. **`cli/args/subcommands.rs`** (new) (~300 lines)
   - Hardware subcommands enum
   - Layout subcommands enum
   - Keymap subcommands enum
   - Runtime subcommands enum
   - Device subcommands enum
   - Golden subcommands enum

5. **`cli/dispatch.rs`** (new) (~350 lines)
   - `run_command()` implementation
   - Command-to-action conversion logic

### Priority: **High**
- Main binary should be minimal
- Command definitions are verbose
- Clear separation between args, dispatch, and execution

---

## 5. scripting/docs/generators/html.rs (1,069 lines)

### Current Structure
HTML documentation generator:
- Public `generate_html()` function (~70 lines)
- Module collection helpers (~50 lines)
- `ModuleData` struct (~20 lines)
- HTML generation functions (~200 lines)
- Helper functions (`escape_html`, `format_signature`) (~40 lines)
- `html_header()` - CSS (~310 lines)
- `html_scripts()` - JavaScript (~150 lines)
- Tests (~190 lines)

### Recommended Split

**Module: `scripting/docs/generators/html/`**

1. **`mod.rs`** (~150 lines)
   - `generate_html()` main function
   - `collect_modules()` helper
   - `ModuleData` struct

2. **`content.rs`** (~250 lines)
   - `generate_module_html()`
   - `generate_type_html()`
   - `generate_function_html()`
   - `generate_param_html()`, `generate_property_html()`, `generate_method_html()`
   - `format_signature()`, `escape_html()`

3. **`styles.rs`** (~320 lines)
   - `html_header()` function
   - All CSS styles as const string

4. **`scripts.rs`** (~160 lines)
   - `html_scripts()` function
   - JavaScript search functionality

5. **`tests.rs`** (~200 lines)
   - All test functions

### Priority: **Medium**
- CSS/JS are large static strings that benefit from isolation
- Content generation is distinct from styling

---

## 6. validation/engine.rs (968 lines)

### Current Structure
Validation engine orchestrator:
- `LocatedOp` struct (~15 lines)
- `ScriptContext` struct and impl (~60 lines)
- `ValidationEngine` struct and impl (~180 lines)
- Helper functions (`collect_definitions`, `find_operation_line`) (~80 lines)
- `create_validation_engine()` - Rhai engine setup (~250 lines)
- `parse_error_to_validation_error()` (~20 lines)
- Tests (~360 lines)

### Recommended Split

**Module: `validation/engine/`**

1. **`mod.rs`** (~100 lines)
   - Re-exports
   - `ValidationEngine` struct definition
   - Constructor methods

2. **`validation.rs`** (~200 lines)
   - `ValidationEngine::validate()` method
   - `ValidationEngine::validate_with_visual()` method
   - `run_semantic_validation()`, `run_timing_validation()`, `run_conflict_detection()`

3. **`parsing.rs`** (~350 lines)
   - `parse_script()`, `parse_script_with_context()` methods
   - `create_validation_engine()` - Rhai engine setup
   - `parse_error_to_validation_error()`

4. **`context.rs`** (~100 lines)
   - `LocatedOp` struct
   - `ScriptContext` struct and impl
   - `ParsedScript` struct
   - Helper functions: `collect_definitions()`, `find_operation_line()`, `populate_context_from_ops()`

5. **`tests.rs`** (~360 lines)
   - All test functions

### Priority: **Medium**
- Rhai engine setup is verbose and self-contained
- Tests are significant portion

---

## 7. config/loader.rs (949 lines)

### Current Structure
Configuration loading with:
- Configuration structs (~200 lines)
- Loading functions (~150 lines)
- Validation functions (~100 lines)
- CLI override functions (~100 lines)
- Migration functions (~100 lines)
- Tests (~300 lines)

### Recommended Split

**Module: `config/loader/`**

1. **`mod.rs`** (~80 lines)
   - Re-exports
   - `load_config()` main entry point

2. **`structs.rs`** (~200 lines)
   - `Config` struct
   - `TimingConfig` struct
   - All configuration data structures
   - Default implementations

3. **`loading.rs`** (~200 lines)
   - TOML parsing logic
   - File reading
   - Path resolution
   - Error handling

4. **`validation.rs`** (~150 lines)
   - Range validation
   - Required field checks
   - Type validation

5. **`cli.rs`** (~100 lines)
   - `merge_cli_overrides()` function
   - Command-line argument processing

6. **`migration.rs`** (~100 lines)
   - Version migration logic
   - Schema updates

7. **`tests.rs`** (~300 lines)
   - All test functions

### Priority: **Medium**
- Clear functional boundaries
- Tests are large portion

---

## 8. registry/profile.rs (918 lines)

### Current Structure
Profile data model:
- Type definitions (`PhysicalPosition`, `LayoutType`, `KeyAction`, `MappingRule`) (~200 lines)
- `Profile` struct and impl (~250 lines)
- `ProfileRegistry` struct and impl (~200 lines)
- Tests for Profile (~150 lines)
- Tests for ProfileRegistry (~120 lines)

### Recommended Split

**Module: `registry/profile/`**

1. **`mod.rs`** (~50 lines)
   - Re-exports from submodules

2. **`types.rs`** (~200 lines)
   - `PhysicalPosition` struct
   - `LayoutType` enum
   - `KeyAction` enum
   - `MappingRule` struct
   - Position calculation helpers

3. **`profile.rs`** (~280 lines)
   - `Profile` struct
   - Profile builder methods
   - Serialization/deserialization
   - Validation

4. **`registry.rs`** (~250 lines)
   - `ProfileRegistry` struct
   - Async CRUD operations
   - File I/O
   - Caching

5. **`tests.rs`** (~280 lines)
   - Profile tests
   - Registry tests

### Priority: **Medium**
- Clear separation between types, profile, and registry
- Tests are substantial

---

## 9. engine/advanced.rs (906 lines)

### Current Structure
Advanced remapping engine:
- `KeyStateView` adapter (~50 lines)
- `AdvancedEngine` struct (~80 lines)
- Core engine methods (~200 lines)
- Tick/timing methods (~100 lines)
- Combo handling (~150 lines)
- Layer action methods (~100 lines)
- Safe mode methods (~50 lines)
- Tests (~180 lines)

### Recommended Split

**Module: `engine/advanced/`**

1. **`mod.rs`** (~150 lines)
   - Re-exports
   - `AdvancedEngine` struct definition
   - Constructor and core fields

2. **`state_view.rs`** (~80 lines)
   - `KeyStateView` adapter struct
   - State query methods

3. **`processing.rs`** (~250 lines)
   - `process_event()` method
   - `process_key_down()`, `process_key_up()`
   - Core event handling logic

4. **`timing.rs`** (~200 lines)
   - `tick()` method
   - Timeout handling
   - Timer management

5. **`combos.rs`** (~150 lines)
   - Combo detection
   - Combo resolution
   - Combo state management

6. **`tests.rs`** (~200 lines)
   - Tap-hold tests
   - Combo tests
   - Safe mode tests

### Priority: **Medium**
- Complex logic benefits from separation
- Timing and combo logic are distinct

---

## 10. cli/commands/run.rs (899 lines)

### Current Structure
Engine run command:
- `DeviceRuntime` helper struct (~100 lines)
- `RunCommand` struct (~150 lines)
- Builder methods (~100 lines)
- Mock run implementation (~100 lines)
- Linux run implementation (~200 lines)
- Windows run implementation (~150 lines)
- Common run helpers (~100 lines)

### Recommended Split

**Module: `cli/commands/run/`**

1. **`mod.rs`** (~150 lines)
   - Re-exports
   - `RunCommand` struct definition
   - Builder methods
   - `execute()` dispatch

2. **`device_runtime.rs`** (~120 lines)
   - `DeviceRuntime` helper struct
   - Device abstraction layer

3. **`mock.rs`** (~120 lines)
   - Mock run implementation
   - Testing utilities

4. **`linux.rs`** (~220 lines)
   - Linux-specific run implementation
   - Evdev handling
   - uinput setup

5. **`windows.rs`** (~170 lines)
   - Windows-specific run implementation
   - Win32 API integration

6. **`common.rs`** (~120 lines)
   - Shared run logic
   - Signal handling
   - Recording functionality

### Priority: **Medium**
- Platform-specific code should be isolated
- Clear separation between mock and real implementations

---

## Implementation Order

### Phase 1 (High Priority)
1. `scripting/bindings.rs` - Largest file, clear split points
2. `bin/keyrx.rs` - Main binary should be minimal

### Phase 2 (Medium Priority - Engine)
3. `engine/transitions/log.rs` - Feature flag complexity
4. `engine/advanced.rs` - Complex logic separation
5. `engine/state/mod.rs` - Test extraction only

### Phase 3 (Medium Priority - Other)
6. `validation/engine.rs` - Rhai engine setup isolation
7. `config/loader.rs` - Clear functional boundaries
8. `registry/profile.rs` - Type/registry separation
9. `cli/commands/run.rs` - Platform isolation
10. `scripting/docs/generators/html.rs` - CSS/JS extraction

---

## Notes

- All splits should maintain backward compatibility via re-exports in `mod.rs`
- Tests can often be extracted first as a quick win
- Feature-gated code should be in separate modules where possible
- Each new module should have its own focused test file
