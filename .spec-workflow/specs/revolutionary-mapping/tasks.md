# Tasks Document - Revolutionary Mapping

## Phase 1: Core Data Structures & Device Identity (2 weeks)

- [x] 1.1. Create DeviceIdentity types
  - Files: `core/src/identity/types.rs`, `core/src/identity/mod.rs`
  - Define DeviceIdentity struct with vendor_id, product_id, serial_number, user_label
  - Implement to_key() and from_key() methods
  - Add Hash, Eq, PartialEq implementations for HashMap usage
  - Add Serialize/Deserialize for persistence
  - _Leverage: serde crate for serialization_
  - _Requirements: 1_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Systems Developer specializing in type systems and serialization | Task: Create comprehensive DeviceIdentity type system following requirement 1, implementing Hash/Eq for HashMap usage and Serialize/Deserialize for JSON persistence | Restrictions: Must use standard library types only (no dependencies except serde), maintain zero-copy where possible, ensure hash collision resistance | _Leverage: Existing serde patterns from config/ module | Success: DeviceIdentity compiles, all traits implemented correctly, can be used as HashMap key, serializes to/from JSON correctly. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool including detailed artifacts (structs, methods, traits implemented), then mark as [x] when fully tested._

- [x] 1.2. Implement Windows serial extraction
  - Files: `core/src/identity/windows.rs`, `core/src/identity/mod.rs`
  - Implement extract_serial_number(device_path: &str) -> Result<String>
  - Implement parse_instance_id_from_path() helper
  - Implement read_iserial_descriptor() using HidD_GetSerialNumberString
  - Handle fallback to InstanceID when iSerial unavailable
  - _Leverage: windows-rs crate for HID API_
  - _Requirements: 8_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Windows Systems Developer with expertise in Win32 API and USB/HID protocols | Task: Implement Windows serial number extraction following requirement 8, using Raw Input API device paths and HID descriptors | Restrictions: Must use windows-rs crate only, handle all error cases gracefully, conditional compile with #[cfg(windows)], never panic on invalid paths | _Leverage: Existing windows driver patterns from core/src/drivers/windows/, windows-rs integration patterns | Success: Extracts serial from devices with iSerial, falls back to InstanceID correctly, handles malformed paths, all unit tests pass. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool including detailed artifacts (functions created, Win32 APIs used, error handling patterns), then mark as [x]._

- [x] 1.3. Implement Linux serial extraction
  - Files: `core/src/identity/linux.rs`, `core/src/identity/mod.rs`
  - Implement extract_serial_number(device_path: &Path) -> Result<String>
  - Implement read_udev_serial() helper for sysfs reading
  - Implement generate_synthetic_id() using phys path hashing
  - Use EVIOCGUNIQ ioctl via evdev crate
  - _Leverage: evdev crate, std::fs for udev sysfs_
  - _Requirements: 9_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Linux Systems Developer with expertise in evdev and udev subsystems | Task: Implement Linux serial number extraction following requirement 9, using EVIOCGUNIQ ioctl and udev properties | Restrictions: Must use evdev crate API only, conditional compile with #[cfg(target_os = "linux")], ensure synthetic ID stability across reboots | _Leverage: Existing Linux driver patterns from core/src/drivers/linux/, evdev crate usage | Success: Extracts serial via EVIOCGUNIQ when available, falls back to udev then synthetic ID, hash is stable, all unit tests pass. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool including detailed artifacts (functions, ioctl calls, hash algorithm), then mark as [x]._

- [x] 1.4. Add serial extraction to Windows driver
  - Files: `core/src/drivers/windows/raw_input.rs`
  - Call identity::windows::extract_serial_number() on device connection
  - Include serial_number in InputEvent metadata
  - Update RawDeviceHandle to include device path
  - _Leverage: Existing Raw Input device detection code_
  - _Requirements: 8_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Windows Driver Developer with expertise in Raw Input API integration | Task: Integrate serial extraction into Windows driver following requirement 8, calling identity::windows::extract_serial_number() on device events | Restrictions: Must maintain existing driver performance (<50μs overhead), handle extraction failures gracefully, log warnings for port-bound devices | _Leverage: Existing device detection in raw_input.rs | Success: Serial number extracted on device connection, included in events, no performance regression, all integration tests pass. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool including artifacts (modified functions, integration points), then mark as [x]._

- [x] 1.5. Add serial extraction to Linux driver
  - Files: `core/src/drivers/linux/evdev_input.rs`
  - Call identity::linux::extract_serial_number() on device open
  - Include serial_number in InputEvent metadata
  - Update device detection to use DeviceIdentity
  - _Leverage: Existing evdev device opening code_
  - _Requirements: 9_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Linux Driver Developer with expertise in evdev integration | Task: Integrate serial extraction into Linux driver following requirement 9, calling identity::linux::extract_serial_number() on device open | Restrictions: Must maintain driver performance, handle extraction failures, conditional compilation | _Leverage: Existing evdev_input.rs device opening | Success: Serial number extracted on device open, included in events, no performance regression, integration tests pass. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool, then mark as [x]._

