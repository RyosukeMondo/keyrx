import React from 'react';
// Drag and drop removed - keys are now click-only for better UX clarity
import { Search, X, Star, Clock, Check, AlertCircle, HelpCircle, Grid3x3, List, Keyboard } from 'lucide-react';
import { Card } from './Card';
import { KEY_DEFINITIONS, KeyDefinition } from '../data/keyDefinitions';
import { KeyPaletteItem } from './KeyPaletteItem';

/**
 * Key Palette - Shows available keys/modifiers/layers for assignment
 * Based on VIA-style categories: Basic, Modifiers, Media, Macro, Layers, Special, Any
 */

export interface PaletteKey {
  id: string;
  label: string;
  category: 'basic' | 'modifiers' | 'media' | 'macro' | 'layers' | 'special' | 'any';
  subcategory?: string;
  description?: string;
}

interface SearchMatch {
  key: KeyDefinition;
  score: number;
  matches: {
    field: 'id' | 'label' | 'description' | 'alias';
    text: string;
    indices: number[];
  }[];
}

interface KeyPaletteProps {
  onKeySelect: (key: PaletteKey) => void;
  selectedKey?: PaletteKey | null;
  /** Compact mode for embedding in modals - reduced height, no header/recent/favorites */
  compact?: boolean;
}

/**
 * Calculate fuzzy match score and find matching character indices
 * Returns { score, indices } where score is higher for better matches
 */
function fuzzyMatch(text: string, query: string): { score: number; indices: number[] } | null {
  const textLower = text.toLowerCase();
  const queryLower = query.toLowerCase();

  // Exact match gets highest score
  if (textLower === queryLower) {
    return { score: 1000, indices: Array.from({ length: text.length }, (_, i) => i) };
  }

  // Starts with query gets high score
  if (textLower.startsWith(queryLower)) {
    return { score: 500, indices: Array.from({ length: query.length }, (_, i) => i) };
  }

  // Contains query gets medium score
  const containsIndex = textLower.indexOf(queryLower);
  if (containsIndex >= 0) {
    const indices = Array.from({ length: query.length }, (_, i) => containsIndex + i);
    return { score: 200 - containsIndex, indices };
  }

  // Fuzzy matching: find all query characters in order
  const indices: number[] = [];
  let textIdx = 0;
  let queryIdx = 0;
  let consecutiveMatches = 0;
  let score = 0;

  while (textIdx < textLower.length && queryIdx < queryLower.length) {
    if (textLower[textIdx] === queryLower[queryIdx]) {
      indices.push(textIdx);
      queryIdx++;
      consecutiveMatches++;
      // Bonus for consecutive matches
      score += 10 + consecutiveMatches;
    } else {
      consecutiveMatches = 0;
    }
    textIdx++;
  }

  // All query characters must be found
  if (queryIdx !== queryLower.length) {
    return null;
  }

  // Penalty for gaps between matches
  score -= (textIdx - queryLower.length) * 2;

  return { score: Math.max(score, 1), indices };
}

/**
 * Search keys with fuzzy matching across all fields
 */
function searchKeysWithFuzzy(query: string): SearchMatch[] {
  if (!query.trim()) return [];

  const results: SearchMatch[] = [];

  for (const key of KEY_DEFINITIONS) {
    const matches: SearchMatch['matches'] = [];
    let totalScore = 0;

    // Search in ID
    const idMatch = fuzzyMatch(key.id, query);
    if (idMatch) {
      matches.push({ field: 'id', text: key.id, indices: idMatch.indices });
      totalScore += idMatch.score * 2; // ID matches are important
    }

    // Search in label
    const labelMatch = fuzzyMatch(key.label, query);
    if (labelMatch) {
      matches.push({ field: 'label', text: key.label, indices: labelMatch.indices });
      totalScore += labelMatch.score * 1.5;
    }

    // Search in description
    const descMatch = fuzzyMatch(key.description, query);
    if (descMatch) {
      matches.push({ field: 'description', text: key.description, indices: descMatch.indices });
      totalScore += descMatch.score;
    }

    // Search in aliases
    for (const alias of key.aliases) {
      const aliasMatch = fuzzyMatch(alias, query);
      if (aliasMatch) {
        matches.push({ field: 'alias', text: alias, indices: aliasMatch.indices });
        totalScore += aliasMatch.score * 1.5;
        break; // Only count first matching alias
      }
    }

    if (matches.length > 0) {
      results.push({ key, score: totalScore, matches });
    }
  }

  // Sort by score descending
  return results.sort((a, b) => b.score - a.score);
}

/**
 * Highlight matching characters in text
 */
function highlightMatches(text: string, indices: number[]): React.ReactNode {
  if (indices.length === 0) return text;

  const result: React.ReactNode[] = [];
  let lastIndex = 0;

  // Create set for O(1) lookup
  const indexSet = new Set(indices);

  for (let i = 0; i < text.length; i++) {
    if (indexSet.has(i)) {
      // Add non-highlighted text before this match
      if (i > lastIndex) {
        result.push(text.slice(lastIndex, i));
      }
      // Add highlighted character
      result.push(
        <mark key={i} className="bg-yellow-400/40 text-yellow-200 font-semibold">
          {text[i]}
        </mark>
      );
      lastIndex = i + 1;
    }
  }

  // Add remaining text
  if (lastIndex < text.length) {
    result.push(text.slice(lastIndex));
  }

  return <>{result}</>;
}

