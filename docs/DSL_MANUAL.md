# KeyRx Configuration DSL Manual

**Version**: 1.0
**Last Updated**: 2025-12-21

## Overview

KeyRx uses a **Rhai-based DSL** (Domain-Specific Language) for defining keyboard remapping configurations. All configurations are **compiled at build-time** into deterministic `.krx` binary files, ensuring:

- **Zero runtime overhead** - No script interpretation
- **Deterministic behavior** - Same input always produces same output
- **Hash-based verification** - Configuration integrity via SHA256
- **Type safety** - Rhai's type system validates syntax

This manual documents the complete DSL syntax and features.

---

## Table of Contents

1. [Core Concepts](#core-concepts)
2. [Key Prefixes](#key-prefixes)
3. [Operations](#operations)
4. [Physical Modifiers](#physical-modifiers)
5. [Examples](#examples)
6. [Best Practices](#best-practices)
7. [Error Reference](#error-reference)

---

## Core Concepts

### 1. Physical Keys vs Virtual Keys

**Physical keys**: The actual keys on your keyboard
- Referenced without prefix: `"A"`, `"Enter"`, `"LShift"`, `"CapsLock"`
- These are the input keys you physically press

**Virtual keys**: The output keys sent to the OS
- Referenced with `VK_` prefix: `"VK_A"`, `"VK_Enter"`, `"VK_LShift"`
- These are what the OS receives after remapping

**Example**:
```rhai
map("A", "VK_B")  // Physical A → Virtual B
// Press A key → OS receives B
```

### 2. Custom Modifiers (255 available)

**Custom modifiers** are virtual modifier states (like Shift/Ctrl, but custom)
- Referenced with `MD_` prefix: `"MD_00"` through `"MD_FE"` (255 total)
- Keys can **act as** custom modifiers
- Used for creating custom modifier behaviors

**Example**:
```rhai
map("CapsLock", "MD_00")  // CapsLock acts as custom Modifier 0

when("MD_00") {
    map("H", "VK_Left")   // CapsLock+H → Left arrow
}
```

**Important**: NO physical modifier names allowed in MD_ prefix!
- ❌ `"MD_LShift"` - Invalid
- ✅ `"MD_00"` - Correct

### 3. Custom Locks (255 available)

**Custom locks** are toggle states (like CapsLock, but custom)
- Referenced with `LK_` prefix: `"LK_00"` through `"LK_FE"` (255 total)
- Press once → ON, press again → OFF
- Used for creating persistent state changes

**Example**:
```rhai
map("ScrollLock", "LK_00")  // ScrollLock toggles Lock 0

when("LK_00") {
    map("B", "VK_Y")        // When Lock 0 is ON, B → Y
}
```

### 4. Cross-Device State Sharing

**Global state model**: All devices share the same modifier/lock state
- Hold Shift on Keyboard A → affects keys on Keyboard B
- Inspired by QMK split keyboard architecture

**Example**:
```rhai
device("USB\\SERIAL_LEFT") {
    map("LShift", "MD_00")     // Left keyboard's LShift = Modifier 0
}

device("USB\\SERIAL_RIGHT") {
    when("MD_00") {            // Right keyboard responds to left's Modifier 0
        map("A", "VK_B")       // When left LShift held, right A → B
    }
}
```

---

## Key Prefixes

### VK_ - Virtual Keys (Output)

**Purpose**: Specify which virtual key to output

**Format**: `"VK_" + KeyName`

**Valid key names**:
- Letters: `VK_A` through `VK_Z`
- Numbers: `VK_0` through `VK_9`
- Function keys: `VK_F1` through `VK_F12`
- Modifiers: `VK_LShift`, `VK_RShift`, `VK_LCtrl`, `VK_RCtrl`, `VK_LAlt`, `VK_RAlt`, `VK_LWin`, `VK_RWin`
- Special: `VK_Enter`, `VK_Escape`, `VK_Backspace`, `VK_Tab`, `VK_Space`, `VK_CapsLock`
- Arrows: `VK_Left`, `VK_Right`, `VK_Up`, `VK_Down`
- Navigation: `VK_Home`, `VK_End`, `VK_PageUp`, `VK_PageDown`, `VK_Insert`, `VK_Delete`
- Symbols: `VK_Comma`, `VK_Period`, `VK_Slash`, `VK_Semicolon`, `VK_Quote`, `VK_Minus`, `VK_Equal`
- Brackets: `VK_LeftBracket`, `VK_RightBracket`, `VK_Backslash`
- And more...

**Examples**:
```rhai
map("A", "VK_B")           // A outputs B
map("Enter", "VK_Escape")  // Enter outputs Escape
```

### MD_ - Custom Modifiers (Act As)

**Purpose**: Make a key act as a custom modifier

**Format**: `"MD_" + HexID` where HexID is `00` through `FE` (255 total)

**Examples**:
```rhai
map("CapsLock", "MD_00")   // CapsLock acts as Modifier 0
map("Space", "MD_01")      // Space acts as Modifier 1
map("A", "MD_12")          // A acts as Modifier 18 (0x12 = 18)
```

**Restrictions**:
- ❌ NO physical modifier names: `MD_LShift`, `MD_Ctrl` are invalid
- ✅ ONLY hex IDs: `MD_00` through `MD_FE`

### LK_ - Custom Locks (Toggle)

**Purpose**: Make a key toggle a custom lock state

**Format**: `"LK_" + HexID` where HexID is `00` through `FE` (255 total)

**Examples**:
```rhai
map("ScrollLock", "LK_00")  // ScrollLock toggles Lock 0
map("NumLock", "LK_01")     // NumLock toggles Lock 1
map("Z", "LK_05")           // Z toggles Lock 5
```

**Behavior**: Press once = ON, press again = OFF (toggle)

---

## Operations

### 1. `map(from, to)` - Basic Mapping

**Purpose**: Map physical key to virtual output, custom modifier, or custom lock

**Syntax**:
```rhai
map(physical_key, output)
```

**Parameters**:
- `physical_key` (string): Physical key name (no prefix)
- `output` (string): Output with prefix (`VK_`, `MD_`, or `LK_`)

**Examples**:
```rhai
// Physical → Virtual
map("A", "VK_B")              // A outputs B
map("CapsLock", "VK_Escape")  // CapsLock outputs Escape

// Physical → Custom Modifier
map("CapsLock", "MD_00")      // CapsLock acts as Modifier 0
map("Space", "MD_01")         // Space acts as Modifier 1

// Physical → Custom Lock
map("ScrollLock", "LK_00")    // ScrollLock toggles Lock 0
```

**De-modifying physical modifiers**:
```rhai
// Remove modifier behavior from physical modifier keys
map("LShift", "VK_A")         // LShift outputs 'a' (no shift effect)
map("RCtrl", "VK_Z")          // RCtrl outputs 'z' (no ctrl effect)

// Now: Hold LShift + press B → outputs "aaaab" (lowercase b, no shift)
```

---

### 2. `tap_hold(key, tap, hold, threshold_ms)` - Dual Behavior

**Purpose**: Key behaves differently when tapped vs held

**Syntax**:
```rhai
tap_hold(key, tap_output, hold_modifier, threshold_ms)
```

**Parameters**:
- `key` (string): Physical key (no prefix)
- `tap_output` (string): Virtual key on tap (`VK_` prefix)
- `hold_modifier` (string): Custom modifier when held (`MD_` prefix)
- `threshold_ms` (number, optional): Time threshold in milliseconds (default: 200)

**Examples**:
```rhai
// Space: tap = space, hold = Modifier 0
tap_hold("Space", "VK_Space", "MD_00", 200)

// Enter: tap = enter, hold = Modifier 1 (default 200ms)
tap_hold("Enter", "VK_Enter", "MD_01")

// Escape: tap = escape, hold = Modifier 2 (100ms threshold)
tap_hold("Escape", "VK_Escape", "MD_02", 100)
```

**Restrictions**:
- `tap_output` MUST have `VK_` prefix
- `hold_modifier` MUST have `MD_` prefix (NO physical names like `MD_LCtrl`)

---

### 3. `when(condition) { ... }` - Conditional Mappings

**Purpose**: Define mappings active only when condition is true

**Syntax**:
```rhai
when(condition) {
    map("A", "VK_B")
    // ... more mappings
}
```

**Parameters**:
- `condition` (string or array):
  - Single modifier: `"MD_XX"`
  - Single lock: `"LK_XX"`
  - Multiple (AND): `["MD_00", "MD_01"]` or `["MD_00", "LK_00"]`

**Examples**:

**Single modifier**:
```rhai
when("MD_00") {
    map("H", "VK_Left")       // When Mod0 held, H → Left
    map("J", "VK_Down")       // J → Down
    map("K", "VK_Up")         // K → Up
    map("L", "VK_Right")      // L → Right
}
```

**Multiple modifiers (AND logic)**:
```rhai
when(["MD_00", "MD_01"]) {    // Both Mod0 AND Mod1 must be active
    map("X", "VK_Z")
}
```

**Lock state**:
```rhai
when("LK_00") {               // When Lock 0 is ON
    map("B", "VK_Y")
}
```

**Mixed (modifier AND lock)**:
```rhai
when(["MD_00", "LK_01"]) {    // Mod0 held AND Lock1 ON
    map("E", "VK_R")
}
```

**Nested modifier cascade**:
```rhai
map("A", "MD_00")             // A acts as Modifier 0

when("MD_00") {
    map("S", "MD_01")         // When A held, S acts as Modifier 1
}

when("MD_01") {
    map("D", "MD_02")         // When A+S held, D acts as Modifier 2
}

when("MD_02") {
    map("F", "VK_Z")          // When A+S+D held, F outputs Z
}

// Usage: Hold A+S+D, press F → outputs Z
```

---

### 4. `when_not(condition) { ... }` - Negated Conditionals

**Purpose**: Define mappings active only when condition is FALSE

**Syntax**:
```rhai
when_not(condition) {
    map("A", "VK_B")
}
```

**Parameters**:
- `condition` (string): Modifier (`"MD_XX"`) or lock (`"LK_XX"`)

**Examples**:
```rhai
when_not("LK_00") {           // When Lock 0 is OFF
    map("N", "VK_M")
}

when_not("MD_00") {           // When Modifier 0 is NOT held
    map("A", "VK_B")
}
```

**Note**: Only supports single condition (no arrays)

---

### 5. `device(pattern) { ... }` - Device-Specific Mappings

**Purpose**: Define mappings for specific device by serial number

**Syntax**:
```rhai
device(serial_pattern) {
    map("A", "VK_B")
    // ... more mappings
}
```

**Parameters**:
- `serial_pattern` (string): USB serial number pattern or device ID

**Examples**:

**Linux (evdev)**:
```rhai
device("/dev/input/by-id/usb-Vendor_Keyboard_Serial123-event-kbd") {
    map("LShift", "MD_00")
}
```

**Windows**:
```rhai
device("USB\\VID_AAAA&PID_1111\\SERIAL_LEFT") {
    map("LShift", "MD_00")
}

device("USB\\VID_BBBB&PID_2222\\SERIAL_RIGHT") {
    when("MD_00") {          // Responds to left keyboard's Modifier 0
        map("A", "VK_B")
    }
}
```

**Cross-device example**:
```rhai
// Left keyboard
device("SERIAL_LEFT") {
    map("CapsLock", "MD_00")

    when("MD_00") {
        map("Z", "LK_00")    // CapsLock+Z toggles Lock 0
    }
}

// Right keyboard
device("SERIAL_RIGHT") {
    when("LK_00") {          // Responds to left's Lock 0 toggle
        map("B", "VK_Y")
    }
}
```

---

## Physical Modifiers

### Output Keys with Physical Modifiers

**Purpose**: Output keys that require physical modifiers (Shift, Ctrl, Alt, Win)

**Use cases**:
- `"` (double quote) = Shift+2 on Japanese keyboard
- `@` = Shift+2 on US keyboard
- `Ctrl+C` = copy shortcut

### Helper Functions

#### `with_shift(key)`

**Purpose**: Output key with Shift modifier

**Syntax**:
```rhai
map("A", with_shift("VK_2"))
```

**Examples**:
```rhai
map("Quote", with_shift("VK_2"))     // Outputs Shift+2 (")
map("At", with_shift("VK_2"))        // Outputs Shift+2 (@)
map("Exclaim", with_shift("VK_1"))   // Outputs Shift+1 (!)
```

#### `with_ctrl(key)`

**Purpose**: Output key with Ctrl modifier

**Examples**:
```rhai
map("Copy", with_ctrl("VK_C"))       // Outputs Ctrl+C
map("Paste", with_ctrl("VK_V"))      // Outputs Ctrl+V
map("Save", with_ctrl("VK_S"))       // Outputs Ctrl+S
```

#### `with_alt(key)`

**Purpose**: Output key with Alt modifier

**Examples**:
```rhai
map("Close", with_alt("VK_F4"))      // Outputs Alt+F4
```

#### `with_mods(key, ...modifiers)`

**Purpose**: Output key with multiple physical modifiers

**Syntax**:
```rhai
// Named parameters
map("TaskMgr", with_mods("VK_Escape", shift: true, ctrl: true))

// Or array syntax
map("TaskMgr", with_mods("VK_Escape", ["Shift", "Ctrl"]))
```

**Examples**:
```rhai
// Ctrl+Shift+Escape (Task Manager)
map("TaskMgr", with_mods("VK_Escape", shift: true, ctrl: true))

// Ctrl+Shift+Delete
map("SecureDesktop", with_mods("VK_Delete", shift: true, ctrl: true))

// Ctrl+Alt+Delete
map("CAD", with_mods("VK_Delete", ctrl: true, alt: true))
```

### Display Names (Optional)

**Purpose**: Document what the output actually produces (for user clarity)

**Syntax**:
```rhai
map("Key", with_shift("VK_2"), display: '"')
```

**Examples**:
```rhai
// Japanese keyboard layout
map("Quote", with_shift("VK_2"), display: '"')       // Double quote
map("SingleQuote", with_shift("VK_7"), display: "'") // Single quote

// US keyboard layout
map("At", with_shift("VK_2"), display: '@')          // @ symbol
map("Hash", with_shift("VK_3"), display: '#')        // # symbol
```

**Note**: `display:` is documentation only - doesn't affect behavior

---

## Examples

### Example 1: Vim-Style Navigation

```rhai
// CapsLock acts as navigation layer
map("CapsLock", "MD_00")

when("MD_00") {
    map("H", "VK_Left")
    map("J", "VK_Down")
    map("K", "VK_Up")
    map("L", "VK_Right")
    map("W", with_ctrl("VK_Right"))  // Word forward
    map("B", with_ctrl("VK_Left"))   // Word backward
    map("D", "VK_Delete")
    map("U", with_ctrl("VK_Z"))      // Undo
}
```

### Example 2: Japanese Keyboard Symbols

```rhai
// Map number row to symbols (Shift+number)
map("1", with_shift("VK_1"), display: '!')
map("2", with_shift("VK_2"), display: '"')
map("3", with_shift("VK_3"), display: '#')
map("4", with_shift("VK_4"), display: '$')
map("5", with_shift("VK_5"), display: '%')
```

### Example 3: Tap/Hold with Navigation

```rhai
// Space: tap = space, hold = navigation layer
tap_hold("Space", "VK_Space", "MD_00", 200)

when("MD_00") {
    map("H", "VK_Left")
    map("J", "VK_Down")
    map("K", "VK_Up")
    map("L", "VK_Right")
}

// Usage:
// - Tap Space → space character
// - Hold Space + H → Left arrow
```

### Example 4: Nested Modifier Cascade

```rhai
// Create 3-level modifier hierarchy
map("A", "MD_00")

when("MD_00") {
    map("S", "MD_01")
}

when("MD_01") {
    map("D", "MD_02")
}

when("MD_02") {
    map("F", "VK_Z")
}

// Usage: Hold A+S+D, press F → outputs Z
```

### Example 5: Cross-Device Setup

```rhai
// Left keyboard
device("USB\\SERIAL_LEFT") {
    map("LShift", "MD_00")
    map("RShift", "MD_01")

    when("MD_00") {
        map("Z", "LK_00")  // LShift+Z toggles Lock 0
    }
}

// Right keyboard
device("USB\\SERIAL_RIGHT") {
    when("MD_00") {
        map("A", "VK_B")   // When left LShift held, A → B
    }

    when("LK_00") {
        map("B", "VK_Y")   // When Lock 0 ON, B → Y
    }
}
```

### Example 6: Gaming Layer with Lock

```rhai
// F12 toggles gaming mode
map("F12", "LK_00")

when("LK_00") {
    // WASD → Arrow keys
    map("W", "VK_Up")
    map("A", "VK_Left")
    map("S", "VK_Down")
    map("D", "VK_Right")

    // Space → Ctrl (crouch in games)
    map("Space", with_ctrl("VK_Space"))
}
```

---

## Best Practices

### 1. Use Descriptive Variable Names (Comments)

```rhai
// Good: Clear intent
map("CapsLock", "MD_00")  // Navigation layer
map("Space", "MD_01")     // Symbol layer

// Bad: No context
map("CapsLock", "MD_00")
map("Space", "MD_01")
```

### 2. Group Related Mappings

```rhai
// ============================================
// NAVIGATION LAYER (CapsLock)
// ============================================
map("CapsLock", "MD_00")

when("MD_00") {
    map("H", "VK_Left")
    map("J", "VK_Down")
    map("K", "VK_Up")
    map("L", "VK_Right")
}

// ============================================
// SYMBOL LAYER (Space held)
// ============================================
tap_hold("Space", "VK_Space", "MD_01", 200)

when("MD_01") {
    map("1", with_shift("VK_1"))
    map("2", with_shift("VK_2"))
}
```

### 3. Document Display Names for Modified Keys

```rhai
// Good: Clear what gets output
map("Quote", with_shift("VK_2"), display: '"')
map("At", with_shift("VK_2"), display: '@')

// Bad: Unclear output
map("Quote", with_shift("VK_2"))
```

### 4. Avoid Deep Modifier Nesting

```rhai
// Good: 2-3 levels max
map("A", "MD_00")
when("MD_00") {
    map("S", "MD_01")
}
when("MD_01") {
    map("D", "VK_Z")
}

// Bad: Too complex (hard to remember)
// A → MD_00 → MD_01 → MD_02 → MD_03 → MD_04 ...
```

### 5. Use Locks for Persistent States

```rhai
// Good: Lock for gaming mode (persistent)
map("F12", "LK_00")

when("LK_00") {
    // Gaming mappings stay active until F12 pressed again
}

// Bad: Modifier for persistent state (must hold key)
map("F12", "MD_00")

when("MD_00") {
    // Must hold F12 entire time (not ergonomic)
}
```

---

## Error Reference

### Common Errors

#### 1. Invalid Prefix

**Error**: `Unknown key prefix: LK_LShift`

**Cause**: Using physical modifier name with MD_/LK_ prefix

**Fix**:
```rhai
// Wrong
map("CapsLock", "MD_LShift")

// Correct
map("CapsLock", "MD_00")
```

#### 2. Missing Prefix

**Error**: `Output must have VK_, MD_, or LK_ prefix: B`

**Cause**: Forgot prefix on output

**Fix**:
```rhai
// Wrong
map("A", "B")

// Correct
map("A", "VK_B")
```

#### 3. Wrong Prefix in tap_hold

**Error**: `tap_hold tap output must have VK_ prefix`

**Cause**: Used MD_ or LK_ in tap output

**Fix**:
```rhai
// Wrong
tap_hold("Space", "MD_00", "MD_01")

// Correct
tap_hold("Space", "VK_Space", "MD_00")
```

#### 4. Invalid Modifier ID

**Error**: `Invalid modifier ID: MD_100 (must be MD_00 through MD_FE)`

**Cause**: Modifier ID out of range (255 max = 0xFE)

**Fix**:
```rhai
// Wrong
map("A", "MD_100")  // 256 in decimal

// Correct
map("A", "MD_FE")   // 254 in decimal (max)
```

#### 5. Circular Import

**Error**: `Circular import detected: main.rhai → devices.rhai → main.rhai`

**Cause**: Files import each other in a loop

**Fix**: Restructure imports to avoid cycles

---

## Platform Differences

### KeyCode Translation

**Physical keys** → Platform-specific codes → **KeyRx KeyCode** (universal) → **Virtual keys** → Platform-specific output

#### Linux (evdev)
- KeyRx maps evdev scancodes → KeyCode enum
- Example: evdev KEY_A (30) → KeyCode::A (0x00) → evdev KEY_B (48)

#### Windows
- KeyRx maps Virtual Key codes → KeyCode enum
- Example: VK_A (0x41) → KeyCode::A (0x00) → VK_B (0x42)

**Your config is platform-agnostic!** Same `.krx` file works on Linux/Windows.

---

## Configuration File Organization

### Recommended Structure

```
~/.config/keyrx/
├── main.rhai              # Entry point (compiler loads this)
├── devices/
│   ├── left_hand.rhai     # Per-device configs
│   ├── right_hand.rhai
│   └── numpad.rhai
├── layers/
│   ├── navigation.rhai    # Shared layers
│   ├── symbols.rhai
│   └── gaming.rhai
└── utils/
    └── common.rhai        # Shared utility functions
```

### Import Example

```rhai
// main.rhai
import "devices/left_hand.rhai"
import "devices/right_hand.rhai"
import "layers/navigation.rhai"

// Conditional imports
if device_exists("USB\\SERIAL_GAMING") {
    import "devices/gaming_keyboard.rhai"
}
```

---

## Compilation

### Compile Configuration

```bash
# Compile Rhai script to .krx binary
keyrx_compiler compile main.rhai -o config.krx

# Verify compiled binary
keyrx_compiler verify config.krx

# Get hash of configuration
keyrx_compiler hash config.krx

# Parse and output JSON (debugging)
keyrx_compiler parse main.rhai --json
```

### Daemon Usage

```bash
# Load configuration
keyrx_daemon --config config.krx

# Reload configuration (live update)
keyrx_daemon --reload config.krx
```

---

## Appendix: Complete Syntax Reference

### Keywords
- `map` - Basic mapping
- `tap_hold` - Dual behavior
- `when` - Conditional block
- `when_not` - Negated conditional
- `device` - Device-specific block
- `import` - Import other files

### Helper Functions
- `with_shift(key)` - Output with Shift
- `with_ctrl(key)` - Output with Ctrl
- `with_alt(key)` - Output with Alt
- `with_mods(key, mods)` - Output with multiple modifiers

### Prefixes
- `VK_` - Virtual key output
- `MD_` - Custom modifier (00-FE)
- `LK_` - Custom lock (00-FE)

### Parameters
- `display:` - Display name (optional, documentation only)
- `shift:`, `ctrl:`, `alt:`, `win:` - Physical modifier flags (in `with_mods`)

---

**End of Manual**
