# Tasks Document: Flutter UI Fixes

## Phase 1: Fix Failing Tests

- [x] 1. Create FakeDeviceService for tests
  - File: `test/helpers/fake_services.dart`
  - Create `_FakeDeviceService` implementing `DeviceService`
  - Return empty list for `listDevices()`
  - Return success for `selectDevice()`
  - Purpose: Provide mock device service for all tests
  - _Requirements: REQ-1_
  - _Prompt: Implement the task for spec flutter-ui-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Test Developer | Task: Create test/helpers/fake_services.dart with FakeDeviceService class implementing DeviceService interface. Add FakeEngineService, FakeAudioService if not already centralized. | Restrictions: Keep implementations minimal, return success/empty values | Success: Helper file created and importable. Mark [-] before, log after, mark [x] complete._

- [x] 2. Fix keyrx_training_screen_test.dart
  - File: `test/keyrx_training_screen_test.dart`
  - Add `deviceService` parameter to `ServiceRegistry.withOverrides()`
  - Import fake services helper
  - Purpose: Fix compilation error
  - _Requirements: REQ-1_
  - _Prompt: Implement the task for spec flutter-ui-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Test Developer | Task: Fix test/keyrx_training_screen_test.dart by adding deviceService parameter to ServiceRegistry.withOverrides() call around line 100. Import FakeDeviceService from helpers. | Restrictions: Only fix the parameter issue, don't change test logic | Success: Test compiles and passes. Mark [-] before, log after, mark [x] complete._

- [x] 3. Fix trade_off_test.dart
  - File: `test/trade_off_test.dart`
  - Add `deviceService` parameter to `ServiceRegistry.withOverrides()`
  - Purpose: Fix compilation error
  - _Requirements: REQ-1_
  - _Prompt: Implement the task for spec flutter-ui-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Test Developer | Task: Fix test/trade_off_test.dart by adding deviceService parameter to ServiceRegistry.withOverrides() call around line 98. | Restrictions: Only fix the parameter issue | Success: Test compiles and passes. Mark [-] before, log after, mark [x] complete._

- [x] 4. Fix debugger_page_test.dart
  - File: `test/debugger_page_test.dart`
  - Add `deviceService` parameter to `ServiceRegistry.withOverrides()`
  - Purpose: Fix compilation error
  - _Requirements: REQ-1_
  - _Prompt: Implement the task for spec flutter-ui-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Test Developer | Task: Fix test/debugger_page_test.dart by adding deviceService parameter to ServiceRegistry.withOverrides(). | Restrictions: Only fix the parameter issue | Success: Test compiles and passes. Mark [-] before, log after, mark [x] complete._

- [x] 5. Fix training_screen_test.dart
  - File: `test/pages/training_screen_test.dart`
  - Add `deviceService` parameter to `ServiceRegistry.withOverrides()`
  - Purpose: Fix compilation error
  - _Requirements: REQ-1_
  - _Prompt: Implement the task for spec flutter-ui-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Test Developer | Task: Fix test/pages/training_screen_test.dart by adding deviceService parameter to ServiceRegistry.withOverrides(). | Restrictions: Only fix the parameter issue | Success: Test compiles and passes. Mark [-] before, log after, mark [x] complete._

- [x] 6. Fix any remaining test failures
  - Files: Any tests still failing after above fixes
  - Check for other missing required parameters
  - Purpose: Ensure all tests pass
  - _Requirements: REQ-1_
  - _Prompt: Implement the task for spec flutter-ui-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Test Developer | Task: Run `flutter test` and fix any remaining compilation errors or test failures. Check all ServiceRegistry.withOverrides() calls have required parameters. | Restrictions: Only fix issues, don't change test assertions | Success: `flutter test` reports 0 failures. Mark [-] before, log after, mark [x] complete._

## Phase 2: FFI Bridge Refactoring

- [x] 7. Create bridge_core.dart
  - File: `lib/ffi/bridge_core.dart`
  - Extract: `_init()`, `version` getter, `dispose()`, `_freeString()`, initialization logic
  - Purpose: Isolate core FFI initialization
  - _Requirements: REQ-2_
  - _Prompt: Implement the task for spec flutter-ui-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter FFI Developer | Task: Create lib/ffi/bridge_core.dart by extracting initialization, version, dispose, and string management functions from bridge.dart. Create a mixin or extension that KeyrxBridge can use. | Restrictions: Keep function signatures identical | Success: Core functions work when imported. Mark [-] before, log after, mark [x] complete._

- [x] 8. Create bridge_engine.dart
  - File: `lib/ffi/bridge_engine.dart`
  - Extract: `loadScript()`, `eval()`, `listKeys()`, `stateStream`, state-related methods
  - Purpose: Isolate engine control FFI methods
  - _Requirements: REQ-2_
  - _Prompt: Implement the task for spec flutter-ui-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter FFI Developer | Task: Create lib/ffi/bridge_engine.dart by extracting engine control methods (loadScript, eval, listKeys, isBypassActive, setBypass) and stateStream from bridge.dart. | Restrictions: Maintain stream controller patterns | Success: Engine methods work correctly. Mark [-] before, log after, mark [x] complete._

- [x] 9. Create bridge_audio.dart
  - File: `lib/ffi/bridge_audio.dart`
  - Extract: `startAudio()`, `stopAudio()`, `setBpm()`, classification stream
  - Purpose: Isolate audio capture FFI methods
  - _Requirements: REQ-2_
  - _Prompt: Implement the task for spec flutter-ui-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter FFI Developer | Task: Create lib/ffi/bridge_audio.dart by extracting audio methods (startAudio, stopAudio, setBpm) and classificationStream from bridge.dart. | Restrictions: Maintain stream controller patterns | Success: Audio methods work correctly. Mark [-] before, log after, mark [x] complete._

