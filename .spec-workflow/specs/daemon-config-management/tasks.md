# Tasks: Daemon Config Management

- [ ] 1. Implement config.reload IPC handler in core/src/daemon/ipc.rs
  - File: core/src/daemon/ipc.rs
  - Add `METHOD_CONFIG_RELOAD` constant
  - Parse `EngineReloadParams`
  - Call `manager.reload()`
  - Purpose: Allow UI to trigger hot-reload of settings
  - _Leverage: core/src/daemon/protocol.rs_
  - _Requirements: 2.2 (Reload Config)_
  - _Prompt: Role: Rust Developer | Task: Implement `config.reload` handler in `core/src/daemon/ipc.rs`. Must parse params and call manager. | Restrictions: Handle partial updates (null paths)_

- [ ] 2. Enhance EngineManager reload logic in core/src/daemon/manager.rs
  - File: core/src/daemon/manager.rs
  - Ensure `reload` method handles `Option<PathBuf>` correctly (update only if provided)
  - Implement rollback or error reporting if script compilation fails
  - Purpose: Robust configuration applying
  - _Leverage: core/src/engine/mod.rs_
  - _Requirements: 2.2 (Atomic Updates)_
  - _Prompt: Role: Rust Engineer | Task: Update `EngineManager::reload` to support atomic updates. If script compilation fails, do not swap the engine state. Return error. | Restrictions: Ensure thread safety during swap_

- [ ] 3. Update DaemonService in ui/lib/services/daemon_service.dart
  - File: ui/lib/services/daemon_service.dart
  - Implement `reloadConfig({String? configPath, String? scriptPath})`
  - Purpose: Client-side IPC wrapper
  - _Leverage: ui/lib/models/daemon_protocol.dart_
  - _Requirements: 2.2 (UI Sync)_
  - _Prompt: Role: Flutter Developer | Task: Add `reloadConfig` method to `DaemonService` class. Serialize parameters correctly. | Restrictions: None_

- [ ] 4. Wire SlotActions to Daemon in ui/lib/state/slot_actions.dart
  - File: ui/lib/state/slot_actions.dart
  - Inject `DaemonService` (or `DaemonManager`)
  - Call `reloadConfig` after successful slot updates/additions
  - Purpose: Auto-apply changes made in Flow Editor
  - _Leverage: ui/lib/state/daemon_providers.dart_
  - _Requirements: 2.2 (UI Sync)_
  - _Prompt: Role: Flutter Logic Developer | Task: Update `SlotActions` to call `daemonService.reloadConfig()` after modifying the runtime configuration. | Restrictions: Handle daemon disconnection gracefully (optimistic UI update)_
