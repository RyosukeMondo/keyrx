# Tasks Document: code-quality-refactor

## Phase 1: Foundation (Keycodes & Traits)

- [x] 1. Create keycode macro system
  - File: `core/src/drivers/keycodes.rs` (new)
  - Define `define_keycodes!` macro that generates:
    - KeyCode enum with all variants
    - Display impl
    - FromStr impl with aliases
    - `evdev_to_keycode()` / `keycode_to_evdev()` (cfg linux)
    - `vk_to_keycode()` / `keycode_to_vk()` (cfg windows)
    - `all_keycodes()` for uinput registration
  - Invoke macro with all 127 keycodes
  - Purpose: Single source of truth for all keycode definitions
  - _Leverage: existing keycode mappings from linux.rs, windows.rs, types.rs_
  - _Requirements: REQ-1, REQ-7_
  - _Prompt: Implement the task for spec code-quality-refactor, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Macro Developer specializing in declarative macros | Task: Create define_keycodes! macro in core/src/drivers/keycodes.rs that generates KeyCode enum and all conversion functions from a single definition, extracting existing mappings from linux.rs lines 1148-1387, windows.rs lines 941-1204, and types.rs | Restrictions: Must compile to match statements (no HashMap), must preserve all existing keycodes and aliases, no runtime overhead | Success: cargo build succeeds, all existing keycode tests pass, evdev/vk roundtrips verified | Instructions: Mark task [-] in tasks.md before starting, use log-implementation tool after completion with artifacts, mark [x] when done_

- [x] 2. Create KeyInjector trait
  - File: `core/src/drivers/injector.rs` (new)
  - Define `KeyInjector` trait with `inject()` and `sync()` methods
  - Create `MockKeyInjector` implementation for testing
  - Add unit tests for mock
  - Purpose: Enable dependency injection for key output
  - _Leverage: InputSource trait pattern from traits/mod.rs_
  - _Requirements: REQ-4_
  - _Prompt: Implement the task for spec code-quality-refactor, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer specializing in trait design and DI | Task: Create KeyInjector trait in core/src/drivers/injector.rs following InputSource pattern, with MockKeyInjector for testing | Restrictions: Trait must be Send, must not require async, keep interface minimal | Success: MockKeyInjector captures all injections, tests pass | Instructions: Mark task [-] in tasks.md before starting, use log-implementation tool after completion with artifacts, mark [x] when done_

- [x] 3. Extract shared utilities
  - File: `core/src/drivers/common.rs` (extend)
  - Add `extract_panic_message()` function
  - File: `core/src/scripting/helpers.rs` (new)
  - Add `parse_key_or_error()` function
  - Add unit tests for both
  - Purpose: DRY - eliminate duplicated patterns
  - _Leverage: existing panic handling in linux.rs:841-854, windows.rs:847-854_
  - _Requirements: REQ-5_
  - _Prompt: Implement the task for spec code-quality-refactor, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Extract extract_panic_message() to common.rs and parse_key_or_error() to scripting/helpers.rs, with unit tests | Restrictions: Must be exact behavioral match to existing code | Success: Functions extracted, tests pass, ready for driver refactoring | Instructions: Mark task [-] in tasks.md before starting, use log-implementation tool after completion with artifacts, mark [x] when done_

## Phase 2: Linux Driver Refactor

- [x] 4. Split Linux driver into submodules
  - Create directory: `core/src/drivers/linux/`
  - File: `linux/mod.rs` - LinuxInput struct, InputSource impl
  - File: `linux/reader.rs` - EvdevReader
  - File: `linux/writer.rs` - UinputWriter
  - File: `linux/keymap.rs` - re-export from keycodes.rs
  - Move code preserving all functionality
  - Purpose: File size compliance (<500 lines each)
  - _Leverage: existing linux.rs structure_
  - _Requirements: REQ-2, REQ-6_
  - _Prompt: Implement the task for spec code-quality-refactor, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer specializing in module organization | Task: Split core/src/drivers/linux.rs (1600 lines) into linux/mod.rs, linux/reader.rs, linux/writer.rs, linux/keymap.rs with each file <500 lines | Restrictions: Preserve all public API, no behavioral changes, maintain backward compatibility of imports | Success: cargo build succeeds, all linux driver tests pass, each file <500 lines | Instructions: Mark task [-] in tasks.md before starting, use log-implementation tool after completion with artifacts, mark [x] when done_

