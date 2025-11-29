# KeyRx Project Steering: The Path to Success

**Status:** Living Document  
**Goal:** Define the strategic decisions, technology justifications, and quality standards required to build the "Ultimate" input remapper.

---

## 1. Vision & Core Philosophy
**"Empower the user to prescribe their own input reality."**

*   **Logic > Configuration:** We don't just map A to B. We program behaviors. Static YAML is insufficient for power users; Scripting (Rhai) is the answer.
*   **Visual > Abstract:** Complex state machines (layers, mod-taps) must be visible to be understood. The UI is a window into the engine's brain.
*   **Performance > Features:** Latency is the enemy. The tool must feel invisible. 1ms overhead is the maximum acceptable limit.
*   **Target Platforms:** We officially target **Windows** and **Linux**. The architecture ensures consistent behavior across these operating systems while respecting their native input paradigms.

---

## 2. Technology Stack: The Strategic "Why"

### The Core: Rust + Tokio
We chose Rust not just for memory safety, but for **Fearless Concurrency** and **Reliability**.
*   **Async-First:** Input handling is inherently asynchronous. `tokio` allows us to handle keyboard, mouse, MIDI, and timers concurrently without complex, error-prone threading locks found in legacy C++ codebases.
*   **Zero-Cost Abstractions:** We can build high-level traits (`InputSource`) for modularity without sacrificing the raw speed of C.

### The Brain: Rhai (Scripting)
We chose Rhai over Lua, Python, or JS.
*   **Rust-Native:** Rhai compiles into the Rust binary. There is no external VM or heavy runtime to manage.
*   **Sandboxed Safety:** A script cannot delete files, access the network, or crash the app unless we explicitly bind those functions. This makes sharing community scripts safe.
*   **Type Integration:** It shares types with Rust, reducing runtime errors and conversion overhead.

### The Face: Flutter (UI)
We chose Flutter over Electron or Tauri.
*   **Productivity:** The "Hot Reload" cycle allows for rapid iteration of complex UI components (like a visual, drag-and-drop keyboard editor).
*   **Rendering Performance:** Skia/Impeller provides 60fps+ smooth animations, critical for a "Pro" tool feel.
*   **FFI (Foreign Function Interface):** Flutter (Dart) talks directly to Rust memory via C-ABI. There is no HTTP/JSON serialization overhead (unlike Electron IPC). This ensures the UI reflects the engine state instantly.

---

## 3. Architectural Pillars (The "Greenfield" Advantage)

Since we are starting from scratch, we must strictly adhere to these modern patterns:

1.  **No Global State:** Legacy codebases suffer from global variables. KeyRx instances must be self-contained structs.
2.  **Event Sourcing:** Treat input as an immutable stream of events. This allows for "Replay Debugging" (recording a session and re-running it to reproduce a bug).
3.  **Modular Drivers:** The Core Logic must **never** import `windows.h` or `linux/input.h`. Drivers are plugins that implement a generic `InputSource` trait.

---

## 4. The Data Model: Blank Slate & Infinite Potential

We reject the traditional "Remapper" model (patching an existing layout). We adopt the **"Synthesizer"** model.

### A. The Blank Slate (Flush to Zero)
*   **Concept:** The engine does not assume "QWERTY" or "JIS". It assumes **Nothing**.
*   **Mechanism:** When KeyRx loads, it logically "flushes" the keyboard. The user selectively places keys onto this blank canvas.
*   **Benefit:** You are not fighting the OS's default behavior. You are defining the *only* behavior.

### B. Infinite Custom Modifiers
*   **No Limits:** You are not restricted to Shift, Ctrl, Alt, and Win.
*   **Virtual Mods:** You can define up to **255 Custom Modifiers** (e.g., `Mod_Thumb`, `Mod_Gaming`, `Mod_Photoshop`).
*   **Logic:** These modifiers exist purely in the KeyRx Engine. The OS never sees them; it only sees the final result (e.g., `Ctrl+C`).

