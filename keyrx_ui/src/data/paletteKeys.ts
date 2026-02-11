import { KEY_DEFINITIONS } from './keys';
import type { PaletteKey } from '../components/KeyPalette';

// VIA-style key definitions with categories and subcategories
export const BASIC_KEYS: PaletteKey[] = [
  // Letters subcategory
  { id: 'A', label: 'A', category: 'basic', subcategory: 'letters' },
  { id: 'B', label: 'B', category: 'basic', subcategory: 'letters' },
  { id: 'C', label: 'C', category: 'basic', subcategory: 'letters' },
  { id: 'D', label: 'D', category: 'basic', subcategory: 'letters' },
  { id: 'E', label: 'E', category: 'basic', subcategory: 'letters' },
  { id: 'F', label: 'F', category: 'basic', subcategory: 'letters' },
  { id: 'G', label: 'G', category: 'basic', subcategory: 'letters' },
  { id: 'H', label: 'H', category: 'basic', subcategory: 'letters' },
  { id: 'I', label: 'I', category: 'basic', subcategory: 'letters' },
  { id: 'J', label: 'J', category: 'basic', subcategory: 'letters' },
  { id: 'K', label: 'K', category: 'basic', subcategory: 'letters' },
  { id: 'L', label: 'L', category: 'basic', subcategory: 'letters' },
  { id: 'M', label: 'M', category: 'basic', subcategory: 'letters' },
  { id: 'N', label: 'N', category: 'basic', subcategory: 'letters' },
  { id: 'O', label: 'O', category: 'basic', subcategory: 'letters' },
  { id: 'P', label: 'P', category: 'basic', subcategory: 'letters' },
  { id: 'Q', label: 'Q', category: 'basic', subcategory: 'letters' },
  { id: 'R', label: 'R', category: 'basic', subcategory: 'letters' },
  { id: 'S', label: 'S', category: 'basic', subcategory: 'letters' },
  { id: 'T', label: 'T', category: 'basic', subcategory: 'letters' },
  { id: 'U', label: 'U', category: 'basic', subcategory: 'letters' },
  { id: 'V', label: 'V', category: 'basic', subcategory: 'letters' },
  { id: 'W', label: 'W', category: 'basic', subcategory: 'letters' },
  { id: 'X', label: 'X', category: 'basic', subcategory: 'letters' },
  { id: 'Y', label: 'Y', category: 'basic', subcategory: 'letters' },
  { id: 'Z', label: 'Z', category: 'basic', subcategory: 'letters' },
  // Numbers subcategory
  { id: '0', label: '0', category: 'basic', subcategory: 'numbers' },
  { id: '1', label: '1', category: 'basic', subcategory: 'numbers' },
  { id: '2', label: '2', category: 'basic', subcategory: 'numbers' },
  { id: '3', label: '3', category: 'basic', subcategory: 'numbers' },
  { id: '4', label: '4', category: 'basic', subcategory: 'numbers' },
  { id: '5', label: '5', category: 'basic', subcategory: 'numbers' },
  { id: '6', label: '6', category: 'basic', subcategory: 'numbers' },
  { id: '7', label: '7', category: 'basic', subcategory: 'numbers' },
  { id: '8', label: '8', category: 'basic', subcategory: 'numbers' },
  { id: '9', label: '9', category: 'basic', subcategory: 'numbers' },
  // Function keys
  { id: 'F1', label: 'F1', category: 'basic', subcategory: 'function' },
  { id: 'F2', label: 'F2', category: 'basic', subcategory: 'function' },
  { id: 'F3', label: 'F3', category: 'basic', subcategory: 'function' },
  { id: 'F4', label: 'F4', category: 'basic', subcategory: 'function' },
  { id: 'F5', label: 'F5', category: 'basic', subcategory: 'function' },
  { id: 'F6', label: 'F6', category: 'basic', subcategory: 'function' },
  { id: 'F7', label: 'F7', category: 'basic', subcategory: 'function' },
  { id: 'F8', label: 'F8', category: 'basic', subcategory: 'function' },
  { id: 'F9', label: 'F9', category: 'basic', subcategory: 'function' },
  { id: 'F10', label: 'F10', category: 'basic', subcategory: 'function' },
  { id: 'F11', label: 'F11', category: 'basic', subcategory: 'function' },
  { id: 'F12', label: 'F12', category: 'basic', subcategory: 'function' },
  // Navigation subcategory
  {
    id: 'Escape',
    label: 'Esc',
    category: 'basic',
    subcategory: 'navigation',
    description: 'Escape key',
  },
  {
    id: 'Enter',
    label: 'Enter',
    category: 'basic',
    subcategory: 'navigation',
    description: 'Enter/Return',
  },
  {
    id: 'Space',
    label: 'Space',
    category: 'basic',
    subcategory: 'navigation',
    description: 'Space bar',
  },
  {
    id: 'Backspace',
    label: 'BS',
    category: 'basic',
    subcategory: 'navigation',
    description: 'Backspace',
  },
  {
    id: 'Tab',
    label: 'Tab',
    category: 'basic',
    subcategory: 'navigation',
    description: 'Tab key',
  },
  {
    id: 'Delete',
    label: 'Del',
    category: 'basic',
    subcategory: 'navigation',
    description: 'Delete',
  },
  {
    id: 'Insert',
    label: 'Ins',
    category: 'basic',
    subcategory: 'navigation',
    description: 'Insert',
  },
  {
    id: 'Home',
    label: 'Home',
    category: 'basic',
    subcategory: 'navigation',
    description: 'Home',
  },
  {
    id: 'End',
    label: 'End',
    category: 'basic',
    subcategory: 'navigation',
    description: 'End',
  },
  {
    id: 'PageUp',
    label: 'PgUp',
    category: 'basic',
    subcategory: 'navigation',
    description: 'Page Up',
  },
  {
    id: 'PageDown',
    label: 'PgDn',
    category: 'basic',
    subcategory: 'navigation',
    description: 'Page Down',
  },
  {
    id: 'Up',
    label: '↑',
    category: 'basic',
    subcategory: 'navigation',
    description: 'Arrow Up',
  },
  {
    id: 'Down',
    label: '↓',
    category: 'basic',
    subcategory: 'navigation',
    description: 'Arrow Down',
  },
  {
    id: 'Left',
    label: '←',
    category: 'basic',
    subcategory: 'navigation',
    description: 'Arrow Left',
  },
  {
    id: 'Right',
    label: '→',
    category: 'basic',
    subcategory: 'navigation',
    description: 'Arrow Right',
  },
  // Punctuation subcategory
  { id: 'Minus', label: '-', category: 'basic', subcategory: 'punctuation' },
  { id: 'Equal', label: '=', category: 'basic', subcategory: 'punctuation' },
  {
    id: 'LeftBracket',
    label: '[',
    category: 'basic',
    subcategory: 'punctuation',
  },
  {
    id: 'RightBracket',
    label: ']',
    category: 'basic',
    subcategory: 'punctuation',
  },
  {
    id: 'Backslash',
    label: '\\',
    category: 'basic',
    subcategory: 'punctuation',
  },
  {
    id: 'Semicolon',
    label: ';',
    category: 'basic',
    subcategory: 'punctuation',
  },
  { id: 'Quote', label: "'", category: 'basic', subcategory: 'punctuation' },
  { id: 'Comma', label: ',', category: 'basic', subcategory: 'punctuation' },
  { id: 'Period', label: '.', category: 'basic', subcategory: 'punctuation' },
  { id: 'Slash', label: '/', category: 'basic', subcategory: 'punctuation' },
];

