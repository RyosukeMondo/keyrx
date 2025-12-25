# KeyRX Standard Library

The KeyRX standard library (`stdlib`) provides reusable Rhai configuration files for common keyboard remapping patterns. These files eliminate boilerplate when configuring virtual modifiers to behave like physical modifiers.

## Available Libraries

### shift.rhai
Complete shifted key mappings for making a virtual modifier behave like the Shift key.

**Includes:**
- All uppercase letters (A-Z)
- Shifted numbers (1→!, 2→@, 3→#, etc.)
- Shifted punctuation (-, =, [, ], \, ;, ', comma, period, /, `)

**Usage:**
```rhai
when_start("MD_00");
    load("shift.rhai");
when_end();
```

Now when you hold the key mapped to `MD_00`, all letter keys will produce uppercase, numbers will produce symbols, etc.

### ctrl.rhai
Common Ctrl+key shortcuts for making a virtual modifier behave like the Ctrl key.

**Includes:**
- Editing shortcuts (Ctrl+C copy, Ctrl+V paste, Ctrl+X cut, Ctrl+Z undo, etc.)
- File operations (Ctrl+S save, Ctrl+O open, Ctrl+N new, Ctrl+P print, etc.)
- Navigation (Ctrl+A select all, Ctrl+F find, Ctrl+Home/End, Ctrl+Arrow word navigation)
- Browser/tab controls (Ctrl+T new tab, Ctrl+W close tab, Ctrl+Tab switch tabs)
- Function keys (Ctrl+F1-F12)

**Usage:**
```rhai
when_start("MD_01");
    load("ctrl.rhai");
when_end();
```

## Load Search Path

When you use `load("filename.rhai")`, KeyRX searches for the file in the following locations (in order):

1. **Relative to current file**: `./filename.rhai`
2. **Local stdlib**: `./stdlib/filename.rhai`
3. **User stdlib**: `~/.config/keyrx/stdlib/filename.rhai`
4. **System stdlib**: `/usr/share/keyrx/stdlib/filename.rhai` (Linux only)

This allows you to:
- Use the default stdlib files from the system installation
- Override them with user-specific customizations in `~/.config/keyrx/stdlib/`
- Use project-local stdlib files in `./stdlib/`
- Load relative files directly

## Usage Patterns

### Global Virtual Modifier

Make a virtual modifier affect all devices:

```rhai
// Global scope - applies to all devices
device_start("Keyboard");
    when_start("MD_00");
        load("shift.rhai");
    when_end();

    // Map Caps Lock to virtual Shift
    map("CapsLock", "MD_00");

    // When CapsLock is held, letters produce uppercase
    // This works across all keyboards
device_end();
```

### Device-Specific Virtual Modifier

Make a virtual modifier only affect keys on the same device:

```rhai
device_start("LeftHand");
    // Map G to virtual Shift (only for this device)
    map("G", "MD_00");

    // Define Shift behavior (only for this device)
    when_start("MD_00");
        load("shift.rhai");
    when_end();
device_end();

device_start("RightHand");
    // G on RightHand is not affected by LeftHand's MD_00
    map("G", "F");
device_end();
```

### Combining Multiple Modifiers

```rhai
device_start("Keyboard");
    when_start("MD_00");
        load("shift.rhai");
    when_end();

    when_start("MD_01");
        load("ctrl.rhai");
    when_end();

    map("CapsLock", "MD_00");  // CapsLock → Shift
    tap_hold("RightShift", "VK_RightShift", "MD_01", 200);  // RShift → Tap:Shift, Hold:Ctrl
device_end();
```

### Split Keyboard Example

```rhai
device_start("LeftHand-serial-ABC123");
    // Global Shift behavior
    when_start("MD_00");
        load("shift.rhai");
    when_end();

    // Left thumb cluster
    map("Space", "MD_00");  // Space → Shift (affects both keyboards)
    map("Enter", "MD_01");  // Enter → Ctrl
device_end();

device_start("RightHand-serial-DEF456");
    // Right hand can use left hand's Shift
    // Holding Space on LeftHand + tapping A on RightHand = uppercase A
device_end();
```

## Customizing Stdlib Files

You can customize stdlib files by copying them to your user stdlib directory:

```bash
# Create user stdlib directory
mkdir -p ~/.config/keyrx/stdlib

# Copy and customize
cp /usr/share/keyrx/stdlib/shift.rhai ~/.config/keyrx/stdlib/shift.rhai

# Edit with your preferred editor
nano ~/.config/keyrx/stdlib/shift.rhai
```

Your customized version will be used instead of the system default.

## Creating Your Own Stdlib Files

You can create your own reusable stdlib files:

```bash
# Create custom stdlib file
cat > ~/.config/keyrx/stdlib/my_custom.rhai << 'EOF'
//! Custom key mappings

// Vim navigation on arrow keys
map("Left", "VK_H");
map("Down", "VK_J");
map("Up", "VK_K");
map("Right", "VK_L");
EOF
```

Then load it in your config:

```rhai
device_start("Keyboard");
    when_start("MD_02");
        load("my_custom.rhai");
    when_end();
device_end();
```

## Best Practices

1. **Use load() for repetitive patterns**: If you're copy-pasting the same mappings, create a stdlib file.

2. **Document your stdlib files**: Add comments explaining what the file does and how to use it.

3. **Keep stdlib files focused**: One file per modifier or functionality pattern (shift, ctrl, vim-nav, etc.).

4. **Test before committing**: Always test loaded configurations thoroughly.

5. **Version control custom stdlib**: Store your `~/.config/keyrx/stdlib/` in a dotfiles repository.

## Circular Load Detection

KeyRX automatically detects and prevents circular loads:

```rhai
// file_a.rhai (DON'T DO THIS)
load("file_b.rhai");

// file_b.rhai
load("file_a.rhai");  // ERROR: Potential infinite recursion
```

## Troubleshooting

### File not found

**Error**: `Import failed: ImportNotFound { path: "shift.rhai", ...}`

**Solutions**:
1. Check the file exists in one of the search paths
2. Verify filename spelling (case-sensitive)
3. Check file permissions (must be readable)

### Unclosed device block in loaded file

**Error**: `Unclosed device() block`

**Solution**: Ensure all `device_start()` calls have matching `device_end()` in loaded files.

### Duplicate mappings

If you load multiple files that map the same key, the last mapping wins:

```rhai
device_start("Keyboard");
    when_start("MD_00");
        load("shift.rhai");      // Maps A → with_shift(VK_A)
        map("A", "VK_Z");        // Overwrites: A → Z
    when_end();
device_end();
```

## Future Stdlib Files

Planned for future releases:
- `alt.rhai` - Alt+key shortcuts
- `meta.rhai` - Meta/Super/Win+key shortcuts (window management)
- `vim.rhai` - Vim-style navigation (hjkl, word motions)
- `emacs.rhai` - Emacs-style bindings
- `gaming.rhai` - Common gaming key remaps (WASD → arrows, etc.)
