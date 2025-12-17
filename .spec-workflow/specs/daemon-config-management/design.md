# Design: Daemon Config Management

## Architecture
-   **Core:** `EngineManager` handles the hot-reloading logic (swapping `RhaiRuntime` and `DeviceRegistry`).
-   **Protocol:** JSON-RPC methods for config sync.

## IPC Protocol
### `config.reload`
-   **Params:** `{ "config_path": string?, "script_path": string? }`
-   **Result:** `{ "status": "reloaded", "loaded_config": string, "loaded_script": string }`
-   **Error:** `{ "code": -32002, "message": "Config load failed", "data": "Parse error..." }`

### `config.get_active`
-   **Params:** `{}`
-   **Result:** `{ "config_path": string?, "script_path": string? }`

## Code Changes
1.  **Core (`core/src/daemon/ipc.rs`):** Add `config.*` handlers.
2.  **Core (`core/src/daemon/manager.rs`):** Enhance `reload` method to support partial updates.
3.  **UI (`ui/lib/services/daemon_service.dart`):** Add `reloadConfig` method.
4.  **UI (`ui/lib/state/slot_actions.dart`):** Trigger reload on slot changes.