- [x] 1.6. Define Profile data model
  - Files: `core/src/registry/profile.rs`, `core/src/registry/mod.rs`
  - Define Profile struct with id, name, layout_type, mappings, timestamps
  - Define LayoutType enum (Standard, Matrix, Split)
  - Define PhysicalPosition struct (row, col)
  - Define KeyAction enum (Key, Chord, Script, Block, Pass)
  - Add Serialize/Deserialize
  - _Leverage: serde for serialization, existing KeyCode from drivers/keycodes/_
  - _Requirements: 3_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer specializing in data modeling and type design | Task: Create Profile data model following requirement 3, defining all layout types and key actions | Restrictions: Must be platform-agnostic, maintain type safety, use existing KeyCode enum | _Leverage: Existing KeyCode from drivers/keycodes/definitions.rs, PhysicalKey concept from discovery/types.rs | Success: All types compile, serialize correctly, HashMap<PhysicalPosition, KeyAction> works, unit tests pass. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool including artifacts (all structs/enums created), then mark as [x]._

- [x] 1.7. Implement DeviceRegistry
  - Files: `core/src/registry/device.rs`, `core/src/registry/mod.rs`
  - Create DeviceRegistry struct with Arc<RwLock<HashMap<DeviceIdentity, DeviceState>>>
  - Implement register_device(), unregister_device()
  - Implement set_remap_enabled(), assign_profile()
  - Implement get_device_state(), list_devices()
  - Add event emission (DeviceEvent enum)
  - _Leverage: tokio for async, mpsc for events_
  - _Requirements: 2_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Concurrent Systems Developer with expertise in async Rust and thread-safe data structures | Task: Implement DeviceRegistry following requirement 2, managing runtime device state with thread-safe concurrent access | Restrictions: Must use RwLock for shared access, minimize lock hold time, emit events on all state changes | _Leverage: Existing event channel pattern from engine/multi_device.rs | Success: All CRUD operations work, concurrent access is safe, events emitted correctly, unit tests with concurrency scenarios pass. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool including artifacts (struct, all methods, event types), then mark as [x]._

- [x] 1.8. Implement ProfileRegistry
  - Files: `core/src/registry/profile.rs` (extend from 1.6)
  - Create ProfileRegistry struct with in-memory cache + disk storage
  - Implement save_profile() with atomic write (temp file + rename)
  - Implement get_profile(), delete_profile(), list_profiles()
  - Implement find_compatible_profiles(layout: &LayoutType)
  - Implement load_all_profiles() on initialization
  - Implement validate_profile() helper
  - _Leverage: tokio::fs for async I/O, serde_json_
  - _Requirements: 3_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Storage Systems Developer with expertise in atomic I/O and caching strategies | Task: Implement ProfileRegistry following requirement 3, managing persistent profile storage with in-memory cache | Restrictions: Must use atomic writes (temp + rename), validate profiles before save, cache for performance, handle corrupt files gracefully | _Leverage: Atomic write pattern from discovery/storage.rs, config paths from config/paths.rs | Success: All CRUD operations work, atomic writes prevent corruption, cache improves performance, validation catches errors, integration tests pass. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool including artifacts (registry struct, storage methods, validation functions), then mark as [x]._

- [x] 1.9. Implement DeviceBindings persistence
  - Files: `core/src/registry/bindings.rs`, `core/src/registry/mod.rs`
  - Create DeviceBindings struct with HashMap<DeviceIdentity, DeviceBinding>
  - Implement load() from device_bindings.json
  - Implement save() with atomic write
  - Implement get_binding(), set_binding(), remove_binding()
  - Handle missing/corrupted files gracefully
  - _Leverage: serde_json, atomic write pattern_
  - _Requirements: 6_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Data Persistence Developer with expertise in JSON serialization and file I/O | Task: Implement DeviceBindings persistence following requirement 6, storing device-profile assignments and labels | Restrictions: Must use atomic writes, handle corrupt files, create backups on corruption, default to empty bindings | _Leverage: Atomic write from registry/profile.rs, config paths | Success: Bindings persist across restarts, atomic writes work, corrupt file recovery works, unit tests pass. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool including artifacts (bindings struct, I/O methods), then mark as [x]._

- [x] 1.10. Write identity module tests
  - Files: `core/tests/identity_tests.rs`
  - Test DeviceIdentity to_key/from_key roundtrip
  - Test Windows path parsing with various formats
  - Test Linux synthetic ID generation stability
  - Test Hash/Eq implementations
  - _Leverage: proptest for property-based testing_
  - _Requirements: 1, 8, 9_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Test Engineer with expertise in unit testing and property-based testing | Task: Write comprehensive tests for identity module covering requirements 1, 8, 9, using property-based testing for robustness | Restrictions: Must test all edge cases, use proptest for fuzzing, platform-specific tests with #[cfg], mock OS calls | _Leverage: Existing test patterns from core/tests/ | Success: All identity functions tested, property tests pass, platform tests work, coverage >90%. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool, then mark as [x]._