- [x] 5. Refactor EvdevReader::spawn() for SLAP compliance
  - File: `core/src/drivers/linux/reader.rs`
  - Extract `run_loop()` method (~30 lines)
  - Extract `process_events()` method (~20 lines)
  - Extract `handle_thread_exit()` method (~20 lines)
  - Use `extract_panic_message()` from common.rs
  - Purpose: Function size compliance (<50 lines), SLAP
  - _Leverage: extracted panic utility from task 3_
  - _Requirements: REQ-3, REQ-5_
  - _Prompt: Implement the task for spec code-quality-refactor, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Refactor EvdevReader::spawn() (currently 140 lines) into spawn() + run_loop() + process_events() + handle_thread_exit(), each <50 lines, using extract_panic_message() | Restrictions: Identical behavior, no new features | Success: spawn() is <50 lines, all tests pass | Instructions: Mark task [-] in tasks.md before starting, use log-implementation tool after completion with artifacts, mark [x] when done_

- [x] 6. Implement KeyInjector for UinputWriter
  - File: `core/src/drivers/linux/writer.rs`
  - Implement `KeyInjector` trait for `UinputWriter`
  - Update `LinuxInput` to use generic `KeyInjector`
  - Add `new_with_injector()` constructor
  - Add tests with `MockKeyInjector`
  - Purpose: Testability improvement
  - _Leverage: KeyInjector trait from task 2_
  - _Requirements: REQ-4_
  - _Prompt: Implement the task for spec code-quality-refactor, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Implement KeyInjector trait for UinputWriter, add LinuxInput::new_with_injector() constructor, write tests using MockKeyInjector | Restrictions: Preserve existing new() behavior as default | Success: Tests pass with mock injector, no hardware needed for unit tests | Instructions: Mark task [-] in tasks.md before starting, use log-implementation tool after completion with artifacts, mark [x] when done_

- [x] 7. Update Linux keymap to use keycodes.rs
  - File: `core/src/drivers/linux/keymap.rs`
  - Remove manual `evdev_to_keycode()` / `keycode_to_evdev()` implementations
  - Re-export from `keycodes.rs`
  - Remove `build_key_set()`, use `all_keycodes()`
  - Update imports in writer.rs
  - Purpose: SSOT compliance
  - _Leverage: keycodes.rs from task 1_
  - _Requirements: REQ-1_
  - _Prompt: Implement the task for spec code-quality-refactor, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Replace linux/keymap.rs manual implementations with re-exports from keycodes.rs, update writer.rs to use all_keycodes() | Restrictions: All existing keycode conversions must work identically | Success: Keymap tests pass, build_key_set removed, SSOT achieved | Instructions: Mark task [-] in tasks.md before starting, use log-implementation tool after completion with artifacts, mark [x] when done_

## Phase 3: Windows Driver Refactor

- [x] 8. Split Windows driver into submodules
  - Create directory: `core/src/drivers/windows/`
  - File: `windows/mod.rs` - WindowsInput struct, InputSource impl
  - File: `windows/hook.rs` - HookManager, low_level_keyboard_proc
  - File: `windows/injector.rs` - SendInputInjector
  - File: `windows/keymap.rs` - re-export from keycodes.rs
  - Move code preserving all functionality
  - Purpose: File size compliance (<500 lines each)
  - _Leverage: existing windows.rs structure_
  - _Requirements: REQ-2, REQ-6_
  - _Prompt: Implement the task for spec code-quality-refactor, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer specializing in Windows API | Task: Split core/src/drivers/windows.rs (1743 lines) into windows/mod.rs, windows/hook.rs, windows/injector.rs, windows/keymap.rs with each file <500 lines | Restrictions: Preserve thread-local storage patterns, maintain all public API | Success: cargo build succeeds on Windows target, each file <500 lines | Instructions: Mark task [-] in tasks.md before starting, use log-implementation tool after completion with artifacts, mark [x] when done_

- [x] 9. Implement KeyInjector for SendInputInjector
  - File: `core/src/drivers/windows/injector.rs`
  - Implement `KeyInjector` trait for `SendInputInjector`
  - Update `WindowsInput` to use generic `KeyInjector`
  - Add `new_with_injector()` constructor
  - Add tests with `MockKeyInjector`
  - Purpose: Testability improvement
  - _Leverage: KeyInjector trait from task 2_
  - _Requirements: REQ-4_
  - _Prompt: Implement the task for spec code-quality-refactor, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Implement KeyInjector trait for SendInputInjector, add WindowsInput::new_with_injector() constructor, write tests using MockKeyInjector | Restrictions: Preserve existing new() behavior | Success: Tests pass with mock injector | Instructions: Mark task [-] in tasks.md before starting, use log-implementation tool after completion with artifacts, mark [x] when done_

- [x] 10. Update Windows keymap to use keycodes.rs
  - File: `core/src/drivers/windows/keymap.rs`
  - Remove manual `vk_to_keycode()` / `keycode_to_vk()` implementations
  - Re-export from `keycodes.rs`
  - Update imports in injector.rs and hook.rs
  - Purpose: SSOT compliance
  - _Leverage: keycodes.rs from task 1_
  - _Requirements: REQ-1_
  - _Prompt: Implement the task for spec code-quality-refactor, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Replace windows/keymap.rs manual implementations with re-exports from keycodes.rs | Restrictions: All existing VK conversions must work identically | Success: Keymap tests pass, SSOT achieved | Instructions: Mark task [-] in tasks.md before starting, use log-implementation tool after completion with artifacts, mark [x] when done_

