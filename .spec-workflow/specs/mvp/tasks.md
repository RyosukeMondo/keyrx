# Tasks Document: MVP

## Task 1: Complete KeyCode Enum and Event Types

- [x] 1. Expand KeyCode enum with full keyboard coverage
  - File: `core/src/engine/types.rs`
  - Add all standard keyboard keys (A-Z, 0-9, F1-F12, modifiers, special keys)
  - Add `RemapAction` enum (Remap, Block, Pass)
  - Ensure `KeyCode` implements `Hash`, `Eq`, `Copy`, `Clone`, `Debug`
  - Purpose: Establish complete type system for key handling
  - _Leverage: Existing `InputEvent`, `OutputAction` in types.rs_
  - _Requirements: REQ-4_
  - _Prompt: Implement the task for spec mvp, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer specializing in type systems | Task: Expand KeyCode enum in core/src/engine/types.rs to cover full keyboard (A-Z, 0-9, F1-F12, modifiers, special keys). Add RemapAction enum with variants Remap(KeyCode), Block, Pass. Ensure all types derive Hash, Eq, Copy, Clone, Debug, Serialize, Deserialize | Restrictions: Keep file under 200 lines, use standard US keyboard layout as reference, do not add mouse or gamepad codes yet | _Leverage: Existing InputEvent, OutputAction structs | _Requirements: REQ-4 (Basic Key Remapping) | Success: All common keyboard keys represented, types compile without errors, can be used as HashMap keys. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

## Task 2: Create RemapRegistry

- [x] 2. Implement RemapRegistry for tracking key mappings
  - File: `core/src/scripting/registry.rs`
  - Create `RemapRegistry` struct with `HashMap<KeyCode, RemapAction>`
  - Implement `remap()`, `block()`, `pass()`, `lookup()`, `clear()` methods
  - Add `Default` impl that returns Pass for unmapped keys
  - Purpose: Central storage for script-defined key behaviors
  - _Leverage: KeyCode, RemapAction from engine/types.rs_
  - _Requirements: REQ-4_
  - _Prompt: Implement the task for spec mvp, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with expertise in data structures | Task: Create RemapRegistry in core/src/scripting/registry.rs. Struct holds HashMap<KeyCode, RemapAction>. Implement: remap(from, to), block(key), pass(key), lookup(key) -> RemapAction, clear(). Default returns Pass for unmapped keys | Restrictions: Keep struct simple, no async, no external deps, max 100 lines | _Leverage: KeyCode, RemapAction from engine/types.rs | _Requirements: REQ-4 | Success: All methods work correctly, lookup returns Pass by default, unit tests pass. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

- [x] 2.1 Add RemapRegistry to scripting module
  - File: `core/src/scripting/mod.rs`
  - Add `mod registry;` and `pub use registry::RemapRegistry;`
  - Purpose: Export registry from scripting module
  - _Leverage: Existing mod.rs structure_
  - _Requirements: REQ-4_
  - _Prompt: Implement the task for spec mvp, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Update core/src/scripting/mod.rs to add mod registry and pub use registry::RemapRegistry | Restrictions: Only add 2 lines, preserve existing exports | _Requirements: REQ-4 | Success: RemapRegistry is importable from keyrx_core::scripting. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

## Task 3: Register Rhai Remap Functions

- [x] 3. Register remap/block/pass functions in RhaiRuntime
  - File: `core/src/scripting/runtime.rs`
  - Add `RemapRegistry` field to `RhaiRuntime`
  - Register `remap(from: &str, to: &str)` that calls `registry.remap()`
  - Register `block(key: &str)` that calls `registry.block()`
  - Register `pass(key: &str)` that calls `registry.pass()`
  - Add `registry()` accessor method
  - Purpose: Enable Rhai scripts to define key behaviors
  - _Leverage: Existing Rhai engine setup, RemapRegistry_
  - _Requirements: REQ-3, REQ-4_
  - _Prompt: Implement the task for spec mvp, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with Rhai experience | Task: Modify core/src/scripting/runtime.rs to add RemapRegistry field. Register functions: remap(from: &str, to: &str), block(key: &str), pass(key: &str). Parse string keys to KeyCode. Add registry() -> &RemapRegistry accessor | Restrictions: Handle invalid key names gracefully (log warning, ignore), keep sandbox limits | _Leverage: Existing RhaiRuntime, RemapRegistry | _Requirements: REQ-3, REQ-4 | Success: Rhai scripts can call remap("A", "B"), block("CapsLock"), pass("Enter"). Registry reflects calls. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