- [x] 1.11. Write registry module tests
  - Files: `core/tests/registry_tests.rs`
  - Test DeviceRegistry CRUD and concurrent access
  - Test ProfileRegistry save/load/validation
  - Test DeviceBindings persistence
  - Test event emission
  - _Leverage: tokio test macros, temp directories_
  - _Requirements: 2, 3, 6_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Integration Test Engineer with expertise in async testing and concurrency | Task: Write comprehensive tests for registry modules covering requirements 2, 3, 6, including concurrent access scenarios | Restrictions: Must use tokio::test, test with temp directories, verify thread safety, test corruption recovery | _Leverage: Existing async test patterns from engine tests | Success: All registry operations tested, concurrent access verified safe, persistence tested, coverage >90%. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool, then mark as [x]._

## Phase 2: Device Definitions & Pipeline Integration (2 weeks)

- [x] 2.1. Define DeviceDefinition data model
  - Files: `core/src/definitions/types.rs`, `core/src/definitions/mod.rs`
  - Define DeviceDefinition struct matching TOML schema
  - Define LayoutDefinition, VisualMetadata structs
  - Add Deserialize for TOML parsing
  - Add validation methods
  - _Leverage: toml crate for deserialization_
  - _Requirements: 7_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Data Modeling Developer with expertise in TOML and schema design | Task: Define DeviceDefinition data model following requirement 7, matching TOML schema for device specifications | Restrictions: Must use TOML-compatible types, validate all required fields, support optional visual metadata | _Leverage: toml crate patterns | Success: Structs deserialize from TOML correctly, validation catches errors, all fields accessible. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool, then mark as [x]._

- [x] 2.2. Implement DeviceDefinitionLibrary
  - Files: `core/src/definitions/library.rs`, `core/src/definitions/mod.rs`
  - Create DeviceDefinitionLibrary with HashMap<(u16, u16), DeviceDefinition>
  - Implement load_from_directory() using walkdir
  - Implement load_definition() with TOML parsing
  - Implement validate_definition()
  - Implement find_definition(), list_definitions()
  - _Leverage: toml crate, walkdir for recursive search_
  - _Requirements: 7_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Systems Developer with expertise in file I/O and data indexing | Task: Implement DeviceDefinitionLibrary following requirement 7, loading and indexing TOML device definitions | Restrictions: Must recursively search directories, validate all definitions on load, skip invalid files with warnings, O(1) lookup by VID:PID | _Leverage: walkdir crate, toml parsing patterns | Success: Loads all .toml files recursively, VID:PID lookup works, validation catches errors, unit tests pass. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool, then mark as [x]._

- [x] 2.3. Create standard device definitions
  - Files: `device_definitions/standard/ansi-104.toml`, `device_definitions/standard/iso-105.toml`, `device_definitions/README.md`
  - Create ANSI 104-key keyboard definition with full scancode map
  - Create ISO 105-key keyboard definition
  - Write README with TOML format specification
  - _Leverage: Standard keyboard scancode references_
  - _Requirements: 7_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Hardware Specification Expert with knowledge of keyboard scancodes and layouts | Task: Create standard device definitions following requirement 7, documenting ANSI and ISO keyboard layouts with complete scancode mappings | Restrictions: Must use correct scancodes for each layout, document all 104/105 keys, include visual metadata | _Leverage: Standard keyboard layout references, existing scancode knowledge | Success: Definitions parse correctly, all keys mapped, README is comprehensive, validated against definition loader. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool, then mark as [x]._

- [x] 2.4. Create Stream Deck device definitions
  - Files: `device_definitions/elgato/stream-deck-mk2.toml`, `device_definitions/elgato/stream-deck-xl.toml`, `device_definitions/elgato/stream-deck-mini.toml`
  - Create Stream Deck MK.2 definition (3×5 matrix, VID:0x0fd9 PID:0x0080)
  - Create Stream Deck XL definition (4×8 matrix)
  - Create Stream Deck Mini definition (2×3 matrix)
  - _Leverage: Stream Deck HID specifications_
  - _Requirements: 7_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: USB/HID Developer with knowledge of Elgato Stream Deck protocols | Task: Create Stream Deck device definitions following requirement 7, documenting matrix layouts and HID usage mappings | Restrictions: Must use correct VID:PID, map all buttons to (row, col), include visual metadata (button size/spacing) | _Leverage: Stream Deck SDK documentation if available | Success: Definitions parse correctly, all buttons mapped, visual metadata accurate, tested with real devices if available. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool, then mark as [x]._

- [x] 2.5. Implement DeviceResolver
  - Files: `core/src/engine/device_resolver.rs`, `core/src/engine/mod.rs`
  - Create DeviceResolver with Arc<RwLock<DeviceRegistry>>
  - Implement resolve(device_handle) -> Result<Option<DeviceState>>
  - Implement extract_identity() with platform-specific calls
  - _Leverage: Platform drivers for handle conversion_
  - _Requirements: 10_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Pipeline Developer with expertise in async event processing | Task: Implement DeviceResolver following requirement 10, resolving OS device handles to DeviceState in <50μs | Restrictions: Must meet latency target, handle unknown devices gracefully, platform-specific identity extraction | _Leverage: Existing driver handle types, identity extraction from phase 1 | Success: Resolves handles correctly, meets latency target, handles errors, integration tests pass. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool, then mark as [x]._

