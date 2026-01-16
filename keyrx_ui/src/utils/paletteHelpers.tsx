import React from 'react';
import { KEY_DEFINITIONS } from '../data/keyDefinitions';
import type { PaletteKey } from '../components/KeyPalette';
import {
  BASIC_KEYS,
  MODIFIER_KEYS,
  MEDIA_KEYS,
  MACRO_KEYS,
  LAYER_KEYS,
  SPECIAL_KEYS,
} from '../data/paletteKeys';

/**
 * LocalStorage keys for view mode persistence
 */
const STORAGE_KEY_VIEW_MODE = 'keyrx_palette_view_mode';

export type ViewMode = 'grid' | 'list';

/**
 * Validation result for custom keycodes
 */
export interface ValidationResult {
  valid: boolean;
  error?: string;
  normalizedId?: string;
  label?: string;
}

/**
 * Highlight matching characters in text
 */
export function highlightMatches(
  text: string,
  indices: number[]
): React.ReactNode {
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
        <mark
          key={i}
          className="bg-yellow-400/40 text-yellow-200 font-semibold"
        >
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

/**
 * Load view mode from localStorage with error handling
 */
export function loadViewMode(): ViewMode {
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
export function saveViewMode(mode: ViewMode): void {
  try {
    localStorage.setItem(STORAGE_KEY_VIEW_MODE, mode);
  } catch (err) {
    console.error(`Failed to save view mode to localStorage:`, err);
  }
}

/**
 * Find a key definition by ID
 */
export function findKeyById(keyId: string): PaletteKey | null {
  // Search in KEY_DEFINITIONS first
  const keyDef = KEY_DEFINITIONS.find((k) => k.id === keyId);
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
  const allKeys = [
    ...BASIC_KEYS,
    ...MODIFIER_KEYS,
    ...MEDIA_KEYS,
    ...MACRO_KEYS,
    ...LAYER_KEYS,
    ...SPECIAL_KEYS,
  ];
  return allKeys.find((k) => k.id === keyId) || null;
}

/**
 * Map DOM KeyboardEvent.code to our key ID
 * Uses KEY_DEFINITIONS aliases to find matching key
 */
export function mapDomCodeToKeyId(domCode: string): PaletteKey | null {
  // Normalize the DOM code - it might be like "KeyA", "Digit1", etc.
  // We need to map these to our KEY_ format

  // First try direct match with aliases
  const keyDef = KEY_DEFINITIONS.find((k) => k.aliases.includes(domCode));
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
  const normalizedKeyDef = KEY_DEFINITIONS.find((k) =>
    k.aliases.includes(normalizedCode)
  );
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
export function validateCustomKeycode(input: string): ValidationResult {
  const trimmed = input.trim();

  if (!trimmed) {
    return { valid: false, error: 'Please enter a keycode' };
  }

  // Check if it's a simple key ID (matches existing key)
  const keyDef = KEY_DEFINITIONS.find(
    (k) => k.id === trimmed || k.aliases.includes(trimmed)
  );

  if (keyDef) {
    return {
      valid: true,
      normalizedId: keyDef.id,
      label: keyDef.label,
    };
  }

  // Check for modifier combinations: LCTL(KC_C), LSFT(A), etc.
  const modifierPattern =
    /^(LCTL|RCTL|LSFT|RSFT|LALT|RALT|LMETA|RMETA)\(([A-Za-z0-9_]+)\)$/;
  const modMatch = trimmed.match(modifierPattern);
  if (modMatch) {
    const [, modifier, keyPart] = modMatch;
    // Validate inner key exists
    const innerKey = KEY_DEFINITIONS.find(
      (k) => k.id === keyPart || k.aliases.includes(keyPart)
    );

    if (!innerKey) {
      return {
        valid: false,
        error: `Unknown key: ${keyPart}. Try KC_A, KC_ENTER, etc.`,
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
        error: 'Layer number must be between 0-15',
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
        error: 'Layer number must be between 0-15',
      };
    }

    // Validate inner key exists
    const innerKey = KEY_DEFINITIONS.find(
      (k) => k.id === keyPart || k.aliases.includes(keyPart)
    );

    if (!innerKey) {
      return {
        valid: false,
        error: `Unknown key: ${keyPart}. Try KC_A, KC_ENTER, etc.`,
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
