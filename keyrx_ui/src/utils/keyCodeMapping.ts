/**
 * keyCodeMapping - Centralized key code translation utilities
 *
 * This module provides pure functions for converting between different key code formats:
 * - Numeric event codes (Linux evdev)
 * - VK_ names (Rhai DSL format)
 * - Human-readable labels (for UI display)
 */

/**
 * Map Linux event codes to Rhai VK_ key names.
 * Based on evdev KEY_* constants and keyrx VK_ naming.
 */
const EVENT_CODE_TO_VK: Record<number, string> = {
  // Letters
  30: 'VK_A', 48: 'VK_B', 46: 'VK_C', 32: 'VK_D', 18: 'VK_E',
  33: 'VK_F', 34: 'VK_G', 35: 'VK_H', 23: 'VK_I', 36: 'VK_J',
  37: 'VK_K', 38: 'VK_L', 50: 'VK_M', 49: 'VK_N', 24: 'VK_O',
  25: 'VK_P', 16: 'VK_Q', 19: 'VK_R', 31: 'VK_S', 20: 'VK_T',
  22: 'VK_U', 47: 'VK_V', 17: 'VK_W', 45: 'VK_X', 21: 'VK_Y',
  44: 'VK_Z',

  // Numbers
  2: 'VK_Num1', 3: 'VK_Num2', 4: 'VK_Num3', 5: 'VK_Num4', 6: 'VK_Num5',
  7: 'VK_Num6', 8: 'VK_Num7', 9: 'VK_Num8', 10: 'VK_Num9', 11: 'VK_Num0',

  // Function keys
  59: 'VK_F1', 60: 'VK_F2', 61: 'VK_F3', 62: 'VK_F4', 63: 'VK_F5',
  64: 'VK_F6', 65: 'VK_F7', 66: 'VK_F8', 67: 'VK_F9', 68: 'VK_F10',
  87: 'VK_F11', 88: 'VK_F12',
  183: 'VK_F13', 184: 'VK_F14', 185: 'VK_F15', 186: 'VK_F16',
  187: 'VK_F17', 188: 'VK_F18', 189: 'VK_F19', 190: 'VK_F20',

  // Special keys
  1: 'VK_Escape', 14: 'VK_Backspace', 15: 'VK_Tab', 28: 'VK_Enter',
  57: 'VK_Space', 58: 'VK_CapsLock',

  // Modifiers
  29: 'VK_LeftCtrl', 97: 'VK_RightCtrl',
  42: 'VK_LeftShift', 54: 'VK_RightShift',
  56: 'VK_LeftAlt', 100: 'VK_RightAlt',
  125: 'VK_LeftSuper', 126: 'VK_RightSuper',

  // Navigation
  102: 'VK_Home', 107: 'VK_End',
  104: 'VK_PageUp', 109: 'VK_PageDown',
  103: 'VK_Up', 108: 'VK_Down', 105: 'VK_Left', 106: 'VK_Right',
  110: 'VK_Insert', 111: 'VK_Delete',

  // Symbols
  12: 'VK_Minus', 13: 'VK_Equal',
  26: 'VK_LeftBracket', 27: 'VK_RightBracket',
  39: 'VK_Semicolon', 40: 'VK_Quote',
  41: 'VK_Grave', 43: 'VK_Backslash',
  51: 'VK_Comma', 52: 'VK_Period', 53: 'VK_Slash',

  // Locks
  69: 'VK_NumLock', 70: 'VK_ScrollLock',

  // Media keys
  113: 'VK_Mute', 114: 'VK_VolumeDown', 115: 'VK_VolumeUp',
  163: 'VK_MediaNext', 165: 'VK_MediaPrevious', 164: 'VK_MediaPlayPause',
};

/**
 * Reverse mapping from VK_ names to event codes.
 * Generated from EVENT_CODE_TO_VK.
 */
const VK_TO_EVENT_CODE: Record<string, number> = Object.fromEntries(
  Object.entries(EVENT_CODE_TO_VK).map(([code, vk]) => [vk, Number(code)])
);

/**
 * Map VK_ names to human-readable labels.
 * Removes VK_ prefix and simplifies common names.
 */
