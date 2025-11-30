# KeyRx Scripting Guide

KeyRx uses the [Rhai scripting language](https://rhai.rs/) to define key remappings and behaviors. This guide covers all available functions, the hook lifecycle, error handling, and provides complete examples.

## Overview

Scripts are loaded and executed when KeyRx starts. The scripting engine provides:

- **Key remapping functions** - Transform one key into another
- **Key blocking** - Consume keys without producing output
- **Lifecycle hooks** - Execute code at specific points
- **Error handling** - Catch and handle errors gracefully

## Quick Start

```rhai
// my_config.rhai
fn on_init() {
    // CapsLock becomes Escape (popular for Vim users)
    remap("CapsLock", "Escape");

    // Block the Insert key (commonly hit by accident)
    block("Insert");
}
```

Load with: `keyrx run --script my_config.rhai`

## Function Reference

### remap(from, to)

Remaps one key to another. When the `from` key is pressed, the `to` key is sent instead.

**Parameters:**
- `from` (String) - The key to remap from. See [docs/KEYS.md](KEYS.md) for valid names.
- `to` (String) - The key to remap to.

**Returns:** Nothing on success.

**Errors:** Throws a runtime error if either key name is invalid.

**Example:**
```rhai
// CapsLock produces Escape
remap("CapsLock", "Escape");

// Swap A and B keys
remap("A", "B");
remap("B", "A");

// Make Grave (`) produce Tab
remap("Grave", "Tab");
```

**Notes:**
- Key names are case-insensitive: `"CapsLock"`, `"capslock"`, and `"CAPSLOCK"` are equivalent
- The original key is consumed; both press and release events are remapped
- Later calls override earlier ones for the same `from` key

### block(key)

Blocks a key entirely. The key press/release is consumed and no output is produced.

**Parameters:**
- `key` (String) - The key to block. See [docs/KEYS.md](KEYS.md) for valid names.

**Returns:** Nothing on success.

**Errors:** Throws a runtime error if the key name is invalid.

**Example:**
```rhai
// Block Insert key (commonly hit by accident)
block("Insert");

// Block CapsLock entirely (no output at all)
block("CapsLock");

// Block Pause/Break key
block("Pause");
```

**Notes:**
- Blocked keys produce no output whatsoever
- Useful for disabling keys you never use but accidentally hit
- Overrides any previous remap for the same key

### pass(key)

Explicitly passes a key through unchanged. This removes any existing remap or block for the key.

**Parameters:**
- `key` (String) - The key to pass through. See [docs/KEYS.md](KEYS.md) for valid names.

**Returns:** Nothing on success.

**Errors:** Throws a runtime error if the key name is invalid.

**Example:**
```rhai
// First, block CapsLock
block("CapsLock");

// Later, restore normal CapsLock behavior
pass("CapsLock");
```

**Notes:**
- All keys pass through by default; this function is for overriding previous mappings
- Useful for conditionally enabling/disabling remaps

### print_debug(message)

Prints a debug message to the log. Useful for troubleshooting scripts.

**Parameters:**
- `message` (String) - The message to print.

**Returns:** Nothing.

**Example:**
```rhai
print_debug("Script loaded!");
print_debug("Setting up remappings...");
```

**Notes:**
- Messages appear in the debug log (enable with `--debug` flag)
- Does not produce any visible output during normal operation

## Hook Lifecycle

Hooks are special functions that KeyRx calls at specific points. Define them in your script and they will be automatically invoked.

### on_init()

Called once when the script is first loaded. Use this to set up your key remappings.

**When called:** After the script file is compiled and loaded, before KeyRx starts processing keys.

**Example:**
```rhai
fn on_init() {
    print_debug("Initializing KeyRx configuration...");

    // Set up all your remappings here
    remap("CapsLock", "Escape");
    block("Insert");

    print_debug("Configuration complete!");
}
```

**Best practices:**
- Put all your `remap()`, `block()`, and `pass()` calls inside `on_init()`
- Keep initialization fast - don't do expensive operations
- Use `print_debug()` to confirm your script loaded correctly

### Hook Detection

KeyRx automatically scans your script for defined hooks. You only need to define the hooks you want to use - undefined hooks are simply not called.

```rhai
// This script only defines on_init
fn on_init() {
    remap("CapsLock", "Escape");
}
// on_key, on_window_change, etc. are not defined - that's fine!
```

## Error Handling

The scripting functions return errors for invalid key names. You can handle these errors using Rhai's `try/catch` syntax.

### Basic Error Handling

```rhai
fn on_init() {
    // This will throw an error - "InvalidKey" doesn't exist
    remap("InvalidKey", "Escape");  // Script stops here with error
}
```

Error message format:
```
Unknown key 'InvalidKey'. See docs/KEYS.md for valid key names.
```

### Using try/catch

```rhai
fn on_init() {
    // Catch errors and continue execution
    try {
        remap("MaybeValidKey", "Escape");
    } catch {
        print_debug("Warning: Could not set up optional remap");
    }

    // This still runs even if the above failed
    remap("CapsLock", "Escape");
}
```

### Validation Pattern

```rhai
fn safe_remap(from, to) {
    try {
        remap(from, to);
        print_debug("Remapped: " + from + " -> " + to);
    } catch(err) {
        print_debug("Failed to remap " + from + ": " + err);
    }
}

