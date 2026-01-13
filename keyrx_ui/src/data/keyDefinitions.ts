/**
 * Comprehensive key definitions database
 * Single source of truth for all keyboard keys with QMK-compatible naming
 */

export interface KeyDefinition {
  id: string;
  label: string;
  category: 'basic' | 'modifiers' | 'media' | 'macro' | 'layers' | 'special' | 'any';
  subcategory?: string;
  description: string;
  aliases: string[];
  icon?: string;
}

/**
 * Complete key definitions database (250+ keys)
 */
export const KEY_DEFINITIONS: KeyDefinition[] = [
  // ============================================================
  // BASIC KEYS - LETTERS (A-Z)
  // ============================================================
  { id: 'A', label: 'A', category: 'basic', subcategory: 'letters', description: 'Letter A', aliases: ['KC_A', 'VK_A', 'KEY_A'] },
  { id: 'B', label: 'B', category: 'basic', subcategory: 'letters', description: 'Letter B', aliases: ['KC_B', 'VK_B', 'KEY_B'] },
  { id: 'C', label: 'C', category: 'basic', subcategory: 'letters', description: 'Letter C', aliases: ['KC_C', 'VK_C', 'KEY_C'] },
  { id: 'D', label: 'D', category: 'basic', subcategory: 'letters', description: 'Letter D', aliases: ['KC_D', 'VK_D', 'KEY_D'] },
  { id: 'E', label: 'E', category: 'basic', subcategory: 'letters', description: 'Letter E', aliases: ['KC_E', 'VK_E', 'KEY_E'] },
  { id: 'F', label: 'F', category: 'basic', subcategory: 'letters', description: 'Letter F', aliases: ['KC_F', 'VK_F', 'KEY_F'] },
  { id: 'G', label: 'G', category: 'basic', subcategory: 'letters', description: 'Letter G', aliases: ['KC_G', 'VK_G', 'KEY_G'] },
  { id: 'H', label: 'H', category: 'basic', subcategory: 'letters', description: 'Letter H', aliases: ['KC_H', 'VK_H', 'KEY_H'] },
  { id: 'I', label: 'I', category: 'basic', subcategory: 'letters', description: 'Letter I', aliases: ['KC_I', 'VK_I', 'KEY_I'] },
  { id: 'J', label: 'J', category: 'basic', subcategory: 'letters', description: 'Letter J', aliases: ['KC_J', 'VK_J', 'KEY_J'] },
  { id: 'K', label: 'K', category: 'basic', subcategory: 'letters', description: 'Letter K', aliases: ['KC_K', 'VK_K', 'KEY_K'] },
  { id: 'L', label: 'L', category: 'basic', subcategory: 'letters', description: 'Letter L', aliases: ['KC_L', 'VK_L', 'KEY_L'] },
  { id: 'M', label: 'M', category: 'basic', subcategory: 'letters', description: 'Letter M', aliases: ['KC_M', 'VK_M', 'KEY_M'] },
  { id: 'N', label: 'N', category: 'basic', subcategory: 'letters', description: 'Letter N', aliases: ['KC_N', 'VK_N', 'KEY_N'] },
  { id: 'O', label: 'O', category: 'basic', subcategory: 'letters', description: 'Letter O', aliases: ['KC_O', 'VK_O', 'KEY_O'] },
  { id: 'P', label: 'P', category: 'basic', subcategory: 'letters', description: 'Letter P', aliases: ['KC_P', 'VK_P', 'KEY_P'] },
  { id: 'Q', label: 'Q', category: 'basic', subcategory: 'letters', description: 'Letter Q', aliases: ['KC_Q', 'VK_Q', 'KEY_Q'] },
  { id: 'R', label: 'R', category: 'basic', subcategory: 'letters', description: 'Letter R', aliases: ['KC_R', 'VK_R', 'KEY_R'] },
  { id: 'S', label: 'S', category: 'basic', subcategory: 'letters', description: 'Letter S', aliases: ['KC_S', 'VK_S', 'KEY_S'] },
  { id: 'T', label: 'T', category: 'basic', subcategory: 'letters', description: 'Letter T', aliases: ['KC_T', 'VK_T', 'KEY_T'] },
  { id: 'U', label: 'U', category: 'basic', subcategory: 'letters', description: 'Letter U', aliases: ['KC_U', 'VK_U', 'KEY_U'] },
  { id: 'V', label: 'V', category: 'basic', subcategory: 'letters', description: 'Letter V', aliases: ['KC_V', 'VK_V', 'KEY_V'] },
  { id: 'W', label: 'W', category: 'basic', subcategory: 'letters', description: 'Letter W', aliases: ['KC_W', 'VK_W', 'KEY_W'] },
  { id: 'X', label: 'X', category: 'basic', subcategory: 'letters', description: 'Letter X', aliases: ['KC_X', 'VK_X', 'KEY_X'] },
  { id: 'Y', label: 'Y', category: 'basic', subcategory: 'letters', description: 'Letter Y', aliases: ['KC_Y', 'VK_Y', 'KEY_Y'] },
  { id: 'Z', label: 'Z', category: 'basic', subcategory: 'letters', description: 'Letter Z', aliases: ['KC_Z', 'VK_Z', 'KEY_Z'] },

  // ============================================================
  // BASIC KEYS - NUMBERS (0-9)
  // ============================================================
  { id: '0', label: '0', category: 'basic', subcategory: 'numbers', description: 'Number 0', aliases: ['KC_0', 'VK_0', 'KEY_0'] },
  { id: '1', label: '1', category: 'basic', subcategory: 'numbers', description: 'Number 1', aliases: ['KC_1', 'VK_1', 'KEY_1'] },
  { id: '2', label: '2', category: 'basic', subcategory: 'numbers', description: 'Number 2', aliases: ['KC_2', 'VK_2', 'KEY_2'] },
  { id: '3', label: '3', category: 'basic', subcategory: 'numbers', description: 'Number 3', aliases: ['KC_3', 'VK_3', 'KEY_3'] },
  { id: '4', label: '4', category: 'basic', subcategory: 'numbers', description: 'Number 4', aliases: ['KC_4', 'VK_4', 'KEY_4'] },
  { id: '5', label: '5', category: 'basic', subcategory: 'numbers', description: 'Number 5', aliases: ['KC_5', 'VK_5', 'KEY_5'] },
  { id: '6', label: '6', category: 'basic', subcategory: 'numbers', description: 'Number 6', aliases: ['KC_6', 'VK_6', 'KEY_6'] },
  { id: '7', label: '7', category: 'basic', subcategory: 'numbers', description: 'Number 7', aliases: ['KC_7', 'VK_7', 'KEY_7'] },
  { id: '8', label: '8', category: 'basic', subcategory: 'numbers', description: 'Number 8', aliases: ['KC_8', 'VK_8', 'KEY_8'] },
  { id: '9', label: '9', category: 'basic', subcategory: 'numbers', description: 'Number 9', aliases: ['KC_9', 'VK_9', 'KEY_9'] },

  // ============================================================
  // BASIC KEYS - FUNCTION KEYS (F1-F24)
  // ============================================================
  { id: 'F1', label: 'F1', category: 'basic', subcategory: 'function', description: 'Function key F1', aliases: ['KC_F1', 'VK_F1', 'KEY_F1'] },
  { id: 'F2', label: 'F2', category: 'basic', subcategory: 'function', description: 'Function key F2', aliases: ['KC_F2', 'VK_F2', 'KEY_F2'] },
  { id: 'F3', label: 'F3', category: 'basic', subcategory: 'function', description: 'Function key F3', aliases: ['KC_F3', 'VK_F3', 'KEY_F3'] },
  { id: 'F4', label: 'F4', category: 'basic', subcategory: 'function', description: 'Function key F4', aliases: ['KC_F4', 'VK_F4', 'KEY_F4'] },
  { id: 'F5', label: 'F5', category: 'basic', subcategory: 'function', description: 'Function key F5', aliases: ['KC_F5', 'VK_F5', 'KEY_F5'] },
  { id: 'F6', label: 'F6', category: 'basic', subcategory: 'function', description: 'Function key F6', aliases: ['KC_F6', 'VK_F6', 'KEY_F6'] },
  { id: 'F7', label: 'F7', category: 'basic', subcategory: 'function', description: 'Function key F7', aliases: ['KC_F7', 'VK_F7', 'KEY_F7'] },
  { id: 'F8', label: 'F8', category: 'basic', subcategory: 'function', description: 'Function key F8', aliases: ['KC_F8', 'VK_F8', 'KEY_F8'] },
  { id: 'F9', label: 'F9', category: 'basic', subcategory: 'function', description: 'Function key F9', aliases: ['KC_F9', 'VK_F9', 'KEY_F9'] },
  { id: 'F10', label: 'F10', category: 'basic', subcategory: 'function', description: 'Function key F10', aliases: ['KC_F10', 'VK_F10', 'KEY_F10'] },
  { id: 'F11', label: 'F11', category: 'basic', subcategory: 'function', description: 'Function key F11', aliases: ['KC_F11', 'VK_F11', 'KEY_F11'] },
  { id: 'F12', label: 'F12', category: 'basic', subcategory: 'function', description: 'Function key F12', aliases: ['KC_F12', 'VK_F12', 'KEY_F12'] },
  { id: 'F13', label: 'F13', category: 'basic', subcategory: 'function', description: 'Function key F13', aliases: ['KC_F13', 'VK_F13', 'KEY_F13'] },
  { id: 'F14', label: 'F14', category: 'basic', subcategory: 'function', description: 'Function key F14', aliases: ['KC_F14', 'VK_F14', 'KEY_F14'] },
  { id: 'F15', label: 'F15', category: 'basic', subcategory: 'function', description: 'Function key F15', aliases: ['KC_F15', 'VK_F15', 'KEY_F15'] },
  { id: 'F16', label: 'F16', category: 'basic', subcategory: 'function', description: 'Function key F16', aliases: ['KC_F16', 'VK_F16', 'KEY_F16'] },
  { id: 'F17', label: 'F17', category: 'basic', subcategory: 'function', description: 'Function key F17', aliases: ['KC_F17', 'VK_F17', 'KEY_F17'] },
  { id: 'F18', label: 'F18', category: 'basic', subcategory: 'function', description: 'Function key F18', aliases: ['KC_F18', 'VK_F18', 'KEY_F18'] },
  { id: 'F19', label: 'F19', category: 'basic', subcategory: 'function', description: 'Function key F19', aliases: ['KC_F19', 'VK_F19', 'KEY_F19'] },
  { id: 'F20', label: 'F20', category: 'basic', subcategory: 'function', description: 'Function key F20', aliases: ['KC_F20', 'VK_F20', 'KEY_F20'] },
  { id: 'F21', label: 'F21', category: 'basic', subcategory: 'function', description: 'Function key F21', aliases: ['KC_F21', 'VK_F21', 'KEY_F21'] },
  { id: 'F22', label: 'F22', category: 'basic', subcategory: 'function', description: 'Function key F22', aliases: ['KC_F22', 'VK_F22', 'KEY_F22'] },
  { id: 'F23', label: 'F23', category: 'basic', subcategory: 'function', description: 'Function key F23', aliases: ['KC_F23', 'VK_F23', 'KEY_F23'] },
  { id: 'F24', label: 'F24', category: 'basic', subcategory: 'function', description: 'Function key F24', aliases: ['KC_F24', 'VK_F24', 'KEY_F24'] },

  // ============================================================
  // BASIC KEYS - NAVIGATION
  // ============================================================
  { id: 'Escape', label: 'Esc', category: 'basic', subcategory: 'navigation', description: 'Escape key', aliases: ['KC_ESC', 'KC_ESCAPE', 'VK_ESCAPE', 'KEY_ESC'] },
  { id: 'Enter', label: 'Enter', category: 'basic', subcategory: 'navigation', description: 'Enter/Return key', aliases: ['KC_ENT', 'KC_ENTER', 'VK_RETURN', 'KEY_ENTER'] },
  { id: 'Space', label: 'Space', category: 'basic', subcategory: 'navigation', description: 'Space bar', aliases: ['KC_SPC', 'KC_SPACE', 'VK_SPACE', 'KEY_SPACE'] },
  { id: 'Backspace', label: 'BS', category: 'basic', subcategory: 'navigation', description: 'Backspace key', aliases: ['KC_BSPC', 'KC_BACKSPACE', 'VK_BACK', 'KEY_BACKSPACE'] },
  { id: 'Tab', label: 'Tab', category: 'basic', subcategory: 'navigation', description: 'Tab key', aliases: ['KC_TAB', 'VK_TAB', 'KEY_TAB'] },
  { id: 'Delete', label: 'Del', category: 'basic', subcategory: 'navigation', description: 'Delete key', aliases: ['KC_DEL', 'KC_DELETE', 'VK_DELETE', 'KEY_DELETE'] },
  { id: 'Insert', label: 'Ins', category: 'basic', subcategory: 'navigation', description: 'Insert key', aliases: ['KC_INS', 'KC_INSERT', 'VK_INSERT', 'KEY_INSERT'] },
  { id: 'Home', label: 'Home', category: 'basic', subcategory: 'navigation', description: 'Home key', aliases: ['KC_HOME', 'VK_HOME', 'KEY_HOME'] },
  { id: 'End', label: 'End', category: 'basic', subcategory: 'navigation', description: 'End key', aliases: ['KC_END', 'VK_END', 'KEY_END'] },
  { id: 'PageUp', label: 'PgUp', category: 'basic', subcategory: 'navigation', description: 'Page Up key', aliases: ['KC_PGUP', 'KC_PAGEUP', 'VK_PRIOR', 'KEY_PAGEUP'] },
  { id: 'PageDown', label: 'PgDn', category: 'basic', subcategory: 'navigation', description: 'Page Down key', aliases: ['KC_PGDN', 'KC_PAGEDOWN', 'VK_NEXT', 'KEY_PAGEDOWN'] },
  { id: 'Up', label: '↑', category: 'basic', subcategory: 'navigation', description: 'Arrow Up', aliases: ['KC_UP', 'VK_UP', 'KEY_UP'] },
  { id: 'Down', label: '↓', category: 'basic', subcategory: 'navigation', description: 'Arrow Down', aliases: ['KC_DOWN', 'VK_DOWN', 'KEY_DOWN'] },
  { id: 'Left', label: '←', category: 'basic', subcategory: 'navigation', description: 'Arrow Left', aliases: ['KC_LEFT', 'VK_LEFT', 'KEY_LEFT'] },
  { id: 'Right', label: '→', category: 'basic', subcategory: 'navigation', description: 'Arrow Right', aliases: ['KC_RIGHT', 'VK_RIGHT', 'KEY_RIGHT'] },

  // ============================================================
  // BASIC KEYS - PUNCTUATION
  // ============================================================
  { id: 'Minus', label: '-', category: 'basic', subcategory: 'punctuation', description: 'Minus/Hyphen', aliases: ['KC_MINS', 'KC_MINUS', 'VK_OEM_MINUS', 'KEY_MINUS'] },
  { id: 'Equal', label: '=', category: 'basic', subcategory: 'punctuation', description: 'Equals', aliases: ['KC_EQL', 'KC_EQUAL', 'VK_OEM_PLUS', 'KEY_EQUAL'] },
  { id: 'LeftBracket', label: '[', category: 'basic', subcategory: 'punctuation', description: 'Left Bracket', aliases: ['KC_LBRC', 'KC_LBRACKET', 'VK_OEM_4', 'KEY_LEFTBRACE'] },
  { id: 'RightBracket', label: ']', category: 'basic', subcategory: 'punctuation', description: 'Right Bracket', aliases: ['KC_RBRC', 'KC_RBRACKET', 'VK_OEM_6', 'KEY_RIGHTBRACE'] },
  { id: 'Backslash', label: '\\', category: 'basic', subcategory: 'punctuation', description: 'Backslash', aliases: ['KC_BSLS', 'KC_BACKSLASH', 'VK_OEM_5', 'KEY_BACKSLASH'] },
  { id: 'Semicolon', label: ';', category: 'basic', subcategory: 'punctuation', description: 'Semicolon', aliases: ['KC_SCLN', 'KC_SEMICOLON', 'VK_OEM_1', 'KEY_SEMICOLON'] },
  { id: 'Quote', label: "'", category: 'basic', subcategory: 'punctuation', description: 'Quote/Apostrophe', aliases: ['KC_QUOT', 'KC_QUOTE', 'VK_OEM_7', 'KEY_APOSTROPHE'] },
  { id: 'Grave', label: '`', category: 'basic', subcategory: 'punctuation', description: 'Grave/Backtick', aliases: ['KC_GRV', 'KC_GRAVE', 'VK_OEM_3', 'KEY_GRAVE'] },
  { id: 'Comma', label: ',', category: 'basic', subcategory: 'punctuation', description: 'Comma', aliases: ['KC_COMM', 'KC_COMMA', 'VK_OEM_COMMA', 'KEY_COMMA'] },
  { id: 'Period', label: '.', category: 'basic', subcategory: 'punctuation', description: 'Period/Dot', aliases: ['KC_DOT', 'KC_PERIOD', 'VK_OEM_PERIOD', 'KEY_DOT'] },
  { id: 'Slash', label: '/', category: 'basic', subcategory: 'punctuation', description: 'Slash', aliases: ['KC_SLSH', 'KC_SLASH', 'VK_OEM_2', 'KEY_SLASH'] },

  // ============================================================
  // BASIC KEYS - NUMPAD
  // ============================================================
  { id: 'Numpad0', label: 'Num 0', category: 'basic', subcategory: 'numpad', description: 'Numpad 0', aliases: ['KC_P0', 'KC_KP_0', 'VK_NUMPAD0', 'KEY_KP0'] },
  { id: 'Numpad1', label: 'Num 1', category: 'basic', subcategory: 'numpad', description: 'Numpad 1', aliases: ['KC_P1', 'KC_KP_1', 'VK_NUMPAD1', 'KEY_KP1'] },
  { id: 'Numpad2', label: 'Num 2', category: 'basic', subcategory: 'numpad', description: 'Numpad 2', aliases: ['KC_P2', 'KC_KP_2', 'VK_NUMPAD2', 'KEY_KP2'] },
  { id: 'Numpad3', label: 'Num 3', category: 'basic', subcategory: 'numpad', description: 'Numpad 3', aliases: ['KC_P3', 'KC_KP_3', 'VK_NUMPAD3', 'KEY_KP3'] },
  { id: 'Numpad4', label: 'Num 4', category: 'basic', subcategory: 'numpad', description: 'Numpad 4', aliases: ['KC_P4', 'KC_KP_4', 'VK_NUMPAD4', 'KEY_KP4'] },
  { id: 'Numpad5', label: 'Num 5', category: 'basic', subcategory: 'numpad', description: 'Numpad 5', aliases: ['KC_P5', 'KC_KP_5', 'VK_NUMPAD5', 'KEY_KP5'] },
  { id: 'Numpad6', label: 'Num 6', category: 'basic', subcategory: 'numpad', description: 'Numpad 6', aliases: ['KC_P6', 'KC_KP_6', 'VK_NUMPAD6', 'KEY_KP6'] },
  { id: 'Numpad7', label: 'Num 7', category: 'basic', subcategory: 'numpad', description: 'Numpad 7', aliases: ['KC_P7', 'KC_KP_7', 'VK_NUMPAD7', 'KEY_KP7'] },
  { id: 'Numpad8', label: 'Num 8', category: 'basic', subcategory: 'numpad', description: 'Numpad 8', aliases: ['KC_P8', 'KC_KP_8', 'VK_NUMPAD8', 'KEY_KP8'] },
  { id: 'Numpad9', label: 'Num 9', category: 'basic', subcategory: 'numpad', description: 'Numpad 9', aliases: ['KC_P9', 'KC_KP_9', 'VK_NUMPAD9', 'KEY_KP9'] },
  { id: 'NumpadMultiply', label: 'Num *', category: 'basic', subcategory: 'numpad', description: 'Numpad Multiply', aliases: ['KC_PAST', 'KC_KP_ASTERISK', 'VK_MULTIPLY', 'KEY_KPASTERISK'] },
  { id: 'NumpadMinus', label: 'Num -', category: 'basic', subcategory: 'numpad', description: 'Numpad Minus', aliases: ['KC_PMNS', 'KC_KP_MINUS', 'VK_SUBTRACT', 'KEY_KPMINUS'] },
  { id: 'NumpadPlus', label: 'Num +', category: 'basic', subcategory: 'numpad', description: 'Numpad Plus', aliases: ['KC_PPLS', 'KC_KP_PLUS', 'VK_ADD', 'KEY_KPPLUS'] },
  { id: 'NumpadDot', label: 'Num .', category: 'basic', subcategory: 'numpad', description: 'Numpad Decimal', aliases: ['KC_PDOT', 'KC_KP_DOT', 'VK_DECIMAL', 'KEY_KPDOT'] },
  { id: 'NumpadSlash', label: 'Num /', category: 'basic', subcategory: 'numpad', description: 'Numpad Divide', aliases: ['KC_PSLS', 'KC_KP_SLASH', 'VK_DIVIDE', 'KEY_KPSLASH'] },
  { id: 'NumpadEnter', label: 'Num Enter', category: 'basic', subcategory: 'numpad', description: 'Numpad Enter', aliases: ['KC_PENT', 'KC_KP_ENTER', 'VK_RETURN', 'KEY_KPENTER'] },
  { id: 'NumpadEqual', label: 'Num =', category: 'basic', subcategory: 'numpad', description: 'Numpad Equal', aliases: ['KC_PEQL', 'KC_KP_EQUAL', 'KEY_KPEQUAL'] },

  // ============================================================
  // MODIFIERS
  // ============================================================
  { id: 'LCtrl', label: 'LCtrl', category: 'modifiers', description: 'Left Control', aliases: ['KC_LCTL', 'KC_LCTRL', 'VK_LCONTROL', 'KEY_LEFTCTRL'] },
  { id: 'RCtrl', label: 'RCtrl', category: 'modifiers', description: 'Right Control', aliases: ['KC_RCTL', 'KC_RCTRL', 'VK_RCONTROL', 'KEY_RIGHTCTRL'] },
  { id: 'LShift', label: 'LShift', category: 'modifiers', description: 'Left Shift', aliases: ['KC_LSFT', 'KC_LSHIFT', 'VK_LSHIFT', 'KEY_LEFTSHIFT'] },
  { id: 'RShift', label: 'RShift', category: 'modifiers', description: 'Right Shift', aliases: ['KC_RSFT', 'KC_RSHIFT', 'VK_RSHIFT', 'KEY_RIGHTSHIFT'] },
  { id: 'LAlt', label: 'LAlt', category: 'modifiers', description: 'Left Alt/Option', aliases: ['KC_LALT', 'VK_LMENU', 'KEY_LEFTALT'] },
  { id: 'RAlt', label: 'RAlt', category: 'modifiers', description: 'Right Alt/Option', aliases: ['KC_RALT', 'VK_RMENU', 'KEY_RIGHTALT', 'KC_ALGR'] },
  { id: 'LMeta', label: 'LWin', category: 'modifiers', description: 'Left Windows/Super/Command', aliases: ['KC_LGUI', 'KC_LWIN', 'KC_LCMD', 'VK_LWIN', 'KEY_LEFTMETA'] },
  { id: 'RMeta', label: 'RWin', category: 'modifiers', description: 'Right Windows/Super/Command', aliases: ['KC_RGUI', 'KC_RWIN', 'KC_RCMD', 'VK_RWIN', 'KEY_RIGHTMETA'] },
  { id: 'MD_00', label: 'MD_00', category: 'modifiers', description: 'Custom Modifier 0', aliases: [] },
  { id: 'MD_01', label: 'MD_01', category: 'modifiers', description: 'Custom Modifier 1', aliases: [] },
  { id: 'MD_02', label: 'MD_02', category: 'modifiers', description: 'Custom Modifier 2', aliases: [] },
  { id: 'MD_03', label: 'MD_03', category: 'modifiers', description: 'Custom Modifier 3', aliases: [] },
  { id: 'MD_04', label: 'MD_04', category: 'modifiers', description: 'Custom Modifier 4', aliases: [] },
  { id: 'MD_05', label: 'MD_05', category: 'modifiers', description: 'Custom Modifier 5', aliases: [] },
  { id: 'MD_06', label: 'MD_06', category: 'modifiers', description: 'Custom Modifier 6', aliases: [] },
  { id: 'MD_07', label: 'MD_07', category: 'modifiers', description: 'Custom Modifier 7', aliases: [] },
  { id: 'MD_08', label: 'MD_08', category: 'modifiers', description: 'Custom Modifier 8', aliases: [] },
  { id: 'MD_09', label: 'MD_09', category: 'modifiers', description: 'Custom Modifier 9', aliases: [] },

  // ============================================================
  // MEDIA KEYS
  // ============================================================
  { id: 'MediaPlayPause', label: 'Play/Pause', category: 'media', description: 'Media Play/Pause', aliases: ['KC_MPLY', 'VK_MEDIA_PLAY_PAUSE', 'KEY_PLAYPAUSE'] },
  { id: 'MediaStop', label: 'Stop', category: 'media', description: 'Media Stop', aliases: ['KC_MSTP', 'VK_MEDIA_STOP', 'KEY_STOPCD'] },
  { id: 'MediaPrev', label: 'Previous', category: 'media', description: 'Media Previous Track', aliases: ['KC_MPRV', 'VK_MEDIA_PREV_TRACK', 'KEY_PREVIOUSSONG'] },
  { id: 'MediaNext', label: 'Next', category: 'media', description: 'Media Next Track', aliases: ['KC_MNXT', 'VK_MEDIA_NEXT_TRACK', 'KEY_NEXTSONG'] },
  { id: 'VolumeUp', label: 'Vol+', category: 'media', description: 'Volume Up', aliases: ['KC_VOLU', 'VK_VOLUME_UP', 'KEY_VOLUMEUP'] },
  { id: 'VolumeDown', label: 'Vol-', category: 'media', description: 'Volume Down', aliases: ['KC_VOLD', 'VK_VOLUME_DOWN', 'KEY_VOLUMEDOWN'] },
  { id: 'VolumeMute', label: 'Mute', category: 'media', description: 'Volume Mute', aliases: ['KC_MUTE', 'VK_VOLUME_MUTE', 'KEY_MUTE'] },
  { id: 'BrightnessUp', label: 'Bright+', category: 'media', description: 'Brightness Up', aliases: ['KC_BRIU', 'KEY_BRIGHTNESSUP'] },
  { id: 'BrightnessDown', label: 'Bright-', category: 'media', description: 'Brightness Down', aliases: ['KC_BRID', 'KEY_BRIGHTNESSDOWN'] },

  // ============================================================
  // MACRO KEYS
  // ============================================================
  { id: 'M0', label: 'M0', category: 'macro', description: 'User Macro 0', aliases: [] },
  { id: 'M1', label: 'M1', category: 'macro', description: 'User Macro 1', aliases: [] },
  { id: 'M2', label: 'M2', category: 'macro', description: 'User Macro 2', aliases: [] },
  { id: 'M3', label: 'M3', category: 'macro', description: 'User Macro 3', aliases: [] },
  { id: 'M4', label: 'M4', category: 'macro', description: 'User Macro 4', aliases: [] },
  { id: 'M5', label: 'M5', category: 'macro', description: 'User Macro 5', aliases: [] },
  { id: 'M6', label: 'M6', category: 'macro', description: 'User Macro 6', aliases: [] },
  { id: 'M7', label: 'M7', category: 'macro', description: 'User Macro 7', aliases: [] },
  { id: 'M8', label: 'M8', category: 'macro', description: 'User Macro 8', aliases: [] },
  { id: 'M9', label: 'M9', category: 'macro', description: 'User Macro 9', aliases: [] },
  { id: 'M10', label: 'M10', category: 'macro', description: 'User Macro 10', aliases: [] },
  { id: 'M11', label: 'M11', category: 'macro', description: 'User Macro 11', aliases: [] },
  { id: 'M12', label: 'M12', category: 'macro', description: 'User Macro 12', aliases: [] },
  { id: 'M13', label: 'M13', category: 'macro', description: 'User Macro 13', aliases: [] },
  { id: 'M14', label: 'M14', category: 'macro', description: 'User Macro 14', aliases: [] },
  { id: 'M15', label: 'M15', category: 'macro', description: 'User Macro 15', aliases: [] },

  // ============================================================
  // LAYER KEYS
  // ============================================================
  { id: 'Layer0', label: 'Base', category: 'layers', description: 'Base Layer (MD_00)', aliases: ['MD_00', 'L0'] },
  { id: 'Layer1', label: 'Layer 1', category: 'layers', description: 'Layer 1 (MD_01)', aliases: ['MD_01', 'L1'] },
  { id: 'Layer2', label: 'Layer 2', category: 'layers', description: 'Layer 2 (MD_02)', aliases: ['MD_02', 'L2'] },
  { id: 'Layer3', label: 'Layer 3', category: 'layers', description: 'Layer 3 (MD_03)', aliases: ['MD_03', 'L3'] },
  { id: 'Layer4', label: 'Layer 4', category: 'layers', description: 'Layer 4 (MD_04)', aliases: ['MD_04', 'L4'] },
  { id: 'Layer5', label: 'Layer 5', category: 'layers', description: 'Layer 5 (MD_05)', aliases: ['MD_05', 'L5'] },
  { id: 'Layer6', label: 'Layer 6', category: 'layers', description: 'Layer 6 (MD_06)', aliases: ['MD_06', 'L6'] },
  { id: 'Layer7', label: 'Layer 7', category: 'layers', description: 'Layer 7 (MD_07)', aliases: ['MD_07', 'L7'] },
  { id: 'Layer8', label: 'Layer 8', category: 'layers', description: 'Layer 8 (MD_08)', aliases: ['MD_08', 'L8'] },
  { id: 'Layer9', label: 'Layer 9', category: 'layers', description: 'Layer 9 (MD_09)', aliases: ['MD_09', 'L9'] },

  // ============================================================
  // SPECIAL KEYS (Locks, System Keys)
  // ============================================================
  { id: 'LK_00', label: 'CapsLock', category: 'special', description: 'Caps Lock', aliases: ['KC_CAPS', 'VK_CAPITAL', 'KEY_CAPSLOCK'] },
  { id: 'LK_01', label: 'NumLock', category: 'special', description: 'Num Lock', aliases: ['KC_NLCK', 'VK_NUMLOCK', 'KEY_NUMLOCK'] },
  { id: 'LK_02', label: 'ScrollLock', category: 'special', description: 'Scroll Lock', aliases: ['KC_SLCK', 'VK_SCROLL', 'KEY_SCROLLLOCK'] },
  { id: 'LK_03', label: 'LK_03', category: 'special', description: 'Custom Lock 3', aliases: [] },
  { id: 'LK_04', label: 'LK_04', category: 'special', description: 'Custom Lock 4', aliases: [] },
  { id: 'LK_05', label: 'LK_05', category: 'special', description: 'Custom Lock 5', aliases: [] },
  { id: 'LK_06', label: 'LK_06', category: 'special', description: 'Custom Lock 6', aliases: [] },
  { id: 'LK_07', label: 'LK_07', category: 'special', description: 'Custom Lock 7', aliases: [] },
  { id: 'LK_08', label: 'LK_08', category: 'special', description: 'Custom Lock 8', aliases: [] },
  { id: 'LK_09', label: 'LK_09', category: 'special', description: 'Custom Lock 9', aliases: [] },
  { id: 'PrintScreen', label: 'PrtSc', category: 'special', description: 'Print Screen/SysRq', aliases: ['KC_PSCR', 'KC_SYSRQ', 'VK_SNAPSHOT', 'KEY_SYSRQ'] },
  { id: 'Pause', label: 'Pause', category: 'special', description: 'Pause/Break', aliases: ['KC_PAUS', 'KC_BRK', 'VK_PAUSE', 'KEY_PAUSE'] },
  { id: 'Application', label: 'Menu', category: 'special', description: 'Application/Context Menu', aliases: ['KC_APP', 'VK_APPS', 'KEY_COMPOSE'] },
];