export const MODIFIER_KEYS: PaletteKey[] = [
  {
    id: 'LCtrl',
    label: 'LCtrl',
    category: 'modifiers',
    description: 'Left Control',
  },
  {
    id: 'RCtrl',
    label: 'RCtrl',
    category: 'modifiers',
    description: 'Right Control',
  },
  {
    id: 'LShift',
    label: 'LShift',
    category: 'modifiers',
    description: 'Left Shift',
  },
  {
    id: 'RShift',
    label: 'RShift',
    category: 'modifiers',
    description: 'Right Shift',
  },
  { id: 'LAlt', label: 'LAlt', category: 'modifiers', description: 'Left Alt' },
  {
    id: 'RAlt',
    label: 'RAlt',
    category: 'modifiers',
    description: 'Right Alt',
  },
  {
    id: 'LMeta',
    label: 'LWin',
    category: 'modifiers',
    description: 'Left Windows/Super',
  },
  {
    id: 'RMeta',
    label: 'RWin',
    category: 'modifiers',
    description: 'Right Windows/Super',
  },
  // Generate all 256 custom modifiers (MD_00 to MD_FF)
  ...Array.from({ length: 256 }, (_, i) => {
    const hexValue = i.toString(16).toUpperCase().padStart(2, '0');
    return {
      id: `MD_${hexValue}`,
      label: `MD_${hexValue}`,
      category: 'modifiers' as const,
      description: `Custom Modifier ${hexValue} (0x${hexValue} / ${i})`,
    };
  }),
];