## Task 4: Implement Event Processing Loop

- [x] 4. Add process_event and run_loop to Engine
  - File: `core/src/engine/event_loop.rs`
  - Implement `process_event(&mut self, event: InputEvent) -> Result<OutputAction>`
  - Query ScriptRuntime's registry for remapping decision
  - Implement `run_loop(&mut self) -> Result<()>` that polls InputSource
  - Handle graceful shutdown on stop signal
  - Purpose: Core event processing pipeline
  - _Leverage: Existing Engine struct, InputSource trait, ScriptRuntime trait_
  - _Requirements: REQ-2, REQ-4_
  - _Prompt: Implement the task for spec mvp, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with async experience | Task: Add to Engine in core/src/engine/event_loop.rs: process_event(event: InputEvent) -> Result<OutputAction> that looks up event.key in script runtime's registry and returns appropriate OutputAction. Add run_loop() that polls input source in a loop, processes events, and respects self.running flag | Restrictions: Keep loop simple, no complex buffering, handle both key-down and key-up | _Leverage: Existing Engine, InputSource, ScriptRuntime | _Requirements: REQ-2, REQ-4 | Success: Events flow through pipeline, remappings applied correctly, loop exits cleanly. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

- [x] 4.1 Add registry accessor to ScriptRuntime trait
  - File: `core/src/traits/script_runtime.rs`
  - Add `fn lookup_remap(&self, key: KeyCode) -> RemapAction;` to trait (cleaner than exposing full registry)
  - Update MockRuntime to implement new method
  - Purpose: Allow Engine to query remappings from any ScriptRuntime
  - _Leverage: Existing ScriptRuntime trait_
  - _Requirements: REQ-4_
  - _Prompt: Implement the task for spec mvp, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Add registry() -> &RemapRegistry method to ScriptRuntime trait in core/src/traits/script_runtime.rs. Update MockRuntime in mocks/mock_runtime.rs to implement this method (return empty registry) | Restrictions: Keep trait changes minimal | _Leverage: Existing trait definition | _Requirements: REQ-4 | Success: Trait compiles, MockRuntime compiles, Engine can call script.registry(). Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

## Task 5: Implement Simulate Command

- [x] 5. Create simulate command for headless testing
  - File: `core/src/cli/commands/simulate.rs`
  - Parse `--input "A,B,C"` into sequence of InputEvents
  - Load script with RhaiRuntime
  - Process each event through Engine with MockInput
  - Output results (original key -> output action)
  - Support `--json` flag via OutputWriter
  - Purpose: Enable autonomous testing without real keyboard
  - _Leverage: Engine, MockInput, RhaiRuntime, OutputWriter_
  - _Requirements: REQ-5_
  - _Prompt: Implement the task for spec mvp, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with CLI experience | Task: Create SimulateCommand in core/src/cli/commands/simulate.rs. Parse --input "A,B,C" into InputEvents. Create Engine with MockInput, load script with RhaiRuntime. Process each event, collect OutputActions. Print results using OutputWriter (human: "A -> B", json: structured output) | Restrictions: No real keyboard hooks, use mocks only, handle parse errors gracefully | _Leverage: Engine, MockInput, RhaiRuntime, OutputWriter | _Requirements: REQ-5 | Success: keyrx simulate --input "A" --script test.rhai outputs remapped result. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