### C. Combinatorial Mapping
*   **The Formula:** `Action = f(PhysicalKey, ActiveModifiers)`
*   **Freedom:** You can assign a unique action to `A`, `Shift+A`, `Mod1+A`, `Mod1+Shift+A`, or even `Mod1+Mod2+Mod3+A`.
*   **Result:** There are **no restrictions** on key placement or feature density. If you can press it, you can map it.

---

## 5. Quality Assurance: The "Unbreakable" Standard

Input remappers are "Tier 0" software. If they crash, the user's computer becomes unusable. We cannot rely on "it works on my machine."

### A. How We Run Tests
We use a layered approach to catch bugs at different depths:

1.  **Logic Tests (Unit):**
    *   *Goal:* Verify small logic pieces (e.g., "Does `Layer::activate` actually set the flag?").
    *   *Tool:* Standard `cargo test`.
    *   *Frequency:* Run on every file save.

2.  **Property-Based Fuzzing (The Chaos):**
    *   *Goal:* Discover edge cases humans forget (e.g., holding 50 keys at once, pressing keys at `u64::MAX` timestamps).
    *   *Tool:* `proptest` or `cargo-fuzz`.
    *   *Method:* We define invariants (e.g., "Engine must never panic," "Output count must not exceed Input count x 10"). The fuzzer tries to break them.

3.  **Deterministic Simulation (The Replay):**
    *   *Goal:* Reproduce "flaky" bugs that only happen once a week.
    *   *Method:* Because we use **Event Sourcing**, we can record a bug session into a file (`crash.log`). The test suite reads this file and feeds it into the Engine.
    *   *Result:* We can step through the exact millisecond the bug occurred, 100% consistently.

4.  **Virtual Integration (The Mock OS):**
    *   *Goal:* Test the full loop without a real keyboard.
    *   *Method:* We create a `MockInputSource` struct.
        ```rust
        // Real integration test code
        let (mut engine, mock_os) = Engine::new_test_harness();
        mock_os.press(Key::A);
        assert_eq!(mock_os.pop_output(), Action::Key(Key::B)); // Did A become B?
        ```
    *   *Frequency:* Run on CI/CD (GitHub Actions) for Windows and Linux targets.

5.  **Latency Benchmarking:**
    *   *Goal:* Enforce the "Invisible" feel.
    *   *Tool:* `criterion` crate.
    *   *Rule:* If a PR increases input processing latency > 100 microseconds, the build fails.

### B. Snapshot Testing (Configuration Guard)
To ensure user scripts don't break after updates:
*   **Parser Tests:** Maintain a folder of complex Rhai scripts.
*   **Method:** Parse them and compare the resulting Abstract Syntax Tree (AST) against a "golden" snapshot.
*   **Benefit:** We can refactor the Rhai engine freely, knowing we haven't broken compatibility with existing user configs.

---

## 6. Roadmap to Success

### Phase 1: The Iron Core (Headless)
*   **Goal:** A running Rust binary that can accept input, run a Rhai script, and print the output.
*   **Deliverable:** `keyrx_core` crate with `InputSource` trait and `Engine` struct.

### Phase 2: The Nervous System (Drivers)
*   **Goal:** Connect the Core to the real world.
*   **Deliverable:** `keyrx_driver_win` (using `WH_KEYBOARD_LL`) and `keyrx_driver_linux` (using `uinput`).

### Phase 3: The Face (Flutter)
*   **Goal:** A beautiful GUI to control the beast.
*   **Deliverable:** Flutter app skeleton, FFI bindings generation, and the Visual Layer Editor.

### Phase 4: The Ecosystem
*   **Goal:** Community sharing.
*   **Deliverable:** A standard library of Rhai scripts (`std/layouts`, `std/modifiers`) so users don't have to reinvent the wheel.
