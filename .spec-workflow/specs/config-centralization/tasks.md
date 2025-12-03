# Tasks Document: Configuration Centralization

## Phase 1: Rust Config Module Foundation

- [x] 1. Create config module structure
  - Files: `core/src/config/mod.rs`
  - Create module directory and root mod.rs with re-exports
  - Purpose: Establish config module foundation
  - _Requirements: 1_
  - _Prompt: Implement the task for spec config-centralization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer specializing in module architecture | Task: Create `core/src/config/mod.rs` that establishes the config module with re-exports for timing, keys, paths, limits, exit_codes, and loader submodules. Add the module to lib.rs exports. | Restrictions: Do not create the submodule files yet (just the mod.rs), follow existing module patterns in the codebase, use pub use for re-exports | _Leverage: core/src/lib.rs, core/src/engine/mod.rs for module patterns_ | Success: Module compiles, is exported from lib.rs, submodule declarations exist (will error until files created). Mark task [-] in tasks.md before starting, log implementation with log-implementation tool, then mark [x] when complete._

- [x] 2. Create timing constants module
  - File: `core/src/config/timing.rs`
  - Extract timing constants from engine/decision/timing.rs
  - Define DEFAULT_TAP_TIMEOUT_MS, DEFAULT_COMBO_TIMEOUT_MS, DEFAULT_HOLD_DELAY_MS, MICROS_PER_MS
  - Purpose: Centralize all timing-related constants
  - _Leverage: core/src/engine/decision/timing.rs_
  - _Requirements: 1.1_
  - _Prompt: Implement the task for spec config-centralization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Create `core/src/config/timing.rs` with timing constants extracted from engine/decision/timing.rs. Include DEFAULT_TAP_TIMEOUT_MS=200, DEFAULT_COMBO_TIMEOUT_MS=50, DEFAULT_HOLD_DELAY_MS=0, MICROS_PER_MS=1000. Add doc comments for each constant explaining purpose and valid range. | Restrictions: Keep existing TimingConfig struct in engine/decision/timing.rs but have it use these constants, do not break existing code | _Leverage: core/src/engine/decision/timing.rs:23-25, core/src/engine/decision/pending.rs:5_ | Success: Constants defined, documented, module compiles. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool, then mark [x] when complete._

- [x] 3. Create key codes constants module
  - File: `core/src/config/keys.rs`
  - Extract evdev and Windows VK codes
  - Define EVDEV_* and VK_* constants
  - Purpose: Centralize all key code constants
  - _Leverage: core/src/drivers/linux/reader.rs, core/src/drivers/windows/hook.rs_
  - _Requirements: 1.2_
  - _Prompt: Implement the task for spec config-centralization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Create `core/src/config/keys.rs` with key code constants for both Linux evdev (EVDEV_KEY_ESC=1, EVDEV_KEY_LEFTCTRL=29, EVDEV_KEY_LEFTSHIFT=42, EVDEV_KEY_LEFTALT=56, EVDEV_KEY_RIGHTCTRL=97, EVDEV_KEY_RIGHTSHIFT=54, EVDEV_KEY_RIGHTALT=100) and Windows (VK_ESCAPE=0x1B, VK_CONTROL=0x11, VK_SHIFT=0x10, VK_MENU=0x12). Group by platform with doc comments. | Restrictions: Use cfg attributes for platform-specific constants if appropriate, maintain consistency with existing code | _Leverage: core/src/drivers/linux/reader.rs:21-27, core/src/drivers/windows/hook.rs:28-31_ | Success: All key codes extracted and documented. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool, then mark [x] when complete._