const VK_TO_LABEL: Record<string, string> = {
  // Letters (A-Z already clean after VK_ removal)
  VK_A: 'A', VK_B: 'B', VK_C: 'C', VK_D: 'D', VK_E: 'E',
  VK_F: 'F', VK_G: 'G', VK_H: 'H', VK_I: 'I', VK_J: 'J',
  VK_K: 'K', VK_L: 'L', VK_M: 'M', VK_N: 'N', VK_O: 'O',
  VK_P: 'P', VK_Q: 'Q', VK_R: 'R', VK_S: 'S', VK_T: 'T',
  VK_U: 'U', VK_V: 'V', VK_W: 'W', VK_X: 'X', VK_Y: 'Y',
  VK_Z: 'Z',

  // Numbers
  VK_Num1: '1', VK_Num2: '2', VK_Num3: '3', VK_Num4: '4', VK_Num5: '5',
  VK_Num6: '6', VK_Num7: '7', VK_Num8: '8', VK_Num9: '9', VK_Num0: '0',

  // Function keys
  VK_F1: 'F1', VK_F2: 'F2', VK_F3: 'F3', VK_F4: 'F4', VK_F5: 'F5',
  VK_F6: 'F6', VK_F7: 'F7', VK_F8: 'F8', VK_F9: 'F9', VK_F10: 'F10',
  VK_F11: 'F11', VK_F12: 'F12', VK_F13: 'F13', VK_F14: 'F14', VK_F15: 'F15',
  VK_F16: 'F16', VK_F17: 'F17', VK_F18: 'F18', VK_F19: 'F19', VK_F20: 'F20',

  // Special keys - use shorter, common names
  VK_Escape: 'ESC', VK_Backspace: 'BACKSPACE', VK_Tab: 'TAB',
  VK_Enter: 'ENTER', VK_Space: 'SPACE', VK_CapsLock: 'CAPS',

  // Modifiers - use standard abbreviations
  VK_LeftCtrl: 'LCTRL', VK_RightCtrl: 'RCTRL',
  VK_LeftShift: 'LSHIFT', VK_RightShift: 'RSHIFT',
  VK_LeftAlt: 'LALT', VK_RightAlt: 'RALT',
  VK_LeftSuper: 'LSUPER', VK_RightSuper: 'RSUPER',

  // Navigation
  VK_Home: 'HOME', VK_End: 'END',
  VK_PageUp: 'PGUP', VK_PageDown: 'PGDN',
  VK_Up: 'UP', VK_Down: 'DOWN', VK_Left: 'LEFT', VK_Right: 'RIGHT',
  VK_Insert: 'INS', VK_Delete: 'DEL',

  // Symbols - use actual symbols or common names
  VK_Minus: '-', VK_Equal: '=',
  VK_LeftBracket: '[', VK_RightBracket: ']',
  VK_Semicolon: ';', VK_Quote: "'",
  VK_Grave: '`', VK_Backslash: '\\',
  VK_Comma: ',', VK_Period: '.', VK_Slash: '/',

  // Locks
  VK_NumLock: 'NUMLOCK', VK_ScrollLock: 'SCROLLLOCK',

  // Media keys
  VK_Mute: 'MUTE', VK_VolumeDown: 'VOL-', VK_VolumeUp: 'VOL+',
  VK_MediaNext: 'NEXT', VK_MediaPrevious: 'PREV', VK_MediaPlayPause: 'PLAY',
};

/**
 * Convert numeric event code to VK_ name.
 * Returns VK_Unknown{code} format for unmapped codes.
 *
 * @param code - Numeric event code (Linux evdev format)
 * @returns VK_ format name (e.g., "VK_A", "VK_Enter")
 *
 * @example
 * eventCodeToVK(30) // returns "VK_A"
 * eventCodeToVK(28) // returns "VK_Enter"
 * eventCodeToVK(999) // returns "VK_Unknown999"
 */
export function eventCodeToVK(code: number): string {
  return EVENT_CODE_TO_VK[code] || `VK_Unknown${code}`;
}

/**
 * Convert VK_ name to numeric event code.
 * Returns null for unknown VK_ names.
 *
 * @param vkName - VK_ format name (e.g., "VK_A", "VK_Enter")
 * @returns Numeric event code or null if unknown
 *
 * @example
 * vkToEventCode("VK_A") // returns 30
 * vkToEventCode("VK_Enter") // returns 28
 * vkToEventCode("VK_Invalid") // returns null
 */
