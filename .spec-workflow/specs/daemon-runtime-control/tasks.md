# Tasks: Daemon Runtime Control

- [x] 1. Implement engine.start IPC handler in core/src/daemon/ipc.rs
  - File: core/src/daemon/ipc.rs
  - Add `METHOD_ENGINE_START` constant
  - Implement request handler parsing `EngineStartParams`
  - Call `manager.start()` and return JSON-RPC response
  - Purpose: Enable starting the engine via IPC
  - _Leverage: core/src/daemon/protocol.rs, core/src/daemon/manager.rs_
  - _Requirements: 2.1 (Start Engine)_
  - _Prompt: Role: Rust Backend Developer | Task: Implement the `engine.start` JSON-RPC handler in `core/src/daemon/ipc.rs`. It must parse parameters, call `manager.start`, and return a standard `JsonRpcResponse` with status. | Restrictions: Must use existing JSON-RPC types from protocol.rs, handle errors gracefully returning JsonRpcError | Success: Handler compiles, correctly parses input, calls manager, and returns valid JSON response_

- [x] 2. Implement engine.stop IPC handler in core/src/daemon/ipc.rs
  - File: core/src/daemon/ipc.rs
  - Add `METHOD_ENGINE_STOP` constant
  - Implement request handler calling `manager.stop()`
  - Return JSON-RPC response
  - Purpose: Enable stopping the engine via IPC
  - _Leverage: core/src/daemon/protocol.rs, core/src/daemon/manager.rs_
  - _Requirements: 2.1 (Stop Engine)_
  - _Prompt: Role: Rust Backend Developer | Task: Implement the `engine.stop` JSON-RPC handler in `core/src/daemon/ipc.rs`. It must call `manager.stop` and return a standard `JsonRpcResponse`. | Restrictions: Handle case where engine is not running gracefully | Success: Handler stops engine and returns success status_

- [-] 3. Implement EngineManager status method in core/src/daemon/manager.rs
  - File: core/src/daemon/manager.rs
  - Implement `status()` method returning `EngineStatus` struct
  - Populate state, uptime, and connected devices list
  - Purpose: Provide visibility into daemon state
  - _Leverage: core/src/daemon/protocol.rs_
  - _Requirements: 2.1 (Status Query)_
  - _Prompt: Role: Rust Systems Engineer | Task: Implement `status()` method in `EngineManager` struct in `core/src/daemon/manager.rs`. It should return a populated `EngineStatus` object including uptime and device list. | Restrictions: Thread-safe state access, accurate uptime calculation | Success: Method returns correct status reflecting internal state_

- [ ] 4. Update DaemonService in ui/lib/services/daemon_service.dart
  - File: ui/lib/services/daemon_service.dart
  - Ensure `startEngine`, `stopEngine`, `getStatus` send correct IPC methods
  - Add error handling and response parsing
  - Purpose: Client-side implementation of the protocol
  - _Leverage: ui/lib/models/daemon_protocol.dart_
  - _Requirements: 2.1 (UI Integration)_
  - _Prompt: Role: Flutter Developer | Task: Update `DaemonService` in `ui/lib/services/daemon_service.dart` to fully implement `startEngine`, `stopEngine`, and `getStatus` using the `sendRequest` helper. | Restrictions: Must match JSON-RPC method names defined in Core exactly | Success: Methods correctly serialize params and deserialize responses_

- [ ] 5. Refactor RunControlsPage in ui/lib/pages/run_controls_page.dart
  - File: ui/lib/pages/run_controls_page.dart
  - Replace `KeyrxFacade` calls with `DaemonService` calls
  - Use `daemonConnectionStateProvider` for UI state
  - Remove local `_runState` if redundant
  - Purpose: Connect UI to new Daemon architecture
  - _Leverage: ui/lib/state/daemon_providers.dart_
  - _Requirements: 2.1 (UI Integration)_
  - _Prompt: Role: Flutter UI Developer | Task: Refactor `RunControlsPage` to use `DaemonService` via Riverpod providers instead of `KeyrxFacade`. Replace Start/Stop button logic. | Restrictions: Maintain existing UI layout, ensure responsive feedback during async calls | Success: UI successfully controls daemon and reflects state changes_