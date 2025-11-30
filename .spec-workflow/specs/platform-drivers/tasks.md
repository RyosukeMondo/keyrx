# Tasks Document: platform-drivers

## Task 1: Update Dependencies and Error Types

- [x] 1.1 Update Cargo.toml for evdev uinput feature
  - File: `core/Cargo.toml`
  - Change evdev to: `evdev = { version = "0.12", features = ["tokio"] }`
  - Add: `nix = { version = "0.27", features = ["ioctl"] }` for EVIOCGRAB
  - Purpose: Enable uinput device creation and async evdev
  - _Requirements: REQ-1, REQ-2_
  - _Prompt: Implement the task for spec platform-drivers, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Update core/Cargo.toml to modify evdev dependency to include tokio feature. Add nix crate with ioctl feature for EVIOCGRAB support. Keep existing dependencies unchanged | Restrictions: Use compatible versions, don't break existing code | Success: cargo build succeeds with new dependencies. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

- [x] 1.2 Add driver error types
  - File: `core/src/error.rs`
  - Add LinuxDriverError enum with DeviceNotFound, PermissionDenied, GrabFailed, UinputFailed
  - Add WindowsDriverError enum with HookInstallFailed, SendInputFailed, MessagePumpPanic
  - Implement From conversions for KeyRxError
  - Purpose: Structured driver error handling
  - _Requirements: REQ-7_
  - _Prompt: Implement the task for spec platform-drivers, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Add to core/src/error.rs: LinuxDriverError enum with variants DeviceNotFound{path}, PermissionDenied{path}, GrabFailed(io::Error), UinputFailed(io::Error). Add WindowsDriverError with HookInstallFailed(u32), SendInputFailed(u32), MessagePumpPanic. Add From impls to convert to KeyRxError | Restrictions: Include remediation hints in error messages | Success: Driver errors integrate with existing error hierarchy. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

## Task 2: Create Shared Driver Infrastructure

- [x] 2.1 Create common.rs with DeviceInfo
  - File: `core/src/drivers/common.rs`
  - Define DeviceInfo struct (path, name, vendor_id, product_id, is_keyboard)
  - Implement Serialize for JSON output
  - Add Display impl for human-readable output
  - Purpose: Shared device listing types
  - _Requirements: REQ-8_
  - _Prompt: Implement the task for spec platform-drivers, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Create core/src/drivers/common.rs with DeviceInfo struct: path: PathBuf, name: String, vendor_id: u16, product_id: u16, is_keyboard: bool. Derive Debug, Clone, Serialize. Add Display impl showing "name (vendor:product) at path" | Restrictions: Keep platform-agnostic | Success: DeviceInfo can be serialized to JSON. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

- [x] 2.2 Update drivers/mod.rs
  - File: `core/src/drivers/mod.rs`
  - Add `mod common;`
  - Conditionally export LinuxInput or WindowsInput based on platform
  - Create type alias `pub type PlatformInput = ...` for current platform
  - Purpose: Clean platform abstraction
  - _Requirements: REQ-5_
  - _Prompt: Implement the task for spec platform-drivers, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Update core/src/drivers/mod.rs to add mod common and pub use common::DeviceInfo. Add conditional compilation: #[cfg(target_os = "linux")] pub type PlatformInput = LinuxInput; #[cfg(windows)] pub type PlatformInput = WindowsInput. Export list_keyboards function | Restrictions: Must compile on both platforms | Success: PlatformInput resolves to correct driver. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

## Task 3: Implement Linux evdev Reader

- [x] 3.1 Create EvdevReader struct
  - File: `core/src/drivers/linux.rs`
  - Struct with: device (evdev::Device), tx (Sender), running (Arc<AtomicBool>)
  - Implement grab() using EVIOCGRAB ioctl
  - Implement ungrab() for cleanup
  - Purpose: Keyboard event capture
  - _Requirements: REQ-1_
  - _Prompt: Implement the task for spec platform-drivers, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with Linux experience | Task: In core/src/drivers/linux.rs, create EvdevReader struct with evdev::Device, crossbeam Sender<InputEvent>, Arc<AtomicBool> for running flag. Implement grab() that calls device.grab() to get exclusive access. Implement ungrab() for cleanup. Use evdev crate's built-in grab method | Restrictions: Handle grab failure gracefully with clear error | Success: EvdevReader can grab and ungrab keyboard. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