// VIA-style key definitions with categories and subcategories
const BASIC_KEYS: PaletteKey[] = [
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
  { id: 'Escape', label: 'Esc', category: 'basic', subcategory: 'navigation', description: 'Escape key' },
  { id: 'Enter', label: 'Enter', category: 'basic', subcategory: 'navigation', description: 'Enter/Return' },
  { id: 'Space', label: 'Space', category: 'basic', subcategory: 'navigation', description: 'Space bar' },
  { id: 'Backspace', label: 'BS', category: 'basic', subcategory: 'navigation', description: 'Backspace' },
  { id: 'Tab', label: 'Tab', category: 'basic', subcategory: 'navigation', description: 'Tab key' },
  { id: 'Delete', label: 'Del', category: 'basic', subcategory: 'navigation', description: 'Delete' },
  { id: 'Insert', label: 'Ins', category: 'basic', subcategory: 'navigation', description: 'Insert' },
  { id: 'Home', label: 'Home', category: 'basic', subcategory: 'navigation', description: 'Home' },
  { id: 'End', label: 'End', category: 'basic', subcategory: 'navigation', description: 'End' },
  { id: 'PageUp', label: 'PgUp', category: 'basic', subcategory: 'navigation', description: 'Page Up' },
  { id: 'PageDown', label: 'PgDn', category: 'basic', subcategory: 'navigation', description: 'Page Down' },
  { id: 'Up', label: '↑', category: 'basic', subcategory: 'navigation', description: 'Arrow Up' },
  { id: 'Down', label: '↓', category: 'basic', subcategory: 'navigation', description: 'Arrow Down' },
  { id: 'Left', label: '←', category: 'basic', subcategory: 'navigation', description: 'Arrow Left' },
  { id: 'Right', label: '→', category: 'basic', subcategory: 'navigation', description: 'Arrow Right' },
  // Punctuation subcategory
  { id: 'Minus', label: '-', category: 'basic', subcategory: 'punctuation' },
  { id: 'Equal', label: '=', category: 'basic', subcategory: 'punctuation' },
  { id: 'LeftBracket', label: '[', category: 'basic', subcategory: 'punctuation' },
  { id: 'RightBracket', label: ']', category: 'basic', subcategory: 'punctuation' },
  { id: 'Backslash', label: '\\', category: 'basic', subcategory: 'punctuation' },
  { id: 'Semicolon', label: ';', category: 'basic', subcategory: 'punctuation' },
  { id: 'Quote', label: "'", category: 'basic', subcategory: 'punctuation' },
  { id: 'Comma', label: ',', category: 'basic', subcategory: 'punctuation' },
  { id: 'Period', label: '.', category: 'basic', subcategory: 'punctuation' },
  { id: 'Slash', label: '/', category: 'basic', subcategory: 'punctuation' },
];