- [x] 4. Create paths constants module
  - File: `core/src/config/paths.rs`
  - Extract path constants (uinput, config directories, file names)
  - Purpose: Centralize all path-related constants
  - _Leverage: core/src/drivers/linux/mod.rs, core/src/cli/commands/repl.rs_
  - _Requirements: 1.3_
  - _Prompt: Implement the task for spec config-centralization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Create `core/src/config/paths.rs` with path constants: UINPUT_PATH="/dev/uinput", UINPUT_DEVICE_NAME="KeyRx Virtual Keyboard", REPL_HISTORY_FILE=".keyrx_repl_history", PERF_BASELINE_FILE="target/perf-baseline.json", CONFIG_FILE_NAME="config.toml", SCRIPTS_DIR="scripts". Include helper function for XDG config path resolution. | Restrictions: Reuse existing device_profiles_dir() pattern from discovery/types.rs | _Leverage: core/src/drivers/linux/mod.rs:25, core/src/drivers/linux/writer.rs:17, core/src/cli/commands/repl.rs:12, core/src/uat/perf.rs:27, core/src/discovery/types.rs_ | Success: All path constants defined with doc comments. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool, then mark [x] when complete._

- [x] 5. Create limits constants module
  - File: `core/src/config/limits.rs`
  - Extract capacity and threshold limits
  - Purpose: Centralize all limit-related constants
  - _Leverage: core/src/engine/decision/pending.rs, core/src/engine/state/modifiers.rs_
  - _Requirements: 1.4_
  - _Prompt: Implement the task for spec config-centralization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Create `core/src/config/limits.rs` with capacity constants: MAX_PENDING_DECISIONS=32, MIN_COMBO_KEYS=2, MAX_COMBO_KEYS=4, MAX_MODIFIER_ID=255, MAX_TIMEOUT_MS=5000, DEFAULT_EVENT_GAP_US=1000, LATENCY_THRESHOLD_NS=1_000_000, DEFAULT_REGRESSION_THRESHOLD_US=100. Add doc comments explaining each limit's purpose. | Restrictions: Ensure values exactly match current hardcoded values | _Leverage: core/src/engine/decision/pending.rs:86,144, core/src/engine/state/modifiers.rs:68, core/src/scripting/builtins.rs:236_ | Success: All limits extracted with documentation. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool, then mark [x] when complete._