fn on_init() {
    safe_remap("CapsLock", "Escape");
    safe_remap("Insert", "Delete");     // Will fail if "Insert" is invalid
    safe_remap("F13", "F1");            // Will fail - F13 not supported
}
```

## Complete Examples

### Example 1: Basic Vim-Friendly Configuration

```rhai
// vim_config.rhai
// A simple configuration for Vim users

fn on_init() {
    print_debug("Loading Vim-friendly configuration...");

    // The classic: CapsLock becomes Escape
    // No more reaching for the corner!
    remap("CapsLock", "Escape");

    // Block Insert key (use 'i' in Vim instead)
    block("Insert");

    print_debug("Vim configuration loaded!");
}
```

### Example 2: Disable Problematic Keys

```rhai
// block_annoyances.rhai
// Block keys that cause accidental problems

fn on_init() {
    // Insert key - accidentally enables overwrite mode
    block("Insert");

    // NumLock - state confusion
    block("NumLock");

    // ScrollLock - rarely useful
    block("ScrollLock");

    // Pause/Break - almost never needed
    block("Pause");
}
```

### Example 3: Modifier Swaps

```rhai
// modifier_swap.rhai
// Swap modifier keys for ergonomics

fn on_init() {
    // Swap left Ctrl and CapsLock (Emacs-style)
    remap("CapsLock", "LeftCtrl");
    remap("LeftCtrl", "CapsLock");

    // Or: Make CapsLock a second Escape (Vim-style)
    // remap("CapsLock", "Escape");
}
```

### Example 4: Navigation Enhancements

```rhai
// navigation.rhai
// Make navigation keys more accessible

fn on_init() {
    // Turn ScrollLock into Home (easy access)
    remap("ScrollLock", "Home");

    // Turn Pause into End
    remap("Pause", "End");

    // PrintScreen becomes Delete
    remap("PrintScreen", "Delete");
}
```

### Example 5: Error-Tolerant Configuration

```rhai
// robust_config.rhai
// Configuration with graceful error handling

fn try_remap(from, to) {
    try {
        remap(from, to);
        return true;
    } catch {
        print_debug("Could not remap: " + from);
        return false;
    }
}

fn try_block(key) {
    try {
        block(key);
        return true;
    } catch {
        print_debug("Could not block: " + key);
        return false;
    }
}

fn on_init() {
    print_debug("Starting robust configuration...");

    let remaps_ok = 0;

    if try_remap("CapsLock", "Escape") { remaps_ok += 1; }
    if try_block("Insert") { remaps_ok += 1; }
    if try_block("NumLock") { remaps_ok += 1; }

    print_debug("Successfully configured " + remaps_ok + " mappings");
}
```

### Example 6: Conditional Setup (Future Feature)

```rhai
// conditional.rhai
// Demonstrates pattern for conditional remaps

fn on_init() {
    // Platform-specific configuration
    // (platform detection coming in future version)

    // Common mappings for all platforms
    remap("CapsLock", "Escape");

    // The following shows the pattern for conditional logic
    // when platform detection becomes available:
    //
    // if platform == "linux" {
    //     remap("LeftMeta", "LeftAlt");  // For Linux desktop
    // }
    // if platform == "macos" {
    //     remap("LeftAlt", "LeftMeta");  // Option <-> Command swap
    // }
}
```

## Script Sandbox

KeyRx scripts run in a sandboxed environment with the following limits:

| Limit | Value | Purpose |
|-------|-------|---------|
| Max expression depth | 64 | Prevent stack overflow |
| Max operations | 100,000 | Prevent infinite loops |

These limits ensure scripts cannot hang or crash KeyRx.

## Best Practices

1. **Use `on_init()` for setup** - Put all remaps in this hook for predictable behavior

2. **Comment your remaps** - Document why each remap exists
   ```rhai
   // CapsLock -> Escape: Vim muscle memory
   remap("CapsLock", "Escape");
   ```

3. **Test incrementally** - Add one remap at a time and verify it works

4. **Use canonical names** - Prefer `"CapsLock"` over `"caps"` for readability

5. **Handle errors for optional features** - Use try/catch for non-critical remaps

6. **Keep scripts simple** - One purpose per script file, compose via multiple configs

## Troubleshooting

### "Unknown key" Error

```
Unknown key 'foo'. See docs/KEYS.md for valid key names.
```

**Solution:** Check the key name spelling. See [docs/KEYS.md](KEYS.md) for all valid names and aliases.

### Script Not Loading

**Check:**
1. File path is correct: `keyrx run --script path/to/script.rhai`
2. File has `.rhai` extension
3. No syntax errors in script

### Remaps Not Working

**Check:**
1. Verify script loaded (add `print_debug("loaded")` to `on_init()`)
2. Check for errors blocking execution
3. Ensure you're running KeyRx with correct permissions

### Debug Mode

Run with debug output to see script execution:
```bash
keyrx run --script config.rhai --debug
```

## See Also

- [KEYS.md](KEYS.md) - Complete list of valid key names and aliases
- [ARCHITECTURE.md](ARCHITECTURE.md) - Technical architecture details