export function vkToEventCode(vkName: string): number | null {
  return VK_TO_EVENT_CODE[vkName] ?? null;
}

/**
 * Convert VK_ name to human-readable label for UI display.
 * Removes VK_ prefix and uses common abbreviations.
 *
 * @param vkName - VK_ format name (e.g., "VK_A", "VK_Enter")
 * @returns Human-readable label (e.g., "A", "ENTER")
 *
 * @example
 * vkToLabel("VK_A") // returns "A"
 * vkToLabel("VK_Enter") // returns "ENTER"
 * vkToLabel("VK_LeftCtrl") // returns "LCTRL"
 * vkToLabel("VK_Unknown123") // returns "KEY_123"
 */
export function vkToLabel(vkName: string): string {
  // Check explicit mapping first
  if (VK_TO_LABEL[vkName]) {
    return VK_TO_LABEL[vkName];
  }

  // Handle VK_Unknown{code} format
  if (vkName.startsWith('VK_Unknown')) {
    const code = vkName.replace('VK_Unknown', '');
    return `KEY_${code}`;
  }

  // Default: just remove VK_ prefix
  return vkName.replace('VK_', '');
}

/**
 * Convert numeric event code to human-readable label.
 * Convenience function combining eventCodeToVK and vkToLabel.
 *
 * @param code - Numeric event code (Linux evdev format)
 * @returns Human-readable label (e.g., "A", "ENTER", "KEY_123")
 *
 * @example
 * formatKeyCode(30) // returns "A"
 * formatKeyCode(28) // returns "ENTER"
 * formatKeyCode(999) // returns "KEY_999"
 */
export function formatKeyCode(code: number): string {
  const vkName = eventCodeToVK(code);
  return vkToLabel(vkName);
}

/**
 * Convert human-readable label back to VK_ name.
 * Returns null for labels that don't map to known keys.
 *
 * @param label - Human-readable label (case-insensitive)
 * @returns VK_ format name or null if not found
 *
 * @example
 * labelToVK("A") // returns "VK_A"
 * labelToVK("enter") // returns "VK_Enter"
 * labelToVK("LCTRL") // returns "VK_LeftCtrl"
 * labelToVK("invalid") // returns null
 */
export function labelToVK(label: string): string | null {
  const normalizedLabel = label.toUpperCase();

  // Search for exact match in label mappings
  for (const [vk, mappedLabel] of Object.entries(VK_TO_LABEL)) {
    if (mappedLabel.toUpperCase() === normalizedLabel) {
      return vk;
    }
  }

  // Try with VK_ prefix directly
  const vkWithPrefix = `VK_${label}`;
  if (VK_TO_EVENT_CODE[vkWithPrefix] !== undefined) {
    return vkWithPrefix;
  }

  return null;
}

/**
 * Convert human-readable label to numeric event code.
 * Convenience function combining labelToVK and vkToEventCode.
 *
 * @param label - Human-readable label (case-insensitive)
 * @returns Numeric event code or null if not found
 *
 * @example
 * parseKeyCode("A") // returns 30
 * parseKeyCode("enter") // returns 28
 * parseKeyCode("invalid") // returns null
 */
export function parseKeyCode(label: string): number | null {
  const vk = labelToVK(label);
  return vk ? vkToEventCode(vk) : null;
}

/**
 * Check if a numeric code is a known/mapped key.
 *
 * @param code - Numeric event code
 * @returns true if the code has a known mapping
 *
 * @example
 * isKnownKeyCode(30) // returns true (VK_A)
 * isKnownKeyCode(999) // returns false
 */
export function isKnownKeyCode(code: number): boolean {
  return EVENT_CODE_TO_VK[code] !== undefined;
}

/**
 * Get all known event codes.
 * Useful for validation and testing.
 *
 * @returns Array of all mapped event codes
 */
export function getAllEventCodes(): number[] {
  return Object.keys(EVENT_CODE_TO_VK).map(Number);
}

/**
 * Get all known VK_ names.
 * Useful for validation and UI generation.
 *
 * @returns Array of all VK_ format names
 */
export function getAllVKNames(): string[] {
  return Object.values(EVENT_CODE_TO_VK);
}
