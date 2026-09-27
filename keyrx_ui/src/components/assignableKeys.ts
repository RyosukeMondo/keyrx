/**
 * Key category types for organizing key assignments
 */
export type KeyCategory = 'virtual' | 'modifier' | 'lock' | 'layer' | 'macro';

/**
 * Represents a key that can be assigned in the visual editor
 */
export interface AssignableKey {
  /** Unique identifier for the key (e.g., "VK_A", "MD_CTRL") */
  id: string;
  /** Display label for the key (e.g., "A", "Ctrl") */
  label: string;
  /** Category this key belongs to */
  category: KeyCategory;
  /** Optional description for tooltip/help text */
  description?: string;
}

/** [id, label, category, description] */
type KeyTuple = [string, string, KeyCategory, string];

// prettier-ignore
const KEY_TUPLES: KeyTuple[] = [
  // Virtual Keys (VK_*) - Standard keyboard keys
  ['VK_A', 'A', 'virtual', 'Letter A'],
  ['VK_B', 'B', 'virtual', 'Letter B'],
  ['VK_C', 'C', 'virtual', 'Letter C'],
  ['VK_D', 'D', 'virtual', 'Letter D'],
  ['VK_E', 'E', 'virtual', 'Letter E'],
  ['VK_F', 'F', 'virtual', 'Letter F'],
  ['VK_G', 'G', 'virtual', 'Letter G'],
  ['VK_H', 'H', 'virtual', 'Letter H'],
  ['VK_I', 'I', 'virtual', 'Letter I'],
  ['VK_J', 'J', 'virtual', 'Letter J'],
  ['VK_K', 'K', 'virtual', 'Letter K'],
  ['VK_L', 'L', 'virtual', 'Letter L'],
  ['VK_M', 'M', 'virtual', 'Letter M'],
  ['VK_N', 'N', 'virtual', 'Letter N'],
  ['VK_O', 'O', 'virtual', 'Letter O'],
  ['VK_P', 'P', 'virtual', 'Letter P'],
  ['VK_Q', 'Q', 'virtual', 'Letter Q'],
  ['VK_R', 'R', 'virtual', 'Letter R'],
  ['VK_S', 'S', 'virtual', 'Letter S'],
  ['VK_T', 'T', 'virtual', 'Letter T'],
  ['VK_U', 'U', 'virtual', 'Letter U'],
  ['VK_V', 'V', 'virtual', 'Letter V'],
  ['VK_W', 'W', 'virtual', 'Letter W'],
  ['VK_X', 'X', 'virtual', 'Letter X'],
  ['VK_Y', 'Y', 'virtual', 'Letter Y'],
  ['VK_Z', 'Z', 'virtual', 'Letter Z'],
  ['VK_1', '1', 'virtual', 'Number 1'],
  ['VK_2', '2', 'virtual', 'Number 2'],
  ['VK_3', '3', 'virtual', 'Number 3'],
  ['VK_4', '4', 'virtual', 'Number 4'],
  ['VK_5', '5', 'virtual', 'Number 5'],
  ['VK_6', '6', 'virtual', 'Number 6'],
  ['VK_7', '7', 'virtual', 'Number 7'],
  ['VK_8', '8', 'virtual', 'Number 8'],
  ['VK_9', '9', 'virtual', 'Number 9'],
  ['VK_0', '0', 'virtual', 'Number 0'],
  ['VK_ENTER', 'Enter', 'virtual', 'Enter key'],
  ['VK_ESCAPE', 'Esc', 'virtual', 'Escape key'],
  ['VK_BACKSPACE', 'Backspace', 'virtual', 'Backspace key'],
  ['VK_TAB', 'Tab', 'virtual', 'Tab key'],
  ['VK_SPACE', 'Space', 'virtual', 'Space bar'],
  ['VK_F1', 'F1', 'virtual', 'Function key F1'],
  ['VK_F2', 'F2', 'virtual', 'Function key F2'],
  ['VK_F3', 'F3', 'virtual', 'Function key F3'],
  ['VK_F4', 'F4', 'virtual', 'Function key F4'],
  ['VK_F5', 'F5', 'virtual', 'Function key F5'],
  ['VK_F6', 'F6', 'virtual', 'Function key F6'],
  ['VK_F7', 'F7', 'virtual', 'Function key F7'],
  ['VK_F8', 'F8', 'virtual', 'Function key F8'],
  ['VK_F9', 'F9', 'virtual', 'Function key F9'],
  ['VK_F10', 'F10', 'virtual', 'Function key F10'],
  ['VK_F11', 'F11', 'virtual', 'Function key F11'],
  ['VK_F12', 'F12', 'virtual', 'Function key F12'],
  ['VK_F13', 'F13', 'virtual', 'Function key F13'],
  ['VK_F14', 'F14', 'virtual', 'Function key F14'],
  ['VK_F15', 'F15', 'virtual', 'Function key F15'],
  ['VK_F16', 'F16', 'virtual', 'Function key F16'],
  ['VK_F17', 'F17', 'virtual', 'Function key F17'],
  ['VK_F18', 'F18', 'virtual', 'Function key F18'],
  ['VK_F19', 'F19', 'virtual', 'Function key F19'],
  ['VK_F20', 'F20', 'virtual', 'Function key F20'],
  ['VK_F21', 'F21', 'virtual', 'Function key F21'],
  ['VK_F22', 'F22', 'virtual', 'Function key F22'],
  ['VK_F23', 'F23', 'virtual', 'Function key F23'],
  ['VK_F24', 'F24', 'virtual', 'Function key F24'],
  ['VK_UP', '↑', 'virtual', 'Arrow Up'],
  ['VK_DOWN', '↓', 'virtual', 'Arrow Down'],
  ['VK_LEFT', '←', 'virtual', 'Arrow Left'],
  ['VK_RIGHT', '→', 'virtual', 'Arrow Right'],
  ['VK_HOME', 'Home', 'virtual', 'Home key'],
  ['VK_END', 'End', 'virtual', 'End key'],
  ['VK_PAGEUP', 'PgUp', 'virtual', 'Page Up'],
  ['VK_PAGEDOWN', 'PgDn', 'virtual', 'Page Down'],
  ['VK_DELETE', 'Del', 'virtual', 'Delete key'],
  ['VK_INSERT', 'Insert', 'virtual', 'Insert key'],
  ['VK_PRINTSCREEN', 'PrtScn', 'virtual', 'Print Screen'],
  ['VK_PAUSE', 'Pause', 'virtual', 'Pause key'],

  // Numpad
  ['VK_KP0', 'Num 0', 'virtual', 'Numpad 0'],
  ['VK_KP1', 'Num 1', 'virtual', 'Numpad 1'],
  ['VK_KP2', 'Num 2', 'virtual', 'Numpad 2'],
  ['VK_KP3', 'Num 3', 'virtual', 'Numpad 3'],
  ['VK_KP4', 'Num 4', 'virtual', 'Numpad 4'],
  ['VK_KP5', 'Num 5', 'virtual', 'Numpad 5'],
  ['VK_KP6', 'Num 6', 'virtual', 'Numpad 6'],
  ['VK_KP7', 'Num 7', 'virtual', 'Numpad 7'],
  ['VK_KP8', 'Num 8', 'virtual', 'Numpad 8'],
  ['VK_KP9', 'Num 9', 'virtual', 'Numpad 9'],
  ['VK_KPASTERISK', 'Num *', 'virtual', 'Numpad multiply'],
  ['VK_KPMINUS', 'Num -', 'virtual', 'Numpad minus'],
  ['VK_KPPLUS', 'Num +', 'virtual', 'Numpad plus'],
  ['VK_KPDOT', 'Num .', 'virtual', 'Numpad decimal'],
  ['VK_KPSLASH', 'Num /', 'virtual', 'Numpad divide'],
  ['VK_KPENTER', 'Num Enter', 'virtual', 'Numpad enter'],

  // Punctuation
  ['VK_MINUS', '-', 'virtual', 'Minus/Hyphen'],
  ['VK_EQUAL', '=', 'virtual', 'Equals'],
  ['VK_LEFTBRACE', '[', 'virtual', 'Left bracket'],
  ['VK_RIGHTBRACE', ']', 'virtual', 'Right bracket'],
  ['VK_SEMICOLON', ';', 'virtual', 'Semicolon'],
  ['VK_APOSTROPHE', "'", 'virtual', 'Apostrophe/Quote'],
  ['VK_GRAVE', '`', 'virtual', 'Grave/Backtick'],
  ['VK_BACKSLASH', '\\', 'virtual', 'Backslash'],
  ['VK_COMMA', ',', 'virtual', 'Comma'],
  ['VK_DOT', '.', 'virtual', 'Period/Dot'],
  ['VK_SLASH', '/', 'virtual', 'Forward slash'],

  // Modifiers (MD_*)
  ['MD_CTRL', 'Ctrl', 'modifier', 'Control modifier'],
  ['MD_SHIFT', 'Shift', 'modifier', 'Shift modifier'],
  ['MD_ALT', 'Alt', 'modifier', 'Alt modifier'],
  ['MD_GUI', 'Super', 'modifier', 'Super/Windows/Command modifier'],
  ['MD_RCTRL', 'RCtrl', 'modifier', 'Right Control modifier'],
  ['MD_RSHIFT', 'RShift', 'modifier', 'Right Shift modifier'],
  ['MD_RALT', 'RAlt', 'modifier', 'Right Alt modifier'],
  ['MD_RGUI', 'RSuper', 'modifier', 'Right Super modifier'],

  // Locks (LK_*)
  ['VK_CAPSLOCK', 'CapsLock', 'lock', 'Caps Lock key'],
  ['VK_NUMLOCK', 'NumLock', 'lock', 'Num Lock key'],
  ['VK_SCROLLLOCK', 'ScrollLock', 'lock', 'Scroll Lock key'],

  // Layers (common layer names)
  ['LAYER_BASE', 'Base Layer', 'layer', 'Switch to base layer'],
  ['LAYER_NAV', 'Nav Layer', 'layer', 'Switch to navigation layer'],
  ['LAYER_NUM', 'Num Layer', 'layer', 'Switch to number layer'],
  ['LAYER_FN', 'Fn Layer', 'layer', 'Switch to function layer'],
  ['LAYER_GAMING', 'Gaming Layer', 'layer', 'Switch to gaming layer'],

  // Macros (example macros)
  ['MACRO_COPY', 'Copy', 'macro', 'Copy macro (Ctrl+C)'],
  ['MACRO_PASTE', 'Paste', 'macro', 'Paste macro (Ctrl+V)'],
  ['MACRO_CUT', 'Cut', 'macro', 'Cut macro (Ctrl+X)'],
  ['MACRO_UNDO', 'Undo', 'macro', 'Undo macro (Ctrl+Z)'],
  ['MACRO_REDO', 'Redo', 'macro', 'Redo macro (Ctrl+Y)'],
];

/**
 * All keys available for assignment in the visual configuration editor,
 * grouped by category (Virtual Keys, Modifiers, Locks, Layers, Macros).
 */
export const ALL_ASSIGNABLE_KEYS: AssignableKey[] = KEY_TUPLES.map(
  ([id, label, category, description]) => ({ id, label, category, description })
);