- [x] 5.1 Wire simulate command into CLI
  - File: `core/src/bin/keyrx.rs`, `core/src/cli/commands/mod.rs`
  - Add `Simulate` variant to Commands enum
  - Import and call `SimulateCommand`
  - Export SimulateCommand from mod.rs
  - Purpose: Make simulate command accessible via CLI
  - _Leverage: Existing CLI structure_
  - _Requirements: REQ-5_
  - _Prompt: Implement the task for spec mvp, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Update core/src/bin/keyrx.rs to add Simulate command with --input and --script args. Update core/src/cli/commands/mod.rs to export SimulateCommand. Wire up command execution | Restrictions: Follow existing command patterns | _Requirements: REQ-5 | Success: keyrx simulate --help works, command executes. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

## Task 6: Implement Bench Command

- [x] 6. Create bench command for latency measurement
  - File: `core/src/cli/commands/bench.rs`
  - Create `BenchCommand` struct with iterations count
  - Create `BenchResult` struct (min, max, mean, p99 in nanoseconds)
  - Run N iterations of event processing with MockInput
  - Calculate statistics using `std::time::Instant`
  - Warn if mean > 1ms
  - Purpose: Verify performance requirements
  - _Leverage: Engine, MockInput, OutputWriter_
  - _Requirements: REQ-7_
  - _Prompt: Implement the task for spec mvp, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with performance testing experience | Task: Create BenchCommand in core/src/cli/commands/bench.rs. Accept --iterations (default 10000). Time event processing using Instant::now(). Calculate min/max/mean/p99 latency. Create BenchResult struct. Output via OutputWriter. Add warning if mean > 1_000_000 ns (1ms) | Restrictions: Use mocks not real input, keep measurement overhead minimal | _Leverage: Engine, MockInput, OutputWriter | _Requirements: REQ-7 | Success: keyrx bench outputs latency stats, --json works. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

- [x] 6.1 Wire bench command into CLI
  - File: `core/src/bin/keyrx.rs`, `core/src/cli/commands/mod.rs`
  - Replace placeholder bench implementation
  - Export BenchCommand from mod.rs
  - Purpose: Make bench command functional
  - _Leverage: Existing CLI structure_
  - _Requirements: REQ-7_
  - _Prompt: Implement the task for spec mvp, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Update core/src/bin/keyrx.rs to call BenchCommand instead of placeholder. Update mod.rs to export BenchCommand | Restrictions: Follow existing patterns | _Requirements: REQ-7 | Success: keyrx bench runs and outputs stats. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

## Task 7: Complete Doctor Command

- [x] 7. Implement platform-specific diagnostics
  - File: `core/src/cli/commands/doctor.rs`
  - Create `DiagnosticCheck` struct (name, status, message, remediation)
  - Linux: Check `/dev/uinput` exists and is accessible
  - Linux: Check user is in `input` group
  - Windows: Check keyboard hook API accessible
  - Collect all checks, report pass/fail with remediation
  - Purpose: Help users diagnose system setup issues
  - _Leverage: OutputWriter, platform-specific conditionals_
  - _Requirements: REQ-6_
  - _Prompt: Implement the task for spec mvp, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with cross-platform experience | Task: Enhance DoctorCommand in core/src/cli/commands/doctor.rs. Create DiagnosticCheck struct. Add checks: Linux - /dev/uinput exists, user in input group. Windows - keyboard hook registerable. Use #[cfg(target_os)] for platform code. Output all checks with pass/fail and remediation hints | Restrictions: Checks must not require elevated privileges to run, only to pass | _Leverage: OutputWriter | _Requirements: REQ-6 | Success: keyrx doctor shows all checks with clear status and remediation. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

## Task 8: Implement Linux Driver Stub

- [x] 8. Create Linux InputSource implementation
  - File: `core/src/drivers/linux.rs`
  - Implement `LinuxInput` struct implementing `InputSource`
  - Start: Open `/dev/uinput` or return error with remediation
  - Poll: Read events from evdev (stub: return empty vec)
  - Stop: Close file descriptors
  - Purpose: Linux platform driver foundation
  - _Leverage: evdev crate, InputSource trait_
  - _Requirements: REQ-1, REQ-8_
  - _Prompt: Implement the task for spec mvp, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with Linux systems experience | Task: Implement LinuxInput in core/src/drivers/linux.rs. Struct holds evdev device handle (Option for stub). Implement InputSource trait: start() opens device or returns descriptive error, poll_events() returns Vec<InputEvent> (empty for stub), stop() closes handle. Use async-trait | Restrictions: Keep as working stub, full evdev integration is post-MVP, handle permission errors gracefully | _Leverage: evdev crate, InputSource trait | _Requirements: REQ-1, REQ-8 | Success: Compiles on Linux, start() gives clear error if uinput unavailable. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

