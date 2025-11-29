// KeyRx Example Configuration Script
//
// This script demonstrates the core remapping functions available in KeyRx.
// Load this script with: keyrx run --script scripts/std/example.rhai
//
// Available functions:
//   remap(from, to) - Remap one key to another
//   block(key)      - Block a key (consume it, no output)
//   pass(key)       - Explicitly pass a key through unchanged
//   print_debug(msg) - Print a debug message to the log

// on_init() is called when the script is first loaded.
// Use this to set up your key remappings.
fn on_init() {
    print_debug("KeyRx example script loaded!");

    // ========================================
    // REMAP EXAMPLES
    // ========================================

    // Classic CapsLock to Escape remap (popular for Vim users)
    // When you press CapsLock, Escape will be sent instead
    remap("CapsLock", "Escape");

    // Swap two keys: make Grave (`) produce Escape
    // Useful if you want quick access to Escape on the top-left
    // remap("Grave", "Escape");

    // ========================================
    // BLOCK EXAMPLES
    // ========================================

    // Block the Insert key entirely (commonly hit by accident)
    // The key press will be consumed and nothing will happen
    block("Insert");

    // Block Pause/Break key (rarely used)
    // block("Pause");

    // ========================================
    // PASS EXAMPLES
    // ========================================

    // Explicitly mark Enter to pass through unchanged.
    // This is the default behavior, but you can use pass() to be explicit
    // or to override a previous remap/block.
    pass("Enter");

    // Pass common modifier keys (also default behavior, shown for clarity)
    // pass("LeftShift");
    // pass("LeftCtrl");
    // pass("LeftAlt");

    print_debug("Remappings configured successfully!");
}

// ========================================
// KEY REFERENCE
// ========================================
//
// Letters: A, B, C, ... Z
// Numbers: 0, 1, 2, ... 9
// Function: F1, F2, ... F12
//
// Modifiers:
//   LeftShift, RightShift (aliases: Shift, LShift, RShift)
//   LeftCtrl, RightCtrl (aliases: Ctrl, Control, LCtrl, RCtrl)
//   LeftAlt, RightAlt (aliases: Alt, LAlt, RAlt, AltGr)
//   LeftMeta, RightMeta (aliases: Meta, Win, Super, Cmd)
//
// Navigation:
//   Up, Down, Left, Right
//   Home, End, PageUp, PageDown (aliases: PgUp, PgDn)
//
// Editing:
//   Insert (alias: Ins), Delete (alias: Del), Backspace
//
// Whitespace:
//   Space, Tab, Enter (alias: Return)
//
// Locks:
//   CapsLock (alias: Caps), NumLock, ScrollLock
//
// Special:
//   Escape (alias: Esc), PrintScreen (aliases: Print, PrtSc), Pause
//
// Punctuation:
//   Grave, Minus, Equal, LeftBracket, RightBracket
//   Backslash, Semicolon, Apostrophe, Comma, Period, Slash
//
// Numpad:
//   Numpad0-9, NumpadAdd, NumpadSubtract, NumpadMultiply
//   NumpadDivide, NumpadEnter, NumpadDecimal
//
// Media:
//   VolumeUp, VolumeDown, VolumeMute
//   MediaPlayPause, MediaStop, MediaNext, MediaPrev