## Phase 4: Scripting & Types Cleanup

- [x] 11. Update RhaiRuntime to use parse_key_or_error()
  - File: `core/src/scripting/runtime.rs`
  - Replace inline key parsing in `remap()`, `block()`, `pass()` closures
  - Use `parse_key_or_error()` from helpers.rs
  - Reduce code duplication
  - Purpose: DRY compliance
  - _Leverage: helpers.rs from task 3_
  - _Requirements: REQ-5_
  - _Prompt: Implement the task for spec code-quality-refactor, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Refactor runtime.rs lines 56-153 to use parse_key_or_error() helper, eliminating duplicate key parsing code | Restrictions: Identical error messages and behavior | Success: All Rhai script tests pass, code is DRY | Instructions: Mark task [-] in tasks.md before starting, use log-implementation tool after completion with artifacts, mark [x] when done_

- [x] 12. Update types.rs to use keycodes.rs
  - File: `core/src/engine/types.rs`
  - Remove `KeyCode` enum definition (now in keycodes.rs)
  - Remove `Display` impl (generated by macro)
  - Remove `FromStr` impl (generated by macro)
  - Re-export `KeyCode` from keycodes.rs
  - Keep `InputEvent`, `OutputAction`, `RemapAction` in place
  - Purpose: SSOT compliance, file size reduction
  - _Leverage: keycodes.rs from task 1_
  - _Requirements: REQ-1, REQ-2_
  - _Prompt: Implement the task for spec code-quality-refactor, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Remove KeyCode enum, Display, FromStr from types.rs, re-export from keycodes.rs, reduce file from 627 to <400 lines | Restrictions: All existing imports must continue to work | Success: types.rs <500 lines, all tests pass, public API unchanged | Instructions: Mark task [-] in tasks.md before starting, use log-implementation tool after completion with artifacts, mark [x] when done_

## Phase 5: Verification & Cleanup

- [x] 13. Run full test suite and benchmarks
  - Run `cargo test --all-features`
  - Run `cargo bench`
  - Compare benchmark results to baseline
  - Fix any regressions
  - Purpose: Verify zero performance regression
  - _Requirements: REQ-7_
  - _Prompt: Implement the task for spec code-quality-refactor, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer | Task: Run full test suite and benchmarks, compare to pre-refactor baseline, document any differences | Restrictions: No regressions allowed >100 microseconds | Success: All tests pass, benchmarks within threshold | Instructions: Mark task [-] in tasks.md before starting, use log-implementation tool after completion with artifacts, mark [x] when done_

- [x] 14. Update module exports and documentation
  - File: `core/src/drivers/mod.rs`
  - Update re-exports for new module structure
  - File: `core/src/lib.rs`
  - Verify public API unchanged
  - Update any broken doc comments
  - Purpose: Maintain backward compatibility
  - _Requirements: REQ-2, REQ-6_
  - _Prompt: Implement the task for spec code-quality-refactor, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Update drivers/mod.rs and lib.rs exports for new module structure, verify rustdoc builds without warnings | Restrictions: Public API must be identical | Success: cargo doc succeeds, existing examples compile | Instructions: Mark task [-] in tasks.md before starting, use log-implementation tool after completion with artifacts, mark [x] when done_

- [x] 15. Verify code metrics compliance
  - Run line count on all modified files
  - Verify each file <500 lines (excluding comments/blanks)
  - Verify each function <50 lines
  - Document final metrics
  - Purpose: Confirm KPI compliance
  - _Requirements: REQ-2, REQ-3_
  - _Prompt: Implement the task for spec code-quality-refactor, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Code Quality Engineer | Task: Run tokei or cloc on all driver files, verify <500 lines/file and <50 lines/function, document in implementation log | Restrictions: Must meet all metrics | Success: All files and functions within limits, metrics documented | Instructions: Mark task [-] in tasks.md before starting, use log-implementation tool after completion with artifacts, mark [x] when done_

- [ ] 16. Delete old monolithic driver files
  - Delete `core/src/drivers/linux.rs` (replaced by linux/)
  - Delete `core/src/drivers/windows.rs` (replaced by windows/)
  - Final `cargo build` and `cargo test`
  - Purpose: Clean up legacy code
  - _Requirements: REQ-2_
  - _Prompt: Implement the task for spec code-quality-refactor, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Developer | Task: Delete old linux.rs and windows.rs files, run final build and test to confirm nothing is broken | Restrictions: Only delete after all tests pass | Success: Old files removed, cargo build and cargo test pass | Instructions: Mark task [-] in tasks.md before starting, use log-implementation tool after completion with artifacts, mark [x] when done_