- [x] 2.6. Implement ProfileResolver
  - Files: `core/src/engine/profile_resolver.rs`, `core/src/engine/mod.rs`
  - Create ProfileResolver with registry + cache
  - Implement resolve(profile_id) -> Result<Arc<Profile>> with caching
  - Implement invalidate_cache(profile_id)
  - _Leverage: Arc for zero-copy sharing, RwLock for cache_
  - _Requirements: 10_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Performance Engineer with expertise in caching strategies | Task: Implement ProfileResolver following requirement 10, caching profiles for <100μs lookup | Restrictions: Must use Arc<Profile> for zero-copy, RwLock for thread-safe cache, invalidate on update | _Leverage: Standard caching patterns | Success: Cache improves performance, cold load <10ms, warm load <100μs, invalidation works, tests pass. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool, then mark as [x]._

- [x] 2.7. Implement CoordinateTranslator
  - Files: `core/src/engine/coordinate_translator.rs`, `core/src/engine/mod.rs`
  - Create CoordinateTranslator with DeviceDefinitionLibrary + cache
  - Implement translate(device_identity, scancode) -> Result<PhysicalPosition>
  - Build and cache translation maps per device
  - _Leverage: DeviceDefinitionLibrary from phase 2.2_
  - _Requirements: 10_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Input Processing Developer with expertise in low-latency transformations | Task: Implement CoordinateTranslator following requirement 10, translating scancodes to (row, col) in <20μs | Restrictions: Must cache translation maps, meet latency target, handle unmapped scancodes gracefully | _Leverage: DeviceDefinitionLibrary, HashMap for O(1) lookup | Success: Translation is fast (<20μs), cache works, handles unknowns, tests pass. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool, then mark as [x]._

- [x] 2.8. Integrate pipeline into engine
  - Files: `core/src/engine/core.rs`
  - Add DeviceResolver, ProfileResolver, CoordinateTranslator to engine
  - Update process_input_event() to use new pipeline stages
  - Implement passthrough mode for disabled devices
  - Add fallback handling for errors at each stage
  - _Leverage: Existing engine event loop_
  - _Requirements: 10_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Systems Architect with expertise in event processing pipelines | Task: Integrate revolutionary mapping pipeline into engine following requirement 10, maintaining <1ms total latency | Restrictions: Must preserve existing functionality, handle all error cases, passthrough mode must be fast, no breaking changes | _Leverage: Existing engine/core.rs event processing | Success: Pipeline integrated, latency target met, passthrough works, all existing tests still pass, integration tests pass. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool including artifacts (pipeline stages, integration points), then mark as [x]._

- [x] 2.9. Write definition library tests
  - Files: `core/tests/definition_tests.rs`
  - Test TOML parsing with sample definitions
  - Test validation catches errors
  - Test VID:PID lookup
  - Test recursive directory loading
  - _Leverage: temp directories, sample TOML files_
  - _Requirements: 7_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Test Engineer with expertise in parsing and validation testing | Task: Write tests for definition library covering requirement 7, including TOML parsing and validation | Restrictions: Must test valid and invalid TOML, verify all validation rules, test directory recursion | _Leverage: Existing test utilities | Success: All definition loading tested, validation tested, lookup tested, coverage >90%. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool, then mark as [x]._

- [x] 2.10. Write pipeline integration tests
  - Files: `core/tests/pipeline_integration_tests.rs`
  - Test full pipeline: raw event → output
  - Test passthrough mode
  - Test error handling at each stage
  - Test latency benchmarks
  - _Leverage: criterion for benchmarking, mock devices_
  - _Requirements: 10_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Performance Test Engineer with expertise in benchmarking and integration testing | Task: Write integration tests for pipeline covering requirement 10, including latency benchmarks | Restrictions: Must test end-to-end flow, verify latency targets, test all error paths, use criterion for benchmarks | _Leverage: Existing benchmark patterns from benches/, mock device utilities | Success: E2E tests pass, latency benchmarks meet targets (<1ms p99), error handling tested, coverage complete. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool, then mark as [x]._

## Phase 3: FFI Layer & Flutter Integration (1.5 weeks)

