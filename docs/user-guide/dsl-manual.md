# KeyRx Configuration DSL Manual

**Version**: 1.1
**Last Updated**: 2026-10-01

Every code block in this manual that is tagged `rhai` is compiled by the test
suite (`keyrx_compiler/tests/manual_examples_test.rs`), so what you read here
is what the compiler accepts. Blocks tagged `rhai,fragment` are statements that
the test wraps in `device_start("*"); ... device_end();`; blocks tagged
`rhai,error` are deliberate mistakes that the compiler must reject.

## Overview

KeyRx uses a **Rhai-based DSL** (Domain-Specific Language) for defining keyboard remapping configurations. All configurations are **compiled ahead of time** into deterministic `.krx` binary files, ensuring:

- **Zero runtime overhead** - No script interpretation
- **Deterministic behavior** - Same input always produces the same bytes
- **Hash-based verification** - Configuration integrity via SHA256

---

## Table of Contents

1. [Script Basics](#script-basics)
2. [Key Naming Rule](#key-naming-rule)
3. [Core Concepts](#core-concepts)
4. [Operations](#operations)
5. [Physical Modifiers in Output](#physical-modifiers-in-output)
6. [Recipes](#recipes)
   - [Accessibility / one-handed](#accessibility--one-handed)
7. [Compiler Checks](#compiler-checks)
8. [Error Reference](#error-reference)
9. [Platform Differences](#platform-differences)
10. [Splitting a Configuration Across Files](#splitting-a-configuration-across-files)
11. [Compilation](#compilation)
12. [Appendix: Syntax Reference](#appendix-syntax-reference)

---

## Script Basics

KeyRx configurations are written in [Rhai](https://rhai.rs/). You only need these basics:

- **Every statement ends with `;`.** `map("A", "VK_B")` without the semicolon is a syntax error.
- Comments are `//` or `/* ... */`.
- Key names are **strings** (`"A"`, `'A'`).
- **Blocks are function calls, not braces.** A device is `device_start(...)` ... `device_end();` and a layer is `when_start(...)` ... `when_end();`. There is no `when(...) { }` form.
- Numbers are plain integers (`200` milliseconds).

```rhai
// A complete script: one device block with one mapping.
device_start("*");
    map("A", "VK_B");   // press A, get B
device_end();
```

Variables and functions are ordinary Rhai and are handy for repetition:

```rhai
let nav_layer = "MD_00";

fn home_row(key, modifier) {
    tap_hold(key, "VK_" + key, modifier, 200);
}

device_start("*");
    map("CapsLock", nav_layer);
    home_row("F", "VK_LShift");
device_end();
```

---

## Key Naming Rule

There is one rule, and the compiler enforces it:

| Position | Form | Example |
|----------|------|---------|
| The key you **press** (first argument of `map`, `tap_hold`, `hold_only`, `sequence`) | bare name | `"CapsLock"`, `"A"`, `"LShift"` |
| A key that is **output** (second argument of `map`, the tap and hold of `tap_hold`, keys of `sequence`, `with_*` helpers) | `VK_` + name | `"VK_Escape"`, `"VK_LCtrl"` |
| A custom modifier (layer) | `MD_00` .. `MD_FE` | `"MD_00"` |
| A custom lock (toggle) | `LK_00` .. `LK_FE` | `"LK_00"` |

- **Inputs: a `VK_` prefix is tolerated and means the same key** (`map("VK_A", ...)` is `map("A", ...)`), because older configs and the visual editor write it. Prefer the bare form in hand-written files.
- **Outputs must have `VK_`.** `map("A", "B")` is rejected with a hint.
- `MD_` and `LK_` can only be outputs (or conditions); `map("MD_00", ...)` is rejected.
- Names are case-sensitive: `"CapsLock"`, not `"capslock"` or `"Caps"`. A typo suggests the closest valid names.

```rhai,error
device_start("*");
    map("A", "B");
device_end();
```

```rhai,error
device_start("*");
    map("MD_00", "VK_A");
device_end();
```

### Valid key names

`A`..`Z`, `Num0`..`Num9` (also `0`..`9`), `F1`..`F24`, `LShift` `RShift` `LCtrl` `RCtrl` `LAlt` `RAlt` `LMeta` `RMeta`,
`Escape` `Enter` `Backspace` `Tab` `Space` `CapsLock` `NumLock` `ScrollLock` `PrintScreen` `Pause`,
`Left` `Right` `Up` `Down` `Home` `End` `PageUp` `PageDown` `Insert` `Delete`,
`Comma` `Period` `Slash` `Semicolon` `Quote` `Minus` `Equal` `Grave` `LeftBracket` `RightBracket` `Backslash`,
`Numpad0`..`Numpad9` `NumpadDivide` `NumpadMultiply` `NumpadSubtract` `NumpadAdd` `NumpadEnter` `NumpadDecimal`,
media/system/browser keys (`Mute` `VolumeUp` `VolumeDown` `MediaPlayPause` `MediaStop` `MediaPrevious` `MediaNext` `Power` `Sleep` `Wake` `BrowserBack` `BrowserForward` `BrowserRefresh` `BrowserStop` `BrowserSearch` `BrowserFavorites` `BrowserHome` `AppMail` `AppCalculator` `AppMyComputer`),
editing keys (`Menu` `Help` `Select` `Execute` `Undo` `Redo` `Cut` `Copy` `Paste` `Find`),
and Japanese/Korean keys (`Zenkaku` `Katakana` `Hiragana` `Henkan` `Muhenkan` `Yen` `Ro` `KatakanaHiragana` `Hangeul` `Hanja` `Iso102nd`).
Short QMK-style aliases also work (`ESC`, `BSPC`, `LCTL`, `LSFT`, `SPC`, `ENT`, `CAPS`, ...).

---

## Core Concepts

### 1. Physical keys, output keys

Pressing the physical key `"A"` makes the OS receive whatever the mapping outputs.

```rhai
device_start("*");
    map("A", "VK_B");   // Press A -> the OS receives B
device_end();
```

### 2. Custom modifiers (255 available)

A **custom modifier** is a virtual modifier state (a layer). `MD_00` through `MD_FE`; physical names such as `MD_LShift` are invalid.

```rhai
device_start("*");
    map("CapsLock", "MD_00");   // CapsLock holds layer 0

    when_start("MD_00");
        map("H", "VK_Left");    // CapsLock+H -> Left arrow
    when_end();
device_end();
```

### 3. Custom locks (255 available)

A **custom lock** is a toggle (like Caps Lock, but yours): press once = ON, press again = OFF.

```rhai
device_start("*");
    map("ScrollLock", "LK_00");   // ScrollLock toggles lock 0

    when_start("LK_00");
        map("B", "VK_Y");         // while lock 0 is ON, B types Y
    when_end();
device_end();
```

### 4. State is shared across devices

All devices share the same modifier/lock state (like a QMK split keyboard): holding a layer key on keyboard A changes keys on keyboard B.

```rhai
device_start("*LEFT*");
    map("LShift", "MD_00");       // left keyboard: LShift is layer 0
device_end();

device_start("*RIGHT*");
    when_start("MD_00");
        map("A", "VK_B");         // right keyboard: A -> B while the left LShift is held
    when_end();
device_end();
```

### 5. Which device a block applies to

`device_start(pattern)` takes a case-insensitive glob (`*` wildcard) matched against the device id, name, serial or path. A device is routed to the **first** block whose pattern matches it; `"*"` matches everything not claimed earlier. List specific devices before the `"*"` block.

---

## Operations

### `device_start(pattern)` / `device_end()`

Every mapping lives inside a device block. See [Which device a block applies to](#5-which-device-a-block-applies-to).

### `map(from, to)`

Maps a physical key to an output key, a custom modifier, or a custom lock.

```rhai
device_start("*");
    map("A", "VK_B");              // key -> key
    map("CapsLock", "MD_00");      // key acts as custom modifier 0
    map("ScrollLock", "LK_00");    // key toggles custom lock 0
device_end();
```

Mapping a physical modifier to a letter removes its modifier behaviour:

```rhai
device_start("*");
    map("LShift", "VK_A");   // LShift now types a
device_end();
```

### `tap_hold(key, tap, hold, threshold_ms)`

One key, two behaviours: a **tap** types `tap`, a **hold** activates `hold`. All four arguments are required.

- `key` - the physical key (bare name).
- `tap` - the output key typed on a tap (`VK_...`).
- `hold` - either
  - a **real key**, `VK_LCtrl`, `VK_LShift`, `VK_LAlt`, `VK_LMeta` (or the `R` variants, or any `VK_` key): it is pressed when the hold is decided and released when you release the physical key, so `Hold + C` really is `Ctrl+C` in every application; or
  - a **custom modifier** `MD_xx`: it switches a layer (see `when_start`).
- `threshold_ms` - how long (milliseconds) before a hold is decided on time alone.

```rhai
device_start("*");
    // Caps Lock: Escape on tap, Ctrl while held.
    tap_hold("CapsLock", "VK_Escape", "VK_LCtrl", 200);

    // Space: tap = space, hold = navigation layer 0.
    tap_hold("Space", "VK_Space", "MD_00", 200);

    when_start("MD_00");
        map("H", "VK_Left");
        map("J", "VK_Down");
        map("K", "VK_Up");
        map("L", "VK_Right");
    when_end();
device_end();
```

**Tap or hold** (permissive hold, as QMK's `PERMISSIVE_HOLD`):
- Released before anything else -> **tap**.
- Held past `threshold_ms` -> **hold**.
- Another key pressed *and released* while it is still down -> **hold** (that key gets the modifier/layer: fast combos need no waiting).
- Released while another key pressed after it is still down -> **tap**, then that key (typing rollover stays typing). Keys pressed while it is undecided are held back until then.

A real-key hold is released correctly even if you change layers or release other keys in any order while it is down.

### `hold_only(key, hold)` / `hold_only(key, hold, threshold_ms)`

Like `tap_hold` but the **tap does nothing** (a quick press is swallowed). `hold` is `VK_...` (a real key) or `MD_xx`. The two-argument form uses a 200 ms threshold.

```rhai
device_start("*");
    hold_only("Tab", "VK_LShift", 150);   // Tab held = Shift; tapping Tab types nothing
    hold_only("RAlt", "MD_01");           // 200 ms default
device_end();
```

### `tap_hold_timeout_only(key, tap, hold, threshold_ms)`

`tap_hold` with **permissive hold turned off for this key**: other keys typed while it is down never decide it. It becomes a hold only when `threshold_ms` passes; released earlier it is always a tap, and the keys typed meanwhile come out afterwards as plain typing. `hold` must be a real key (`VK_...`); a layer (`MD_xx`) has to be decided by the keys typed in it, so use `tap_hold` for layers.

Use it on a home-row key when fast typists see the "nested overlap" turn into a shortcut: with `tap_hold`, `f` down, `c` down, `c` up, `f` up inside 30 ms is read as Ctrl+C; with `tap_hold_timeout_only` it types `fc`. The price: to get the modifier you must hold the key for the whole threshold before pressing the next key.

```rhai
device_start("*");
    tap_hold_timeout_only("F", "VK_F", "VK_LCtrl", 250);
device_end();
```

### `one_shot(key, modifier)` / `one_shot(key, modifier, timeout_ms)`

A **sticky key**: a quick tap *latches* `modifier` for the **next key press only**, so Shift+A needs no two-key chord. Holding the key works as the plain modifier.

- `key` - the physical key (bare name).
- `modifier` - a physical modifier written `VK_LShift`, `VK_LCtrl`, `VK_LAlt`, `VK_LMeta` (or the `R` variants). Custom modifiers (`MD_xx`) and other keys are rejected.
- `timeout_ms` - optional: release an unused latch after this long. `0` or omitted = wait until a key is typed.

```rhai
device_start("*");
    one_shot("CapsLock", "VK_LShift");        // tap, then a letter: capital letter
    one_shot("Tab", "VK_LCtrl", 3000);        // Ctrl for the next key, or off after 3 s
device_end();
```

Rules: the modifier goes down on the tap and is released right after the next non-modifier key is pressed; pressing another modifier does not use it up, so latches stack (tap Shift, tap Ctrl, then `C`). Tapping a latched key again cancels it. A modifier can never stay stuck: the same state machine that presses it releases it.

### `sequence(key, [keys...])`

One press types several keys, in order (1 to 8 keys, each `VK_...`). Each key is pressed and released in turn on the press; releasing the trigger key emits nothing more.

```rhai
device_start("*");
    sequence("F21", ["VK_H", "VK_I", "VK_Enter"]);   // F21 types "hi" and Enter
device_end();
```

The trigger key is mapped like any other key: a second `map("F21", ...)` in the same scope is a [compile error](#compiler-checks). (A `map("F21", "VK_F22")` left in the file before a `sequence("F21", ...)` was the cause of a report that "sequence only types F22".)

### `when_start(condition)` / `when_end()` - layers

Mappings between the two calls apply only while the condition holds.

- `"MD_xx"` - that custom modifier is active
- `"LK_xx"` - that custom lock is ON
- `["MD_00", "LK_01"]` - all of them (AND)
- `"IME"` / `"LANG_JA"` - input-method conditions (see the platform notes)

```rhai
device_start("*");
    map("A", "MD_00");

    when_start("MD_00");
        map("S", "MD_01");          // while A is held, S holds layer 1
    when_end();

    when_start(["MD_00", "MD_01"]);   // both held
        map("F", "VK_Z");
    when_end();
device_end();
```

Blocks cannot be nested. A block must be closed with `when_end()` before `device_end()`; an unclosed block is a compile error that names the line where it was opened.

### `when_not_start(condition)` / `when_not_end()`

Active while the (single) modifier or lock is **not** active.

```rhai
device_start("*");
    when_not_start("LK_00");
        map("N", "VK_M");
    when_not_end();
device_end();
```

### `when_device_start(pattern)` / `when_device_end()`

Mappings that apply only to devices matching the glob, inside a larger block:

```rhai
device_start("*");
    when_device_start("*numpad*");
        map("Numpad1", "VK_F13");
    when_device_end();
device_end();
```

### `load(path)`

Includes another script at this point, inheriting the current device/layer context. See [Splitting a configuration across files](#splitting-a-configuration-across-files).

---

## Physical Modifiers in Output

Use these when the output needs a real Shift/Ctrl/Alt/Win held around the key:

| Helper | Meaning |
|--------|---------|
| `with_shift(key)` | key with Shift |
| `with_ctrl(key)` | key with Ctrl |
| `with_alt(key)` | key with Alt |
| `with_win(key)` | key with Win/Meta |
| `with_mods(key, shift, ctrl, alt, win)` | any combination (booleans) |

```rhai
device_start("*");
    map("F1", with_shift("VK_2"));                          // Shift+2
    map("F2", with_ctrl("VK_C"));                           // Ctrl+C
    map("F3", with_alt("VK_F4"));                           // Alt+F4
    map("F4", with_mods("VK_Escape", true, true, false, false));   // Ctrl+Shift+Escape
device_end();
```

---

## Recipes

### Caps Lock = Escape on tap, Ctrl on hold

```rhai
device_start("*");
    tap_hold("CapsLock", "VK_Escape", "VK_LCtrl", 200);
device_end();
```

### Home-row mods

Tap a home-row key to type its letter, hold it for a real modifier. `examples/07-home-row-mods.rhai` is the full version; the helper function keeps it to one line per key. Use a longer threshold on weaker fingers, and raise thresholds if fast typing triggers modifiers.

```rhai
fn home_row(key, modifier, threshold_ms) {
    tap_hold(key, "VK_" + key, modifier, threshold_ms);
}

device_start("*");
    home_row("A", "VK_LMeta", 250);
    home_row("S", "VK_LAlt", 225);
    home_row("D", "VK_LCtrl", 200);
    home_row("F", "VK_LShift", 200);
    home_row("J", "VK_RShift", 200);
    home_row("K", "VK_RCtrl", 200);
    home_row("L", "VK_RAlt", 225);
    tap_hold("Semicolon", "VK_Semicolon", "VK_RMeta", 250);
device_end();
```

### Vim-style navigation layer

```rhai
device_start("*");
    map("CapsLock", "MD_00");

    when_start("MD_00");
        map("H", "VK_Left");
        map("J", "VK_Down");
        map("K", "VK_Up");
        map("L", "VK_Right");
        map("W", with_ctrl("VK_Right"));   // word forward
        map("B", with_ctrl("VK_Left"));    // word backward
        map("D", "VK_Delete");
        map("U", with_ctrl("VK_Z"));       // undo
    when_end();
device_end();
```

### Cascading layers

```rhai
device_start("*");
    map("A", "MD_00");

    when_start("MD_00");
        map("S", "MD_01");     // while A is held, S holds layer 1
    when_end();

    when_start("MD_01");
        map("D", "VK_Z");      // A+S held: D types Z
    when_end();
device_end();
```

Keep cascades to two or three levels.

### Gaming layer on a lock

```rhai
device_start("*");
    map("F12", "LK_00");   // F12 toggles gaming mode

    when_start("LK_00");
        map("W", "VK_Up");
        map("A", "VK_Left");
        map("S", "VK_Down");
        map("D", "VK_Right");
        map("Space", with_ctrl("VK_Space"));
    when_end();
device_end();
```

Use a lock (not a modifier) for a state that must persist without holding a key.

### Two keyboards, different jobs

```rhai
device_start("*numpad*");
    map("Numpad1", "VK_F13");
device_end();

device_start("*");
    tap_hold("CapsLock", "VK_Escape", "VK_LCtrl", 200);
device_end();
```

### Accessibility / one-handed

`examples/08-one-handed.rhai` is a complete left-hand-only setup: hold Space to mirror the keyboard, tap CapsLock to latch the mirror layer, tap Tab / Grave for sticky Shift / Ctrl. The building blocks:

```rhai
device_start("*");
    // Sticky keys: tap, then the next key is modified. No chords needed.
    one_shot("Tab", "VK_LShift", 5000);
    one_shot("Grave", "VK_LCtrl", 5000);

    // Slow or unsteady taps: a generous threshold. Space is still a space
    // on any tap shorter than this; holding it past it opens the layer.
    tap_hold("Space", "VK_Space", "MD_00", 350);

    when_start("MD_00");
        map("A", "VK_Semicolon");   // mirror the home row
        map("S", "VK_L");
    when_end();
device_end();
```

Tips:

- **Tap thresholds.** `threshold_ms` is how long a key must be held before it counts as a hold. For slow or tremor-affected taps raise it (300-400 ms); for fast typists lower it, or switch to `tap_hold_timeout_only` so rolled keys never become shortcuts.
- **Sticky keys** (`one_shot`) replace chords: tap the modifier, then the key. Add a timeout so a forgotten latch lets go by itself, or use `0` to wait indefinitely. Sticky modifiers are per keyboard.
- **Latching a layer.** `map("CapsLock", "LK_00")` toggles a lock; use the same mappings under `when_start("LK_00")` to keep a layer on without holding anything (the example shares them through a `fn`).
- **Accidental presses.** Nothing here debounces repeated presses; use the operating system's "slow/bounce keys" setting for that.

---

## Compiler Checks

These mistakes are **errors** (the file does not compile), reported with the line number:

| Mistake | Message starts with |
|---------|---------------------|
| The same key mapped twice in one scope (device level, or inside one `when_*` block) | `Duplicate mapping for key ...` - names both lines |
| The same `device_start` pattern used in two blocks (a device is routed to the first match, so the second block would never run; patterns compare case-insensitively) | `Duplicate device_start("...")` - names both lines |
| `device_start` without `device_end()` | `device_start("...") (line N) is never closed` |
| `tap_hold` / `hold_only` threshold of 0, negative or over 65535 | `threshold_ms must be between 1 and 65535` |
| `when_start` / `when_not_start` / `when_device_start` without its `..._end` | `... block opened (line N) is never closed` |
| `..._end` without a start, nested blocks, mappings outside `device_start` | the message says what is missing |
| Output key without `VK_`, or an input that is `MD_`/`LK_` | hint with the fix |

Same key in *different* scopes is fine and is how layers override the base:

```rhai
device_start("*");
    map("H", "VK_H");
    when_start("MD_00");
        map("H", "VK_Left");
    when_end();
device_end();
```

```rhai,error
device_start("*");
    map("A", "VK_B");
    map("A", "VK_C");
device_end();
```

```rhai,error
device_start("*");
    when_start("MD_00");
        map("A", "VK_B");
device_end();
```

Two blocks with the same `device_start` pattern are rejected too, instead of silently ignoring the second one: merge them into one block (this matters when you paste a snippet that has its own `device_start("*")`).

```rhai,error
device_start("*");
    map("A", "VK_B");
device_end();
device_start("*");
    map("C", "VK_D");
device_end();
```

The compiler also warns (without failing) about mappings that can never fire because a more general mapping of the same key is always tried first.

---

## Error Reference

#### Output without a prefix

`Output must have VK_, MD_, or LK_ prefix: B` - write `map("A", "VK_B")`.

#### Wrong prefix in `tap_hold`

`tap_hold tap parameter must have VK_ prefix` - the tap is always a key: `tap_hold("Space", "VK_Space", "MD_00", 200)`. The hold is `MD_xx` or `VK_xx`.

```rhai,error
device_start("*");
    tap_hold("Space", "MD_00", "MD_01", 200);
device_end();
```

#### `tap_hold` needs four arguments

`tap_hold needs 4 argument(s), got 3. Usage: tap_hold(key, tap, hold, threshold_ms)` - add the threshold: `tap_hold("Space", "VK_Space", "MD_00", 200)`.

#### Physical modifier name in a custom modifier

`Physical modifier name 'LShift' cannot be used with MD_ prefix` - use `MD_00`..`MD_FE`; to hold a real modifier use `VK_LShift` as the hold of `tap_hold`/`hold_only`.

```rhai,error
device_start("*");
    map("CapsLock", "MD_LShift");
device_end();
```

#### Modifier ID out of range

`MD_100` is invalid: the maximum is `MD_FE`.

```rhai,error
device_start("*");
    map("A", "MD_100");
device_end();
```

#### Circular `load`

`load("a.rhai")` that eventually loads itself again is rejected; restructure the files.

#### Unknown key name

`Unknown key name: 'Escpae'` - a `Did you mean` list appears only when a real key name is close (a typo or an abbreviation such as `Ente`); an unrelated name gets no suggestion. Key names are in [Valid key names](#valid-key-names).

#### Missing semicolon

`Expecting ';' to terminate this statement` - every call ends with `;`.

---

## Platform Differences

KeyRx maps platform scan codes to one universal `KeyCode` enum, so one `.krx` works on Linux and Windows. Only `device_start` patterns differ: Linux matches evdev names/paths (`/dev/input/by-id/...`), Windows matches device paths (`USB\\VID_AAAA&PID_1111\\SERIAL`).

---

## Splitting a Configuration Across Files

```
~/.config/keyrx/
├── main.rhai              # entry point (the compiler loads this)
├── layers/
│   └── navigation.rhai    # shared pieces
```

`load("layers/navigation.rhai");` runs the other file at that point, relative to the loading file, and its mappings join the current device or layer block. The duplicate-key check applies across the loaded files too.

---

## Compilation

```bash
# Compile a script to a .krx binary (-o is optional)
keyrx_compiler compile main.rhai -o config.krx

# Check a compiled binary / print its hash
keyrx_compiler verify config.krx
keyrx_compiler hash config.krx

# Show what a script parses to (add --json for machine-readable output)
keyrx_compiler parse main.rhai

# Self-contained HTML picture of the mappings
keyrx_compiler view main.rhai
```

Compiling is reproducible: the same script gives a byte-identical `.krx` (no wall-clock time is embedded; the metadata timestamp is `0`, or `SOURCE_DATE_EPOCH` when that is set). `compile` prints the file's real `SHA256`, the same value `sha256sum config.krx` gives; `verify` and `hash` use the integrity hash stored in the header, which covers the data section.

Run it:

```bash
keyrx_daemon run --config config.krx   # pin this file
keyrx_daemon run                       # follow the active profile
keyrx_daemon validate                  # check config and device matching without grabbing devices
keyrx_daemon simulate --events "press:A,wait:50,release:A"
```

---

## Appendix: Syntax Reference

| Call | Purpose |
|------|---------|
| `device_start(pattern);` ... `device_end();` | per-device block |
| `map(key, "VK_x" \| "MD_xx" \| "LK_xx" \| with_*(...));` | remap, layer key, lock key, modified output |
| `tap_hold(key, "VK_tap", "MD_xx" \| "VK_hold", ms);` | tap/hold, hold = layer or real key |
| `tap_hold_timeout_only(key, "VK_tap", "VK_hold", ms);` | tap/hold that typing never decides (no permissive hold) |
| `hold_only(key, "MD_xx" \| "VK_hold" [, ms]);` | hold without tap |
| `one_shot(key, "VK_LShift" [, timeout_ms]);` | sticky modifier for the next key |
| `sequence(key, ["VK_a", ...]);` | type several keys |
| `when_start(cond);` ... `when_end();` | layer while `cond` holds |
| `when_not_start(cond);` ... `when_not_end();` | layer while `cond` does not hold |
| `when_device_start(glob);` ... `when_device_end();` | device-scoped mappings |
| `with_shift` `with_ctrl` `with_alt` `with_win` `with_mods` | modified output |
| `load(path);` | include another script |

Prefixes: `VK_` output key, `MD_` custom modifier (`00`-`FE`), `LK_` custom lock (`00`-`FE`).

---

**End of Manual**
