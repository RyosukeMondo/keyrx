/**
 * Special key definitions (macros, locks, layer keys, etc.)
 */
import { KeyDefinition } from './types';

export const MACROS: KeyDefinition[] = [
  {
    id: 'M0',
    label: 'M0',
    category: 'macro',
    description: 'User Macro 0',
    aliases: [],
  },
  {
    id: 'M1',
    label: 'M1',
    category: 'macro',
    description: 'User Macro 1',
    aliases: [],
  },
  {
    id: 'M2',
    label: 'M2',
    category: 'macro',
    description: 'User Macro 2',
    aliases: [],
  },
  {
    id: 'M3',
    label: 'M3',
    category: 'macro',
    description: 'User Macro 3',
    aliases: [],
  },
  {
    id: 'M4',
    label: 'M4',
    category: 'macro',
    description: 'User Macro 4',
    aliases: [],
  },
  {
    id: 'M5',
    label: 'M5',
    category: 'macro',
    description: 'User Macro 5',
    aliases: [],
  },
  {
    id: 'M6',
    label: 'M6',
    category: 'macro',
    description: 'User Macro 6',
    aliases: [],
  },
  {
    id: 'M7',
    label: 'M7',
    category: 'macro',
    description: 'User Macro 7',
    aliases: [],
  },
  {
    id: 'M8',
    label: 'M8',
    category: 'macro',
    description: 'User Macro 8',
    aliases: [],
  },
  {
    id: 'M9',
    label: 'M9',
    category: 'macro',
    description: 'User Macro 9',
    aliases: [],
  },
  {
    id: 'M10',
    label: 'M10',
    category: 'macro',
    description: 'User Macro 10',
    aliases: [],
  },
  {
    id: 'M11',
    label: 'M11',
    category: 'macro',
    description: 'User Macro 11',
    aliases: [],
  },
  {
    id: 'M12',
    label: 'M12',
    category: 'macro',
    description: 'User Macro 12',
    aliases: [],
  },
  {
    id: 'M13',
    label: 'M13',
    category: 'macro',
    description: 'User Macro 13',
    aliases: [],
  },
  {
    id: 'M14',
    label: 'M14',
    category: 'macro',
    description: 'User Macro 14',
    aliases: [],
  },
  {
    id: 'M15',
    label: 'M15',
    category: 'macro',
    description: 'User Macro 15',
    aliases: [],
  },
];

export const LOCKS: KeyDefinition[] = [
  {
    id: 'LK_00',
    label: 'CapsLock',
    category: 'special',
    description: 'Caps Lock',
    aliases: ['KC_CAPS', 'VK_CAPITAL', 'KEY_CAPSLOCK'],
  },
  {
    id: 'LK_01',
    label: 'NumLock',
    category: 'special',
    description: 'Num Lock',
    aliases: ['KC_NLCK', 'VK_NUMLOCK', 'KEY_NUMLOCK'],
  },
  {
    id: 'LK_02',
    label: 'ScrollLock',
    category: 'special',
    description: 'Scroll Lock',
    aliases: ['KC_SLCK', 'VK_SCROLL', 'KEY_SCROLLLOCK'],
  },
  {
    id: 'LK_03',
    label: 'LK_03',
    category: 'special',
    description: 'Custom Lock 3',
    aliases: [],
  },
  {
    id: 'LK_04',
    label: 'LK_04',
    category: 'special',
    description: 'Custom Lock 4',
    aliases: [],
  },
  {
    id: 'LK_05',
    label: 'LK_05',
    category: 'special',
    description: 'Custom Lock 5',
    aliases: [],
  },
  {
    id: 'LK_06',
    label: 'LK_06',
    category: 'special',
    description: 'Custom Lock 6',
    aliases: [],
  },
  {
    id: 'LK_07',
    label: 'LK_07',
    category: 'special',
    description: 'Custom Lock 7',
    aliases: [],
  },
  {
    id: 'LK_08',
    label: 'LK_08',
    category: 'special',
    description: 'Custom Lock 8',
    aliases: [],
  },
  {
    id: 'LK_09',
    label: 'LK_09',
    category: 'special',
    description: 'Custom Lock 9',
    aliases: [],
  },
  {
    id: 'PrintScreen',
    label: 'PrtSc',
    category: 'special',
    description: 'Print Screen/SysRq',
    aliases: ['KC_PSCR', 'KC_SYSRQ', 'VK_SNAPSHOT', 'KEY_SYSRQ'],
  },
  {
    id: 'Pause',
    label: 'Pause',
    category: 'special',
    description: 'Pause/Break',
    aliases: ['KC_PAUS', 'KC_BRK', 'VK_PAUSE', 'KEY_PAUSE'],
  },
  {
    id: 'Application',
    label: 'Menu',
    category: 'special',
    description: 'Application/Context Menu',
    aliases: ['KC_APP', 'VK_APPS', 'KEY_COMPOSE'],
  },
];