- [x] 6. Create exit codes constants module
  - File: `core/src/config/exit_codes.rs`
  - Consolidate CLI exit codes into single module
  - Purpose: Centralize all exit code constants for CLI commands
  - _Leverage: core/src/cli/commands/*.rs_
  - _Requirements: 1.5_
  - _Prompt: Implement the task for spec config-centralization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Create `core/src/config/exit_codes.rs` consolidating all CLI exit codes: SUCCESS=0, ERROR=1, VERIFICATION_FAILED=2, TIMEOUT=3, ASSERTION_FAIL=2, TEST_FAIL=1, GATE_FAIL=2, CRASH=3, REGRESSION=2, CONFIRMATION_REQUIRED=3. Use an enum ExitCode with i32 values. Add doc comments explaining when each code is used. | Restrictions: Maintain backward compatibility with existing exit code semantics | _Leverage: core/src/cli/commands/test.rs:18-21, core/src/cli/commands/replay.rs:304-308, core/src/cli/commands/uat.rs:15-18_ | Success: Unified ExitCode enum with all codes. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool, then mark [x] when complete._

## Phase 2: Refactor Rust Code to Use Config Module

- [x] 7. Refactor timing.rs to use config constants
  - File: `core/src/engine/decision/timing.rs`
  - Import and use constants from config module
  - Purpose: Use centralized timing constants
  - _Leverage: core/src/config/timing.rs_
  - _Requirements: 1.6_
  - _Prompt: Implement the task for spec config-centralization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Refactor `core/src/engine/decision/timing.rs` to import DEFAULT_TAP_TIMEOUT_MS, DEFAULT_COMBO_TIMEOUT_MS, DEFAULT_HOLD_DELAY_MS from crate::config::timing and use them in TimingConfig::default(). Also update pending.rs to use MICROS_PER_MS from config. | Restrictions: Do not change behavior, only source of constants | _Leverage: core/src/config/timing.rs, core/src/engine/decision/pending.rs_ | Success: TimingConfig::default() uses config constants, tests pass. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool, then mark [x] when complete._

- [x] 8. Refactor Linux driver to use key/path constants
  - Files: `core/src/drivers/linux/reader.rs`, `core/src/drivers/linux/mod.rs`, `core/src/drivers/linux/writer.rs`
  - Import and use constants from config module
  - Purpose: Use centralized key codes and paths
  - _Leverage: core/src/config/keys.rs, core/src/config/paths.rs_
  - _Requirements: 1.2, 1.3_
  - _Prompt: Implement the task for spec config-centralization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Refactor Linux driver files to use config constants. In reader.rs: replace hardcoded evdev codes with EVDEV_KEY_* from crate::config::keys. In mod.rs: use UINPUT_PATH from crate::config::paths. In writer.rs: use UINPUT_DEVICE_NAME from config. | Restrictions: Do not change behavior, maintain #[cfg(target_os = "linux")] guards | _Leverage: core/src/config/keys.rs, core/src/config/paths.rs_ | Success: All hardcoded key codes and paths replaced, tests pass. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool, then mark [x] when complete._

- [x] 9. Refactor Windows driver to use key constants
  - File: `core/src/drivers/windows/hook.rs`
  - Import and use VK_* constants from config module
  - Purpose: Use centralized Windows key codes
  - _Leverage: core/src/config/keys.rs_
  - _Requirements: 1.2_
  - _Prompt: Implement the task for spec config-centralization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Refactor `core/src/drivers/windows/hook.rs` to use VK_ESCAPE, VK_CONTROL, VK_SHIFT, VK_MENU from crate::config::keys instead of hardcoded hex values (0x1B, 0x11, 0x10, 0x12). | Restrictions: Maintain #[cfg(target_os = "windows")] guards, do not change behavior | _Leverage: core/src/config/keys.rs_ | Success: All hardcoded VK codes replaced. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool, then mark [x] when complete._

- [x] 10. Refactor pending.rs to use limit constants
  - File: `core/src/engine/decision/pending.rs`
  - Import and use MAX_PENDING_DECISIONS, combo key limits from config
  - Purpose: Use centralized limit constants
  - _Leverage: core/src/config/limits.rs_
  - _Requirements: 1.4_
  - _Prompt: Implement the task for spec config-centralization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Refactor `core/src/engine/decision/pending.rs` to use MAX_PENDING_DECISIONS, MIN_COMBO_KEYS, MAX_COMBO_KEYS from crate::config::limits. Replace DecisionQueue::MAX_PENDING constant with the config value. | Restrictions: Ensure validation logic using these limits works identically | _Leverage: core/src/config/limits.rs_ | Success: Limit constants centralized, tests pass. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool, then mark [x] when complete._

- [x] 11. Refactor CLI commands to use exit codes
  - Files: `core/src/cli/commands/test.rs`, `replay.rs`, `golden.rs`, `uat.rs`, `regression.rs`, `ci_check_summary.rs`
  - Import and use ExitCode enum from config
  - Purpose: Use centralized exit codes
  - _Leverage: core/src/config/exit_codes.rs_
  - _Requirements: 1.5_
  - _Prompt: Implement the task for spec config-centralization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Refactor all CLI command files to use ExitCode enum from crate::config::exit_codes instead of local EXIT_* constants. Files: test.rs, replay.rs, golden.rs, uat.rs, regression.rs, ci_check/ci_check_summary.rs. Remove duplicated constant definitions. | Restrictions: Maintain exact same exit code semantics | _Leverage: core/src/config/exit_codes.rs_ | Success: All CLI commands use centralized ExitCode, no duplicate definitions. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool, then mark [x] when complete._

- [x] 12. Refactor remaining Rust files to use config constants
  - Files: `modifiers.rs`, `builtins.rs`, `repl.rs`, `simulate.rs`, `bench.rs`, `perf.rs`
  - Replace remaining hardcoded values
  - Purpose: Complete Rust config centralization
  - _Leverage: core/src/config/*_
  - _Requirements: 1.6_
  - _Prompt: Implement the task for spec config-centralization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Refactor remaining files with magic numbers: modifiers.rs (MAX_ID=255), builtins.rs (MAX_TIMEOUT_MS=5000), repl.rs (HISTORY_FILE), simulate.rs (DEFAULT_EVENT_GAP_US=1000), bench.rs (LATENCY_THRESHOLD_NS), perf.rs (DEFAULT_REGRESSION_THRESHOLD_US, BASELINE_FILE). Import from crate::config. | Restrictions: Verify tests still pass after each file change | _Leverage: core/src/config/limits.rs, core/src/config/paths.rs_ | Success: All identified magic numbers in Rust codebase use config module. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool, then mark [x] when complete._

## Phase 3: TOML Config Loader

- [x] 13. Create config loader module
  - File: `core/src/config/loader.rs`
  - Implement TOML loading with validation and defaults
  - Purpose: Enable runtime configuration via file
  - _Leverage: .keyrx/quality-gates.toml loading pattern_
  - _Requirements: 3_
  - _Prompt: Implement the task for spec config-centralization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Create `core/src/config/loader.rs` with: Config struct (timing, ui, performance, paths sections), load_config(path: Option<&Path>) -> Config function that loads TOML with serde, validate_config() to check ranges, merge_cli_overrides() to apply CLI args. Use dirs crate for XDG path resolution. Return defaults if file not found. | Restrictions: Log warnings for invalid values but don't crash, clamp values to valid ranges | _Leverage: .keyrx/quality-gates.toml, toml crate, dirs crate_ | Success: Config loads from file, falls back to defaults, validates ranges. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool, then mark [x] when complete._

- [x] 14. Create default config.toml template
  - File: `.keyrx/config.toml.example`
  - Document all configurable options
  - Purpose: Provide user documentation for config file
  - _Requirements: 4_
  - _Prompt: Implement the task for spec config-centralization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical Writer | Task: Create `.keyrx/config.toml.example` with all configurable options documented. Include sections: [timing] (tap_timeout_ms, combo_timeout_ms, hold_delay_ms), [ui] (max_events_history, animation_duration_ms), [performance] (latency_warning_us, latency_caution_us, regression_threshold_us), [paths] (scripts_dir, temp_dir). Add inline comments explaining each option with valid ranges. | Restrictions: Use default values in example, ensure TOML is valid | Success: Well-documented config template. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool, then mark [x] when complete._

- [x] 15. Integrate config loading into CLI
  - Files: `core/src/bin/keyrx.rs`, `core/src/cli/commands/run.rs`
  - Add --config flag and load config at startup
  - Purpose: Enable config file usage in CLI
  - _Leverage: core/src/config/loader.rs_
  - _Requirements: 3.6_
  - _Prompt: Implement the task for spec config-centralization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Add --config <path> argument to clap in keyrx.rs. Load config at startup using load_config(). Pass config to relevant commands (run, simulate, bench). Add --tap-timeout, --combo-timeout CLI overrides that call merge_cli_overrides(). | Restrictions: Maintain backward compatibility - no config file = same behavior | _Leverage: core/src/bin/keyrx.rs, core/src/config/loader.rs_ | Success: keyrx run --config path/to/config.toml works, CLI overrides work. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool, then mark [x] when complete._

## Phase 4: Dart Config Module

- [x] 16. Create Dart config module structure
  - File: `ui/lib/config/config.dart`
  - Create barrel export for all config modules
  - Purpose: Establish Dart config module foundation
  - _Requirements: 2.5_
  - _Prompt: Implement the task for spec config-centralization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer | Task: Create `ui/lib/config/config.dart` as barrel export that will export timing_config.dart, ui_constants.dart, storage_keys.dart, ffi_constants.dart. Create the directory structure. | Restrictions: Follow existing export patterns in codebase | _Leverage: ui/lib/ffi/bindings.dart for export pattern_ | Success: Config module structure created. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool, then mark [x] when complete._

- [x] 17. Create Dart timing config
  - File: `ui/lib/config/timing_config.dart`
  - Extract animation and debounce timing constants
  - Purpose: Centralize UI timing constants
  - _Leverage: ui/lib/pages/debugger_page.dart, ui/lib/widgets/visual_keyboard_keys.dart_
  - _Requirements: 2.1_
  - _Prompt: Implement the task for spec config-centralization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer | Task: Create `ui/lib/config/timing_config.dart` with abstract class TimingConfig containing: animationDurationMs=150, pulseAnimationMs=300, debounceMs=500, keyAnimationMs=100, typingTimeLimitSec=30. Use static const for all values. Add doc comments. | Restrictions: Use abstract class to prevent instantiation | _Leverage: ui/lib/pages/debugger_page.dart:29,51,73_ | Success: All UI timing constants centralized. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool, then mark [x] when complete._

- [x] 18. Create Dart UI constants
  - File: `ui/lib/config/ui_constants.dart`
  - Extract padding, elevation, scale constants
  - Purpose: Centralize UI dimension constants
  - _Leverage: ui/lib/ui/styles/surfaces.dart, ui/lib/widgets/visual_keyboard.dart_
  - _Requirements: 2.2_
  - _Prompt: Implement the task for spec config-centralization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer | Task: Create `ui/lib/config/ui_constants.dart` with abstract class UiConstants containing: defaultPadding=16.0, smallPadding=8.0, tinyPadding=4.0, defaultElevation=6.0, minKeyboardScale=0.5, maxKeyboardScale=1.0, defaultIconSize=24.0, defaultBorderRadius=4.0. Add doc comments. | Restrictions: Use abstract class, maintain double types for dimension values | _Leverage: ui/lib/ui/styles/surfaces.dart:19, ui/lib/widgets/visual_keyboard.dart:146-148_ | Success: All UI dimension constants centralized. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool, then mark [x] when complete._

- [x] 19. Create Dart storage keys
  - File: `ui/lib/config/storage_keys.dart`
  - Extract SharedPreferences key constants
  - Purpose: Centralize storage key strings
  - _Leverage: ui/lib/state/app_state.dart, ui/lib/pages/keyrx_training_screen.dart_
  - _Requirements: 2.3_
  - _Prompt: Implement the task for spec config-centralization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer | Task: Create `ui/lib/config/storage_keys.dart` with abstract class StorageKeys containing: developerModeKey="developer_mode", trainingProgressKey="keyrx_training_progress". Add doc comments explaining what each key stores. | Restrictions: Use abstract class, ensure exact string matches existing values | _Leverage: ui/lib/state/app_state.dart:13, ui/lib/pages/keyrx_training_screen.dart:26_ | Success: All SharedPreferences keys centralized. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool, then mark [x] when complete._

- [x] 20. Create Dart FFI constants
  - File: `ui/lib/config/ffi_constants.dart`
  - Extract FFI function names and JSON response keys
  - Purpose: Centralize FFI-related strings
  - _Leverage: ui/lib/ffi/bindings.dart, ui/lib/ffi/bridge_*.dart_
  - _Requirements: 2.4_
  - _Prompt: Implement the task for spec config-centralization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer | Task: Create `ui/lib/config/ffi_constants.dart` with: abstract class FfiFunctions (all keyrx_* function names), abstract class JsonKeys (response keys like "success", "error", "totalKeys", "avgLatencyUs", etc.), abstract class ResponsePrefixes ("ok:", "error:"). Extract from bindings.dart and bridge_*.dart files. | Restrictions: Use abstract classes, organize by category | _Leverage: ui/lib/ffi/bindings.dart:189-439, ui/lib/ffi/bridge_engine.dart, ui/lib/ffi/bridge_session.dart_ | Success: All FFI strings centralized. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool, then mark [x] when complete._

- [x] 21. Create Dart threshold constants
  - File: `ui/lib/config/threshold_constants.dart`
  - Extract performance threshold constants
  - Purpose: Centralize threshold values
  - _Leverage: ui/lib/pages/debugger_meters.dart, ui/lib/pages/developer/benchmark_page.dart_
  - _Requirements: 2.1_
  - _Prompt: Implement the task for spec config-centralization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer | Task: Create `ui/lib/config/threshold_constants.dart` with abstract class ThresholdConstants containing: latencyWarningUs=20000, latencyCautionUs=10000, warningThresholdNs=1000000, minKeystrokes=10, pauseThresholdMs=2000, maxEventsHistory=300. Add doc comments with units. Update config.dart barrel export. | Restrictions: Use abstract class, maintain int types | _Leverage: ui/lib/pages/debugger_meters.dart:14-15, ui/lib/pages/developer/benchmark_page.dart_ | Success: All threshold constants centralized. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool, then mark [x] when complete._

- [x] 22. Create Dart path constants
  - File: `ui/lib/config/path_constants.dart`
  - Extract file path constants
  - Purpose: Centralize path strings
  - _Leverage: ui/lib/pages/editor_page.dart, ui/lib/pages/visual_editor_page.dart_
  - _Requirements: 2.3_
  - _Prompt: Implement the task for spec config-centralization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer | Task: Create `ui/lib/config/path_constants.dart` with abstract class PathConstants containing: defaultScriptPath="scripts/generated.rhai", tempValidationPath="/tmp/keyrx_validation.rhai", scriptsDir="scripts/", defaultConfigFileName="config.rhai". Add doc comments. Update config.dart barrel export. | Restrictions: Use abstract class | _Leverage: ui/lib/pages/editor_page.dart:66-67, ui/lib/pages/visual_editor_page.dart:40,311_ | Success: All path constants centralized. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool, then mark [x] when complete._

## Phase 5: Refactor Dart Code to Use Config Module

- [x] 23. Refactor debugger_page.dart to use config
  - File: `ui/lib/pages/debugger_page.dart`
  - Import and use timing/threshold constants
  - Purpose: Use centralized constants
  - _Leverage: ui/lib/config/timing_config.dart, ui/lib/config/threshold_constants.dart_
  - _Requirements: 2.1_
  - _Prompt: Implement the task for spec config-centralization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer | Task: Refactor `ui/lib/pages/debugger_page.dart` to import from 'package:keyrx/config/config.dart' and use TimingConfig.animationDurationMs, TimingConfig.pulseAnimationMs, TimingConfig.debounceMs, ThresholdConstants.maxEventsHistory instead of hardcoded values. | Restrictions: Do not change behavior | _Leverage: ui/lib/config/config.dart_ | Success: No hardcoded timing/threshold values in debugger_page.dart. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool, then mark [x] when complete._

- [x] 24. Refactor debugger_meters.dart to use config
  - File: `ui/lib/pages/debugger_meters.dart`
  - Import and use threshold constants
  - Purpose: Use centralized constants
  - _Leverage: ui/lib/config/threshold_constants.dart, ui/lib/config/timing_config.dart_
  - _Requirements: 2.1_
  - _Prompt: Implement the task for spec config-centralization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer | Task: Refactor `ui/lib/pages/debugger_meters.dart` to use ThresholdConstants.latencyWarningUs, ThresholdConstants.latencyCautionUs, TimingConfig.animationDurationMs from config module instead of hardcoded values in LatencyThresholds class. | Restrictions: Do not change behavior | _Leverage: ui/lib/config/config.dart_ | Success: No hardcoded threshold values. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool, then mark [x] when complete._

- [x] 25. Refactor visual_keyboard.dart and visual_keyboard_keys.dart to use config
  - Files: `ui/lib/widgets/visual_keyboard.dart`, `ui/lib/widgets/visual_keyboard_keys.dart`
  - Import and use UI/timing constants
  - Purpose: Use centralized constants
  - _Leverage: ui/lib/config/ui_constants.dart, ui/lib/config/timing_config.dart_
  - _Requirements: 2.2_
  - _Prompt: Implement the task for spec config-centralization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer | Task: Refactor visual_keyboard.dart to use UiConstants.minKeyboardScale, UiConstants.maxKeyboardScale. Refactor visual_keyboard_keys.dart to use TimingConfig.keyAnimationMs instead of hardcoded 100ms. | Restrictions: Do not change behavior | _Leverage: ui/lib/config/config.dart_ | Success: Scale bounds and animation duration from config. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool, then mark [x] when complete._

- [x] 26. Refactor editor_page.dart to use config
  - File: `ui/lib/pages/editor_page.dart`
  - Import and use path/UI constants
  - Purpose: Use centralized constants
  - _Leverage: ui/lib/config/path_constants.dart, ui/lib/config/ui_constants.dart_
  - _Requirements: 2.2, 2.3_
  - _Prompt: Implement the task for spec config-centralization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer | Task: Refactor `ui/lib/pages/editor_page.dart` to use PathConstants.defaultScriptPath, PathConstants.tempValidationPath, UiConstants.defaultPadding instead of hardcoded values. | Restrictions: Do not change behavior | _Leverage: ui/lib/config/config.dart_ | Success: No hardcoded paths or padding values. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool, then mark [x] when complete._

- [x] 27. Refactor app_state.dart to use config
  - File: `ui/lib/state/app_state.dart`
  - Import and use storage key constants
  - Purpose: Use centralized constants
  - _Leverage: ui/lib/config/storage_keys.dart_
  - _Requirements: 2.3_
  - _Prompt: Implement the task for spec config-centralization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer | Task: Refactor `ui/lib/state/app_state.dart` to use StorageKeys.developerModeKey instead of hardcoded "_kDeveloperModeKey" constant. Import from config module. | Restrictions: Do not change behavior | _Leverage: ui/lib/config/config.dart_ | Success: SharedPreferences key from config. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool, then mark [x] when complete._

- [x] 28. Refactor remaining Dart files to use config
  - Files: `surfaces.dart`, `trade_off_*.dart`, `typing_simulator.dart`, FFI bridge files
  - Replace remaining hardcoded values
  - Purpose: Complete Dart config centralization
  - _Leverage: ui/lib/config/*_
  - _Requirements: 2_
  - _Prompt: Implement the task for spec config-centralization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer | Task: Refactor remaining files: surfaces.dart (defaultElevation), trade_off_page.dart/chart.dart/widgets.dart (timeout ranges, padding), typing_simulator.dart (time limits, thresholds), keyrx_training_screen.dart (storage key), FFI bridge files (JSON keys, prefixes). Import from config module. | Restrictions: Verify tests still pass | _Leverage: ui/lib/config/config.dart_ | Success: All identified magic numbers/strings in Dart codebase use config module. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool, then mark [x] when complete._

## Phase 6: Testing and Documentation

- [ ] 29. Add Rust config module tests
  - File: `core/src/config/tests.rs` or inline tests
  - Test config loading, validation, defaults
  - Purpose: Ensure config module reliability
  - _Requirements: NFR - Reliability_
  - _Prompt: Implement the task for spec config-centralization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Add unit tests for config module: test_default_values_match_original() verifying defaults match previous hardcoded values, test_load_missing_file_uses_defaults(), test_load_invalid_toml_uses_defaults(), test_value_range_clamping(), test_cli_override_merging(). | Restrictions: Use #[cfg(test)] module, follow existing test patterns | _Leverage: core/src/config/loader.rs_ | Success: >80% coverage on config module. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool, then mark [x] when complete._

- [ ] 30. Add Dart config module tests
  - File: `ui/test/config/config_test.dart`
  - Test constant values and exports
  - Purpose: Ensure Dart config module correctness
  - _Requirements: NFR - Reliability_
  - _Prompt: Implement the task for spec config-centralization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer | Task: Add unit tests for Dart config module: verify all constants have expected values, verify barrel export includes all modules, verify no runtime errors when importing config. | Restrictions: Follow existing Flutter test patterns | _Leverage: ui/test/ existing tests_ | Success: Config tests pass. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool, then mark [x] when complete._

- [ ] 31. Update documentation
  - Files: README.md, inline doc comments
  - Document new config system
  - Purpose: Help users understand configuration
  - _Requirements: 4_
  - _Prompt: Implement the task for spec config-centralization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical Writer | Task: Add Configuration section to README.md explaining: config file location (~/.config/keyrx/config.toml), available options with defaults, CLI override flags. Ensure all config module doc comments are complete. | Restrictions: Keep concise, link to config.toml.example | Success: Users can find and understand config options. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool, then mark [x] when complete._

- [ ] 32. Final verification and cleanup
  - Run full test suite, verify no regressions
  - Remove any remaining magic numbers/strings
  - Purpose: Ensure complete centralization
  - _Requirements: 5, All_
  - _Prompt: Implement the task for spec config-centralization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Senior Developer | Task: Final verification: run `just check` and `flutter test`, grep codebase for remaining magic numbers (search for common values like 200, 150, 32), verify backward compatibility by testing without config file, clean up any TODO comments from refactoring. | Restrictions: Fix any failing tests, do not introduce new features | _Leverage: justfile, existing test suites_ | Success: All tests pass, no magic numbers remain, backward compatible. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool, then mark [x] when complete._
