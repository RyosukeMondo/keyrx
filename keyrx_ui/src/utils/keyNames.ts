/**
 * Key naming: the single place that maps layout key codes (QMK-style `KC_*`
 * in the layout JSON) to the names the Rhai DSL and the daemon use (`VK_*`,
 * e.g. KC_CAPS -> VK_CapsLock), and that formats keys for display.
 *
 * User-facing text (aria-labels, tooltips, editor headings) must use
 * `dslKeyName()` so it matches what users type in `map("CapsLock", ...)`.
 */

import { getLocale } from '@/i18n';

/**
 * Normalize key code to VK_ format for mapping lookup
 * Maps QMK-style KC_ codes to system VK_ codes based on DSL manual
 *
 * Handles:
 * - KC_A -> VK_A (letters)
 * - KC_0-9 -> VK_Num0-9 (top row numbers)
 * - KC_P0-9 -> VK_Numpad0-9 (numpad digit keys)
 * - KC_NLCK -> VK_NumLock, etc. (numpad special keys)
 * - VK_A -> VK_A (already normalized)
 */
export function normalizeKeyCode(code: string): string {
  if (!code) return code;

  // Already in VK_ format
  if (code.startsWith('VK_')) return code;

  // Handle top row number keys: KC_0-KC_9 -> VK_Num0-VK_Num9
  if (code.match(/^KC_[0-9]$/)) {
    const digit = code.charAt(code.length - 1);
    return `VK_Num${digit}`;
  }

  // Handle numpad digit keys: KC_P0-KC_P9 -> VK_Numpad0-VK_Numpad9
  if (code.match(/^KC_P[0-9]$/)) {
    const digit = code.charAt(code.length - 1);
    return `VK_Numpad${digit}`;
  }

  // Handle special keys that need name translation (QMK → keyrx DSL)
  const specialKeyMap: Record<string, string> = {
    // Punctuation / symbol keys
    KC_LBRC: 'VK_LeftBracket',
    KC_RBRC: 'VK_RightBracket',
    KC_BSLS: 'VK_Backslash',
    KC_SCLN: 'VK_Semicolon',
    KC_QUOT: 'VK_Quote',
    KC_COMM: 'VK_Comma',
    KC_DOT: 'VK_Period',
    KC_SLSH: 'VK_Slash',
    KC_GRV: 'VK_Grave',
    KC_MINS: 'VK_Minus',
    KC_EQL: 'VK_Equal',
    // Control keys
    KC_ESC: 'VK_Escape',
    KC_TAB: 'VK_Tab',
    KC_CAPS: 'VK_CapsLock',
    KC_SPC: 'VK_Space',
    KC_ENT: 'VK_Enter',
    KC_BSPC: 'VK_Backspace',
    KC_DEL: 'VK_Delete',
    KC_INS: 'VK_Insert',
    KC_HOME: 'VK_Home',
    KC_END: 'VK_End',
    KC_PGUP: 'VK_PageUp',
    KC_PGDN: 'VK_PageDown',
    KC_UP: 'VK_Up',
    KC_DOWN: 'VK_Down',
    KC_LEFT: 'VK_Left',
    KC_RGHT: 'VK_Right',
    KC_PSCR: 'VK_PrintScreen',
    KC_SCRL: 'VK_ScrollLock',
    KC_PAUS: 'VK_Pause',
    // Modifier keys
    KC_LSFT: 'VK_LShift',
    KC_RSFT: 'VK_RShift',
    KC_LCTL: 'VK_LCtrl',
    KC_RCTL: 'VK_RCtrl',
    KC_LALT: 'VK_LAlt',
    KC_RALT: 'VK_RAlt',
    KC_LGUI: 'VK_LMeta',
    KC_RGUI: 'VK_RMeta',
    // Function keys
    KC_F1: 'VK_F1',
    KC_F2: 'VK_F2',
    KC_F3: 'VK_F3',
    KC_F4: 'VK_F4',
    KC_F5: 'VK_F5',
    KC_F6: 'VK_F6',
    KC_F7: 'VK_F7',
    KC_F8: 'VK_F8',
    KC_F9: 'VK_F9',
    KC_F10: 'VK_F10',
    KC_F11: 'VK_F11',
    KC_F12: 'VK_F12',
    // Numpad special keys
    KC_NLCK: 'VK_NumLock',
    KC_PSLS: 'VK_NumpadDivide',
    KC_PAST: 'VK_NumpadMultiply',
    KC_PMNS: 'VK_NumpadSubtract',
    KC_PPLS: 'VK_NumpadAdd',
    KC_PENT: 'VK_NumpadEnter',
    KC_PDOT: 'VK_NumpadDecimal',
    // JIS-specific keys
    KC_JYEN: 'VK_Yen',
    KC_RO: 'VK_Ro',
    KC_MHEN: 'VK_Muhenkan',
    KC_HENK: 'VK_Henkan',
    KC_KANA: 'VK_Hiragana',
  };

  if (specialKeyMap[code]) {
    return specialKeyMap[code];
  }

  // Convert KC_ to VK_
  if (code.startsWith('KC_')) return code.replace(/^KC_/, 'VK_');

  // No prefix - add VK_
  return `VK_${code}`;
}

/**
 * Format key label for display
 */
