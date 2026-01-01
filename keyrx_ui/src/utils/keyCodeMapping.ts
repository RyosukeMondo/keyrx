/**
 * Key code translation utilities.
 *
 * Provides functions to convert between numeric key codes and human-readable
 * labels, handling special keys and formatting.
 */

/**
 * Map of key code strings to human-readable labels.
 * Covers common keys including letters, numbers, and special keys.
 */
const KEY_LABELS: Record<string, string> = {
  // Letters
  KEY_A: "A",
  KEY_B: "B",
  KEY_C: "C",
  KEY_D: "D",
  KEY_E: "E",
  KEY_F: "F",
  KEY_G: "G",
  KEY_H: "H",
  KEY_I: "I",
  KEY_J: "J",
  KEY_K: "K",
  KEY_L: "L",
  KEY_M: "M",
  KEY_N: "N",
  KEY_O: "O",
  KEY_P: "P",
  KEY_Q: "Q",
  KEY_R: "R",
  KEY_S: "S",
  KEY_T: "T",
  KEY_U: "U",
  KEY_V: "V",
  KEY_W: "W",
  KEY_X: "X",
  KEY_Y: "Y",
  KEY_Z: "Z",

  // Numbers
  KEY_0: "0",
  KEY_1: "1",
  KEY_2: "2",
  KEY_3: "3",
  KEY_4: "4",
  KEY_5: "5",
  KEY_6: "6",
  KEY_7: "7",
  KEY_8: "8",
  KEY_9: "9",

  // Function keys
  KEY_F1: "F1",
  KEY_F2: "F2",
  KEY_F3: "F3",
  KEY_F4: "F4",
  KEY_F5: "F5",
  KEY_F6: "F6",
  KEY_F7: "F7",
  KEY_F8: "F8",
  KEY_F9: "F9",
  KEY_F10: "F10",
  KEY_F11: "F11",
  KEY_F12: "F12",

  // Special keys
  KEY_ENTER: "Enter",
  KEY_ESC: "Esc",
  KEY_ESCAPE: "Esc",
  KEY_SPACE: "Space",
  KEY_TAB: "Tab",
  KEY_BACKSPACE: "Backspace",
  KEY_DELETE: "Delete",
  KEY_INSERT: "Insert",
  KEY_HOME: "Home",
  KEY_END: "End",
  KEY_PAGEUP: "PgUp",
  KEY_PAGEDOWN: "PgDn",

  // Arrow keys
  KEY_LEFT: "←",
  KEY_RIGHT: "→",
  KEY_UP: "↑",
  KEY_DOWN: "↓",

  // Modifiers
  KEY_LEFTCTRL: "LCtrl",
  KEY_RIGHTCTRL: "RCtrl",
  KEY_LEFTSHIFT: "LShift",
  KEY_RIGHTSHIFT: "RShift",
  KEY_LEFTALT: "LAlt",
  KEY_RIGHTALT: "RAlt",
  KEY_LEFTMETA: "LMeta",
  KEY_RIGHTMETA: "RMeta",

  // Punctuation
  KEY_MINUS: "-",
  KEY_EQUAL: "=",
  KEY_LEFTBRACE: "[",
  KEY_RIGHTBRACE: "]",
  KEY_SEMICOLON: ";",
  KEY_APOSTROPHE: "'",
  KEY_GRAVE: "`",
  KEY_BACKSLASH: "\\",
  KEY_COMMA: ",",
  KEY_DOT: ".",
  KEY_SLASH: "/",
};

/**
 * Formats a key code string as a human-readable label.
 *
 * @param code - Key code string (e.g., "KEY_A", "KEY_ENTER")
 * @returns Human-readable label
 *
 * @example
 * formatKeyCode("KEY_A") // "A"
 * formatKeyCode("KEY_ENTER") // "Enter"
 * formatKeyCode("UNKNOWN") // "UNKNOWN"
 */
export function formatKeyCode(code: string): string {
  return KEY_LABELS[code] || code;
}

/**
 * Converts a key code string to a human-readable label.
 * Alias for formatKeyCode for backwards compatibility.
 *
 * @param code - Key code string
 * @returns Human-readable label
 *
 * @example
 * keyCodeToLabel("KEY_A") // "A"
 * keyCodeToLabel("KEY_ENTER") // "Enter"
 */
export function keyCodeToLabel(code: string): string {
  return formatKeyCode(code);
}

/**
 * Parses a human-readable label back to a key code string.
 *
 * @param label - Human-readable label
 * @returns Key code string or null if not found
 *
 * @example
 * parseKeyCode("A") // "KEY_A"
 * parseKeyCode("Enter") // "KEY_ENTER"
 * parseKeyCode("Unknown") // null
 */
export function parseKeyCode(label: string): string | null {
  // Reverse lookup in the map
  for (const [code, keyLabel] of Object.entries(KEY_LABELS)) {
    if (keyLabel === label) {
      return code;
    }
  }
  return null;
}
