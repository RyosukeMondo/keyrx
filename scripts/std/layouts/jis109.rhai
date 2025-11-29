// KeyRx Standard Library: JIS 109-Key Layout
//
// Key definitions for Japanese JIS keyboard layout.
// Inherits from ANSI and adds JIS-specific keys.

// Import base ANSI layout
// import "std/layouts/ansi" as ansi;

// JIS-specific keys
const HENKAN = 0x79;      // Conversion
const MUHENKAN = 0x7B;    // Non-conversion
const HIRAGANA = 0x70;    // Hiragana/Katakana
const YEN = 0x7D;         // Yen key (backslash position)
const RO = 0x73;          // Ro key (underscore position)

// Additional function keys on some JIS keyboards
const KATAKANA = 0x70;
const HANKAKU = 0x29;     // Half-width/Full-width toggle

// JIS has a different backslash position
const BACKSLASH_JIS = 0x7D;

// Export all ANSI keys plus JIS additions
// All ANSI keys are available through the ansi:: prefix after import

/// Check if a key is a JIS-specific key
fn is_jis_key(keycode) {
    keycode == HENKAN ||
    keycode == MUHENKAN ||
    keycode == HIRAGANA ||
    keycode == YEN ||
    keycode == RO
}