- [x] 3.2 Implement evdev read loop
  - File: `core/src/drivers/linux.rs`
  - Create spawn() that starts blocking read thread
  - Read events via device.fetch_events()
  - Convert evdev events to InputEvent using evdev_to_keycode
  - Send via channel to async engine
  - Purpose: Continuous keyboard event capture
  - _Requirements: REQ-1_
  - _Prompt: Implement the task for spec platform-drivers, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Add spawn() method to EvdevReader that spawns std::thread. In thread loop: check running flag, call device.fetch_events(), filter for EV_KEY events, convert to InputEvent using evdev_to_keycode, send via tx channel. Exit loop when running is false | Restrictions: Use blocking evdev read (not async), handle channel send errors | Success: Events flow from keyboard to channel. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

## Task 4: Implement Linux uinput Writer

- [x] 4.1 Create UinputWriter struct
  - File: `core/src/drivers/linux.rs`
  - Create virtual keyboard via evdev::UInputDevice
  - Register all KEY_* codes we support
  - Set device name to "KeyRx Virtual Keyboard"
  - Purpose: Key injection capability
  - _Requirements: REQ-2_
  - _Prompt: Implement the task for spec platform-drivers, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Create UinputWriter struct in linux.rs. In new(), create evdev::VirtualDeviceBuilder with name "KeyRx Virtual Keyboard". Register KEY events for all keys in KeyCode enum. Call build() to create UInputDevice | Restrictions: Handle uinput permission errors with remediation message | Success: Virtual keyboard device created at /dev/input/eventX. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

- [ ] 4.2 Implement key emission
  - File: `core/src/drivers/linux.rs`
  - Add emit(key, pressed) method
  - Convert KeyCode to evdev key code (reverse of evdev_to_keycode)
  - Write EV_KEY event with value 1 (press) or 0 (release)
  - Add sync() to send EV_SYN
  - Purpose: Inject remapped keys
  - _Requirements: REQ-2_
  - _Prompt: Implement the task for spec platform-drivers, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Add emit(&mut self, key: KeyCode, pressed: bool) -> Result<()> to UinputWriter. Add keycode_to_evdev() function (reverse mapping). Create InputEvent with EV_KEY type, converted code, value 1 or 0. Write to device. Add sync() that writes EV_SYN event | Restrictions: Call sync() after each emit for immediate effect | Success: Emitted keys appear as input to applications. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

## Task 5: Complete LinuxInput Implementation

- [ ] 5.1 Rewrite LinuxInput struct
  - File: `core/src/drivers/linux.rs`
  - Fields: reader_handle, writer, rx (Receiver), running, device_path
  - Implement new() that finds keyboard device
  - Implement list_devices() to enumerate /dev/input/event*
  - Purpose: Coordinate reader and writer
  - _Requirements: REQ-1, REQ-8_
  - _Prompt: Implement the task for spec platform-drivers, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Rewrite LinuxInput struct with: reader_handle: Option<JoinHandle<()>>, writer: UinputWriter, rx: Receiver<InputEvent>, running: Arc<AtomicBool>, device_path: PathBuf. In new(path: Option<PathBuf>), if path is None, find first keyboard in /dev/input/. Create channel, store rx. Create UinputWriter. Add list_devices() that iterates /dev/input/event*, opens each, checks for KEY capability | Restrictions: Don't start reader in new(), wait for start() | Success: LinuxInput can be created with or without explicit device. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

- [ ] 5.2 Implement InputSource trait for LinuxInput
  - File: `core/src/drivers/linux.rs`
  - start(): Spawn EvdevReader, grab keyboard
  - stop(): Set running=false, join thread, ungrab
  - poll_events(): Try receive from channel (non-blocking)
  - send_output(): Match OutputAction, call writer.emit()
  - Purpose: Complete InputSource implementation
  - _Requirements: REQ-1, REQ-2, REQ-5_
  - _Prompt: Implement the task for spec platform-drivers, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Implement InputSource for LinuxInput. start(): create EvdevReader, call spawn(), store handle. stop(): set running=false, join handle, drop writer. poll_events(): try_recv from rx channel, return vec (may be empty). send_output(): match on OutputAction - KeyDown/KeyUp call writer.emit(), Block does nothing, PassThrough re-emits original | Restrictions: poll_events must not block, use try_recv | Success: LinuxInput works with Engine. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