- [x] 3.1. Implement device_registry FFI domain
  - Files: `core/src/ffi/domains/device_registry.rs`, `core/src/ffi/domains/mod.rs`
  - Implement krx_device_registry_list_devices() -> JSON
  - Implement krx_device_registry_set_remap_enabled()
  - Implement krx_device_registry_assign_profile()
  - Implement krx_device_registry_set_user_label()
  - Add panic guards on all extern "C" functions
  - _Leverage: Existing FFI utilities, panic guards_
  - _Requirements: 16_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: FFI Developer with expertise in C ABI and panic safety | Task: Implement device_registry FFI domain following requirement 16, exposing device registry to Flutter | Restrictions: Must use panic guards, validate all pointers, return errors as strings, free memory correctly | _Leverage: Existing FFI patterns from ffi/domains/device.rs, ffi/utils.rs panic guards | Success: All FFI functions work, no panics, memory safety verified, tested from Dart. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool including artifacts (all FFI functions, error handling), then mark as [x]._

- [x] 3.2. Implement profile_registry FFI domain
  - Files: `core/src/ffi/domains/profile_registry.rs`, `core/src/ffi/domains/mod.rs`
  - Implement krx_profile_registry_list_profiles()
  - Implement krx_profile_registry_get_profile()
  - Implement krx_profile_registry_save_profile()
  - Implement krx_profile_registry_delete_profile()
  - Implement krx_profile_registry_find_compatible_profiles()
  - Add panic guards and validation
  - _Leverage: Existing FFI utilities_
  - _Requirements: 17_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: FFI Developer with expertise in JSON marshalling and error handling | Task: Implement profile_registry FFI domain following requirement 17, exposing profile operations to Flutter | Restrictions: Must handle JSON parsing errors, validate profile data, use panic guards, proper memory management | _Leverage: FFI panic guards, JSON serialization patterns | Success: All FFI functions work, JSON parsing safe, errors handled, tested from Dart. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool, then mark as [x]._

- [x] 3.3. Implement device_definitions FFI domain
  - Files: `core/src/ffi/domains/device_definitions.rs`, `core/src/ffi/domains/mod.rs`
  - Implement krx_definitions_list_all()
  - Implement krx_definitions_get_for_device(vid, pid)
  - Add panic guards
  - _Leverage: DeviceDefinitionLibrary_
  - _Requirements: 7_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: FFI Developer with expertise in read-only data exposure | Task: Implement device_definitions FFI domain following requirement 7, exposing device definitions to Flutter | Restrictions: Must return read-only data, handle missing definitions, use panic guards | _Leverage: FFI panic guards, existing definition library | Success: FFI functions return definitions correctly, handle unknowns, tested from Dart. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool, then mark as [x]._

- [x] 3.4. Write FFI tests
  - Files: `core/tests/ffi_tests.rs`
  - Test all FFI functions with valid/invalid inputs
  - Test panic safety
  - Test memory leaks
  - Test null pointer handling
  - _Leverage: Existing FFI test patterns_
  - _Requirements: 16, 17_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Safety Engineer with expertise in FFI testing and memory safety | Task: Write comprehensive FFI tests covering requirements 16 and 17, including panic and memory safety | Restrictions: Must test all error paths, verify no panics cross FFI boundary, check for memory leaks, test null pointers | _Leverage: Existing FFI test patterns from ffi/ tests | Success: All FFI functions tested, panic safety verified, no memory leaks, null pointer tests pass. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool, then mark as [x]._

- [x] 3.5. Create Dart data models
  - Files: `ui/lib/models/device_identity.dart`, `ui/lib/models/device_state.dart`, `ui/lib/models/profile.dart`, `ui/lib/models/layout_type.dart`
  - Define DeviceIdentity, DeviceState, Profile, LayoutType Dart classes
  - Add fromJson/toJson methods
  - Add freezed annotations for immutability
  - _Leverage: freezed, json_serializable packages_
  - _Requirements: 1, 2, 3_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer with expertise in data modeling and serialization | Task: Create Dart data models following requirements 1, 2, 3, using freezed for immutability and json_serializable | Restrictions: Must match Rust types exactly, use freezed for immutability, handle nullability correctly | _Leverage: Existing model patterns, freezed package | Success: Models match Rust types, serialize correctly, freezed generates code, tests pass. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool, then mark as [x]._

- [x] 3.6. Create DeviceRegistryFFI bindings
  - Files: `ui/lib/ffi/device_registry_ffi.dart`
  - Lookup all FFI functions from shared library
  - Create type-safe Dart wrappers
  - Handle pointer conversion and memory management
  - Parse JSON results and errors
  - _Leverage: dart:ffi, ffi package_
  - _Requirements: 16_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Dart FFI Developer with expertise in native bindings | Task: Create DeviceRegistryFFI Dart bindings following requirement 16, wrapping Rust FFI functions | Restrictions: Must use dart:ffi correctly, free C strings, handle errors, type-safe API | _Leverage: Existing FFI binding patterns from ffi/ | Success: All FFI functions wrapped, memory managed correctly, errors handled, tested. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool, then mark as [x]._

- [x] 3.7. Create ProfileRegistryFFI bindings
  - Files: `ui/lib/ffi/profile_registry_ffi.dart`
  - Lookup FFI functions
  - Create Dart wrappers with type safety
  - Handle JSON marshalling
  - _Leverage: dart:ffi_
  - _Requirements: 17_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Dart FFI Developer | Task: Create ProfileRegistryFFI bindings following requirement 17, wrapping profile operations | Restrictions: Must handle JSON correctly, free memory, error handling, type safety | _Leverage: FFI patterns from 3.6 | Success: All functions wrapped, JSON parsing works, tested. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool, then mark as [x]._