## Task 9: Implement Windows Driver Stub

- [x] 9. Create Windows InputSource implementation
  - File: `core/src/drivers/windows.rs`
  - Implement `WindowsInput` struct implementing `InputSource`
  - Start: Attempt keyboard hook registration (stub: log and succeed)
  - Poll: Return empty events (stub)
  - Stop: Unhook keyboard
  - Purpose: Windows platform driver foundation
  - _Leverage: windows-rs crate, InputSource trait_
  - _Requirements: REQ-1, REQ-8_
  - _Prompt: Implement the task for spec mvp, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with Windows API experience | Task: Implement WindowsInput in core/src/drivers/windows.rs. Use windows-rs crate. Implement InputSource trait: start() logs hook registration attempt (stub: always succeed), poll_events() returns empty Vec, stop() logs unhook. Use #[cfg(windows)] guard | Restrictions: Keep as working stub, full hook integration is post-MVP, no blocking calls | _Leverage: windows-rs crate, InputSource trait | _Requirements: REQ-1, REQ-8 | Success: Compiles on Windows, can be instantiated and started. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

## Task 10: Complete Run Command with Real Engine

- [x] 10. Wire up run command with full engine pipeline
  - File: `core/src/cli/commands/run.rs`
  - Load script via RhaiRuntime if provided
  - Call on_init() hook if defined
  - Create Engine with platform driver (or MockInput if --mock flag)
  - Run event loop until Ctrl+C
  - Output debug info if --debug flag
  - Purpose: Make engine actually process real/simulated input
  - _Leverage: Engine, RhaiRuntime, LinuxInput/WindowsInput, OutputWriter_
  - _Requirements: REQ-2, REQ-3_
  - _Prompt: Implement the task for spec mvp, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Enhance RunCommand in core/src/cli/commands/run.rs. If --script provided, load with RhaiRuntime and call on_init() if present. Create Engine with appropriate InputSource (platform driver or MockInput if --mock). Run event loop. On Ctrl+C, stop engine gracefully. Debug flag enables verbose logging | Restrictions: Use tracing for debug output, handle all errors gracefully | _Leverage: Engine, RhaiRuntime, drivers | _Requirements: REQ-2, REQ-3 | Success: keyrx run --script config.rhai starts engine, processes events, stops cleanly. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

## Task 11: Create Example Rhai Script

- [ ] 11. Create example remap script
  - File: `scripts/std/example.rhai`
  - Demonstrate remap(), block(), pass() functions
  - Include on_init() hook with debug message
  - Add comments explaining each function
  - Purpose: Provide working example for users and testing
  - _Leverage: Rhai functions registered in RhaiRuntime_
  - _Requirements: REQ-3, REQ-4_
  - _Prompt: Implement the task for spec mvp, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rhai Script Developer | Task: Create example.rhai in scripts/std/. Define on_init() that prints debug message. Call remap("CapsLock", "Escape"), block("Insert"), pass("Enter"). Add comments explaining each function. Keep script simple and educational | Restrictions: Only use registered functions, no undefined calls | _Leverage: Registered remap/block/pass functions | _Requirements: REQ-3, REQ-4 | Success: keyrx check scripts/std/example.rhai passes, keyrx run --script scripts/std/example.rhai starts without error. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

## Task 12: Add Unit Tests for Core Components

- [ ] 12. Create engine unit tests
  - File: `core/tests/engine_test.rs`
  - Test Engine creation with mocks
  - Test process_event with various RemapActions
  - Test key-down and key-up handling
  - Purpose: Verify core engine behavior
  - _Leverage: MockInput, MockRuntime, MockState_
  - _Requirements: REQ-4_
  - _Prompt: Implement the task for spec mvp, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Test Engineer | Task: Create core/tests/engine_test.rs. Test Engine::new with mocks. Test process_event: remap A->B, block CapsLock, pass Enter. Test both key-down (pressed=true) and key-up (pressed=false) events produce correct OutputActions | Restrictions: Use only mocks, no real input, aim for 80% coverage of event_loop.rs | _Leverage: MockInput, MockRuntime, MockState | _Requirements: REQ-4 | Success: All tests pass, cover main code paths. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

