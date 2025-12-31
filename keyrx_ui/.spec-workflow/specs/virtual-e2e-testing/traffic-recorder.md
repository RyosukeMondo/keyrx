# Traffic Recorder & Replay Specification

## Objective
To capture real-world user input sequences and replay them within the Virtual E2E Testing infrastructure. This allows for:
1.  **Bug Reproduction:** Accurately capturing timing-sensitive edge cases (like the "stuck shift" or "permissive hold" bugs) from actual usage.
2.  **Regression Testing:** converting recorded user sessions into permanent test cases.
3.  **Behavior Verification:** Evaluating whether the current engine produces the expected output for a given input sequence.

## Architecture

### 1. Recorder Component (`keyrx_recorder`)
A lightweight tool (or daemon mode) that taps into the input device and logs raw events.

-   **Input:** Reads from `/dev/input/eventX` (selected by the user or auto-detected).
-   **Output:** Writes a JSON recording file.
-   **Mechanism:** Uses `evdev` crate to read events (non-exclusive grab, so the daemon still works).

### 2. Recording Format (`.krxrec`)
A human-readable JSON format.

```json
{
  "metadata": {
    "version": "1.0",
    "timestamp": "2023-10-27T10:00:00Z",
    "device_name": "Keychron K2",
    "config_hash": "..." // Optional link to config used
  },
  "events": [
    {
      "rel_time_us": 0,
      "type": "Press",
      "key": "A"
    },
    {
      "rel_time_us": 150000,
      "type": "Press",
      "key": "S"
    },
    {
      "rel_time_us": 200000,
      "type": "Release",
      "key": "S"
    },
    {
      "rel_time_us": 250000,
      "type": "Release",
      "key": "A"
    }
  ]
}
```

*   `rel_time_us`: Microseconds relative to the first event (normalized start time).

### 3. Replay Component (`E2EHarness` extension)
Extensions to the existing `E2EHarness` to load and execute these files.

-   **Loader:** Parses `.krxrec` files into `Vec<KeyEvent>`.
-   **Injector:** Feeds events into `VirtualKeyboard` preserving relative timing.
-   **Verifier:** Captures output and compares against an "expected" recording (if available) or simply allows manual verification of the log.

## Implementation Steps

### Task 1: Serialization Support
Add `serde::Serialize` and `serde::Deserialize` to `keyrx_core::runtime::KeyEvent` and `KeyEventType`.
(Note: `KeyCode` already has it).

### Task 2: Recorder CLI
Implement `keyrx_daemon --record <output_file>` or a standalone binary `keyrx-rec`.
*   Decision: Add to `keyrx_daemon` CLI for reuse of existing device discovery code.
*   Command: `keyrx_daemon record --device <path_or_name> --output <file>`

### Task 3: Replay Harness
Add `E2EHarness::play_recording(path: &Path)` to `keyrx_daemon/tests/e2e_harness.rs`.
This method will:
1.  Read the JSON file.
2.  Iterate through events.
3.  Sleep for `delta_time`.
4.  Inject event.

## Usage Workflow

1.  **User encounters bug.**
2.  **User runs recorder:** `sudo keyrx_daemon record --output bug_report.json`
3.  **User reproduces bug.**
4.  **User stops recorder (Ctrl+C).**
5.  **Developer loads `bug_report.json` into a new test case:**
    ```rust
    #[test]
    fn test_repro_bug_123() {
        let harness = E2EHarness::setup(config).unwrap();
        harness.replay_file("tests/recordings/bug_123.json").unwrap();
        // Assert output
    }
    ```