- [x] 3.8. Create DeviceRegistryService
  - Files: `ui/lib/services/device_registry_service.dart`
  - Wrap DeviceRegistryFFI with high-level async API
  - Implement getDevices(), toggleRemap(), assignProfile(), setUserLabel()
  - Add error handling with user-friendly messages
  - _Leverage: DeviceRegistryFFI from 3.6_
  - _Requirements: 16_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Service Layer Developer | Task: Create DeviceRegistryService following requirement 16, wrapping FFI with high-level API | Restrictions: Must provide async API, convert errors to user-friendly messages, follow service patterns | _Leverage: Existing service patterns from services/ | Success: Service API is clean, async methods work, errors are user-friendly, tested. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool, then mark as [x]._

- [x] 3.9. Create ProfileRegistryService
  - Files: `ui/lib/services/profile_registry_service.dart`
  - Wrap ProfileRegistryFFI with async API
  - Implement listProfiles(), getProfile(), saveProfile(), deleteProfile()
  - Add error handling
  - _Leverage: ProfileRegistryFFI_
  - _Requirements: 17_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Service Layer Developer | Task: Create ProfileRegistryService following requirement 17, providing high-level profile operations | Restrictions: Must provide async API, handle errors, follow patterns | _Leverage: Service patterns | Success: Service works, all operations functional, errors handled, tested. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool, then mark as [x]._

## Phase 4: UI Implementation - Devices Tab (1.5 weeks)

- [x] 4.1. Create RemapToggle widget
  - Files: `ui/lib/widgets/remap_toggle.dart`
  - Create stateless widget with Switch
  - Show ON/OFF label with colors
  - Handle onChanged callback
  - _Leverage: Material widgets_
  - _Requirements: 12_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Widget Developer | Task: Create RemapToggle widget following requirement 12, providing visual remap state control | Restrictions: Must use Material Switch, show clear ON/OFF state, accessible, responsive | _Leverage: Material widgets | Success: Widget displays correctly, toggle works, accessible, widget tests pass. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool including artifacts (widget class, props), then mark as [x]._

- [x] 4.2. Create ProfileSelector widget
  - Files: `ui/lib/widgets/profile_selector.dart`
  - Create DropdownButton with FutureBuilder
  - Load profiles from ProfileRegistryService
  - Show current selection and handle changes
  - _Leverage: ProfileRegistryService, Material widgets_
  - _Requirements: 12_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Widget Developer with async UI expertise | Task: Create ProfileSelector widget following requirement 12, showing profile dropdown with async loading | Restrictions: Must handle loading state, show selected profile, call service correctly | _Leverage: FutureBuilder pattern, ProfileRegistryService | Success: Dropdown loads profiles, shows selection, works correctly, widget tests pass. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool, then mark as [x]._

- [x] 4.3. Create DeviceCard widget
  - Files: `ui/lib/widgets/device_card.dart`
  - Create Card widget showing device info (VID:PID:Serial, label)
  - Include ProfileSelector and RemapToggle
  - Add Edit Label and Manage Profiles buttons
  - _Leverage: RemapToggle, ProfileSelector from 4.1, 4.2_
  - _Requirements: 12_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Component Developer | Task: Create DeviceCard widget following requirement 12, composing device controls into card layout | Restrictions: Must show all device info clearly, compose sub-widgets, handle callbacks, responsive layout | _Leverage: Material Card, existing widgets | Success: Card displays beautifully, all controls work, layout responsive, widget tests pass. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool including artifacts (widget, layout structure), then mark as [x]._

- [x] 4.4. Rebuild DevicesPage
  - Files: `ui/lib/pages/devices_page.dart`
  - Use FutureBuilder to load devices from DeviceRegistryService
  - Display list of DeviceCard widgets
  - Show empty state when no devices
  - Add refresh button
  - Implement edit label dialog
  - _Leverage: DeviceRegistryService, DeviceCard_
  - _Requirements: 11, 12_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Page Developer | Task: Rebuild DevicesPage following requirements 11 and 12, showing all connected devices with controls | Restrictions: Must handle loading/empty/error states, refresh works, dialogs are user-friendly | _Leverage: Existing page patterns, DeviceRegistryService, DeviceCard widget | Success: Page shows devices correctly, all interactions work, empty state clear, tested. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool including artifacts (page class, state management), then mark as [x]._

- [x] 4.5. Reorder navigation
  - Files: `ui/lib/main.dart`
  - Move Devices destination above Editor in NavigationRail
  - Set Devices as default landing page (index 0)
  - Update navigation indexes
  - _Leverage: Existing navigation structure_
  - _Requirements: 11_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Navigation Developer | Task: Reorder navigation following requirement 11, making Devices the first tab | Restrictions: Must preserve all other nav items, update indexes correctly, default to Devices | _Leverage: Existing main.dart navigation | Success: Navigation order correct, Devices is default, all tabs work, no broken links. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool, then mark as [x]._