export function formatKeyLabel(key: string): string {
  if (!key) return '';

  // Handle with_* helper functions
  const withMatch = key.match(/^with_(\w+)\(["']?(\w+)["']?\)$/);
  if (withMatch) {
    const [, modifier, innerKey] = withMatch;
    const modSymbols: Record<string, string> = {
      shift: '⇧',
      ctrl: '⌃',
      alt: '⌥',
      meta: '⌘',
      gui: '⌘',
    };
    const modSymbol =
      modSymbols[modifier.toLowerCase()] || modifier.charAt(0).toUpperCase();
    return `${modSymbol}${innerKey.replace(/^VK_/, '')}`;
  }

  const clean = key.replace(/^VK_/, '');
  const shortNames: Record<string, string> = {
    BACKSPACE: 'BS',
    CAPSLOCK: 'Caps',
    ESCAPE: 'Esc',
    DELETE: 'Del',
    INSERT: 'Ins',
    PAGEUP: 'PgUp',
    PAGEDOWN: 'PgDn',
    LEFTSHIFT: 'LShft',
    RIGHTSHIFT: 'RShft',
    LEFTCONTROL: 'LCtrl',
    RIGHTCONTROL: 'RCtrl',
    LEFTALT: 'LAlt',
    RIGHTALT: 'RAlt',
    NUMLOCK: 'Num',
    SCROLLLOCK: 'Scrl',
    PRINTSCREEN: 'PrtSc',
  };

  const upper = clean.toUpperCase();
  if (shortNames[upper]) return shortNames[upper];
  if (clean.length > 5) return clean.slice(0, 4) + '…';
  return clean;
}

/**
 * DSL name of a layout key code without the `VK_` prefix: KC_CAPS -> "CapsLock".
 * Layer ids are normalised to the one spelling the DSL uses (`LK_00`, `MD_0A`),
 * never `LK-00`.
 */
export function dslKeyName(code: string): string {
  const name = normalizeKeyCode(code).replace(/^VK_/, '');
  const id = /^(MD|LK)[-_]([0-9A-Fa-f]{2})$/.exec(name);
  return id ? `${id[1]}_${id[2].toUpperCase()}` : name;
}

const FRIENDLY_NAMES: Record<string, string> = {
  CapsLock: 'Caps Lock',
  LCtrl: 'Ctrl',
  RCtrl: 'Right Ctrl',
  LShift: 'Shift',
  RShift: 'Right Shift',
  LAlt: 'Alt',
  RAlt: 'Right Alt',
  LMeta: 'Win',
  RMeta: 'Right Win',
  Escape: 'Esc',
  Backspace: 'Backspace',
  PageUp: 'Page Up',
  PageDown: 'Page Down',
  PrintScreen: 'Print Screen',
  ScrollLock: 'Scroll Lock',
  NumLock: 'Num Lock',
  Grave: 'Backtick',
  Quote: 'Apostrophe',
  Equal: 'Equals',
  Period: 'Period',
  NumpadSubtract: 'Numpad Minus',
  NumpadAdd: 'Numpad Plus',
  NumpadDecimal: 'Numpad Point',
  NumpadMultiply: 'Numpad Multiply',
  NumpadDivide: 'Numpad Divide',
  Zenkaku: 'Zenkaku/Hankaku',
  KatakanaHiragana: 'Katakana/Hiragana',
};

/**
 * Japanese names for the JIS keys (and the few keys whose JIS legend differs).
 * `Grave` is the physical 半角/全角 key on a JIS board (the daemon maps scancode
 * 0x29 to Grave); `CapsLock` is 英数 there.
 */
const JA_KEY_NAMES: Record<string, string> = {
  Zenkaku: '半角/全角',
  Grave: '半角/全角（`）',
  Henkan: '変換',
  Muhenkan: '無変換',
  Hiragana: 'ひらがな',
  Katakana: 'カタカナ',
  KatakanaHiragana: 'かな',
  CapsLock: 'Caps Lock（英数）',
  Yen: '円記号（¥）',
  Ro: 'ろ（＼）',
};

const MODIFIER_WORDS: Record<string, string> = {
  shift: 'Shift',
  ctrl: 'Ctrl',
  alt: 'Alt',
  meta: 'Win',
  gui: 'Win',
};

/**
 * Plain-language key name for people, not for the DSL: `VK_CapsLock` ->
 * "Caps Lock", `VK_LCtrl` -> "Ctrl", `with_shift(VK_A)` -> "Shift+A".
 * Unknown names are split on word boundaries rather than guessed.
 */
export function friendlyKeyName(key: string): string {
  return keyNameParts(key).primary;
}

/**
 * Name to show people, plus (in Japanese) the English/DSL name to show smaller
 * next to it: `{primary: '無変換', secondary: 'Muhenkan'}`. `secondary` is only
 * set when it adds information (the primary is not already the DSL name).
 */
export interface KeyNameParts {
  primary: string;
  secondary?: string;
}

export function keyNameParts(key: string): KeyNameParts {
  const combo = key.match(/^with_(\w+)\(["']?([\w]+)["']?\)$/);
  if (combo) {
    const mod = MODIFIER_WORDS[combo[1].toLowerCase()] ?? combo[1];
    const inner = keyNameParts(combo[2]);
    return { primary: `${mod}+${inner.primary}`, secondary: inner.secondary };
  }
  const name = dslKeyName(key);
  const ja = getLocale() === 'ja' ? JA_KEY_NAMES[name] : undefined;
  if (ja) return { primary: ja, secondary: name };
  if (FRIENDLY_NAMES[name]) return { primary: FRIENDLY_NAMES[name] };
  const top = name.match(/^Num(\d)$/); // top-row digits
  if (top) return { primary: top[1] };
  if (/^[A-Za-z]$/.test(name)) return { primary: name.toUpperCase() };
  return {
    primary: name
      .replace(/([a-z])([A-Z])/g, '$1 $2')
      .replace(/([A-Za-z])(\d)/g, (m, a, d) => (/^F$/i.test(a) ? m : `${a} ${d}`)),
  };
}