const MODIFIER_KEYS: PaletteKey[] = [
  { id: 'LCtrl', label: 'LCtrl', category: 'modifiers', description: 'Left Control' },
  { id: 'RCtrl', label: 'RCtrl', category: 'modifiers', description: 'Right Control' },
  { id: 'LShift', label: 'LShift', category: 'modifiers', description: 'Left Shift' },
  { id: 'RShift', label: 'RShift', category: 'modifiers', description: 'Right Shift' },
  { id: 'LAlt', label: 'LAlt', category: 'modifiers', description: 'Left Alt' },
  { id: 'RAlt', label: 'RAlt', category: 'modifiers', description: 'Right Alt' },
  { id: 'LMeta', label: 'LWin', category: 'modifiers', description: 'Left Windows/Super' },
  { id: 'RMeta', label: 'RWin', category: 'modifiers', description: 'Right Windows/Super' },
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

const MEDIA_KEYS: PaletteKey[] = [
  // Placeholder for media keys (to be expanded in task 1.2)
];

const MACRO_KEYS: PaletteKey[] = [
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
const LAYER_KEYS: PaletteKey[] = KEY_DEFINITIONS
  .filter(k => k.category === 'layers')
  .map(k => ({
    id: k.id,
    label: k.label,
    category: k.category,
    subcategory: k.subcategory,
    description: k.description,
  }));

const SPECIAL_KEYS: PaletteKey[] = [
  { id: 'LK_00', label: 'CapsLock', category: 'special', description: 'Caps Lock (LK_00)' },
  { id: 'LK_01', label: 'NumLock', category: 'special', description: 'Num Lock (LK_01)' },
  { id: 'LK_02', label: 'ScrollLock', category: 'special', description: 'Scroll Lock (LK_02)' },
  { id: 'LK_03', label: 'LK_03', category: 'special', description: 'Custom Lock 3' },
  { id: 'LK_04', label: 'LK_04', category: 'special', description: 'Custom Lock 4' },
  { id: 'LK_05', label: 'LK_05', category: 'special', description: 'Custom Lock 5' },
  { id: 'LK_06', label: 'LK_06', category: 'special', description: 'Custom Lock 6' },
  { id: 'LK_07', label: 'LK_07', category: 'special', description: 'Custom Lock 7' },
  { id: 'LK_08', label: 'LK_08', category: 'special', description: 'Custom Lock 8' },
  { id: 'LK_09', label: 'LK_09', category: 'special', description: 'Custom Lock 9' },
];

/**
 * LocalStorage keys for persistence
 */
const STORAGE_KEY_RECENT = 'keyrx_recent_keys';
const STORAGE_KEY_FAVORITES = 'keyrx_favorite_keys';
const STORAGE_KEY_VIEW_MODE = 'keyrx_palette_view_mode';
const MAX_RECENT_KEYS = 10;

type ViewMode = 'grid' | 'list';

/**
 * Load array from localStorage with error handling
 */
function loadFromStorage(key: string): string[] {
  try {
    const stored = localStorage.getItem(key);
    if (stored) {
      const parsed = JSON.parse(stored);
      return Array.isArray(parsed) ? parsed : [];
    }
  } catch (err) {
    console.warn(`Failed to load ${key} from localStorage:`, err);
  }
  return [];
}

/**
 * Save array to localStorage with error handling
 */
function saveToStorage(key: string, data: string[]): void {
  try {
    localStorage.setItem(key, JSON.stringify(data));
  } catch (err) {
    console.error(`Failed to save ${key} to localStorage:`, err);
  }
}

/**
 * Load view mode from localStorage with error handling
 */
function loadViewMode(): ViewMode {
  try {
    const stored = localStorage.getItem(STORAGE_KEY_VIEW_MODE);
    if (stored === 'grid' || stored === 'list') {
      return stored;
    }
  } catch (err) {
    console.warn(`Failed to load view mode from localStorage:`, err);
  }
  return 'grid'; // Default to grid view
}

/**
 * Save view mode to localStorage with error handling
 */
function saveViewMode(mode: ViewMode): void {
  try {
    localStorage.setItem(STORAGE_KEY_VIEW_MODE, mode);
  } catch (err) {
    console.error(`Failed to save view mode to localStorage:`, err);
  }
}

/**
 * Find a key definition by ID
 */
function findKeyById(keyId: string): PaletteKey | null {
  // Search in KEY_DEFINITIONS first
  const keyDef = KEY_DEFINITIONS.find(k => k.id === keyId);
  if (keyDef) {
    return {
      id: keyDef.id,
      label: keyDef.label,
      category: keyDef.category,
      subcategory: keyDef.subcategory,
      description: keyDef.description,
    };
  }

  // Fallback: search in static key arrays
  const allKeys = [...BASIC_KEYS, ...MODIFIER_KEYS, ...MEDIA_KEYS, ...MACRO_KEYS, ...LAYER_KEYS, ...SPECIAL_KEYS];
  return allKeys.find(k => k.id === keyId) || null;
}

/**
 * Validation result for custom keycodes
 */
interface ValidationResult {
  valid: boolean;
  error?: string;
  normalizedId?: string;
  label?: string;
}

/**
 * Map DOM KeyboardEvent.code to our key ID
 * Uses KEY_DEFINITIONS aliases to find matching key
 */
function mapDomCodeToKeyId(domCode: string): PaletteKey | null {
  // Normalize the DOM code - it might be like "KeyA", "Digit1", etc.
  // We need to map these to our KEY_ format

  // First try direct match with aliases
  const keyDef = KEY_DEFINITIONS.find(k => k.aliases.includes(domCode));
  if (keyDef) {
    return {
      id: keyDef.id,
      label: keyDef.label,
      category: keyDef.category,
      subcategory: keyDef.subcategory,
      description: keyDef.description,
    };
  }

  // Try normalizing common DOM codes to KEY_ format
  let normalizedCode = domCode;

  // Handle KeyA -> KEY_A
  if (domCode.startsWith('Key')) {
    normalizedCode = `KEY_${domCode.slice(3)}`;
  }
  // Handle Digit0 -> KEY_0
  else if (domCode.startsWith('Digit')) {
    normalizedCode = `KEY_${domCode.slice(5)}`;
  }
  // Handle ArrowUp -> KEY_UP
  else if (domCode.startsWith('Arrow')) {
    normalizedCode = `KEY_${domCode.slice(5).toUpperCase()}`;
  }
  // Handle others directly
  else {
    normalizedCode = `KEY_${domCode.toUpperCase()}`;
  }

  // Try again with normalized code
  const normalizedKeyDef = KEY_DEFINITIONS.find(k => k.aliases.includes(normalizedCode));
  if (normalizedKeyDef) {
    return {
      id: normalizedKeyDef.id,
      label: normalizedKeyDef.label,
      category: normalizedKeyDef.category,
      subcategory: normalizedKeyDef.subcategory,
      description: normalizedKeyDef.description,
    };
  }

  return null;
}

/**
 * Validate QMK-style keycode syntax
 * Supports:
 * - Simple keys: A, KC_A, VK_A
 * - Modifiers: LCTL(KC_C), LSFT(A)
 * - Layer functions: MO(1), TO(2), TG(3), OSL(4)
 * - Layer-tap: LT(2,KC_SPC), LT(1,A)
 */
function validateCustomKeycode(input: string): ValidationResult {
  const trimmed = input.trim();

  if (!trimmed) {
    return { valid: false, error: 'Please enter a keycode' };
  }

  // Check if it's a simple key ID (matches existing key)
  const keyDef = KEY_DEFINITIONS.find(k =>
    k.id === trimmed || k.aliases.includes(trimmed)
  );

  if (keyDef) {
    return {
      valid: true,
      normalizedId: keyDef.id,
      label: keyDef.label,
    };
  }

  // Check for modifier combinations: LCTL(KC_C), LSFT(A), etc.
  const modifierPattern = /^(LCTL|RCTL|LSFT|RSFT|LALT|RALT|LMETA|RMETA)\(([A-Za-z0-9_]+)\)$/;
  const modMatch = trimmed.match(modifierPattern);
  if (modMatch) {
    const [, modifier, keyPart] = modMatch;
    // Validate inner key exists
    const innerKey = KEY_DEFINITIONS.find(k =>
      k.id === keyPart || k.aliases.includes(keyPart)
    );

    if (!innerKey) {
      return {
        valid: false,
        error: `Unknown key: ${keyPart}. Try KC_A, KC_ENTER, etc.`
      };
    }

    return {
      valid: true,
      normalizedId: trimmed,
      label: `${modifier}+${innerKey.label}`,
    };
  }

  // Check for layer functions: MO(n), TO(n), TG(n), OSL(n)
  const layerPattern = /^(MO|TO|TG|OSL)\((\d+)\)$/;
  const layerMatch = trimmed.match(layerPattern);
  if (layerMatch) {
    const [, func, layer] = layerMatch;
    const layerNum = parseInt(layer, 10);

    if (layerNum < 0 || layerNum > 15) {
      return {
        valid: false,
        error: 'Layer number must be between 0-15'
      };
    }

    const funcLabels: Record<string, string> = {
      MO: 'Hold Layer',
      TO: 'To Layer',
      TG: 'Toggle Layer',
      OSL: 'OneShot Layer',
    };

    return {
      valid: true,
      normalizedId: trimmed,
      label: `${funcLabels[func]} ${layer}`,
    };
  }

  // Check for layer-tap: LT(layer, key)
  const layerTapPattern = /^LT\((\d+),\s*([A-Za-z0-9_]+)\)$/;
  const ltMatch = trimmed.match(layerTapPattern);
  if (ltMatch) {
    const [, layer, keyPart] = ltMatch;
    const layerNum = parseInt(layer, 10);

    if (layerNum < 0 || layerNum > 15) {
      return {
        valid: false,
        error: 'Layer number must be between 0-15'
      };
    }

    // Validate inner key exists
    const innerKey = KEY_DEFINITIONS.find(k =>
      k.id === keyPart || k.aliases.includes(keyPart)
    );

    if (!innerKey) {
      return {
        valid: false,
        error: `Unknown key: ${keyPart}. Try KC_A, KC_ENTER, etc.`
      };
    }

    return {
      valid: true,
      normalizedId: trimmed,
      label: `LT${layer}/${innerKey.label}`,
    };
  }

  // Unknown pattern
  return {
    valid: false,
    error: 'Invalid syntax. Examples: KC_A, LCTL(KC_C), MO(1), LT(2,KC_SPC)',
  };
}

export function KeyPalette({ onKeySelect, selectedKey, compact = false }: KeyPaletteProps) {
  const [activeCategory, setActiveCategory] = React.useState<PaletteKey['category']>('basic');
  const [activeSubcategory, setActiveSubcategory] = React.useState<string | null>(null);
  const [searchQuery, setSearchQuery] = React.useState('');
  const [selectedSearchIndex, setSelectedSearchIndex] = React.useState(0);
  const searchInputRef = React.useRef<HTMLInputElement>(null);

  // Drag and drop state
  // Drag functionality removed for better UX clarity

  // View mode state
  const [viewMode, setViewMode] = React.useState<ViewMode>(() => loadViewMode());

  // Recent and Favorite keys state
  const [recentKeyIds, setRecentKeyIds] = React.useState<string[]>(() => loadFromStorage(STORAGE_KEY_RECENT));
  const [favoriteKeyIds, setFavoriteKeyIds] = React.useState<string[]>(() => loadFromStorage(STORAGE_KEY_FAVORITES));

  // Custom keycode input state (for "Any" category)
  const [customKeycode, setCustomKeycode] = React.useState('');
  const [customValidation, setCustomValidation] = React.useState<ValidationResult>({ valid: false });

  // Physical key capture state
  const [isCapturingKey, setIsCapturingKey] = React.useState(false);
  const [capturedKey, setCapturedKey] = React.useState<PaletteKey | null>(null);

  // Add key to recent list (max 10, most recent first)
  const addToRecent = React.useCallback((keyId: string) => {
    setRecentKeyIds(prev => {
      const filtered = prev.filter(id => id !== keyId);
      const updated = [keyId, ...filtered].slice(0, MAX_RECENT_KEYS);
      saveToStorage(STORAGE_KEY_RECENT, updated);
      return updated;
    });
  }, []);

  // Toggle favorite status
  const toggleFavorite = React.useCallback((keyId: string) => {
    setFavoriteKeyIds(prev => {
      const isFavorite = prev.includes(keyId);
      const updated = isFavorite
        ? prev.filter(id => id !== keyId)
        : [...prev, keyId];
      saveToStorage(STORAGE_KEY_FAVORITES, updated);
      return updated;
    });
  }, []);

  // Check if key is favorite
  const isFavorite = React.useCallback((keyId: string) => {
    return favoriteKeyIds.includes(keyId);
  }, [favoriteKeyIds]);

  // Toggle view mode
  const toggleViewMode = React.useCallback(() => {
    setViewMode(prev => {
      const newMode = prev === 'grid' ? 'list' : 'grid';
      saveViewMode(newMode);
      return newMode;
    });
  }, []);

  // Handle key selection with recent tracking
  const handleKeySelect = React.useCallback((key: PaletteKey) => {
    addToRecent(key.id);
    onKeySelect(key);
  }, [addToRecent, onKeySelect]);

  // Handle custom keycode input change
  const handleCustomKeycodeChange = React.useCallback((value: string) => {
    setCustomKeycode(value);
    const validation = validateCustomKeycode(value);
    setCustomValidation(validation);
  }, []);

  // Apply custom keycode
  const handleApplyCustomKeycode = React.useCallback(() => {
    if (customValidation.valid && customValidation.normalizedId && customValidation.label) {
      const customKey: PaletteKey = {
        id: customValidation.normalizedId,
        label: customValidation.label,
        category: 'any',
        description: `Custom keycode: ${customValidation.normalizedId}`,
      };
      handleKeySelect(customKey);
      setCustomKeycode('');
      setCustomValidation({ valid: false });
    }
  }, [customValidation, handleKeySelect]);

  // Start physical key capture mode
  const startKeyCapture = React.useCallback(() => {
    setIsCapturingKey(true);
    setCapturedKey(null);
  }, []);

  // Cancel key capture mode
  const cancelKeyCapture = React.useCallback(() => {
    setIsCapturingKey(false);
    setCapturedKey(null);
  }, []);

  // Confirm captured key
  const confirmCapturedKey = React.useCallback(() => {
    if (capturedKey) {
      handleKeySelect(capturedKey);
      setIsCapturingKey(false);
      setCapturedKey(null);
    }
  }, [capturedKey, handleKeySelect]);

  // Handle physical key press during capture mode
  React.useEffect(() => {
    if (!isCapturingKey) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();

      // Allow Escape to cancel
      if (e.code === 'Escape') {
        cancelKeyCapture();
        return;
      }

      // Map the DOM code to our key ID
      const mappedKey = mapDomCodeToKeyId(e.code);
      if (mappedKey) {
        setCapturedKey(mappedKey);
      } else {
        // Unknown key - show error state
        console.warn('Unknown key code:', e.code);
      }
    };

    // Add listener at document level to capture all keys
    document.addEventListener('keydown', handleKeyDown, true);

    return () => {
      document.removeEventListener('keydown', handleKeyDown, true);
    };
  }, [isCapturingKey, cancelKeyCapture]);

  // Get recent and favorite key objects
  const recentKeys = React.useMemo(() => {
    return recentKeyIds.map(id => findKeyById(id)).filter((k): k is PaletteKey => k !== null);
  }, [recentKeyIds]);

  const favoriteKeys = React.useMemo(() => {
    return favoriteKeyIds.map(id => findKeyById(id)).filter((k): k is PaletteKey => k !== null);
  }, [favoriteKeyIds]);

  const categories = [
    { id: 'basic' as const, label: 'Basic', keys: BASIC_KEYS, icon: '⌨️' },
    { id: 'modifiers' as const, label: 'Modifiers', keys: MODIFIER_KEYS, icon: '⌥' },
    { id: 'special' as const, label: 'Special', keys: SPECIAL_KEYS, icon: '⭐' },
    { id: 'any' as const, label: 'Any', keys: [], icon: '✏️' },
  ];

  // Search results (if query is active)
  const searchResults = React.useMemo(() => {
    return searchQuery.trim() ? searchKeysWithFuzzy(searchQuery) : [];
  }, [searchQuery]);

  // Reset selected index when search changes
  React.useEffect(() => {
    setSelectedSearchIndex(0);
  }, [searchQuery]);

  const activeCategoryData = categories.find(c => c.id === activeCategory);
  let activeKeys = activeCategoryData?.keys || [];

  // If searching, use search results instead
  const isSearching = searchQuery.trim().length > 0;

  // Filter by subcategory if one is active
  if (activeSubcategory && (activeCategory === 'basic' || activeCategory === 'layers') && !isSearching) {
    activeKeys = activeKeys.filter(k => k.subcategory === activeSubcategory);
  }

  // Get unique subcategories for Basic and Layers categories
  const subcategories = activeCategory === 'basic'
    ? Array.from(new Set(BASIC_KEYS.map(k => k.subcategory).filter(Boolean)))
    : activeCategory === 'layers'
    ? Array.from(new Set(LAYER_KEYS.map(k => k.subcategory).filter(Boolean)))
    : [];

  // Handle keyboard navigation in search results
  const handleSearchKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (searchResults.length === 0) return;

    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        setSelectedSearchIndex(prev => Math.min(prev + 1, searchResults.length - 1));
        break;
      case 'ArrowUp':
        e.preventDefault();
        setSelectedSearchIndex(prev => Math.max(prev - 1, 0));
        break;
      case 'Enter':
        e.preventDefault();
        if (searchResults[selectedSearchIndex]) {
          const match = searchResults[selectedSearchIndex];
          handleKeySelect({
            id: match.key.id,
            label: match.key.label,
            category: match.key.category,
            subcategory: match.key.subcategory,
            description: match.key.description,
          });
          setSearchQuery('');
        }
        break;
      case 'Escape':
        e.preventDefault();
        setSearchQuery('');
        searchInputRef.current?.blur();
        break;
    }
  };

  // Drag handlers removed - using click-only interaction now

  // Render a key item with star button using KeyPaletteItem component
  const renderKeyItem = (key: PaletteKey, onClick: () => void, showStar: boolean = true) => {
    const favorite = isFavorite(key.id);

    return (
      <KeyPaletteItem
        key={key.id}
        keyItem={key}
        isSelected={selectedKey?.id === key.id}
        isFavorite={favorite}
        showStar={showStar}
        viewMode={viewMode}
        onClick={onClick}
        onToggleFavorite={showStar ? () => toggleFavorite(key.id) : undefined}
      />
    );
  };

  return (
      <Card className={`flex flex-col ${compact ? 'h-full p-2' : 'h-full'}`}>
        {/* Header with title, capture button, and view toggle - hidden in compact mode */}
        {!compact && (
          <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-slate-100">Key Palette</h3>
          <div className="flex gap-2">
            {/* Capture Key button */}
            <button
              onClick={startKeyCapture}
              className="px-3 py-2 bg-primary-500 hover:bg-primary-600 text-white rounded-lg transition-colors flex items-center gap-2 text-sm font-medium"
              title="Press any physical key to select it"
              aria-label="Capture physical key"
            >
              <Keyboard className="w-4 h-4" />
              <span>Capture Key</span>
            </button>

            {/* View toggle buttons */}
            <div className="flex gap-1">
              <button
                onClick={toggleViewMode}
                className={`p-2 rounded-lg transition-colors ${
                  viewMode === 'grid'
                    ? 'bg-primary-500 text-white'
                    : 'bg-slate-700 text-slate-400 hover:bg-slate-600 hover:text-slate-300'
                }`}
                title="Grid view"
                aria-label="Grid view"
              >
                <Grid3x3 className="w-4 h-4" />
              </button>
              <button
                onClick={toggleViewMode}
                className={`p-2 rounded-lg transition-colors ${
                  viewMode === 'list'
                    ? 'bg-primary-500 text-white'
                    : 'bg-slate-700 text-slate-400 hover:bg-slate-600 hover:text-slate-300'
                }`}
                title="List view"
                aria-label="List view"
              >
                <List className="w-4 h-4" />
              </button>
            </div>
          </div>
        </div>
        )}

      {/* Favorites Section - hidden in compact mode */}
      {!compact && favoriteKeys.length > 0 && (
        <div className="mb-4">
          <div className="flex items-center gap-2 mb-2">
            <Star className="w-4 h-4 text-yellow-400 fill-yellow-400" />
            <h4 className="text-sm font-semibold text-slate-300">Favorites</h4>
          </div>
          <div className={`p-3 bg-slate-800/50 rounded-lg ${
            viewMode === 'grid'
              ? 'grid grid-cols-8 gap-2'
              : 'flex flex-col gap-2'
          }`}>
            {favoriteKeys.map(key => renderKeyItem(key, () => handleKeySelect(key), true))}
          </div>
        </div>
      )}

      {/* Recent Keys Section - hidden in compact mode */}
      {!compact && recentKeys.length > 0 && (
        <div className="mb-4">
          <div className="flex items-center gap-2 mb-2">
            <Clock className="w-4 h-4 text-slate-400" />
            <h4 className="text-sm font-semibold text-slate-300">Recent</h4>
          </div>
          <div className={`p-3 bg-slate-800/50 rounded-lg ${
            viewMode === 'grid'
              ? 'grid grid-cols-8 gap-2'
              : 'flex flex-col gap-2'
          }`}>
            {recentKeys.map(key => renderKeyItem(key, () => handleKeySelect(key), true))}
          </div>
        </div>
      )}

      {/* Empty state when no favorites or recent - hidden in compact mode */}
      {!compact && favoriteKeys.length === 0 && recentKeys.length === 0 && !searchQuery && (
        <div className="mb-4 p-3 bg-slate-800/30 rounded-lg border border-slate-700/50">
          <p className="text-xs text-slate-500 text-center">
            Star keys to add favorites. Recent keys will appear automatically.
          </p>
        </div>
      )}

      {/* Search Input */}
      <div className={`relative ${compact ? 'mb-2' : 'mb-4'}`}>
        <Search className={`absolute left-3 top-1/2 -translate-y-1/2 text-slate-400 ${compact ? 'w-3 h-3' : 'w-4 h-4'}`} />
        <input
          ref={searchInputRef}
          type="text"
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          onKeyDown={handleSearchKeyDown}
          placeholder={compact ? "Search keys..." : "Search keys (e.g., ctrl, enter, KC_A)..."}
          className={`w-full bg-slate-800 border border-slate-700 rounded-lg text-slate-100 placeholder-slate-500 focus:outline-none focus:border-primary-500 focus:ring-1 focus:ring-primary-500 ${
            compact ? 'pl-8 pr-8 py-1.5 text-xs' : 'pl-10 pr-10 py-2 text-sm'
          }`}
        />
        {searchQuery && (
          <button
            onClick={() => setSearchQuery('')}
            className="absolute right-3 top-1/2 -translate-y-1/2 text-slate-400 hover:text-slate-300 transition-colors"
            aria-label="Clear search"
          >
            <X className="w-4 h-4" />
          </button>
        )}
        {/* Search result count */}
        {searchQuery && (
          <div className="absolute -bottom-5 left-0 text-xs text-slate-400">
            {searchResults.length} {searchResults.length === 1 ? 'result' : 'results'}
          </div>
        )}
      </div>

      {/* Category Tabs */}
      {!isSearching && (
        <div className={`flex gap-1 border-b border-slate-700 overflow-x-auto ${compact ? 'mb-2' : 'mb-4'}`}>
        {categories.map(cat => (
          <button
            key={cat.id}
            onClick={() => {
              setActiveCategory(cat.id);
              setActiveSubcategory(null);
            }}
            className={`font-medium transition-colors whitespace-nowrap flex items-center gap-1 ${
              compact ? 'px-2 py-1 text-xs' : 'px-3 py-2 text-sm'
            } ${
              activeCategory === cat.id
                ? 'text-primary-400 border-b-2 border-primary-400'
                : 'text-slate-400 hover:text-slate-300'
            }`}
          >
            {!compact && <span>{cat.icon}</span>}
            <span>{cat.label}</span>
          </button>
        ))}
        </div>
      )}

      {/* Subcategory Pills (for Basic and Layers categories) - hidden in compact mode */}
      {!compact && !isSearching && subcategories.length > 0 && (
        <div className="flex gap-2 mb-3 flex-wrap">
          <button
            onClick={() => setActiveSubcategory(null)}
            className={`px-3 py-1 text-xs rounded-full transition-colors ${
              activeSubcategory === null
                ? 'bg-primary-500 text-white'
                : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
            }`}
          >
            All
          </button>
          {subcategories.map(sub => (
            <button
              key={sub}
              onClick={() => setActiveSubcategory(sub || null)}
              className={`px-3 py-1 text-xs rounded-full transition-colors capitalize ${
                activeSubcategory === sub
                  ? 'bg-primary-500 text-white'
                  : 'bg-slate-700 text-slate-300 hover:bg-slate-600'
              }`}
            >
              {sub}
            </button>
          ))}
        </div>
      )}

      {/* Key Grid - Keyboard keycap style */}
      <div className="flex-1 overflow-y-auto">
        {isSearching ? (
          // Search Results View
          searchResults.length === 0 ? (
            <div className="p-4 text-center text-slate-400">
              <p className="mb-2">No results found for "{searchQuery}"</p>
              <p className="text-xs text-slate-500">Try different search terms like "ctrl", "enter", or "KC_A"</p>
            </div>
          ) : (
            <div className="space-y-2 p-4">
              {searchResults.map((result, idx) => {
                const match = result.matches[0]; // Primary match for highlighting
                const isSelected = idx === selectedSearchIndex;

                return (
                  <button
                    key={`search-${result.key.id}`}
                    onClick={() => {
                      handleKeySelect({
                        id: result.key.id,
                        label: result.key.label,
                        category: result.key.category,
                        subcategory: result.key.subcategory,
                        description: result.key.description,
                      });
                      setSearchQuery('');
                    }}
                    className={`
                      w-full text-left p-3 rounded-lg border transition-all
                      ${isSelected
                        ? 'border-primary-500 bg-primary-500/10 ring-2 ring-primary-500/50'
                        : 'border-slate-700 bg-slate-800 hover:border-slate-600 hover:bg-slate-750'
                      }
                    `}
                  >
                    <div className="flex items-start justify-between gap-3">
                      <div className="flex-1">
                        {/* Key label with highlighting */}
                        <div className="text-lg font-bold text-white font-mono mb-1">
                          {match.field === 'label'
                            ? highlightMatches(result.key.label, match.indices)
                            : result.key.label
                          }
                        </div>

                        {/* Key ID */}
                        <div className="text-xs text-slate-400 font-mono mb-1">
                          {match.field === 'id'
                            ? highlightMatches(result.key.id, match.indices)
                            : result.key.id
                          }
                        </div>

                        {/* Description */}
                        <div className="text-sm text-slate-300">
                          {match.field === 'description'
                            ? highlightMatches(result.key.description, match.indices)
                            : result.key.description
                          }
                        </div>

                        {/* Matched alias */}
                        {match.field === 'alias' && (
                          <div className="text-xs text-slate-500 mt-1">
                            Alias: {highlightMatches(match.text, match.indices)}
                          </div>
                        )}
                      </div>

                      {/* Category badge */}
                      <div className={`
                        px-2 py-1 text-xs rounded capitalize whitespace-nowrap
                        ${result.key.category === 'basic' ? 'bg-blue-500/20 text-blue-300' : ''}
                        ${result.key.category === 'modifiers' ? 'bg-cyan-500/20 text-cyan-300' : ''}
                        ${result.key.category === 'media' ? 'bg-pink-500/20 text-pink-300' : ''}
                        ${result.key.category === 'macro' ? 'bg-green-500/20 text-green-300' : ''}
                        ${result.key.category === 'layers' ? 'bg-yellow-500/20 text-yellow-300' : ''}
                        ${result.key.category === 'special' ? 'bg-purple-500/20 text-purple-300' : ''}
                      `}>
                        {result.key.category}
                      </div>
                    </div>
                  </button>
                );
              })}
            </div>
          )
        ) : activeCategory === 'any' ? (
          // Custom keycode input (Any category)
          <div className="p-6 space-y-6">
            <div>
              <h4 className="text-lg font-semibold text-slate-200 mb-2">Custom Keycode</h4>
              <p className="text-sm text-slate-400 mb-4">
                Enter any valid QMK-style keycode for advanced customization.
              </p>
            </div>

            {/* Input field with validation */}
            <div className="space-y-3">
              <div className="relative">
                <input
                  type="text"
                  value={customKeycode}
                  onChange={(e) => handleCustomKeycodeChange(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter' && customValidation.valid) {
                      handleApplyCustomKeycode();
                    }
                  }}
                  placeholder="e.g., KC_A, LCTL(KC_C), MO(1), LT(2,KC_SPC)"
                  className={`
                    w-full px-4 py-3 pr-10
                    bg-slate-800 border-2 rounded-lg
                    text-slate-100 font-mono text-base
                    placeholder-slate-500
                    focus:outline-none focus:ring-2
                    transition-colors
                    ${customValidation.valid
                      ? 'border-green-500 focus:border-green-400 focus:ring-green-500/50'
                      : customKeycode && !customValidation.valid
                      ? 'border-red-500 focus:border-red-400 focus:ring-red-500/50'
                      : 'border-slate-700 focus:border-primary-500 focus:ring-primary-500/50'
                    }
                  `}
                />
                {/* Validation icon */}
                <div className="absolute right-3 top-1/2 -translate-y-1/2">
                  {customValidation.valid ? (
                    <Check className="w-5 h-5 text-green-400" />
                  ) : customKeycode && !customValidation.valid ? (
                    <AlertCircle className="w-5 h-5 text-red-400" />
                  ) : null}
                </div>
              </div>

              {/* Validation message */}
              {customKeycode && (
                <div className={`text-sm ${customValidation.valid ? 'text-green-400' : 'text-red-400'}`}>
                  {customValidation.valid ? (
                    <div className="flex items-start gap-2">
                      <Check className="w-4 h-4 mt-0.5 flex-shrink-0" />
                      <div>
                        <p className="font-medium">Valid keycode</p>
                        <p className="text-slate-400 text-xs mt-0.5">
                          Will be mapped as: <span className="font-mono">{customValidation.label}</span>
                        </p>
                      </div>
                    </div>
                  ) : (
                    <div className="flex items-start gap-2">
                      <AlertCircle className="w-4 h-4 mt-0.5 flex-shrink-0" />
                      <p>{customValidation.error}</p>
                    </div>
                  )}
                </div>
              )}

              {/* Apply button */}
              <button
                onClick={handleApplyCustomKeycode}
                disabled={!customValidation.valid}
                className={`
                  w-full px-4 py-3 rounded-lg font-medium
                  transition-all
                  ${customValidation.valid
                    ? 'bg-primary-500 hover:bg-primary-600 text-white shadow-lg hover:shadow-xl'
                    : 'bg-slate-700 text-slate-500 cursor-not-allowed'
                  }
                `}
              >
                {customValidation.valid ? 'Apply Keycode' : 'Enter a valid keycode'}
              </button>
            </div>

            {/* Help section */}
            <div className="pt-6 border-t border-slate-700">
              <div className="flex items-start gap-2 mb-3">
                <HelpCircle className="w-4 h-4 text-slate-400 mt-0.5 flex-shrink-0" />
                <h5 className="text-sm font-semibold text-slate-300">Supported Syntax</h5>
              </div>
              <div className="space-y-3 text-sm text-slate-400">
                <div>
                  <p className="font-mono text-xs text-primary-400 mb-1">Simple Keys</p>
                  <p className="text-xs">
                    <span className="font-mono text-slate-300">A</span>,{' '}
                    <span className="font-mono text-slate-300">KC_A</span>,{' '}
                    <span className="font-mono text-slate-300">KC_ENTER</span>
                  </p>
                </div>
                <div>
                  <p className="font-mono text-xs text-primary-400 mb-1">Modifier Combinations</p>
                  <p className="text-xs">
                    <span className="font-mono text-slate-300">LCTL(KC_C)</span>,{' '}
                    <span className="font-mono text-slate-300">LSFT(A)</span>
                  </p>
                </div>
                <div>
                  <p className="font-mono text-xs text-primary-400 mb-1">Layer Functions</p>
                  <p className="text-xs">
                    <span className="font-mono text-slate-300">MO(1)</span> = Hold layer,{' '}
                    <span className="font-mono text-slate-300">TO(2)</span> = Switch to layer<br />
                    <span className="font-mono text-slate-300">TG(3)</span> = Toggle layer,{' '}
                    <span className="font-mono text-slate-300">OSL(4)</span> = One-shot layer
                  </p>
                </div>
                <div>
                  <p className="font-mono text-xs text-primary-400 mb-1">Layer-Tap</p>
                  <p className="text-xs">
                    <span className="font-mono text-slate-300">LT(2,KC_SPC)</span> = Hold for layer 2, tap for Space
                  </p>
                </div>
              </div>
            </div>
          </div>
        ) : activeKeys.length === 0 ? (
          <div className="p-4 text-center text-slate-400">
            <p>No keys in this category yet</p>
          </div>
        ) : (
          <div className={`bg-slate-800/50 rounded-lg ${
            compact ? 'p-2' : 'p-4'
          } ${
            viewMode === 'grid'
              ? compact ? 'grid grid-cols-12 gap-1' : 'grid grid-cols-8 gap-2'
              : 'flex flex-col gap-2'
          }`}>
            {activeKeys.map(key => renderKeyItem(key, () => handleKeySelect(key), !compact))}
          </div>
        )}
      </div>

      {/* Hint - hidden in compact mode */}
      {!compact && (
        <p className="text-xs text-slate-500 mt-4">
          {isSearching
            ? 'Use ↑↓ arrows to navigate, Enter to select, Esc to clear'
            : 'Search for keys or browse by category. Click to select.'
          }
        </p>
      )}

      {/* Key Capture Modal */}
      {isCapturingKey && (
      <div
        className="fixed inset-0 bg-black/70 backdrop-blur-sm flex items-center justify-center z-50"
        onClick={cancelKeyCapture}
      >
        <div
          className="bg-slate-800 border-2 border-primary-500 rounded-xl p-8 shadow-2xl max-w-md w-full mx-4"
          onClick={(e) => e.stopPropagation()}
        >
          {/* Waiting for key press */}
          {!capturedKey ? (
            <div className="text-center">
              <div className="mb-6">
                <Keyboard className="w-16 h-16 text-primary-400 mx-auto animate-pulse" />
              </div>
              <h3 className="text-2xl font-bold text-white mb-3">
                Press any key...
              </h3>
              <p className="text-slate-400 mb-6">
                Press the physical key you want to select.
              </p>
              <div className="text-xs text-slate-500">
                Press <kbd className="px-2 py-1 bg-slate-700 rounded border border-slate-600 font-mono">Esc</kbd> to cancel
              </div>
            </div>
          ) : (
            /* Key captured - show confirmation */
            <div className="text-center">
              <div className="mb-6">
                <div className="inline-block p-4 bg-green-500/20 rounded-full">
                  <Check className="w-12 h-12 text-green-400" />
                </div>
              </div>
              <h3 className="text-2xl font-bold text-white mb-3">
                Key Captured!
              </h3>
              <div className="mb-6 p-4 bg-slate-900/50 rounded-lg border border-slate-700">
                <div className="text-3xl font-bold text-white font-mono mb-2">
                  {capturedKey.label}
                </div>
                <div className="text-sm text-slate-400 font-mono mb-1">
                  {capturedKey.id}
                </div>
                <div className="text-xs text-slate-500">
                  {capturedKey.description}
                </div>
              </div>
              <p className="text-slate-400 mb-6">
                Use this key for mapping?
              </p>
              <div className="flex gap-3">
                <button
                  onClick={cancelKeyCapture}
                  className="flex-1 px-4 py-3 bg-slate-700 hover:bg-slate-600 text-slate-300 rounded-lg transition-colors font-medium"
                >
                  Cancel
                </button>
                <button
                  onClick={confirmCapturedKey}
                  className="flex-1 px-4 py-3 bg-primary-500 hover:bg-primary-600 text-white rounded-lg transition-colors font-medium"
                >
                  Use This Key
                </button>
              </div>
            </div>
          )}
        </div>
      </div>
    )}
    </Card>
  );
}