- [x] 4.6. Write Devices UI tests
  - Files: `ui/test/pages/devices_page_test.dart`, `ui/test/widgets/device_card_test.dart`
  - Test DevicesPage with mock service
  - Test DeviceCard widget interactions
  - Test RemapToggle and ProfileSelector
  - _Leverage: flutter_test, mockito_
  - _Requirements: 11, 12_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Test Engineer | Task: Write UI tests for Devices tab following requirements 11 and 12, using widget testing and mocks | Restrictions: Must test all user interactions, mock services, verify UI updates, test empty/error states | _Leverage: flutter_test, mockito for mocks | Success: All widgets tested, interactions verified, good coverage, tests reliable. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool, then mark as [x]._

## Phase 5: UI Implementation - Visual Editor (2 weeks)

- [x] 5.1. Create LayoutGrid widget
  - Files: `ui/lib/widgets/layout_grid.dart`
  - Create widget that renders layouts dynamically (Matrix, Standard, Split)
  - Use GridView for Matrix layouts
  - Show current mappings on keys
  - Handle key tap events
  - _Leverage: Flutter GridView, existing layout concepts_
  - _Requirements: 13_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Layout Developer with custom rendering expertise | Task: Create LayoutGrid widget following requirement 13, dynamically rendering device layouts | Restrictions: Must support Matrix/Standard/Split layouts, show mappings, clickable keys, responsive | _Leverage: GridView for Matrix, existing keyboard layout concepts | Success: All layout types render correctly, mappings shown, clicks work, widget tests pass. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool including artifacts (widget, layout rendering logic), then mark as [x]._

- [x] 5.2. Create SoftKeyboard widget
  - Files: `ui/lib/widgets/soft_keyboard.dart`
  - Create palette showing all KeyCode values
  - Add search/filter functionality
  - Display keys in grid
  - Handle key selection
  - _Leverage: Existing KeyCode knowledge_
  - _Requirements: 14_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Widget Developer | Task: Create SoftKeyboard palette widget following requirement 14, showing all output keys with search | Restrictions: Must show all keycodes, search works, keys selectable, performant with many keys | _Leverage: GridView, TextField for search | Success: Palette shows all keys, search filters correctly, selection works, tests pass. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool, then mark as [x]._

- [x] 5.3. Create DragDropMapper widget
  - Files: `ui/lib/widgets/drag_drop_mapper.dart`
  - Compose LayoutGrid and SoftKeyboard in split view
  - Implement mapping workflow (select physical → select output)
  - Auto-save profile on mapping changes
  - _Leverage: LayoutGrid, SoftKeyboard, ProfileRegistryService_
  - _Requirements: 13, 14_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter State Management Developer | Task: Create DragDropMapper widget following requirements 13 and 14, coordinating mapping workflow | Restrictions: Must coordinate two-step selection, auto-save works, state management clean, UX intuitive | _Leverage: LayoutGrid, SoftKeyboard widgets, ProfileRegistryService | Success: Mapping workflow intuitive, auto-save works, state consistent, tested. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool including artifacts (widget, state management, integration), then mark as [x]._

- [x] 5.4. Rebuild VisualEditorPage
  - Files: `ui/lib/pages/visual_editor_page.dart`
  - Add profile selector dropdown at top
  - Load profile and display in DragDropMapper
  - Add create new profile button
  - Handle profile save/update
  - _Leverage: DragDropMapper, ProfileRegistryService_
  - _Requirements: 13_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Page Developer | Task: Rebuild VisualEditorPage following requirement 13, enabling profile-based editing with dynamic layouts | Restrictions: Must load profiles correctly, create new profiles, save works, profile selector functional | _Leverage: DragDropMapper widget, ProfileRegistryService | Success: Editor loads profiles, dynamic layouts render, editing works, save persists, tested. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool including artifacts (page structure, profile management), then mark as [x]._

- [-] 5.5. Write Visual Editor tests
  - Files: `ui/test/pages/visual_editor_page_test.dart`, `ui/test/widgets/layout_grid_test.dart`, `ui/test/widgets/soft_keyboard_test.dart`
  - Test LayoutGrid rendering for different layouts
  - Test SoftKeyboard filtering and selection
  - Test DragDropMapper mapping workflow
  - Test VisualEditorPage profile management
  - _Leverage: flutter_test, mockito_
  - _Requirements: 13, 14_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Test Engineer | Task: Write comprehensive tests for Visual Editor following requirements 13 and 14, testing all widgets and workflows | Restrictions: Must test all layout types, mapping workflows, profile operations, mock services | _Leverage: flutter_test, widget testing | Success: All editor components tested, workflows verified, good coverage, tests reliable. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool, then mark as [x]._

## Phase 6: Migration & Polish (1 week)