export const MEDIA_KEYS: PaletteKey[] = [
  // Placeholder for media keys (to be expanded in task 1.2)
];

export const MACRO_KEYS: PaletteKey[] = [
  // User-defined macros (M0-M15)
  { id: 'M0', label: 'M0', category: 'macro', description: 'Macro 0' },
  { id: 'M1', label: 'M1', category: 'macro', description: 'Macro 1' },
  { id: 'M2', label: 'M2', category: 'macro', description: 'Macro 2' },
  { id: 'M3', label: 'M3', category: 'macro', description: 'Macro 3' },
  { id: 'M4', label: 'M4', category: 'macro', description: 'Macro 4' },
  { id: 'M5', label: 'M5', category: 'macro', description: 'Macro 5' },
  { id: 'M6', label: 'M6', category: 'macro', description: 'Macro 6' },
  { id: 'M7', label: 'M7', category: 'macro', description: 'Macro 7' },
  { id: 'M8', label: 'M8', category: 'macro', description: 'Macro 8' },
  { id: 'M9', label: 'M9', category: 'macro', description: 'Macro 9' },
  { id: 'M10', label: 'M10', category: 'macro', description: 'Macro 10' },
  { id: 'M11', label: 'M11', category: 'macro', description: 'Macro 11' },
  { id: 'M12', label: 'M12', category: 'macro', description: 'Macro 12' },
  { id: 'M13', label: 'M13', category: 'macro', description: 'Macro 13' },
  { id: 'M14', label: 'M14', category: 'macro', description: 'Macro 14' },
  { id: 'M15', label: 'M15', category: 'macro', description: 'Macro 15' },
];

// LAYER_KEYS: Now sourced from KEY_DEFINITIONS for single source of truth
export const LAYER_KEYS: PaletteKey[] = KEY_DEFINITIONS.filter(
  (k) => k.category === 'layers'
).map((k) => ({
  id: k.id,
  label: k.label,
  category: k.category,
  subcategory: k.subcategory,
  description: k.description,
}));

export const SPECIAL_KEYS: PaletteKey[] = [
  {
    id: 'LK_00',
    label: 'CapsLock',
    category: 'special',
    description: 'Caps Lock (LK_00)',
  },
  {
    id: 'LK_01',
    label: 'NumLock',
    category: 'special',
    description: 'Num Lock (LK_01)',
  },
  {
    id: 'LK_02',
    label: 'ScrollLock',
    category: 'special',
    description: 'Scroll Lock (LK_02)',
  },
  {
    id: 'LK_03',
    label: 'LK_03',
    category: 'special',
    description: 'Custom Lock 3',
  },
  {
    id: 'LK_04',
    label: 'LK_04',
    category: 'special',
    description: 'Custom Lock 4',
  },
  {
    id: 'LK_05',
    label: 'LK_05',
    category: 'special',
    description: 'Custom Lock 5',
  },
  {
    id: 'LK_06',
    label: 'LK_06',
    category: 'special',
    description: 'Custom Lock 6',
  },
  {
    id: 'LK_07',
    label: 'LK_07',
    category: 'special',
    description: 'Custom Lock 7',
  },
  {
    id: 'LK_08',
    label: 'LK_08',
    category: 'special',
    description: 'Custom Lock 8',
  },
  {
    id: 'LK_09',
    label: 'LK_09',
    category: 'special',
    description: 'Custom Lock 9',
  },
];