- [ ] 12.1 Create simulate command tests
  - File: `core/tests/simulate_test.rs`
  - Test input parsing ("A,B,C" -> events)
  - Test simulate with simple remap script
  - Test JSON output format
  - Purpose: Verify simulate command works correctly
  - _Leverage: SimulateCommand_
  - _Requirements: REQ-5_
  - _Prompt: Implement the task for spec mvp, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Test Engineer | Task: Create core/tests/simulate_test.rs. Test input string parsing. Test simulate with remap script produces correct output. Test --json flag produces valid JSON | Restrictions: Use temp files for test scripts | _Leverage: SimulateCommand, tempfile crate | _Requirements: REQ-5 | Success: All tests pass, simulate command verified. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

- [ ] 12.2 Create RemapRegistry unit tests
  - File: `core/src/scripting/registry.rs` (inline tests)
  - Test remap, block, pass, lookup, clear methods
  - Test default behavior (unmapped keys return Pass)
  - Purpose: Verify registry correctness
  - _Leverage: RemapRegistry_
  - _Requirements: REQ-4_
  - _Prompt: Implement the task for spec mvp, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Test Engineer | Task: Add #[cfg(test)] mod tests to core/src/scripting/registry.rs. Test: remap A->B then lookup A returns Remap(B). Test block CapsLock then lookup returns Block. Test unmapped key returns Pass. Test clear resets all | Restrictions: Keep tests inline in same file | _Requirements: REQ-4 | Success: cargo test registry passes. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

## Task 13: Build Verification

- [ ] 13. Verify Linux build
  - Run `cargo build --release` on Linux
  - Run `cargo test` to verify all tests pass
  - Run `keyrx --help` to verify CLI works
  - Run `keyrx check scripts/std/example.rhai`
  - Purpose: Confirm Linux build works end-to-end
  - _Requirements: REQ-1, REQ-8_
  - _Prompt: Implement the task for spec mvp, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Build Engineer | Task: On Linux, run: cargo build --release, cargo test, ./target/release/keyrx --help, ./target/release/keyrx check scripts/std/example.rhai. Fix any build or test failures | Restrictions: All commands must exit 0 | _Requirements: REQ-1, REQ-8 | Success: All commands succeed, binary runs. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

- [ ] 13.1 Verify Windows build (or cross-compile)
  - Run `cargo build --release` on Windows OR
  - Run `cargo build --target x86_64-pc-windows-gnu` for cross-compile
  - Verify binary is produced
  - Purpose: Confirm Windows build works
  - _Requirements: REQ-1, REQ-8_
  - _Prompt: Implement the task for spec mvp, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Build Engineer | Task: Build for Windows target. Either on Windows run cargo build --release, or on Linux run cargo build --target x86_64-pc-windows-gnu (requires mingw-w64 installed). Verify .exe is produced | Restrictions: Binary must be produced without errors | _Requirements: REQ-1, REQ-8 | Success: Windows binary exists and is valid PE executable. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

## Task 14: Documentation

- [ ] 14. Update CLI help text and README
  - File: `core/src/bin/keyrx.rs`, `README.md`
  - Ensure all commands have descriptive help text
  - Add basic usage examples to README
  - Purpose: Users can understand how to use KeyRx
  - _Requirements: REQ-2_
  - _Prompt: Implement the task for spec mvp, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical Writer | Task: Review all command definitions in keyrx.rs, ensure #[command(about)] and #[arg(help)] are clear. Update README.md with: Installation (cargo build), Basic Usage (keyrx run, check, simulate, doctor, bench), Example Script | Restrictions: Keep README concise, link to detailed docs if needed | _Requirements: REQ-2 | Success: keyrx --help is clear, README has working examples. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._
