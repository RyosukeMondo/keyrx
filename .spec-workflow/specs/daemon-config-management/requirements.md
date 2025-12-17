# Requirements: Daemon Config Management

**Feature:** Configuration Loading via IPC
**Status:** Approved

## Functional Requirements
1.  **Reload Config:** Daemon must accept `config.reload` RPC command.
    *   Parameters: `config_path` (optional), `script_path` (optional).
    *   Behavior: Re-read files from disk, re-compile script, apply to running engine.
2.  **Get Active Config:** Daemon must accept `config.get_active`.
    *   Returns currently loaded paths and summary.
3.  **UI Sync:** Saving a profile/keymap in UI must trigger `config.reload` if it affects the active runtime.

## Non-Functional Requirements
-   **Atomic Updates:** Reload should be atomic; if script fails to compile, rollback to previous state (or enter error state).
-   **Validation:** Invalid config must return detailed error via IPC.