/**
 * Get keys by category
 */
export function getKeysByCategory(category: KeyDefinition['category']): KeyDefinition[] {
  return KEY_DEFINITIONS.filter(k => k.category === category);
}

/**
 * Get keys by subcategory
 */
export function getKeysBySubcategory(subcategory: string): KeyDefinition[] {
  return KEY_DEFINITIONS.filter(k => k.subcategory === subcategory);
}

/**
 * Get key by ID
 */
export function getKeyById(id: string): KeyDefinition | undefined {
  return KEY_DEFINITIONS.find(k => k.id === id);
}

/**
 * Search keys by query (fuzzy search across id, label, description, aliases)
 */
export function searchKeys(query: string): KeyDefinition[] {
  if (!query.trim()) return KEY_DEFINITIONS;

  const lowerQuery = query.toLowerCase();

  return KEY_DEFINITIONS.filter(key => {
    // Check ID
    if (key.id.toLowerCase().includes(lowerQuery)) return true;

    // Check label
    if (key.label.toLowerCase().includes(lowerQuery)) return true;

    // Check description
    if (key.description.toLowerCase().includes(lowerQuery)) return true;

    // Check aliases
    if (key.aliases.some(alias => alias.toLowerCase().includes(lowerQuery))) return true;

    return false;
  });
}

/**
 * Get all unique categories
 */
export function getCategories(): KeyDefinition['category'][] {
  return ['basic', 'modifiers', 'media', 'macro', 'layers', 'special', 'any'];
}

/**
 * Get all unique subcategories for a category
 */
export function getSubcategories(category: KeyDefinition['category']): string[] {
  const subcats = KEY_DEFINITIONS
    .filter(k => k.category === category && k.subcategory)
    .map(k => k.subcategory as string);
  return Array.from(new Set(subcats));
}