- [x] 10. Create bridge_session.dart
  - File: `lib/ffi/bridge_session.dart`
  - Extract: `startRecording()`, `stopRecording()`, `listSessions()`, `analyzeSession()`, `replaySession()`
  - Purpose: Isolate session/recording FFI methods
  - _Requirements: REQ-2_
  - _Prompt: Implement the task for spec flutter-ui-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter FFI Developer | Task: Create lib/ffi/bridge_session.dart by extracting session/recording methods and related result types from bridge.dart. | Restrictions: Keep result types with their methods | Success: Session methods work correctly. Mark [-] before, log after, mark [x] complete._

- [x] 11. Create bridge_discovery.dart
  - File: `lib/ffi/bridge_discovery.dart`
  - Extract: `startDiscovery()`, `processDiscoveryEvent()`, `cancelDiscovery()`, `getDiscoveryProgress()`
  - Purpose: Isolate device discovery FFI methods
  - _Requirements: REQ-2_
  - _Prompt: Implement the task for spec flutter-ui-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter FFI Developer | Task: Create lib/ffi/bridge_discovery.dart by extracting discovery methods from bridge.dart. | Restrictions: Maintain callback patterns | Success: Discovery methods work correctly. Mark [-] before, log after, mark [x] complete._

- [x] 12. Create bridge_testing.dart
  - File: `lib/ffi/bridge_testing.dart`
  - Extract: `simulate()`, `runTests()`, `discoverTests()`, `runBenchmark()`, `runDoctor()`
  - Purpose: Isolate testing/diagnostic FFI methods
  - _Requirements: REQ-2_
  - _Prompt: Implement the task for spec flutter-ui-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter FFI Developer | Task: Create lib/ffi/bridge_testing.dart by extracting testing and diagnostic methods from bridge.dart. | Restrictions: Keep result types with their methods | Success: Testing methods work correctly. Mark [-] before, log after, mark [x] complete._

- [x] 13. Update bridge.dart with re-exports
  - File: `lib/ffi/bridge.dart`
  - Import all bridge_*.dart modules
  - Compose KeyrxBridge class using mixins/parts
  - Maintain public API through re-exports
  - Purpose: Complete bridge refactoring
  - _Requirements: REQ-2_
  - _Prompt: Implement the task for spec flutter-ui-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer | Task: Update lib/ffi/bridge.dart to import and compose all bridge_* modules. Use part/part of or mixins to maintain KeyrxBridge class. Verify file is under 500 lines. | Restrictions: All existing public APIs must remain accessible | Success: `flutter analyze` passes, bridge.dart under 500 lines. Mark [-] before, log after, mark [x] complete._

## Phase 3: Page/Widget Refactoring

- [x] 14. Create run_controls_widgets.dart
  - File: `lib/pages/run_controls_widgets.dart`
  - Extract: `_StatusIndicator`, card builder methods
  - Purpose: Reduce run_controls_page.dart below 500 lines
  - _Requirements: REQ-3_
  - _Prompt: Implement the task for spec flutter-ui-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer | Task: Create lib/pages/run_controls_widgets.dart by extracting _StatusIndicator widget and card builder widgets from run_controls_page.dart. Make _StatusIndicator public as StatusIndicator. | Restrictions: Keep widget APIs compatible | Success: Both files under 500 lines. Mark [-] before, log after, mark [x] complete._

- [x] 15. Create visual_keyboard_keys.dart
  - File: `lib/widgets/visual_keyboard_keys.dart`
  - Extract: Key rendering helpers, key layout calculations
  - Purpose: Reduce visual_keyboard.dart below 500 lines
  - _Requirements: REQ-3_
  - _Prompt: Implement the task for spec flutter-ui-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer | Task: Create lib/widgets/visual_keyboard_keys.dart by extracting key rendering helper methods and layout calculations from visual_keyboard.dart. | Restrictions: Keep widget APIs compatible | Success: Both files under 500 lines. Mark [-] before, log after, mark [x] complete._

- [x] 16. Refactor editor_page.dart if needed
  - File: `lib/pages/editor_page.dart`
  - Move remaining helper methods to editor_widgets.dart
  - Purpose: Reduce editor_page.dart below 500 lines
  - _Requirements: REQ-3_
  - _Prompt: Implement the task for spec flutter-ui-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer | Task: Check if lib/pages/editor_page.dart exceeds 500 lines after previous changes. If so, extract more helpers to editor_widgets.dart. | Restrictions: Keep widget APIs compatible | Success: File under 500 lines or already compliant. Mark [-] before, log after, mark [x] complete._

## Phase 4: Verification

- [x] 17. Run full test suite
  - Run: `flutter test` - verify 0 failures
  - Run: `flutter analyze` - verify 0 errors
  - Purpose: Ensure all tests pass after refactoring
  - _Requirements: REQ-4_
  - _Prompt: Implement the task for spec flutter-ui-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer | Task: Run `flutter test` and `flutter analyze`. Fix any failures or errors introduced by refactoring. | Restrictions: Do not skip tests | Success: All tests pass, no analyzer errors. Mark [-] before, log after, mark [x] complete._

- [ ] 18. Verify file sizes
  - Run: `find lib -name "*.dart" -exec wc -l {} \; | awk '$1 > 500'`
  - Verify no files exceed 500 lines
  - Purpose: Final verification of code quality metrics
  - _Requirements: REQ-4_
  - _Prompt: Implement the task for spec flutter-ui-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer | Task: Verify all Dart files in lib/ are under 500 lines. List any violations and create follow-up tasks if needed. | Restrictions: All lib files must comply | Success: No files over 500 lines. Mark [-] before, log after, mark [x] complete._