## Task 6: Implement Windows Hook Manager

- [ ] 6.1 Create HookManager struct
  - File: `core/src/drivers/windows.rs`
  - Use SetWindowsHookExW for WH_KEYBOARD_LL
  - Store HHOOK handle
  - Create thread-local storage for callback context
  - Purpose: Low-level keyboard hook
  - _Requirements: REQ-3_
  - _Prompt: Implement the task for spec platform-drivers, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with Windows API experience | Task: Create HookManager in windows.rs. Use windows-rs SetWindowsHookExW with WH_KEYBOARD_LL. Store hook handle. Use thread_local! macro to store Sender for callback access. Implement install() that registers hook with low_level_keyboard_proc callback. Handle errors from GetLastError | Restrictions: Hook must be installed from thread with message pump | Success: Hook receives keyboard events. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

- [ ] 6.2 Implement hook callback
  - File: `core/src/drivers/windows.rs`
  - Create extern "system" callback function
  - Extract KBDLLHOOKSTRUCT from lParam
  - Convert vkCode to KeyCode using vk_to_keycode
  - Send InputEvent via channel
  - Return 0 to pass, 1 to block
  - Purpose: Capture keyboard events
  - _Requirements: REQ-3_
  - _Prompt: Implement the task for spec platform-drivers, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Create unsafe extern "system" fn low_level_keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT. If code >= 0: cast lparam to *const KBDLLHOOKSTRUCT, extract vkCode and flags. Determine pressed from WM_KEYDOWN/WM_SYSKEYDOWN vs WM_KEYUP. Convert vkCode to KeyCode. Send to channel via thread_local. Return CallNextHookEx result | Restrictions: Callback must complete within 100ms per Windows requirements | Success: Events sent to channel from callback. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

- [ ] 6.3 Implement message pump
  - File: `core/src/drivers/windows.rs`
  - Create run_message_loop() function
  - Use GetMessageW, TranslateMessage, DispatchMessageW
  - Check running flag periodically via PeekMessageW
  - Purpose: Required for hook callbacks
  - _Requirements: REQ-3_
  - _Prompt: Implement the task for spec platform-drivers, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Add run_message_loop(&self) to HookManager. Loop while running: call PeekMessageW with PM_REMOVE. If message received, TranslateMessage and DispatchMessageW. If WM_QUIT received, break. Sleep 1ms if no message to avoid busy loop | Restrictions: Must handle WM_QUIT for graceful shutdown | Success: Message pump processes hook callbacks. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

## Task 7: Implement Windows Key Injection

- [ ] 7.1 Create SendInputInjector
  - File: `core/src/drivers/windows.rs`
  - Implement inject_key(key, pressed) using SendInput
  - Build KEYBDINPUT structure correctly
  - Handle extended keys (arrows, numpad, etc.)
  - Purpose: Inject remapped keys
  - _Requirements: REQ-4_
  - _Prompt: Implement the task for spec platform-drivers, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Create SendInputInjector struct (stateless). Add inject_key(key: KeyCode, pressed: bool) -> Result<()>. Convert KeyCode to vkCode using keycode_to_vk. Create INPUT structure with type INPUT_KEYBOARD. Set KEYBDINPUT with wVk, dwFlags (KEYEVENTF_KEYUP if !pressed, KEYEVENTF_EXTENDEDKEY for extended keys). Call SendInput. Check return value | Restrictions: Extended key detection needed for arrows, Home/End, etc. | Success: Injected keys received by applications. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

## Task 8: Complete WindowsInput Implementation