- [ ] 6.1. Implement migration module
  - Files: `core/src/migration/mod.rs`, `core/src/migration/v1_to_v2.rs`
  - Create MigrationV1ToV2 struct
  - Implement migrate() method to scan and convert old profiles
  - Implement convert_profile() to transform old DeviceProfile to new Profile
  - Create backup before migration
  - Generate migration report
  - _Leverage: Existing old types from discovery/types.rs_
  - _Requirements: 15_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Migration Engineer with expertise in data transformation | Task: Implement migration from old to new system following requirement 15, converting profiles safely | Restrictions: Must create backups, handle errors gracefully, partial migration allowed, report results | _Leverage: Old DeviceProfile from discovery/types.rs | Success: Migration converts profiles correctly, backups created, errors handled, report accurate, tested. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool including artifacts (migration struct, conversion logic), then mark as [x]._

- [ ] 6.2. Add migration CLI command
  - Files: `core/src/bin/keyrx.rs`
  - Add `keyrx migrate --from v1 --backup` command
  - Call migration module
  - Display migration report
  - _Leverage: Existing CLI structure_
  - _Requirements: 15_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: CLI Developer | Task: Add migration CLI command following requirement 15, exposing migration to users | Restrictions: Must show progress, display report, handle errors, support --backup flag | _Leverage: Existing clap CLI from bin/keyrx.rs | Success: Command works, migration runs, report shows, tested manually. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool, then mark as [x]._

- [ ] 6.3. Create migration prompt UI
  - Files: `ui/lib/pages/migration_prompt_page.dart`
  - Show migration dialog on first run with new version
  - Explain what migration does
  - Allow user to accept/skip
  - Show migration progress
  - Display results
  - _Leverage: Migration FFI (if needed), dialogs_
  - _Requirements: 15_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter UI Developer | Task: Create migration prompt UI following requirement 15, guiding users through migration | Restrictions: Must show clear explanation, progress indicator, results summary, allow skip | _Leverage: Material dialogs, version detection | Success: Dialog appears on first run, migration works, results shown, UX clear, tested. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool, then mark as [x]._

- [ ] 6.4. Write migration tests
  - Files: `core/tests/migration_tests.rs`
  - Test migration with sample old data
  - Test backup creation
  - Test partial migration (some failures)
  - Test idempotency (run twice)
  - _Leverage: Temp directories, sample old profiles_
  - _Requirements: 15_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Test Engineer | Task: Write migration tests following requirement 15, ensuring safe data transformation | Restrictions: Must test all scenarios, verify backups, test errors, idempotency critical | _Leverage: Test utilities, temp dirs | Success: Migration tested thoroughly, backups verified, errors handled, idempotent, coverage good. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool, then mark as [x]._

- [ ] 6.5. Write end-to-end integration tests
  - Files: `core/tests/e2e_revolutionary_mapping.rs`
  - Test: Connect 2 identical devices → assign different profiles → verify isolation
  - Test: Swap profile on device → verify behavior changes
  - Test: Toggle remap per device → verify passthrough/active
  - Test: Disconnect/reconnect → verify binding persists
  - Test: 5×5 macro pad → create profile → verify layout
  - _Leverage: Mock devices, test engine setup_
  - _Requirements: All_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Integration Test Engineer | Task: Write end-to-end tests covering all requirements, simulating real user scenarios | Restrictions: Must test complete workflows, use mock devices, verify all requirements, realistic scenarios | _Leverage: Test harness, mock devices | Success: All E2E scenarios pass, requirements verified, realistic tests, good coverage. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool, then mark as [x]._

- [ ] 6.6. Performance benchmarking
  - Files: `core/benches/revolutionary_mapping_bench.rs`
  - Benchmark device resolution (<50μs target)
  - Benchmark profile lookup (<100μs cached target)
  - Benchmark coordinate translation (<20μs target)
  - Benchmark full pipeline (<1ms target)
  - _Leverage: criterion crate_
  - _Requirements: 10_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Performance Engineer | Task: Create performance benchmarks following requirement 10, verifying latency targets | Restrictions: Must use criterion, measure p99 latency, verify all targets met, realistic scenarios | _Leverage: criterion patterns from benches/ | Success: All benchmarks created, targets met, results documented, CI integrated. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool, then mark as [x]._

- [ ] 6.7. Documentation updates
  - Files: `docs/revolutionary-mapping-guide.md`, `device_definitions/README.md`, `README.md`
  - Write user guide for revolutionary mapping feature
  - Document device definition format
  - Update main README with new capabilities
  - Add migration guide
  - _Leverage: Existing documentation patterns_
  - _Requirements: All_
  - _Prompt: Implement the task for spec revolutionary-mapping, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical Writer | Task: Create comprehensive documentation for revolutionary mapping, covering user guide, device definitions, and migration | Restrictions: Must be user-friendly, include examples, cover all features, migration steps clear | _Leverage: Existing docs structure | Success: Documentation complete, clear, accurate, examples work, reviewed. After completing, update tasks.md to mark this task as [-], log implementation with log-implementation tool, then mark as [x]._