export const LAYER_KEYS: KeyDefinition[] = [
  {
    id: 'Layer0',
    label: 'Base',
    category: 'layers',
    subcategory: 'basic',
    description: 'Base Layer (MD_00)',
    aliases: ['MD_00', 'L0'],
  },
  {
    id: 'Layer1',
    label: 'Layer 1',
    category: 'layers',
    subcategory: 'basic',
    description: 'Layer 1 (MD_01)',
    aliases: ['MD_01', 'L1'],
  },
  {
    id: 'Layer2',
    label: 'Layer 2',
    category: 'layers',
    subcategory: 'basic',
    description: 'Layer 2 (MD_02)',
    aliases: ['MD_02', 'L2'],
  },
  {
    id: 'Layer3',
    label: 'Layer 3',
    category: 'layers',
    subcategory: 'basic',
    description: 'Layer 3 (MD_03)',
    aliases: ['MD_03', 'L3'],
  },
  {
    id: 'Layer4',
    label: 'Layer 4',
    category: 'layers',
    subcategory: 'basic',
    description: 'Layer 4 (MD_04)',
    aliases: ['MD_04', 'L4'],
  },
  {
    id: 'Layer5',
    label: 'Layer 5',
    category: 'layers',
    subcategory: 'basic',
    description: 'Layer 5 (MD_05)',
    aliases: ['MD_05', 'L5'],
  },
  {
    id: 'Layer6',
    label: 'Layer 6',
    category: 'layers',
    subcategory: 'basic',
    description: 'Layer 6 (MD_06)',
    aliases: ['MD_06', 'L6'],
  },
  {
    id: 'Layer7',
    label: 'Layer 7',
    category: 'layers',
    subcategory: 'basic',
    description: 'Layer 7 (MD_07)',
    aliases: ['MD_07', 'L7'],
  },
  {
    id: 'Layer8',
    label: 'Layer 8',
    category: 'layers',
    subcategory: 'basic',
    description: 'Layer 8 (MD_08)',
    aliases: ['MD_08', 'L8'],
  },
  {
    id: 'Layer9',
    label: 'Layer 9',
    category: 'layers',
    subcategory: 'basic',
    description: 'Layer 9 (MD_09)',
    aliases: ['MD_09', 'L9'],
  },
];

// Momentary layer keys (MO)
const MOMENTARY_KEYS: KeyDefinition[] = Array.from({ length: 16 }, (_, i) => ({
  id: `MO(${i})`,
  label: `MO(${i})`,
  category: 'layers' as const,
  subcategory: 'momentary',
  description: `Momentary Layer ${i} - Hold to activate, release to deactivate`,
  aliases: [`MO${i}`],
}));

// Toggle To layer keys (TO)
const TOGGLE_TO_KEYS: KeyDefinition[] = Array.from({ length: 16 }, (_, i) => ({
  id: `TO(${i})`,
  label: `TO(${i})`,
  category: 'layers' as const,
  subcategory: 'toggle-to',
  description: `Toggle To Layer ${i} - Tap to switch to this layer permanently`,
  aliases: [`TO${i}`],
}));

// Toggle layer keys (TG)
const TOGGLE_KEYS: KeyDefinition[] = Array.from({ length: 16 }, (_, i) => ({
  id: `TG(${i})`,
  label: `TG(${i})`,
  category: 'layers' as const,
  subcategory: 'toggle',
  description: `Toggle Layer ${i} - Tap to toggle layer on/off`,
  aliases: [`TG${i}`],
}));