- [ ] 8.1 Rewrite WindowsInput struct
  - File: `core/src/drivers/windows.rs`
  - Fields: hook_thread (JoinHandle), rx (Receiver), running
  - Spawn message pump thread in start()
  - Coordinate hook lifecycle
  - Purpose: Complete Windows driver
  - _Requirements: REQ-3, REQ-4, REQ-5_
  - _Prompt: Implement the task for spec platform-drivers, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Rewrite WindowsInput with: hook_thread: Option<JoinHandle<()>>, rx: Receiver<InputEvent>, running: Arc<AtomicBool>. In start(): spawn thread that installs hook, runs message loop, uninstalls on exit. In stop(): set running=false, post WM_QUIT to thread, join. poll_events(): try_recv from rx. send_output(): call SendInputInjector | Restrictions: Hook must be installed from same thread as message pump | Success: WindowsInput works with Engine. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

## Task 9: CLI Devices Command

- [ ] 9.1 Create devices command
  - File: `core/src/cli/commands/devices.rs`
  - List all keyboard devices
  - Show: name, vendor:product, path
  - Support --json flag
  - Purpose: Device discovery for users
  - _Requirements: REQ-8_
  - _Prompt: Implement the task for spec platform-drivers, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Create DevicesCommand in core/src/cli/commands/devices.rs. Call drivers::list_keyboards(). Output via OutputWriter - human format: "name (vendor:product) - path", JSON format: array of DeviceInfo. Handle empty list case | Restrictions: Work on both Linux and Windows | Success: keyrx devices lists keyboards. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

- [ ] 9.2 Wire devices command into CLI
  - File: `core/src/bin/keyrx.rs`, `core/src/cli/commands/mod.rs`
  - Add Devices subcommand
  - Export DevicesCommand
  - Purpose: Make command accessible
  - _Requirements: REQ-8_
  - _Prompt: Implement the task for spec platform-drivers, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Add Devices variant to Commands enum in keyrx.rs. Update mod.rs to export DevicesCommand. Wire up command execution | Restrictions: Follow existing command patterns | Success: keyrx devices --help works. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

## Task 10: Update Run Command

- [ ] 10.1 Use real driver in run command
  - File: `core/src/cli/commands/run.rs`
  - Use PlatformInput instead of MockInput (unless --mock flag)
  - Add --device flag to select keyboard
  - Handle driver initialization errors
  - Purpose: Enable real keyboard remapping
  - _Requirements: REQ-5_
  - _Prompt: Implement the task for spec platform-drivers, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Update RunCommand to accept --device option. If --mock flag, use MockInput. Otherwise, create PlatformInput with device path. Handle initialization errors gracefully (print error, exit 1). On success, run engine loop | Restrictions: Fallback to mock if driver fails should be optional | Success: keyrx run uses real keyboard on supported platforms. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

## Task 11: Implement Graceful Shutdown

- [ ] 11.1 Add signal handlers
  - File: `core/src/drivers/linux.rs`, `core/src/drivers/windows.rs`
  - Linux: Handle SIGINT, SIGTERM
  - Windows: Handle Ctrl+C via SetConsoleCtrlHandler
  - Trigger driver stop on signal
  - Purpose: Clean keyboard release on termination
  - _Requirements: REQ-7_
  - _Prompt: Implement the task for spec platform-drivers, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: For Linux: use signal-hook crate to register SIGINT/SIGTERM handlers that set running=false. For Windows: use SetConsoleCtrlHandler to catch CTRL_C_EVENT. Ensure keyboard grab is released even on forced termination. Add Drop impl that calls stop() | Restrictions: Handlers must be async-signal-safe | Success: Ctrl+C releases keyboard cleanly. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

- [ ] 11.2 Implement panic recovery
  - File: `core/src/drivers/linux.rs`, `core/src/drivers/windows.rs`
  - Wrap thread code in catch_unwind
  - On panic: release keyboard, log error
  - Set error flag for main thread to detect
  - Purpose: Never leave keyboard stuck
  - _Requirements: REQ-7_
  - _Prompt: Implement the task for spec platform-drivers, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: In driver thread spawn, wrap main loop in std::panic::catch_unwind. On panic: log error, call ungrab/unhook cleanup, set error flag. In poll_events, check error flag and return Err if set | Restrictions: Panic recovery is best-effort, main goal is keyboard release | Success: Panics don't leave keyboard grabbed. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

## Task 12: Testing

- [ ] 12.1 Add driver unit tests
  - File: `core/src/drivers/linux.rs`, `core/src/drivers/windows.rs`
  - Test key code conversions (roundtrip)
  - Test DeviceInfo creation
  - Test error types
  - Purpose: Verify driver components
  - _Requirements: REQ-6_
  - _Prompt: Implement the task for spec platform-drivers, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Test Engineer | Task: Add unit tests to driver files. Test evdev_to_keycode and keycode_to_evdev roundtrip for all keys. Test vk_to_keycode and keycode_to_vk roundtrip. Test DeviceInfo Display impl. Test error message formatting | Restrictions: Don't require real devices for unit tests | Success: cargo test passes with new tests. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

- [ ] 12.2 Add integration tests
  - File: `core/tests/driver_integration_test.rs`
  - Test driver start/stop lifecycle (with mock or skip on CI)
  - Test event channel communication
  - Test graceful shutdown
  - Purpose: Verify driver integration
  - _Requirements: REQ-5, REQ-7_
  - _Prompt: Implement the task for spec platform-drivers, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Test Engineer | Task: Create driver_integration_test.rs. Add #[ignore] tests that require real hardware. Test: create driver, start, stop without panic. Test channel communication works. Test cleanup on drop. Use conditional compilation for platform-specific tests | Restrictions: Mark hardware-requiring tests with #[ignore] for CI | Success: Integration tests pass locally, skipped on CI. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

## Task 13: Documentation

- [ ] 13.1 Add driver module documentation
  - File: `core/src/drivers/linux.rs`, `core/src/drivers/windows.rs`
  - Add //! module documentation
  - Document platform-specific requirements
  - Document error recovery behavior
  - Purpose: Developer documentation
  - _Requirements: REQ-7_
  - _Prompt: Implement the task for spec platform-drivers, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical Writer | Task: Add comprehensive //! module docs to both driver files. Document: platform requirements, permission requirements, thread model, error handling, cleanup behavior. Add doc comments to all public types and methods | Restrictions: Keep accurate to implementation | Success: cargo doc generates useful documentation. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

- [ ] 13.2 Update README with driver info
  - File: `README.md`
  - Add "Platform Setup" section
  - Document Linux: udev rules, input group
  - Document Windows: running as admin (if needed)
  - Add troubleshooting section
  - Purpose: User documentation
  - _Requirements: REQ-7, REQ-8_
  - _Prompt: Implement the task for spec platform-drivers, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical Writer | Task: Update README.md with Platform Setup section. Linux: explain input group, udev rules, modprobe uinput. Windows: explain any admin requirements, antivirus exceptions. Add Troubleshooting section with common errors and solutions | Restrictions: Keep concise, link to detailed docs if needed | Success: Users can set up KeyRx from README. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

## Task 14: Verification

- [ ] 14.1 End-to-end testing on Linux
  - Set up test environment with proper permissions
  - Run: keyrx devices (verify keyboard listed)
  - Run: keyrx run --script example.rhai
  - Verify: CapsLock produces Escape
  - Verify: Ctrl+C stops cleanly
  - Purpose: Confirm Linux driver works
  - _Requirements: REQ-1, REQ-2, REQ-6, REQ-7_
  - _Prompt: Implement the task for spec platform-drivers, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer | Task: On Linux system with proper permissions: 1) Run keyrx devices, verify keyboard shown, 2) Run keyrx run --script scripts/std/example.rhai, 3) Press CapsLock, verify Escape received, 4) Press Insert, verify blocked, 5) Press Ctrl+C, verify clean shutdown. Document any issues found | Restrictions: Requires physical access to Linux system | Success: All tests pass, keyboard works normally after. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

- [ ] 14.2 End-to-end testing on Windows
  - Run: keyrx devices
  - Run: keyrx run --script example.rhai
  - Verify: CapsLock produces Escape
  - Verify: Ctrl+C stops cleanly
  - Purpose: Confirm Windows driver works
  - _Requirements: REQ-3, REQ-4, REQ-6, REQ-7_
  - _Prompt: Implement the task for spec platform-drivers, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer | Task: On Windows system: 1) Run keyrx.exe devices, 2) Run keyrx.exe run --script scripts/std/example.rhai, 3) Press CapsLock, verify Escape, 4) Press Insert, verify blocked, 5) Ctrl+C for clean shutdown. Document any antivirus warnings | Restrictions: Requires physical access to Windows system | Success: All tests pass, keyboard works normally after. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