// One-shot layer keys (OSL)
const ONE_SHOT_KEYS: KeyDefinition[] = Array.from({ length: 16 }, (_, i) => ({
  id: `OSL(${i})`,
  label: `OSL(${i})`,
  category: 'layers' as const,
  subcategory: 'one-shot',
  description: `One-Shot Layer ${i} - Activate layer for the next key press only`,
  aliases: [`OSL${i}`],
}));

// Layer-tap keys (LT)
const LAYER_TAP_KEYS: KeyDefinition[] = [
  {
    id: 'LT(1,Space)',
    label: 'LT(1,Spc)',
    category: 'layers',
    subcategory: 'layer-tap',
    description: 'Layer-Tap: Hold for Layer 1, tap for Space',
    aliases: ['LT1SPC', 'LT(1,KC_SPC)'],
  },
  {
    id: 'LT(2,Space)',
    label: 'LT(2,Spc)',
    category: 'layers',
    subcategory: 'layer-tap',
    description: 'Layer-Tap: Hold for Layer 2, tap for Space',
    aliases: ['LT2SPC', 'LT(2,KC_SPC)'],
  },
  {
    id: 'LT(1,Enter)',
    label: 'LT(1,Ent)',
    category: 'layers',
    subcategory: 'layer-tap',
    description: 'Layer-Tap: Hold for Layer 1, tap for Enter',
    aliases: ['LT1ENT', 'LT(1,KC_ENT)'],
  },
  {
    id: 'LT(2,Enter)',
    label: 'LT(2,Ent)',
    category: 'layers',
    subcategory: 'layer-tap',
    description: 'Layer-Tap: Hold for Layer 2, tap for Enter',
    aliases: ['LT2ENT', 'LT(2,KC_ENT)'],
  },
  {
    id: 'LT(1,Backspace)',
    label: 'LT(1,BS)',
    category: 'layers',
    subcategory: 'layer-tap',
    description: 'Layer-Tap: Hold for Layer 1, tap for Backspace',
    aliases: ['LT1BS', 'LT(1,KC_BSPC)'],
  },
  {
    id: 'LT(2,Backspace)',
    label: 'LT(2,BS)',
    category: 'layers',
    subcategory: 'layer-tap',
    description: 'Layer-Tap: Hold for Layer 2, tap for Backspace',
    aliases: ['LT2BS', 'LT(2,KC_BSPC)'],
  },
  {
    id: 'LT(1,Tab)',
    label: 'LT(1,Tab)',
    category: 'layers',
    subcategory: 'layer-tap',
    description: 'Layer-Tap: Hold for Layer 1, tap for Tab',
    aliases: ['LT1TAB', 'LT(1,KC_TAB)'],
  },
  {
    id: 'LT(2,Tab)',
    label: 'LT(2,Tab)',
    category: 'layers',
    subcategory: 'layer-tap',
    description: 'Layer-Tap: Hold for Layer 2, tap for Tab',
    aliases: ['LT2TAB', 'LT(2,KC_TAB)'],
  },
  {
    id: 'LT(1,Escape)',
    label: 'LT(1,Esc)',
    category: 'layers',
    subcategory: 'layer-tap',
    description: 'Layer-Tap: Hold for Layer 1, tap for Escape',
    aliases: ['LT1ESC', 'LT(1,KC_ESC)'],
  },
  {
    id: 'LT(2,Escape)',
    label: 'LT(2,Esc)',
    category: 'layers',
    subcategory: 'layer-tap',
    description: 'Layer-Tap: Hold for Layer 2, tap for Escape',
    aliases: ['LT2ESC', 'LT(2,KC_ESC)'],
  },
];

export const ALL_LAYER_MODIFIERS = [
  ...LAYER_KEYS,
  ...MOMENTARY_KEYS,
  ...TOGGLE_TO_KEYS,
  ...TOGGLE_KEYS,
  ...ONE_SHOT_KEYS,
  ...LAYER_TAP_KEYS,
];