## Task 15: Event Metadata Infrastructure (REQ-9)

- [ ] 15.1 Extend InputEvent struct with metadata fields
  - File: `core/src/engine/types.rs`
  - Add fields: timestamp_us (u64), device_id (Option<String>), is_repeat (bool), is_synthetic (bool), scan_code (u16)
  - Update Default impl with sensible defaults
  - Add doc comments explaining each field's purpose and platform sources
  - Purpose: Enable advanced remapping features (tap-hold, custom modifiers, device-specific configs)
  - _Requirements: REQ-9_
  - _Prompt: Implement the task for spec platform-drivers, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Update InputEvent in core/src/engine/types.rs to add: timestamp_us: u64 (μs since driver start), device_id: Option<String> (source keyboard), is_repeat: bool (auto-repeat vs initial), is_synthetic: bool (software-injected), scan_code: u16 (raw hardware code). Update Default impl. Add doc comments | Restrictions: Maintain backward compat with existing tests | Success: cargo build succeeds, existing tests pass. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

- [ ] 15.2 Update MockInput to populate metadata
  - File: `core/src/mocks/mock_input.rs`
  - Generate timestamp_us as monotonic counter
  - Set is_synthetic=false, is_repeat=false by default
  - Set scan_code=0, device_id=Some("mock")
  - Purpose: Enable testing of metadata-dependent logic
  - _Requirements: REQ-9_
  - _Prompt: Implement the task for spec platform-drivers, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Update MockInput to populate InputEvent metadata. Add start_time field, compute timestamp_us from elapsed. Set device_id=Some("mock".to_string()), is_synthetic=false, is_repeat=false, scan_code=0 | Restrictions: Keep existing MockInput API working | Success: Mock events have valid metadata. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

- [ ] 15.3 Add synthetic event filtering in Engine
  - File: `core/src/engine/event_loop.rs`
  - Check is_synthetic flag before processing
  - Skip processing and log at debug level
  - Purpose: Prevent infinite loops from re-processing injected events
  - _Requirements: REQ-9_
  - _Prompt: Implement the task for spec platform-drivers, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: In Engine::process_event, add early return if event.is_synthetic is true. Log at debug level: "Skipping synthetic event: {:?}". This prevents infinite loops when our injected keys are recaptured by the hook | Restrictions: Consider making filtering optional via config for special cases | Success: Synthetic events are skipped by default. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

- [ ] 15.4 Capture Linux metadata in evdev reader
  - File: `core/src/drivers/linux.rs`
  - In read loop: extract timestamp from event.time (convert to μs)
  - Set scan_code from event.code
  - Set is_repeat from event.value == 2
  - Set device_id from device path
  - Detect is_synthetic by comparing to uinput device fd
  - Purpose: Full metadata capture on Linux
  - _Requirements: REQ-1, REQ-9_
  - _Prompt: Implement the task for spec platform-drivers, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: In EvdevReader read loop, populate InputEvent metadata: timestamp_us from event.time (sec*1000000 + usec), scan_code from event.code, is_repeat from value==2, device_id from stored device path, is_synthetic by comparing event source fd to uinput fd | Restrictions: Handle missing metadata gracefully with defaults | Success: Linux events have complete metadata. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

- [ ] 15.5 Capture Windows metadata in hook callback
  - File: `core/src/drivers/windows.rs`
  - Extract timestamp from KBDLLHOOKSTRUCT.time (convert ms→μs)
  - Set scan_code from KBDLLHOOKSTRUCT.scanCode
  - Detect is_synthetic from LLKHF_INJECTED flag
  - Track key state for is_repeat detection
  - Purpose: Full metadata capture on Windows
  - _Requirements: REQ-3, REQ-9_
  - _Prompt: Implement the task for spec platform-drivers, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: In hook callback, populate InputEvent metadata: timestamp_us from time field (multiply by 1000 for μs), scan_code from scanCode field, is_synthetic from (flags & LLKHF_INJECTED) != 0. Track HashMap<u16, bool> for key states to detect is_repeat (key down when already down) | Restrictions: Keep callback fast (<100ms requirement) | Success: Windows events have complete metadata. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._
